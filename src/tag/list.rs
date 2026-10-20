use core::slice;
use std::fmt::Debug;
use std::vec;

use crate::util::unsafe_ops::*;
use super::*;
use super::cast::*;

/// A list of [`Tag`]s of a specific type. It represents the contents of a list tag ([`Tag::List`]).
///
/// [`TagList`]s contain [`Tag`]s in a specific order, and all [`Tag`]s in the list must be of the same
/// type (that is, they have the same [`TagId`]).
///
/// # Element types
/// The same-type property is enforced by the fact that [`TagList`] is an enum: each branch other than
/// [`TagList::Empty`] will own a [`Vec`] with values of the matching tag type. This means that a [`TagList`]
/// can never contain two values of different tag types.
///
/// Methods will panic when attempting to extract or insert a value of a type that does not match the
/// type of values stored in the list. All of these methods also have a counterpart that returns a
/// [`Result`] instead of panicking.
///
/// Note that the [`TagList`] does not actually store [`Tag`]s, it instead just stores the specific values
/// per type.
///
/// # Empty [`TagList`]s
/// Empty [`TagList`]s adapt their type to the first value inserted into the list.
/// That means that methods like [`TagList::push`], [`TagList::insert`], etc. always work on an empty
/// list without panicking, since there is no specific type that is expected and the
/// first value to be inserted will eventually determine the type.
///
/// Any empty [`TagList`] behaves exactly like [`TagList::Empty`], even if it isn't necessarily
/// of the form `TagList::Empty`. That is, `TagList::Bytes(vec![])` is considered empty and
/// would behave exactly like `TagList::Empty`.
/// This also meanst that when the last value is removed from a [`TagList`], it will lose its type constraint
/// until a new value is inserted.
///
/// Additionally, if two [`TagList`]s are empty, they are considered equal, even if they aren't necessarily
/// the same enum value. You may check if a [`TagList`] is empty by [`TagList::is_empty`].
/// 
/// # Examples
/// ```
/// use rsnbt::tag::{TagList, TagRef, tag_list};
/// 
/// // tag_list macro accepts a JSON-like syntax
/// let mut list = tag_list![1, 2, 3];
/// 
/// // elements are returned as TagRefs
/// assert_eq!(TagRef::Int(&1), list.get(0));
/// assert_eq!(TagRef::Int(&2), list.get(1));
/// assert_eq!(TagRef::Int(&3), list.get(2));
/// 
/// // add an element: will panic if there is a type mismatch
/// // if you need a Result version, you might like try_push
/// list.push(4);
/// assert_eq!(TagRef::Int(&4), list.get(3));
/// ```
#[derive(Clone, Default)]
pub enum TagList {
    /// An empty [`TagList`]. A [`TagList`] is considered empty either when it
    /// is [`Empty`][TagList::Empty], or its payload contains no elements. For
    /// example, `TagList::Bytes(vec![])` is conidered empty. All empty
    /// [`TagList`] values are logically equivalent and will be treated as such.
    /// Hence, `TagList::Bytes(vec![]) == TagList::Empty` will yield `true`.
    /// 
    /// The only way in which [`Empty`][TagList::Empty] differs over other empty
    /// values is in [`TagList::elem_id`]: this will return [`None`] if and only
    /// if the list is precisely [`TagList::Empty`]. When encoding list tags to
    /// NBT blobs, this logic is preserved if configured as such, allowing you
    /// to explicitly specify what element type an individual [`TagList`] is
    /// encoded with. In the case of [`Empty`][TagList::Empty], the special `End`
    /// tag ID (`0`) is used, unless configured otherwise.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let empty = TagList::Empty;
    /// 
    /// // it is truly empty
    /// assert_eq!(0, empty.len());
    /// assert_eq!(true, empty.is_empty());
    /// 
    /// // Empty is logically equivalent to any other empty value
    /// assert_eq!(TagList::Bytes(vec![]), empty);
    /// ```
    #[default]
    Empty,

    /// A [`TagList`] that contains [byte][`Tag::Byte`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Bytes(vec![1, 2, 3]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Byte(&1), list.get(0));
    /// assert_eq!(TagRef::Byte(&2), list.get(1));
    /// assert_eq!(TagRef::Byte(&3), list.get(2));
    /// ```
    Bytes(Vec<i8>),

    /// A [`TagList`] that contains [short][`Tag::Short`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Shorts(vec![1, 2, 3]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Short(&1), list.get(0));
    /// assert_eq!(TagRef::Short(&2), list.get(1));
    /// assert_eq!(TagRef::Short(&3), list.get(2));
    /// ```
    Shorts(Vec<i16>),

    /// A [`TagList`] that contains [int][`Tag::Int`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Ints(vec![1, 2, 3]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Int(&1), list.get(0));
    /// assert_eq!(TagRef::Int(&2), list.get(1));
    /// assert_eq!(TagRef::Int(&3), list.get(2));
    /// ```
    Ints(Vec<i32>),

    /// A [`TagList`] that contains [long][`Tag::Long`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Longs(vec![1, 2, 3]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Long(&1), list.get(0));
    /// assert_eq!(TagRef::Long(&2), list.get(1));
    /// assert_eq!(TagRef::Long(&3), list.get(2));
    /// ```
    Longs(Vec<i64>),

    /// A [`TagList`] that contains [float][`Tag::Float`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Floats(vec![1.61, 2.71, 3.14]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Float(&1.61), list.get(0));
    /// assert_eq!(TagRef::Float(&2.71), list.get(1));
    /// assert_eq!(TagRef::Float(&3.14), list.get(2));
    /// ```
    Floats(Vec<f32>),

    /// A [`TagList`] that contains [double][`Tag::Double`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Doubles(vec![1.61, 2.71, 3.14]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Double(&1.61), list.get(0));
    /// assert_eq!(TagRef::Double(&2.71), list.get(1));
    /// assert_eq!(TagRef::Double(&3.14), list.get(2));
    /// ```
    Doubles(Vec<f64>),

    /// A [`TagList`] that contains [string][`Tag::String`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::Strings(vec![
    ///     "foo".into(),
    ///     "bar".into(),
    ///     "baz".into()
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::String(&"foo".into()), list.get(0));
    /// assert_eq!(TagRef::String(&"bar".into()), list.get(1));
    /// assert_eq!(TagRef::String(&"baz".into()), list.get(2));
    /// ```
    Strings(Vec<String>),

    /// A [`TagList`] that contains [byte array][`Tag::ByteArray`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::ByteArrays(vec![
    ///     vec![1, 2, 3],
    ///     vec![2, 4, 6],
    ///     vec![42, 101]
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::ByteArray(&vec![1, 2, 3]), list.get(0));
    /// assert_eq!(TagRef::ByteArray(&vec![2, 4, 6]), list.get(1));
    /// assert_eq!(TagRef::ByteArray(&vec![42, 101]), list.get(2));
    /// ```
    ByteArrays(Vec<Vec<i8>>),

    /// A [`TagList`] that contains [int array][`Tag::IntArray`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::IntArrays(vec![
    ///     vec![1, 2, 3],
    ///     vec![2, 4, 6],
    ///     vec![42, 101]
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::IntArray(&vec![1, 2, 3]), list.get(0));
    /// assert_eq!(TagRef::IntArray(&vec![2, 4, 6]), list.get(1));
    /// assert_eq!(TagRef::IntArray(&vec![42, 101]), list.get(2));
    /// ```
    IntArrays(Vec<Vec<i32>>),

    /// A [`TagList`] that contains [long array][`Tag::LongArray`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef};
    /// 
    /// let list = TagList::LongArrays(vec![
    ///     vec![1, 2, 3],
    ///     vec![2, 4, 6],
    ///     vec![42, 101]
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::LongArray(&vec![1, 2, 3]), list.get(0));
    /// assert_eq!(TagRef::LongArray(&vec![2, 4, 6]), list.get(1));
    /// assert_eq!(TagRef::LongArray(&vec![42, 101]), list.get(2));
    /// ```
    LongArrays(Vec<Vec<i64>>),

    /// A [`TagList`] that contains [list][`Tag::List`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// Note that the elements of this are themselves [`TagList`]s. These can
    /// be of different types, which is allowed by the NBT format.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef, tag_list};
    /// 
    /// let list = TagList::Lists(vec![
    ///     tag_list![1i8, 2i8, 3i8],
    ///     tag_list!["foo", "bar"],
    ///     tag_list![42i32, 101i32]
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::List(&tag_list![1i8, 2i8, 3i8]), list.get(0));
    /// assert_eq!(TagRef::List(&tag_list!["foo", "bar"]), list.get(1));
    /// assert_eq!(TagRef::List(&tag_list![42i32, 101i32]), list.get(2));
    /// ```
    Lists(Vec<TagList>),

    /// A [`TagList`] that contains [compound][`Tag::Compound`] payloads. If the vector
    /// of payloads is empty, then the [`TagList`] is logically equivalent to
    /// [`TagList::Empty`]. See its documentation for more information.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, TagRef, tag_map};
    /// 
    /// let list = TagList::Compounds(vec![
    ///     tag_map!{ "pi": 3.14 },
    ///     tag_map!{ "e": 2.71 },
    ///     tag_map!{ "phi": 1.61 }
    /// ]);
    /// assert_eq!(3, list.len());
    /// 
    /// // list.get will panic when index is out of range
    /// assert_eq!(TagRef::Compound(&tag_map!{ "pi": 3.14 }), list.get(0));
    /// assert_eq!(TagRef::Compound(&tag_map!{ "e": 2.71 }), list.get(1));
    /// assert_eq!(TagRef::Compound(&tag_map!{ "phi": 1.61 }), list.get(2));
    /// ```
    Compounds(Vec<TagMap>),
}


/// Default implementations for [`TagList`].
impl TagList {
    /// Creates and returns a new, empty [`TagList`]. This returns [`TagList::Empty`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::new();
    /// assert!(list.is_empty());
    /// ```
    pub const fn new() -> TagList {
        TagList::Empty
    }

