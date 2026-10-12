use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::copy;
use std::io::sink;

use flate2::read::DeflateDecoder;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;

use crate::io::Decompression;
use crate::tag::TagId;
use crate::tag::TagList;
use crate::tag::TagRef;

/// Measured statistics of an NBT tag.
///
/// ```
/// let tag = Tag::Int(3);
/// let name = String::from("foo");
///
/// let stats = NbtStats::measure(&name, &tag);
/// assert_eq!(10, stats.bytes);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NbtStats {
    /// The amount of bytes the measured tag would be wen saved as uncompressed NBT.
    pub bytes: usize,

    /// The deepest amount of nesting in the measured tag.
    pub depth: usize,

    /// The number of **bytes** of the longest string in the measured tag. Includes both tag names and
    /// string values.
    pub longest_string: usize,

    /// The number of **bytes** of the longest byte array in the measured tag.
    pub longest_byte_array: usize,

    /// The number of **integers** of the longest int array in the measured tag.
    /// Note that this means longest as in having the most elements, not in bytes.
    pub longest_int_array: usize,

    /// The number of **longs** of the longest long array in the measured tag.
    /// Note that this means longest as in having the most elements, not in bytes.
    pub longest_long_array: usize,

    /// The number of **elements** of the longest list tag in the measured tag.
    /// Note that this means longest as in having the most elements, not in bytes.
    pub longest_list: usize,

    /// The number of **elements** of the longest compound tag in the measured tag.
    /// Note that this means longest as in having the most elements, not in bytes.
    pub longest_compound: usize,
}

impl NbtStats {
    pub fn measure<'a, T>(name: &String, tag: T) -> NbtStats
    where T: Into<TagRef<'a>> {
        let mut stats = NbtStats::default();
        stats.measure_tag(name, tag.into(), 0);
        stats
    }

    fn measure_tag(&mut self, name: &String, tag: TagRef, depth: usize) {
        self.depth = self.depth.max(depth);

        // Tag ID
        self.bytes += 1;

        // Name
        let name_length = name.len();
        self.longest_string = self.longest_string.max(name_length);
        self.bytes += 2 + name_length;

        // Payload
        self.measure_payload(tag, depth);
    }

    fn measure_payload(&mut self, tag: TagRef, depth: usize) {
        match tag {
            // Simple constant payloads
            TagRef::Byte(_) => self.bytes += 1,
            TagRef::Short(_) => self.bytes += 2,
            TagRef::Int(_) => self.bytes += 4,
            TagRef::Long(_) => self.bytes += 8,
            TagRef::Float(_) => self.bytes += 4,
            TagRef::Double(_) => self.bytes += 8,

            TagRef::String(content) => {
                let content_length = content.len();
                self.longest_string = self.longest_string.max(content_length);
                self.bytes += 2 + content_length; // length + content
            },

            TagRef::ByteArray(content) => {
                let content_length = content.len();
                self.longest_byte_array = self.longest_byte_array.max(content_length);
                self.bytes += 4 + content_length; // length + content
            },

            TagRef::IntArray(content) => {
                let content_length = content.len();
                self.longest_int_array = self.longest_long_array.max(content_length);
                self.bytes += 4 + content_length * 4; // length + content
            },

            TagRef::LongArray(content) => {
                let content_length = content.len();
                self.longest_byte_array = self.longest_byte_array.max(content_length);
                self.bytes += 4 + content_length * 8; // length + content
            },

            // Lists of fixed-size primitives can be easily computed
            TagRef::List(TagList::Empty) => {
                self.depth = self.depth.max(depth);

                self.bytes += 1 + 4; // type + length
            },

            TagRef::List(TagList::Bytes(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length; // type + length + content
            },

            TagRef::List(TagList::Shorts(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length * 2; // type + length + content
            },

            TagRef::List(TagList::Ints(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length * 4; // type + length + content
            },

            TagRef::List(TagList::Longs(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length * 8; // type + length + content
            },

            TagRef::List(TagList::Floats(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length * 4; // type + length + content
            },

            TagRef::List(TagList::Doubles(content)) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4 + content_length * 8; // type + length + content
            },

            TagRef::List(content) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_list = self.longest_list.max(content_length);
                self.bytes += 1 + 4; // type + length

                // content
                for elem in content.iter() {
                    self.measure_payload(elem, depth + 1);
                }
            },

            TagRef::Compound(content) => {
                self.depth = self.depth.max(depth);

                let content_length = content.len();
                self.longest_compound = self.longest_compound.max(content_length);

                // contents
                for (name, elem) in content.iter() {
                    self.measure_tag(name, elem.into(), depth + 1);
                }

                // end tag
                self.bytes += 1;
            },
        }
    }
}

enum Reader<R1, R2>
where
    R1: Read,
    R2: Read + Seek, {
    Unseekable(R1),
    Seekable(R2),
}

impl<R1, R2> Reader<R1, R2>
where
    R1: Read,
    R2: Read + Seek,
{
    fn read<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut bytes = [0u8; N];

        match self {
            Reader::Unseekable(r) => r.read_exact(&mut bytes)?,
            Reader::Seekable(r) => r.read_exact(&mut bytes)?,
        };

        Ok(bytes)
    }

    fn skip(&mut self, n: i64) -> Result<()> {
        match self {
            Reader::Unseekable(r) => {
                copy(&mut r.by_ref().take(n as u64), &mut sink())?;
            },
            Reader::Seekable(r) => {
                r.seek_relative(n as i64)?;
            },
        };

        Ok(())
    }
}

struct DummyStream;

impl Read for DummyStream {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        unreachable!() // Never actually used
    }
}

