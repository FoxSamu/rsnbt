use super::*;
use super::cast::*;
use crate::util::unsafe_ops::*;


impl Tag {
    pub fn empty_byte_array() -> Tag {
        Self::ByteArray(Vec::new())
    }

    pub fn empty_int_array() -> Tag {
        Self::IntArray(Vec::new())
    }

    pub fn empty_long_array() -> Tag {
        Self::LongArray(Vec::new())
    }

    pub fn byte_array<T>(v: T) -> Tag
    where
        T: ArrayPayload<i8>,
    {
        Self::ByteArray(v.array_payload())
    }

    pub fn int_array<T>(v: T) -> Tag
    where
        T: ArrayPayload<i32>,
    {
        Self::IntArray(v.array_payload())
    }

    pub fn long_array<T>(v: T) -> Tag
    where
        T: ArrayPayload<i64>,
    {
        Self::LongArray(v.array_payload())
    }



    pub fn into_i8_vec(self) -> Result<Vec<i8>, TagCastError<Self>> {
        if let Self::ByteArray(a) = self {
            Ok(a)
        } else {
            Err(TagCastError::new(TagId::ByteArray, self.id(), self))
        }
    }

    pub fn into_i32_vec(self) -> Result<Vec<i32>, TagCastError<Self>> {
        if let Self::IntArray(a) = self {
            Ok(a)
        } else {
            Err(TagCastError::new(TagId::IntArray, self.id(), self))
        }
    }

    pub fn into_i64_vec(self) -> Result<Vec<i64>, TagCastError<Self>> {
        if let Self::LongArray(a) = self {
            Ok(a)
        } else {
            Err(TagCastError::new(TagId::LongArray, self.id(), self))
        }
    }

    pub fn into_u8_vec(self) -> Result<Vec<u8>, TagCastError<Self>> {
        match self.as_u8_slice() {
            Ok(v) => Ok(Vec::from(v)),
            Err(e) => Err(TagCastError::couple(e, self))
        }
    }

    pub fn into_u32_vec(self) -> Result<Vec<u32>, TagCastError<Self>> {
        match self.as_u32_slice() {
            Ok(v) => Ok(Vec::from(v)),
            Err(e) => Err(TagCastError::couple(e, self))
        }
    }

    pub fn into_u64_vec(self) -> Result<Vec<u64>, TagCastError<Self>> {
        match self.as_u64_slice() {
            Ok(v) => Ok(Vec::from(v)),
            Err(e) => Err(TagCastError::couple(e, self))
        }
    }

    pub fn as_i8_slice(&self) -> Result<&[i8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_i32_slice(&self) -> Result<&[i32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_i64_slice(&self) -> Result<&[i64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_u8_slice(&self) -> Result<&[u8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i8 => u8))
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_u32_slice(&self) -> Result<&[u32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i32 => u32))
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_u64_slice(&self) -> Result<&[u64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i64 => u64))
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_mut_i8_slice(&mut self) -> Result<&mut [i8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_mut_i32_slice(&mut self) -> Result<&mut [i32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_mut_i64_slice(&mut self) -> Result<&mut [i64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_mut_u8_slice(&mut self) -> Result<&mut [u8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i8 => u8))
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_mut_u32_slice(&mut self) -> Result<&mut [u32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i32 => u32))
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_mut_u64_slice(&mut self) -> Result<&mut [u64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i64 => u64))
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }
}


impl TagRef<'_> {
    pub fn as_i8_slice(&self) -> Result<&[i8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_i32_slice(&self) -> Result<&[i32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_i64_slice(&self) -> Result<&[i64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_u8_slice(&self) -> Result<&[u8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i8 => u8))
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_u32_slice(&self) -> Result<&[u32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i32 => u32))
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_u64_slice(&self) -> Result<&[u64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i64 => u64))
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }
}


impl TagRefMut<'_> {
    pub fn as_i8_slice(&self) -> Result<&[i8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_i32_slice(&self) -> Result<&[i32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_i64_slice(&self) -> Result<&[i64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(a.as_slice())
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_u8_slice(&self) -> Result<&[u8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i8 => u8))
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_u32_slice(&self) -> Result<&[u32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i32 => u32))
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_u64_slice(&self) -> Result<&[u64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(reinterpret_slice!(a.as_slice(), i64 => u64))
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_mut_i8_slice(&mut self) -> Result<&mut [i8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_mut_i32_slice(&mut self) -> Result<&mut [i32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_mut_i64_slice(&mut self) -> Result<&mut [i64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(a.as_mut_slice())
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }

    pub fn as_mut_u8_slice(&mut self) -> Result<&mut [u8], TagMismatchError> {
        if let Self::ByteArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i8 => u8))
        } else {
            Err(TagMismatchError::new(TagId::ByteArray, self.id()))
        }
    }

    pub fn as_mut_u32_slice(&mut self) -> Result<&mut [u32], TagMismatchError> {
        if let Self::IntArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i32 => u32))
        } else {
            Err(TagMismatchError::new(TagId::IntArray, self.id()))
        }
    }

    pub fn as_mut_u64_slice(&mut self) -> Result<&mut [u64], TagMismatchError> {
        if let Self::LongArray(a) = self {
            Ok(reinterpret_mut_slice!(a.as_mut_slice(), i64 => u64))
        } else {
            Err(TagMismatchError::new(TagId::LongArray, self.id()))
        }
    }
}






