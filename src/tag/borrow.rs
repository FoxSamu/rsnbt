use super::*;
use super::cast::*;

macro_rules! casted_borrows {
    ($type:ty, $name:ident, $name_mut:ident, $kind:ident, $example:literal, $example_2:literal $(, $import:literal)?) => {
        impl Tag {
            #[doc = concat!(
                "If this [`Tag`] is a [`Tag::", stringify!($kind), "`], returns an immutable ",
                "reference to the [`", stringify!($type), "`] representing the tag's value.\n\n",

                "If this [`Tag`] is not [`Tag::", stringify!($kind), "`], this method returns ",
                "[`None`]."
            )]
            ///
            /// # Examples
            /// ```
            /// use rsnbt::tag::Tag;
            $(#[doc = concat!("use rsnbt::tag::", $import, ";")])?
            ///
            #[doc = concat!("let tag = Tag::", stringify!($kind), "(", $example, ");")]
            #[doc = concat!("assert_eq!(&", $example, ", tag.", stringify!($name), "().unwrap());")]
            /// ```
            pub fn $name(&self) -> Result<&$type, TagMismatchError> {
                if let Self::$kind(v) = self {
                    Ok(v)
                } else {
                    Err(TagMismatchError::new(TagId::$kind, self.id()))
                }
            }

            #[doc = concat!(
                "If this [`Tag`] is a [`Tag::", stringify!($kind), "`], returns a mutable ",
                "reference to the [`", stringify!($type), "`] representing the tag's value, ",
                "through which the value of the tag can be directly modified.\n\n",

                "If this [`Tag`] is not [`Tag::", stringify!($kind), "`], this method returns ",
                "[`None`]."
            )]
            ///
            /// # Examples
            /// ```
            /// use rsnbt::tag::Tag;
            $(#[doc = concat!("use rsnbt::tag::", $import, ";")])?
            /// 
            #[doc = concat!("let mut tag = Tag::", stringify!($kind), "(", $example, ");")]
            #[doc = concat!("let r = tag.", stringify!($name_mut), "().unwrap();")]
            #[doc = concat!("assert_eq!(&mut ", $example, ", r);")]
            ///
            #[doc = concat!("*r = ", $example_2, ";")]
            #[doc = concat!("assert_eq!(Tag::", stringify!($kind), "(", $example_2, "), tag);")]
            /// ```
            pub fn $name_mut(&mut self) -> Result<&mut $type, TagMismatchError> {
                if let Self::$kind(v) = self {
                    Ok(v)
                } else {
                    Err(TagMismatchError::new(TagId::$kind, self.id()))
                }
            }
        }

        impl<'a> TagRef<'a> {
            #[doc = concat!(
                "If this [`TagRef`] is a [`TagRef::", stringify!($kind), "`], returns an immutable ",
                "reference to the [`", stringify!($type), "`] representing the tag's value.\n\n",

                "If this [`TagRef`] is not [`TagRef::", stringify!($kind), "`], this method returns ",
                "[`None`]."
            )]
            ///
            /// # Examples
            /// ```
            /// use rsnbt::tag::TagRef;
            $(#[doc = concat!("use rsnbt::tag::", $import, ";")])?
            ///
            #[doc = concat!("let payload = ", $example, ";")]
            #[doc = concat!("let tag = TagRef::", stringify!($kind), "(&payload);")]
            #[doc = concat!("assert_eq!(&", $example, ", tag.", stringify!($name), "().unwrap());")]
            /// ```
            pub fn $name(self) -> Result<&'a $type, TagMismatchError> {
                if let Self::$kind(v) = self {
                    Ok(v)
                } else {
                    Err(TagMismatchError::new(TagId::$kind, self.id()))
                }
            }
        }

        impl<'a> TagRefMut<'a> {
            #[doc = concat!(
                "If this [`TagRefMut`] is a [`TagRefMut::", stringify!($kind), "`], returns an immutable ",
                "reference to the [`", stringify!($type), "`] representing the tag's value.\n\n",

                "If this [`TagRefMut`] is not [`TagRefMut::", stringify!($kind), "`], this method returns ",
                "[`None`]."
            )]
            ///
            /// # Examples
            /// ```
            /// use rsnbt::tag::TagRefMut;
            $(#[doc = concat!("use rsnbt::tag::", $import, ";")])?
            ///
            #[doc = concat!("let mut payload = ", $example, ";")]
            #[doc = concat!("let tag = TagRefMut::", stringify!($kind), "(&mut payload);")]
            #[doc = concat!("assert_eq!(&", $example, ", tag.", stringify!($name), "().unwrap());")]
            /// ```
            pub fn $name(self) -> Result<&'a $type, TagMismatchError> {
                if let Self::$kind(v) = self {
                    Ok(v)
                } else {
                    Err(TagMismatchError::new(TagId::$kind, self.id()))
                }
            }

            #[doc = concat!(
                "If this [`TagRefMut`] is a [`TagRefMut::", stringify!($kind), "`], returns a mutable ",
                "reference to the [`", stringify!($type), "`] representing the tag's value, ",
                "through which the value of the tag can be directly modified.\n\n",

                "If this [`TagRefMut`] is not [`TagRefMut::", stringify!($kind), "`], this method returns ",
                "[`None`]."
            )]
            ///
            /// # Examples
            /// ```
            /// use rsnbt::tag::TagRefMut;
            $(#[doc = concat!("use rsnbt::tag::", $import, ";")])?
            /// 
            #[doc = concat!("let mut payload = ", $example, ";")]
            #[doc = concat!("let tag = TagRefMut::", stringify!($kind), "(&mut payload);")]
            #[doc = concat!("let r = tag.", stringify!($name_mut), "().unwrap();")]
            #[doc = concat!("assert_eq!(&mut ", $example, ", r);")]
            ///
            #[doc = concat!("*r = ", $example_2, ";")]
            #[doc = concat!("assert_eq!(", $example_2, ", payload);")]
            /// ```
            pub fn $name_mut(self) -> Result<&'a mut $type, TagMismatchError> {
                if let Self::$kind(v) = self {
                    Ok(v)
                } else {
                    Err(TagMismatchError::new(TagId::$kind, self.id()))
                }
            }
        }
    };
}

casted_borrows!(i8, byte_ref, byte_mut, Byte, "5", "42");
casted_borrows!(i16, short_ref, short_mut, Short, "5", "42");
casted_borrows!(i32, int_ref, int_mut, Int, "5", "42");
casted_borrows!(i64, long_ref, long_mut, Long, "5", "42");
casted_borrows!(f32, float_ref, float_mut, Float, "2.71", "3.14");
casted_borrows!(f64, double_ref, double_mut, Double, "2.71", "3.14");
casted_borrows!(String, string_ref, string_mut, String, "String::from(\"foo\")", "String::from(\"bar\")");
casted_borrows!(Vec<i8>, byte_array_ref, byte_array_mut, ByteArray, "vec![1, 2, 3]", "vec![4, 2]");
casted_borrows!(Vec<i32>, int_array_ref, int_array_mut, IntArray, "vec![1, 2, 3]", "vec![4, 2]");
casted_borrows!(Vec<i64>, long_array_ref, long_array_mut, LongArray, "vec![1, 2, 3]", "vec![4, 2]");
casted_borrows!(TagList, list_ref, list_mut, List, "tag_list![1, 2, 3]", "tag_list![4, 2]", "tag_list");
casted_borrows!(TagMap, compound_ref, compound_mut, Compound, "tag_map!{\"foo\": 5}", "tag_map!{\"bar\": 42}", "tag_map");