impl Seek for DummyStream {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> Result<u64> {
        unreachable!() // Never actually used
    }
}

impl NbtStats {
    pub fn measure_input<R>(reader: R, compression: Decompression, be: bool) -> Result<NbtStats>
    where R: Read {
        let mut stats = NbtStats::default();

        match compression {
            Decompression::Uncompressed => {
                stats.measure_tag_input(&mut Reader::<_, DummyStream>::Unseekable(reader), 0, be)?
            },
            Decompression::GZip => {
                stats.measure_tag_input(&mut Reader::<_, DummyStream>::Unseekable(GzDecoder::new(reader)), 0, be)?
            },
            Decompression::ZLib => stats.measure_tag_input(
                &mut Reader::<_, DummyStream>::Unseekable(ZlibDecoder::new(reader)),
                0,
                be,
            )?,
            Decompression::Deflate => stats.measure_tag_input(
                &mut Reader::<_, DummyStream>::Unseekable(DeflateDecoder::new(reader)),
                0,
                be,
            )?,
        };

        Ok(stats)
    }

    pub fn measure_input_seek<R>(reader: R, compression: Decompression, be: bool) -> Result<NbtStats>
    where R: Read + Seek {
        let mut stats = NbtStats::default();

        match compression {
            Decompression::Uncompressed => {
                stats.measure_tag_input(&mut Reader::<DummyStream, _>::Seekable(reader), 0, be)?
            },
            Decompression::GZip => {
                stats.measure_tag_input(&mut Reader::<_, DummyStream>::Unseekable(GzDecoder::new(reader)), 0, be)?
            },
            Decompression::ZLib => stats.measure_tag_input(
                &mut Reader::<_, DummyStream>::Unseekable(ZlibDecoder::new(reader)),
                0,
                be,
            )?,
            Decompression::Deflate => stats.measure_tag_input(
                &mut Reader::<_, DummyStream>::Unseekable(DeflateDecoder::new(reader)),
                0,
                be,
            )?,
        };

        Ok(stats)
    }

    pub fn measure_blob(blob: &[u8], compression: Decompression, be: bool) -> Result<NbtStats> {
        Self::measure_input_seek(Cursor::new(blob), compression, be)
    }

    fn read_tag_id<R1, R2>(&mut self, reader: &mut Reader<R1, R2>) -> Result<Option<TagId>>
    where
        R1: Read,
        R2: Read + Seek, {
        let byte = reader.read::<1>()?;
        self.bytes += 1;

        if byte[0] == 0 {
            return Ok(None);
        }

        match TagId::from_byte(byte[0]) {
            None => Err(Error::new(ErrorKind::InvalidData, "Invalid tag ID found")),
            s => Ok(s),
        }
    }

    fn skip_and_count_bytes<R1, R2>(&mut self, reader: &mut Reader<R1, R2>, bytes: i64) -> Result<()>
    where
        R1: Read,
        R2: Read + Seek, {
        reader.skip(bytes)?;
        self.bytes += bytes as usize;

        Ok(())
    }

