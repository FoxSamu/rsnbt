use std::fmt::Debug;

use super::*;

/// An NBT tag. The NBT (Named Binary Tag) format structures data similarly to JSON, but is inherently meant
/// for binary serialization rather than something human-readable. However, NBT comes with a
/// human-readable form, SNBT (String Named Binary Tag), which is much like JSON and can be parsed or serialized
/// using the `snbt` module.
/// 
/// # NBT basics
///
/// A tag is one of 12 types:
/// - [`Tag::Byte`]: An 8-bit integer, often interpreted as signed or as a boolean.
/// - [`Tag::Short`]: A 16-bit integer, often interpreted as signed.
/// - [`Tag::Int`]: A 32-bit integer, often interpreted as signed.
/// - [`Tag::Long`]: A 64-bit integer, often interpreted as signed.
/// - [`Tag::Float`]: A 32-bit floating point value.
/// - [`Tag::Double`]: A 64-bit floating point value.
/// - [`Tag::String`]: A UTF-8 string.
/// - [`Tag::ByteArray`]: An array of 8-bit integers, often interpreted as some raw binary data.
/// - [`Tag::IntArray`]: An array of 32-bit integers, often interpreted as coordinates or UUIDs.
/// - [`Tag::LongArray`]: An array of 64-bit integers, often interpreted as tightly packed binary data.
/// - [`Tag::List`]: A list tags payloads of the same type.
/// - [`Tag::Compound`]: A collection of named tags, almost always interpreted as a mapping from names to tags.
/// 
/// Each tag has a corresponding binary representation, consisting of 3 main parts:
/// - an 8-bit tag ID;
/// - a UTF-8 encoded name of at most 65535 bytes, preceded by two bytes that encode the length;
/// - a payload of a type-dependent length.
///
/// These data are tightly packed to form the full tag encoding:
/// ```text
/// Byte: | 0  | 1 | 2 | 3  ..  2+L | 3+L .... |
/// Data: | ID | L     | Name bytes | Payload  |
/// ```
/// 
/// The tag ID is a unique ID identifying the type of the tag's payload (byte, string, compound, etc.).
/// See the documentation of [`TagId`] for more information. The tag ID is never `0`, this ID is reserved
/// to mark the end of a [compound tag][Tag::Compound].
///
/// The tag name starts with a 16-bit unsigned integer (2 bytes), followed by that
/// amount of UTF-8 bytes. It must be valid UTF-8. It is the same format as the payload of
/// a [string tag][Tag::String].
/// 
/// The exact format of the payload depends on the tag type:
/// - [`Tag::Byte`]: a 1-byte integer;
/// - [`Tag::Short`]: a 2-byte integer;
/// - [`Tag::Int`]: a 4-byte integer;
/// - [`Tag::Long`]: an 8-byte integer;
/// - [`Tag::Float`]: a 4-byte IEEE 754 compatible float;
/// - [`Tag::Double`]: an 8-byte IEEE 754 compatible float;
/// - [`Tag::String`]: a 2-byte unsigned length followed by that amount of UTF-8 bytes (same as the tag name);
/// - [`Tag::ByteArray`]: a 4-byte _signed_ length followed by that amount of byte values;
/// - [`Tag::IntArray`]: a 4-byte _signed_ length followed by that amount of int values;
/// - [`Tag::LongArray`]: a 4-byte _signed_ length followed by that amount of long values;
/// - [`Tag::List`]: a tag ID, then a 4-byte _signed_ length, then that amount of (unnamed and untyped) payloads
///   of the respective type;
/// - [`Tag::Compound`]: an arbitrary amount of fully encoded tags (type + name + payload), followed by an
///   end marker (`0`, the value that tag ID's cannot be).
/// 
/// See the documentation of the respective tag types for more information about their encoding.
///
/// NBT was designed by Notch for Minecraft, and remains the primary storage format used Minecraft:
/// Java Edition. Minecraft: Bedrock Edition also uses NBT, but far more conservatively. For more
/// information about NBT, its history, and its use throughout Minecraft, see
/// [the Minecraft wiki](https://minecraft.wiki/w/NBT_format).
/// 
/// # A note about endian-ness
/// NBT was originally designed for Minecraft: Java Edition, and traditionally all numeric values, whether those
/// are numeric payloads, lengths of names/strings or lengths of arrays/lists, are all encoded in **big-endian**.
/// That is, the first byte of any integer or float value is the 8 most significant bits of the value (in the
/// case of a float, that is the byte that contains its sign bit). Thus, the int value `0xC0A0B` is stored as
/// four bytes in the order `00 0C 0A 0B`.
/// 
/// However, Minecraft likes to be inconsistent with its formatting of NBT, so in Minecraft: Bedrock Edition,
/// every numeric value is instead **little-endian**. This means that the first byte of a numeric value is
/// the 8 least significant bits of the number. Thus, the same value `0xC0A0B` is instead stored as
/// `0B 0A 0C 00`.
/// 
/// The endian-ness is always consistent throughout an NBT blob, primarily because Java Edition always uses
/// big-endian encoding and Bedrock Edition always uses little-endian encoding. Since Bedrock Edition does not
/// use NBT as extensively as Java Edition, it is recommended to store NBT in big-endian format unless it is
/// specifically meant to be decoded by Bedrock Edition.
/// 
/// # Implementation Details
/// While the tag's full encoding inherently includes space for a name, the [`Tag`] enum does not include
/// names. This is by design. Instead if shipping names with [`Tag`]s, [`Tag`]s are associated to names in
/// a [`TagMap`], the construct used to represent a [compound tag][Tag::Compound]. The reason for this is that,
/// other than at the root of an NBT data blob, named tags only appear in compound tags. Since this one root
/// name is easily tracked in a separate variable, and [`TagMap`] handles the naming of tags in a compound tag,
/// the [`Tag`] enum does not include names.
///
/// Some other notes:
/// - While primitive tags could easily implement [Copy], because a tag can be more than a primitive, [Copy]
///   is not implemented.
/// - Due to tags being able to contain floats, [Tag] cannot implement [Eq].
#[derive(Clone, PartialEq, Debug)]
pub enum Tag {
    /// A tag storing an 8-bit wide integer, signed or unsigned.
    /// Byte tags are often used to represent booleans.
    /// 
    /// The value used to represent byte tags is [`i8`].
    /// Its numerical ID is `1` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1      | 2
    /// Data: | Value  |
    /// ```
    Byte(i8),

