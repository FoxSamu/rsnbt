use std::fmt::Display;

/// An NBT Tag ID. Each tag can be identified using an 8-bit ID. This
/// ID is the first byte in the encoded format of every tag, and it
/// determines the expected payload format.
/// 
/// The tag ID's are as follows
/// 
/// | ID   | Tag                                      |
/// |------|------------------------------------------|
/// | `1`  | [Byte][crate::tag::Tag::Byte]            |
/// | `2`  | [Short][crate::tag::Tag::Short]          |
/// | `3`  | [Int][crate::tag::Tag::Int]              |
/// | `4`  | [Long][crate::tag::Tag::Long]            |
/// | `5`  | [Float][crate::tag::Tag::Float]          |
/// | `6`  | [Double][crate::tag::Tag::Double]        |
/// | `7`  | [Byte Array][crate::tag::Tag::ByteArray] |
/// | `8`  | [String][crate::tag::Tag::String]        |
/// | `9`  | [List][crate::tag::Tag::List]            |
/// | `10` | [Compound][crate::tag::Tag::Compound]    |
/// | `11` | [Int Array][crate::tag::Tag::IntArray]   |
/// | `12` | [Long Array][crate::tag::Tag::LongArray] |
/// 
/// This is the ID that identifies them when reading binary NBT data.
/// The ID `0` is reserved for the End tag, although this is not really
/// a tag but rather a marker. This ID is not included in [`TagId`].
/// 
/// The enum discriminants of [`TagId`] match the ID byte, though the
/// recommended way to obtain a [`TagId`]'s numeric ID is via [`TagId::byte`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum TagId {
    /// Identifies a [byte][crate::tag::Tag::Byte] tag.
    /// The numerical ID of a byte tag is `1`.
    Byte = 1,

    /// Identifies a [short][crate::tag::Tag::Short] tag.
    /// The numerical ID of a short tag is `2`.
    Short = 2,

    /// Identifies an [int][crate::tag::Tag::Int] tag.
    /// The numerical ID of an int tag is `3`.
    Int = 3,

    /// Identifies a [long][crate::tag::Tag::Long] tag.
    /// The numerical ID of a long tag is `4`.
    Long = 4,

    /// Identifies a [float][crate::tag::Tag::Float] tag.
    /// The numerical ID of a float tag is `5`.
    Float = 5,

    /// Identifies a [double][crate::tag::Tag::Double] tag.
    /// The numerical ID of a double tag is `6`.
    Double = 6,

    /// Identifies a [string][crate::tag::Tag::String] tag.
    /// The numerical ID of a string tag is `8`.
    String = 8,

    /// Identifies a [byte array][crate::tag::Tag::ByteArray] tag.
    /// The numerical ID of a byte array tag is `7`.
    ByteArray = 7,

    /// Identifies an [int array][crate::tag::Tag::IntArray] tag.
    /// The numerical ID of an int array tag is `11`.
    IntArray = 11,

    /// Identifies a [long array][crate::tag::Tag::LongArray] tag.
    /// The numerical ID of a long array tag is `12`.
    LongArray = 12,

    /// Identifies a [list][crate::tag::Tag::List] tag.
    /// The numerical ID of a list tag is `9`.
    List = 9,

    /// Identifies a [compound][crate::tag::Tag::Compound] tag.
    /// The numerical ID of a compound tag is `10`.
    Compound = 10,
}

impl TagId {
    /// The maximum possible tag ID byte.
    pub const MAX: u8 = 12;

    /// The ID of the End tag or marker. Tags with this ID
    /// are not valid and instead indicate the end of a compound
    /// or the type of an empty list.
    pub const END: u8 = 0;

    /// The ID of the [byte][crate::tag::Tag::Byte] tag: `1`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::BYTE, TagId::Byte.byte());
    /// ```
    pub const BYTE: u8 = 1;

    /// The ID of the [short][crate::tag::Tag::Short] tag: `2`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::SHORT, TagId::Short.byte());
    /// ```
    pub const SHORT: u8 = 2;

