//! Small utility module that handles encoding and decoding of LZ4Block data streams.
//! 
//! # What is LZ4Block
//! Minecraft uses the [lz4-java][https://github.com/yawkat/lz4-java/] library for LZ4 compression.
//! This library ships a custom container for LZ4-encoded data that is easily streamed, streaming
//! data in LZ4 blocks with each block containing all necessary data to decompress a block.
//!
//! Minecraft happens to use this container format sometimes to store chunk data in MCA files. This
//! means we need a tool to decode LZ4Block data so that we can properly load and store data from
//! chunks. This module provides just that.
//! 
//! # Format
//! LZ4Block cuts data into (usually) equally sized blocks. Each block is compressed individually
//! and accompanied by a 21-byte header.
//! 
//! ## Header
//! The header of a block consists of 21 bytes:
//! - an 8 byte magic bitstring: `LZ4Block`` in ASCII;
//! - one byte indicating the block format;
//! - a 4 byte little-endian integer indicating the compressed length of the block data;
//! - a 4 byte little-endian integer indicating the decompressed length of the block data;
//! - a 4 byte little-endian checksum of the decompressed data.
//! 
//! The block format contains two nibbles. The high half of the byte specifies the compression mode: 1
//! means no compression and 2 means LZ4 compression. Anything else is invalid. The low half of the
//! byte specifies a level value, such that the decompressed length is no more than `1 << length`.
//! 
//! The checksum is the `xxHash32` of the uncompressed data, using `0x9747B28C` as seed. The checksum's
//! four most significant bits are discarded and set to zero. That is, the checksum is
//! `xxh32 & 0x0FFFFFFF`.
//! 
//! ## Data
//! The data that follows the header will have exactly as many bytes as specified in the block
//! header. If the compression nibble is 2, then this data is raw LZ4 data, which can be decompressed
//! with any well-known LZ4 library. If the compression nibble is 1, this data is not compressed and
//! doesn't need decoding.
//! 
//! ## End
//! The end of the stream is marked by a block header whose decompressed length is 0. While this is
//! the only requirement for a decoder to detect the end of a stream, the default encoding strategy
//! sets the compressed length and checksum to 0 as well. The compression mode is usually set to 
//! uncompressed and the block level is completely arbitrary.

mod block;
mod read;
mod write;

pub use read::*;
pub use write::*;