    /// A tag storing a 16-bit wide integer, signed or unsigned.
    /// 
    /// The value used to represent short tags is [`i16`].
    /// Its numerical ID is `2` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1   | 2   | 3
    /// Data: | Value     |
    /// ```
    /// 
    /// The endianness can vary. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition
    /// writes numbers in little-endian. This can be configured
    /// when loading or saving NBT tags through the `io` module.
    Short(i16),

    /// A tag storing a 32-bit wide integer, signed or unsigned.
    /// 
    /// The value used to represent int tags is [`i32`].
    /// Its numerical ID is `3` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1   | 2   | 3   | 4   | 5
    /// Data: | Value                 |
    /// ```
    /// 
    /// The endianness can vary. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition
    /// writes numbers in little-endian. This can be configured
    /// when loading or saving NBT tags through the `io` module.
    Int(i32),

    /// A tag storing a 64-bit wide integer, signed or unsigned.
    /// 
    /// The value used to represent long tags is [`i64`].
    /// Its numerical ID is `4` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
    /// Data: | Value                         |
    /// ```
    /// 
    /// The endianness can vary. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition
    /// writes numbers in little-endian. This can be configured
    /// when loading or saving NBT tags through the `io` module.
    Long(i64),

    /// A tag storing a 32-bit wide floating point value, as per
    /// the IEEE 754 standard for floating point values.
    /// 
    /// The value used to represent float tags is [`f32`].
    /// Its numerical ID is `5` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1   | 2   | 3   | 4   | 5
    /// Data: | Value                 |
    /// ```
    /// 
    /// The endianness can vary. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition
    /// writes numbers in little-endian. This can be configured
    /// when loading or saving NBT tags through the `io` module.
    Float(f32),

