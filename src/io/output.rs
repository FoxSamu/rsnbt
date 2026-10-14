use std::collections::BTreeSet;
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Write;

use flate2::Compression as CompressionLevel;
use flate2::write::DeflateEncoder;
use flate2::write::GzEncoder;
use flate2::write::ZlibEncoder;

use crate::tag::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyListId {
    /// Use the [`TagList`] branch to determine the ID for empty list type.
    AsIs,

    /// Use the `End` ID (a special ID 0) for empty list types.
    #[default]
    End,

    /// Use specifically the given ID for  the empty list type.
    Specifically(TagId),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CompoundOrder {
    /// Do not sort compound tags
    #[default]
    AsIs,

    /// Serialize compound tags in alphabetic order (A-Z)
    Alphabetically,

    /// Serialize compound tags in reverse alphabetic order (Z-A)
    ReversedAlphabetically,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// Do not compress, write raw data
    Uncompressed,

    /// Use GZip but do not compress
    GZipNone,

    /// Use GZip with fast but less efficient compression
    GZipFast,

    /// Use GZip with medium compression with medium efficiency
    #[default]
    GZipMid,

    /// Use GZip with slow but efficient compression
    GZipBest,

    /// Use ZLib but do not compress
    ZLibNone,

    /// Use ZLib with fast but less efficient compression
    ZLibFast,

    /// Use ZLib with medium compression with medium efficiency
    ZLibMid,

    /// Use ZLib with slow but most efficient compression
    ZLibBest,

    /// Use Deflate but do not compress
    DeflateNone,

    /// Use Deflate with fast but less efficient compression
    DeflateFast,

    /// Use Deflate with medium compression with medium efficiency
    DeflateMid,

    /// Use Deflate with slow but most efficient compression
    DeflateBest,

    /// Use LZ4 but do not compress
    LZ4FrameNone,

    /// Use LZ4 with fast but less efficient compression
    LZ4FrameFast,

    /// Use LZ4 with medium compression with medium efficiency
    LZ4FrameMid,

    /// Use LZ4 with slow but most efficient compression
    LZ4FrameBest,

    // TODO LZ4Block and raw LZ4
    // #[cfg(feature = "lz4b")]
    // LZ4Block ....
}

impl Compression {
    fn gzip_level(&self) -> Option<CompressionLevel> {
        match self {
            Compression::GZipNone => Some(CompressionLevel::none()),
            Compression::GZipFast => Some(CompressionLevel::fast()),
            Compression::GZipMid => Some(CompressionLevel::new(5)),
            Compression::GZipBest => Some(CompressionLevel::best()),
            _ => None,
        }
    }

    fn zlib_level(&self) -> Option<CompressionLevel> {
        match self {
            Compression::ZLibNone => Some(CompressionLevel::none()),
            Compression::ZLibFast => Some(CompressionLevel::fast()),
            Compression::ZLibMid => Some(CompressionLevel::new(5)),
            Compression::ZLibBest => Some(CompressionLevel::best()),
            _ => None,
        }
    }

    fn deflate_level(&self) -> Option<CompressionLevel> {
        match self {
            Compression::DeflateNone => Some(CompressionLevel::none()),
            Compression::DeflateFast => Some(CompressionLevel::fast()),
            Compression::DeflateMid => Some(CompressionLevel::new(5)),
            Compression::DeflateBest => Some(CompressionLevel::best()),
            _ => None,
        }
    }

    fn lz4_level(&self) -> Option<u32> {
        match self {
            Compression::LZ4FrameNone => Some(0),
            Compression::LZ4FrameFast => Some(1),
            Compression::LZ4FrameMid => Some(5),
            Compression::LZ4FrameBest => Some(9),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtOutputConfig {
    /// Specifies the tag ID to write as element type for empty lists.
    ///
    /// # Impact on Minecraft
    /// Usually Minecraft exports empty lists with the `End` ID, but
    /// in older versions of Java Edition this may not be valid.
    pub empty_list_id: EmptyListId,

    /// Specifies in what order compound tags elements should be written.
    ///
    /// # Impact on Minecraft
    /// The order in which elements appear is completely irrelevant to Minecraft,
    /// but it may be preferable in other tools. By default, the insertion order
    /// in [`TagMap`]s is used as order.
    pub compound_order: CompoundOrder,

    /// Specifies whether numbers should be written in big-endian format.
    /// This affects numeric tags, and lengths of strings, arrays and lists.
    ///
    /// # Impact on Minecraft
    /// The endianness differs between Minecraft Java Edition and Minecraft Bedrock Edition.
    /// Java Edition uses big-endian whereas Bedrock Edition uses little-endian.
    pub big_endian: bool,

    /// The max nesting depth of tags. If this is exceeded, an error is generated. Note that
    /// the error is generated during exporting, so any data that came before the error occured
    /// will have been written.
    ///
    /// # Impact on Minecraft
    /// Minecraft will not accept NBT tags with nesting beyond a depth of 512, but in some cases
    /// this nesting limit will be less.
    pub max_depth: usize,

    /// The type and level of compression to use.
    ///
    /// # Impact on Minecraft
    /// Minecraft usually compresses NBT using GZip, but occasionally it uses ZLib or uses no
    /// compression at all. Minecraft does not use Deflate directly. The compression level is
    /// completely irrelevant to Minecraft.
    pub compression: Compression,
}

pub fn encode<'a, R, W>(name: &String, tag: R, mut stream: W, config: &NbtOutputConfig) -> Result<()>
where
    R: Into<TagRef<'a>>,
    W: Write,
{
    if let Some(level) = config.compression.gzip_level() {
        let mut encoder = GzEncoder::new(stream, level);

        let mut output = NbtOutput {
            stream: &mut encoder,
            config,
        };

        output.write_root(name, tag.into())
    } else if let Some(level) = config.compression.zlib_level() {
        let mut encoder = ZlibEncoder::new(stream, level);

        let mut output = NbtOutput {
            stream: &mut encoder,
            config,
        };

        output.write_root(name, tag.into())
    } else if let Some(level) = config.compression.deflate_level() {
        let mut encoder = DeflateEncoder::new(stream, level);

        let mut output = NbtOutput {
            stream: &mut encoder,
            config,
        };

        output.write_root(name, tag.into())
    } else if let Some(level) = config.compression.lz4_level() {
        let mut encoder = lz4::EncoderBuilder::new()
            .level(level)
            .build(stream)?;

        let mut output = NbtOutput {
            stream: &mut encoder,
            config,
        };

        output.write_root(name, tag.into())
    } else {
        let mut output = NbtOutput {
            stream: &mut stream,
            config,
        };

        output.write_root(name, tag.into())
    }
}

pub fn encode_blob<'a, R>(name: &String, tag: R, config: &NbtOutputConfig) -> Result<Vec<u8>>
where
    R: Into<TagRef<'a>>,
{
    let mut cursor = Cursor::default();
    encode(name, tag, &mut cursor, config)?;
    Ok(cursor.into_inner())
}

macro_rules! as_bytes {
    ($value:expr, $config:expr) => {
        if $config.big_endian {
            ($value).to_be_bytes()
        } else {
            ($value).to_le_bytes()
        }
    };
}

impl Default for NbtOutputConfig {
    fn default() -> Self {
        Self {
            empty_list_id: Default::default(),
            compound_order: Default::default(),
            big_endian: true,
            max_depth: 512,
            compression: Default::default(),
        }
    }
}

struct NbtOutput<'a, 'b, W>
where
    W: Write,
{
    stream: &'a mut W,
    config: &'b NbtOutputConfig,
}

impl<'a, 'b, W> NbtOutput<'a, 'b, W>
where
    W: Write,
{
    fn write_end(&mut self) -> Result<()> {
        self.stream.write_all(&[0])
    }

    fn write_tag_id(&mut self, id: TagId) -> Result<()> {
        self.stream.write_all(&[id.byte()])
    }

    fn write_i8(&mut self, v: i8) -> Result<()> {
        self.stream.write_all(&[v as u8])
    }

    fn write_i16(&mut self, v: i16) -> Result<()> {
        self.stream.write_all(&as_bytes!(v, self.config))
    }

    fn write_i32(&mut self, v: i32) -> Result<()> {
        self.stream.write_all(&as_bytes!(v, self.config))
    }

    fn write_i64(&mut self, v: i64) -> Result<()> {
        self.stream.write_all(&as_bytes!(v, self.config))
    }

    fn write_f32(&mut self, v: f32) -> Result<()> {
        self.stream.write_all(&as_bytes!(v, self.config))
    }

    fn write_f64(&mut self, v: f64) -> Result<()> {
        self.stream.write_all(&as_bytes!(v, self.config))
    }

    fn write_string(&mut self, str: &String) -> Result<()> {
        let len = str.len();

        if len > u16::MAX as usize {
            Err(Error::new(
                ErrorKind::QuotaExceeded,
                "String cannot be longer than 65535 bytes",
            ))?;
        };

        self.write_i16(len as i16)?;
        self.stream.write_all(str.as_bytes())?;

        Ok(())
    }

    fn write_byte_array(&mut self, arr: &Vec<i8>) -> Result<()> {
        let len = arr.len();

        if len > i32::MAX as usize {
            Err(Error::new(
                ErrorKind::QuotaExceeded,
                "Byte array cannot be longer than 2147483647 bytes",
            ))?;
        };

        self.write_i32(len as i32)?;

        let mut buf = vec![0u8; len];
        let mut offset = 0usize;

        for b in arr {
            buf[offset] = *b as u8;
            offset += 1;
        }

        self.stream.write_all(buf.as_slice())?;

        Ok(())
    }

    fn write_int_array(&mut self, arr: &Vec<i32>) -> Result<()> {
        let len = arr.len();

        if len > i32::MAX as usize {
            Err(Error::new(
                ErrorKind::QuotaExceeded,
                "Int array cannot be longer than 2147483647 ints",
            ))?;
        };

        if len > (isize::MAX / 4) as usize {
            Err(Error::new(ErrorKind::QuotaExceeded, "Int array exceeds addressing limit"))?;
        };

        self.write_i32(len as i32)?;

        let mut buf = vec![0u8; 4 * len];
        let mut offset = 0usize;

        if self.config.big_endian {
            for int in arr {
                let slice = &mut buf[offset..(offset + 4)];
                slice.copy_from_slice(&int.to_be_bytes());
                offset += 4;
            }
        } else {
            for int in arr {
                let slice = &mut buf[offset..(offset + 4)];
                slice.copy_from_slice(&int.to_le_bytes());
                offset += 4;
            }
        };

        self.stream.write_all(buf.as_slice())?;

        Ok(())
    }

    fn write_long_array(&mut self, arr: &Vec<i64>) -> Result<()> {
        let len = arr.len();

        if len > i32::MAX as usize {
            Err(Error::new(
                ErrorKind::QuotaExceeded,
                "Long array cannot be longer than 2147483647 longs",
            ))?;
        };

        if len > (isize::MAX / 8) as usize {
            Err(Error::new(ErrorKind::QuotaExceeded, "Long array exceeds addressing limit"))?;
        };

        self.write_i32(len as i32)?;

        let mut buf = vec![0u8; 8 * len];
        let mut offset = 0usize;

        if self.config.big_endian {
            for long in arr {
                let slice = &mut buf[offset..(offset + 8)];
                slice.copy_from_slice(&long.to_be_bytes());
                offset += 8;
            }
        } else {
            for long in arr {
                let slice = &mut buf[offset..(offset + 8)];
                slice.copy_from_slice(&long.to_le_bytes());
                offset += 8;
            }
        };

        self.stream.write_all(buf.as_slice())?;

        Ok(())
    }

    fn write_list(&mut self, list: &TagList, depth: usize) -> Result<()> {
        if depth > self.config.max_depth {
            Err(Error::new(ErrorKind::QuotaExceeded, "Too much nesting"))?;
        }

        let len = list.len();

        if len == 0 {
            match self.config.empty_list_id {
                EmptyListId::AsIs => match list.elem_id() {
                    Some(id) => self.write_tag_id(id)?,
                    None => self.write_end()?,
                },
                EmptyListId::End => self.write_end()?,
                EmptyListId::Specifically(id) => self.write_tag_id(id)?,
            };

            self.write_i32(0)?;

            return Ok(());
        };

        if len > i32::MAX as usize {
            return Err(Error::new(
                ErrorKind::QuotaExceeded,
                "List cannot be longer than 2147483647 elements",
            ));
        };

        self.write_i8(list.elem_id_byte() as i8)?;
        self.write_i32(len as i32)?;

        for elem in list.iter() {
            self.write_payload(elem, depth + 1)?;
        }

        Ok(())
    }

    fn write_map(&mut self, map: &TagMap, depth: usize) -> Result<()> {
        if depth > self.config.max_depth {
            Err(Error::new(ErrorKind::QuotaExceeded, "Too much nesting"))?;
        }

        match self.config.compound_order {
            CompoundOrder::AsIs => {
                for (name, tag) in map.iter() {
                    self.write_named_tag(name, tag.into(), depth)?;
                }
            },

            CompoundOrder::Alphabetically => {
                let keys = BTreeSet::from_iter(map.keys());

                for name in keys.into_iter() {
                    self.write_named_tag(name, map.get(name).unwrap().into(), depth)?;
                }
            },

            CompoundOrder::ReversedAlphabetically => {
                let keys = BTreeSet::from_iter(map.keys());

                for name in keys.into_iter().rev() {
                    self.write_named_tag(name, map.get(name).unwrap().into(), depth)?;
                }
            },
        }

        self.write_end()
    }

    fn write_named_tag(&mut self, name: &String, tag: TagRef, depth: usize) -> Result<()> {
        self.write_tag_id(tag.id())?;
        self.write_string(name)?;
        self.write_payload(tag, depth)?;
        Ok(())
    }

    fn write_root(&mut self, name: &String, tag: TagRef) -> Result<()> {
        self.write_named_tag(name, tag, 0)
    }

    fn write_payload(&mut self, tag: TagRef, depth: usize) -> Result<()> {
        match tag {
            TagRef::Byte(v) => self.write_i8(*v),
            TagRef::Short(v) => self.write_i16(*v),
            TagRef::Int(v) => self.write_i32(*v),
            TagRef::Long(v) => self.write_i64(*v),
            TagRef::Float(v) => self.write_f32(*v),
            TagRef::Double(v) => self.write_f64(*v),
            TagRef::String(v) => self.write_string(v),
            TagRef::ByteArray(v) => self.write_byte_array(v),
            TagRef::IntArray(v) => self.write_int_array(v),
            TagRef::LongArray(v) => self.write_long_array(v),
            TagRef::List(v) => self.write_list(v, depth),
            TagRef::Compound(v) => self.write_map(v, depth),
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_write() {
        let data = tag_map! {
            "foo": 1,
            "bar": 2,
            "baz": tag_list! [1, 2, 3],
            "gus": tag_map! {
                "mos": "ris",
                "ken": true
            }
        };

        let mut file = File::create("../../test_data/test.dat").unwrap();

        encode(&String::from("root"), &data, &mut file, &NbtOutputConfig::default()).unwrap();
    }
}
