use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TagId {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    String,
    ByteArray,
    IntArray,
    LongArray,
    List,
    Compound,
}

impl TagId {
    /// The maximum possible tag ID byte.
    pub const MAX: u8 = 12;

    pub const BYTE: u8 = 1;
    pub const SHORT: u8 = 2;
    pub const INT: u8 = 3;
    pub const LONG: u8 = 4;
    pub const FLOAT: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const STRING: u8 = 8;
    pub const BYTE_ARRAY: u8 = 7;
    pub const INT_ARRAY: u8 = 11;
    pub const LONG_ARRAY: u8 = 12;
    pub const LIST: u8 = 9;
    pub const COMPOUND: u8 = 10;

    pub const fn byte(&self) -> u8 {
        match self {
            TagId::Byte => 1,
            TagId::Short => 2,
            TagId::Int => 3,
            TagId::Long => 4,
            TagId::Float => 5,
            TagId::Double => 6,
            TagId::String => 8,
            TagId::ByteArray => 7,
            TagId::IntArray => 11,
            TagId::LongArray => 12,
            TagId::List => 9,
            TagId::Compound => 10,
        }
    }

    pub const fn from_byte(byte: u8) -> Option<TagId> {
        match byte {
            1 => Some(Self::Byte),
            2 => Some(Self::Short),
            3 => Some(Self::Int),
            4 => Some(Self::Long),
            5 => Some(Self::Float),
            6 => Some(Self::Double),
            7 => Some(Self::ByteArray),
            8 => Some(Self::String),
            9 => Some(Self::List),
            10 => Some(Self::Compound),
            11 => Some(Self::IntArray),
            12 => Some(Self::LongArray),
            _ => None,
        }
    }
}

impl Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagId::Byte => write!(f, "Byte"),
            TagId::Short => write!(f, "Short"),
            TagId::Int => write!(f, "Int"),
            TagId::Long => write!(f, "Long"),
            TagId::Float => write!(f, "Float"),
            TagId::Double => write!(f, "Double"),
            TagId::String => write!(f, "String"),
            TagId::ByteArray => write!(f, "ByteArray"),
            TagId::IntArray => write!(f, "IntArray"),
            TagId::LongArray => write!(f, "LongArray"),
            TagId::List => write!(f, "List"),
            TagId::Compound => write!(f, "Compound"),
        }
    }
}
