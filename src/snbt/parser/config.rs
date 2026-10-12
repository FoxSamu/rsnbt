/// A configuration for SNBT parsing.
///
/// SNBT has evolved slightly over the years, and older SNBT is usually compatible with newer SNBT.
/// The parsing config mostly allows enabling or disabling specific features that were added over
/// time, though it also provides several useful options that are not natively supported by
/// Minecraft's SNBT parser.
///
/// Every field of this struct is public, it is therefore possible to produce an instance through
/// a struct literal:
/// ```no_run
/// let config = ParseConfig {
///     ...
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseConfig {
    /// Allows the numeric literals `nan` and `inf` to be used for NaN and infinity floats.
    /// They can be combined with signs and type suffixes, such as `-infd` or `+nanf`.
    /// No guarantees are given about the exact NaN value other than that it is a quiet NaN.
    ///
    /// `nan` and `inf` are not natively supported by Minecraft SNBT.
    pub nonfinite_floats: NonfiniteFloats,

    /// Allows the use of two special operations:
    /// - `bool(x)` returns a byte value that is `1` when `x` is nonzero and `0` otherwise. It is
    ///   an error when `x` is not a numeral.
    /// - `uuid(x)` returns an int array value with four integers representing the UUID string `x`.
    ///   It is an error when `x` is not a string.
    ///
    /// Note that nesting these operations is not supported.
    ///
    /// These operations are natively supported by Minecraft SNBT since 1.21.5 snapshot 25w10a.
    pub ops: Ops,

    /// Allows lists to be hetrogenous. Hetrogenous lists are converted to lists of compound tags,
    /// where every tag that is not a compound tag is wrapped in a compound tag under an empty key.
    /// For example, `[1, true, {"foo": 3}]` would be parsed as `[{"": 1}, {"": true}, {"foo": 3}]`.
    ///
    /// Heterogenous lists are natively supported by Minecraft SNBT since 1.21.5 snapshot 25w04a,
    /// though they only got their proper semantic meaning in snapshot 25w09a. Before that, they
    /// were not parsed as lists of compound tags, but as some internal wrapper object that was
    /// then deserialized immediately.
    pub hetrogenous_lists: HetrogenousLists,

    /// Allows line comments starting with `#` to be used. These comments are ignored by the parser.
    ///
    /// Comments are not natively supported by Minecraft SNBT, but some mods or alternate tools may
    /// support it. For example, FTB supports it in their extended SNBT format.
    pub comments: Comments,

    /// Allows a single trailing comma in lists, arrays and compound tags.
    ///
    /// Trailing commas are natively supported by Minecraft SNBT since 1.21.5 snapshot 25w09a.
    pub extra_commas: ExtraCommas,

    /// Allows line breaks to separate values in lists, arrays and compound tags, in addition to
    /// commas. It would allow a syntax like:
    /// ```snbt
    /// {
    ///     "foo": 3
    ///     "bar": 4
    /// }
    /// ```
    /// Note that if two values appear on the same line, they must still be separated by a comma.
    ///
    /// Line break separation is not natively supported by Minecraft SNBT, but some mods or
    /// alternate tools may support it. For example, FTB supports it in their extended SNBT format.
    pub allow_line_break_separation: bool,

    /// Allows single quotes to be used for strings and tag names. This allows `'foo'` in addition
    /// to `"foo"`.
    ///
    /// Single quoted strings are natively supported by Minecraft SNBT since 1.14 snapshot 19w08a.
    pub string_format: StringFormat,

    /// Allows boolean literals for byte tags. This allows the literals `true` and `false` to stand
    /// in for the values `1B` and `0B` respectively, though only as byte tags. In byte array tags,
    /// booleans are not accepted.
    ///
    /// Note that NBT itself does not support booleans. Instead, booleans are saved as byte tags,
    /// where 0 means false and anything else (but usually 1) means true.
    ///
    /// Boolean literals have always been natively supported by Minecraft SNBT.
    pub integer_format: IntegerFormat,

    /// Sets the level of lenience for tag names in compound tags. Tag names are always strings, so
    /// when parsing a tag name, the parser doesn't necessarily need to distinguish different types.
    /// It is therefore possible for the parser to interpret any literal simply as a string. This
    /// option controls that:
    /// - [`NameFormat::String`] requires names to strictly be quoted strings;
    /// - [`NameFormat::Identifier`] requires names to be quoted strings or valid non-reserved
    ///   identifiers;
    /// - [`NameFormat::NonNumeral`] allows names be quoted strings, any identifier, or a reserved
    ///   word (since reserved words have no meaning in keys);
    /// - [`NameFormat::AnyWord`] allows names to be quoted strings or any word with no restrictions.
    ///
    /// This option is independent of [`ParseConfig::allow_identifier_strings`], which specifies
    /// whether string values can be identifiers.
    ///
    /// The default in Minecraft SNBT is to accept any word, i.e. [`NameFormat::AnyWord`], which has
    /// always been the way Minecraft parses SNBT.
    pub name_format: NameFormat,

    /// Allows spaces around the type prefix of byte, int and long arrays, e.g. it allows things
    /// like `[ B ; ...]` instead of being forced to be tighly packed at the start of the array like
    /// `[B; ...]`.
    ///
    /// Spaces in type prefixes are not natively supported by Minecraft SNBT.
    pub array_format: ArrayFormat,

    /// Allows the usage of `=` in place of `:` to separate keys and values, e.g. `{foo = bar}`
    /// instead of `{foo: bar}`.
    ///
    /// This is not allowed in Minecraft SNBT but is allowed in FTB config SNBT.
    pub allow_equals_sign: bool,
}

