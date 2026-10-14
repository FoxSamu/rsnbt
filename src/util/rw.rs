#![allow(dead_code)]

use std::io;
use std::io::Read;
use std::io::Write;

macro_rules! io_ops {
    ($ty:ident, $bytes:literal, $rd_be:ident $rd_le:ident $wr_be:ident $wr_le:ident) => {
        /// Reads one big-endian
        #[doc = concat!("[`", stringify!($ty), "`]")]
        /// from the given reader.
        /// That is, it reads
        #[doc = stringify!($bytes)]
        /// bytes of which the first byte represents the 8 **most** significant bits of the
        #[doc = concat!("[`", stringify!($ty), "`].")]
        ///
        /// If the end of stream was reached before all bytes could be read, then
        /// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
        pub fn $rd_be<R>(mut reader: R) -> io::Result<$ty> where R : Read {
            let mut arr = [0u8; size_of::<$ty>()];
            reader.read_exact(&mut arr)?;
            Ok($ty::from_be_bytes(arr))
        }

        /// Writes one big-endian
        #[doc = concat!("[`", stringify!($ty), "`]")]
        /// to the given writer.
        /// That is, it writes
        #[doc = stringify!($bytes)]
        /// bytes of which the first byte represents the 8 **most** significant bits of the
        #[doc = concat!("[`", stringify!($ty), "`].")]
        ///
        /// If the end of the writer was reached before all bytes could be written, then
        /// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
        pub fn $wr_be<W>(mut writer: W, v: $ty) -> io::Result<()> where W : Write {
            let arr = v.to_be_bytes();
            writer.write_all(&arr)?;
            Ok(())
        }

        /// Reads one little-endian
        #[doc = concat!("[`", stringify!($ty), "`]")]
        /// from the given reader.
        /// That is, it reads
        #[doc = stringify!($bytes)]
        /// bytes of which the first byte represents the 8 **least** significant bits of the
        #[doc = concat!("[`", stringify!($ty), "`].")]
        ///
        /// If the end of stream was reached before all bytes could be read, then
        /// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
        pub fn $rd_le<R>(mut reader: R) -> io::Result<$ty> where R : Read {
            let mut arr = [0u8; size_of::<$ty>()];
            reader.read_exact(&mut arr)?;
            Ok($ty::from_le_bytes(arr))
        }

        /// Writes one little-endian
        #[doc = concat!("[`", stringify!($ty), "`]")]
        /// to the given writer.
        /// That is, it writes
        #[doc = stringify!($bytes)]
        /// bytes of which the first byte represents the 8 **least** significant bits of the
        #[doc = concat!("[`", stringify!($ty), "`].")]
        ///
        /// If the end of the writer was reached before all bytes could be written, then
        /// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
        pub fn $wr_le<W>(mut writer: W, v: $ty) -> io::Result<()> where W : Write {
            let arr = v.to_le_bytes();
            writer.write_all(&arr)?;
            Ok(())
        }
    };
}

/// Reads one [`u8`] from the given reader. This reads 1 byte from the given
/// reader and returns it as `u8`.
///
/// If the end of stream was reached before the byte could be read, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
pub fn read_u8<R>(mut reader: R) -> io::Result<u8> where R : Read {
    let mut arr = [0u8];
    reader.read_exact(&mut arr)?;
    Ok(arr[0])
}

/// Writes one [`u8`] to the given writer. This writes 1 byte.
///
/// If the end of stream was reached before the byte could be written, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
pub fn write_u8<W>(mut writer: W, v: u8) -> io::Result<()> where W : Write {
    writer.write_all(&[v])?;
    Ok(())
}

/// Reads one [`i8`] from the given reader. This reads 1 byte from the given
/// reader and returns it as `i8`.
///
/// If the end of stream was reached before the byte could be read, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
pub fn read_i8<R>(mut reader: R) -> io::Result<i8> where R : Read {
    let mut arr = [0u8];
    reader.read_exact(&mut arr)?;
    Ok(arr[0] as i8)
}

