use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

use lz4::block::CompressionMode;

use super::write::LZ4BlockCompression;

const MAGIC: [u8; 8] = *b"LZ4Block";
const HEADER_LENGTH: usize = 8 + 1 + 4+4+4;

const NO_COMPRESSION: u8 = 1;
const COMPRESSION: u8 = 2;

const CHECKSUM_SEED: u32 = 0x9747B28C;
const CHECKSUM_MASK: u32 = 0x0FFFFFFF;

#[derive(Debug)]
pub struct LZ4BlockHeader {
    pub magic: [u8; 8],
    pub method: u8,
    pub level: u8,
    pub compressed_len: i32,
    pub decompressed_len: i32,
    pub checksum: u32
}

#[derive(Debug)]
pub struct LZ4Block {
    header: LZ4BlockHeader,
    pub data: Vec<u8>,
    pub compressed: bool,
}

impl LZ4BlockHeader {
    pub fn read<R>(mut reader: R) -> io::Result<Self> where R : Read {
        let mut row = [0u8; HEADER_LENGTH];
        reader.read_exact(&mut row)?;

        let token = row[8];
        let level = (token & 0xF) + 10;
        let method = token >> 4;

        Ok(Self { 
            magic: *row[0..8].as_array().unwrap(),
            method,
            level,
            compressed_len: i32::from_le_bytes(*row[9..13].as_array().unwrap()),
            decompressed_len: i32::from_le_bytes(*row[13..17].as_array().unwrap()),
            checksum: u32::from_le_bytes(*row[17..21].as_array().unwrap())
        })
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()> where W : Write {
        let mut row = [0u8; HEADER_LENGTH];

        row[0..8].clone_from_slice(&self.magic);
        row[8] = (self.method << 4) | (self.level - 10);
        row[9..13].clone_from_slice(&self.compressed_len.to_le_bytes());
        row[13..17].clone_from_slice(&self.decompressed_len.to_le_bytes());
        row[17..21].clone_from_slice(&self.checksum.to_le_bytes());

        writer.write_all(&row)?;

        Ok(())
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.magic != MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "LZ4Block magic not found"));
        }

        let (NO_COMPRESSION | COMPRESSION) = self.method else {
            return Err(Error::new(ErrorKind::InvalidData, "Not a valid compression method"));
        };

        if self.decompressed_len > (1 << self.level) {
            return Err(Error::new(ErrorKind::InvalidData, "Length does not match compression"));
        };

        if self.compressed_len < 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Negative compressed length"));
        }

        if self.decompressed_len < 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Negative length"));
        }

        if self.method == NO_COMPRESSION && self.decompressed_len != self.compressed_len {
            return Err(Error::new(ErrorKind::InvalidData, "No compression yet there is a compression difference"));
        }

        Ok(())
    }
}

impl LZ4Block {
    pub fn new() -> Self {
        Self {
            header: LZ4BlockHeader {
                magic: MAGIC,
                method: 0,
                level: 0,
                compressed_len: 0,
                decompressed_len: 0,
                checksum: 0
            },
            data: Vec::new(),
            compressed: false
        }
    }

    pub fn read<R>(mut reader: R) -> io::Result<Self> where R : Read {
        let header = LZ4BlockHeader::read(&mut reader)?;

        let mut data = vec![0u8; header.compressed_len as usize];
        reader.read_exact(data.as_mut_slice())?;

        Ok(Self { header, data, compressed: true })
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()> where W : Write {
        self.header.write(&mut writer)?;
        writer.write_all(self.data.as_slice())?;
        Ok(())
    }
    
    pub fn decompress(&mut self, validate_checksum: bool) -> io::Result<()> {
        if !self.compressed {
            return Ok(());
        }

        let header = &self.header;
        let data = &mut self.data;

        header.validate()?;

        decompress(data, header.method, header.decompressed_len)?;
        self.compressed = false;

        if validate_checksum {
            let checksum = compute_checksum(data);
            if header.checksum != checksum {
                return Err(Error::new(ErrorKind::InvalidData, "Checksum check failed"));
            }
        }

        Ok(())
    }

    pub fn compress(&mut self, level: u8, compression: LZ4BlockCompression) -> io::Result<()> {
        if self.compressed {
            return Ok(());
        }

        let data = &mut self.data;

        if data.len() > (1 << level) {
            return Err(Error::new(ErrorKind::InvalidData, "Data too large"));
        }

        let checksum = compute_checksum(&data);
        let (method, c_len, d_len) = compress(data, compression)?;

        self.header.magic = MAGIC;
        self.header.method = method;
        self.header.level = level;
        self.header.decompressed_len = d_len as i32;
        self.header.compressed_len = c_len as i32;
        self.header.checksum = checksum;

        self.compressed = true;

        Ok(())
    }
}

fn compute_checksum(data: &Vec<u8>) -> u32 {
    if data.len() == 0 {
        0
    } else {
        xxh::xxh32::xxh32(data.as_slice(), CHECKSUM_SEED) & CHECKSUM_MASK
    }
}

fn compress(data: &mut Vec<u8>, mode: LZ4BlockCompression) -> io::Result<(u8, usize, usize)> {
    let len = data.len();
    if len == 0 {
        return Ok((NO_COMPRESSION, 0, 0));
    }

    let mode = match mode {
        LZ4BlockCompression::Fast(t) => CompressionMode::FAST(t),
        LZ4BlockCompression::High(t) => CompressionMode::HIGHCOMPRESSION(t),
        LZ4BlockCompression::None => return Ok((NO_COMPRESSION, len, len)),
    };
    
    let compressed = lz4::block::compress(data.as_slice(), Some(mode), false)?;
    let c_len = compressed.len();

    if c_len >= len {
        Ok((NO_COMPRESSION, len, len))
    } else {
        *data = compressed;
        Ok((COMPRESSION, c_len, len))
    }
}

fn decompress(data: &mut Vec<u8>, method: u8, d_len: i32) -> io::Result<()> {
    if method == COMPRESSION {
        *data = lz4::block::decompress(data.as_slice(), Some(d_len))?;
    }

    Ok(())
}