use core::slice;
use std::fmt::Debug;
use std::vec;

use super::unsafe_ops::*;
use super::*;

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
#[derive(Clone, Default)]
pub enum TagList {
    #[default]
    Empty,

    Bytes(Vec<i8>),
    Shorts(Vec<i16>),
    Ints(Vec<i32>),
    Longs(Vec<i64>),
    Floats(Vec<f32>),
    Doubles(Vec<f64>),
    Strings(Vec<String>),
    ByteArrays(Vec<Vec<i8>>),
    IntArrays(Vec<Vec<i32>>),
    LongArrays(Vec<Vec<i64>>),
    Lists(Vec<TagList>),
    Compounds(Vec<TagMap>),
}

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

/// Dispatch an invariant vector operation, that is the same and returns the same type regardless of
/// the list type, to whatever type the [TagList] is.
///
/// ```no_run
/// // Get the length of the vector. When empty (TagList::Empty), returns 0.
/// let len = dispatch_invariant!(self, |v| v.len(), 0);
///
/// // When the default is omitted, Default::default() is returned when empty.
/// let len_opt = dispatch_invariant!(self, |v| v.len());
/// ```
macro_rules! dispatch_invariant {
    ($param:expr, |$var:ident| $expr:expr, $default:expr) => {
        match $param {
            TagList::Empty => $default,
            TagList::Bytes($var) => $expr,
            TagList::Shorts($var) => $expr,
            TagList::Ints($var) => $expr,
            TagList::Longs($var) => $expr,
            TagList::Floats($var) => $expr,
            TagList::Doubles($var) => $expr,
            TagList::Strings($var) => $expr,
            TagList::ByteArrays($var) => $expr,
            TagList::IntArrays($var) => $expr,
            TagList::LongArrays($var) => $expr,
            TagList::Lists($var) => $expr,
            TagList::Compounds($var) => $expr,
        }
    };

    ($param:expr, |$var:ident| $expr:expr) => {
        match $param {
            TagList::Empty => Default::default(),
            TagList::Bytes($var) => $expr,
            TagList::Shorts($var) => $expr,
            TagList::Ints($var) => $expr,
            TagList::Longs($var) => $expr,
            TagList::Floats($var) => $expr,
            TagList::Doubles($var) => $expr,
            TagList::Strings($var) => $expr,
            TagList::ByteArrays($var) => $expr,
            TagList::IntArrays($var) => $expr,
            TagList::LongArrays($var) => $expr,
            TagList::Lists($var) => $expr,
            TagList::Compounds($var) => $expr,
        }
    };
}

fn panic_wrong_type<T>(op: &'static str, my_id: Option<TagId>, result: Result<T, Tag>) -> T {
    match result {
        Ok(val) => val,
        Err(id) => panic!(
            "Cannot {} {}: TagList is not empty and instead has elements of type {}",
            op,
            id.id(),
            my_id.unwrap()
        ),
    }
}

fn panic_wrong_id<T>(op: &'static str, my_id: Option<TagId>, result: Result<T, TagId>) -> T {
    match result {
        Ok(val) => val,
        Err(id) => panic!(
            "Cannot {} {}: TagList is not empty and instead has elements of type {}",
            op,
            id,
            my_id.unwrap()
        ),
    }
}

impl TagList {
    /// Creates a new, empty [`TagList`]. The returned value is [`TagList::Empty`].
    ///
    /// ```no_run
    /// use crate::tag::*;
    ///
    /// let mut list = TagList::new();
    /// // ...do stuff with list...
    /// ```
    pub const fn new() -> TagList {
        TagList::Empty
    }

    pub const fn is_empty(&self) -> bool {
        dispatch_invariant!(self, |v| v.is_empty(), true)
    }

    pub const fn len(&self) -> usize {
        dispatch_invariant!(self, |v| v.len(), 0)
    }

    pub const fn elem_id(&self) -> Option<TagId> {
        if self.is_empty() {
            return None;
        };

        Some(match self {
            TagList::Empty => panic!("unreachable"), // Caught by is_empty check

            TagList::Bytes(_) => TagId::Byte,
            TagList::Shorts(_) => TagId::Short,
            TagList::Ints(_) => TagId::Int,
            TagList::Longs(_) => TagId::Long,
            TagList::Floats(_) => TagId::Float,
            TagList::Doubles(_) => TagId::Double,
            TagList::Strings(_) => TagId::String,
            TagList::ByteArrays(_) => TagId::ByteArray,
            TagList::IntArrays(_) => TagId::IntArray,
            TagList::LongArrays(_) => TagId::LongArray,
            TagList::Lists(_) => TagId::List,
            TagList::Compounds(_) => TagId::Compound,
        })
    }

