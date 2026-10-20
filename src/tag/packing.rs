use std::error::Error;
use std::fmt::Display;

use crate::tag::Tag;
use crate::tag::TagId;
use crate::tag::TagMap;
use crate::tag::TagRef;

// TODO this is probably something to be expanded into its own module
// It is essentially a more streamlined version of casting



/// [`Unpack`] is a trait for types that can be unpacked from [`Tag`]s.
///
/// The logic of [`Unpack`] is similar to [`TryFrom`]: we want to unpack
/// the [`Tag`] into `Self`, but this might fail (possibly because the
/// tag structure was wrong in some way). If unpacking fails, the result
/// is an [`Err`] containing an [`PackingError`]. If it succeeds, the
/// result is an [`Ok`] containing the unpacked value.
/// 
/// [`Unpack`] differs from [`TryFrom`] in several ways:
/// - where [`TryFrom`] receives a moved value to convert from, [`Unpack`]
///   receives a reference (`&Tag`);
/// - [`Unpack`] is concrete and always converts from [`Tag`] to `Self`,
///   generating [`PackingError`]s upon failure;
/// - the reciprocal of [`Unpack`] is [`Pack`], and implementations of
///   [`Unpack`] and [`Pack`] are not linked in any way.
/// 
/// When implementing [`Unpack`], you likely also want to implement [`Pack`],
/// to indicate a two-way conversion possibility between your type and
/// [`Tag`].
/// 
/// [`Tag`] implements [`Unpack`], in which it simply clones itself,
/// succeeding infallibly.
/// 
/// # Implementing Unpack
/// ```no_run
/// use rsnbt::tag::{Unpack, PackingResult, TagMap, Tag, TagRef};
/// 
/// struct Foo {
///     x: i32,
///     y: u8
/// }
/// 
/// impl Unpack<'_> for Foo {
///     fn unpack_from(tag: TagRef) -> PackingResult<Self> {
///         let map = tag.unpack_compound()?;
///         
///         Ok(Self {
///             x: map.unpack("x"),
///             y: map.unpack("y"),
///         })
///     }
/// }
/// ```
pub trait Unpack<'a> : Sized {
    /// Unpack `Self` from a [`TagRef`]. This method implements the
    /// logic for a type that is produced by [`Tag::unpack`]. It is
    /// preferrable that you call [`Tag::unpack`] instead of this
    /// method.
    fn unpack_from(tag: TagRef<'a>) -> PackingResult<Self>;
}

/// [`Pack`] is a trait for types that can be packed into [`Tag`]s.
/// 
/// The logic of [`Pack`] is very similar to [`TryInto`]: we want to convert
/// `Self` into a [`Tag`], but this might fail (possibly because of conditions
/// under which some data should not be saved). If packing fails, the result
/// is an [`Err`] containing an [`PackingError`]. If it succeeds, the
/// result is an [`Ok`] containing the packed tag.
/// 
/// [`Pack`] differs from [`TryInto`] in several ways:
/// - where [`TryInto`] receives a moved value to convert from, [`Pack`]
///   receives a reference (`&Self`);
/// - [`Pack`] is concrete and always converts from `Self` to [`Tag`],
///   generating [`PackingError`]s upon failure;
/// - the reciprocal of [`Pack`] is [`Unpack`], and implementations of
///   [`Pack`] and [`Unpack`] are not linked in any way.
/// 
/// When implementing [`Pack`], you likely also want to implement [`Unpack`],
/// to indicate a two-way conversion possibility between your type and
/// [`Tag`].
/// 
/// [`Tag`] implements [`Pack`], in which it simply clones itself,
/// succeeding infallibly.
/// 
/// # Implementing Pack
/// ```no_run
/// use rsnbt::tag::{Pack, PackingResult, Tag, tag_map};
/// 
/// struct Foo {
///     x: i32,
///     y: u8
/// }
/// 
/// impl Pack for Foo {
///     fn pack(&self) -> PackingResult<Tag> {
///         Ok(tag_map! {
///             "x": self.x.pack()?,
///             "y": self.y.pack()?
///         })
///     }
/// }
/// ```
pub trait Pack {
    fn pack(&self) -> PackingResult<Tag>;
}



impl Tag {
    /// Unpack
    pub fn unpack<'a, T>(&'a self) -> PackingResult<T> where T : Unpack<'a> {
        T::unpack_from(self.into())
    }

    pub fn pack_from(value: &impl Pack) -> PackingResult<Self> {
        value.pack()
    }
}

impl<'a> TagRef<'a> {
    pub fn unpack<T>(self) -> PackingResult<T> where T : Unpack<'a> {
        T::unpack_from(self)
    }
}

impl Unpack<'_> for Tag {
    fn unpack_from(tag: TagRef) -> PackingResult<Self> {
        Ok(tag.clone_tag())
    }
}

impl Pack for Tag {
    fn pack(&self) -> PackingResult<Tag> {
        Ok(self.clone())
    }
}

impl<'a> Unpack<'a> for &'a TagMap {
    fn unpack_from(tag: TagRef<'a>) -> PackingResult<Self> {
        if let TagRef::Compound(v) = tag {
            Ok(v)
        } else {
            wrong_type(tag.id(), TagId::Compound)
        }
    }
}



fn wrong_type<T>(id: TagId, expected: TagId) -> PackingResult<T> {
    Err(PackingError::new(
        PackingErrorKind::WrongType,
        format!("cannot convert {} to {}", id, expected)
    ))
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackingErrorKind {
    WrongType,
    MissingKey,
    ArrayTooShort,
    ArrayTooLong,
    InvalidInput,
    InvalidData,
    Other
}

impl Display for PackingErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackingErrorKind::WrongType => write!(f, "wrong type")?,
            PackingErrorKind::MissingKey => write!(f, "missing key")?,
            PackingErrorKind::ArrayTooShort => write!(f, "too short array or list")?,
            PackingErrorKind::ArrayTooLong => write!(f, "too long array or list")?,
            PackingErrorKind::InvalidInput => write!(f, "invalid input")?,
            PackingErrorKind::InvalidData => write!(f, "invalid data")?,
            PackingErrorKind::Other => write!(f, "other")?,
        }

        Ok(())
    }
}


pub type PackingResult<T> = Result<T, PackingError>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackingError {
    kind: PackingErrorKind,
    message: String
}

impl PackingError {
    pub fn new<I>(kind: PackingErrorKind, message: I) -> Self where I : Into<String> {
        Self { kind, message: message.into() }  
    }

    pub fn kind(&self) -> PackingErrorKind {
        self.kind
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

impl Display for PackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "unpack error ({})", self.kind)
        } else {
            write!(f, "unpack error ({}): {}", self.kind, self.message)
        }
    }
}

impl From<PackingError> for std::io::Error {
    fn from(value: PackingError) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}", value)
        )
    }
}

impl Error for PackingError {
}
