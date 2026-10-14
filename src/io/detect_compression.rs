use std::io;
use std::io::Read;

use crate::io::Decompression::*;
use crate::io::Decompression;
use crate::tag::TagId;
use crate::util::ring_buf::RingBuf;


/// Given the first 4 bytes of the input, detects the mode in which the input NBT data was compressed.
/// It returns a [`Vec`] containing possible [decompression modes][Decompression].
///
/// Note that it isn't possible to boil down the compression format to just one compression format from
/// only the first four bytes. This function will return all possible compression modes in order of
/// likelyhood.
///
/// # Why 4 bytes?
/// Any NBT file at any supported compression must be at least 4 bytes long. Let's see why.
/// 
/// The shortest uncompressed NBT tag is 4 bytes. Every tag consists of a 3+ bytes header: a 1-byte tag
/// ID, a 2-byte name length and, depending on said length, 0 or more name bytes. Then follows a payload,
/// which must be at least 1 byte:
/// - any fixed-size tag (byte, short, int, long, float, double) must have at least 1 byte;
/// - array tags begin with a 4-byte length, so these are at least 4 bytes;
/// - list tags begin with a type ID and a 4-byte length, so these are at least 4 bytes;
/// - string tags begin with a 2-byte length, so these are at least 2 bytes;
/// - compound tags must end with an end byte, so these are at least 1 byte.
/// 
/// The last of these is actually quite likely, since NBT data exported by Minecraft is very often a
/// compound tag with a zero-length name at root level, and this tag being empty is a possibility. Overall
/// the tag ID, name and minimum payload combined result in NBT tags being at least 4 bytes.
/// 
/// If we use compression, we have a few options.
/// 
/// - **GZip**: GZip starts with a header that is longer than 4 bytes. Case closed.
/// 
/// - **ZLib**: ZLib starts with 2 bytes of header and ends with 4 bytes of trailer, thus being at least 6
///   bytes, ZLib data must be longer than 4 bytes.
/// 
/// - **LZ4**: LZ4 commonly starts with a magic number `0x04224D18`, or `0x02214C18` in older LZ4 data,
///   but it doesn't necessarily have to have this magic number. A common LZ4 implementation Minecraft
///   uses instead uses the ASCII string `LZ4Block`. All of these magic values are longer than 4 bytes,
///   but are all optional.
/// 
///   If there is no LZ4 magic number, then we have raw data. The longest possible LZ4 data that fits
///   less than 4 bytes is the sequence `20 XX YY` where `XX YY` are two arbitrary bytes. This encodes
///   exactly the bytes `XX YY`, which is too short for NBT data. LZ4 thus needs to be 4 bytes or longer.
/// 
/// - **DEFLATE**: DEFLATE has 3 types of blocks, uncompressed, compressed with a fixed Huffman tree
///   or compressed with a dynamic Huffman tree. The uncompressed data contains a 2 byte length and a
///   2 byte complement of that length, thus being over 4 bytes. The dynamic Huffman tree compression
///   includes data to reconstruct the Huffman tree, which is longer than 4 bytes.
///   This leaves the fixed Huffman coding. Fixed Huffman blocks can be less than 4 bytes but can only
///   contain zero or one byte of data that way. For them to store valid NBT, they must be longer than
///   4 bytes.
pub fn detect_compression<R>(first_bytes: &[u8; 4]) -> Vec<Decompression> where R : Read {
    match first_bytes {
        // GZip starts with two magic bytes: 1F 8B
        // It then follows with a compression type byte which must be 08
        //
        // We can be 100% sure it's GZip:
        // - Cannot be raw since the first byte is not a tag ID
        // - Cannot be deflate since the compression mode of the first block is 0b11
        // - Cannot be raw LZ4 since an offset of 8 cannot exist in 1 byte
        // - Cannot be ZLib as 0x1F*256 + 0x8B is not a multiple of 31
        // - Cannot be LZ4 block as 1F != 'L'
        // - Cannot be LZ4 frame as 1F != 04 and 1F != 02
        &[0x1F, 0x8B, 0x08, _] => vec![GZip],

        // LZ4Block format, starts with "LZ4Block" in ASCII
        // Might also be deflate raw data
        // Cannot be a tag since 'L' does not identify a tag type
        &[b'L', b'Z', b'4', b'B'] => vec![LZ4Block, Deflate],

        // Default LZ4 frame format, starts with 04 22 4D 18
        // Might also be deflate raw data or an uncompressed Long tag (with an awkwardly long name)
        &[0x04, 0x22, 0x4D, 0x18] => vec![LZ4Frame, Deflate, Uncompressed],

        // Legacy LZ4 frame format, starts with 02 21 4C 18
        // Might also be deflate raw data or an uncompressed Short tag (with an awkwardly long name)
        &[0x02, 0x21, 0x4C, 0x18] => vec![LZ4Frame, Deflate, Uncompressed],
        
        &[x, y, z, w] => {
            // We are out of magic numbers here, we need a more heuristic approach
            // Let's try to give each possibility a score

            let mut uncompressed = 0;
            let mut zlib = 0;
            let mut deflate = 0;
            let mut lz4 = 0;


            // UNCOMPRESSED

            if x > 0 && x <= TagId::MAX {
                // This is a valid tag header
                uncompressed += 4;

                // Both of the length bytes are zero, which means
                // the root tag has an empty name. This is very common
                // in Minecraft.
                if y == 0 && z == 0 {
                    uncompressed += 40;
                }

                // One of the length bytes is zero, which,
                // in the right endianness, means the length
                // of the name is less than 256, which is the most
                // likely
                else if y == 0 || z == 0 {
                    uncompressed += 10;
                }

                if x == TagId::COMPOUND {
                    // Tag is a compound tag, which is by far the most
                    // commonly used root tag by Minecraft
                    uncompressed <<= 2;
                } else if x != TagId::LIST {
                    // Tag is not a list tag, list tags are the second
                    // most common
                    //
                    // Also, any tag other than compound or list is usually
                    // not accepted by Minecraft so this is unlikely
                    uncompressed >>= 2;
                }
            }



            // DEFLATE

            let deflate_mode = x & 0b110;

            if deflate_mode == 0b000 {
                // Uncompressed deflate block

                // The remaining bits in this byte are meaningless, and most deflate encoders
                // will set these to 0, so if they are not 0 the likelyhood of deflate
                // decreases.

                if x & !1 == 0 {
                    deflate += 7;
                }

                deflate += 2;
            }

            if deflate_mode == 0b010 {
                // Deflate block with predefined huffman tree
                // This is unlikely with binary data
                deflate += 3;
            }

            if deflate_mode == 0b100 {
                // Deflate block with embedded huffman tree
                // This is most likely with deflate
                deflate += 25;
            }



            // ZLIB

            // ZLib starts with two header bytes X Y, but the header
            // bytes contain a checksum such that (x*256 + y) % 31 = 0
            if zlib_checksum(x, y) {
                zlib += 100;
            }



            // LZ4
            let l = x >> 4;

            lz4 += match (l, y, z, w) {
                // Invalid LZ4
                (0, ..) => 0,

                // `lit` is the first byte of the compressed data
                // hiding in plain sight, this should be a tag ID
                (1, lit, 0x01, 0x00)  | (2, lit, _, 1 | 2) => match lit {
                    TagId::COMPOUND => 38,
                    TagId::LIST => 15,
                    1..=TagId::MAX => 5,

                    _ => 0 // if this is NBT data then it can't be LZ4
                },

                (_, xx, yy, zz) => {
                    // We can see the first 3 uncompressed bits, let's do the same
                    // likelyhood check as before
                    let mut score = 0;

                    if xx > 0 && xx <= TagId::MAX {
                        // This is a valid tag header
                        score += 4;

                        // Zero length name
                        if yy == 0 && zz == 0 {
                            score += 30;
                        }

                        // Short length name
                        else if yy == 0 || zz == 0 {
                            score += 15;
                        }

                        if xx == TagId::COMPOUND {
                            // Likely compound tag
                            score <<= 2;
                        } else if xx != TagId::LIST {
                            // Unlikely tag
                            score >>= 2;
                        }
                    }

                    score
                },
            };


            let mut scoreboard = vec![
                (Uncompressed, uncompressed),
                (Deflate, deflate),
                (LZ4Raw, lz4),
                (ZLib, zlib)
            ];

            scoreboard.sort_by_key(|v| v.1);
            scoreboard.retain(|v| v.1 > 0);

            scoreboard.into_iter().map(|v| v.0).collect()
        }
    }
}

