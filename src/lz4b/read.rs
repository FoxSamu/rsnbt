use std::io;
use std::io::Read;

use super::block::LZ4Block;

/// An LZ4Block decoder.
/// 
/// LZ4Block is an unconventional compression format introduced in [lz4-java][https://github.com/yawkat/lz4-java].
/// It is used by Minecraft to store chunk data in MCA files. [`Lz4bDecoder`] is a decoder for this exact format.
pub struct LZ4BlockDecoder<R> where R : Read {
    reader: R,
    current_block: Option<LZ4Block>,
    pointer: usize,
    validate_checksums: bool
}

impl<R> LZ4BlockDecoder<R> where R : Read {
    pub fn new(reader: R, validate_checksums: bool) -> Self {
        Self { reader, current_block: None, pointer: 0, validate_checksums }
    }

    fn fetch<'a>(block_ref: &'a mut Option<LZ4Block>, pointer: &mut usize, reader: &mut R, validate_checksums: bool) -> io::Result<&'a LZ4Block> {
        let Some(block) = block_ref else {
            // Start of stream
            let mut b = LZ4Block::read(reader)?;
            b.decompress(validate_checksums)?;

            *block_ref = Some(b);
            *pointer = 0;

            return Ok(block_ref.as_ref().unwrap());
        };

        // If not end of block we are good to go
        if block.data.len() == 0 {
            return Ok(block)
        } else if *pointer < block.data.len() {
            return Ok(block)
        }

        // End of block: load more
        let mut b = LZ4Block::read(reader)?;
        b.decompress(validate_checksums)?;

        *block = b;
        *pointer = 0;
        Ok(block)
    }
}

impl<R> Read for LZ4BlockDecoder<R> where R : Read {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Self {
            reader,
            current_block,
            pointer,
            validate_checksums
        } = self;

        let block = Self::fetch(current_block, pointer, reader, *validate_checksums)?;

        if block.data.is_empty() {
            return Ok(0);
        }

        let len = block.data.len();
        let pos = *pointer;
        let rem = len - pos;

        let read = rem.min(buf.len());

        buf[0..read].copy_from_slice(&block.data[pos..(read + pos)]);

        *pointer += read;

        Ok(read)
    }
}
