use TypeSuffix::*;

use super::config::ParseConfig;
use crate::snbt::NonfiniteFloats;
use crate::tag::Tag;

/// Parses a numeral token.
///
/// The function accepts the numeral string that was emitted by the lexer, and returns a [`Tag`]
/// from that numeral or [`None`] if the numeral is malformed. The [`ParseConfig`] of the parser
/// must also be provided since it has several options that alter parsing.
pub fn parse_numeral(numeral: &String, config: &ParseConfig) -> Option<Tag> {
    let mut parser = NumeralParser {
        bytes: numeral.as_bytes(),
        index: 0,
        config,
    };

    parser.parse()
}

/// Parser state for numeral parsing.
struct NumeralParser<'a, 'b> {
    bytes: &'a [u8],
    index: usize,
    config: &'b ParseConfig,
}

/// The type suffix supported by SNBT.
enum TypeSuffix {
    UB,
    US,
    UI,
    UL,
    SB,
    SS,
    SI,
    SL,
    F,
    D,

    /// No suffix
    NA,
}

/// Implementation for [`NumeralParser`].
impl NumeralParser<'_, '_> {
    fn parse(&mut self) -> Option<Tag> {
        // Sign
        let neg = self.neg();

        // Hexadecimal or binary
        // The length check ensures we don't parse the byte value 0b as malformed binary
        if self.config.integer_format.allow_hex_and_bin && self.bytes.len() > 2 {
            if let Some(raw) = self.parse_hex() {
                return RawInt(neg, raw).convert(self.type_suffix()?);
            }

            if let Some(raw) = self.parse_bin() {
                return RawInt(neg, raw).convert(self.type_suffix()?);
            }
        }

        // Raw float data
        let mut f = RawFloat(Vec::new());
        if neg {
            f.append_byte(b'-');
        }

        // Check config if we allow non-finite values
        let allow_infinity = match self.config.nonfinite_floats {
            NonfiniteFloats::Disabled {
                allow_too_large_finite, ..
            } => allow_too_large_finite,

            NonfiniteFloats::InfinityOnly {
                allow_too_large_finite, ..
            } => allow_too_large_finite,

            NonfiniteFloats::Enabled {
                allow_too_large_finite, ..
            } => allow_too_large_finite,
        };

        // Float without decimal part
        if let b"." = self.la(1) {
            f.append_byte(b'0');

            self.fraction(false, &mut f)?;
            self.exponent(&mut f)?;
            return f.convert(self.type_suffix()?, allow_infinity);
        }

        // We must have a decimal bit now
        let dec = self.digits(false, usize::MAX, 10)?;

        // Float with fractional part
        if let b"." = self.la(1) {
            f.append_digits(dec);
            self.fraction(true, &mut f)?;
            self.exponent(&mut f)?;
            return f.convert(self.type_suffix()?, allow_infinity);
        }

        // Float without fractional part
        if let &[b'e' | b'E'] = self.la(1) {
            f.append_digits(dec);
            self.exponent(&mut f)?;
            return f.convert(self.type_suffix()?, allow_infinity);
        }

        // We must have an integral value at this point
        let suffix = self.type_suffix()?;

        // Integral float
        if let F | D = suffix {
            // 123d
            // 123f

            f.append_digits(dec);
            return f.convert(suffix, allow_infinity);
        }

        // At this point all float options are eliminated so it's an integer

        // Convert to binary while preventing overflow
        let mut raw = 0u64;
        for dig in dec {
            let (r, false) = raw.overflowing_mul(10) else {
                return None;
            };
            raw = r;

            let (r, false) = raw.overflowing_add(dig as u64) else {
                return None;
            };
            raw = r;
        }

        RawInt(neg, raw).convert(suffix)
    }

    /// Advances `n` steps through the input.
    fn advance(&mut self, n: usize) {
        self.index = (self.index + n).min(self.bytes.len());
    }

    /// Looks ahead up to `n` characters from the input as a slice.
    /// If the remaining input is shorter, a shorter input is returned.
    fn la(&self, n: usize) -> &[u8] {
        let end = (self.index + n).min(self.bytes.len());
        &self.bytes[self.index..end]
    }

    /// Parses a sign, either `+` or `-`, or nothing at all, and returns whether the sign indicates
    /// a negative number.
    fn neg(&mut self) -> bool {
        match self.la(1) {
            b"+" => {
                self.advance(1);
                false
            },
            b"-" => {
                self.advance(1);
                true
            },
            _ => false,
        }
    }