/// Configures the allowed formats of integers in the SNBT format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerFormat {
    /// Allows unsigned integer values. Unsigned values are indicated with `ub`, `us`, `ui`, `ul`,
    /// and have different limits than signed values. In addition, this also allows `sb`, `ss`, `si`
    /// and `sl` to be used for signed integer values along with the usual `b`, `s`, `i` and `l`.
    /// As usual, all type specifiers are case insensitive, so `Ub`, `uB`, `UB` and `ub` all behave
    /// the same.
    pub allow_unsigned: bool,

    /// Allow boolean literals `true` and `false` to respectively stand in for exactly the values
    /// `1B` and `0B`.
    pub allow_booleans: bool,

    /// Allow hexadecimal and binary integer literals, starting respectively with `0x` and `0b`.
    /// These prefixes are case-insensitive, so `0X` and `0B` are equally valid.
    pub allow_hex_and_bin: bool,
}

/// Configures the formats of arrays in the SNBT format. This applies to array tags, not to lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayFormat {
    /// Allows spaced type prefixes, such as `[ B ;`. When disabled, these prefixes have to be
    /// tightly packed like `[B;`.
    ///
    /// This is allowed by Minecraft SNBT and FTB config SNBT.
    pub allow_spaced_prefix: bool,

    /// Allows type prefixes to be lowercase, allowing things like `[b;`, `[i;` and `[l;` in
    /// addition to `[B;`, `[I;` and `[L;`.
    ///
    /// This is not allowed in Minecraft SNBT, but is accepted by FTB config SNBT.
    pub allow_lowercase_prefix: bool,

    /// Allows boolean values in arrays, allowing things like `[B; true, false]` and, given that
    /// `allow_coercion` is set, `[L; true, false]`. The booleans can be mixed with numbers, e.g.
    /// `[B; 1, true]`.
    ///
    /// This option also allows any operators that convert to booleans, e.g. `bool`, to be used in
    /// arrays.
    ///
    /// This is not allowed in Minecraft SNBT or FTB config SNBT.
    pub allow_booleans: bool,

    /// Allows the use of any integer value in arrays, which will be coerced to the right integer
    /// type. This allows things like `[L; 1, 2s, 3b, 4l]`. Note that integers are only coerced to
    /// bigger integers, thus, bytes coerce to longs but longs don't coerce to bytes.
    ///
    /// This is allowed by Minecraft SNBT and FTB config SNBT.
    ///
    /// It is worth noting that unsuffixed integer literals in arrays are always treated as the type
    /// of the array they appear in.
    pub allow_coercion: bool,
}

