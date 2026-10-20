use super::*;
use super::cast::*;

pub trait NumPayload : Copy {
    fn byte_payload(self) -> i8;
    fn short_payload(self) -> i16;
    fn int_payload(self) -> i32;
    fn long_payload(self) -> i64;
    fn float_payload(self) -> f32;
    fn double_payload(self) -> f64;
}

impl Tag {
    pub fn byte<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Byte(v.byte_payload())
    }

    pub fn short<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Short(v.short_payload())
    }

    pub fn int<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Int(v.int_payload())
    }

    pub fn long<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Long(v.long_payload())
    }

    pub fn float<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Float(v.float_payload())
    }

    pub fn double<T>(v: T) -> Tag
    where
        T: NumPayload,
    {
        Tag::Double(v.double_payload())
    }
}

pub(crate) const NUMERIC_IDS: &[TagId] = &[
    TagId::Byte,
    TagId::Short,
    TagId::Int,
    TagId::Long,
    TagId::Float,
    TagId::Double
];

macro_rules! as_num {
    ($name:ident, $type:ty) => {
        impl Tag {
            pub fn $name(&self) -> Result<$type, TagMismatchError> {
                match self {
                    Tag::Byte(v) => Ok(*v as $type),
                    Tag::Short(v) => Ok(*v as $type),
                    Tag::Int(v) => Ok(*v as $type),
                    Tag::Long(v) => Ok(*v as $type),
                    Tag::Float(v) => Ok(*v as $type),
                    Tag::Double(v) => Ok(*v as $type),
                    _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
                }
            }
        }

        impl TagRef<'_> {
            pub fn $name(&self) -> Result<$type, TagMismatchError> {
                match self {
                    TagRef::Byte(v) => Ok(**v as $type),
                    TagRef::Short(v) => Ok(**v as $type),
                    TagRef::Int(v) => Ok(**v as $type),
                    TagRef::Long(v) => Ok(**v as $type),
                    TagRef::Float(v) => Ok(**v as $type),
                    TagRef::Double(v) => Ok(**v as $type),
                    _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
                }
            }
        }

        impl TagRefMut<'_> {
            pub fn $name(&self) -> Result<$type, TagMismatchError> {
                match self {
                    TagRefMut::Byte(v) => Ok(**v as $type),
                    TagRefMut::Short(v) => Ok(**v as $type),
                    TagRefMut::Int(v) => Ok(**v as $type),
                    TagRefMut::Long(v) => Ok(**v as $type),
                    TagRefMut::Float(v) => Ok(**v as $type),
                    TagRefMut::Double(v) => Ok(**v as $type),
                    _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
                }
            }
        }

        impl NumPayload for $type {
            fn byte_payload(self) -> i8 {
                self as i8
            }

            fn short_payload(self) -> i16 {
                self as i16
            }

            fn int_payload(self) -> i32 {
                self as i32
            }

            fn long_payload(self) -> i64 {
                self as i64
            }

            fn float_payload(self) -> f32 {
                self as f32
            }

            fn double_payload(self) -> f64 {
                self as f64
            }
        }
    };
}

macro_rules! from_num {
    ($type:ty, $n:ident, $conv:expr) => {
        impl From<$type> for Tag {
            fn from($n: $type) -> Self {
                $conv
            }
        }
    };
}

as_num!(as_i8, i8);
as_num!(as_i16, i16);
as_num!(as_i32, i32);
as_num!(as_i64, i64);
as_num!(as_i128, i128);
as_num!(as_u8, u8);
as_num!(as_u16, u16);
as_num!(as_u32, u32);
as_num!(as_u64, u64);
as_num!(as_u128, u128);
as_num!(as_f32, f32);
as_num!(as_f64, f64);
as_num!(as_isize, isize);
as_num!(as_usize, usize);

from_num!(i8, v, Tag::Byte(v));
from_num!(i16, v, Tag::Short(v));
from_num!(i32, v, Tag::Int(v));
from_num!(i64, v, Tag::Long(v));
from_num!(u8, v, Tag::Byte(v as i8));
from_num!(u16, v, Tag::Short(v as i16));
from_num!(u32, v, Tag::Int(v as i32));
from_num!(u64, v, Tag::Long(v as i64));
from_num!(f32, v, Tag::Float(v));
from_num!(f64, v, Tag::Double(v));
from_num!(isize, v, Tag::Long(v as i64));
from_num!(usize, v, Tag::Long(v as i64));



macro_rules! ter {
    ($v:expr, $true:expr, $false:expr) => {
        if $v { $true } else { $false }
    };
}


impl Tag {
    pub fn as_bool(&self) -> Result<bool, TagMismatchError> {
        match self {
            Tag::Byte(v) => Ok(*v != 0),
            Tag::Short(v) => Ok(*v != 0),
            Tag::Int(v) => Ok(*v != 0),
            Tag::Long(v) => Ok(*v != 0),
            Tag::Float(v) => Ok(*v != 0.0),
            Tag::Double(v) => Ok(*v != 0.0),
            _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
        }
    }
}

impl TagRef<'_> {
    pub fn as_bool(&self) -> Result<bool, TagMismatchError> {
        match self {
            TagRef::Byte(v) => Ok(**v != 0),
            TagRef::Short(v) => Ok(**v != 0),
            TagRef::Int(v) => Ok(**v != 0),
            TagRef::Long(v) => Ok(**v != 0),
            TagRef::Float(v) => Ok(**v != 0.0),
            TagRef::Double(v) => Ok(**v != 0.0),
            _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
        }
    }
}

impl TagRefMut<'_> {
    pub fn as_bool(&self) -> Result<bool, TagMismatchError> {
        match self {
            TagRefMut::Byte(v) => Ok(**v != 0),
            TagRefMut::Short(v) => Ok(**v != 0),
            TagRefMut::Int(v) => Ok(**v != 0),
            TagRefMut::Long(v) => Ok(**v != 0),
            TagRefMut::Float(v) => Ok(**v != 0.0),
            TagRefMut::Double(v) => Ok(**v != 0.0),
            _ => Err(TagMismatchError::new_multi(NUMERIC_IDS, self.id())),
        }
    }
}

impl NumPayload for bool {
    fn byte_payload(self) -> i8 {
        ter!(self, 1, 0)
    }

    fn short_payload(self) -> i16 {
        ter!(self, 1, 0)
    }

    fn int_payload(self) -> i32 {
        ter!(self, 1, 0)
    }

    fn long_payload(self) -> i64 {
        ter!(self, 1, 0)
    }

    fn float_payload(self) -> f32 {
        ter!(self, 1.0, 0.0)
    }

    fn double_payload(self) -> f64 {
        ter!(self, 1.0, 0.0)
    }
}

impl From<bool>for Tag {
    fn from(v: bool) -> Self {
        Tag::byte(v)
    }
}
