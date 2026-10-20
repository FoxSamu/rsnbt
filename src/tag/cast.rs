use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

use super::*;

impl Tag {
    pub fn try_cast(self, id: TagId) -> Result<Tag, TagCastError<Tag>> {
        Ok(match id {
            TagId::Byte => Tag::Byte(match self {
                Tag::Byte(v) => v as i8,
                Tag::Short(v) => v as i8,
                Tag::Int(v) => v as i8,
                Tag::Long(v) => v as i8,
                Tag::Float(v) => v as i8,
                Tag::Double(v) => v as i8,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            TagId::Short => Tag::Short(match self {
                Tag::Byte(v) => v as i16,
                Tag::Short(v) => v as i16,
                Tag::Int(v) => v as i16,
                Tag::Long(v) => v as i16,
                Tag::Float(v) => v as i16,
                Tag::Double(v) => v as i16,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            TagId::Int => Tag::Int(match self {
                Tag::Byte(v) => v as i32,
                Tag::Short(v) => v as i32,
                Tag::Int(v) => v as i32,
                Tag::Long(v) => v as i32,
                Tag::Float(v) => v as i32,
                Tag::Double(v) => v as i32,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            TagId::Long => Tag::Long(match self {
                Tag::Byte(v) => v as i64,
                Tag::Short(v) => v as i64,
                Tag::Int(v) => v as i64,
                Tag::Long(v) => v as i64,
                Tag::Float(v) => v as i64,
                Tag::Double(v) => v as i64,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            TagId::Float => Tag::Float(match self {
                Tag::Byte(v) => v as f32,
                Tag::Short(v) => v as f32,
                Tag::Int(v) => v as f32,
                Tag::Long(v) => v as f32,
                Tag::Float(v) => v as f32,
                Tag::Double(v) => v as f32,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            TagId::Double => Tag::Double(match self {
                Tag::Byte(v) => v as f64,
                Tag::Short(v) => v as f64,
                Tag::Int(v) => v as f64,
                Tag::Long(v) => v as f64,
                Tag::Float(v) => v as f64,
                Tag::Double(v) => v as f64,
                tag => Err(TagCastError::new(id, tag.id(), tag))?,
            }),

            id => {
                if id != self.id() {
                    Err(TagCastError::new(id, self.id(), self))?
                } else {
                    self
                }
            },
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagMismatchError {
    expected: Vec<TagId>,
    actual: TagId
}

impl TagMismatchError {
    pub fn new(expected: TagId, actual: TagId) -> Self {
        Self { expected: vec![expected], actual }
    }

    pub fn new_multi(expected: impl Into<Vec<TagId>>, actual: TagId) -> Self {
        Self { expected: expected.into(), actual }
    }

    pub fn expected(&self) -> &Vec<TagId> {
        &self.expected
    }

    pub fn actual(&self) -> TagId {
        self.actual
    }
}

impl Display for TagMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.expected.is_empty() {
            // This is weird but why not
            return write!(f, "tag type mismatch: expected none, got {}", self.actual);
        }

        if self.expected.len() == 1 {
            let just = self.expected.first().unwrap();
            return write!(f, "tag type mismatch: expected {}, got {}", just, self.actual);
        }

        let mut formatted_str = String::new();
        let elems = self.expected.len();

        for i in 0..elems {
            if i == elems - 1 {
                formatted_str.push_str(", or ");
            } else if i > 0 {
                formatted_str.push_str(", ");
            }
            
            formatted_str.push_str(&format!("{}", self.expected[i]));
        }

        write!(f, "tag type mismatch: expected either {}, got {}", formatted_str, self.actual)
    }
}

impl Error for TagMismatchError {
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCastError<T> where T : Debug {
    cause: TagMismatchError,
    payload: T
}

impl<T> TagCastError<T> where T : Debug {
    pub fn new(expected: TagId, actual: TagId, payload: T) -> Self {
        Self { cause: TagMismatchError::new(expected, actual), payload }
    }

    pub fn new_multi(expected: impl Into<Vec<TagId>>, actual: TagId, payload: T) -> Self {
        Self { cause: TagMismatchError::new_multi(expected.into(), actual), payload }
    }

    pub fn couple(e: TagMismatchError, payload: T) -> Self {
        Self { cause: e, payload }
    }

    pub fn expected(&self) -> &Vec<TagId> {
        &self.cause.expected
    }

    pub fn actual(&self) -> TagId {
        self.cause.actual
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn decouple(self) -> (T, TagMismatchError) {
        (self.payload, self.cause)
    }
}

impl<T> Display for TagCastError<T> where T : Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", TagMismatchError::from(self))
    }
}

impl<T> Error for TagCastError<T> where T : Debug {
}

impl<T> From<TagCastError<T>> for TagMismatchError where T : Debug {
    fn from(value: TagCastError<T>) -> Self {
        value.cause
    }
}

impl<T> From<&TagCastError<T>> for TagMismatchError where T : Debug {
    fn from(value: &TagCastError<T>) -> Self {
        value.cause.clone()
    }
}


impl From<TagMismatchError> for std::io::Error {
    fn from(value: TagMismatchError) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            value
        )
    }
}

impl<T> From<TagCastError<T>> for std::io::Error where T : Debug {
    fn from(value: TagCastError<T>) -> Self {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            value.decouple().1
        )
    }
}