    /// Parses the fractional part of a float if present and returns [`Some`] if it successfully
    /// did so. Returns [`None`] if it is malformed. Appends the found symbols to the [`RawFloat`]
    /// value.
    fn fraction(&mut self, allow_omit: bool, f: &mut RawFloat) -> Option<()> {
        let &[b'.'] = self.la(1) else {
            return Some(());
        };

        self.advance(1);
        f.append_byte(b'.');

        f.append_digits(self.digits(allow_omit, usize::MAX, 10)?);

        Some(())
    }

    /// Parses the exponent part of a float if present and returns [`Some`] if it successfully
    /// did so. Returns [`None`] if it is malformed. Appends the found symbols to the [`RawFloat`]
    /// value.
    fn exponent(&mut self, f: &mut RawFloat) -> Option<()> {
        let &[b'e' | b'E'] = self.la(1) else {
            return Some(());
        };

        self.advance(1);
        f.append_byte(b'e');

        if self.neg() {
            f.append_byte(b'-');
        }

        f.append_digits(self.digits(false, usize::MAX, 10)?);

        Some(())
    }

    /// Parses a type suffix and checks if the end of the input was reached.
    /// Since type suffixes are the last bit of the numeral, this [`None`] if there was trailing
    /// input after the suffix.
    fn type_suffix(&mut self) -> Option<TypeSuffix> {
        let allow_signedness = self.config.integer_format.allow_unsigned;

        let (suf, adv) = match self.la(2) {
            // Unsigned suffixes
            b"ub" | b"uB" | b"Ub" | b"UB" if allow_signedness => (UB, 2),
            b"us" | b"uS" | b"Us" | b"US" if allow_signedness => (US, 2),
            b"ui" | b"uI" | b"Ui" | b"UI" if allow_signedness => (UI, 2),
            b"ul" | b"uL" | b"Ul" | b"UL" if allow_signedness => (UL, 2),

            // Signed suffixes
            b"sb" | b"sB" | b"Sb" | b"SB" if allow_signedness => (SB, 2),
            b"ss" | b"sS" | b"Ss" | b"SS" if allow_signedness => (SS, 2),
            b"si" | b"sI" | b"Si" | b"SI" if allow_signedness => (SI, 2),
            b"sl" | b"sL" | b"Sl" | b"SL" if allow_signedness => (SL, 2),

            // Unannotated suffixes
            &[b'b' | b'B', ..] => (SB, 1),
            &[b's' | b'S', ..] => (SS, 1),
            &[b'i' | b'I', ..] => (SI, 1),
            &[b'l' | b'L', ..] => (SL, 1),
            &[b'f' | b'F', ..] => (F, 1),
            &[b'd' | b'D', ..] => (D, 1),

            // No suffix
            &[..] => (NA, 0),
        };

        self.advance(adv);

        // Type suffixes must be the last bit of the numeral
        if self.index != self.bytes.len() {
            return None;
        }

        Some(suf)
    }

    /// Expects and parses a hexadecimal literal into a raw int value, or returns [`None`].
    fn parse_hex(&mut self) -> Option<u64> {
        let b"0x" = self.la(2) else {
            return None;
        };

        self.advance(2);

        let hexits = self.digits(false, 16, 16)?;
        let mut n = 0u64;

        for h in hexits {
            n <<= 4;
            n |= h as u64;
        }

        Some(n)
    }

    /// Expects and parses a binary literal into a raw int value, or returns [`None`].
    fn parse_bin(&mut self) -> Option<u64> {
        let b"0b" = self.la(2) else {
            return None;
        };

        self.advance(2);

        let hexits = self.digits(false, 64, 2)?;
        let mut n = 0u64;

        for h in hexits {
            n <<= 1;
            n |= h as u64;
        }

        Some(n)
    }

    /// Parses a sequence of digits under a given radix. A vector is returned with the values of the
    /// digits (not the ASCII characters themselves).
    ///
    /// Underscores are ignored if not at the start or the end of the sequence, so `1_2` is valid
    /// and parses the same as `12`, while `_12` and `12_` are not valid.
    fn digits(&mut self, allow_none: bool, mut quota: usize, radix: u8) -> Option<Vec<u8>> {
        let mut digits = Vec::new();
        let mut last_underscore = false;

        if let b"_" = self.la(1) {
            return None;
        }

        while let &[c] = self.la(1) {
            if c == b'_' {
                last_underscore = true;
                continue;
            }

            last_underscore = false;

            if quota == 0 {
                return None;
            }

            let val = match c {
                b'a'..=b'z' => c - b'a' + 10,
                b'A'..=b'Z' => c - b'A' + 10,
                b'0'..=b'9' => c - b'0',
                _ => break,
            };

            if val > radix {
                break;
            }

            digits.push(val);
            self.advance(1);
            quota -= 1;
        }

        if last_underscore {
            return None;
        }

        if digits.is_empty() && !allow_none {
            return None;
        }

        Some(digits)
    }
}