/// Configures the operators available in the SNBT format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ops {
    /// Allows the `bool` operator, which converts numbers to booleans.
    pub bool: bool,

    /// Allows the `uuid` operator, which converts strings to UUID int arrays.
    pub uuid: bool,

    /// Allows the `base64` operator, which converts Base-64 strings to byte arrays.
    pub base64: bool,

    /// Allows the `pack` operator, packs integers into a long array with the smallest bit
    /// width possible. An optional argument allows setting the bit width.
    ///
    /// This packing format is used in Minecraft chunk data.
    pub pack: bool,
}

/// Configures how comments are handled in SNBT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comments {
    /// Allows line comments starting with `#`.
    pub hash: bool,

    /// Allows C-style comments, which include:
    /// - line comments starting with `//`;
    /// - block comments delimited with `/*` and `*/`.
    pub c_style: bool,

    /// Allows tilde-style comments, which include:
    /// - line comments starting with `~~` and optionally ending with `~~`;
    /// - block comments starting with `~~~` or longer and ending with the same amount of tildes.
    pub tildes: bool,
}

/// Configures how extra commas are accepted in compound and collection tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtraCommas {
    /// Strictly only allow commas as separation of two values.
    Never,

    /// Strictly only allow commas as separation of two values, with one additional comma being
    /// optional after the last value.
    SingleTrailing,

    /// Allow multiple commas between, before and after values.
    Anywhere,
}

/// Configures how to handle hetrogenous lists. These options are ignored if a list is homogenous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HetrogenousLists {
    /// Disallows hetrogenous lists.
    Disallowed,

    /// Wrap values in compound tags under an emtpy name, i.e. `1` to `{"": 1}`.
    WrapInCompound {
        /// Whether to wrap values that are already compound tags.
        wrap_compounds: bool,
    },

    /// Wrap values in singleton lists, i.e. `1` to `[1]`.
    WrapInList {
        /// Whether to wrap values that are already list tags.
        wrap_lists: bool,
    },
}

/// The possible values for [`ParseConfig::name_mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameFormat {
    /// Allows only the use of quoted strings as tag names in compound tags, and disallows any
    /// numeral, identifier or reserved name.
    ///
    /// Note that this disables identifiers in names even [`ParseConfig::allow_identifier_strings`]
    /// is set to `true`. However, name strings follow the same quoting requirements as value
    /// strings, as determined by [`ParseConfig::allow_single_quote_strings`].
    String,

    /// Allows the use of strings and identifiers as tag name in compound tags, but disallows
    /// numerals and reserved names.
    ///
    /// Note that, depending on some other parsing options, some names that are normally reserved
    /// might be accepted as identifiers instead. Thus, when this option is set to `false`, it may
    /// still accept some special words in the following scenarios:
    /// - with [`ParseConfig::allow_boolean_literals`] set to `false`, the words `true` and `false`
    ///   will be valid identifiers and therefore valid names;
    /// - with [`ParseConfig::allow_ops`] set to `false`, the words `bool` and `uuid`
    ///   will be valid identifiers and therefore valid names;
    /// - with [`ParseConfig::allow_nan_inf`] set to `false`, the words `nan`, `inf`, and the
    ///   variants with type specifiers (e.g. `nanF`, `infD`, etc.) will be valid identifiers and
    ///   therefore valid names.
    ///
    /// Note that this enables identifiers in names even [`ParseConfig::allow_identifier_strings`]
    /// is set to `false`.
    ///
    /// Note that words starting with `+`, `-`, `.` or any digit will be treated as numerals in any
    /// case and thus won't be valid.
    Identifier,

    /// Allows the use of strings and any word that is not a numeral as tag name in compound tags,
    /// but disallows numerals.
    ///
    /// Note that with [`ParseConfig::allow_nan_inf`] set to `false`, the words `nan`, `inf`,
    /// and the variants with type specifiers (e.g. `nanF`, `infD`, etc.) will be valid identifiers
    /// and therefore valid names. Words starting with `+`, `-`, `.` or any digit will be treated as
    /// numerals in any case and thus won't be valid.
    NonNumeral,

    /// Allows the use of any word as tag name in compound tags. This means that:
    /// - numerals are valid names;
    /// - booleans are valid names;
    /// - the operator terms `bool` and `uuid` are valid names.
    ///
    /// Note that numerals don't need to represent actually valid numerals. Numerals are not parsed
    /// until their numerical value is needed so numeral tokens that would fail to parse as number
    /// values, would still be valid as tag names. For example, `-foo` is a numeral token that
    /// would not parse as a number, but would be accepted as tag name.
    AnyWord,
}