    fn measure_string_input<R1, R2>(&mut self, reader: &mut Reader<R1, R2>, be: bool) -> Result<()>
    where
        R1: Read,
        R2: Read + Seek, {
        let len_bytes = reader.read::<2>()?;
        self.bytes += 2;

        let len = if be { u16::from_be_bytes(len_bytes) } else { u16::from_le_bytes(len_bytes) };

        self.bytes += len as usize;
        self.longest_string = self.longest_string.max(len as usize);

        Ok(())
    }

    fn get_length<R1, R2>(&mut self, reader: &mut Reader<R1, R2>, be: bool) -> Result<i32>
    where
        R1: Read,
        R2: Read + Seek, {
        let len_bytes = reader.read::<4>()?;
        self.bytes += 4;

        let len = if be { i32::from_be_bytes(len_bytes) } else { i32::from_le_bytes(len_bytes) };

        Ok(len)
    }

    fn measure_tag_input<R1, R2>(&mut self, reader: &mut Reader<R1, R2>, depth: usize, be: bool) -> Result<bool>
    where
        R1: Read,
        R2: Read + Seek, {
        let id = match self.read_tag_id(reader)? {
            None => return Ok(false),
            Some(id) => id,
        };

        self.measure_string_input(reader, be)?;
        self.measure_payload_input(reader, depth, be, id)?;

        Ok(true)
    }

    fn measure_payload_input<R1, R2>(
        &mut self,
        reader: &mut Reader<R1, R2>,
        depth: usize,
        be: bool,
        id: TagId,
    ) -> Result<()>
    where
        R1: Read,
        R2: Read + Seek,
    {
        match id {
            TagId::Byte => self.skip_and_count_bytes(reader, 1)?,
            TagId::Short => self.skip_and_count_bytes(reader, 2)?,
            TagId::Int => self.skip_and_count_bytes(reader, 4)?,
            TagId::Long => self.skip_and_count_bytes(reader, 8)?,
            TagId::Float => self.skip_and_count_bytes(reader, 4)?,
            TagId::Double => self.skip_and_count_bytes(reader, 8)?,
            TagId::String => self.measure_string_input(reader, be)?,

            TagId::ByteArray => {
                let len = self.get_length(reader, be)?;
                self.skip_and_count_bytes(reader, len as i64)?;

                self.longest_byte_array = self.longest_byte_array.max(len as usize);
            },

            TagId::IntArray => {
                let len = self.get_length(reader, be)?;
                self.skip_and_count_bytes(reader, (len as i64) * 4)?;

                self.longest_int_array = self.longest_int_array.max(len as usize);
            },

            TagId::LongArray => {
                let len = self.get_length(reader, be)?;
                self.skip_and_count_bytes(reader, (len as i64) * 8)?;

                self.longest_long_array = self.longest_long_array.max(len as usize);
            },

            TagId::List => {
                self.depth = self.depth.max(depth);

                let id = self.read_tag_id(reader)?;
                let len = self.get_length(reader, be)?;

                match id {
                    None => {},
                    Some(TagId::Byte) => self.skip_and_count_bytes(reader, len as i64)?,
                    Some(TagId::Short) => self.skip_and_count_bytes(reader, (len as i64) * 2)?,
                    Some(TagId::Int) => self.skip_and_count_bytes(reader, (len as i64) * 4)?,
                    Some(TagId::Long) => self.skip_and_count_bytes(reader, (len as i64) * 8)?,
                    Some(TagId::Float) => self.skip_and_count_bytes(reader, (len as i64) * 4)?,
                    Some(TagId::Double) => self.skip_and_count_bytes(reader, (len as i64) * 8)?,

                    Some(id) => {
                        for _ in 0..len {
                            self.measure_payload_input(reader, depth + 1, be, id)?;
                        }
                    },
                };

                self.longest_list = self.longest_list.max(len as usize);
            },

            TagId::Compound => {
                self.depth = self.depth.max(depth);

                let mut len = 0usize;

                while self.measure_tag_input(reader, depth + 1, be)? {
                    len += 1;
                }

                self.longest_compound = self.longest_compound.max(len);
            },
        };

        Ok(())
    }
}
