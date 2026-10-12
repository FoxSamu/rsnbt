use std::mem::replace;

use super::config::*;
use super::error::*;
use super::lex::*;
use super::numeral::*;
use crate::tag::*;
use crate::tag_map;

// TODO check with actual Minecraft/FTB SNBT parser
// - can type prefixes in arrays have spaces? -> Yes
// - boolean literals in bool? -> Yes
// - ops in ops? -> Yes
// - what is the exact NaN and infinity formatting allowed in FTB SNBT?

// Ok so FTB SNBT accepts the literal ∞ symbol as infinity and inherently therefore as part of
// words.

/*

FTB
===

- no signedness specifiers.
- ∞ and Infinity as infinity literal.
- NaN as not a number literal.
- no line breaks allowed in strings.
- only \t \b \n \r \f \\ and \" are valid (weirdly not \').
- array type tags can be lowercase.
- it is possible to use = as kv separator.
- explicit use of the end tag => we'll just not add these to objects.


 */

// TODO uuid parsing

#[derive(Debug)]
pub struct Parser<'a, I, E>
where
    I: Iterator<Item = ParseResult<Token, E>>,
{
    tokens: I,
    config: &'a ParseConfig,
    next:   Option<Token>,
    index:  u64,
}

macro_rules! consume {
    ($self:expr, $v:pat_param) => {
        let Some(tok) = $self.advance()? else {
            unreachable!()
        };

        let $v = tok.value;
    };

    ($self:expr) => {
        let Some(_) = $self.advance()? else {
            unreachable!()
        };
    };
}

macro_rules! expect {
    ($self:expr, $($k:ident)|+) => {
        let Some(($($k)|+, _)) = $self.next.decompose() else {
            let got = $self.next.as_ref().map(|v| v.kind);
            return $self.syntax_error(ExpectedToken(vec![$($k),+], false, got));
        };
        $self.advance()?;
    };

    ($self:expr, $($k:ident)|+, $v:pat_param $(, $idx:pat_param)?) => {
        let Some(($($k)|+, _)) = $self.next.decompose() else {
            let got = $self.next.as_ref().map(|v| v.kind);
            return $self.syntax_error(ExpectedToken(vec![$($k),+], false, got));
        };

        let Some(tok) = $self.advance()? else {
            unreachable!()
        };

        let $v = tok.value;
        $(let $idx = tok.index;)?
    };
}

macro_rules! diverge {
    {
        $self:expr
        $(, $($k:ident)|+ $(if $guard:expr)? => |$v:pat_param| $r:expr)*
        $(, * $(if $rguard:expr)? => $rr:expr)?
        $(, ! $(if $eguard:expr)? => $er:expr)?
        $(,)?
    } => {
        match $self.next.decompose() {
            $(
                Some(($($k)|+, _)) $(if $guard)? => {
                    let Some(tok) = $self.advance()? else {
                        unreachable!()
                    };

                    let $v = tok.value;
                    $r
                }
            )*
            $(
                Some((_, _)) $(if $rguard)? => {
                    $rr
                }
            )*
            $(
                // the if true prevents the compiler from generate a warning when
                // the diverge is exhaustive
                None if true $(&& ($eguard))? => {
                    $er
                }
            )*
            _ => {
                let mut expected = Vec::new();
                $(
                    $(if $guard)? {
                        $(expected.push($k);)+
                    }
                )*
                expected.dedup();

                let end = false $(
                    || {
                        stringify!($er); // <- phantom use of $er
                        true
                    }
                )?;

                let got = $self.next.as_ref().map(|v| v.kind);
                $self.syntax_error(ExpectedToken(expected, end, got))?
            },
        }
    };
}

macro_rules! diverge_ahead {
    {
        $self:expr
        $(, $($k:ident)|+ $(if $guard:expr)? => |$v:pat_param| $r:expr)*
        $(, * $(if $rguard:expr)? => $rr:expr)?
        $(, ! $(if $eguard:expr)? => $er:expr)?
        $(,)?
    } => {
        match $self.next.decompose() {
            $(
                Some(($($k)|+, $v)) $(if $guard)? => {
                    $r
                }
            )*
            $(
                Some((_, _)) $(if $rguard)? => {
                    $rr
                }
            )?
            $(
                // the if true prevents the compiler from generate a warning when
                // the diverge is exhaustive
                None if true $(&& ($eguard))? => {
                    $er
                }
            )?
            _ => {
                let mut expected = Vec::new();
                $(
                    $(if $guard)? {
                        $(expected.push($k);)+
                    }
                )*
                expected.dedup();

                let end = false $(
                    || {
                        stringify!($er); // <- phantom use of $er
                        true
                    }
                )?;

                let got = $self.next.as_ref().map(|v| v.kind);
                $self.syntax_error(ExpectedToken(expected, end, got))?
            },
        }
    };
}