/// The string formats allowed.
///
/// This determines the escape codes allowed in strings and some other formatting rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringFormat {
    /// Allows identifiers to be used as strings, given they are not reserved words.
    pub allow_identifiers: bool,

    /// Allows `"` as string delimiter.
    pub single_quotes: bool,

    /// Allows `'` as string delimiter.
    pub double_quotes: bool,

    /// Ignores invalid escapes and inserts them as-is into the string.
    /// This allows `\` to be directly inserted when it does not form a valid escape code.
    ///
    /// This option is irrelevant if `esc_chars` is `true`.
    pub ignnore_invalid_escapes: bool,

    /// Allows `\0` for NUL.
    pub esc_nul: bool,

    /// Allows `\'` for `'`.
    pub esc_single_quote: bool,

    /// Allows `\"` for `"`.
    pub esc_double_quote: bool,

    /// Allows `\/` for `/`.
    pub esc_slash: bool,

    /// Allows `\\` for `\`.
    pub esc_backslash: bool,

    /// Allows `\n` for line feed.
    pub esc_lf: bool,

    /// Allows `\r` for carriage return.
    pub esc_cr: bool,

    /// Allows `\f` for form feed.
    pub esc_ff: bool,

    /// Allows `\f` for form feed.
    pub esc_space: bool,

    /// Allows `\t` for tab.
    pub esc_tab: bool,

    /// Allows `\b` for backspace.
    pub esc_bs: bool,

    /// Allows `\xXX` for a character with hexadecimal unicode index `XX`.
    pub esc_unicode_8: bool,

    /// Allows `\uXXXX` for a character with hexadecimal unicode index `XXXX`.
    pub esc_unicode_16: bool,

    /// Allows `\UXXXXXXXX` for a character with hexadecimal unicode index `XXXXXXXX`.
    pub esc_unicode_32: bool,

    /// Allows `\N{<name>}` for a character with hexadecimal unicode name `<name>`.
    pub esc_unicode_name: bool,

    /// Allows `\C` for any character `C` that does not form any other valid escape code.
    /// E.g. this allows `\e` for `e`.
    ///
    /// When `true`, `ignnore_invalid_escapes` is meaningless since any escape is now valid.
    pub esc_chars: bool,

    /// Allows line breaks in strings. They will be interpreted as `\n`.
    pub allow_line_breaks: bool,

    /// Allows line breaks escaped with `\`. They will be ignored entirely.
    pub allow_escaped_line_breaks: bool,
}

