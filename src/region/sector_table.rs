use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

pub const CHUNKS_PER_REGION: usize = 32 * 32;
pub const SECTOR_SIZE: usize = 4096;
pub const HEADER_SIZE: usize = 2 * SECTOR_SIZE;
pub const MAX_PADDED_SIZE: usize = 255 * SECTOR_SIZE;

#[derive(Debug)]
pub struct RegionHeader {
    data: [u8; HEADER_SIZE]
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SectorInfo {
    pub offset: u32,
    pub length: u8,
    pub last_updated: u32
}

impl RegionHeader {
    pub fn new() -> Self {
        Self {
            data: [0u8; HEADER_SIZE]
        }
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()> where W : Write {
        writer.write_all(&self.data)
    }

    pub fn read<R>(&mut self, mut reader: R) -> io::Result<()> where R : Read {
        reader.read_exact(&mut self.data)
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn get_chunk_by_index(&self, index: usize) -> io::Result<SectorInfo> {
        if index > CHUNKS_PER_REGION {
            panic!("Index ({}) exceeds number of chunks per region ({})", index, CHUNKS_PER_REGION);
        }

        let position_addr = index * 4;
        let timestamp_addr = position_addr + SECTOR_SIZE;

        let mut position_bytes = [0u8; 4];
        let mut timestamp_bytes = [0u8; 4];

        position_bytes.copy_from_slice(&self.data[position_addr..(position_addr+4)]);
        timestamp_bytes.copy_from_slice(&self.data[timestamp_addr..(timestamp_addr+4)]);

        let position = u32::from_be_bytes(position_bytes);
        let timestamp = u32::from_be_bytes(timestamp_bytes);

        let offset = position >> 8;
        let length = (position & 0xFF) as u8;
        
        if offset < 2 && length > 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Non-empty chunk data overlaps with header"));
        }

        Ok(SectorInfo { offset, length, last_updated: timestamp })
    }

    pub fn put_chunk_by_index(&mut self, index: usize, info: SectorInfo) -> io::Result<()> {
        if index > CHUNKS_PER_REGION {
            panic!("Index ({}) exceeds number of chunks per region ({})", index, CHUNKS_PER_REGION);
        }

        let SectorInfo { 
            offset,
            length,
            last_updated: timestamp
        } = info;

        if offset > 0xFFFFFF {
            return Err(Error::new(ErrorKind::InvalidInput, "Offset cannot exceed 2^24 - 1"));
        }

        if offset < 2 && length > 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Non-empty chunk data overlaps with header"));
        }

        let position = offset << 8 | length as u32;

        let position_addr = index * 4;
        let timestamp_addr = position_addr + SECTOR_SIZE;

        let position_bytes = position.to_be_bytes();
        let timestamp_bytes = timestamp.to_be_bytes();

        (&mut self.data[position_addr..(position_addr+4)]).copy_from_slice(&position_bytes);
        (&mut self.data[timestamp_addr..(timestamp_addr+4)]).copy_from_slice(&timestamp_bytes);

        Ok(())
    }

    pub fn delete_chunk_by_index(&mut self, index: usize) -> io::Result<()> {
        if index > CHUNKS_PER_REGION {
            panic!("Index ({}) exceeds number of chunks per region ({})", index, CHUNKS_PER_REGION);
        }

        let position_addr = index * 4;
        let timestamp_addr = position_addr + SECTOR_SIZE;

        (&mut self.data[position_addr..(position_addr+4)]).fill(0);
        (&mut self.data[timestamp_addr..(timestamp_addr+4)]).fill(0);

        Ok(())
    }
}

