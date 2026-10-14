use crate::util::unsafe_ops::*;
use super::*;
use crate::tag::Tag::Byte;

/// An NBT tag. The NBT (Named Binary Tag) format structures data similarly to JSON, but is inherently meant
/// for binary serialization rather than something human-readable. However, NBT comes with a
/// human-readable form, SNBT (String Named Binary Tag).
///
/// A tag is one of 12 types:
/// - [Tag::Byte]: An 8-bit integer, often interpreted as signed or as a boolean.
/// - [Tag::Short]: A 16-bit integer, often interpreted as signed.
/// - [Tag::Int]: A 32-bit integer, often interpreted as signed.
/// - [Tag::Long]: A 64-bit integer, often interpreted as signed.
/// - [Tag::Float]: A 32-bit floating point value.
/// - [Tag::Double]: A 64-bit floating point value.
/// - [Tag::String]: A UTF-8 string.
/// - [Tag::ByteArray]: An array of 8-bit integers, often interpreted as some raw binary data.
/// - [Tag::IntArray]: An array of 32-bit integers, often interpreted as coordinates or UUIDs.
/// - [Tag::LongArray]: An array of 64-bit integers, often interpreted as tightly packed binary data.
/// - [Tag::List]: A list tags of the same type, without a name.
/// - [Tag::Compound]: A collection of named tags, almost always interpreted as a mapping from names to tags.
///
/// Within binary representation, all tags have names, unless they appear inside a `List` tag. However,
/// the name is omitted in [Tag] and instead tags are associated names using [TagMap].
///
/// NBT was designed for and is used extensively by Minecraft. For more information about
/// NBT, see [https://minecraft.wiki/w/NBT_format].
///
/// ## Implementation notes
/// - While primitive tags could easily implement [Copy], because a tag can be more than a primitive, [Copy]
///   is not implemented.
/// - Due to tags being able to contain floats, [Tag] cannot implement [Eq].
#[derive(Clone, PartialEq, Debug)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(TagList),
    Compound(TagMap),
}

impl Tag {
    /// Returns the [TagId] of a [Tag].
    ///
    /// ```
    /// use crate::tag::*;
    ///
    /// let string = Tag::String(String::from("Hello world!"));
    /// assert_eq!(TagId::String, string.id());
    /// ```
    pub fn id(&self) -> TagId {
        match self {
            Tag::Byte(_) => TagId::Byte,
            Tag::Short(_) => TagId::Short,
            Tag::Int(_) => TagId::Int,
            Tag::Long(_) => TagId::Long,
            Tag::Float(_) => TagId::Float,
            Tag::Double(_) => TagId::Double,
            Tag::String(_) => TagId::String,
            Tag::ByteArray(_) => TagId::ByteArray,
            Tag::IntArray(_) => TagId::IntArray,
            Tag::LongArray(_) => TagId::LongArray,
            Tag::List(_) => TagId::List,
            Tag::Compound(_) => TagId::Compound,
        }
    }

    /// Returns the 8-bit binary ID of this tag's type.
    /// This is analogous to calling `id().byte()`.
    ///
    /// ```
    /// use crate::tag::Tag;
    /// use crate::tag::TagId;
    ///
    /// let string = Tag::String(String::from("Hello world!"));
    /// assert_eq!(8, string.id_byte());
    /// ```
    pub fn id_byte(&self) -> u8 {
        self.id().byte()
    }
}

impl Default for Tag {
    fn default() -> Self {
        Byte(0)
    }
}

// ===========================================================================================================
//
//  HELPER MACROS AND FUNCTIONS
//
// ===========================================================================================================

macro_rules! delegated_from {
    ({$($generics:tt)*}, $from:ty, $via:ty) => {
        impl$($generics)* From<$from> for Tag {
            fn from(value: $from) -> Self {
                From::<$via>::from(value)
            }
        }
    };

    ({$($generics:tt)*}, $from:ty, $via:ty, |$name:ident| $expr:expr) => {
        impl$($generics)* From<$from> for Tag {
            fn from($name: $from) -> Self {
                From::<$via>::from($expr)
            }
        }
    };
}