    /// A tag storing a 64-bit wide floating point value, as per
    /// the IEEE 754 standard for floating point values.
    /// 
    /// The value used to represent double tags is [`f64`].
    /// Its numerical ID is `6` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
    /// Data: | Value                         |
    /// ```
    /// 
    /// The endianness can vary. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition
    /// writes numbers in little-endian. This can be configured
    /// when loading or saving NBT tags through the `io` module.
    Double(f64),

    /// A tag storing an UTF-8 string. In binary, strings consist
    /// of a 16-bit unsigned length followed by that amount of
    /// bytes of UTF-8 data. The string cannot be more than
    /// 65535 bytes long and must be valid UTF-8.
    /// 
    /// The value used to represent string tags is [`String`].
    /// Its numerical ID is `8` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1   | 2   | 3      | 4      | .. | N+1      | N+2
    /// Data: | N         | Byte 0 | Byte 1 | .. | Byte N-1 |
    /// ```
    /// 
    /// The endianness of the length can vary. Minecraft: Java
    /// Edition writes numbers in big-endian while Minecraft:
    /// Bedrock Edition writes numbers in little-endian. This
    /// can be configured when loading or saving NBT tags through
    /// the `io` module.
    String(String),

    /// A tag storing an array of byte payloads. In binary, byte
    /// arrays consist of a 32-bit **signed** length followed by
    /// that amount of byte payloads. Fundamentally, that length
    /// cannot be negative.
    /// 
    /// The value used to represent byte array tags is a [`Vec`]
    /// of [`i8`]s.
    /// Its numerical ID is `7` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1 | 2 | 3 | 4 | 5      | 6      | .. | N+3      | N+4
    /// Data: | N             | Byte 0 | Byte 1 | .. | Byte N-1 |
    /// ```
    /// 
    /// The endianness of the length can vary. Minecraft: Java
    /// Edition writes numbers in big-endian while Minecraft:
    /// Bedrock Edition writes numbers in little-endian. This
    /// can be configured when loading or saving NBT tags through
    /// the `io` module.
    ByteArray(Vec<i8>),

    /// A tag storing an array of int payloads. In binary, int
    /// arrays consist of a 32-bit **signed** length followed by
    /// that amount of int payloads. Fundamentally, that length
    /// cannot be negative.
    /// 
    /// The value used to represent int array tags is a [`Vec`]
    /// of [`i32`]s.
    /// Its numerical ID is `11` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1 | 2 | 3 | 4 | 5 .. 8 | 9 .. 12 | .. | 4N .. 4N+3 | 4N+4
    /// Data: | N             | Int 0  | Int 1   | .. | Byte N-1   |
    /// ```
    /// 
    /// The endianness of the length and elements can vary but
    /// is the same across an NBT blob. Minecraft: Java Edition
    /// writes numbers in big-endian while Minecraft: Bedrock
    /// Edition writes numbers in little-endian. This can be
    /// configured when loading or saving NBT tags through
    /// the `io` module.
    IntArray(Vec<i32>),

    /// A tag storing an array of long payloads. In binary, long
    /// arrays consist of a 32-bit **signed** length followed by
    /// that amount of long payloads. Fundamentally, that length
    /// cannot be negative.
    /// 
    /// The value used to represent long array tags is a [`Vec`]
    /// of [`i64`]s.
    /// Its numerical ID is `12` (see [`TagId`]).
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 1 | 2 | 3 | 4 | 5 .. 11 | 12 .. 19 | .. | 8N-4 .. 8N+3 | 8N+4
    /// Data: | N             | Long 0  | Long 1   | .. | Byte N-1     |
    /// ```
    /// 
    /// The endianness of the length and elements can vary but
    /// is the same across an NBT blob. Minecraft: Java Edition
    /// writes numbers in big-endian while Minecraft: Bedrock
    /// Edition writes numbers in little-endian. This can be
    /// configured when loading or saving NBT tags through
    /// the `io` module.
    LongArray(Vec<i64>),