    /// Creates and returns a new [`TagList`] with the given [byte][Tag::Byte]
    /// payloads. This returns [`TagList::Bytes`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Bytes`] as it
    /// automatically converts other numeric values to byte payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::bytes(vec![1u32, 2, 3]);
    /// assert_eq!(TagList::Bytes(vec![1i8, 2, 3]), list);
    /// ```
    pub fn bytes<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.byte_payload()).collect();
        TagList::Bytes(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [short][Tag::Short]
    /// payloads. This returns [`TagList::Shorts`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Shorts`] as it
    /// automatically converts other numeric values to short payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::shorts(vec![1u32, 2, 3]);
    /// assert_eq!(TagList::Shorts(vec![1i16, 2, 3]), list);
    /// ```
    pub fn shorts<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.short_payload()).collect();
        TagList::Shorts(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [int][Tag::Int]
    /// payloads. This returns [`TagList::Ints`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Ints`] as it
    /// automatically converts other numeric values to int payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::ints(vec![1u8, 2, 3]);
    /// assert_eq!(TagList::Ints(vec![1i32, 2, 3]), list);
    /// ```
    pub fn ints<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.int_payload()).collect();
        TagList::Ints(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [long][Tag::Long]
    /// payloads. This returns [`TagList::Longs`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Longs`] as it
    /// automatically converts other numeric values to long payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::longs(vec![1u8, 2, 3]);
    /// assert_eq!(TagList::Longs(vec![1i64, 2, 3]), list);
    /// ```
    pub fn longs<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.long_payload()).collect();
        TagList::Longs(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [float][Tag::Float]
    /// payloads. This returns [`TagList::Floats`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Floats`] as it
    /// automatically converts other numeric values to float payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::floats(vec![1, 2, 3]);
    /// assert_eq!(TagList::Floats(vec![1.0, 2.0, 3.0]), list);
    /// ```
    pub fn floats<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.float_payload()).collect();
        TagList::Floats(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [double][Tag::Double]
    /// payloads. This returns [`TagList::Doubles`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Doubles`] as it
    /// automatically converts other numeric values to double payloads (via
    /// [`NumPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::doubles(vec![1, 2, 3]);
    /// assert_eq!(TagList::Doubles(vec![1.0, 2.0, 3.0]), list);
    /// ```
    pub fn doubles<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : NumPayload {
        let vec = v.into_iter().map(|v| v.double_payload()).collect();
        TagList::Doubles(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [string][Tag::String]
    /// payloads. This returns [`TagList::Strings`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Strings`] as it
    /// automatically converts other string-like values to string payloads (via
    /// [`Into`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::strings(vec![
    ///     "foo",
    ///     "bar",
    ///     "baz"
    /// ]);
    /// assert_eq!(
    ///     TagList::Strings(vec![
    ///         "foo".into(),
    ///         "bar".into(),
    ///         "baz".into()
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn strings<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : Into<String> {
        let vec = v.into_iter().map(|v| v.into()).collect();
        TagList::Strings(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [byte array][Tag::ByteArray]
    /// payloads. This returns [`TagList::ByteArrays`].
    /// 
    /// This is preferrable over directly constructing [`TagList::ByteArrays`] as it
    /// automatically converts other byte array-like values to byte array payloads (via
    /// [`ArrayPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::byte_arrays(vec![
    ///     vec![1u8, 2, 3],
    ///     vec![2u8, 4, 6],
    ///     vec![42u8, 101]
    /// ]);
    /// assert_eq!(
    ///     TagList::ByteArrays(vec![
    ///         vec![1i8, 2, 3],
    ///         vec![2i8, 4, 6],
    ///         vec![42i8, 101]
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn byte_arrays<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : ArrayPayload<i8> {
        let vec = v.into_iter().map(|v| v.array_payload()).collect();
        TagList::ByteArrays(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [int array][Tag::IntArray]
    /// payloads. This returns [`TagList::IntArrays`].
    /// 
    /// This is preferrable over directly constructing [`TagList::IntArrays`] as it
    /// automatically converts other int array-like values to int array payloads (via
    /// [`ArrayPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::int_arrays(vec![
    ///     vec![1u32, 2, 3],
    ///     vec![2u32, 4, 6],
    ///     vec![42u32, 101]
    /// ]);
    /// assert_eq!(
    ///     TagList::IntArrays(vec![
    ///         vec![1i32, 2, 3],
    ///         vec![2i32, 4, 6],
    ///         vec![42i32, 101]
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn int_arrays<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : ArrayPayload<i32> {
        let vec = v.into_iter().map(|v| v.array_payload()).collect();
        TagList::IntArrays(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [long array][Tag::LongArray]
    /// payloads. This returns [`TagList::LongArrays`].
    /// 
    /// This is preferrable over directly constructing [`TagList::LongArrays`] as it
    /// automatically converts other long array-like values to long array payloads (via
    /// [`ArrayPayload`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    ///
    /// let list = TagList::long_arrays(vec![
    ///     vec![1u64, 2, 3],
    ///     vec![2u64, 4, 6],
    ///     vec![42u64, 101]
    /// ]);
    /// assert_eq!(
    ///     TagList::LongArrays(vec![
    ///         vec![1i64, 2, 3],
    ///         vec![2i64, 4, 6],
    ///         vec![42i64, 101]
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn long_arrays<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : ArrayPayload<i64> {
        let vec = v.into_iter().map(|v| v.array_payload()).collect();
        TagList::LongArrays(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [list][Tag::List]
    /// payloads. This returns [`TagList::Lists`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Lists`] as it
    /// automatically converts other list-like values to list payloads (via
    /// [`Into`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, tag_list};
    ///
    /// let list = TagList::lists(vec![
    ///     // note: we are passing a Vec<Vec<f32>>
    ///     vec![1.61, 2.71],
    ///     vec![3.14, 4.00],
    ///     vec![5.24, 6.28]
    /// ]);
    /// assert_eq!(
    ///     TagList::Lists(vec![
    ///         tag_list![1.61, 2.71],
    ///         tag_list![3.14, 4.00],
    ///         tag_list![5.24, 6.28]
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn lists<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : Into<TagList> {
        let vec = v.into_iter().map(|v| v.into()).collect();
        TagList::Lists(vec)
    }

    /// Creates and returns a new [`TagList`] with the given [compound][Tag::Compound]
    /// payloads. This returns [`TagList::Compounds`].
    /// 
    /// This is preferrable over directly constructing [`TagList::Compounds`] as it
    /// automatically converts other compound-like values to compound payloads (via
    /// [`Into`]).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, tag_map};
    /// use std::collections::HashMap;
    /// 
    /// let mut b = HashMap::new();
    /// b.insert(String::from("phi"), 1.61);
    /// b.insert(String::from("e"), 2.71);
    /// b.insert(String::from("pi"), 3.14);
    ///
    /// let list = TagList::compounds(vec![b]);
    /// assert_eq!(
    ///     TagList::Compounds(vec![
    ///         tag_map!{
    ///             "phi": 1.61,
    ///             "e": 2.71,
    ///             "pi": 3.14
    ///         }
    ///     ]),
    ///     list
    /// );
    /// ```
    pub fn compounds<U, V>(v: V) -> TagList where V : IntoIterator<Item = U>, U : Into<TagMap> {
        let vec = v.into_iter().map(|v| v.into()).collect();
        TagList::Compounds(vec)
    }


    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [`Tag`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagList};
    ///
    /// let list = TagList::just(Tag::int(3));
    /// assert_eq!(TagList::ints([3]), list);
    /// ```
    pub fn just(value: Tag) -> Self {
        let mut list = Self::new();
        list.push(value);
        list
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [byte][Tag::Byte] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::byte(3);
    /// assert_eq!(TagList::bytes([3]), list);
    /// ```
    pub fn byte<U>(value: U) -> Self where U : NumPayload {
        Self::bytes([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [short][Tag::Short] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::short(3);
    /// assert_eq!(TagList::shorts([3]), list);
    /// ```
    pub fn short<U>(value: U) -> Self where U : NumPayload {
        Self::shorts([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [int][Tag::Int] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::int(3);
    /// assert_eq!(TagList::ints([3]), list);
    /// ```
    pub fn int<U>(value: U) -> Self where U : NumPayload {
        Self::ints([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [long][Tag::Long] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::long(3);
    /// assert_eq!(TagList::longs([3]), list);
    /// ```
    pub fn long<U>(value: U) -> Self where U : NumPayload {
        Self::longs([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [float][Tag::Float] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::float(3); // <- converts via NumPayload
    /// assert_eq!(TagList::floats([3.0]), list);
    /// ```
    pub fn float<U>(value: U) -> Self where U : NumPayload {
        Self::floats([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [double][Tag::Double] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::double(3); // <- converts via NumPayload
    /// assert_eq!(TagList::doubles([3.0]), list);
    /// ```
    pub fn double<U>(value: U) -> Self where U : NumPayload {
        Self::doubles([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [string][Tag::String] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::string("foo");
    /// assert_eq!(TagList::strings(["foo"]), list);
    /// ```
    pub fn string<U>(value: U) -> Self where U : Into<String> {
        Self::strings([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [byte array][Tag::ByteArrray] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::byte_array([1i8, 2, 3]);
    /// assert_eq!(TagList::byte_arrays([[1i8, 2, 3]]), list);
    /// ```
    pub fn byte_array<U>(value: U) -> Self where U : ArrayPayload<i8> {
        Self::byte_arrays([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [int array][Tag::IntArray] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::int_array([1i32, 2, 3]);
    /// assert_eq!(TagList::int_arrays([[1i32, 2, 3]]), list);
    /// ```
    pub fn int_array<U>(value: U) -> Self where U : ArrayPayload<i32> {
        Self::int_arrays([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [long array][Tag::LongArray] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::long_array([1i64, 2, 3]);
    /// assert_eq!(TagList::long_arrays([[1i64, 2, 3]]), list);
    /// ```
    pub fn long_array<U>(value: U) -> Self where U : ArrayPayload<i64> {
        Self::long_arrays([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [list][Tag::List] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let list = TagList::list([1, 2, 3]);
    /// assert_eq!(TagList::lists([[1, 2, 3]]), list);
    /// ```
    pub fn list<U>(value: U) -> Self where U : Into<TagList> {
        Self::lists([value])
    }

    /// Creates and returns a new, singleton [`TagList`], containing just the given
    /// [compound][Tag::Compound] payload.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, tag_map};
    /// 
    /// let list = TagList::compound(tag_map!{ "pi": 3.14 });
    /// assert_eq!(TagList::compounds([tag_map!{ "pi": 3.14 }]), list);
    /// ```
    pub fn compound<U>(value: U) -> Self where U : Into<TagMap> {
        Self::compounds([value])
    }

    /// Tests whether a [`TagList`] is empty. A [`TagList`] is empty when
    /// either of these conditions hold:
    /// - it is [`TagList::Empty`];
    /// - it is not [`TagList::Empty`] but its vector of payloads is
    ///   [empty][Vec::is_empty].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::tag_list;
    /// 
    /// let empty = tag_list![];
    /// let not_empty = tag_list![1];
    /// 
    /// assert!(empty.is_empty());
    /// assert!(!not_empty.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        match self {
            TagList::Empty => true,
            TagList::Bytes(v) => v.is_empty(),
            TagList::Shorts(v) => v.is_empty(),
            TagList::Ints(v) => v.is_empty(),
            TagList::Longs(v) => v.is_empty(),
            TagList::Floats(v) => v.is_empty(),
            TagList::Doubles(v) => v.is_empty(),
            TagList::Strings(v) => v.is_empty(),
            TagList::ByteArrays(v) => v.is_empty(),
            TagList::IntArrays(v) => v.is_empty(),
            TagList::LongArrays(v) => v.is_empty(),
            TagList::Lists(v) => v.is_empty(),
            TagList::Compounds(v) => v.is_empty(),
        }
    }

    /// Returns the length of a [`TagList`]. A [`TagList`]'s length is
    /// determined as follows:
    /// - if the list is [`TagList::Empty`], then its length is 0;
    /// - otherwise its length is the number of payloads in its payload
    ///   vector (`Vec::len`).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::tag_list;
    /// 
    /// let one = tag_list![1];
    /// let three = tag_list![1, 2, 3];
    /// let zero = tag_list![];
    /// 
    /// assert_eq!(1, one.len());
    /// assert_eq!(3, three.len());
    /// assert_eq!(0, zero.len());
    /// ```
    pub const fn len(&self) -> usize {
        match self {
            TagList::Empty => 0,
            TagList::Bytes(v) => v.len(),
            TagList::Shorts(v) => v.len(),
            TagList::Ints(v) => v.len(),
            TagList::Longs(v) => v.len(),
            TagList::Floats(v) => v.len(),
            TagList::Doubles(v) => v.len(),
            TagList::Strings(v) => v.len(),
            TagList::ByteArrays(v) => v.len(),
            TagList::IntArrays(v) => v.len(),
            TagList::LongArrays(v) => v.len(),
            TagList::Lists(v) => v.len(),
            TagList::Compounds(v) => v.len(),
        }
    }

    /// Returns the [`TagId`] of the type of payloads a [`TagList`] stores.
    /// 
    /// This method returns [`None`] precisely when `self` is
    /// [`TagList::Empty`]. This and [`TagList::elem_id_byte`] are the
    /// only methods that behave differently when a tag list is the
    /// [`Empty`][TagList::Empty] constant, versus when a tag list is
    /// not [`Empty`][TagList::Empty] but contains no payloads.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{tag_list, TagId, TagList};
    /// 
    /// let strings = TagList::strings(["foo"]);
    /// assert_eq!(Some(TagId::String), strings.elem_id());
    /// 
    /// 
    /// // elem_id differs in Empty vs empty vector Ints
    /// let empty = TagList::new();
    /// let empty_ints = TagList::Ints(vec![]);
    /// 
    /// assert_eq!(None, empty.elem_id());
    /// assert_eq!(Some(TagId::Int), empty_ints.elem_id());
    /// 
    /// // however, empty arrays are logically equivalent otherwise
    /// assert_eq!(empty, empty_ints);
    /// ```
    pub const fn elem_id(&self) -> Option<TagId> {
        match self {
            TagList::Empty => None,
            TagList::Bytes(_) => Some(TagId::Byte),
            TagList::Shorts(_) => Some(TagId::Short),
            TagList::Ints(_) => Some(TagId::Int),
            TagList::Longs(_) => Some(TagId::Long),
            TagList::Floats(_) => Some(TagId::Float),
            TagList::Doubles(_) => Some(TagId::Double),
            TagList::Strings(_) => Some(TagId::String),
            TagList::ByteArrays(_) => Some(TagId::ByteArray),
            TagList::IntArrays(_) => Some(TagId::IntArray),
            TagList::LongArrays(_) => Some(TagId::LongArray),
            TagList::Lists(_) => Some(TagId::List),
            TagList::Compounds(_) => Some(TagId::Compound),
        }
    }

    /// Returns the ID byte of the type of payloads a [`TagList`] stores.
    /// This is equivalent to `elem_id().byte()` (see [`TagList::elem_id`]).
    /// 
    /// This method returns `0` precisely when `self` is
    /// [`TagList::Empty`]. This and [`TagList::elem_id`] are the
    /// only methods that behave differently when a tag list is the
    /// [`Empty`][TagList::Empty] constant, versus when a tag list is
    /// not [`Empty`][TagList::Empty] but contains no payloads.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{tag_list, TagId, TagList};
    /// 
    /// let strings = TagList::strings(["foo"]);
    /// assert_eq!(8, strings.elem_id_byte());
    /// 
    /// 
    /// // elem_id_byte differs in Empty vs empty vector Ints
    /// let empty = TagList::new();
    /// let empty_ints = TagList::Ints(vec![]);
    /// 
    /// assert_eq!(0, empty.elem_id_byte());
    /// assert_eq!(3, empty_ints.elem_id_byte());
    /// 
    /// // however, empty arrays are logically equivalent otherwise
    /// assert_eq!(empty, empty_ints);
    /// ```
    pub const fn elem_id_byte(&self) -> u8 {
        match self.elem_id() {
            Some(id) => id.byte(),
            None => 0,
        }
    }

    /// Turns any empty [`TagList`] into precisely [`TagList::Empty`].
    /// If the list is not empty, this returns `self` with no changes.
    /// 
    /// If you have a mutable reference to a [`TagList`] and cannot
    /// move out of it, use [`reduce`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let empty = TagList::Ints(vec![]);
    /// let not_empty = TagList::ints([1, 2, 3]);
    /// 
    /// // empty lists reduce to Empty
    /// assert_eq!(TagList::Ints(vec![]), empty);
    /// assert_eq!(TagList::Empty, empty.reduced());
    /// 
    /// // non-empty lists remain the same
    /// assert_eq!(TagList::Ints(vec![1, 2, 3]), not_empty);
    /// assert_eq!(TagList::Ints(vec![1, 2, 3]), not_empty.reduced());
    /// ```
    pub fn reduced(self) -> Self {
        if self.is_empty() {
            Self::Empty
        } else {
            self
        }
    }

    /// Turns any empty [`TagList`] into precisely [`TagList::Empty`].
    /// If the list is not empty, this leaves the list unchanged.
    /// 
    /// If you own the [`TagList`] and instead wants a method that works
    /// in an expression, use [`reduced`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let mut empty = TagList::Ints(vec![]);
    /// let mut not_empty = TagList::ints([1, 2, 3]);
    /// 
    /// assert_eq!(TagList::Ints(vec![]), empty);
    /// assert_eq!(TagList::Ints(vec![1, 2, 3]), not_empty);
    /// 
    /// // empty lists reduce to Empty
    /// empty.reduce();
    /// assert_eq!(TagList::Empty, empty);
    /// 
    /// // non-empty lists remain the same
    /// not_empty.reduce();
    /// assert_eq!(TagList::Ints(vec![1, 2, 3]), not_empty);
    /// ```
    pub fn reduce(&mut self) {
        if self.is_empty() {
            *self = Self::Empty;
        }
    }

    /// Removes and drops all elements from a [`TagList`], turning it
    /// into an empty list. More precisely, the list is turned into
    /// [`TagList::Empty`]. If this is undesirable, use
    /// [`TagList::clear_raw`].
    /// 
    /// This method is equivalent to simply reassigning to
    /// [`TagList::Empty`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagList, tag_list};
    /// 
    /// let mut empty = TagList::Ints(vec![]);
    /// let mut not_empty = TagList::ints([1, 2, 3]);
    /// 
    /// empty.clear();
    /// assert_eq!(TagList::Empty, empty);
    /// 
    /// not_empty.clear();
    /// assert_eq!(TagList::Empty, not_empty);
    /// ```
    pub fn clear(&mut self) {
        *self = Self::Empty;
    }

    /// Removes and drops all elements from a [`TagList`], turning it
    /// into an empty list. Unlike [`TagList::clear`], this preserves the
    /// lists's element type.
    /// 
    /// This method is equivalent to calling [`Vec::clear`] on the
    /// vector of payloads of the list (if it has one).
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagList;
    /// 
    /// let mut empty = TagList::new();
    /// let mut not_empty = TagList::ints([1, 2, 3]);
    /// 
    /// empty.clear(); // <- it is already Empty, it remains Empty
    /// assert_eq!(TagList::Empty, empty);
    /// 
    /// not_empty.soft_clear(); // <- it is Ints, it remains Ints
    /// assert_eq!(TagList::Ints(vec![]), not_empty);
    /// ```
    pub fn soft_clear(&mut self) {
        match self {
            TagList::Empty => (),
            TagList::Bytes(v) => v.clear(),
            TagList::Shorts(v) => v.clear(),
            TagList::Ints(v) => v.clear(),
            TagList::Longs(v) => v.clear(),
            TagList::Floats(v) => v.clear(),
            TagList::Doubles(v) => v.clear(),
            TagList::Strings(v) => v.clear(),
            TagList::ByteArrays(v) => v.clear(),
            TagList::IntArrays(v) => v.clear(),
            TagList::LongArrays(v) => v.clear(),
            TagList::Lists(v) => v.clear(),
            TagList::Compounds(v) => v.clear(),
        };
    }

    /// Reverses the order of the elements of a [`TagList`]. If the list
    /// is empty or has just one single element, this has no effect.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::tag_list;
    /// 
    /// let mut list = tag_list![1, 2, 3];
    /// 
    /// list.reverse();
    /// assert_eq!(tag_list![3, 2, 1], list);
    /// ```
    pub fn reverse(&mut self) {
        match self {
            TagList::Empty => (),
            TagList::Bytes(v) => v.reverse(),
            TagList::Shorts(v) => v.reverse(),
            TagList::Ints(v) => v.reverse(),
            TagList::Longs(v) => v.reverse(),
            TagList::Floats(v) => v.reverse(),
            TagList::Doubles(v) => v.reverse(),
            TagList::Strings(v) => v.reverse(),
            TagList::ByteArrays(v) => v.reverse(),
            TagList::IntArrays(v) => v.reverse(),
            TagList::LongArrays(v) => v.reverse(),
            TagList::Lists(v) => v.reverse(),
            TagList::Compounds(v) => v.reverse(),
        };
    }

    /// Borrow an element from a [`TagList`], given its index, with index `0`
    /// referring to the first element, index `1` to the second, and so on.
    /// The last element has index `len() - 1` (see [`len`][TagList::len]).
    /// 
    /// This method returns a [`TagRef`]. It does not return a reference to
    /// [`Tag`] directly (that is, it does not return `&Tag`), because no actual
    /// [`Tag`] values are stored in a [`TagList`] - only payloads are stored
    /// and [`TagRef`] can borrow a payload of a tag without needing a [`Tag`].
    /// 
    /// # Panics
    /// This method panics when the given index is greater than or equal to
    /// `len()`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{tag_list, TagRef};
    /// 
    /// let mut list = tag_list![1, 2, 3];
    /// 
    /// assert_eq!(TagRef::Int(&1), list.get(0));
    /// assert_eq!(TagRef::Int(&2), list.get(1));
    /// assert_eq!(TagRef::Int(&3), list.get(2));
    /// ```
    pub fn get(&self, index: usize) -> TagRef<'_> {
        if self.is_empty() {
            panic!("TagList is empty, index is always out of range");
        };

        match self {
            TagList::Empty => unreachable!(),

            TagList::Bytes(v) => TagRef::Byte(&v[index]),
            TagList::Shorts(v) => TagRef::Short(&v[index]),
            TagList::Ints(v) => TagRef::Int(&v[index]),
            TagList::Longs(v) => TagRef::Long(&v[index]),
            TagList::Floats(v) => TagRef::Float(&v[index]),
            TagList::Doubles(v) => TagRef::Double(&v[index]),
            TagList::Strings(v) => TagRef::String(&v[index]),
            TagList::ByteArrays(v) => TagRef::ByteArray(&v[index]),
            TagList::IntArrays(v) => TagRef::IntArray(&v[index]),
            TagList::LongArrays(v) => TagRef::LongArray(&v[index]),
            TagList::Lists(v) => TagRef::List(&v[index]),
            TagList::Compounds(v) => TagRef::Compound(&v[index]),
        }
    }

    /// Mutably borrow an element from a [`TagList`], given its index, with
    /// index `0` referring to the first element, index `1` to the second,
    /// and so on. The last element has index `len() - 1` (see
    /// [`len`][TagList::len]).
    /// 
    /// This method returns a [`TagRefMut`]. It does not return a reference to
    /// [`Tag`] directly (that is, it does not return `&mut Tag`), because no
    /// actual [`Tag`] values are stored in a [`TagList`] - only payloads are
    /// stored and [`TagRefMut`] can mutably borrow a payload of a tag without
    /// needing a [`Tag`].
    /// 
    /// # Panics
    /// This method panics when the given index is greater than or equal to
    /// `len()`.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{tag_list, TagRefMut};
    /// 
    /// let mut list = tag_list![1, 2, 3];
    /// 
    /// assert_eq!(TagRefMut::Int(&mut 1), list.get_mut(0));
    /// assert_eq!(TagRefMut::Int(&mut 2), list.get_mut(1));
    /// assert_eq!(TagRefMut::Int(&mut 3), list.get_mut(2));
    /// 
    /// let TagRefMut::Int(int) = list.get_mut(1) else {
    ///     panic!(); // we have ints, this shouldn't happen
    /// };
    /// 
    /// // update the borrowed payload
    /// *int = 5;
    /// assert_eq!(tag_list![1, 5, 3], list);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> TagRefMut<'_> {
        if self.is_empty() {
            panic!("TagList is empty, index is always out of range");
        };

        match self {
            TagList::Empty => unreachable!(),

            TagList::Bytes(v) => TagRefMut::Byte(&mut v[index]),
            TagList::Shorts(v) => TagRefMut::Short(&mut v[index]),
            TagList::Ints(v) => TagRefMut::Int(&mut v[index]),
            TagList::Longs(v) => TagRefMut::Long(&mut v[index]),
            TagList::Floats(v) => TagRefMut::Float(&mut v[index]),
            TagList::Doubles(v) => TagRefMut::Double(&mut v[index]),
            TagList::Strings(v) => TagRefMut::String(&mut v[index]),
            TagList::ByteArrays(v) => TagRefMut::ByteArray(&mut v[index]),
            TagList::IntArrays(v) => TagRefMut::IntArray(&mut v[index]),
            TagList::LongArrays(v) => TagRefMut::LongArray(&mut v[index]),
            TagList::Lists(v) => TagRefMut::List(&mut v[index]),
            TagList::Compounds(v) => TagRefMut::Compound(&mut v[index]),
        }
    }


    // TODO document
    // Also we can possibly change a lot of `<T>(...: T) where T : ...`
    //   signatures into `impl ...` signatures, they'll be cleaner

    /// Tries to append a value to the end of a [`TagList`].
    /// 
    /// # Type Conversion
    /// Unless the list is empty, the given value must match the element
    /// type of the [`TagList`]. That is, if the [`Tag`] that the given
    /// value converts to cannot be [cast][Tag::try_cast] to the
    /// [element type][TagList::elem_id] of the list, then the insertion
    /// will fail. If the list is empty, whether that is
    /// [`TagList::Empty`] or any other type of tag list with zero
    /// elements, the insertion will always succeed. In this case, the
    /// list will adapt its [element type][TagList::elem_id] to the type
    /// of the inserted [`Tag`], without performing any
    /// [cast][Tag::try_cast].
    /// 
    /// If the given parameter is a [`Tag`], and the [tag's ID][Tag::id]
    /// does not match the [list's element ID][TagList::elem_id], then
    /// insertion will succeed only if:
    /// - the list is [empty][TagList::is_empty], or;
    /// - the tag's ID and the list's element ID are both numeric,
    ///   allowing a [cast][Tag::try_cast].
    /// 
    /// If the insertion succeeds, this method returns [`Ok`], returning
    /// the unit value.
    /// 
    /// # Errors
    /// This method will return [`Err`] if the given value does not
    /// convert to a [`Tag`] that can be correctly [cast][Tag::try_cast]
    /// into the type of elements in this [`TagList`]. The resulting
    /// error is a [`TagCastError`], whose payload is the [`Tag`] that
    /// the given value was converted into (via [`Into`]).
    /// 
    /// # See Also
    /// - If you want to insert an element at another index, you can use
    ///   [`TagList::try_insert`].
    /// - If you want to get a [mutable reference][TagRefMut] to the
    ///   inserted element, you can use [`TagList::try_push_mut`].
    /// - If you are fully sure that the given value can be inserted
    ///   infallibly and don't want to bother handling an error error
    ///   that cannot happen, you can use [`TagList::push`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// assert_matches!(list.try_push(Tag::int(4)), Ok(_));
    /// assert_eq!(list, tag_list![1.61, 2.71, 3.14, 4.0]);
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work
    /// assert_matches!(list.try_push(Tag::string("foo")), Err(_));
    /// ```
    pub fn try_push(&mut self, value: impl Into<Tag>) -> Result<(), TagCastError<Tag>> {
        self.try_push_mut(value)?;
        Ok(())
    }

    /// Appends a value to the end of a [`TagList`].
    /// 
    /// # Type Conversion
    /// Unless the list is empty, the given value must match the element
    /// type of the [`TagList`]. That is, if the [`Tag`] that the given
    /// value converts to cannot be [cast][Tag::try_cast] to the
    /// [element type][TagList::elem_id] of the list, then the insertion
    /// will fail. If the list is empty, whether that is
    /// [`TagList::Empty`] or any other type of tag list with zero
    /// elements, the insertion will always succeed. In this case, the
    /// list will adapt its [element type][TagList::elem_id] to the type
    /// of the inserted [`Tag`], without performing any
    /// [cast][Tag::try_cast].
    /// 
    /// If the given parameter is a [`Tag`], and the [tag's ID][Tag::id]
    /// does not match the [list's element ID][TagList::elem_id], then
    /// insertion will succeed only if:
    /// - the list is [empty][TagList::is_empty], or;
    /// - the tag's ID and the list's element ID are both numeric,
    ///   allowing a [cast][Tag::try_cast].
    /// 
    /// If the insertion succeeds, this method returns the unit value.
    /// 
    /// # Panics
    /// This method will panic if the given value does not convert to a
    /// [`Tag`] that can be correctly [cast][Tag::try_cast] into the type
    /// of elements in this [`TagList`]. You should therefore be careful
    /// using this method and only use it if you can fully ensure that
    /// the given value can be correctly inserted into the list. If you
    /// cannot be sure of that, use [`TagList::try_push`] instead, which
    /// does the same thing but returns a [`Result`] instead of
    /// panicking.
    /// 
    /// # See Also
    /// - If you want to insert an element at another index, you can use
    ///   [`TagList::insert`].
    /// - If you want to get a [mutable reference][TagRefMut] to the
    ///   inserted element, you can use [`TagList::push_mut`].
    /// - If you aren't sure that the given value can be inserted
    ///   infallibly and want to handle a failing insertion, you can use
    ///   [`TagList::try_push`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// list.push(Tag::int(4));
    /// assert_eq!(list, tag_list![1.61, 2.71, 3.14, 4.0]);
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```should_panic
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work and panics
    /// list.push(Tag::string("foo"));
    /// ```
    pub fn push(&mut self, value: impl Into<Tag>) {
        ok_or_panic("push", self.try_push(value))
    }

    /// Tries to append a value to the end of a [`TagList`], and returns
    /// a [mutable reference][TagRefMut] to the inserted payload when
    /// successful.
    /// 
    /// This method follows the same type conversion and error logic as
    /// [`TagList::try_push`], with the only difference being that a
    /// mutable reference to the inserted element is returned, instead
    /// of a unit value. See [`try_push`][`TagList::try_push`] for the
    /// exact details under which insertion succeeds or fails.
    /// 
    /// # See Also
    /// - If you want to insert an element at another index, you can use
    ///   [`TagList::try_insert_mut`].
    /// - If you do not need a reference to the inserted element, you
    ///   can use [`TagList::try_push`].
    /// - If you are fully sure that the given value can be inserted
    ///   infallibly and don't want to bother handling an error error
    ///   that cannot happen, you can use [`TagList::push_mut`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// use rsnbt::tag::{Tag, tag_list, TagRefMut};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// let inserted = list.try_push_mut(Tag::int(4))?;
    /// assert_eq!(inserted, TagRefMut::Double(&mut 4.0));
    /// 
    /// let reference = inserted.double_mut()?;
    /// *reference = 6.28;
    /// assert_eq!(list, tag_list![1.61, 2.71, 3.14, 6.28]);
    /// 
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work
    /// assert_matches!(list.try_push_mut(Tag::string("foo")), Err(_));
    /// ```
    pub fn try_push_mut(&mut self, value: impl Into<Tag>) -> Result<TagRefMut<'_>, TagCastError<Tag>> {
        let tag = value.into();

        let mut_ref = if let Some(id) = self.elem_id() {
            let cast = tag.try_cast(id)?;
            match (self, cast) {
                (TagList::Bytes(vs), Tag::Byte(v)) => TagRefMut::Byte(vs.push_mut(v)),
                (TagList::Shorts(vs), Tag::Short(v)) => TagRefMut::Short(vs.push_mut(v)),
                (TagList::Ints(vs), Tag::Int(v)) => TagRefMut::Int(vs.push_mut(v)),
                (TagList::Longs(vs), Tag::Long(v)) => TagRefMut::Long(vs.push_mut(v)),
                (TagList::Floats(vs), Tag::Float(v)) => TagRefMut::Float(vs.push_mut(v)),
                (TagList::Doubles(vs), Tag::Double(v)) => TagRefMut::Double(vs.push_mut(v)),
                (TagList::Strings(vs), Tag::String(v)) => TagRefMut::String(vs.push_mut(v)),
                (TagList::ByteArrays(vs), Tag::ByteArray(v)) => TagRefMut::ByteArray(vs.push_mut(v)),
                (TagList::IntArrays(vs), Tag::IntArray(v)) => TagRefMut::IntArray(vs.push_mut(v)),
                (TagList::LongArrays(vs), Tag::LongArray(v)) => TagRefMut::LongArray(vs.push_mut(v)),
                (TagList::Lists(vs), Tag::List(v)) => TagRefMut::List(vs.push_mut(v)),
                (TagList::Compounds(vs), Tag::Compound(v)) => TagRefMut::Compound(vs.push_mut(v)),

                _ => unreachable!(),
            }
        } else {
            macro_rules! make_list_and_return_mut_ref {
                ($id:ident, $ids:ident, $v:expr) => {{
                    *self = TagList::$ids(Vec::new());

                    match self {
                        TagList::$ids(vs) => TagRefMut::$id(vs.push_mut($v)),
                        _ => unreachable!(),
                    }
                }};
            }

            match tag {
                Tag::Byte(v) => make_list_and_return_mut_ref!(Byte, Bytes, v),
                Tag::Short(v) => make_list_and_return_mut_ref!(Short, Shorts, v),
                Tag::Int(v) => make_list_and_return_mut_ref!(Int, Ints, v),
                Tag::Long(v) => make_list_and_return_mut_ref!(Long, Longs, v),
                Tag::Float(v) => make_list_and_return_mut_ref!(Float, Floats, v),
                Tag::Double(v) => make_list_and_return_mut_ref!(Double, Doubles, v),
                Tag::String(v) => make_list_and_return_mut_ref!(String, Strings, v),
                Tag::ByteArray(v) => make_list_and_return_mut_ref!(ByteArray, ByteArrays, v),
                Tag::IntArray(v) => make_list_and_return_mut_ref!(IntArray, IntArrays, v),
                Tag::LongArray(v) => make_list_and_return_mut_ref!(LongArray, LongArrays, v),
                Tag::List(v) => make_list_and_return_mut_ref!(List, Lists, v),
                Tag::Compound(v) => make_list_and_return_mut_ref!(Compound, Compounds, v),
            }
        };

        Ok(mut_ref)
    }

    /// Appends a value to the end of a [`TagList`], and returns a
    /// [mutable reference][TagRefMut] to the inserted payload.
    /// 
    /// This method follows the same type conversion and panic logic as
    /// [`TagList::push`], with the only difference being that a
    /// mutable reference to the inserted element is returned, instead
    /// of a unit value. See [`push`][`TagList::push`] for the
    /// exact details under which insertion succeeds or fails.
    /// 
    /// # Panics
    /// Unlike [`try_push_mut`][`TagList::try_push_mut`], this method
    /// will panic when conversion to the correct type fails for the
    /// given value. See [`push`][`tagList::push`] for details.
    /// 
    /// # See Also
    /// - If you want to insert an element at another index, you can use
    ///   [`TagList::insert_mut`].
    /// - If you do not need a reference to the inserted element, you
    ///   can use [`TagList::push`].
    /// - If you aren't sure that the given value can be inserted
    ///   infallibly and want to handle a failing insertion, you can use
    ///   [`TagList::try_push_mut`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// use rsnbt::tag::{Tag, tag_list, TagRefMut};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// let inserted = list.push_mut(Tag::int(4));
    /// assert_eq!(inserted, TagRefMut::Double(&mut 4.0));
    /// 
    /// let reference = inserted.double_mut()?;
    /// *reference = 6.28;
    /// assert_eq!(list, tag_list![1.61, 2.71, 3.14, 6.28]);
    /// 
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```should_panic
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work and panics
    /// list.push_mut(Tag::string("foo"));
    /// ```
    pub fn push_mut(&mut self, value: impl Into<Tag>) -> TagRefMut<'_> {
        ok_or_panic("push", self.try_push_mut(value))
    }

    /// Tries to insert a value at the given index into a [`TagList`].
    /// The given index must be no greater than [the length of the
    /// list][TagList::len]. If the index is exactly the list's length,
    /// then this method behaves exactly like [`TagList::try_push`].
    /// 
    /// # Type Conversion
    /// Unless the list is empty, the given value must match the element
    /// type of the [`TagList`]. That is, if the [`Tag`] that the given
    /// value converts to cannot be [cast][Tag::try_cast] to the
    /// [element type][TagList::elem_id] of the list, then the insertion
    /// will fail. If the list is empty, whether that is
    /// [`TagList::Empty`] or any other type of tag list with zero
    /// elements, the insertion will always succeed. In this case, the
    /// list will adapt its [element type][TagList::elem_id] to the type
    /// of the inserted [`Tag`], without performing any
    /// [cast][Tag::try_cast].
    /// 
    /// If the given parameter is a [`Tag`], and the [tag's ID][Tag::id]
    /// does not match the [list's element ID][TagList::elem_id], then
    /// insertion will succeed only if:
    /// - the list is [empty][TagList::is_empty], or;
    /// - the tag's ID and the list's element ID are both numeric,
    ///   allowing a [cast][Tag::try_cast].
    /// 
    /// If the insertion succeeds, this method returns [`Ok`], returning
    /// the unit value.
    /// 
    /// # Errors
    /// This method will return [`Err`] if the given value does not
    /// convert to a [`Tag`] that can be correctly [cast][Tag::try_cast]
    /// into the type of elements in this [`TagList`]. The resulting
    /// error is a [`TagCastError`], whose payload is the [`Tag`] that
    /// the given value was converted into (via [`Into`]).
    /// 
    /// # Panics
    /// This method will panic if the given index is out of range. That
    /// is, the index is strictly greater than [`len`][TagList::len].
    /// 
    /// # See Also
    /// - If you want to insert an element at the end, you can use
    ///   [`TagList::try_push`].
    /// - If you want to get a [mutable reference][TagRefMut] to the
    ///   inserted element, you can use [`TagList::try_insert_mut`].
    /// - If you are fully sure that the given value can be inserted
    ///   infallibly and don't want to bother handling an error error
    ///   that cannot happen, you can use [`TagList::insert`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// assert_matches!(list.try_insert(0, Tag::int(4)), Ok(_));
    /// assert_eq!(list, tag_list![4.0, 1.61, 2.71, 3.14]);
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work
    /// assert_matches!(list.try_insert(0, Tag::string("foo")), Err(_));
    /// ```
    pub fn try_insert(&mut self, index: usize, value: impl Into<Tag>) -> Result<(), TagCastError<Tag>> {
        self.try_insert_mut(index, value)?;
        Ok(())
    }

    /// Inserts a value at the given index into a [`TagList`].
    /// The given index must be no greater than [the length of the
    /// list][TagList::len]. If the index is exactly the list's length,
    /// then this method behaves exactly like [`TagList::push`].
    /// 
    /// # Type Conversion
    /// Unless the list is empty, the given value must match the element
    /// type of the [`TagList`]. That is, if the [`Tag`] that the given
    /// value converts to cannot be [cast][Tag::try_cast] to the
    /// [element type][TagList::elem_id] of the list, then the insertion
    /// will fail. If the list is empty, whether that is
    /// [`TagList::Empty`] or any other type of tag list with zero
    /// elements, the insertion will always succeed. In this case, the
    /// list will adapt its [element type][TagList::elem_id] to the type
    /// of the inserted [`Tag`], without performing any
    /// [cast][Tag::try_cast].
    /// 
    /// If the given parameter is a [`Tag`], and the [tag's ID][Tag::id]
    /// does not match the [list's element ID][TagList::elem_id], then
    /// insertion will succeed only if:
    /// - the list is [empty][TagList::is_empty], or;
    /// - the tag's ID and the list's element ID are both numeric,
    ///   allowing a [cast][Tag::try_cast].
    /// 
    /// If the insertion succeeds, this method returns the unit value.
    /// 
    /// # Panics
    /// This method will panic if the given value does not convert to a
    /// [`Tag`] that can be correctly [cast][Tag::try_cast] into the type
    /// of elements in this [`TagList`]. You should therefore be careful
    /// using this method and only use it if you can fully ensure that
    /// the given value can be correctly inserted into the list. If you
    /// cannot be sure of that, use [`TagList::try_push`] instead, which
    /// does the same thing but returns a [`Result`] instead of
    /// panicking.
    /// 
    /// This method will also panic if the given index is out of range.
    /// That is, the index is strictly greater than
    /// [`len`][TagList::len].
    /// 
    /// # See Also
    /// - If you want to insert an element at the end, you can use
    ///   [`TagList::push`].
    /// - If you want to get a [mutable reference][TagRefMut] to the
    ///   inserted element, you can use [`TagList::insert_mut`].
    /// - If you aren't sure that the given value can be inserted
    ///   infallibly and want to handle a failing insertion, you can use
    ///   [`TagList::try_insert`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// list.insert(0, Tag::int(4));
    /// assert_eq!(list, tag_list![4.0, 1.61, 2.71, 3.14]);
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```should_panic
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work and panics
    /// list.insert(0, Tag::string("foo"));
    /// ```
    pub fn insert(&mut self, index: usize, value: impl Into<Tag>) {
        ok_or_panic("insert", self.try_insert(index, value))
    }

    /// Tries to insert a value at the given index into a [`TagList`],
    /// and returns a [mutable reference][TagRefMut] to the inserted
    /// payload when successful. The given index must be no greater
    /// than [the length of the list][TagList::len]. If the index is
    /// exactly the list's length, then this method behaves exactly
    /// like [`TagList::try_push_mut`].
    /// 
    /// This method follows the same type conversion and error logic as
    /// [`TagList::try_insert`], with the only difference being that a
    /// mutable reference to the inserted element is returned, instead
    /// of a unit value. See [`try_insert`][`TagList::try_insert`] for
    /// the exact details under which insertion succeeds or fails.
    /// 
    /// # Panics
    /// This method will panic if the given index is out of range. That
    /// is, the index is strictly greater than [`len`][TagList::len].
    /// 
    /// # See Also
    /// - If you want to insert an element at the end, you can use
    ///   [`TagList::try_push_mut`].
    /// - If you do not need a reference to the inserted element, you
    ///   can use [`TagList::try_insert`].
    /// - If you are fully sure that the given value can be inserted
    ///   infallibly and don't want to bother handling an error error
    ///   that cannot happen, you can use [`TagList::insert_mut`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// use rsnbt::tag::{Tag, tag_list, TagRefMut};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// let inserted = list.try_insert_mut(0, Tag::int(4))?;
    /// assert_eq!(inserted, TagRefMut::Double(&mut 4.0));
    /// 
    /// let reference = inserted.double_mut()?;
    /// *reference = 6.28;
    /// assert_eq!(list, tag_list![6.28, 1.61, 2.71, 3.14]);
    /// 
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work
    /// assert_matches!(list.try_insert_mut(0, Tag::string("foo")), Err(_));
    /// ```
    pub fn try_insert_mut(&mut self, index: usize, value: impl Into<Tag>) -> Result<TagRefMut<'_>, TagCastError<Tag>> {
        let tag = value.into();

        let mut_ref = if let Some(id) = self.elem_id() {
            let cast = tag.try_cast(id)?;
            match (self, cast) {
                (TagList::Bytes(vs), Tag::Byte(v)) => TagRefMut::Byte(vs.insert_mut(index, v)),
                (TagList::Shorts(vs), Tag::Short(v)) => TagRefMut::Short(vs.insert_mut(index, v)),
                (TagList::Ints(vs), Tag::Int(v)) => TagRefMut::Int(vs.insert_mut(index, v)),
                (TagList::Longs(vs), Tag::Long(v)) => TagRefMut::Long(vs.insert_mut(index, v)),
                (TagList::Floats(vs), Tag::Float(v)) => TagRefMut::Float(vs.insert_mut(index, v)),
                (TagList::Doubles(vs), Tag::Double(v)) => TagRefMut::Double(vs.insert_mut(index, v)),
                (TagList::Strings(vs), Tag::String(v)) => TagRefMut::String(vs.insert_mut(index, v)),
                (TagList::ByteArrays(vs), Tag::ByteArray(v)) => TagRefMut::ByteArray(vs.insert_mut(index, v)),
                (TagList::IntArrays(vs), Tag::IntArray(v)) => TagRefMut::IntArray(vs.insert_mut(index, v)),
                (TagList::LongArrays(vs), Tag::LongArray(v)) => TagRefMut::LongArray(vs.insert_mut(index, v)),
                (TagList::Lists(vs), Tag::List(v)) => TagRefMut::List(vs.insert_mut(index, v)),
                (TagList::Compounds(vs), Tag::Compound(v)) => TagRefMut::Compound(vs.insert_mut(index, v)),

                _ => unreachable!(),
            }
        } else {
            macro_rules! make_list_and_return_mut_ref {
                ($id:ident, $ids:ident, $v:expr) => {{
                    *self = TagList::$ids(Vec::new());

                    match self {
                        TagList::$ids(vs) => TagRefMut::$id(vs.insert_mut(index, $v)),
                        _ => unreachable!(),
                    }
                }};
            }

            match tag {
                Tag::Byte(v) => make_list_and_return_mut_ref!(Byte, Bytes, v),
                Tag::Short(v) => make_list_and_return_mut_ref!(Short, Shorts, v),
                Tag::Int(v) => make_list_and_return_mut_ref!(Int, Ints, v),
                Tag::Long(v) => make_list_and_return_mut_ref!(Long, Longs, v),
                Tag::Float(v) => make_list_and_return_mut_ref!(Float, Floats, v),
                Tag::Double(v) => make_list_and_return_mut_ref!(Double, Doubles, v),
                Tag::String(v) => make_list_and_return_mut_ref!(String, Strings, v),
                Tag::ByteArray(v) => make_list_and_return_mut_ref!(ByteArray, ByteArrays, v),
                Tag::IntArray(v) => make_list_and_return_mut_ref!(IntArray, IntArrays, v),
                Tag::LongArray(v) => make_list_and_return_mut_ref!(LongArray, LongArrays, v),
                Tag::List(v) => make_list_and_return_mut_ref!(List, Lists, v),
                Tag::Compound(v) => make_list_and_return_mut_ref!(Compound, Compounds, v),
            }
        };

        Ok(mut_ref)
    }

    /// Inserts a value at the given index into a [`TagList`], and
    /// returns a [mutable reference][TagRefMut] to the inserted
    /// payload. The given index must be no greater than [the length
    /// of the list][TagList::len]. If the index is exactly the list's
    /// length, then this method behaves exactly like
    /// [`TagList::push_mut`].
    /// 
    /// This method follows the same type conversion and panic logic as
    /// [`TagList::insert`], with the only difference being that a
    /// mutable reference to the inserted element is returned, instead
    /// of a unit value. See [`insert`][`TagList::insert`] for the
    /// exact details under which insertion succeeds or fails.
    /// 
    /// # Panics
    /// Unlike [`try_insert_mut`][`TagList::try_insert_mut`], this method
    /// will panic when conversion to the correct type fails for the
    /// given value. See [`insert`][`tagList::insert`] for details.
    /// 
    /// This method will also panic if the given index is out of range.
    /// That is, the index is strictly greater than
    /// [`len`][TagList::len].
    /// 
    /// # See Also
    /// - If you want to insert an element at the end, you can use
    ///   [`TagList::push_mut`].
    /// - If you do not need a reference to the inserted element, you
    ///   can use [`TagList::insert`].
    /// - If you aren't sure that the given value can be inserted
    ///   infallibly and want to handle a failing insertion, you can use
    ///   [`TagList::try_insert_mut`].
    /// 
    /// # Examples
    /// You can insert any number into a list of numbers, as these are
    /// coerced automatically:
    /// ```
    /// use rsnbt::tag::{Tag, tag_list, TagRefMut};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this works even though the list contains doubles
    /// let inserted = list.insert_mut(0, Tag::int(4));
    /// assert_eq!(inserted, TagRefMut::Double(&mut 4.0));
    /// 
    /// let reference = inserted.double_mut()?;
    /// *reference = 6.28;
    /// assert_eq!(list, tag_list![6.28, 1.61, 2.71, 3.14]);
    /// 
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    /// 
    /// <br/>
    /// 
    /// You cannot insert a string into a list of numbers:
    /// ```should_panic
    /// # use std::assert_matches;
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// // this does not work and panics
    /// list.insert_mut(0, Tag::string("foo"));
    /// ```
    pub fn insert_mut(&mut self, index: usize, value: impl Into<Tag>) -> TagRefMut<'_> {
        ok_or_panic("insert", self.try_insert_mut(index, value))
    }

    /// Removes an element from a [`TagList`], given its index. The
    /// element is moved out of the list and returned as a [`Tag`]. The
    /// given index must be strictly less than the [length of the
    /// list][`TagList::len`].
    /// 
    /// When an element is removed from the middle of the list, then all
    /// elements with higher indices must be moved one index back. This
    /// means that removing elements is not a constant-time operation. If
    /// performance is key, then use this method sparingly. Note that
    /// [`pop`][TagList::pop] does run in constant time.
    /// 
    /// If the given index is `len - 1`, then this method behaves much
    /// like [`pop`], except that a panic will occur when the list is
    /// empty.
    /// 
    /// # Panics
    /// This method will panic when the given index is out of range, that
    /// is, it will panic when `index` is greater than
    /// [`len`][TagList::len].
    /// 
    /// # See also
    /// - If you want to remove the last element of the list, you can use
    ///   [`TagList::pop`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// let element = list.remove(1);
    /// assert_eq!(element, Tag::double(2.71));
    /// assert_eq!(list, tag_list![1.61, 3.14]);
    /// ```
    pub fn remove(&mut self, index: usize) -> Tag {
        if self.is_empty() {
            panic!("TagList is empty, index is always out of range");
        };

        match self {
            TagList::Empty => unreachable!(),

            TagList::Bytes(vs) => Tag::Byte(vs.remove(index)),
            TagList::Shorts(vs) => Tag::Short(vs.remove(index)),
            TagList::Ints(vs) => Tag::Int(vs.remove(index)),
            TagList::Longs(vs) => Tag::Long(vs.remove(index)),
            TagList::Floats(vs) => Tag::Float(vs.remove(index)),
            TagList::Doubles(vs) => Tag::Double(vs.remove(index)),
            TagList::Strings(vs) => Tag::String(vs.remove(index)),
            TagList::ByteArrays(vs) => Tag::ByteArray(vs.remove(index)),
            TagList::IntArrays(vs) => Tag::IntArray(vs.remove(index)),
            TagList::LongArrays(vs) => Tag::LongArray(vs.remove(index)),
            TagList::Lists(vs) => Tag::List(vs.remove(index)),
            TagList::Compounds(vs) => Tag::Compound(vs.remove(index)),
        }
    }

    /// Removes the last element from a [`TagList`], if it has one. The
    /// element is returned in an [`Option`], and [`None`] is returned
    /// when the list is empty.
    /// 
    /// Unlike [`TagList::remove`], popping an element is a constant-time
    /// operation.
    /// 
    /// # See Also
    /// - If you need to remove an element from a specific index, you can
    ///   use [`TagList::remove`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list = tag_list![1.61, 2.71, 3.14];
    /// 
    /// let element = list.pop();
    /// assert_eq!(element, Some(Tag::double(3.14)));
    /// assert_eq!(list, tag_list![1.61, 2.71]);
    /// ```
    pub fn pop(&mut self) -> Option<Tag> where {
        Some(match self {
            TagList::Empty => None?,

            TagList::Bytes(vs) => Tag::Byte(vs.pop()?),
            TagList::Shorts(vs) => Tag::Short(vs.pop()?),
            TagList::Ints(vs) => Tag::Int(vs.pop()?),
            TagList::Longs(vs) => Tag::Long(vs.pop()?),
            TagList::Floats(vs) => Tag::Float(vs.pop()?),
            TagList::Doubles(vs) => Tag::Double(vs.pop()?),
            TagList::Strings(vs) => Tag::String(vs.pop()?),
            TagList::ByteArrays(vs) => Tag::ByteArray(vs.pop()?),
            TagList::IntArrays(vs) => Tag::IntArray(vs.pop()?),
            TagList::LongArrays(vs) => Tag::LongArray(vs.pop()?),
            TagList::Lists(vs) => Tag::List(vs.pop()?),
            TagList::Compounds(vs) => Tag::Compound(vs.pop()?),
        })
    }

    /// Returns an [`Iterator`] that borrows the elements of a [`TagList`].
    /// That is, the iterator iterates [`TagRef`] values. The elements
    /// are iterated in the order they appear in the list, from lowest
    /// index to highest index.
    /// 
    /// # See Also
    /// - If you need to iterate mutable references, you can use
    ///   [`TagList::iter_mut`].
    /// - If you own the list and want to move out the values in order,
    ///   you can use [`TagList::into_iter`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagRef, tag_list};
    /// 
    /// let list = tag_list![1.61, 2.71, 3.14];
    /// let mut iter = list.iter();
    ///
    /// assert_eq!(Some(TagRef::Double(&1.61)), iter.next());
    /// assert_eq!(Some(TagRef::Double(&2.71)), iter.next());
    /// assert_eq!(Some(TagRef::Double(&3.14)), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            internal: match self {
                TagList::Empty => IterInternal::Empty,
                TagList::Bytes(vs) => IterInternal::Byte(vs.iter()),
                TagList::Shorts(vs) => IterInternal::Short(vs.iter()),
                TagList::Ints(vs) => IterInternal::Int(vs.iter()),
                TagList::Longs(vs) => IterInternal::Long(vs.iter()),
                TagList::Floats(vs) => IterInternal::Float(vs.iter()),
                TagList::Doubles(vs) => IterInternal::Double(vs.iter()),
                TagList::Strings(vs) => IterInternal::String(vs.iter()),
                TagList::ByteArrays(vs) => IterInternal::ByteArray(vs.iter()),
                TagList::IntArrays(vs) => IterInternal::IntArray(vs.iter()),
                TagList::LongArrays(vs) => IterInternal::LongArray(vs.iter()),
                TagList::Lists(vs) => IterInternal::List(vs.iter()),
                TagList::Compounds(vs) => IterInternal::Compound(vs.iter()),
            },
        }
    }

    /// Returns an [`Iterator`] that mutably borrows the elements of a
    /// [`TagList`]. That is, the iterator iterates [`TagRef`] values. The
    /// elements are iterated in the order they appear in the list, from
    /// lowest index to highest index.
    /// 
    /// # See Also
    /// - If you don't need to mutate the values of the list, you can use
    ///   [`TagList::iter`].
    /// - If you own the list and want to move out the values in order,
    ///   you can use [`TagList::into_iter`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list = tag_list![1, 2, 3];
    /// let mut iter = list.iter_mut();
    ///
    /// for r in iter {
    ///     let r = r.int_mut()?;
    ///     *r += 1;
    /// }
    /// 
    /// assert_eq!(list, tag_list![2, 3, 4]);
    /// 
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            internal: match self {
                TagList::Empty => IterMutInternal::Empty,
                TagList::Bytes(vs) => IterMutInternal::Byte(vs.iter_mut()),
                TagList::Shorts(vs) => IterMutInternal::Short(vs.iter_mut()),
                TagList::Ints(vs) => IterMutInternal::Int(vs.iter_mut()),
                TagList::Longs(vs) => IterMutInternal::Long(vs.iter_mut()),
                TagList::Floats(vs) => IterMutInternal::Float(vs.iter_mut()),
                TagList::Doubles(vs) => IterMutInternal::Double(vs.iter_mut()),
                TagList::Strings(vs) => IterMutInternal::String(vs.iter_mut()),
                TagList::ByteArrays(vs) => IterMutInternal::ByteArray(vs.iter_mut()),
                TagList::IntArrays(vs) => IterMutInternal::IntArray(vs.iter_mut()),
                TagList::LongArrays(vs) => IterMutInternal::LongArray(vs.iter_mut()),
                TagList::Lists(vs) => IterMutInternal::List(vs.iter_mut()),
                TagList::Compounds(vs) => IterMutInternal::Compound(vs.iter_mut()),
            },
        }
    }

    /// Tries to append the contents of the given [`TagList`] to another.
    /// This removes all elements from the given [`TagList`] and moves
    /// them into the receiver [`TagList`].
    /// 
    /// This method follows the same type conversion and error logic as
    /// methods like [`try_insert`][TagList::try_insert] and
    /// [`try_push`][TagList::try_push]. That is, [tag
    /// casting][Tag::try_cast] will take place if needed, and [`Err`] is
    /// returned when the type of elements in the given list do not match
    /// those in the receiver. Note that values will be cast to the type
    /// of the receiver.
    /// 
    /// The given list will be emptied, but its type will be preserved,
    /// meaning that appending a [`TagList::Ints`] successfully will
    /// result in the given list staying [`TagList::Ints`] but without
    /// any elements.
    /// 
    /// If both lists are empty, then the type of the receiver will
    /// remain the same as it was, not adapting to the new type.
    /// 
    /// # See Also
    /// - If you are fully sure that the given list can be appended
    ///   infallibly and don't want to bother handling an error
    ///   that cannot happen, you can use [`TagList::append`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// # use rsnbt::tag::cast::TagMismatchError;
    /// 
    /// let mut list1 = tag_list![1.61, 2.71, 3.14];
    /// let mut list2 = tag_list![4.0, 5.0, 6.0];
    /// 
    /// list1.try_append(&mut list2)?;
    /// assert_eq!(list1, tag_list![1.61, 2.71, 3.14, 4.0, 5.0, 6.0]);
    /// assert!(list2.is_empty());
    ///
    /// # Ok::<(), TagMismatchError>(())
    /// ```
    pub fn try_append(&mut self, other: &mut TagList) -> Result<(), TagMismatchError> {
        // TODO this currently performs no casting.
        if other.is_empty() {
            return Ok(());
        };

        if self.is_empty() {
            // Create an empty list to match the type of `other`
            let default = match other {
                TagList::Empty => TagList::Empty,
                TagList::Bytes(_) => TagList::Bytes(vec![]),
                TagList::Shorts(_) => TagList::Shorts(vec![]),
                TagList::Ints(_) => TagList::Ints(vec![]),
                TagList::Longs(_) => TagList::Longs(vec![]),
                TagList::Floats(_) => TagList::Floats(vec![]),
                TagList::Doubles(_) => TagList::Doubles(vec![]),
                TagList::Strings(_) => TagList::Strings(vec![]),
                TagList::ByteArrays(_) => TagList::ByteArrays(vec![]),
                TagList::IntArrays(_) => TagList::IntArrays(vec![]),
                TagList::LongArrays(_) => TagList::LongArrays(vec![]),
                TagList::Lists(_) => TagList::Lists(vec![]),
                TagList::Compounds(_) => TagList::Compounds(vec![]),
            };

            *self = std::mem::replace(other, default);
            return Ok(());
        };

        let self_id = self.elem_id().unwrap();

        // take the value so it resets to Empty
        match (self, other) {
            (Self::Bytes(l), Self::Bytes(r)) => l.append(r),
            (Self::Shorts(l), Self::Shorts(r)) => l.append(r),
            (Self::Ints(l), Self::Ints(r)) => l.append(r),
            (Self::Longs(l), Self::Longs(r)) => l.append(r),
            (Self::Floats(l), Self::Floats(r)) => l.append(r),
            (Self::Doubles(l), Self::Doubles(r)) => l.append(r),
            (Self::Strings(l), Self::Strings(r)) => l.append(r),
            (Self::ByteArrays(l), Self::ByteArrays(r)) => l.append(r),
            (Self::IntArrays(l), Self::IntArrays(r)) => l.append(r),
            (Self::LongArrays(l), Self::LongArrays(r)) => l.append(r),
            (Self::Lists(l), Self::Lists(r)) => l.append(r),
            (Self::Compounds(l), Self::Compounds(r)) => l.append(r),

            (_, other) => {
                Err(TagMismatchError::new(self_id, other.elem_id().unwrap()))?
            },
        }

        Ok(())
    }

    /// Appends the contents of the given [`TagList`] to another.
    /// This removes all elements from the given [`TagList`] and moves
    /// them into the receiver [`TagList`].
    /// 
    /// This method follows the same type conversion and panic logic as
    /// methods like [`insert`][TagList::insert] and
    /// [`push`][TagList::push]. That is, [tag casting][Tag::try_cast]
    /// will take place if needed, and the method panics when the type
    /// of elements in the given list do not match those in the receiver.
    /// Note that values will be cast to the type of the receiver.
    /// 
    /// The given list will be emptied, but its type will be preserved,
    /// meaning that appending a [`TagList::Ints`] successfully will
    /// result in the given list staying [`TagList::Ints`] but without
    /// any elements.
    /// 
    /// If both lists are empty, then the type of the receiver will
    /// remain the same as it was, not adapting to the new type.
    /// 
    /// # Panics
    /// This method will panic when the tags in the given list cannot be
    /// cast to the type of the receiver. If you cannot guarantee this,
    /// use [`TagList::try_append`].
    /// 
    /// # See Also
    /// - If you aren't sure that the given list can be appended
    ///   infallibly and want to handle a failing insertion, you can use
    ///   [`TagList::try_append`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let mut list1 = tag_list![1.61, 2.71, 3.14];
    /// let mut list2 = tag_list![4.0, 5.0, 6.0];
    /// 
    /// // appending ints to doubles will cast the ints to doubles
    /// list1.append(&mut list2);
    /// assert_eq!(list1, tag_list![1.61, 2.71, 3.14, 4.0, 5.0, 6.0]);
    /// assert!(list2.is_empty());
    /// ```
    pub fn append(&mut self, other: &mut TagList) {
        ok_or_panic("append", self.try_append(other));
    }
}

/// [`PartialEq`] implementation for [`TagList`]. Since floats do not
/// implement [`Eq`], [`TagList`] neither implements [`Eq`].
/// 
/// In addition, note that empty lists are always equal, regardless of
/// type.
impl PartialEq for TagList {
    fn eq(&self, other: &Self) -> bool {
        // Special case since any enum variant with an empty vector
        // is considered the same as the empty variant
        if self.is_empty() && other.is_empty() {
            return true;
        }

        match (self, other) {
            (TagList::Empty, TagList::Empty) => true,
            (TagList::Bytes(l), TagList::Bytes(r)) => l == r,
            (TagList::Shorts(l), TagList::Shorts(r)) => l == r,
            (TagList::Ints(l), TagList::Ints(r)) => l == r,
            (TagList::Longs(l), TagList::Longs(r)) => l == r,
            (TagList::Floats(l), TagList::Floats(r)) => l == r,
            (TagList::Doubles(l), TagList::Doubles(r)) => l == r,
            (TagList::Strings(l), TagList::Strings(r)) => l == r,
            (TagList::ByteArrays(l), TagList::ByteArrays(r)) => l == r,
            (TagList::IntArrays(l), TagList::IntArrays(r)) => l == r,
            (TagList::LongArrays(l), TagList::LongArrays(r)) => l == r,
            (TagList::Lists(l), TagList::Lists(r)) => l == r,
            (TagList::Compounds(l), TagList::Compounds(r)) => l == r,

            _ => false,
        }
    }
}

/// [`IntoIterator`] implementation for [`TagList`].
/// 
/// [`TagList`]s iterate [`Tag`]s.
impl IntoIterator for TagList {
    type Item = Tag;
    type IntoIter = IntoIter;

    /// Returns an [`Iterator`] that moves the elements from a [`TagList`].
    /// That is, the iterator iterates [`TagRef`] values. The
    /// elements are iterated in the order they appear in the list, from
    /// lowest index to highest index.
    /// 
    /// # See Also
    /// - If you don't need to mutate the values of the list, you can use
    ///   [`TagList::iter`].
    /// - If you own the list and want to move out the values in order,
    ///   you can use [`TagList::into_iter`].
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, tag_list};
    /// 
    /// let list = tag_list![1.61, 2.71, 3.14];
    /// let mut iter = list.into_iter();
    ///
    /// assert_eq!(Some(Tag::Double(1.61)), iter.next());
    /// assert_eq!(Some(Tag::Double(2.71)), iter.next());
    /// assert_eq!(Some(Tag::Double(3.14)), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            internal: match self {
                TagList::Empty => IntoIterInternal::Empty,
                TagList::Bytes(vs) => IntoIterInternal::Byte(vs.into_iter()),
                TagList::Shorts(vs) => IntoIterInternal::Short(vs.into_iter()),
                TagList::Ints(vs) => IntoIterInternal::Int(vs.into_iter()),
                TagList::Longs(vs) => IntoIterInternal::Long(vs.into_iter()),
                TagList::Floats(vs) => IntoIterInternal::Float(vs.into_iter()),
                TagList::Doubles(vs) => IntoIterInternal::Double(vs.into_iter()),
                TagList::Strings(vs) => IntoIterInternal::String(vs.into_iter()),
                TagList::ByteArrays(vs) => IntoIterInternal::ByteArray(vs.into_iter()),
                TagList::IntArrays(vs) => IntoIterInternal::IntArray(vs.into_iter()),
                TagList::LongArrays(vs) => IntoIterInternal::LongArray(vs.into_iter()),
                TagList::Lists(vs) => IntoIterInternal::List(vs.into_iter()),
                TagList::Compounds(vs) => IntoIterInternal::Compound(vs.into_iter()),
            },
        }
    }
}

impl Debug for TagList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty []"),

            Self::Bytes(vs) => {
                write!(f, "Bytes ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Shorts(vs) => {
                write!(f, "Shorts ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Ints(vs) => {
                write!(f, "Ints ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Longs(vs) => {
                write!(f, "Longs ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Floats(vs) => {
                write!(f, "Floats ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Doubles(vs) => {
                write!(f, "Doubles ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Strings(vs) => {
                write!(f, "Strings ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::ByteArrays(vs) => {
                write!(f, "ByteArrays ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::IntArrays(vs) => {
                write!(f, "IntArrays ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::LongArrays(vs) => {
                write!(f, "LongArrays ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Lists(vs) => {
                write!(f, "Lists ")?;
                f.debug_list().entries(vs).finish()
            },
            Self::Compounds(vs) => {
                write!(f, "Compounds ")?;
                f.debug_list().entries(vs).finish()
            },
        }
    }
}



fn ok_or_panic<T>(op: &'static str, result: Result<T, impl Into<TagMismatchError>>) -> T {
    match result {
        Ok(val) => val,
        Err(err) => panic!("{} failed due to {}", op, err.into()),
    }
}


// ================================================================================================
//
//  TAG CONVERSION
// 
// ================================================================================================

impl Tag {
    pub fn list<T>(v: T) -> Tag
    where
        T: Into<TagList>,
    {
        Tag::List(v.into())
    }

    pub fn into_tag_list(self) -> Result<TagList, TagCastError<Self>> {
        if let Self::List(v) = self {
            Ok(v)
        } else {
            Err(TagCastError::new(TagId::List, self.id(), self))
        }
    }
}


impl From<TagList> for Tag {
    fn from(value: TagList) -> Self {
        Self::List(value)
    }
}





macro_rules! from {
    ($kind:ident, $type:ty $([$lt:lifetime])?, |$name:ident| $expr:expr) => {
        impl$(<$lt>)? From<Vec<$type>> for TagList {
            fn from(value: Vec<$type>) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = value;
                    TagList::$kind($expr)
                }
            }
        }

        impl<$($lt, )? const N: usize> From<[$type; N]> for TagList {
            fn from(value: [$type; N]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind($expr)
                }
            }
        }

        impl<$($lt, )? const N: usize> From<&[$type; N]> for TagList {
            fn from(value: &[$type; N]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind($expr)
                }
            }
        }

        impl$(<$lt>)? From<&[$type]> for TagList {
            fn from(value: &[$type]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind($expr)
                }
            }
        }

        impl$(<$lt>)? FromIterator<$type> for TagList {
            fn from_iter<T: IntoIterator<Item = $type>>(iter: T) -> Self {
                Self::from(Vec::from_iter(iter))
            }
        }
    };
}

from!(Bytes, i8, |value| value);
from!(Shorts, i16, |value| value);
from!(Ints, i32, |value| value);
from!(Longs, i64, |value| value);
from!(Bytes, u8, |value| reinterpret_vec!(value, u8 => i8));
from!(Shorts, u16, |value| reinterpret_vec!(value, u16 => i16));
from!(Ints, u32, |value| reinterpret_vec!(value, u32 => i32));
from!(Longs, u64, |value| reinterpret_vec!(value, u64 => i64));
from!(Floats, f32, |value| value);
from!(Doubles, f64, |value| value);
from!(Strings, String, |value| value);
from!(Strings, &'a str ['a], |value| value.iter().map(|v| String::from(*v)).collect());
from!(ByteArrays, Vec<i8>, |value| value);
from!(IntArrays, Vec<i32>, |value| value);
from!(LongArrays, Vec<i64>, |value| value);
from!(ByteArrays, Vec<u8>, |value| reinterpret_vec_vec!(value, u8 => i8));
from!(IntArrays, Vec<u32>, |value| reinterpret_vec_vec!(value, u32 => i32));
from!(LongArrays, Vec<u64>, |value| reinterpret_vec_vec!(value, u64 => i64));
from!(Lists, TagList, |value| value);
from!(Compounds, TagMap, |value| value);



// ================================================================================================
//
//  ITERATORS
// 
// ================================================================================================

// A pattern we use here is to have a private enum as to not expose the different constants. The
// user does not need to use these since they can match on TagRef/TagRefMut/Tag.

enum IterInternal<'a> {
    Empty,
    Byte(slice::Iter<'a, i8>),
    Short(slice::Iter<'a, i16>),
    Int(slice::Iter<'a, i32>),
    Long(slice::Iter<'a, i64>),
    Float(slice::Iter<'a, f32>),
    Double(slice::Iter<'a, f64>),
    String(slice::Iter<'a, String>),
    ByteArray(slice::Iter<'a, Vec<i8>>),
    IntArray(slice::Iter<'a, Vec<i32>>),
    LongArray(slice::Iter<'a, Vec<i64>>),
    List(slice::Iter<'a, TagList>),
    Compound(slice::Iter<'a, TagMap>),
}


/// The type of [`Iterator`] that is returned from [`TagList::iter`].
/// It borrows elements of a [`TagList`] immutably.
/// 
/// # Examples
/// ```
/// use rsnbt::tag::{Tag, TagRef, tag_list};
/// 
/// let list = tag_list![1.61, 2.71, 3.14];
/// let mut iter = list.iter();
///
/// assert_eq!(Some(TagRef::Double(&1.61)), iter.next());
/// assert_eq!(Some(TagRef::Double(&2.71)), iter.next());
/// assert_eq!(Some(TagRef::Double(&3.14)), iter.next());
/// assert_eq!(None, iter.next());
/// ```
pub struct Iter<'a> {
    internal: IterInternal<'a>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = TagRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            IterInternal::Empty => None,
            IterInternal::Byte(iter) => Some(TagRef::Byte(iter.next()?)),
            IterInternal::Short(iter) => Some(TagRef::Short(iter.next()?)),
            IterInternal::Int(iter) => Some(TagRef::Int(iter.next()?)),
            IterInternal::Long(iter) => Some(TagRef::Long(iter.next()?)),
            IterInternal::Float(iter) => Some(TagRef::Float(iter.next()?)),
            IterInternal::Double(iter) => Some(TagRef::Double(iter.next()?)),
            IterInternal::String(iter) => Some(TagRef::String(iter.next()?)),
            IterInternal::ByteArray(iter) => Some(TagRef::ByteArray(iter.next()?)),
            IterInternal::IntArray(iter) => Some(TagRef::IntArray(iter.next()?)),
            IterInternal::LongArray(iter) => Some(TagRef::LongArray(iter.next()?)),
            IterInternal::List(iter) => Some(TagRef::List(iter.next()?)),
            IterInternal::Compound(iter) => Some(TagRef::Compound(iter.next()?)),
        }
    }
}




enum IterMutInternal<'a> {
    Empty,
    Byte(slice::IterMut<'a, i8>),
    Short(slice::IterMut<'a, i16>),
    Int(slice::IterMut<'a, i32>),
    Long(slice::IterMut<'a, i64>),
    Float(slice::IterMut<'a, f32>),
    Double(slice::IterMut<'a, f64>),
    String(slice::IterMut<'a, String>),
    ByteArray(slice::IterMut<'a, Vec<i8>>),
    IntArray(slice::IterMut<'a, Vec<i32>>),
    LongArray(slice::IterMut<'a, Vec<i64>>),
    List(slice::IterMut<'a, TagList>),
    Compound(slice::IterMut<'a, TagMap>),
}

/// The type of [`Iterator`] that is returned from [`TagList::iter_mut`].
/// It borrows elements of a [`TagList`] mutably.
/// 
/// # Examples
/// ```
/// use rsnbt::tag::{Tag, tag_list};
/// # use rsnbt::tag::cast::TagMismatchError;
/// 
/// let mut list = tag_list![1, 2, 3];
/// let mut iter = list.iter_mut();
///
/// for r in iter {
///     let r = r.int_mut()?;
///     *r += 1;
/// }
/// 
/// assert_eq!(list, tag_list![2, 3, 4]);
/// 
/// # Ok::<(), TagMismatchError>(())
/// ```
pub struct IterMut<'a> {
    internal: IterMutInternal<'a>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = TagRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            IterMutInternal::Empty => None,
            IterMutInternal::Byte(iter) => Some(TagRefMut::Byte(iter.next()?)),
            IterMutInternal::Short(iter) => Some(TagRefMut::Short(iter.next()?)),
            IterMutInternal::Int(iter) => Some(TagRefMut::Int(iter.next()?)),
            IterMutInternal::Long(iter) => Some(TagRefMut::Long(iter.next()?)),
            IterMutInternal::Float(iter) => Some(TagRefMut::Float(iter.next()?)),
            IterMutInternal::Double(iter) => Some(TagRefMut::Double(iter.next()?)),
            IterMutInternal::String(iter) => Some(TagRefMut::String(iter.next()?)),
            IterMutInternal::ByteArray(iter) => Some(TagRefMut::ByteArray(iter.next()?)),
            IterMutInternal::IntArray(iter) => Some(TagRefMut::IntArray(iter.next()?)),
            IterMutInternal::LongArray(iter) => Some(TagRefMut::LongArray(iter.next()?)),
            IterMutInternal::List(iter) => Some(TagRefMut::List(iter.next()?)),
            IterMutInternal::Compound(iter) => Some(TagRefMut::Compound(iter.next()?)),
        }
    }
}




enum IntoIterInternal {
    Empty,
    Byte(vec::IntoIter<i8>),
    Short(vec::IntoIter<i16>),
    Int(vec::IntoIter<i32>),
    Long(vec::IntoIter<i64>),
    Float(vec::IntoIter<f32>),
    Double(vec::IntoIter<f64>),
    String(vec::IntoIter<String>),
    ByteArray(vec::IntoIter<Vec<i8>>),
    IntArray(vec::IntoIter<Vec<i32>>),
    LongArray(vec::IntoIter<Vec<i64>>),
    List(vec::IntoIter<TagList>),
    Compound(vec::IntoIter<TagMap>),
}


/// The type of [`Iterator`] that is returned from [`TagList::into_iter`].
/// It moves elements from a [`TagList`].
/// 
/// # Examples
/// ```
/// use rsnbt::tag::{Tag, tag_list};
/// 
/// let list = tag_list![1.61, 2.71, 3.14];
/// let mut iter = list.into_iter();
///
/// assert_eq!(Some(Tag::Double(1.61)), iter.next());
/// assert_eq!(Some(Tag::Double(2.71)), iter.next());
/// assert_eq!(Some(Tag::Double(3.14)), iter.next());
/// assert_eq!(None, iter.next());
/// ```
pub struct IntoIter {
    internal: IntoIterInternal,
}

impl Iterator for IntoIter {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            IntoIterInternal::Empty => None,
            IntoIterInternal::Byte(iter) => Some(Tag::Byte(iter.next()?)),
            IntoIterInternal::Short(iter) => Some(Tag::Short(iter.next()?)),
            IntoIterInternal::Int(iter) => Some(Tag::Int(iter.next()?)),
            IntoIterInternal::Long(iter) => Some(Tag::Long(iter.next()?)),
            IntoIterInternal::Float(iter) => Some(Tag::Float(iter.next()?)),
            IntoIterInternal::Double(iter) => Some(Tag::Double(iter.next()?)),
            IntoIterInternal::String(iter) => Some(Tag::String(iter.next()?)),
            IntoIterInternal::ByteArray(iter) => Some(Tag::ByteArray(iter.next()?)),
            IntoIterInternal::IntArray(iter) => Some(Tag::IntArray(iter.next()?)),
            IntoIterInternal::LongArray(iter) => Some(Tag::LongArray(iter.next()?)),
            IntoIterInternal::List(iter) => Some(Tag::List(iter.next()?)),
            IntoIterInternal::Compound(iter) => Some(Tag::Compound(iter.next()?)),
        }
    }
}



// ================================================================================================
//
//  MACRO
// 
// ================================================================================================


/// Constructs a [`TagList`]. The macro accepts a simple list-like syntax similar to SNBT.
/// 
/// The type of list it creates is derived from the first element. If no elements are given,
/// the type of the list is [`TagList::Empty`]. You may optionally specify the type of the
/// list explicitly, see the example below.
/// 
/// # Examples
/// ```no_run
/// use rsnbt::tag::tag_list;
/// 
/// let empty = tag_list![]; // -> creates a TagList::Empty
/// let ints = tag_list![1, 2, 3];
/// let doubles = tag_list![1.61, 2.71, 3.14];
/// let strings = tag_list!["foo", "bar", "baz"];
/// 
/// // You can optionally specify the list type.
/// // These names match the similarly named associated
/// // functions that instantiate lists of different types.
/// let empty_ints = tag_list![ints: ]; // -> creates an empty TagList::Ints
/// let floats = tag_list![floats: 1.61, 2.71, 3.14];
/// ```
#[macro_export]
macro_rules! tag_list {
    [] => {
        {
            use $crate::tag::TagList;
            TagList::new()
        }
    };

    [bytes: ] => {
        {
            use $crate::tag::TagList;
            TagList::bytes::<i8, _>([])
        }
    };

    [shorts: ] => {
        {
            use $crate::tag::TagList;
            TagList::shorts::<i16, _>([])
        }
    };

    [ints: ] => {
        {
            use $crate::tag::TagList;
            TagList::ints::<i32, _>([])
        }
    };

    [longs: ] => {
        {
            use $crate::tag::TagList;
            TagList::longs::<i64, _>([])
        }
    };

    [floats: ] => {
        {
            use $crate::tag::TagList;
            TagList::longs::<f32, _>([])
        }
    };

    [doubles: ] => {
        {
            use $crate::tag::TagList;
            TagList::longs::<f64, _>([])
        }
    };

    [strings: ] => {
        {
            use $crate::tag::TagList;
            TagList::strings::<String, _>([])
        }
    };

    [byte_arrays: ] => {
        {
            use $crate::tag::TagList;
            TagList::byte_arrays::<Vec<i8>, _>([])
        }
    };

    [int_arrays: ] => {
        {
            use $crate::tag::TagList;
            TagList::int_arrays::<Vec<i32>, _>([])
        }
    };

    [long_arrays: ] => {
        {
            use $crate::tag::TagList;
            TagList::long_arrays::<Vec<i64>, _>([])
        }
    };

    [lists: ] => {
        {
            use $crate::tag::TagList;
            TagList::lists::<TagList, _>([])
        }
    };

    [compounds: ] => {
        {
            use $crate::tag::{TagList, TagMap};
            TagList::compounds::<TagMap, _>([])
        }
    };

    [$($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::from(vec![$($expr),+])
        }
    };

    [bytes: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::bytes(vec![$($expr),+])
        }
    };

    [shorts: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::shorts(vec![$($expr),+])
        }
    };

    [ints: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::ints(vec![$($expr),+])
        }
    };

    [longs: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::longs(vec![$($expr),+])
        }
    };

    [floats: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::floats(vec![$($expr),+])
        }
    };

    [doubles: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::doubles(vec![$($expr),+])
        }
    };

    [strings: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::strings(vec![$($expr),+])
        }
    };

    [byte_arrays: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::byte_arrays(vec![$($expr),+])
        }
    };

    [int_arrays: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::int_arrays(vec![$($expr),+])
        }
    };

    [long_arrays: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::long_arrays(vec![$($expr),+])
        }
    };

    [lists: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::lists(vec![$($expr),+])
        }
    };

    [compounds: $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            
            TagList::compounds(vec![$($expr),+])
        }
    };
}

pub use tag_list;