// ===========================================================================================================
//
//  CONVERSION OF NUMBERS
//
// ===========================================================================================================

/// A trait for all types that can convert into numeric tags. This is implemented on all numeric types native
/// to Rust, allowing things like `4.into_byte_tag()`.
///
/// ```
/// use crate::tag::*;
///
/// assert_eq!(Tag::Long(3415), 3415.into_long_tag());
/// assert_eq!(Tag::Byte(88), 88.into_byte_tag());
/// ```
pub trait IntoNumericTag {
    fn into_byte_tag_value(self) -> i8;
    fn into_short_tag_value(self) -> i16;
    fn into_int_tag_value(self) -> i32;
    fn into_long_tag_value(self) -> i64;
    fn into_float_tag_value(self) -> f32;
    fn into_double_tag_value(self) -> f64;
}

impl Tag {
    pub fn byte_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Byte(v.into_byte_tag_value())
    }

    pub fn short_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Short(v.into_short_tag_value())
    }

    pub fn int_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Int(v.into_int_tag_value())
    }

    pub fn long_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Long(v.into_long_tag_value())
    }

    pub fn float_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Float(v.into_float_tag_value())
    }

    pub fn double_from<T>(v: T) -> Tag
    where
        T: IntoNumericTag,
    {
        Tag::Double(v.into_double_tag_value())
    }
}

macro_rules! convert_number_impl {
    ($name:ident, $type:ty) => {
        impl Tag {
            pub fn $name(&self) -> Option<$type> {
                match self {
                    &Tag::Byte(v) => Some(v as $type),
                    &Tag::Short(v) => Some(v as $type),
                    &Tag::Int(v) => Some(v as $type),
                    &Tag::Long(v) => Some(v as $type),
                    &Tag::Float(v) => Some(v as $type),
                    &Tag::Double(v) => Some(v as $type),
                    _ => None,
                }
            }
        }

        impl IntoNumericTag for $type {
            fn into_byte_tag_value(self) -> i8 {
                self as i8
            }

            fn into_short_tag_value(self) -> i16 {
                self as i16
            }

            fn into_int_tag_value(self) -> i32 {
                self as i32
            }

            fn into_long_tag_value(self) -> i64 {
                self as i64
            }

            fn into_float_tag_value(self) -> f32 {
                self as f32
            }

            fn into_double_tag_value(self) -> f64 {
                self as f64
            }
        }
    };

    ($name:ident, $type:ty, $n:ident, $r:expr, $rf:expr, $b:expr, $bf:expr) => {
        impl Tag {
            pub fn $name(&self) -> Option<$type> {
                match self {
                    &Tag::Byte($n) => Some($r),
                    &Tag::Short($n) => Some($r),
                    &Tag::Int($n) => Some($r),
                    &Tag::Long($n) => Some($r),
                    &Tag::Float($n) => Some($rf),
                    &Tag::Double($n) => Some($rf),
                    _ => None,
                }
            }
        }

        impl IntoNumericTag for $type {
            fn into_byte_tag_value(self) -> i8 {
                let $n = self;
                $b
            }

            fn into_short_tag_value(self) -> i16 {
                let $n = self;
                $b
            }

            fn into_int_tag_value(self) -> i32 {
                let $n = self;
                $b
            }

            fn into_long_tag_value(self) -> i64 {
                let $n = self;
                $b
            }

            fn into_float_tag_value(self) -> f32 {
                let $n = self;
                $bf
            }

            fn into_double_tag_value(self) -> f64 {
                let $n = self;
                $bf
            }
        }
    };
}

