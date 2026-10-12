use std::convert::Infallible;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::result::Result as StdResult;

pub use ParseError::*;
pub use SyntaxError::*;
pub use TokenKind::*;

use super::config::ParseConfig;
use super::error::ParseError;
use super::error::ParseResult;
use super::error::SyntaxError;
use crate::snbt::NonfiniteFloats;
use crate::snbt::parser::ring_buf::RingBuf;

const LEMNISCATE_C: char = '\u{221E}';
const LEMNISCATE: &[u8] = b"\x22\x1E";
const LEMNISCATE_F: &[u8] = b"\x22\x1Ef";
const LEMNISCATE_FF: &[u8] = b"\x22\x1EF";
const LEMNISCATE_D: &[u8] = b"\x22\x1Ed";
const LEMNISCATE_DD: &[u8] = b"\x22\x1ED";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Num,
    Bool,
    Ident,
    Str,
    Op,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Equals,
    Semicolon,
    Whitespace,
    LineBreak,
    Comment,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Num => write!(f, "numeric"),
            Bool => write!(f, "boolean"),
            Ident => write!(f, "identifier"),
            Str => write!(f, "string"),
            Op => write!(f, "operator"),
            LParen => write!(f, "'('"),
            RParen => write!(f, "')'"),
            LBracket => write!(f, "'['"),
            RBracket => write!(f, "']'"),
            LBrace => write!(f, "'{{'"),
            RBrace => write!(f, "'}}'"),
            Comma => write!(f, "','"),
            Colon => write!(f, "':'"),
            Equals => write!(f, "'='"),
            Semicolon => write!(f, "';'"),
            Whitespace => write!(f, "whitespace"),
            LineBreak => write!(f, "line break"),
            Comment => write!(f, "comment"),
        }
    }
}

impl Debug for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub index: u64,
    pub kind:  TokenKind,
    pub value: String,
}

pub trait Decompose<'a, K> {
    fn decompose(&'a self) -> K;
}

impl<'a> Decompose<'a, (TokenKind, &'a String)> for Token {
    fn decompose(&'a self) -> (TokenKind, &'a String) {
        (self.kind, &self.value)
    }
}

impl<'a> Decompose<'a, Option<(TokenKind, &'a String)>> for Option<Token> {
    fn decompose(&'a self) -> Option<(TokenKind, &'a String)> {
        match self {
            Some(v) => Some((v.kind, &v.value)),
            None => None,
        }
    }
}

pub trait Symbol {
    type Error;

    fn result(self) -> StdResult<char, Self::Error>;
}

impl Symbol for char {
    type Error = Infallible;

    fn result(self) -> StdResult<char, Self::Error> {
        Ok(self)
    }
}

impl<E> Symbol for StdResult<char, E> {
    type Error = E;

    fn result(self) -> StdResult<char, E> {
        self
    }
}


enum Escape {
    Ignore,
    Char(u16),
    SurrogatePair(u16, u16),
}


pub struct Lexer<'a, I, S>
where
    I: Iterator<Item = S>,
    S: Symbol,
{
    input:  I,
    next:   Option<char>,
    index:  u64,
    config: &'a ParseConfig,
}

impl<'a, I> Lexer<'a, I, char>
where
    I: Iterator<Item = char>,
{
    pub fn direct(mut input: I, config: &'a ParseConfig) -> Self {
        let next = input.next();

        Self {
            input,
            next,
            index: 0,
            config,
        }
    }

    #[cfg(test)]
    pub fn all(&mut self) -> ParseResult<Vec<Token>, Infallible> {
        let mut vec = Vec::new();

        while let Some(e) = self.next_token()? {
            vec.push(e);
        }

        Ok(vec)
    }
}

impl<'a, I, E> Lexer<'a, I, StdResult<char, E>>
where
    I: Iterator<Item = StdResult<char, E>>,
{
    pub fn new(mut input: I, config: &'a ParseConfig) -> ParseResult<Self, E> {
        let next = input.next().transpose().map_err(|e| Io(e))?;

        Ok(Self {
            input,
            next,
            index: 0,
            config,
        })
    }

    #[cfg(test)]
    pub fn all(&mut self) -> ParseResult<Vec<Token>, E> {
        let mut vec = Vec::new();

        while let Some(e) = self.next_token()? {
            vec.push(e);
        }

        Ok(vec)
    }
}