/// A raw integer value.
///
/// Integer values are parsed into binary values of type [`u64`], since no int in SNBT can reach
/// higher than the limits of [`u64`], and the sign is kept separate.
struct RawInt(bool, u64);

/// Macro generating a conversion method for a specific integer size and signedness. The generated
/// conversion method checks the size of the raw input against the limits and converts the result to
/// a signed integer.
///
/// E.g. for [`UI`], use:
/// ```no_run
/// convert!(ub, u32, i32);
/// ```
/// to generate:
/// ```no_run
/// fn ub(self) -> Option<i32> {
///     let RawInt(neg, abs) = self.validate(u32::MIN as u64, u32::MAX as u64)?;
///     if neg {
///         Some((abs as i32).wrapping_neg())
///     } else {
///         Some(abs as i32)
///     }
/// }
/// ```
macro_rules! convert {
    ($name:ident, $lim:ident $(as $unsigned:ty)?, $res:ty) => {
        fn $name(self) -> Option<$res> {
            let RawInt(neg, abs) = self.validate(
                // we want bits of the min and max values in the least
                // significant bits of u64, with the rest zero; but casting
                // directly to u64 will cause those remainder bits to be 1
                // if the limit is negative
                $lim::MIN $(as $unsigned)? as u64,
                $lim::MAX as u64
            )?;
            if neg {
                Some((abs as $res).wrapping_neg())
            } else {
                Some(abs as $res)
            }
        }
    };
}

impl RawInt {
    fn convert(self, suffix: TypeSuffix) -> Option<Tag> {
        Some(match suffix {
            UB => Tag::Byte(self.ub()?),
            US => Tag::Short(self.us()?),
            UI => Tag::Int(self.ui()?),
            UL => Tag::Long(self.ul()?),
            SB => Tag::Byte(self.sb()?),
            SS => Tag::Short(self.ss()?),
            NA | SI => Tag::Int(self.si()?),
            SL => Tag::Long(self.sl()?),
            F | D => None?,
        })
    }

    fn validate(self, max_neg: u64, max_pos: u64) -> Option<Self> {
        let RawInt(neg, abs) = self;

        println!("{} {}", max_pos, abs);

        if (neg && abs > max_neg) || (!neg && abs > max_pos) {
            None
        } else {
            Some(self)
        }
    }

    convert!(ub, u8, i8);
    convert!(us, u16, i16);
    convert!(ui, u32, i32);
    convert!(ul, u64, i64);
    convert!(sb, i8 as u8, i8);
    convert!(ss, i16 as u16, i16);
    convert!(si, i32 as u32, i32);
    convert!(sl, i64 as u64, i64);
}

/// A raw float value.
///
/// Raw floats are parsed using Rust's built-in flaot parser, but are validated by the SNBT parser
/// beforehand. They are represented as byte strings containing the ASCII characters of the float
/// numeral in a format that Rust's standard library will comprehend.
struct RawFloat(Vec<u8>);

impl RawFloat {
    fn append_digits(&mut self, digits: Vec<u8>) {
        self.0.append(&mut digits.into_iter().map(|v| v + b'0').collect());
    }

    fn append_byte(&mut self, b: u8) {
        self.0.push(b);
    }

    fn f(value: f32, allow_nonfinite: bool) -> Option<Tag> {
        if value.is_finite() || allow_nonfinite {
            Some(Tag::Float(value))
        } else {
            None
        }
    }

    fn d(value: f64, allow_nonfinite: bool) -> Option<Tag> {
        if value.is_finite() || allow_nonfinite {
            Some(Tag::Double(value))
        } else {
            None
        }
    }