    /// A tag storing an array of payloads of a specific type ID
    /// specified in the payload. In binary, lists consist of an
    /// 8-bit element tag ID and 32-bit **signed** length followed
    /// by that amount of long payloads of the according type.
    /// Fundamentally, that length cannot be negative and the element
    /// ID must be a valid tag ID. The element ID may be `0` if the
    /// list is empty.
    /// 
    /// The value used to represent list tags is a [`TagList`].
    /// Its numerical ID is `9` (see [`TagId`]).
    /// 
    /// List tags may be nested with other list tags or compound tags.
    /// Note that the lists in a list of lists may each have a different
    /// type of elements.
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 0  | 1 | 2 | 3 | 4 | 5  ..  X0-1 | X0 .. X1-1 | .. | X(N-1) .. XN-1 | XN
    /// Data: | ID | N             | Payload 0   | Payload 1  | .. | Payload N-1    |
    /// ```
    /// 
    /// The endianness of the length and elements can vary but
    /// is the same across an NBT blob. Minecraft: Java Edition
    /// writes numbers in big-endian while Minecraft: Bedrock
    /// Edition writes numbers in little-endian. This can be
    /// configured when loading or saving NBT tags through
    /// the `io` module.
    List(TagList),

    /// A tag storing a table of fully named tag values. In binary,
    /// compound tags consist simply of all the tag values packed
    /// together, followed by an end marker (tag ID `0`). Tags must
    /// have unique names so they can be identified by name after
    /// decoding.
    /// 
    /// The value used to represent compound tags is a [`TagMap`].
    /// Its numerical ID is `10` (see [`TagId`]).
    /// 
    /// Compound tags may be nested with other compound tags or list tags.
    /// 
    /// # Payload Layout
    /// ```text
    /// Byte: | 0   | 1 | 2 | 3 .. 2+L0 | 3+L0 .. X0-1 | X0           ..          X1-1 | .. | XN  | XN + 1
    /// Data: | ID0 | L0    | Name 0    | Payload 0    | ID1 + L1 + Name 1 + Payload 1 | .. | END |
    /// ```
    /// 
    /// The endianness of elements can vary per NBT blob but is the
    /// same across an NBT blob. Minecraft: Java Edition writes
    /// numbers in big-endian while Minecraft: Bedrock Edition writes
    /// numbers in little-endian. This can be configured when loading
    /// or saving NBT tags through the `io` module.
    Compound(TagMap),
}

impl Tag {
    /// Returns the [`TagId`] of a [`Tag`].
    ///
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagId};
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
    /// See [`TagId::byte`].
    ///
    /// # Examples
    /// ```
    /// use rsnbt::tag::{Tag, TagId};
    ///
    /// let string = Tag::String(String::from("Hello world!"));
    /// assert_eq!(8, string.id_byte());
    /// 
    /// // id_byte() is equivalent to id().byte()
    /// assert_eq!(string.id().byte(), string.id_byte());
    /// ```
    pub fn id_byte(&self) -> u8 {
        self.id().byte()
    }
}

/// A [`Default`] implementation for [`Tag`].
/// The default value of [`Tag`] is a zero [byte tag][Tag::Byte].
/// 
/// # Examples
/// ```
/// use rsnbt::tag::Tag;
/// 
/// assert_eq!(Tag::Byte(0), Tag::default());
/// ```
impl Default for Tag {
    fn default() -> Self {
        Self::Byte(0)
    }
}