impl<I, E, S> Lexer<'_, I, S>
where
    I: Iterator<Item = S>,
    S: Symbol<Error = E>,
{
    fn advance(&mut self) -> ParseResult<bool, E> {
        self.next = match self.input.next().map(|v| v.result()) {
            Some(Ok(c)) => {
                if self.index == u64::MAX {
                    return Err(SizeLimit);
                }

                self.index += 1;

                Some(c)
            },

            Some(Err(e)) => return Err(Io(e)),

            None => None,
        };

        Ok(true)
    }

    fn symbol(&mut self, token: TokenKind) -> ParseResult<(TokenKind, String), E> {
        let Some(c) = self.next else {
            return self.syntax_error(ExpectedToken(vec![token], false, None));
        };

        self.advance()?;
        Ok((token, String::from(c)))
    }

    fn whitespace(&mut self) -> ParseResult<(TokenKind, String), E> {
        let mut space = String::new();

        let mut line_break = false;
        while let Some(c @ (' ' | '\n' | '\r' | '\t')) = self.next {
            if c == '\n' || c == '\r' {
                line_break = true;
            }

            space.push(c);

            self.advance()?;
        }

        Ok((
            if line_break {
                LineBreak
            } else {
                Whitespace
            },
            space,
        ))
    }

    /// Reads a word.
    ///
    /// Words identify numerals, identifiers and other special keywords, since they are all
    /// essentially some sequence of the same character group: `[-+._a-zA-Z0-9]`. Additionally, if
    /// the lemniscate symbol (`∞`) is allowed by the [`ParseConfig`] to be used for infinity
    /// numerals, it may also be used anywhere in identifiers.
    ///
    /// Produces:
    /// - [`Op`] when the word is a valid operators like `bool` or `uuid`, and operators are enabled
    ///   in the [`ParseConfig`];
    /// - [`Num`] when the word is an infinity- or NaN-like symbol, depending on what and what not
    ///   is enabled in the [`ParseConfig`];
    /// - [`Bool`] when the word is `true` or `false`, and boolean literals are enabled by the
    ///   [`ParseConfig`];
    /// - [`Num`] when the word starts with a character of the group `[-+.0-9]`;
    /// - [`Ident`] in any other case.
    fn word(&mut self) -> ParseResult<(TokenKind, String), E> {
        let mut ident = String::new();

        let (inf, nan, infinity, big_nan, lemniscate) = match self.config.nonfinite_floats {
            NonfiniteFloats::Disabled {
                ..
            } => (false, false, false, false, false),

            NonfiniteFloats::InfinityOnly {
                js_like,
                rust_like,
                lemniscate,
                ..
            } => (rust_like, false, js_like, false, lemniscate),

            NonfiniteFloats::Enabled {
                js_like,
                rust_like,
                lemniscate,
                ..
            } => (rust_like, rust_like, js_like, js_like, lemniscate),
        };

        while let Some(c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '+' | LEMNISCATE_C)) = self.next {
            // Only allow lemniscates in words if needed for numerals
            if c == LEMNISCATE_C && !lemniscate {
                break;
            }

            ident.push(c);
            self.advance()?;
        }


        // match as byte slice for 2 reasons:
        // - all characters are ascii so why bother with utf-8?
        // - we can destructure the slice
        match ident.as_bytes() {
            // Ops
            b"bool" if self.config.ops.bool => Ok((Op, ident)),
            b"uuid" if self.config.ops.uuid => Ok((Op, ident)),
            b"base64" if self.config.ops.base64 => Ok((Op, ident)),
            b"pack" if self.config.ops.pack => Ok((Op, ident)),

            // Infinity, NaN
            b"inf" | b"infF" | b"infD" | b"inff" | b"infd" if inf => Ok((Num, ident)),
            b"nan" | b"nanF" | b"nanD" | b"nanf" | b"nand" if nan => Ok((Num, ident)),
            b"Infinity" | b"InfinityF" | b"InfinityD" | b"Infinityf" | b"Infinityd" if infinity => Ok((Num, ident)),
            b"NaN" | b"NaNF" | b"NaND" | b"NaNf" | b"NaNd" if big_nan => Ok((Num, ident)),
            LEMNISCATE | LEMNISCATE_F | LEMNISCATE_FF | LEMNISCATE_D | LEMNISCATE_DD if lemniscate => Ok((Num, ident)),

            // Booleans
            b"true" | b"false" if self.config.integer_format.allow_booleans => Ok((Bool, ident)),

            // Numerals
            &[b'0'..=b'9' | b'-' | b'.' | b'+', ..] => Ok((Num, ident)),

            // Other identifiers
            _ => Ok((Ident, ident)),
        }
    }

    fn syntax_error<R>(&self, error: SyntaxError) -> ParseResult<R, E> {
        Err(Syntax(self.index, error))
    }

    fn utf8_to_utf16(&self, c: char) -> ParseResult<Escape, E> {
        let mut utf16 = [0u16; 2];

        Ok(match c.encode_utf16(&mut utf16) {
            [a] => Escape::Char(*a),
            [a, b] => Escape::SurrogatePair(*a, *b),
            _ => unreachable!(), // length is always 1 or 2
        })
    }

    fn utf32_to_utf16(&self, c: u32) -> ParseResult<Escape, E> {
        let Some(c) = char::from_u32(c) else {
            return self.syntax_error(InvalidCodepoint(c));
        };

        self.utf8_to_utf16(c)
    }

    #[inline]
    fn esc_char(&mut self, c: char) -> ParseResult<Escape, E> {
        self.advance()?;
        self.utf8_to_utf16(c)
    }

    fn esc_hex<const N: usize>(&mut self) -> ParseResult<u32, E> {
        self.advance()?;
        let mut c = 0u32;

        for _ in 0..N {
            match self.next {
                Some(d @ ('0'..='9')) => {
                    c <<= 4;
                    c |= d as u32 - '0' as u32;
                },
                Some(d @ ('a'..='f')) => {
                    c <<= 4;
                    c |= d as u32 - 'a' as u32 + 10;
                },
                Some(d @ ('A'..='F')) => {
                    c <<= 4;
                    c |= d as u32 - 'A' as u32 + 10;
                },
                _ => return self.syntax_error(IncompleteEscapeSequence),
            };
        }

        Ok(c)
    }

    fn esc_ascii(&mut self) -> ParseResult<Escape, E> {
        let c = self.esc_hex::<2>()?;
        Ok(Escape::Char(c as u16))
    }

    fn esc_utf16(&mut self) -> ParseResult<Escape, E> {
        let c = self.esc_hex::<4>()?;
        Ok(Escape::Char(c as u16))
    }

    fn esc_utf32(&mut self) -> ParseResult<Escape, E> {
        let c = self.esc_hex::<8>()?;
        if c <= 0xFFFF {
            // this is here so that one can form surrogate pairs as `\U0000D811\U0000DD35` even
            // though there is no point in doing that.
            Ok(Escape::Char(c as u16))
        } else {
            self.utf32_to_utf16(c)
        }
    }

    fn esc_break(&mut self) -> ParseResult<Escape, E> {
        while let Some(' ' | '\t') = self.next {
            self.advance()?;
        }

        let Some('\n' | '\r') = self.next else {
            return self.syntax_error(IncompleteEscapeSequence);
        };

        while let Some('\n' | '\r') = self.next {
            self.advance()?;
        }

        Ok(Escape::Ignore)
    }

    fn esc_name(&mut self) -> ParseResult<Escape, E> {
        self.advance()?;
        match self.next {
            Some('{') => self.advance()?,
            Some(c) => self.syntax_error(UnexpectedCharacter(c))?,
            None => self.syntax_error(UnexpectedEof)?,
        };

        let mut name = Vec::new();

        while let Some(c) = self.next &&
            c != '}'
        {
            name.push(c);
            self.advance()?;
        }

        match self.next {
            Some('}') => self.advance()?,
            Some(c) => self.syntax_error(UnexpectedCharacter(c))?,
            None => self.syntax_error(UnexpectedEof)?,
        };

        let name = String::from_iter(name);

        match unicode_names::character(name.as_str()) {
            Some(c) => self.utf8_to_utf16(c),
            None => self.syntax_error(NoSuchCodepoint(name)),
        }
    }

    fn string_esc(&mut self) -> ParseResult<Escape, E> {
        let escs = self.config.string_format;

        match self.next {
            Some('0') if escs.esc_nul => self.esc_char('\0'),
            Some('n') if escs.esc_lf => self.esc_char('\n'),
            Some('r') if escs.esc_cr => self.esc_char('\r'),
            Some('b') if escs.esc_bs => self.esc_char('\u{08}'),
            Some('f') if escs.esc_ff => self.esc_char('\u{0C}'),
            Some('s') if escs.esc_space => self.esc_char(' '),
            Some('t') if escs.esc_tab => self.esc_char('\t'),
            Some('\'') if escs.esc_single_quote => self.esc_char('\''),
            Some('"') if escs.esc_double_quote => self.esc_char('"'),
            Some('\\') if escs.esc_backslash => self.esc_char('\\'),
            Some('/') if escs.esc_slash => self.esc_char('/'),
            Some('x') if escs.esc_unicode_8 => self.esc_ascii(),
            Some('u') if escs.esc_unicode_16 => self.esc_utf16(),
            Some('U') if escs.esc_unicode_32 => self.esc_utf32(),
            Some('N') if escs.esc_unicode_name => self.esc_name(),
            Some(' ' | '\t' | '\r' | '\n') if escs.esc_unicode_name => self.esc_break(),
            Some(c) if escs.esc_chars => self.esc_char(c),
            _ if escs.ignnore_invalid_escapes => Ok(Escape::Char('\\' as u16)),

            _ => self.syntax_error(IncompleteEscapeSequence),
        }
    }

    fn string(&mut self, quote: char) -> ParseResult<(TokenKind, String), E> {
        let mut value = Vec::new();

        // track the code address of each character so we can report errors on location
        let mut addresses = Vec::new();

        // open quote was already identified
        self.advance()?;

        while let Some(c) = self.next {
            let addr = self.index;
            self.advance()?;

            let esc = if c == quote {
                break;
            } else if c == '\\' {
                self.string_esc()?
            } else {
                self.utf8_to_utf16(c)?
            };

            match esc {
                Escape::Ignore => {},

                Escape::Char(c) => {
                    value.push(c);
                    addresses.push(addr);
                },

                Escape::SurrogatePair(c1, c2) => {
                    value.push(c1);
                    value.push(c2);

                    addresses.push(addr);
                    addresses.push(addr);
                },
            }
        }

        // manual conversion from utf-16 to utf-8 as this will allow us to carefully report
        // errors on location

        // any utf-16 is at most 3 utf-8s
        let mut out = String::with_capacity(value.len() * 3);

        let mut i = 0;
        while i < value.len() {
            let unit = value[i];
            let index = addresses[i];

            let cp = match unit {
                // high surrogate -> expect low surrogate
                0xD800..=0xDBFF => {
                    let Some(&low @ 0xDC00..=0xDFFF) = value.get(i + 1) else {
                        // not a low surrogate
                        return Err(Syntax(index, DanglingSurrogate(unit)));
                    };

                    let high = unit as u32;
                    let low = low as u32;

                    i += 2;
                    0x10000 + (((high - 0xD800) << 10) | (low - 0xDC00))
                },

                // low surrogate -> error
                0xDC00..=0xDFFF => return Err(Syntax(index, DanglingSurrogate(unit))),

                // basic multilingual plane
                _ => {
                    i += 1;
                    unit as u32
                },
            };

            out.push(unsafe {
                // we did the checks ourselves, this is safe
                char::from_u32_unchecked(cp)
            });
        }

        Ok((Str, out))
    }

    fn c_comment(&mut self) -> ParseResult<(TokenKind, String), E> {
        let mut value = String::new();

        // skip /
        self.advance()?;

        match self.next {
            Some('/') => {
                self.advance()?;
                while let Some(c) = self.next &&
                    !matches!(c, '\r' | '\n')
                {
                    value.push(c);
                    self.advance()?;
                }
            },
            Some('*') => {
                self.advance()?;

                // Lookbehind buffer in which we check if we saw a */
                let mut lb = RingBuf::new(2);

                while let Some(c) = self.next &&
                    !matches!(lb.as_slice(), b"*/")
                {
                    value.push(c);

                    if let Ok(ascii) = c.try_into() {
                        lb.put(ascii);
                    } else {
                        lb.put(b' ');
                    }

                    self.advance()?;
                }

                if !matches!(lb.as_slice(), b"*/") {
                    self.syntax_error(UnexpectedEof)?;
                }

                // Remove the */ bit
                value.truncate(value.len() - 2);
            },
            Some(c) => self.syntax_error(UnexpectedCharacter(c))?,
            None => self.syntax_error(UnexpectedEof)?,
        }


        Ok((Comment, value))
    }

    fn hash_comment(&mut self) -> ParseResult<(TokenKind, String), E> {
        let mut value = String::new();

        // skip #
        self.advance()?;

        while let Some(c) = self.next &&
            !matches!(c, '\r' | '\n')
        {
            value.push(c);
            self.advance()?;
        }

        Ok((Comment, value))
    }

    fn tilde_comment(&mut self) -> ParseResult<(TokenKind, String), E> {
        let mut value = String::new();

        // skip ~
        self.advance()?;

        let mut open_tildes = 1usize;
        while let Some('~') = self.next {
            open_tildes += 1;
            self.advance()?;
        }

        if open_tildes == 1 {
            match self.next {
                Some(c) => self.syntax_error(UnexpectedCharacter(c))?,
                None => self.syntax_error(UnexpectedEof)?,
            }
        }

        // Lookbehind of how much consecutive tildes we saw
        let mut close_tildes = 0;

        while let Some(c) = self.next &&
            (!matches!(c, '\r' | '\n') || open_tildes > 2)
        {
            value.push(c);

            if c == '~' {
                close_tildes += 1;
            } else {
                close_tildes = 0;
            }

            self.advance()?;

            if close_tildes == open_tildes {}
        }

        // Remove the closing tildes
        if close_tildes == open_tildes {
            value.truncate(value.len() - open_tildes);
        }

        Ok((Comment, value))
    }

    fn next_token(&mut self) -> ParseResult<Option<Token>, E> {
        let index = self.index;

        let value = match self.next {
            None => None,

            Some(char) => Some(match char {
                ':' => self.symbol(Colon)?,
                ',' => self.symbol(Comma)?,
                ';' => self.symbol(Semicolon)?,
                '(' => self.symbol(LParen)?,
                ')' => self.symbol(RParen)?,
                '[' => self.symbol(LBracket)?,
                ']' => self.symbol(RBracket)?,
                '{' => self.symbol(LBrace)?,
                '}' => self.symbol(RBrace)?,
                '=' => self.symbol(RBrace)?,

                ' ' | '\n' | '\r' | '\t' => self.whitespace()?,
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '+' | '-' | '.' => self.word()?,
                '\'' if self.config.string_format.single_quotes => self.string('\'')?,
                '\"' if self.config.string_format.double_quotes => self.string('\"')?,

                '#' if self.config.comments.hash => self.hash_comment()?,
                '~' if self.config.comments.tildes => self.tilde_comment()?,
                '/' if self.config.comments.c_style => self.c_comment()?,

                v => self.syntax_error(UnexpectedCharacter(v))?,
            }),
        };

        Ok(value.map(|(kind, value)| Token {
            index,
            kind,
            value,
        }))
    }
}


impl<I, E, S> Iterator for Lexer<'_, I, S>
where
    I: Iterator<Item = S>,
    S: Symbol<Error = E>,
{
    type Item = ParseResult<Token, E>;

    fn next(&mut self) -> Option<ParseResult<Token, E>> {
        self.next_token().transpose()
    }
}


#[cfg(test)]
mod test {
    use crate::snbt::parser::lex::Lexer;

    #[test]
    fn test() {
        let input = r##"
            {
                # comment
                foo: "bar",
                baz: "gus",
                gus: baz,
                a: [B; 3, 2, 1]
            }
        "##;

        // let mut direct = Lexer::direct(input.chars());
        // let tokens = direct.all().unwrap();
        // dbg!(tokens);
    }
}