/// Configures the way in which "infinity" and "not a number" (NaN) float values are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonfiniteFloats {
    /// Do not allow floats to be infinity or NaN. Minecraft SNBT does not allow floats to be
    /// infinity or NaN, they must strictly be finite. This also means that finite float literals
    /// that exceed the limits of the underlying type will be denied. This behavior can be disabled
    /// with the `allow_too_large_finite` option.
    Disabled {
        /// Allows finite floats to exceed their limits and be parsed as infinity.
        ///
        /// For example, `12e12345` exceeds both 32-bit and 64-bit float limits, and when parsed it
        /// will simply be interpreted as infinity. This is normally not allowed when infinity is
        /// disabled, but this option allows this.
        allow_too_large_finite: bool,
    },

    /// Allows infinity values, but not NaN values.
    InfinityOnly {
        /// Allows finite floats to exceed their limits and be parsed as infinity.
        ///
        /// For example, `12e12345` exceeds both 32-bit and 64-bit float limits, and when parsed it
        /// will simply be interpreted as infinity. This is normally not allowed when infinity is
        /// disabled, but this option allows this.
        allow_too_large_finite: bool,

        /// Allows JavaScript-like infinity literals, i.e. `Infinity`. These literals may be
        /// prefixed with a sign (`+`, `-`) and suffixed with a type specifier (`f`, `F`, `d`, `D`).
        /// For example, `-Infinityf`, `InfinityD`, etc.
        js_like: bool,

        /// Allows Rust-like infinity literals, i.e. `inf`. These literals may be prefixed
        /// with a sign (`+`, `-`) and suffixed with a type specifier (`f`, `F`, `d`, `D`). For
        /// example, `-inff`, `infD`, etc.
        ///
        /// Note that Rust does not necessarily allow `inf` as a constant in code, but the
        /// [`FromStr`] implementation on floats accepts `inf` as infinity.
        rust_like: bool,

        /// Allows the infinity loop symbol, i.e. `∞`. These literals may be prefixed
        /// with a sign (`+`, `-`) and suffixed with a type specifier (`f`, `F`, `d`, `D`). For
        /// example, `-∞f`, `∞D`, etc.
        ///
        /// This has the side effect that the infinity loop symbol will be accepted in identifiers.
        lemniscate: bool,
    },

    /// Allows infinity and NaN values.
    Enabled {
        /// Allows finite floats to exceed their limits and be parsed as infinity.
        ///
        /// For example, `12e12345` exceeds both 32-bit and 64-bit float limits, and when parsed it
        /// will simply be interpreted as infinity. This is normally not allowed when infinity is
        /// disabled, but this option allows this.
        allow_too_large_finite: bool,

        /// Allows NaN values to be signed. Prepending signs to NaN values is usually not relevant
        /// in any way, since -NaN and +NaN produce the same value. FeedTheBeast does not allow
        /// signs on NaN values.
        signed_nan: bool,

        /// Allows JavaScript-like infinity literals, i.e. `Infinity` and `NaN`. Infinity literals,
        /// and NaN literals depending on whether `signed_nan` is set, may be prefixed with a sign
        /// (`+`, `-`). Both infinity and NaN may be suffixed with a type specifier
        /// (`f`, `F`, `d`, `D`). For example, `-Infinityf`, `InfinityD`, `NaNf`, etc.
        js_like: bool,

        /// Allows Rust-like infinity literals, i.e. `inf` and `nan`. Infinity literals, and
        /// NaN literals depending on whether `signed_nan` is set, may be prefixed with a sign
        /// (`+`, `-`). Both infinity and NaN may be suffixed with a type specifier
        /// (`f`, `F`, `d`, `D`). For example, `-inff`, `infD`, `nanf`, etc.
        ///
        /// Note that Rust does not necessarily allow `inf` as a constant in code, but the
        /// [`FromStr`] implementation on floats accepts `inf` as infinity.
        rust_like: bool,

        /// Allows the infinity loop symbol, i.e. `∞`. These literals may be prefixed
        /// with a sign (`+`, `-`) and suffixed with a type specifier (`f`, `F`, `d`, `D`). For
        /// example, `-∞f`, `∞D`, etc.
        ///
        /// This has the side effect that the infinity loop symbol will be accepted in identifiers.
        lemniscate: bool,
    },
}

impl ParseConfig {
    /// Returns the [`ParseConfig`] that accepts SNBT exactly as vanilla Minecraft accepts it.
    ///
    /// Minecraft SNBT is very lenient but quite strict in its structuring. Values are separated by
    /// commas and comments are not allowed.
    pub const fn minecraft() -> Self {
        Self {
            nonfinite_floats: NonfiniteFloats::minecraft(),

            ops: Ops::minecraft(),

            hetrogenous_lists: HetrogenousLists::WrapInCompound {
                wrap_compounds: false
            },

            comments: Comments {
                hash:    false,
                c_style: false,
                tildes:  false,
            },

            extra_commas: ExtraCommas::SingleTrailing,

            allow_line_break_separation: false,

            string_format: StringFormat::minecraft(),

            integer_format: IntegerFormat {
                allow_unsigned:    true,
                allow_booleans:    true,
                allow_hex_and_bin: true,
            },

            name_format: NameFormat::AnyWord,

            array_format: ArrayFormat {
                allow_spaced_prefix:    true,
                allow_lowercase_prefix: false,
                allow_booleans:         false,
                allow_coercion:         true,
            },

            allow_equals_sign: false,
        }
    }

