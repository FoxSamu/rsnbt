use super::*;
use super::cast::*;

impl Tag {
    pub fn string<T>(v: T) -> Tag
    where
        T: Into<String>,
    {
        Tag::String(v.into())
    }

    pub fn into_string(self) -> Result<String, TagCastError<Self>> {
        if let Tag::String(str) = self {
            Ok(str)
        } else {
            Err(TagCastError::new(TagId::String, self.id(), self))
        }
    }

    pub fn as_str(&self) -> Result<&str, TagMismatchError> {
        if let Tag::String(str) = self {
            Ok(str.as_str())
        } else {
            Err(TagMismatchError::new(TagId::String, self.id()))
        }
    }

    pub fn as_mut_str(&mut self) -> Result<&mut str, TagMismatchError> {
        if let Tag::String(str) = self {
            Ok(str.as_mut_str())
        } else {
            Err(TagMismatchError::new(TagId::String, self.id()))
        }
    }
}

impl TagRef<'_> {
    pub fn as_str(&self) -> Result<&str, TagMismatchError> {
        if let TagRef::String(str) = self {
            Ok(str.as_str())
        } else {
            Err(TagMismatchError::new(TagId::String, self.id()))
        }
    }
}

impl TagRefMut<'_> {
    pub fn as_str(&self) -> Result<&str, TagMismatchError> {
        if let TagRefMut::String(str) = self {
            Ok(str.as_str())
        } else {
            Err(TagMismatchError::new(TagId::String, self.id()))
        }
    }

    pub fn as_mut_str(&mut self) -> Result<&mut str, TagMismatchError> {
        if let TagRefMut::String(str) = self {
            Ok(str.as_mut_str())
        } else {
            Err(TagMismatchError::new(TagId::String, self.id()))
        }
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