macro_rules! simple_numeric_borrows {
    ($type:ty, $name:ident, $name_mut:ident, $into_name:ident, $kind:ident) => {
        impl Tag {
            #[doc = concat!(
                                                "If this [Tag] is a [Tag::", stringify!($kind), "], returns an immutable ",
                                                "reference to the [", stringify!($type), "] representing the tag's value."
                                            )]
            ///
            /// ```
            /// use crate::tag::*;
            #[doc = concat!("let tag = Tag::", stringify!($kind), "(5);")]
            #[doc = concat!("assert_eq!(5, *tag.", stringify!($name), "());")]
            /// ```
            pub fn $name(&self) -> Option<&$type> {
                if let Tag::$kind(v) = self {
                    Some(v)
                } else {
                    None
                }
            }

            #[doc = concat!(
                                                "If this [Tag] is a [Tag::", stringify!($kind), "], returns a mutable ",
                                                "reference to the [", stringify!($type), "] representing the tag's value, ",
                                                "through which the value of the tag can be directly modified."
                                            )]
            ///
            /// ```
            /// use crate::tag::*;
            #[doc = concat!("let mut tag = Tag::", stringify!($kind), "(5);")]
            #[doc = concat!("let r = tag.", stringify!($name_mut), "();")]
            #[doc = concat!("assert_eq!(5, *r);")]
            ///
            #[doc = concat!("*r = 42;")]
            #[doc = concat!("assert_eq!(Tag::", stringify!($kind), "(42), tag);")]
            /// ```
            pub fn $name_mut(&mut self) -> Option<&mut $type> {
                if let Tag::$kind(v) = self {
                    Some(v)
                } else {
                    None
                }
            }

            #[doc = concat!(
                                                "If this [Tag] is a [Tag::", stringify!($kind), "], returns the ",
                                                "[", stringify!($type), "] representing the tag's value."
                                            )]
            ///
            /// ```
            /// use crate::tag::*;
            #[doc = concat!("let tag = Tag::", stringify!($kind), "(5);")]
            #[doc = concat!("assert_eq!(5, tag.", stringify!($into_name), "());")]
            /// ```
            pub fn $into_name(self) -> Option<$type> {
                if let Tag::$kind(v) = self {
                    Some(v)
                } else {
                    None
                }
            }
        }
    };
}

macro_rules! numeric_from_try_into {
    ($type:ty, $n:ident, $conv:expr, $conv_back:expr, $conv_backf:expr) => {
        impl From<$type> for Tag {
            fn from($n: $type) -> Self {
                $conv
            }
        }

        impl TryFrom<Tag> for $type {
            type Error = Tag;

            fn try_from(value: Tag) -> Result<Self, Self::Error> {
                match value {
                    Tag::Byte($n) => Ok($conv_back),
                    Tag::Short($n) => Ok($conv_back),
                    Tag::Int($n) => Ok($conv_back),
                    Tag::Long($n) => Ok($conv_back),
                    Tag::Float($n) => Ok($conv_backf),
                    Tag::Double($n) => Ok($conv_backf),
                    t => Err(t),
                }
            }
        }
    };
}

convert_number_impl!(as_i8, i8);
convert_number_impl!(as_i16, i16);
convert_number_impl!(as_i32, i32);
convert_number_impl!(as_i64, i64);
convert_number_impl!(as_i128, i128);
convert_number_impl!(as_u8, u8);
convert_number_impl!(as_u16, u16);
convert_number_impl!(as_u32, u32);
convert_number_impl!(as_u64, u64);
convert_number_impl!(as_u128, u128);
convert_number_impl!(as_f32, f32);
convert_number_impl!(as_f64, f64);
convert_number_impl!(as_isize, isize);
convert_number_impl!(as_usize, usize);
convert_number_impl!(
    as_bool,
    bool,
    v,
    v != 0,
    v != 0.0,
    if v { 1 } else { 0 },
    if v { 1.0 } else { 0.0 }
);

numeric_from_try_into!(i8, v, Tag::Byte(v), v as i8, v as i8);
numeric_from_try_into!(i16, v, Tag::Short(v), v as i16, v as i16);
numeric_from_try_into!(i32, v, Tag::Int(v), v as i32, v as i32);
numeric_from_try_into!(i64, v, Tag::Long(v), v as i64, v as i64);
numeric_from_try_into!(u8, v, Tag::Byte(v as i8), v as u8, v as u8);
numeric_from_try_into!(u16, v, Tag::Short(v as i16), v as u16, v as u16);
numeric_from_try_into!(u32, v, Tag::Int(v as i32), v as u32, v as u32);
numeric_from_try_into!(u64, v, Tag::Long(v as i64), v as u64, v as u64);
numeric_from_try_into!(f32, v, Tag::Float(v), v as f32, v as f32);
numeric_from_try_into!(f64, v, Tag::Double(v), v as f64, v as f64);
numeric_from_try_into!(bool, v, Tag::Byte(if v { 1 } else { 0 }), v != 0, v != 0.0);
numeric_from_try_into!(isize, v, Tag::Long(v as i64), v as isize, v as isize);
numeric_from_try_into!(usize, v, Tag::Long(v as i64), v as usize, v as usize);