pub trait ArrayPayload<T> where T : Copy + NumPayload {
    fn array_payload(self) -> Vec<T>;
}

impl ArrayPayload<i8> for () {
    fn array_payload(self) -> Vec<i8> {
        Vec::new()
    }
}

impl ArrayPayload<i32> for () {
    fn array_payload(self) -> Vec<i32> {
        Vec::new()
    }
}

impl ArrayPayload<i64> for () {
    fn array_payload(self) -> Vec<i64> {
        Vec::new()
    }
}

impl ArrayPayload<i8> for i8 {
    fn array_payload(self) -> Vec<i8> {
        vec![self]
    }
}

impl ArrayPayload<i32> for i32 {
    fn array_payload(self) -> Vec<i32> {
        vec![self]
    }
}

impl ArrayPayload<i64> for i64 {
    fn array_payload(self) -> Vec<i64> {
        vec![self]
    }
}

impl ArrayPayload<i8> for u8 {
    fn array_payload(self) -> Vec<i8> {
        vec![self as i8]
    }
}

impl ArrayPayload<i32> for u32 {
    fn array_payload(self) -> Vec<i32> {
        vec![self as i32]
    }
}

impl ArrayPayload<i64> for u64 {
    fn array_payload(self) -> Vec<i64> {
        vec![self as i64]
    }
}

impl ArrayPayload<i8> for &i8 {
    fn array_payload(self) -> Vec<i8> {
        vec![*self]
    }
}

impl ArrayPayload<i32> for &i32 {
    fn array_payload(self) -> Vec<i32> {
        vec![*self]
    }
}

impl ArrayPayload<i64> for &i64 {
    fn array_payload(self) -> Vec<i64> {
        vec![*self]
    }
}

impl ArrayPayload<i8> for &u8 {
    fn array_payload(self) -> Vec<i8> {
        vec![*self as i8]
    }
}

impl ArrayPayload<i32> for &u32 {
    fn array_payload(self) -> Vec<i32> {
        vec![*self as i32]
    }
}

impl ArrayPayload<i64> for &u64 {
    fn array_payload(self) -> Vec<i64> {
        vec![*self as i64]
    }
}

impl ArrayPayload<i8> for Vec<i8> {
    fn array_payload(self) -> Vec<i8> {
        self
    }
}

impl ArrayPayload<i32> for Vec<i32> {
    fn array_payload(self) -> Vec<i32> {
        self
    }
}

impl ArrayPayload<i64> for Vec<i64> {
    fn array_payload(self) -> Vec<i64> {
        self
    }
}

