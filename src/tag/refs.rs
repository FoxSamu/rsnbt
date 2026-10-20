use super::*;

/// An immutable reference to an NBT payload. [`TagRef`] is similar
/// to an immutable reference to [`Tag`] (i.e. `&Tag`), but instead
/// of having the type information and payload both behind a reference,
/// an owner of a [`TagRef`] owns the tag type but borrows its payload.
/// 
/// [`TagRef`]s are especially useful when working with [`TagList`]s,
/// as [`TagList`]s store payloads rather than full tags. It is thus
/// not possible to refer to an element of a [`TagList`] using `&Tag`,
/// but it is possible using [`TagRef`].
/// 
/// For a mutable alternative, see [`TagRefMut`].
/// 
/// # `&Tag` vs `TagRef`
/// The difference between `&Tag` and `TagRef` (or, more specifically,
/// `TagRef<'_>`) is best demonstrated graphically:
/// 
/// ```text
///                             Tag
/// +--------+                  +------------------------+
/// | &Tag  ------------------> | ByteArray              |
/// +--------+                  +------------------------+
///                             | data: Vec<i8>          |
///                             |       +----------------+
///                             |       | len: usize     |
///                             |       +----------------+
///                             |       | cap: usize     |
///                             |       +----------------+
///                             |       | ptr: *mut i8  ------> HEAP
///                             +-------+----------------+
/// 
/// TagRef
/// +-----------------+
/// | ByteArray       |         Vec<i8>
/// +-----------------+         +----------------+
/// | ref: &Vec<i8>  ---------> | len: usize     |
/// +-----------------+         +----------------+
///                             | cap: usize     |
///                             +----------------+
///                             | ptr: *mut i8  --------------> HEAP
///                             +----------------+
/// ```
/// 
/// What this demonstrates is that [`TagRef`] can borrow arbitrary tag
/// payloads, without having to wrap those payloads into a [`Tag`].
///
/// It is possible to convert from `&Tag` to `TagRef` through [`Into`].
/// This essentially copies the tag's type to the local scope, while
/// borrowing its contents. This creates the following layout:
/// 
/// ```text
///                             Tag
///                             +------------------------+
/// TagRef                      | ByteArray              |
/// +-----------------+         +------------------------+
/// | ByteArray       |         | data: Vec<i8>          |
/// +-----------------+         |       +----------------+
/// | ref: &Vec<i8>  -----------------> | len: usize     |
/// +-----------------+         |       +----------------+
///                             |       | cap: usize     |
///                             |       +----------------+
///                             |       | ptr: *mut i8  ------> HEAP
///                             +-------+----------------+
/// ```
///
/// For this reason, passing [`TagRef`]s can be preferrable over passing
/// `&Tag`s.
/// 
/// # Examples
/// Borrowing a [`Tag`]:
/// ```
/// use rsnbt::tag::{Tag, TagRef};
/// 
/// let tag = Tag::Float(3.14);
/// assert_eq!(TagRef::Float(&3.14), tag.borrow());
/// 
/// // This is the same:
/// assert_eq!(TagRef::Float(&3.14), (&tag).into());
/// ```
/// 
/// <br/>
/// 
/// Borrowing an element from a [`TagList`]:
/// ```
/// use rsnbt::tag::{Tag, TagRef, tag_list};
/// 
/// let list = tag_list![1i32, 2, 3, 4];
/// assert_eq!(TagRef::Int(&3), list.get(2));
/// ```
/// 
/// <br/>
/// 
/// Accepting either `&Tag` or `TagRef` as method parameter:
/// ```
/// use rsnbt::tag::{Tag, TagRef};
/// 
/// fn check_is_3<'a, T : Into<TagRef<'a>>>(tag: T) {
///     assert_eq!(TagRef::Int(&3), tag.into());
/// }
/// 
/// let tag = Tag::Int(3);
/// check_is_3(tag.borrow());
/// check_is_3(&tag);
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TagRef<'a> {
    /// An immutably borrowed [byte][Tag::Byte] payload, i.e. `&i8`.
    Byte(&'a i8),

    /// An immutably borrowed [short][Tag::Short] payload, i.e. `&i16`.
    Short(&'a i16),

    /// An immutably borrowed [int][Tag::Int] payload, i.e. `&i32`.
    Int(&'a i32),

    /// An immutably borrowed [long][Tag::Long] payload, i.e. `&i64`.
    Long(&'a i64),

    /// An immutably borrowed [float][Tag::Float] payload, i.e. `&f32`.
    Float(&'a f32),

    /// An immutably borrowed [double][Tag::Double] payload, i.e. `&f64`.
    Double(&'a f64),

    /// An immutably borrowed [string][Tag::String] payload, i.e. `&String`.
    String(&'a String),

    /// An immutably borrowed [byte array][Tag::ByteArray] payload, i.e. `&Vec<i8>`.
    ByteArray(&'a Vec<i8>),

    /// An immutably borrowed [int array][Tag::IntArray] payload, i.e. `&Vec<i32>`.
    IntArray(&'a Vec<i32>),

    /// An immutably borrowed [long array][Tag::LongArray] payload, i.e. `&Vec<i64>`.
    LongArray(&'a Vec<i64>),

    /// An immutably borrowed [list][Tag::List] payload, i.e. `&TagList`.
    List(&'a TagList),

    /// An immutably borrowed [compound][Tag::Compound] payload, i.e. `&TagMap`.
    Compound(&'a TagMap),
}

/// A mutable reference to an NBT payload. [`TagRefMut`] is similar
/// to a mutable reference to [`Tag`] (i.e. `&mut Tag`), but instead
/// of having the type information and payload both behind a reference,
/// an owner of a [`TagRefMut`] owns the tag type but borrows its payload.
/// 
/// [`TagRefMut`]s are especially useful when working with [`TagList`]s,
/// as [`TagList`]s store payloads rather than full tags. It is thus
/// not possible to refer to an element of a [`TagList`] using `&mut Tag`,
/// but it is possible using [`TagRefMut`].
/// 
/// For an immutable alternative, see [`TagRef`].
/// 
/// # `&mut Tag` vs `TagRefMut`
/// The difference between `&mut Tag` and `TagRefMut` (or, more specifically,
/// `TagRefMut<'_>`) is best demonstrated graphically:
/// 
/// ```text
///                             Tag
/// +------------+              +------------------------+
/// | &mut Tag  --------------> | ByteArray              |
/// +------------+              +------------------------+
///                             | data: Vec<i8>          |
///                             |       +----------------+
///                             |       | len: usize     |
///                             |       +----------------+
///                             |       | cap: usize     |
///                             |       +----------------+
///                             |       | ptr: *mut i8  ------> HEAP
///                             +-------+----------------+
/// 
/// TagRefMut
/// +---------------------+
/// | ByteArray           |     Vec<i8>
/// +---------------------+     +----------------+
/// | ref: &mut Vec<i8>  -----> | len: usize     |
/// +---------------------+     +----------------+
///                             | cap: usize     |
///                             +----------------+
///                             | ptr: *mut i8  --------------> HEAP
///                             +----------------+
/// ```
/// 
/// What this demonstrates is that [`TagRefMut`] can borrow arbitrary tag
/// payloads, without having to wrap those payloads into a [`Tag`]. It has
/// the additional restriction that the borrowed payload can't be changed
/// into a payload of another type.
///
/// It is possible to convert from `&mut Tag` to `TagRefMut` through [`Into`].
/// This essentially copies the tag's type to the local scope, while
/// borrowing its contents. This creates the following layout:
/// 
/// ```text
///                             Tag
///                             +------------------------+
/// TagRefMut                   | ByteArray              |
/// +---------------------+     +------------------------+
/// | ByteArray           |     | data: Vec<i8>          |
/// +---------------------+     |       +----------------+
/// | ref: &mut Vec<i8>  -------------> | len: usize     |
/// +---------------------+     |       +----------------+
///                             |       | cap: usize     |
///                             |       +----------------+
///                             |       | ptr: *mut i8  ------> HEAP
///                             +-------+----------------+
/// ```
///
/// For this reason, passing [`TagRefMut`]s can be preferrable over passing
/// `&mut Tag`s.
/// 
/// # Examples
/// Borrowing and modifying a [`Tag`]:
/// ```
/// use rsnbt::tag::{Tag, TagRefMut};
/// 
/// let mut tag = Tag::Float(3.14);
/// 
/// // Destructure the reference
/// let TagRefMut::Float(payload_ref) = tag.borrow_mut() else {
///     panic!();
/// };
/// 
/// *payload_ref = 2.71;
/// assert_eq!(Tag::Float(2.71), tag);
/// ```
/// 
/// <br/>
/// 
/// Borrowing and modifying an element from a [`TagList`]:
/// ```
/// use rsnbt::tag::{Tag, TagRefMut, tag_list};
/// 
/// let mut list = tag_list![1i32, 2, 3, 4];
/// 
/// // Destructure an element
/// let TagRefMut::Int(payload_ref) = list.get_mut(2) else {
///     panic!();
/// };
/// 
/// *payload_ref = 42;
/// assert_eq!(tag_list![1, 2, 42, 4], list);
/// ```
/// 
/// <br/>
/// 
/// Accepting either `&mut Tag` or `TagRefMut` as method parameter:
/// ```
/// use rsnbt::tag::{Tag, TagRefMut};
/// 
/// fn add_3<'a, T : Into<TagRefMut<'a>>>(tag: T) {
///     if let TagRefMut::Int(v) = tag.into() {
///         *v += 3;
///     }
/// }
/// 
/// let mut tag = Tag::Int(3);
/// add_3(tag.borrow_mut());
/// add_3(&mut tag);
/// 
/// ```
#[derive(PartialEq, Debug)]
pub enum TagRefMut<'a> {
    /// A mutable borrowed [byte][Tag::Byte] payload, i.e. `&mut i8`.
    Byte(&'a mut i8),
    
    /// A mutably borrowed [short][Tag::Short] payload, i.e. `&mut i16`.
    Short(&'a mut i16),
    
    /// A mutably borrowed [int][Tag::Int] payload, i.e. `&mut i32`.
    Int(&'a mut i32),
    
    /// A mutably borrowed [long][Tag::Long] payload, i.e. `&mut i64`.
    Long(&'a mut i64),
    
    /// A mutably borrowed [float][Tag::Float] payload, i.e. `&mut f32`.
    Float(&'a mut f32),
    
    /// A mutably borrowed [double][Tag::Double] payload, i.e. `&mut f64`.
    Double(&'a mut f64),
    
    /// A mutably borrowed [string][Tag::String] payload, i.e. `&mut String`.
    String(&'a mut String),
    
    /// A mutably borrowed [byte array][Tag::ByteArray] payload, i.e. `&mut Vec<i8>`.
    ByteArray(&'a mut Vec<i8>),
    
    /// A mutably borrowed [int array][Tag::IntArray] payload, i.e. `&mut Vec<i32>`.
    IntArray(&'a mut Vec<i32>),
    
    /// A mutably borrowed [long array][Tag::LongArray] payload, i.e. `&mut Vec<i64>`.
    LongArray(&'a mut Vec<i64>),
    
    /// A mutably borrowed [list][Tag::List] payload, i.e. `&mut TagList`.
    List(&'a mut TagList),
    
    /// A mutably borrowed [compound][Tag::Compound] payload, i.e. `&mut TagMap`.
    Compound(&'a mut TagMap),
}



impl Tag {
    /// Borrows a [`Tag`] as a [`TagRef`]. The expression `tag.borrow()`
    /// is essentially equivalent to `(&tag).into()`, except that [`borrow`]
    /// has explicit type information and often looks prettier.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagRef};
    /// 
    /// let tag = Tag::Float(3.14);
    /// assert_eq!(TagRef::Float(&3.14), tag.borrow());
    /// 
    /// // This is the same:
    /// assert_eq!(TagRef::Float(&3.14), (&tag).into());
    /// ```
    pub fn borrow(&self) -> TagRef<'_> {
        self.into()
    }

    /// Borrows a [`Tag`] as a [`TagRefMut`]. The expression `tag.borrow_mut()`
    /// is essentially equivalent to `(&mut tag).into()`, except that [`borrow_mut`]
    /// has explicit type information and often looks prettier.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagRefMut};
    /// 
    /// let mut tag = Tag::Float(3.14);
    /// 
    /// // Destructure the reference
    /// let TagRefMut::Float(payload_ref) = tag.borrow_mut() else {
    ///     panic!();
    /// };
    /// 
    /// *payload_ref = 2.71;
    /// assert_eq!(Tag::Float(2.71), tag);
    /// ```
    pub fn borrow_mut(&mut self) -> TagRefMut<'_> {
        self.into()
    }
}



macro_rules! tag_ref_impl {
($name:ident $($mut:ident)?) => {


impl<'a> $name<'a> {
    #[doc = concat!(
        "Returns the [`TagId`] of a [`", stringify!($name), "`].\n",
        "\n",
        "# Examples\n",
        "```\n",
        "use rsnbt::tag::{", stringify!($name), ", TagId};\n",
        "\n",
        "let ", $(stringify!($mut), " ",)? "payload = String::from(\"Hello world!\");\n",
        "let string = ", stringify!($name), "::String(&", $(stringify!($mut), " ",)? "payload);\n",
        "assert_eq!(TagId::String, string.id());\n",
        "```",
    )]
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

    #[doc = concat!(
        "Returns the 8-bit binary ID of this reference's type.\n",
        "This is analogous to calling `id().byte()`.\n",
        "See [`TagId::byte`].\n",
        "\n",
        "# Examples\n",
        "```\n",
        "use rsnbt::tag::{", stringify!($name), ", TagId};\n",
        "\n",
        "let ", $(stringify!($mut), " ",)? "payload = String::from(\"Hello world!\");\n",
        "let string = ", stringify!($name), "::String(&", $(stringify!($mut), " ",)? "payload);\n",
        "assert_eq!(8, string.id_byte());\n",
        "\n",
        "// id_byte() is equivalent to id().byte()\n",
        "assert_eq!(string.id().byte(), string.id_byte());\n",
        "```",
    )]
    pub fn id_byte(&self) -> u8 {
        self.id().byte()
    }

    #[doc = concat!(
        "Clones the payload borrowed by this [`TagRef`] and constructs\n",
        "a new [`Tag`] with that payload. The returned tag will have the\n",
        "same [`TagId`] as this reference.\n",
        "\n",
        "# Examples\n",
        "```\n",
        "use rsnbt::tag::{", stringify!($name), ", Tag};\n",
        "\n",
        "let ", $(stringify!($mut), " ",)? "payload = String::from(\"Hello world!\");\n",
        "let string_ref = ", stringify!($name), "::String(&", $(stringify!($mut), " ",)? "payload);\n",
        "let string_clone = string_ref.clone_tag();\n",
        "\n",
        "assert_eq!(Tag::String(payload.clone()), string_clone);\n",
        "```\n",
    )]
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

impl<'a> TagRef<'a> {
    /// Borrow the referenced payload into a [`TagRef`].
    /// This converts any `&TagRef` into an equivalent [`TagRef`],
    /// dereferencing the payload's type in the process.
    /// 
    /// Like [`Tag::borrow`], [`TagRef::borrow`] is implemented
    /// as [`Into`], so `tag_ref.borrow()` is similar to
    /// `(&tag_ref).into()` with the only difference being that
    /// `borrow` has a concrete type signature.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagRef;
    /// 
    /// let payload = 3;
    /// let tag_ref = TagRef::Int(&payload);
    /// let tag_ref_ref = &tag_ref;
    /// 
    /// assert_eq!(tag_ref_ref.borrow(), tag_ref);
    /// ```
    pub fn borrow(&'a self) -> TagRef<'a> {
        self.into()
    }
}

impl<'a> TagRefMut<'a> {
    /// Borrow the referenced payload into a [`TagRef`].
    /// This converts any `&TagRefMut` into an equivalent [`TagRef`],
    /// dereferencing the payload's type in the process.
    /// 
    /// Like [`Tag::borrow`], [`TagRefMut::borrow`] is implemented
    /// as [`Into`], so `tag_ref.borrow()` is similar to
    /// `(&tag_ref).into()` with the only difference being that
    /// `borrow` has a concrete type signature.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::{TagRefMut, TagRef};
    /// 
    /// let mut payload = 3;
    /// let tag_ref = TagRefMut::Int(&mut payload);
    /// let tag_ref_ref = &tag_ref;
    /// 
    /// assert_eq!(tag_ref_ref.borrow(), TagRef::Int(&3));
    /// ```
    pub fn borrow(&'a self) -> TagRef<'a> {
        self.into()
    }

    /// Borrow the referenced payload into a [`TagRefMut`].
    /// This converts any `&mut TagRefMut` into an equivalent [`TagRefMut`],
    /// dereferencing the payload's type in the process.
    /// 
    /// Like [`Tag::borrow_mut`], [`TagRefMut::borrow_mut`] is implemented
    /// as [`Into`], so `tag_ref.borrow_mut()` is similar to
    /// `(&mut tag_ref).into()` with the only difference being that
    /// `borrow_mut` has a concrete type signature.
    /// 
    /// # Examples
    /// ```
    /// use rsnbt::tag::TagRefMut;
    /// 
    /// let mut payload = 3;
    /// let mut tag_ref = TagRefMut::Int(&mut payload);
    /// let tag_ref_ref = &mut tag_ref;
    ///
    /// assert_eq!(TagRefMut::Int(&mut 3), tag_ref_ref.borrow_mut());
    /// ```
    pub fn borrow_mut(&'a mut self) -> TagRefMut<'a> {
        self.into()
    }
}

/// [`From`] implementation to convert from `&TagRef` to
/// `TagRef`. This conversion behaves exactly the same as
/// [`TagRef::borrow`].
impl<'a, 'b : 'a> From<&'a TagRef<'b>> for TagRef<'a> {
    fn from(val: &'a TagRef<'b>) -> Self {
        *val
    }
}

/// [`From`] implementation to convert from `&TagRefMut` to
/// `TagRef`. This conversion behaves exactly the same as
/// [`TagRefMut::borrow`].
impl<'a, 'b : 'a> From<&'a TagRefMut<'b>> for TagRef<'a> {
    fn from(val: &'a TagRefMut<'b>) -> Self {
        match val {
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

/// [`From`] implementation to convert from `&mut TagRefMut` to
/// `TagRefMut`. This conversion behaves exactly the same as
/// [`TagRefMut::borrow_mut`].
impl<'a, 'b : 'a> From<&'a mut TagRefMut<'b>> for TagRefMut<'a> {
    fn from(val: &'a mut TagRefMut<'b>) -> Self {
        match val {
            TagRefMut::Byte(v) => TagRefMut::Byte(v),
            TagRefMut::Short(v) => TagRefMut::Short(v),
            TagRefMut::Int(v) => TagRefMut::Int(v),
            TagRefMut::Long(v) => TagRefMut::Long(v),
            TagRefMut::Float(v) => TagRefMut::Float(v),
            TagRefMut::Double(v) => TagRefMut::Double(v),
            TagRefMut::String(v) => TagRefMut::String(v),
            TagRefMut::ByteArray(v) => TagRefMut::ByteArray(v),
            TagRefMut::IntArray(v) => TagRefMut::IntArray(v),
            TagRefMut::LongArray(v) => TagRefMut::LongArray(v),
            TagRefMut::List(v) => TagRefMut::List(v),
            TagRefMut::Compound(v) => TagRefMut::Compound(v),
        }
    }
}

/// [`From`] implementation to convert from [`TagRefMut`] to
/// [`TagRef`]. This consumes the mutable reference and returns
/// an immutable reference.
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