simple_numeric_borrows!(i8, byte, byte_mut, into_byte, Byte);
simple_numeric_borrows!(i16, short, short_mut, into_short, Short);
simple_numeric_borrows!(i32, int, int_mut, into_int, Int);
simple_numeric_borrows!(i64, long, long_mut, into_long, Long);
simple_numeric_borrows!(f32, float, float_mut, into_float, Float);
simple_numeric_borrows!(f64, double, double_mut, into_double, Double);

// ===========================================================================================================
//
//  CONVERSION OF STRINGS
//
// ===========================================================================================================

pub trait IntoStringTag {
    fn into_string_tag_value(self) -> String;
}

impl IntoStringTag for String {
    fn into_string_tag_value(self) -> String {
        self
    }
}

impl IntoStringTag for &str {
    fn into_string_tag_value(self) -> String {
        String::from(self)
    }
}

impl Tag {
    pub fn string(&self) -> Option<&String> {
        if let Tag::String(str) = self {
            Some(str)
        } else {
            None
        }
    }

    pub fn string_mut(&mut self) -> Option<&mut String> {
        if let Tag::String(str) = self {
            Some(str)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        if let Tag::String(str) = self {
            Some(str)
        } else {
            None
        }
    }

    pub fn str(&self) -> Option<&str> {
        if let Tag::String(str) = self {
            Some(str.as_str())
        } else {
            None
        }
    }

    pub fn string_from<T>(v: T) -> Tag
    where
        T: IntoStringTag,
    {
        Tag::String(v.into_string_tag_value())
    }
}

impl From<String> for Tag {
    fn from(v: String) -> Self {
        Tag::String(v)
    }
}

impl From<&str> for Tag {
    fn from(v: &str) -> Self {
        Tag::String(String::from(v))
    }
}

impl TryFrom<Tag> for String {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::String(v) => Ok(v),
            t => Err(t),
        }
    }
}

// ===========================================================================================================
//
//  CONVERSION OF ARRAYS
//
// ===========================================================================================================

pub trait IntoByteArrayTag {
    fn into_byte_array_tag_value(self) -> Vec<i8>;
}

pub trait IntoIntArrayTag {
    fn into_int_array_tag_value(self) -> Vec<i32>;
}

pub trait IntoLongArrayTag {
    fn into_long_array_tag_value(self) -> Vec<i64>;
}

macro_rules! into_array_tag_impl {
    ($trait:ty, $fn:ident, $ret:ty, {$($generics:tt)*}, $type:ty) => {
        impl$($generics)* $trait for $type {
            fn $fn(self) -> Vec<$ret> {
                Vec::from(self)
            }
        }
    };

    ($trait:ty, $fn:ident, $via:ty => $ret:ty, {$($generics:tt)*}, $type:ty) => {
        impl$($generics)* $trait for $type {
            fn $fn(self) -> Vec<$ret> {
                reinterpret_vec!(Vec::from(self), $via => $ret)
            }
        }
    };
}

into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, u8 => i8, {<const N : usize>}, [u8; N]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, u8 => i8, {<const N : usize>}, &[u8; N]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, u8 => i8, {}, &[u8]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, u8 => i8, {}, Vec<u8>);

into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, i8, {<const N : usize>}, [i8; N]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, i8, {<const N : usize>}, &[i8; N]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, i8, {}, &[i8]);
into_array_tag_impl!(IntoByteArrayTag, into_byte_array_tag_value, i8, {}, Vec<i8>);

into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, u32 => i32, {<const N : usize>}, [u32; N]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, u32 => i32, {<const N : usize>}, &[u32; N]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, u32 => i32, {}, &[u32]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, u32 => i32, {}, Vec<u32>);