    /// Returns the [`ParseConfig`] that accepts SNBT exactly as FTB (FeadTheBeast) accepts it.
    ///
    /// FTB SNBT is a bit more lenient than Minecraft SNBT, as it allows comments and allows values
    /// to be separated by line breaks.
    pub const fn ftb() -> Self {
        Self {
            nonfinite_floats: NonfiniteFloats::ftb(),

            ops: Ops::none(),

            hetrogenous_lists: HetrogenousLists::Disallowed,

            comments: Comments {
                hash:    true,
                c_style: false,
                tildes:  false,
            },

            extra_commas: ExtraCommas::Anywhere,

            allow_line_break_separation: true,

            string_format: StringFormat::ftb(),

            integer_format: IntegerFormat {
                allow_unsigned:    true,
                allow_booleans:    true,
                allow_hex_and_bin: true,
            },

            name_format: NameFormat::AnyWord,

            array_format: ArrayFormat {
                allow_spaced_prefix:    true,
                allow_lowercase_prefix: true,
                allow_booleans:         false,
                allow_coercion:         true,
            },

            allow_equals_sign: true,
        }
    }

    /// Returns the [`ParseConfig`] that accepts SNBT as leniently as possible.
    pub const fn lenient() -> Self {
        Self {
            nonfinite_floats: NonfiniteFloats::lenient(),

            ops: Ops::all(),

            hetrogenous_lists: HetrogenousLists::WrapInCompound {
                wrap_compounds: false
            },

            comments: Comments {
                hash:    true,
                c_style: true,
                tildes:  true,
            },

            extra_commas: ExtraCommas::Anywhere,

            allow_line_break_separation: true,

            string_format: StringFormat::lenient(),

            integer_format: IntegerFormat {
                allow_unsigned:    true,
                allow_booleans:    true,
                allow_hex_and_bin: true,
            },

            name_format: NameFormat::AnyWord,

            array_format: ArrayFormat {
                allow_spaced_prefix:    true,
                allow_lowercase_prefix: true,
                allow_booleans:         true,
                allow_coercion:         true,
            },

            allow_equals_sign: true,
        }
    }
}

impl StringFormat {
    pub const fn old_minecraft() -> Self {
        Self {
            allow_identifiers:         true,
            single_quotes:             true,
            double_quotes:             true,
            ignnore_invalid_escapes:   true,
            esc_nul:                   false,
            esc_single_quote:          false,
            esc_double_quote:          false,
            esc_slash:                 false,
            esc_backslash:             false,
            esc_lf:                    false,
            esc_cr:                    false,
            esc_ff:                    false,
            esc_space:                 false,
            esc_tab:                   false,
            esc_bs:                    false,
            esc_unicode_8:             false,
            esc_unicode_16:            false,
            esc_unicode_32:            false,
            esc_unicode_name:          false,
            esc_chars:                 false,
            allow_line_breaks:         false,
            allow_escaped_line_breaks: false,
        }
    }

    pub const fn minecraft() -> Self {
        Self {
            allow_identifiers:         true,
            single_quotes:             true,
            double_quotes:             true,
            ignnore_invalid_escapes:   false,
            esc_nul:                   false,
            esc_single_quote:          true,
            esc_double_quote:          true,
            esc_slash:                 false,
            esc_backslash:             true,
            esc_lf:                    true,
            esc_cr:                    true,
            esc_ff:                    true,
            esc_space:                 false,
            esc_tab:                   true,
            esc_bs:                    true,
            esc_unicode_8:             true,
            esc_unicode_16:            true,
            esc_unicode_32:            true,
            esc_unicode_name:          true,
            esc_chars:                 false,
            allow_line_breaks:         false,
            allow_escaped_line_breaks: false,
        }
    }

    pub const fn ftb() -> Self {
        Self {
            allow_identifiers:         true,
            single_quotes:             true,
            double_quotes:             true,
            ignnore_invalid_escapes:   false,
            esc_nul:                   false,
            esc_single_quote:          false,
            esc_double_quote:          true,
            esc_slash:                 false,
            esc_backslash:             true,
            esc_lf:                    true,
            esc_cr:                    true,
            esc_ff:                    true,
            esc_space:                 true,
            esc_tab:                   true,
            esc_bs:                    true,
            esc_unicode_8:             false,
            esc_unicode_16:            false,
            esc_unicode_32:            false,
            esc_unicode_name:          false,
            esc_chars:                 false,
            allow_line_breaks:         false,
            allow_escaped_line_breaks: false,
        }
    }