    fn convert(self, suffix: TypeSuffix, allow_nonfinite: bool) -> Option<Tag> {
        // We have already verified that this is ASCII data, no need to check it again.
        let str = unsafe { String::from_utf8_unchecked(self.0) };

        match suffix {
            F => Self::f(str.parse().ok()?, allow_nonfinite),
            D | NA => Self::d(str.parse().ok()?, allow_nonfinite),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use crate::snbt::ParseConfig;
    use crate::snbt::parser::numeral::parse_numeral;
    use crate::tag::Tag;

    const CFG: ParseConfig = ParseConfig::minecraft();

    macro_rules! st {
        ($config:ident | $name:ident, $input:expr, $exp:pat) => {
            #[test]
            fn $name() {
                let input = String::from($input);

                let result = parse_numeral(&input, &$config);
                assert_matches!(result, $exp);
            }
        };
    }

    st!(CFG | long_1, "0l", Some(Tag::Long(0)));
    st!(CFG | long_2, "412l", Some(Tag::Long(412)));
    st!(CFG | long_3, "+412l", Some(Tag::Long(412)));
    st!(CFG | long_4, "-412l", Some(Tag::Long(-412)));

    st!(CFG | int_1, "0", Some(Tag::Int(0)));
    st!(CFG | int_2, "412", Some(Tag::Int(412)));
    st!(CFG | int_3, "+412", Some(Tag::Int(412)));
    st!(CFG | int_4, "-412", Some(Tag::Int(-412)));

    st!(CFG | short_1, "0s", Some(Tag::Short(0)));
    st!(CFG | short_2, "412s", Some(Tag::Short(412)));
    st!(CFG | short_3, "+412s", Some(Tag::Short(412)));
    st!(CFG | short_4, "-412s", Some(Tag::Short(-412)));

    st!(CFG | byte_1, "0b", Some(Tag::Byte(0)));
    st!(CFG | byte_2, "18b", Some(Tag::Byte(18)));
    st!(CFG | byte_3, "+18b", Some(Tag::Byte(18)));
    st!(CFG | byte_4, "-18b", Some(Tag::Byte(-18)));

    st!(CFG | byte_too_big, "128b", None);
    st!(CFG | short_too_big, "32768s", None);
    st!(CFG | int_too_big, "2147483648i", None);
    st!(CFG | long_too_big, "9223372036854775808l", None);

    st!(CFG | byte_too_small, "-129b", None);
    st!(CFG | short_too_small, "-32769s", None);
    st!(CFG | int_too_small, "-2147483649i", None);
    st!(CFG | long_too_small, "-9223372036854775809l", None);

    st!(CFG | byte_max, "127b", Some(Tag::Byte(127)));
    st!(CFG | short_max, "32767s", Some(Tag::Short(32767)));
    st!(CFG | int_max, "2147483647i", Some(Tag::Int(2147483647)));
    st!(CFG | long_max, "9223372036854775807l", Some(Tag::Long(9223372036854775807)));

    st!(CFG | byte_min, "-128b", Some(Tag::Byte(-128)));
    st!(CFG | short_min, "-32768s", Some(Tag::Short(-32768)));
    st!(CFG | int_min, "-2147483648i", Some(Tag::Int(-2147483648)));
    st!(CFG | long_min, "-9223372036854775808l", Some(Tag::Long(-9223372036854775808)));

    st!(CFG | float_z, "0f", Some(Tag::Float(0.0)));
    st!(CFG | float_df, "3.14f", Some(Tag::Float(3.14)));
    st!(CFG | float_dd, "3.f", Some(Tag::Float(3.0)));
    st!(CFG | float_d, "3f", Some(Tag::Float(3.0)));
    st!(CFG | float_f, ".14f", Some(Tag::Float(0.14)));
    st!(CFG | float_dfe, "3.14e21f", Some(Tag::Float(3.14e21)));
    st!(CFG | float_dde, "3.e21f", Some(Tag::Float(3e21)));
    st!(CFG | float_de, "3e21f", Some(Tag::Float(3e21)));
    st!(CFG | float_fe, ".14e21f", Some(Tag::Float(0.14e21)));

    st!(CFG | double_z, "0d", Some(Tag::Double(0.0)));
    st!(CFG | double_df, "3.14", Some(Tag::Double(3.14)));
    st!(CFG | double_dd, "3.", Some(Tag::Double(3.0)));
    st!(CFG | double_d, "3d", Some(Tag::Double(3.0)));
    st!(CFG | double_f, ".14", Some(Tag::Double(0.14)));
    st!(CFG | double_dfe, "3.14e21", Some(Tag::Double(3.14e21)));
    st!(CFG | double_dde, "3.e21", Some(Tag::Double(3e21)));
    st!(CFG | double_de, "3e21", Some(Tag::Double(3e21)));
    st!(CFG | double_fe, ".14e21", Some(Tag::Double(0.14e21)));

    st!(CFG | double_too_big, "79769313486231571E+308", None);
    st!(CFG | double_max, "1.7976931348623157E+308", Some(Tag::Double(1.7976931348623157E+308)));
}