fn zlib_checksum(mut x: u8, mut y: u8) -> bool {
    // (x*256 + y) % 31 = 0
    //
    // if we distribute the modulo:
    // (((x%31) * (256%31)) % 31 + y%31) % 31 = 0
    //
    // we have 256%31 = 8:
    // (((x%31) * 8) % 31 + y%31) % 31 = 0

    x = (x % 31) * 8;
    x %= 31;
    y %= 31;

    if (x + y) % 31 != 0 {
        return false;
    }

    // Extra check: the 4 LSBs of x must be 0b1000
    return (x & 0b1111) == 0b1000;
}


pub struct LookaheadReader<R> where R : Read {
    init: bool,
    read: R,
    la: RingBuf<u8>,
    end: bool,
}

impl<R> LookaheadReader<R> where R : Read {
    pub fn new(read: R, la_size: usize) -> Self {
        Self {
            init: false,
            read,
            la: RingBuf::new(la_size),
            end: false,
        }
    }

    pub fn la_size(&self) -> usize {
        self.la.capacity()
    }

    pub fn la_len(&mut self) -> io::Result<usize> {
        self.init()?;
        return Ok(self.la.len());
    }

    pub fn la(&mut self) -> io::Result<&[u8]> {
        self.init()?;
        return Ok(self.la.as_slice());
    }

    pub fn read_one(&mut self) -> io::Result<Option<u8>> {
        self.init()?;

        let mut arr = [0u8];
        let n = self.read.read(&mut arr)?;

        let result = if n == 0 {
            self.end = true;
            self.la.poll()
        } else {
            self.la.put(arr[0])
        };

        Ok(result)
    }

    fn init(&mut self) -> io::Result<()> {
        if self.init {
            return Ok(());
        }

        let mut bytes_to_read = self.la_size();
        let mut vec = Vec::<u8>::with_capacity(bytes_to_read);
        let slice = vec.as_mut_slice();

        while bytes_to_read > 0 {
            let n = self.read.read(slice)?;
            if n == 0 {
                self.end = true;
                return Ok(());
            }

            bytes_to_read -= n;

            // Put bytes in ring buf
            // TODO maybe add bulk add to ring buf for more performance
            for i in 0..n {
                self.la.put(slice[i]);
            }
        }

        self.init = true;

        Ok(())
    }
}

impl<R> Read for LookaheadReader<R> where R : Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // TODO this can probably be done more efficiently but this works for now
        let mut r = 0usize;

        for i in 0..buf.len() {
            let Some(v) = self.read_one()? else {
                break;
            };

            buf[i] = v;
            r += 1;
        }

        Ok(r)
    }
}