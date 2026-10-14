use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;

use crate::io::Decompression;
use crate::io::NbtInputConfig;
use crate::io::decode;
use crate::region::bit_array::BitArray;
use crate::tag::Tag;
use crate::util::rw;

pub use super::sector_table::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkCompressionMode {
    GZip,
    ZLib,
    Raw,
    LZ4Block,

    Custom(String)
}

impl ChunkCompressionMode {
    /// Calculates the number of bytes required to store a header with this compression
    /// mode. For built-in compression modes, this is always 5 bytes. For the custom
    /// compression mode, the header includes a length-prefixed name of at most 65537
    /// bytes (two-byte length included).
    /// 
    /// This header is stored within the chunk's sectors, so its length determines the
    /// quota that remains for chunk data.
    pub fn header_len(&self) -> usize {
        let extra_bytes = match self {
            ChunkCompressionMode::Custom(name) => 2 + name.len(),
            _ => 0
        };

        return 5 + extra_bytes;
    }

    /// Calculates the maximum number of bytes that can be stored in a region file with
    /// this compression mode. This is equal to `1044480 - header_len`, see
    /// [`header_len`][ChunkCompressionMode::header_len].
    /// 
    /// Chunk data that exceeds this limit must be stored in an external file.
    pub fn internal_quota(&self) -> usize {
        MAX_PADDED_SIZE - self.header_len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkLocation {
    /// Chunk is stored in the region file.
    Internal {
        /// The offset of the chunk data within its allocated region, in bytes.
        /// The exact data address can be obtained via `data_offset + padded_address`.
        offset: usize,

        /// The exact length of the chunk data in its allocate region, in bytes.
        length: usize
    },

    /// Chunk is written to an external file.
    External
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkHeader {
    /// The last-updated timestamp recorded in the file header.
    last_updated: u32,

    /// The location of the chunk data. Chunks under 1020 KiB can be stored
    /// in the region file.
    location: ChunkLocation,

    /// The address of the first sector of the chunk data, in bytes. This is
    /// always a multiple of 4096. This is **NOT** the start of the chunk data.
    padded_address: u64,

    /// The padded length of the chunk data. This is always a multiple of 4096.
    /// This is **NOT** the length of the chunk data.
    padded_length: usize,

    /// The index of the first sector of the chunk data, in number of sectors.
    start_sector: u32,

    /// The number of sectors the chunk data spans.
    num_sectors: u8,

    /// The compression mode of the chunk data.
    compression: ChunkCompressionMode
}

impl ChunkHeader {
    pub fn last_updated(&self) -> u32 {
        self.last_updated
    }

    pub fn location(&self) -> ChunkLocation {
        self.location
    }

    pub fn padded_address(&self) -> u64 {
        self.padded_address
    }

    pub fn padded_length(&self) -> usize {
        self.padded_length
    }

    pub fn start_sector(&self) -> u32 {
        self.start_sector
    }

    pub fn sector_count(&self) -> u8 {
        self.num_sectors
    }

    pub fn compression(&self) -> &ChunkCompressionMode {
        &self.compression
    }

    pub fn nbt_decompression(&self) -> Option<Decompression> {
        match &self.compression {
            ChunkCompressionMode::GZip => Some(Decompression::GZip),
            ChunkCompressionMode::ZLib => Some(Decompression::ZLib),
            ChunkCompressionMode::Raw => Some(Decompression::Uncompressed),
            ChunkCompressionMode::LZ4Block => Some(Decompression::LZ4Block),
            ChunkCompressionMode::Custom(_) => None,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkBlob {
    header: ChunkHeader,
    padded_data: Vec<u8>
}

impl ChunkBlob {
    pub fn header(&self) -> &ChunkHeader {
        &self.header
    }

    pub fn data(&self) -> Option<&[u8]> {
        match self.header.location {
            ChunkLocation::Internal { offset, length } => Some(&self.padded_data[offset..offset+length]),
            ChunkLocation::External => None,
        }
    }

    pub fn padded_data(&self) -> &[u8] {
        &self.padded_data.as_slice()
    }

    pub fn decode_nbt(&self) -> io::Result<Option<(String, Tag)>> {
        let Some(data) = self.data() else {
            return Ok(None);
        };

        let Some(mode) = self.header.nbt_decompression() else {
            return Ok(None);
        };
        
        let decode_options = NbtInputConfig {
            big_endian: true,
            allow_empty_end_list: true,
            max_depth: 512,
            decompression: mode,
        };

        Ok(Some(decode(Cursor::new(data), &decode_options)?))
    }
}




pub struct RegionFile<F> {
    file: F,
    header: RegionHeader,
    sectors: BitArray
}

impl RegionFile<File> {
    pub fn open_file<P>(path: P) -> io::Result<Self> where P : AsRef<Path> {
        Self::open(File::open(path)?)
    }

    pub fn create_file<P>(path: P) -> io::Result<Self> where P : AsRef<Path> {
        Self::create(File::create(path)?)
    }

    pub fn overwrite_file<P>(path: P) -> io::Result<Self> where P : AsRef<Path> {
        Self::overwrite(File::create(path)?)
    }
}

impl<F> RegionFile<F> where F : Read + Seek {
    pub fn open(mut file: F) -> io::Result<Self> {
        let mut header = RegionHeader::new();

        file.seek(SeekFrom::Start(0))?;
        header.read(&mut file)?;

        let mut sectors = BitArray::new();
        for i in 0..CHUNKS_PER_REGION {
            let ch = header.get_chunk_by_index(i)?;
            sectors.set_range(
                ch.offset as usize, 
                ch.offset as usize + ch.length as usize
            );
        }

        Ok(Self {
            file,
            header,
            sectors
        })
    }

    pub fn header_by_index(&mut self, index: usize) -> io::Result<Option<ChunkHeader>> {
        let info = self.header.get_chunk_by_index(index)?;
        if info.length == 0 {
            return Ok(None);
        }

        let offset = info.offset as u64 * SECTOR_SIZE as u64;
        
        self.file.seek(SeekFrom::Start(offset))?;

        let len = rw::read_u32_be(&mut self.file)?;
        let mode_bits = rw::read_u8(&mut self.file)?;
        
        let mode = match mode_bits & 0x7F {
            1 => ChunkCompressionMode::GZip,
            2 => ChunkCompressionMode::ZLib,
            3 => ChunkCompressionMode::Raw,
            4 => ChunkCompressionMode::LZ4Block,
            127 => ChunkCompressionMode::Custom(rw::read_string_be(&mut self.file)?),
            _ => return Err(Error::new(ErrorKind::InvalidData, "unknown compression mode"))
        };

        let external = mode_bits & 0x80 == 0x80;

        Ok(Some(ChunkHeader {
            last_updated: info.last_updated,

            location: if external {
                ChunkLocation::External
            } else {
                ChunkLocation::Internal {
                    offset: (self.file.stream_position()? - offset) as usize,
                    length: len as usize
                }
            },

            padded_address: offset,
            padded_length: info.length as usize * SECTOR_SIZE,

            start_sector: info.offset,
            num_sectors: info.length,

            compression: mode
        }))
    }

    pub fn header(&mut self, x: u32, z: u32) -> io::Result<Option<ChunkHeader>> {
        let x = (x & 31) as usize;
        let z = (z & 31) as usize;
        self.header_by_index(z * 32 + x)
    }

    pub fn load_by_index(&mut self, index: usize) -> io::Result<Option<ChunkBlob>> {
        let Some(header) = self.header_by_index(index)? else {
            return Ok(None);
        };
        
        let mut padded_data = vec![0u8; header.padded_length];

        self.file.seek(SeekFrom::Start(header.padded_address))?;
        self.file.read_exact(padded_data.as_mut_slice())?;

        Ok(Some(ChunkBlob { header, padded_data }))
    }

    pub fn load(&mut self, x: u32, z: u32) -> io::Result<Option<ChunkBlob>> {
        let x = (x & 31) as usize;
        let z = (z & 31) as usize;
        self.load_by_index(z * 32 + x)
    }
}


impl<F> RegionFile<F> where F : Read + Seek + Write {
    pub fn create(mut file: F) -> io::Result<Self> {
        if file.seek(SeekFrom::End(0))? == 0 {
            Self::overwrite(file)
        } else {
            Self::open(file)
        }
    }

    pub fn overwrite(mut file: F) -> io::Result<Self> {
        let header = RegionHeader::new();

        file.seek(SeekFrom::Start(0))?;
        header.write(&mut file)?;

        Ok(Self {
            file,
            header,
            sectors: BitArray::new()
        })
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.header.clear();
        self.sectors.clear();

        self.file.seek(SeekFrom::Start(0))?;
        self.header.write(&mut self.file)?;

        Ok(())
    }

    pub fn delete_by_index(&mut self, index: usize) -> io::Result<()> {
        let ch = self.header.get_chunk_by_index(index)?;
        self.sectors.reset_range(
            ch.offset as usize, 
            ch.offset as usize + ch.length as usize
        );

        self.header.delete_chunk_by_index(index)?;

        self.file.seek(SeekFrom::Start(0))?;
        self.header.write(&mut self.file)?;

        Ok(())
    }

    pub fn delete(&mut self, x: u32, z: u32) -> io::Result<()> {
        let x = (x & 31) as usize;
        let z = (z & 31) as usize;
        self.delete_by_index(z * 32 + x)
    }

    fn reallocate_chunk_sectors(&mut self, index: usize, num_sectors: u8) -> io::Result<u32> {
        let start_sector = if let Some(ch) = self.header_by_index(index)? {
            // Old chunk data exists

            // Deallocate currently used sectors
            self.sectors.reset_range(
                ch.start_sector as usize, 
                ch.start_sector as usize + ch.num_sectors as usize
            );

            // Check if there is enough space here for our data
            // (if we use less sectors than the previous chunk we for sure
            // have enough space)
            let gap = num_sectors <= ch.num_sectors || self.sectors.is_gap(
                ch.start_sector as usize, 
                ch.start_sector as usize + num_sectors as usize
            );

            if gap {
                // Our data fits where the old chunk data was,
                // so we don't need to search for more space
                ch.start_sector
            } else {
                // Our data doesn't fit here, find something better.
                self.sectors.find_gap(2, num_sectors as usize) as u32
            }
        } else {
            // No old chunk data, find space
            self.sectors.find_gap(2, num_sectors as usize) as u32
        };

        Ok(start_sector)
    }

    fn write_header(&mut self, len: usize, mode: &ChunkCompressionMode) -> io::Result<()> {
        // Write to a buffer to reduce IO synchronizations with underlying file
        let mut data = Vec::new();

        rw::write_u32_be(&mut data, len as u32)?;

        match &mode {
            ChunkCompressionMode::GZip => rw::write_u8(&mut data, 1)?,
            ChunkCompressionMode::ZLib => rw::write_u8(&mut data, 2)?,
            ChunkCompressionMode::Raw => rw::write_u8(&mut data, 3)?,
            ChunkCompressionMode::LZ4Block => rw::write_u8(&mut data, 4)?,
            ChunkCompressionMode::Custom(name) => {
                rw::write_u8(&mut data, 127)?;
                rw::write_string_be(&mut data, name)?;
            },
        }

        self.file.write_all(data.as_slice())?;

        Ok(())
    }
    
    fn seek_or_write_to(&mut self, index: u64) -> io::Result<()> {
        let end = self.file.seek(SeekFrom::Start(index))?;

        if end < index {
            // Seek got clamped to end? Let's just write til we are
            // at the location we want to be.
            let diff = index - end;
            self.file.write_all(vec![0; diff as usize].as_slice())?;
        }

        Ok(())
    }

    pub fn store_internal_by_index(&mut self, index: usize, data: &[u8], update_timestamp: u32, mode: ChunkCompressionMode) -> io::Result<ChunkHeader> {
        let len = data.len();

        if len > mode.internal_quota() {
            return Err(Error::new(ErrorKind::QuotaExceeded, "Data is too large to store in region file, save it externally"));
        }

        let total_len = mode.header_len() + len;

        // Compute number of sectors needed
        let num_sectors = if total_len & 0xFFF == 0 {
            total_len >> 12
        } else {
            (total_len >> 12) + 1
        } as u8;
        
        // Find a start sector
        let start_sector = self.reallocate_chunk_sectors(index, num_sectors)?;

        let padded_address = SECTOR_SIZE as u64 * start_sector as u64;
        let padded_length = SECTOR_SIZE * num_sectors as usize;


        // Writea header
        self.seek_or_write_to(padded_address)?;
        self.write_header(len, &mode)?;
        
        // Write data
        self.file.write_all(data)?;

        // Add padding
        let padding_remaining = padded_length - total_len;
        self.file.write_all(vec![0; padding_remaining].as_slice())?;

        // Update and write file header
        self.header.put_chunk_by_index(index, SectorInfo {
            offset: start_sector,
            length: num_sectors,
            last_updated: update_timestamp
        })?;

        self.file.seek(SeekFrom::Start(0))?;
        self.header.write(&mut self.file)?;


        // Return generated chunk header
        Ok(ChunkHeader {
            last_updated: update_timestamp,
            location: ChunkLocation::Internal {
                offset: mode.header_len(),
                length: len
            },
            padded_address,
            padded_length,
            start_sector,
            num_sectors,
            compression: mode,
        })
    }

    pub fn store_internal(&mut self, x: u32, z: u32, data: &[u8], update_timestamp: u32, mode: ChunkCompressionMode) -> io::Result<ChunkHeader> {
        let x = (x & 31) as usize;
        let z = (z & 31) as usize;
        self.store_internal_by_index(z * 32 + x, data, update_timestamp, mode)
    }

    pub fn store_external_by_index(&mut self, index: usize, update_timestamp: u32, mode: ChunkCompressionMode) -> io::Result<ChunkHeader> {
        let total_len = mode.header_len();

        // Compute number of sectors needed
        let num_sectors = if total_len & 0xFFF == 0 {
            total_len >> 12
        } else {
            (total_len >> 12) + 1
        } as u8;
        
        // Find a start sector
        let start_sector = self.reallocate_chunk_sectors(index, num_sectors)?;


        let padded_address = SECTOR_SIZE as u64 * start_sector as u64;
        let padded_length = SECTOR_SIZE * num_sectors as usize;


        // Writea header
        self.seek_or_write_to(padded_address)?;
        self.write_header(1, &mode)?;

        // Add padding
        let padding_remaining = padded_length - total_len;
        self.file.write_all(vec![0; padding_remaining].as_slice())?;

        // Update and write file header
        self.header.put_chunk_by_index(index, SectorInfo {
            offset: start_sector,
            length: num_sectors,
            last_updated: update_timestamp
        })?;

        self.file.seek(SeekFrom::Start(0))?;
        self.header.write(&mut self.file)?;


        // Return generated chunk header
        Ok(ChunkHeader {
            last_updated: update_timestamp,
            location: ChunkLocation::External,
            padded_address,
            padded_length,
            start_sector,
            num_sectors,
            compression: mode,
        })
    }

    pub fn store_external(&mut self, x: u32, z: u32, update_timestamp: u32, mode: ChunkCompressionMode) -> io::Result<ChunkHeader> {
        let x = (x & 31) as usize;
        let z = (z & 31) as usize;
        self.store_external_by_index(z * 32 + x, update_timestamp, mode)
    }
}