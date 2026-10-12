use super::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TagRef<'a> {
    Byte(&'a i8),
    Short(&'a i16),
    Int(&'a i32),
    Long(&'a i64),
    Float(&'a f32),
    Double(&'a f64),
    String(&'a String),
    ByteArray(&'a Vec<i8>),
    IntArray(&'a Vec<i32>),
    LongArray(&'a Vec<i64>),
    List(&'a TagList),
    Compound(&'a TagMap),
}

#[derive(PartialEq, Debug)]
pub enum TagRefMut<'a> {
    Byte(&'a mut i8),
    Short(&'a mut i16),
    Int(&'a mut i32),
    Long(&'a mut i64),
    Float(&'a mut f32),
    Double(&'a mut f64),
    String(&'a mut String),
    ByteArray(&'a mut Vec<i8>),
    IntArray(&'a mut Vec<i32>),
    LongArray(&'a mut Vec<i64>),
    List(&'a mut TagList),
    Compound(&'a mut TagMap),
}

macro_rules! tag_ref_impl {
($name:ident $($mut:ident)?) => {


impl<'a> $name<'a> {
    pub fn id(&self) -> TagId {
        match self {
            Self::Byte(_) => TagId::Byte,
            Self::Short(_) => TagId::Short,
            Self::Int(_) => TagId::Int,
            Self::Long(_) => TagId::Long,
            Self::Float(_) => TagId::Float,
            Self::Double(_) => TagId::Double,
            Self::String(_) => TagId::String,
            Self::ByteArray(_) => TagId::ByteArray,
            Self::IntArray(_) => TagId::IntArray,
            Self::LongArray(_) => TagId::LongArray,
            Self::List(_) => TagId::List,
            Self::Compound(_) => TagId::Compound,
        }
    }

    pub fn id_byte(&self) -> u8 {
        self.id().byte()
    }

    pub fn clone_tag(&self) -> Tag {
        match self {
            Self::Byte(v) => Tag::Byte((*v).clone()),
            Self::Short(v) => Tag::Short((*v).clone()),
            Self::Int(v) => Tag::Int((*v).clone()),
            Self::Long(v) => Tag::Long((*v).clone()),
            Self::Float(v) => Tag::Float((*v).clone()),
            Self::Double(v) => Tag::Double((*v).clone()),
            Self::String(v) => Tag::String((*v).clone()),
            Self::ByteArray(v) => Tag::ByteArray((*v).clone()),
            Self::IntArray(v) => Tag::IntArray((*v).clone()),
            Self::LongArray(v) => Tag::LongArray((*v).clone()),
            Self::List(v) => Tag::List((*v).clone()),
            Self::Compound(v) => Tag::Compound((*v).clone()),
        }
    }
}


impl<'a> From<&'a $($mut)? Tag> for $name<'a> {
    fn from(value: &'a $($mut)? Tag) -> Self {
        match value {
            Tag::Byte(v) => Self::Byte(v),
            Tag::Short(v) => Self::Short(v),
            Tag::Int(v) => Self::Int(v),
            Tag::Long(v) => Self::Long(v),
            Tag::Float(v) => Self::Float(v),
            Tag::Double(v) => Self::Double(v),
            Tag::String(v) => Self::String(v),
            Tag::ByteArray(v) => Self::ByteArray(v),
            Tag::IntArray(v) => Self::IntArray(v),
            Tag::LongArray(v) => Self::LongArray(v),
            Tag::List(v) => Self::List(v),
            Tag::Compound(v) => Self::Compound(v),
        }
    }
}

};
}

tag_ref_impl!(TagRef);
tag_ref_impl!(TagRefMut mut);

impl<'a> TagRefMut<'a> {
    pub fn borrow(&'a self) -> TagRef<'a> {
        match self {
            TagRefMut::Byte(v) => TagRef::Byte(v),
            TagRefMut::Short(v) => TagRef::Short(v),
            TagRefMut::Int(v) => TagRef::Int(v),
            TagRefMut::Long(v) => TagRef::Long(v),
            TagRefMut::Float(v) => TagRef::Float(v),
            TagRefMut::Double(v) => TagRef::Double(v),
            TagRefMut::String(v) => TagRef::String(v),
            TagRefMut::ByteArray(v) => TagRef::ByteArray(v),
            TagRefMut::IntArray(v) => TagRef::IntArray(v),
            TagRefMut::LongArray(v) => TagRef::LongArray(v),
            TagRefMut::List(v) => TagRef::List(v),
            TagRefMut::Compound(v) => TagRef::Compound(v),
        }
    }
}

impl<'a> From<TagRefMut<'a>> for TagRef<'a> {
    fn from(value: TagRefMut<'a>) -> Self {
        match value {
            TagRefMut::Byte(v) => Self::Byte(v),
            TagRefMut::Short(v) => Self::Short(v),
            TagRefMut::Int(v) => Self::Int(v),
            TagRefMut::Long(v) => Self::Long(v),
            TagRefMut::Float(v) => Self::Float(v),
            TagRefMut::Double(v) => Self::Double(v),
            TagRefMut::String(v) => Self::String(v),
            TagRefMut::ByteArray(v) => Self::ByteArray(v),
            TagRefMut::IntArray(v) => Self::IntArray(v),
            TagRefMut::LongArray(v) => Self::LongArray(v),
            TagRefMut::List(v) => Self::List(v),
            TagRefMut::Compound(v) => Self::Compound(v),
        }
    }
}

macro_rules! into_and_try_from {
    ($type:ty, $kind:ident) => {
        impl<'a> From<&'a $type> for TagRef<'a> {
            fn from(value: &'a $type) -> Self {
                Self::$kind(value)
            }
        }

        impl<'a> From<&'a mut $type> for TagRefMut<'a> {
            fn from(value: &'a mut $type) -> Self {
                Self::$kind(value)
            }
        }

        impl<'a> TryFrom<TagRef<'a>> for &'a $type {
            type Error = TagRef<'a>;

            fn try_from(value: TagRef<'a>) -> Result<Self, Self::Error> {
                match value {
                    TagRef::$kind(r) => Ok(r),
                    _ => Err(value),
                }
            }
        }

        impl<'a> TryFrom<TagRefMut<'a>> for &'a mut $type {
            type Error = TagRefMut<'a>;

            fn try_from(value: TagRefMut<'a>) -> Result<Self, Self::Error> {
                match value {
                    TagRefMut::$kind(r) => Ok(r),
                    _ => Err(value),
                }
            }
        }
    };
}

into_and_try_from!(i8, Byte);
into_and_try_from!(i16, Short);
into_and_try_from!(i32, Int);
into_and_try_from!(i64, Long);
into_and_try_from!(f32, Float);
into_and_try_from!(f64, Double);
into_and_try_from!(String, String);
into_and_try_from!(Vec<i8>, ByteArray);
into_and_try_from!(Vec<i32>, IntArray);
into_and_try_from!(Vec<i64>, LongArray);
into_and_try_from!(TagList, List);
into_and_try_from!(TagMap, Compound);