    /// The ID of the [int][crate::tag::Tag::Int] tag: `3`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::INT, TagId::Int.byte());
    /// ```
    pub const INT: u8 = 3;

    /// The ID of the [long][crate::tag::Tag::Long] tag: `4`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::LONG, TagId::Long.byte());
    /// ```
    pub const LONG: u8 = 4;

    /// The ID of the [float][crate::tag::Tag::Float] tag: `5`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::FLOAT, TagId::Float.byte());
    /// ```
    pub const FLOAT: u8 = 5;

    /// The ID of the [double][crate::tag::Tag::Double] tag: `6`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::DOUBLE, TagId::Double.byte());
    /// ```
    pub const DOUBLE: u8 = 6;

    /// The ID of the [string][crate::tag::Tag::String] tag: `8`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::STRING, TagId::String.byte());
    /// ```
    pub const STRING: u8 = 8;

    /// The ID of the [byte array][crate::tag::Tag::ByteArray] tag: `7`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::BYTE_ARRAY, TagId::ByteArray.byte());
    /// ```
    pub const BYTE_ARRAY: u8 = 7;

    /// The ID of the [int array][crate::tag::Tag::IntArray] tag: `11`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::INT_ARRAY, TagId::IntArray.byte());
    /// ```
    pub const INT_ARRAY: u8 = 11;

    /// The ID of the [long array][crate::tag::Tag::LongArray] tag: `12`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::LONG_ARRAY, TagId::LongArray.byte());
    /// ```
    pub const LONG_ARRAY: u8 = 12;

    /// The ID of the [list][crate::tag::Tag::List] tag: `9`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::LIST, TagId::List.byte());
    /// ```
    pub const LIST: u8 = 9;

    /// The ID of the [compound][crate::tag::Tag::Compound] tag: `10`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(TagId::COMPOUND, TagId::Compound.byte());
    /// ```
    pub const COMPOUND: u8 = 10;

    /// Returns the numeric ID of a [`TagId`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(5, TagId::Float.byte());
    /// assert_eq!(9, TagId::List.byte());
    /// ```
    pub const fn byte(&self) -> u8 {
        match self {
            Self::Byte => Self::BYTE,
            Self::Short => Self::SHORT,
            Self::Int => Self::INT,
            Self::Long => Self::LONG,
            Self::Float => Self::FLOAT,
            Self::Double => Self::DOUBLE,
            Self::String => Self::STRING,
            Self::ByteArray => Self::BYTE_ARRAY,
            Self::IntArray => Self::INT_ARRAY,
            Self::LongArray => Self::LONG_ARRAY,
            Self::List => Self::LIST,
            Self::Compound => Self::COMPOUND,
        }
    }

    /// Returns the [`TagId`] corresponding to the given numeric ID.
    ///
    /// When the given ID is `0` or is greater than [`TagId::MAX`],
    /// then this function returns [`None`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagId;
    /// 
    /// assert_eq!(Some(TagId::Float), TagId::from_byte(5));
    /// assert_eq!(Some(TagId::List), TagId::from_byte(9));
    /// 
    /// assert_eq!(None, TagId::from_byte(0));  // end tag
    /// assert_eq!(None, TagId::from_byte(38)); // invalid tag id
    /// ```
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

/// An implementation of [`Display`] for [`TagId`]. The displayed value
/// is simply the name of the enum constant.
/// 
/// # Examples
/// ```
/// use rsnbt::tag::TagId;
/// 
/// assert_eq!(String::from("String"), format!("{}", TagId::String));
/// ```
impl Display for TagId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => write!(f, "Byte"),
            Self::Short => write!(f, "Short"),
            Self::Int => write!(f, "Int"),
            Self::Long => write!(f, "Long"),
            Self::Float => write!(f, "Float"),
            Self::Double => write!(f, "Double"),
            Self::String => write!(f, "String"),
            Self::ByteArray => write!(f, "ByteArray"),
            Self::IntArray => write!(f, "IntArray"),
            Self::LongArray => write!(f, "LongArray"),
            Self::List => write!(f, "List"),
            Self::Compound => write!(f, "Compound"),
        }
    }
}