into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, i32, {<const N : usize>}, [i32; N]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, i32, {<const N : usize>}, &[i32; N]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, i32, {}, &[i32]);
into_array_tag_impl!(IntoIntArrayTag, into_int_array_tag_value, i32, {}, Vec<i32>);

into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, u64 => i64, {<const N : usize>}, [u64; N]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, u64 => i64, {<const N : usize>}, &[u64; N]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, u64 => i64, {}, &[u64]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, u64 => i64, {}, Vec<u64>);

into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, i64, {<const N : usize>}, [i64; N]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, i64, {<const N : usize>}, &[i64; N]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, i64, {}, &[i64]);
into_array_tag_impl!(IntoLongArrayTag, into_long_array_tag_value, i64, {}, Vec<i64>);

impl Tag {
    pub fn byte_array_from<T>(v: T) -> Tag
    where
        T: IntoByteArrayTag,
    {
        Tag::ByteArray(v.into_byte_array_tag_value())
    }

    pub fn int_array_from<T>(v: T) -> Tag
    where
        T: IntoIntArrayTag,
    {
        Tag::IntArray(v.into_int_array_tag_value())
    }

    pub fn long_array_from<T>(v: T) -> Tag
    where
        T: IntoLongArrayTag,
    {
        Tag::LongArray(v.into_long_array_tag_value())
    }
}

delegated_from!({<const N: usize>}, [u8; N], &[u8; N], |value| &value);
delegated_from!({<const N: usize>}, [u32; N], &[u32; N], |value| &value);
delegated_from!({<const N: usize>}, [u64; N], &[u64; N], |value| &value);

delegated_from!({<const N: usize>}, &[u8; N], &[u8]);
delegated_from!({<const N: usize>}, &[u32; N], &[u32]);
delegated_from!({<const N: usize>}, &[u64; N], &[u64]);

delegated_from!({}, &[u8], Vec<i8>, |value| Vec::from(reinterpret_slice!(value, u8 => i8)));
delegated_from!({}, &[u32], Vec<i32>, |value| Vec::from(reinterpret_slice!(value, u32 => i32)));
delegated_from!({}, &[u64], Vec<i64>, |value| Vec::from(reinterpret_slice!(value, u64 => i64)));

delegated_from!({<const N: usize>}, [i8; N], &[i8; N], |value| &value);
delegated_from!({<const N: usize>}, [i32; N], &[i32; N], |value| &value);
delegated_from!({<const N: usize>}, [i64; N], &[i64; N], |value| &value);

delegated_from!({<const N: usize>}, &[i8; N], &[i8]);
delegated_from!({<const N: usize>}, &[i32; N], &[i32]);
delegated_from!({<const N: usize>}, &[i64; N], &[i64]);

delegated_from!({}, &[i8], Vec<i8>, |value| Vec::from(value));
delegated_from!({}, &[i32], Vec<i32>, |value| Vec::from(value));
delegated_from!({}, &[i64], Vec<i64>, |value| Vec::from(value));

delegated_from!({}, Vec<u8>, Vec<i8>, |value| reinterpret_vec!(value, u8 => i8));
delegated_from!({}, Vec<u32>, Vec<i32>, |value| reinterpret_vec!(value, u32 => i32));
delegated_from!({}, Vec<u64>, Vec<i64>, |value| reinterpret_vec!(value, u64 => i64));

impl From<Vec<i8>> for Tag {
    fn from(value: Vec<i8>) -> Self {
        Tag::ByteArray(value)
    }
}

impl From<Vec<i32>> for Tag {
    fn from(value: Vec<i32>) -> Self {
        Tag::IntArray(value)
    }
}

impl From<Vec<i64>> for Tag {
    fn from(value: Vec<i64>) -> Self {
        Tag::LongArray(value)
    }
}

impl TryFrom<Tag> for Vec<i8> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::ByteArray(v) => Ok(v),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for Vec<u8> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::ByteArray(v) => Ok(reinterpret_vec!(v, i8 => u8)),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for Vec<i32> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::IntArray(v) => Ok(v),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for Vec<u32> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::IntArray(v) => Ok(reinterpret_vec!(v, i32 => u32)),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for Vec<i64> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::LongArray(v) => Ok(v),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for Vec<u64> {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::LongArray(v) => Ok(reinterpret_vec!(v, i64 => u64)),
            t => Err(t),
        }
    }
}