    pub const fn elem_id_byte(&self) -> u8 {
        match self.elem_id() {
            Some(id) => id.byte(),
            None => 0,
        }
    }

    pub fn reduce_if_empty(self) -> Self {
        if self.is_empty() {
            Self::Empty
        } else {
            self
        }
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }

    pub fn reverse(&mut self) {
        dispatch_invariant!(self, |v| v.reverse());
    }

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

    pub fn try_push<T>(&mut self, value: T) -> Result<(), Tag>
    where
        T: Into<Tag>,
    {
        self.try_push_mut(value)?;
        Ok(())
    }

    pub fn push<T>(&mut self, value: T) -> ()
    where
        T: Into<Tag>,
    {
        panic_wrong_type("insert", self.elem_id(), self.try_push(value))
    }

    pub fn try_push_mut<T>(&mut self, value: T) -> Result<TagRefMut<'_>, Tag>
    where
        T: Into<Tag>,
    {
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

    pub fn push_mut<T>(&mut self, value: T) -> TagRefMut<'_>
    where
        T: Into<Tag>,
    {
        panic_wrong_type("insert", self.elem_id(), self.try_push_mut(value))
    }

    pub fn try_insert<T>(&mut self, index: usize, value: T) -> Result<(), Tag>
    where
        T: Into<Tag>,
    {
        self.try_insert_mut(index, value)?;
        Ok(())
    }

    pub fn insert<T>(&mut self, index: usize, value: T) -> ()
    where
        T: Into<Tag>,
    {
        panic_wrong_type("insert", self.elem_id(), self.try_insert(index, value))
    }

    pub fn try_insert_mut<T>(&mut self, index: usize, value: T) -> Result<TagRefMut<'_>, Tag>
    where
        T: Into<Tag>,
    {
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

    pub fn insert_mut<T>(&mut self, index: usize, value: T) -> TagRefMut<'_>
    where
        T: Into<Tag>,
    {
        panic_wrong_type("insert", self.elem_id(), self.try_insert_mut(index, value))
    }

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

    pub fn vec<'a, T>(&'a self) -> Vec<T>
    where
        T: From<TagRef<'a>>,
    {
        Vec::from_iter(self.iter().map(|v| v.into()))
    }

    pub fn vec_mut<'a, T>(&'a mut self) -> Vec<T>
    where
        T: From<TagRefMut<'a>>,
    {
        Vec::from_iter(self.iter_mut().map(|v| v.into()))
    }

    pub fn into_vec<T>(self) -> Vec<T>
    where
        T: From<Tag>,
    {
        Vec::from_iter(self.into_iter().map(|v| v.into()))
    }

    pub fn try_vec<'a, T, E>(&'a self) -> Result<Vec<T>, E>
    where
        T: TryFrom<TagRef<'a>, Error = E>,
    {
        self.iter().map(|v| v.try_into()).collect()
    }

    pub fn try_vec_mut<'a, T, E>(&'a mut self) -> Result<Vec<T>, E>
    where
        T: TryFrom<TagRefMut<'a>, Error = E>,
    {
        self.iter_mut().map(|v| v.try_into()).collect()
    }

    pub fn try_into_vec<T, E>(self) -> Result<Vec<T>, E>
    where
        T: TryFrom<Tag, Error = E>,
    {
        self.into_iter().map(|v| v.try_into()).collect()
    }

    pub fn try_append(&mut self, other: &mut TagList) -> Result<(), TagId> {
        if self.is_empty() {
            *self = std::mem::take(other);
            return Ok(());
        };

        if other.is_empty() {
            return Ok(());
        };

        let self_id = self.elem_id().unwrap();

        match (self, std::mem::take(other)) {
            // Take the value so it resets to Empty
            (Self::Bytes(l), Self::Bytes(mut r)) => l.append(&mut r),
            (Self::Shorts(l), Self::Shorts(mut r)) => l.append(&mut r),
            (Self::Ints(l), Self::Ints(mut r)) => l.append(&mut r),
            (Self::Longs(l), Self::Longs(mut r)) => l.append(&mut r),
            (Self::Floats(l), Self::Floats(mut r)) => l.append(&mut r),
            (Self::Doubles(l), Self::Doubles(mut r)) => l.append(&mut r),
            (Self::Strings(l), Self::Strings(mut r)) => l.append(&mut r),
            (Self::ByteArrays(l), Self::ByteArrays(mut r)) => l.append(&mut r),
            (Self::IntArrays(l), Self::IntArrays(mut r)) => l.append(&mut r),
            (Self::LongArrays(l), Self::LongArrays(mut r)) => l.append(&mut r),
            (Self::Lists(l), Self::Lists(mut r)) => l.append(&mut r),
            (Self::Compounds(l), Self::Compounds(mut r)) => l.append(&mut r),

            _ => Err(self_id)?,
        }

        Ok(())
    }

    pub fn append(&mut self, other: &mut TagList) {
        panic_wrong_id("append", self.elem_id(), self.try_append(other));
    }

    pub fn singleton(value: Tag) -> Self {
        let mut list = Self::new();
        list.push(value);
        list
    }
}