/// Writes one [`i8`] to the given writer. This writes 1 byte.
///
/// If the end of stream was reached before the byte could be written, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
pub fn write_i8<W>(mut writer: W, v: i8) -> io::Result<()> where W : Write {
    writer.write_all(&[v as u8])?;
    Ok(())
}

io_ops!(u16, 2, read_u16_be read_u16_le write_u16_be write_u16_le);
io_ops!(u32, 4, read_u32_be read_u32_le write_u32_be write_u32_le);
io_ops!(u64, 8, read_u64_be read_u64_le write_u64_be write_u64_le);
io_ops!(u128, 16, read_u128_be read_u128_le write_u128_be write_u128_le);

io_ops!(i16, 2, read_i16_be read_i16_le write_i16_be write_i16_le);
io_ops!(i32, 4, read_i32_be read_i32_le write_i32_be write_i32_le);
io_ops!(i64, 8, read_i64_be read_i64_le write_i64_be write_i64_le);
io_ops!(i128, 16, read_i128_be read_i128_le write_i128_be write_i128_le);

io_ops!(f32, 4, read_f32_be read_f32_le write_f32_be write_f32_le);
io_ops!(f64, 8, read_f64_be read_f64_le write_f64_be write_f64_le);

/// Reads a UTF-8 [`String`] from the given reader. This reads a two-byte big-endian 
/// length, followed by that amount of bytes.
/// 
/// If the end of stream was reached before the string could be read, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
/// 
/// If the string bytes were not valid UTF-8, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::InvalidData] is generated.
pub fn read_string_be<R>(mut reader: R) -> io::Result<String> where R : Read {
    let len = read_u16_be(&mut reader)? as usize;
    let mut bin = vec![0u8; len];
    reader.read_exact(bin.as_mut_slice())?;

    Ok(String::from_utf8(bin).map_err(|v| io::Error::new(io::ErrorKind::InvalidData, v))?)
}

/// Reads a UTF-8 [`String`] from the given reader. This reads a two-byte little-endian 
/// length, followed by that amount of bytes.
/// 
/// If the end of stream was reached before the string could be read, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
/// 
/// If the string bytes were not valid UTF-8, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::InvalidData] is generated.
pub fn read_string_le<R>(mut reader: R) -> io::Result<String> where R : Read {
    let len = read_u16_le(&mut reader)? as usize;
    let mut bin = vec![0u8; len];
    reader.read_exact(bin.as_mut_slice())?;

    Ok(String::from_utf8(bin).map_err(|v| io::Error::new(io::ErrorKind::InvalidData, v))?)
}

/// Writes a UTF-8 [`String`] to the given writer. This writes a two-byte big-endian 
/// length, followed by that amount of bytes.
/// 
/// If the end of stream was reached before the string could be written, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
/// 
/// If the string length exceeds 65535 bytes (the maximum for a two-byte length), then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::QuotaExceeded] is generated.
pub fn write_string_be<W>(mut writer: W, v: &String) -> io::Result<()> where W : Write {
    let len = v.len();
    if len > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::QuotaExceeded, "String too long"));
    }
    
    let bytes: Vec<u8> = v.bytes().collect();

    write_u16_be(&mut writer, len as u16)?;
    writer.write_all(bytes.as_slice())?;

    Ok(())
}

/// Writes a UTF-8 [`String`] to the given writer. This writes a two-byte little-endian 
/// length, followed by that amount of bytes.
/// 
/// If the end of stream was reached before the string could be written, then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::UnexpectedEof] is generated.
/// 
/// If the string length exceeds 65535 bytes (the maximum for a two-byte length), then
/// [`ErrorKind::UnexpectedEof`][io::ErrorKind::QuotaExceeded] is generated.
pub fn write_string_le<W>(mut writer: W, v: &String) -> io::Result<()> where W : Write {
    let len = v.len();
    if len > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::QuotaExceeded, "String too long"));
    }
    
    let bytes: Vec<u8> = v.bytes().collect();

    write_u16_le(&mut writer, len as u16)?;
    writer.write_all(bytes.as_slice())?;

    Ok(())
}