// ===========================================================================================================
//
//  CONVERSION OF LISTS AND COMPOUNDS
//
// ===========================================================================================================

pub trait IntoListTag {
    fn into_list_tag_value(self) -> TagList;
}

pub trait IntoCompoundTag {
    fn into_compound_tag_value(self) -> TagMap;
}

impl IntoListTag for TagList {
    fn into_list_tag_value(self) -> TagList {
        self
    }
}

impl IntoCompoundTag for TagMap {
    fn into_compound_tag_value(self) -> TagMap {
        self
    }
}

impl From<TagList> for Tag {
    fn from(value: TagList) -> Self {
        Self::List(value)
    }
}

impl From<TagMap> for Tag {
    fn from(value: TagMap) -> Self {
        Self::Compound(value)
    }
}

impl TryFrom<Tag> for TagList {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::List(v) => Ok(v),
            t => Err(t),
        }
    }
}

impl TryFrom<Tag> for TagMap {
    type Error = Tag;

    fn try_from(value: Tag) -> Result<Self, Self::Error> {
        match value {
            Tag::Compound(v) => Ok(v),
            t => Err(t),
        }
    }
}

// ===========================================================================================================
//
//  CASTING
//
// ===========================================================================================================

impl Tag {
    pub fn try_cast(self, id: TagId) -> Result<Tag, Tag> {
        Ok(match id {
            TagId::Byte => Tag::Byte(match self {
                Tag::Byte(v) => v as i8,
                Tag::Short(v) => v as i8,
                Tag::Int(v) => v as i8,
                Tag::Long(v) => v as i8,
                Tag::Float(v) => v as i8,
                Tag::Double(v) => v as i8,
                tag => Err(tag)?,
            }),

            TagId::Short => Tag::Short(match self {
                Tag::Byte(v) => v as i16,
                Tag::Short(v) => v as i16,
                Tag::Int(v) => v as i16,
                Tag::Long(v) => v as i16,
                Tag::Float(v) => v as i16,
                Tag::Double(v) => v as i16,
                tag => Err(tag)?,
            }),

            TagId::Int => Tag::Int(match self {
                Tag::Byte(v) => v as i32,
                Tag::Short(v) => v as i32,
                Tag::Int(v) => v as i32,
                Tag::Long(v) => v as i32,
                Tag::Float(v) => v as i32,
                Tag::Double(v) => v as i32,
                tag => Err(tag)?,
            }),

            TagId::Long => Tag::Long(match self {
                Tag::Byte(v) => v as i64,
                Tag::Short(v) => v as i64,
                Tag::Int(v) => v as i64,
                Tag::Long(v) => v as i64,
                Tag::Float(v) => v as i64,
                Tag::Double(v) => v as i64,
                tag => Err(tag)?,
            }),

            TagId::Float => Tag::Float(match self {
                Tag::Byte(v) => v as f32,
                Tag::Short(v) => v as f32,
                Tag::Int(v) => v as f32,
                Tag::Long(v) => v as f32,
                Tag::Float(v) => v as f32,
                Tag::Double(v) => v as f32,
                tag => Err(tag)?,
            }),

            TagId::Double => Tag::Double(match self {
                Tag::Byte(v) => v as f64,
                Tag::Short(v) => v as f64,
                Tag::Int(v) => v as f64,
                Tag::Long(v) => v as f64,
                Tag::Float(v) => v as f64,
                Tag::Double(v) => v as f64,
                tag => Err(tag)?,
            }),

            id => {
                if id != self.id() {
                    Err(self)?
                } else {
                    self
                }
            },
        })
    }
}

// ===========================================================================================================
//
//  TESTING
//
// ===========================================================================================================

#[cfg(test)]
mod test {
    use crate::tag::*;

    #[test]
    fn test_conv() {
        let c: u128 = 3415;

        assert_eq!(Tag::Long(3415), Tag::long_from(c));
    }

    #[test]
    fn test_conv_2() {
        assert_eq!(Tag::Long(3415), Tag::long_from(3415));
    }
}
