use std::io;
use std::io::Write;

use super::block::LZ4Block;

const MIN_BLOCK_SIZE: usize = 64;
const MAX_BLOCK_SIZE: usize = 1 << (10 + 0x0F);

/// An LZ4Block encoder. **Be sure to call [`finish`] upon completion!**
/// 
/// LZ4Block is an unconventional compression format introduced in [lz4-java][https://github.com/yawkat/lz4-java].
/// It is used by Minecraft to store chunk data in MCA files. [`LZ4BlockEncoder`] is an encoder for this exact format.
/// 
/// LZ4Block splits the data in blocks of a certain size and uses LZ4 to compress each block individually. Each block
/// contains a header detailing all the information needed to decode the block. This means that, while usually each
/// block in a stream has the same decompressed length, each block can actually be encoded with its own length and
/// compression mode, and the decoder will be able to decode each of them without having to know anything about the
/// encoder that encoded them. At the end of the stream, one final block header is written that indicates a block of
/// zero bytes. This indicates the end of the stream.
/// 
/// [`LZ4BlockEncoder`] implements the [`Write`] trait, and will write to the underlying writer block by block. Each
/// block will be written to the underlying stream once it reaches the configured block size.
#[derive(Debug)]
pub struct LZ4BlockEncoder<W> where W : Write {
    // writer is an Option so that we can take ownership whenever needed
    // every method that takes ownership will put it back, except for finish (or drop)
    writer: Option<W>,

    block_size: usize,
    compression: LZ4BlockCompression,
    level: u8,