const LEMNISCATE: &[u8] = b"\x22\x1E";
const LEMNISCATE_F: &[u8] = b"\x22\x1Ef";
const LEMNISCATE_FF: &[u8] = b"\x22\x1EF";
const LEMNISCATE_D: &[u8] = b"\x22\x1Ed";
const LEMNISCATE_DD: &[u8] = b"\x22\x1ED";

const POS_LEMNISCATE: &[u8] = b"+\x22\x1E";
const POS_LEMNISCATE_F: &[u8] = b"+\x22\x1Ef";
const POS_LEMNISCATE_FF: &[u8] = b"+\x22\x1EF";
const POS_LEMNISCATE_D: &[u8] = b"+\x22\x1Ed";
const POS_LEMNISCATE_DD: &[u8] = b"+\x22\x1ED";

const NEG_LEMNISCATE: &[u8] = b"-\x22\x1E";
const NEG_LEMNISCATE_F: &[u8] = b"-\x22\x1Ef";
const NEG_LEMNISCATE_FF: &[u8] = b"-\x22\x1EF";
const NEG_LEMNISCATE_D: &[u8] = b"-\x22\x1Ed";
const NEG_LEMNISCATE_DD: &[u8] = b"-\x22\x1ED";

const NAND: Tag = Tag::Double(f64::NAN);
const INFD: Tag = Tag::Double(f64::INFINITY);
const NINFD: Tag = Tag::Double(f64::NEG_INFINITY);
const NANF: Tag = Tag::Float(f32::NAN);
const INFF: Tag = Tag::Float(f32::INFINITY);
const NINFF: Tag = Tag::Float(f32::NEG_INFINITY);

const NO_OPS: Ops = Ops::none();
const NO_COMMENTS: Comments = Comments {
    hash:    false,
    c_style: false,
    tildes:  false,
};

enum Collector {
    Bytes(Vec<i8>),
    Ints(Vec<i32>),
    Longs(Vec<i64>),
    List(Vec<Tag>),
}

impl Collector {
    fn add_tag(&mut self, tag: Tag) {
        match self {
            Collector::Bytes(v) => v.push(tag.as_i8().unwrap()),
            Collector::Ints(v) => v.push(tag.as_i32().unwrap()),
            Collector::Longs(v) => v.push(tag.as_i64().unwrap()),
            Collector::List(v) => v.push(tag),
        }
    }
}