impl ArrayPayload<i8> for Vec<u8> {
    fn array_payload(self) -> Vec<i8> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i32> for Vec<u32> {
    fn array_payload(self) -> Vec<i32> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i64> for Vec<u64> {
    fn array_payload(self) -> Vec<i64> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i8> for &Vec<i8> {
    fn array_payload(self) -> Vec<i8> {
        self.clone()
    }
}

impl ArrayPayload<i32> for &Vec<i32> {
    fn array_payload(self) -> Vec<i32> {
        self.clone()
    }
}

impl ArrayPayload<i64> for &Vec<i64> {
    fn array_payload(self) -> Vec<i64> {
        self.clone()
    }
}

impl ArrayPayload<i8> for &Vec<u8> {
    fn array_payload(self) -> Vec<i8> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i32> for &Vec<u32> {
    fn array_payload(self) -> Vec<i32> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i64> for &Vec<u64> {
    fn array_payload(self) -> Vec<i64> {
        self.as_slice().array_payload()
    }
}

impl ArrayPayload<i8> for &[i8] {
    fn array_payload(self) -> Vec<i8> {
        self.into()
    }
}

impl ArrayPayload<i32> for &[i32] {
    fn array_payload(self) -> Vec<i32> {
        self.into()
    }
}

impl ArrayPayload<i64> for &[i64] {
    fn array_payload(self) -> Vec<i64> {
        self.into()
    }
}

impl ArrayPayload<i8> for &[u8] {
    fn array_payload(self) -> Vec<i8> {
        reinterpret_slice!(self, u8 => i8).into()
    }
}

impl ArrayPayload<i32> for &[u32] {
    fn array_payload(self) -> Vec<i32> {
        reinterpret_slice!(self, u32 => i32).into()
    }
}

impl ArrayPayload<i64> for &[u64] {
    fn array_payload(self) -> Vec<i64> {
        reinterpret_slice!(self, u64 => i64).into()
    }
}

impl<const N: usize> ArrayPayload<i8> for [i8; N] {
    fn array_payload(self) -> Vec<i8> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i32> for [i32; N] {
    fn array_payload(self) -> Vec<i32> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i64> for [i64; N] {
    fn array_payload(self) -> Vec<i64> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i8> for [u8; N] {
    fn array_payload(self) -> Vec<i8> {
        reinterpret_array!(self, u8 => i8).into()
    }
}

impl<const N: usize> ArrayPayload<i32> for [u32; N] {
    fn array_payload(self) -> Vec<i32> {
        reinterpret_array!(self, u32 => i32).into()
    }
}

impl<const N: usize> ArrayPayload<i64> for [u64; N] {
    fn array_payload(self) -> Vec<i64> {
        reinterpret_array!(self, u64 => i64).into()
    }
}

impl<const N: usize> ArrayPayload<i8> for &[i8; N] {
    fn array_payload(self) -> Vec<i8> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i32> for &[i32; N] {
    fn array_payload(self) -> Vec<i32> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i64> for &[i64; N] {
    fn array_payload(self) -> Vec<i64> {
        self.into()
    }
}

impl<const N: usize> ArrayPayload<i8> for &[u8; N] {
    fn array_payload(self) -> Vec<i8> {
        reinterpret_array_ref!(self, u8 => i8).into()
    }
}

impl<const N: usize> ArrayPayload<i32> for &[u32; N] {
    fn array_payload(self) -> Vec<i32> {
        reinterpret_array_ref!(self, u32 => i32).into()
    }
}

impl<const N: usize> ArrayPayload<i64> for &[u64; N] {
    fn array_payload(self) -> Vec<i64> {
        reinterpret_array_ref!(self, u64 => i64).into()
    }
}

impl ArrayPayload<i8> for String {
    fn array_payload(self) -> Vec<i8> {
        self.as_bytes().array_payload()
    }
}

impl ArrayPayload<i8> for &String {
    fn array_payload(self) -> Vec<i8> {
        self.as_bytes().array_payload()
    }
}

impl ArrayPayload<i8> for &str {
    fn array_payload(self) -> Vec<i8> {
        self.as_bytes().array_payload()
    }
}

macro_rules! void {
    ($l:tt, $r:tt) => {
        $l
    };
}

macro_rules! from_tuple {
    ($t:ty, $b:ty, $($n:tt),+) => {
        impl ArrayPayload<$b> for ($(void!($t, $n)),+) {
            fn array_payload(self) -> Vec<$b> {
                let arr = [$(self.$n),+];
                arr.array_payload()
            }
        }
    };
}

macro_rules! from_tuples {
    ($t:ty, $b:ty) => {
        from_tuple!($t, $b, 0, 1);
        from_tuple!($t, $b, 0, 1, 2);
        from_tuple!($t, $b, 0, 1, 2, 3);
        from_tuple!($t, $b, 0, 1, 2, 3, 4);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7, 8);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        from_tuple!($t, $b, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
    };
}

from_tuples!(i8, i8);
from_tuples!(u8, i8);
from_tuples!(i32, i32);
from_tuples!(u32, i32);
from_tuples!(i64, i64);
from_tuples!(u64, i64);

macro_rules! from {
    ($type:ty, $fn:ident, $kind:literal $(, $n:ident)?) => {
        #[doc = concat!(
            "Converts from a [`", stringify!($type), "`] ",
            "into ", $kind, "."
        )]
        impl$(<const $n: usize>)? From<$type> for Tag {
            fn from(value: $type) -> Self {
                Tag::$fn(value)
            }
        }
    };
}

macro_rules! froms {
    ($ty:ident, $fn:ident, $kind:literal) => {
        from!(Vec<$ty>, $fn, $kind);
        from!(&Vec<$ty>, $fn, $kind);
        from!(&[$ty], $fn, $kind);
        from!([$ty; N], $fn, $kind, N);
        from!(&[$ty; N], $fn, $kind, N);
    };
}

froms!(i8, byte_array, "a [byte array tag][crate::tag::Tag::ByteArray]");
froms!(u8, byte_array, "a [byte array tag][crate::tag::Tag::ByteArray]");
froms!(i32, int_array, "an [int array tag][crate::tag::Tag::IntArray]");
froms!(u32, int_array, "an [int array tag][crate::tag::Tag::IntArray]");
froms!(i64, long_array, "a [long array tag][crate::tag::Tag::LongArray]");
froms!(u64, long_array, "a [long array tag][crate::tag::Tag::LongArray]");