    pub const fn lenient() -> Self {
        Self {
            allow_identifiers:         true,
            single_quotes:             true,
            double_quotes:             true,
            ignnore_invalid_escapes:   false,
            esc_nul:                   true,
            esc_single_quote:          true,
            esc_double_quote:          true,
            esc_slash:                 true,
            esc_backslash:             true,
            esc_lf:                    true,
            esc_cr:                    true,
            esc_ff:                    true,
            esc_space:                 true,
            esc_tab:                   true,
            esc_bs:                    true,
            esc_unicode_8:             true,
            esc_unicode_16:            true,
            esc_unicode_32:            true,
            esc_unicode_name:          true,
            esc_chars:                 false,
            allow_line_breaks:         true,
            allow_escaped_line_breaks: true,
        }
    }

    pub const fn json() -> Self {
        Self {
            allow_identifiers:         false,
            single_quotes:             false,
            double_quotes:             true,
            ignnore_invalid_escapes:   false,
            esc_nul:                   false,
            esc_single_quote:          true,
            esc_double_quote:          true,
            esc_slash:                 true,
            esc_backslash:             true,
            esc_lf:                    true,
            esc_cr:                    true,
            esc_ff:                    true,
            esc_space:                 false,
            esc_tab:                   true,
            esc_bs:                    true,
            esc_unicode_8:             false,
            esc_unicode_16:            true,
            esc_unicode_32:            false,
            esc_unicode_name:          false,
            esc_chars:                 false,
            allow_line_breaks:         false,
            allow_escaped_line_breaks: false,
        }
    }

    pub const fn json5_like() -> Self {
        Self {
            allow_identifiers:         false,
            single_quotes:             true,
            double_quotes:             true,
            ignnore_invalid_escapes:   false,
            esc_nul:                   true,
            esc_single_quote:          true,
            esc_double_quote:          true,
            esc_slash:                 true,
            esc_backslash:             true,
            esc_lf:                    true,
            esc_cr:                    true,
            esc_ff:                    true,
            esc_space:                 false,
            esc_tab:                   true,
            esc_bs:                    true,
            esc_unicode_8:             true,
            esc_unicode_16:            true,
            esc_unicode_32:            false,
            esc_unicode_name:          false,
            esc_chars:                 true,
            allow_line_breaks:         false,
            allow_escaped_line_breaks: true,
        }
    }
}

impl NonfiniteFloats {
    pub const fn minecraft() -> Self {
        Self::Disabled {
            allow_too_large_finite: false,
        }
    }

    pub const fn ftb() -> Self {
        Self::Enabled {
            allow_too_large_finite: true,
            signed_nan:             false,
            js_like:                true,
            rust_like:              false,
            lemniscate:             true,
        }
    }

    pub const fn lenient() -> Self {
        Self::Enabled {
            allow_too_large_finite: true,
            signed_nan:             true,
            js_like:                true,
            rust_like:              true,
            lemniscate:             true,
        }
    }

    pub const fn rust() -> Self {
        Self::Enabled {
            allow_too_large_finite: true,
            signed_nan:             true,
            js_like:                false,
            rust_like:              true,
            lemniscate:             false,
        }
    }

    pub const fn json5() -> Self {
        Self::Enabled {
            allow_too_large_finite: true,
            signed_nan:             true,
            js_like:                true,
            rust_like:              false,
            lemniscate:             false,
        }
    }
}

impl Ops {
    pub const fn minecraft() -> Self {
        Self {
            bool:   true,
            uuid:   true,
            base64: false,
            pack:   false,
        }
    }

    pub const fn all() -> Self {
        Self {
            bool:   true,
            uuid:   true,
            base64: true,
            pack:   true,
        }
    }

    pub const fn none() -> Self {
        Self {
            bool:   false,
            uuid:   false,
            base64: false,
            pack:   false,
        }
    }
}
