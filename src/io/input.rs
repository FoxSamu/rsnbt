use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;

use flate2::read::DeflateDecoder;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use lz4::Decoder as Lz4Decoder;

use crate::tag::*;

const MAX_INTS: usize = (isize::MAX / 4) as usize;
const MAX_LONGS: usize = (isize::MAX / 8) as usize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Decompression {
    Uncompressed,
    #[default]
    GZip,
    ZLib,
    Lz4,
    Deflate,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NbtInputConfig {
    pub big_endian: bool,
    pub allow_empty_end_list: bool,
    pub max_depth: usize,
    pub decompression: Decompression,
}

pub fn decode<R>(mut stream: R, config: &NbtInputConfig) -> Result<(String, Tag)>
where
    R: Read,
{
    match config.decompression {
        Decompression::Uncompressed => {
            let mut input = NbtInput {
                stream: &mut stream,
                config,
            };

            input.read_root()
        },

        Decompression::GZip => {
            let mut input = NbtInput {
                stream: &mut GzDecoder::new(stream),
                config,
            };

            input.read_root()
        },

        Decompression::ZLib => {
            let mut input = NbtInput {
                stream: &mut ZlibDecoder::new(stream),
                config,
            };

            input.read_root()
        },

        Decompression::Deflate => {
            let mut input = NbtInput {
                stream: &mut DeflateDecoder::new(stream),
                config,
            };

            input.read_root()
        },

        Decompression::Lz4 => {
            let mut input = NbtInput {
                stream: &mut Lz4Decoder::new(stream)?,
                config,
            };

            input.read_root()
        },
    }
}

pub fn decode_blob(blob: &[u8], config: &NbtInputConfig) -> Result<(String, Tag)> {
    decode(&mut Cursor::new(blob), config)
}

struct NbtInput<'a, 'b, R>
where
    R: Read,
{
    stream: &'a mut R,
    config: &'b NbtInputConfig,
}

fn invalid_data<T, R>(reason: R) -> Result<T>
where
    R: Into<String>,
{
    Err(Error::new(ErrorKind::InvalidData, reason.into()))
}

fn too_large<T, R>(reason: R) -> Result<T>
where
    R: Into<String>,
{
    Err(Error::new(ErrorKind::QuotaExceeded, reason.into()))
}

macro_rules! from_bytes {
    ($num:ident, $value:expr, $config:expr) => {
        if $config.big_endian {
            $num::from_be_bytes($value)
        } else {
            $num::from_le_bytes($value)
        }
    };
}

impl<'a, 'b, R> NbtInput<'a, 'b, R>
where
    R: Read,
{
    fn read_tag_id(&mut self) -> Result<Option<TagId>> {
        let mut buf = [0u8];
        self.stream.read_exact(&mut buf)?;

        let byte = buf[0];

        if byte == 0 {
            Ok(None)
        } else if let Some(id) = TagId::from_byte(byte) {
            Ok(Some(id))
        } else {
            invalid_data(format!("Found invalid tag ID byte: {}", byte))
        }
    }

    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8];
        self.stream.read_exact(&mut buf)?;

        Ok(buf[0] as i8)
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.stream.read_exact(&mut buf)?;

        Ok(from_bytes!(i16, buf, self.config))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf)?;

        Ok(from_bytes!(i32, buf, self.config))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.stream.read_exact(&mut buf)?;

        Ok(from_bytes!(i64, buf, self.config))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.stream.read_exact(&mut buf)?;

        Ok(from_bytes!(f32, buf, self.config))
    }

    fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.stream.read_exact(&mut buf)?;

        Ok(from_bytes!(f64, buf, self.config))
    }

    fn read_n_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; n];
        self.stream.read_exact(buf.as_mut_slice())?;
        Ok(buf)
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_i16()? as u16 as usize;
        let bytes = self.read_n_bytes(len)?;

        String::from_utf8(bytes).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }

    fn read_byte_array(&mut self) -> Result<Vec<i8>> {
        let len = self.read_i32()?.max(0) as usize;
        let bytes = self.read_n_bytes(len)?;

        let mut ibytes = Vec::new();
        for v in bytes {
            ibytes.push(v as i8);
        }

        Ok(ibytes)
    }

    fn read_int_array(&mut self) -> Result<Vec<i32>> {
        let len = self.read_i32()?.max(0) as usize;

        if len > MAX_INTS {
            too_large(format!("More ints than allowed in memory: {}", len))?;
        };

        let bytes = self.read_n_bytes(len * 4)?;

        let mut ints = Vec::new();
        if self.config.big_endian {
            for v in bytes.chunks(4) {
                ints.push(i32::from_be_bytes(*v.as_array().unwrap()));
            }
        } else {
            for v in bytes.chunks(4) {
                ints.push(i32::from_le_bytes(*v.as_array().unwrap()));
            }
        }

        Ok(ints)
    }

    fn read_long_array(&mut self) -> Result<Vec<i64>> {
        let len = self.read_i32()?.max(0) as usize;

        if len > MAX_LONGS {
            too_large(format!("More longs than allowed in memory: {}", len))?;
        };

        let bytes = self.read_n_bytes(len * 8)?;

        let mut longs = Vec::new();
        if self.config.big_endian {
            for v in bytes.chunks(8) {
                longs.push(i64::from_be_bytes(*v.as_array().unwrap()));
            }
        } else {
            for v in bytes.chunks(8) {
                longs.push(i64::from_le_bytes(*v.as_array().unwrap()));
            }
        }

        Ok(longs)
    }

    fn read_list(&mut self, depth: usize) -> Result<TagList> {
        if depth >= self.config.max_depth {
            return too_large("Too much nesting");
        }

        let maybe_id = self.read_tag_id()?;
        let len = self.read_i32()?.max(0) as usize;

        if let Some(id) = maybe_id {
            let mut list = TagList::new();

            for _ in 0..len {
                list.push(self.read_payload(id, depth + 1)?);
            }

            Ok(list)
        } else {
            if len != 0 {
                invalid_data("Found non-empty list of end type")?;
            }

            Ok(TagList::Empty)
        }
    }

    fn read_compound(&mut self, depth: usize) -> Result<TagMap> {
        if depth >= self.config.max_depth {
            return too_large("Too much nesting");
        }

        let mut result = TagMap::new();

        let mut maybe_id = self.read_tag_id()?;
        while let Some(id) = maybe_id {
            let name = self.read_string()?;
            let payload = self.read_payload(id, depth + 1)?;

            result.insert(name, payload);

            maybe_id = self.read_tag_id()?;
        }

        Ok(result)
    }

    fn read_payload(&mut self, id: TagId, depth: usize) -> Result<Tag> {
        Ok(match id {
            TagId::Byte => Tag::Byte(self.read_i8()?),
            TagId::Short => Tag::Short(self.read_i16()?),
            TagId::Int => Tag::Int(self.read_i32()?),
            TagId::Long => Tag::Long(self.read_i64()?),
            TagId::Float => Tag::Float(self.read_f32()?),
            TagId::Double => Tag::Double(self.read_f64()?),
            TagId::String => Tag::String(self.read_string()?),
            TagId::ByteArray => Tag::ByteArray(self.read_byte_array()?),
            TagId::IntArray => Tag::IntArray(self.read_int_array()?),
            TagId::LongArray => Tag::LongArray(self.read_long_array()?),
            TagId::List => Tag::List(self.read_list(depth)?),
            TagId::Compound => Tag::Compound(self.read_compound(depth)?),
        })
    }

    fn read_root(&mut self) -> Result<(String, Tag)> {
        if let Some(id) = self.read_tag_id()? {
            let name = self.read_string()?;
            let payload = self.read_payload(id, 0)?;

            Ok((name, payload))
        } else {
            invalid_data("Found tag of end type")
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use super::*;

    #[test]
    pub fn test_level_dat() {
        let mut file = File::open("../../test_data/test.dat").unwrap();

        let config = NbtInputConfig {
            big_endian: true,
            allow_empty_end_list: true,
            max_depth: 512,
            decompression: Decompression::GZip,
        };

        let decoded = decode(&mut file, &config).unwrap();
        dbg!(decoded);
    }
}