    block: LZ4Block
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LZ4BlockCompression {
    None,
    Fast(i32),
    High(i32),
}

impl<W> LZ4BlockEncoder<W> where W : Write {
    pub fn new(writer: W, block_size: usize, compression: LZ4BlockCompression) -> Self {
        if !(MIN_BLOCK_SIZE <= block_size && block_size <= MAX_BLOCK_SIZE) {
            panic!("Invalid block size");
        }

        let log = 32 - (block_size as i32 - 1).leading_zeros() as i32;
        let level = (0.max(log - 10) + 10) as u8;

        Self {
            block_size,
            compression,
            level,
            writer: Some(writer),
            block: LZ4Block::new()
        }
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn compression(&self) -> LZ4BlockCompression {
        self.compression
    }

    pub fn flush_and_update(&mut self, block_size: usize, compression: LZ4BlockCompression) -> io::Result<()> {
        if !(MIN_BLOCK_SIZE <= block_size && block_size <= MAX_BLOCK_SIZE) {
            panic!("Invalid block size");
        }

        self.flush()?;

        self.block_size = block_size;
        self.compression = compression;

        let log = 32 - (block_size as i32 - 1).leading_zeros() as i32;
        let level = (0.max(log - 10) + 10) as u8;
        self.level = level;

        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.writer.is_none()
    }

    pub fn inner(&self) -> &W {
        self.writer.as_ref().expect("Stream finished")
    }

    pub fn inner_mut(&mut self) -> &mut W {
        self.writer.as_mut().expect("Stream finished")
    }

    pub fn into_inner(mut self) -> Result<W, IntoInnerError<LZ4BlockEncoder<W>>> {
        self.flush()
            .map(|_| self.writer.take().unwrap())
            .map_err(|e| IntoInnerError(self, e))
    }

    /// Finalizes the LZ4Block stream with an empty block and returns back the underlying stream.
    /// You very likely want to call this or [`finish`][LZ4BlockEncoder::finish] once you're done writing
    /// data, to make sure the stream is completed.
    /// 
    /// The encoder will lose ownership over the writer, and will be left in an unusable finished state.
    /// Attempting to use the encoder in any way in this state will cause it to panic, as it no longer
    /// has a stream to write to. This method solely exists for cases where the caller does not own the
    /// encoder but is tasked with finishing the stream nonetheless. Use [`finish`][LZ4BlockEncoder::finish]
    /// wherever possible.
    /// 
    /// [`LZ4BlockEncoder`] will attempt to finish the stream if it is dropped while unfinished, but if this
    /// fails, the error will be completely ignored and it will not make another attempt. If the stream needs to
    /// be deliberately left unfinished, call [`into_inner`][LZ4BlockEncoder::into_inner].
    /// 
    /// If finalization fails due to whatever I/O error, then the encoder will not be finished and further data
    /// can be written. However, it may have partly written some data, so this may only further corrupt the data
    /// stream.
    pub fn try_finish(&mut self) -> io::Result<W> {
        let mut writer = self.writer.as_mut().expect("Stream finished");

        // Write remaining data
        self.block.compress(self.level, self.compression)?;
        self.block.write(&mut writer)?;
        self.block.data.clear();
        self.block.compressed = false;

        // write empty block to indicate end
        self.block.compress(self.level, self.compression)?;
        self.block.write(&mut writer)?;
        writer.flush()?;

        // take ownership of writer and leave None behind
        Ok(self.writer.take().unwrap())
    }

    /// Finalizes the LZ4Block stream with an empty block and returns back the underlying stream.
    /// You very likely want to call this method once you're done streaming data, to make sure the
    /// stream is completed.
    /// 
    /// When finishing the stream, prefer this method over [`try_finish`][LZ4BlockEncoder::try_finish] whenever
    /// possible, since this method consumes and drops the encoder, leaving no possibility for any code to
    /// erroneously use the encoder after it is finished.
    /// 
    /// [`LZ4BlockEncoder`] will attempt to finish the stream if it is dropped while unfinished, but if this
    /// fails, the error will be completely ignored and it will not make another attempt. If the stream needs to
    /// be deliberately left unfinished, call [`into_inner`][LZ4BlockEncoder::into_inner].
    /// 
    /// If finalization fails due to whatever I/O error, then the encoder will not be finished, and it will,
    /// in a similar fashion as [`into_inner`][LZ4BlockEncoder::into_inner], return an [`IntoInnerError`]
    /// containing the error and this encoder for further use. Despite not being finished upon an error,
    /// it may have partly written some data, so using the encoder after it failed to finalize may only
    /// further corrupt the data stream.
    pub fn finish(mut self) -> Result<W, IntoInnerError<LZ4BlockEncoder<W>>> {
        self.try_finish().map_err(|e| IntoInnerError(self, e))
    }
}

impl<W> Write for LZ4BlockEncoder<W> where W : Write {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // If there is no writer, fail immediately
        self.writer.as_ref().expect("Stream finished");
        
        let len = buf.len();
        let rem = self.block_size as usize - self.block.data.len();

        let write = len.min(rem);

        self.block.data.extend_from_slice(&buf[0..write]);

        if self.block.data.len() == self.block_size {
            self.flush()?;
        }

        Ok(write)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut writer = self.writer.as_mut().expect("Stream finished");

        self.block.compress(self.level, self.compression)?;
        self.block.write(&mut writer)?;
        self.block.data.clear();
        self.block.compressed = false;

        writer.flush()?;

        Ok(())
    }
}

impl<W> Drop for LZ4BlockEncoder<W> where W : Write {
    fn drop(&mut self) {
        if let Some(_) = self.writer {
            // user abandoned me unfinished...
            //
            // let me take a final effort before I rest,
            // in the hope that it succeeds
            let _result = self.try_finish();
        }
    }
}


#[derive(Debug)]
pub struct IntoInnerError<W>(W, io::Error);

impl<W> IntoInnerError<W> {
    pub fn error(&self) -> &io::Error {
        &self.1
    }

    pub fn into_inner(self) -> W {
        self.0
    }

    pub fn into_error(self) -> io::Error {
        self.1
    }

    pub fn into_parts(self) -> (io::Error, W) {
        (self.1, self.0)
    }
}

impl<W> From<IntoInnerError<W>> for io::Error {
    fn from(it: IntoInnerError<W>) -> io::Error {
        it.1
    }
}