impl Into<Vec<Tag>> for TagList {
    fn into(self) -> Vec<Tag> {
        self.into_vec()
    }
}

impl IntoIterator for TagList {
    type Item = Tag;
    type IntoIter = IntoIter;

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

macro_rules! from_and_try_into_impl {
    ($kind:ident, $type:ty, |$name:ident| $expr:expr) => {
        impl TryInto<Vec<$type>> for TagList {
            type Error = Tag;

            fn try_into(self) -> Result<Vec<$type>, Self::Error> {
                self.try_into_vec()
            }
        }

        impl From<Vec<$type>> for TagList {
            fn from(value: Vec<$type>) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = value;
                    TagList::$kind($expr)
                }
            }
        }

        impl<const N: usize> From<[$type; N]> for TagList {
            fn from(value: [$type; N]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind(Vec::from($expr))
                }
            }
        }

        impl<const N: usize> From<&[$type; N]> for TagList {
            fn from(value: &[$type; N]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind(Vec::from($expr))
                }
            }
        }

        impl From<&[$type]> for TagList {
            fn from(value: &[$type]) -> Self {
                if value.is_empty() {
                    TagList::Empty
                } else {
                    let $name = Vec::from(value);
                    TagList::$kind(Vec::from($expr))
                }
            }
        }

        impl FromIterator<$type> for TagList {
            fn from_iter<T: IntoIterator<Item = $type>>(iter: T) -> Self {
                Self::from(Vec::from_iter(iter))
            }
        }
    };
}

from_and_try_into_impl!(Bytes, i8, |value| value);
from_and_try_into_impl!(Shorts, i16, |value| value);
from_and_try_into_impl!(Ints, i32, |value| value);
from_and_try_into_impl!(Longs, i64, |value| value);
from_and_try_into_impl!(Bytes, u8, |value| reinterpret_vec!(value, u8 => i8));
from_and_try_into_impl!(Shorts, u16, |value| reinterpret_vec!(value, u16 => i16));
from_and_try_into_impl!(Ints, u32, |value| reinterpret_vec!(value, u32 => i32));
from_and_try_into_impl!(Longs, u64, |value| reinterpret_vec!(value, u64 => i64));
from_and_try_into_impl!(Floats, f32, |value| value);
from_and_try_into_impl!(Doubles, f64, |value| value);
from_and_try_into_impl!(Strings, String, |value| value);
from_and_try_into_impl!(ByteArrays, Vec<i8>, |value| value);
from_and_try_into_impl!(IntArrays, Vec<i32>, |value| value);
from_and_try_into_impl!(LongArrays, Vec<i64>, |value| value);
from_and_try_into_impl!(ByteArrays, Vec<u8>, |value| reinterpret_vec!(value, Vec<u8> => Vec<i8>));
from_and_try_into_impl!(
    IntArrays,
    Vec<u32>,
    |value| reinterpret_vec!(value, Vec<u32> => Vec<i32>)
);
from_and_try_into_impl!(
    LongArrays,
    Vec<u64>,
    |value| reinterpret_vec!(value, Vec<u64> => Vec<i64>)
);
from_and_try_into_impl!(Lists, TagList, |value| value);
from_and_try_into_impl!(Compounds, TagMap, |value| value);

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

pub struct IterValue<'a, T> {
    internal: Option<std::slice::Iter<'a, T>>,
}

impl<'a, T> Iterator for IterValue<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            None => None,
            Some(iter) => iter.next(),
        }
    }
}

pub struct IterValueMut<'a, T> {
    internal: Option<std::slice::IterMut<'a, T>>,
}

impl<'a, T> Iterator for IterValueMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            None => None,
            Some(iter) => iter.next(),
        }
    }
}

pub struct IntoIterValue<T> {
    internal: Option<std::vec::IntoIter<T>>,
}

impl<T> Iterator for IntoIterValue<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.internal {
            None => None,
            Some(iter) => iter.next(),
        }
    }
}

impl Debug for TagList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),

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

#[macro_export]
macro_rules! tag_list {
    [] => {
        {
            use $crate::tag::TagList;
            TagList::new()
        }
    };

    [ $($expr:expr),+ $(,)? ] => {
        {
            use $crate::tag::TagList;
            TagList::from_iter(vec![$($expr),+])
        }
    }
}

pub use tag_list;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_test() {
        let mut list = tag_list![1, 2, 3];
        list.push(3);

        assert_eq!(vec![1, 2, 3, 3], list.try_into_vec().unwrap())
    }
}