impl<'a, I, E> Parser<'a, I, E>
where
    I: Iterator<Item = ParseResult<Token, E>>,
{
    pub fn new(mut tokens: I, config: &'a ParseConfig) -> ParseResult<Self, E> {
        let next = tokens.next().transpose()?;

        Ok(Self {
            tokens,
            config,
            next,
            index: 0,
        })
    }

    fn syntax_error<R>(&self, error: SyntaxError) -> ParseResult<R, E> {
        Err(Syntax(self.index, error))
    }

    fn advance(&mut self) -> ParseResult<Option<Token>, E> {
        let Some(cur) = &self.next else {
            return Ok(None);
        };

        self.index = cur.index;

        let next = self.tokens.next().transpose()?;
        Ok(replace(&mut self.next, next))
    }

    fn numeral(&self, numeral: String) -> ParseResult<Tag, E> {
        let (nonfinites, inf, nan, infinity, big_nan, lemniscate, sig_nan) = match self.config.nonfinite_floats {
            NonfiniteFloats::Disabled {
                ..
            } => (false, false, false, false, false, false, false),

            NonfiniteFloats::InfinityOnly {
                js_like,
                rust_like,
                lemniscate,
                ..
            } => (true, rust_like, false, js_like, false, lemniscate, false),

            NonfiniteFloats::Enabled {
                js_like,
                rust_like,
                lemniscate,
                signed_nan,
                ..
            } => (true, rust_like, rust_like, js_like, js_like, lemniscate, signed_nan),
        };

        if nonfinites {
            let tag = match numeral.as_bytes() {
                b"nan" | b"nand" | b"nanD" if nan => Some(NAND),
                b"+nan" | b"+nand" | b"+nanD" if nan && sig_nan => Some(NAND),
                b"-nan" | b"-nand" | b"-nanD" if nan && sig_nan => Some(NAND),
                b"inf" | b"infd" | b"infD" if inf => Some(INFD),
                b"+inf" | b"+infd" | b"+infD" if inf => Some(INFD),
                b"-inf" | b"-infd" | b"-infD" if inf => Some(NINFD),

                b"NaN" | b"NaNd" | b"NaND" if big_nan => Some(NAND),
                b"+NaN" | b"+NaNd" | b"+NaND" if big_nan && sig_nan => Some(NAND),
                b"-NaN" | b"-NaNd" | b"-NaND" if big_nan && sig_nan => Some(NAND),
                b"Infinity" | b"Infinityd" | b"InfinityD" if infinity => Some(INFD),
                b"+Infinity" | b"+Infinityd" | b"+InfinityD" if infinity => Some(INFD),
                b"-Infinity" | b"-Infinityd" | b"-InfinityD" if infinity => Some(NINFD),

                LEMNISCATE | LEMNISCATE_D | LEMNISCATE_DD if lemniscate => Some(INFD),
                POS_LEMNISCATE | POS_LEMNISCATE_D | POS_LEMNISCATE_DD if lemniscate => Some(INFD),
                NEG_LEMNISCATE | NEG_LEMNISCATE_D | NEG_LEMNISCATE_DD if lemniscate => Some(NINFD),

                b"nanf" | b"nanF" if nan => Some(NANF),
                b"+nanf" | b"+nanF" if nan && sig_nan => Some(NANF),
                b"-nanf" | b"-nanF" if nan && sig_nan => Some(NANF),
                b"+inff" | b"+infF" if inf => Some(INFF),
                b"-inff" | b"-infF" if inf => Some(NINFF),

                b"NaNf" | b"NaNF" if big_nan => Some(NANF),
                b"+NaNf" | b"+NaNF" if big_nan && sig_nan => Some(NANF),
                b"-NaNf" | b"-NaNF" if big_nan && sig_nan => Some(NANF),
                b"+Infinityf" | b"+InfinityF" if infinity => Some(INFF),
                b"-Infinityf" | b"-InfinityF" if infinity => Some(NINFF),

                LEMNISCATE_F | LEMNISCATE_FF if lemniscate => Some(INFD),
                POS_LEMNISCATE_F | POS_LEMNISCATE_FF if lemniscate => Some(INFD),
                NEG_LEMNISCATE_F | NEG_LEMNISCATE_FF if lemniscate => Some(NINFD),

                _ => None,
            };

            if let Some(tag) = tag {
                return Ok(tag);
            }
        }

        match parse_numeral(&numeral, self.config) {
            Some(tag) => Ok(tag),
            None => self.syntax_error(MalformedNumeral(numeral)),
        }
    }

    fn flatten_hetrogenous_list(&self, list: Vec<Tag>) -> ParseResult<TagList, E> {
        let Some(first) = list.first() else {
            return Ok(TagList::Empty);
        };

        let id = first.id();
        for elem in list.iter() {
            if elem.id() != id {
                let (into_list, wrap_always) = match self.config.hetrogenous_lists {
                    HetrogenousLists::Disallowed => {
                        return self.syntax_error(DisallowedHetrogenousList(id, elem.id()));
                    },
                    HetrogenousLists::WrapInCompound {
                        wrap_compounds,
                    } => (false, wrap_compounds),
                    HetrogenousLists::WrapInList {
                        wrap_lists,
                    } => (true, wrap_lists),
                };

                let flattened = if into_list {
                    list.into_iter()
                        .map(|v| match v {
                            Tag::List(c) if !wrap_always => c,
                            other => TagList::singleton(other),
                        })
                        .collect()
                } else {
                    list.into_iter()
                        .map(|v| match v {
                            Tag::Compound(c) if !wrap_always => c,
                            other => tag_map! { "": other },
                        })
                        .collect()
                };

                return Ok(flattened);
            }
        }

        let mut flattened = TagList::new();

        for elem in list {
            // Safe to just push as we just verified all elements have the same type
            flattened.push(elem);
        }

        Ok(flattened)
    }

    fn commas(&mut self) -> ParseResult<bool, E> {
        let mut comma = false;
        loop {
            self.whitespace()?;
            diverge! {
                self,
                Comma => |_| comma = true,
                * => break Ok(comma),
                ! => break Ok(comma),
            }
        }
    }

    fn list_or_array(&mut self) -> ParseResult<Tag, E> {
        expect!(self, LBracket);

        let array_fmt = self.config.array_format;

        if array_fmt.allow_spaced_prefix {
            self.whitespace()?;
        }
        let (mut mode, numeric) = diverge_ahead! {
            self,
            Ident => |_| {
                consume!(self, val);

                // TODO this currently breaks lists if the first item is an identifier!

                let mode = match val.as_bytes() {
                    b"B" => Collector::Bytes(Vec::new()),
                    b"I" => Collector::Ints(Vec::new()),
                    b"L" => Collector::Longs(Vec::new()),
                    b"b" if array_fmt.allow_spaced_prefix => Collector::Bytes(Vec::new()),
                    b"i" if array_fmt.allow_spaced_prefix => Collector::Ints(Vec::new()),
                    b"l" if array_fmt.allow_spaced_prefix => Collector::Longs(Vec::new()),
                    _ => self.syntax_error(InvalidArrayType(val))?,
                };

                if array_fmt.allow_spaced_prefix {
                    self.whitespace()?;
                }
                expect!(self, Semicolon);

                (mode, true)
            },
            * => {
                (Collector::List(Vec::new()), false)
            }
        };

        let mut first = true;
        loop {
            self.whitespace()?;
            let v = diverge_ahead! {
                self,

                Comma if self.config.extra_commas == ExtraCommas::Anywhere => |_| {
                    self.commas()?;
                    continue
                },

                RBracket if first || self.config.extra_commas != ExtraCommas::Never => |_| {
                    consume!(self);
                    break;
                },

                Num if numeric => |_| {
                    consume!(self, val);
                    self.numeral(val)?
                },

                Bool if numeric && array_fmt.allow_booleans => |_| {
                    consume!(self, val);
                    self.numeral(val)?
                },

                Num | Str | Bool | Op | Ident | LBracket | LBrace if !numeric => |_| self.value()?,
            };

            first = false;
            mode.add_tag(v);

            let line_break = self.whitespace()?;
            diverge! {
                self,

                RBracket => |_| break,

                Comma if self.config.extra_commas == ExtraCommas::Anywhere => |_| {
                    self.commas()?;
                },

                Comma => |_| {},

                * if self.config.allow_line_break_separation && line_break => {}
            }
        }

        Ok(match mode {
            Collector::Bytes(v) => Tag::ByteArray(v),
            Collector::Ints(v) => Tag::IntArray(v),
            Collector::Longs(v) => Tag::LongArray(v),
            Collector::List(v) => Tag::List(self.flatten_hetrogenous_list(v)?),
        })
    }

    fn compound(&mut self) -> ParseResult<Tag, E> {
        expect!(self, LBrace);

        let mut map = TagMap::new();

        let mut first = true;
        loop {
            self.whitespace()?;
            let k = diverge! {
                self,

                Comma if self.config.extra_commas == ExtraCommas::Anywhere => |_| {
                    self.commas()?;
                    continue
                },

                RBrace if first || self.config.extra_commas != ExtraCommas::Never => |_| break,

                // Different tokens are accepted based on name mode
                Str if self.config.name_format == NameFormat::String => |val| val,
                Str | Ident if self.config.name_format == NameFormat::Identifier => |val| val,
                Str | Ident | Bool | Op if self.config.name_format == NameFormat::NonNumeral => |val| val,
                Num | Str | Bool | Op | Ident if self.config.name_format == NameFormat::AnyWord => |val| val,
            };

            self.whitespace()?;
            diverge! {
                self,
                Colon => |_| {},
                Equals if self.config.allow_equals_sign => |_| {},
            }

            let v = self.value()?;
            map.insert(k, v);

            first = false;

            let line_break = self.whitespace()?;
            diverge! {
                self,

                RBrace => |_| break,

                Comma if self.config.extra_commas == ExtraCommas::Anywhere => |_| {
                    self.commas()?;
                },

                Comma => |_| {},


                * if self.config.allow_line_break_separation && line_break => {}
            }
        }

        Ok(Tag::Compound(map))
    }

    fn primitive(&mut self) -> ParseResult<Tag, E> {
        diverge! {
            self,

            // "abc"
            Str => |val| Ok(Tag::String(val)),

            // 3.14
            Num => |val| Ok(self.numeral(val)?),

            // true, false
            Bool if self.config.integer_format.allow_booleans => |val| Ok(match val.as_bytes() {
                b"true" => Tag::Byte(1),
                b"false" => Tag::Byte(0),

                _ => unreachable!("Lexer should not emit boolean that is not 'true' or 'false'")
            }),

            // bool(...), uuid(...)
            Op if self.config.ops != NO_OPS => |val| Ok(match val.as_bytes() {
                b"bool" if self.config.ops.bool => {
                    self.whitespace()?;
                    expect!(self, LParen);
                    self.whitespace()?;
                    expect!(self, Num, val);
                    self.whitespace()?;
                    expect!(self, RParen);

                    match self.numeral(val)? {
                        Tag::Byte(0)
                        | Tag::Short(0)
                        | Tag::Int(0)
                        | Tag::Long(0)
                        | Tag::Float(0.0)
                        | Tag::Double(0.0) => Tag::Byte(0),

                        Tag::Byte(_)
                        | Tag::Short(_)
                        | Tag::Int(_)
                        | Tag::Long(_)
                        | Tag::Float(_)
                        | Tag::Double(_) => Tag::Byte(1),
                        _ => unreachable!()
                    }
                },

                b"uuid" if self.config.ops.uuid => {
                    self.whitespace()?;
                    expect!(self, LParen);

                    self.whitespace()?;
                    let val = if self.config.string_format.allow_identifiers {
                        expect!(self, Str | Ident, val);
                        val
                    } else {
                        expect!(self, Str, val);
                        val
                    };

                    self.whitespace()?;
                    expect!(self, RParen);

                    todo!() // TODO parse UUID
                },

                b"base64" if self.config.ops.base64 => {
                    self.whitespace()?;
                    expect!(self, LParen);

                    self.whitespace()?;
                    let val = if self.config.string_format.allow_identifiers {
                        expect!(self, Str | Ident, val);
                        val
                    } else {
                        expect!(self, Str, val);
                        val
                    };

                    self.whitespace()?;
                    expect!(self, RParen);

                    todo!() // TODO parse base64
                },

                b"pack" if self.config.ops.pack => {
                    todo!() // TODO
                },

                _ => unreachable!("lexer should not have emitted this")
            }),

            // abc, foo, etc
            Ident if self.config.string_format.allow_identifiers => |val| Ok(Tag::String(val))
        }
    }

    fn whitespace(&mut self) -> ParseResult<bool, E> {
        let mut line_break = false;
        loop {
            diverge! {
                self,
                Whitespace => |_| {},
                LineBreak => |_| line_break = true,
                Comment if self.config.comments != NO_COMMENTS => |_| line_break = true,
                * => break,
                ! => break
            }
        }

        Ok(line_break)
    }

    pub fn value(&mut self) -> ParseResult<Tag, E> {
        self.whitespace()?;

        diverge_ahead! {
            self,
            Str => |_| self.primitive(),
            Num => |_| self.primitive(),
            Bool if self.config.integer_format.allow_booleans => |_| self.primitive(),
            Op if self.config.ops != NO_OPS => |_| self.primitive(),
            Ident if self.config.string_format.allow_identifiers => |_| self.primitive(),
            LBracket => |_| self.list_or_array(),
            LBrace => |_| self.compound(),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let config = ParseConfig::ftb();

        let code = r#"
            # comment
            {
              foo: bar, baz: "gus"
              qux: [B; 1, 2, 3, 4]
            }
        "#;
        let lexer = Lexer::direct(code.chars(), &config);
        let mut parser = Parser::new(lexer, &config).unwrap();

        dbg!(parser.value().unwrap());
    }
}
