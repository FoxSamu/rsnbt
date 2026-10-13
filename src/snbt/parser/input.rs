use std::convert::Infallible;
use std::iter::Copied;
use std::slice::Iter;
use std::str::Chars;

use crate::snbt::ParseConfig;
use crate::snbt::parser::error::ParseResult;
use crate::snbt::parser::lex::Lexer;
use crate::snbt::parser::parse::Parser;
use crate::tag::Tag;

/// Parses the given input as SNBT and returns the parsed [`Tag`].
pub fn parse_chars<T, I, E>(input: T, config: &ParseConfig) -> ParseResult<Tag, E>
where
    T: IntoInput<I, E>,
    I: Iterator<Item = Result<char, E>>,
{
    let mut lexer = Lexer::new(input.into_input(), config)?;
    let mut parser = Parser::new(&mut lexer, config)?;
    parser.value()
}

pub trait IntoInput<I, E>
where
    I: Iterator<Item = Result<char, E>>,
{
    fn into_input(self) -> I;
}

/// A wrapper for infallible char inputs. These are for input sources that can produce the entire
/// char stream with no failures. A prime example is [`String`].
pub struct InfallibleIter<I>
where
    I: Iterator<Item = char>,
{
    chars: I,
}

// String-like -> IntoInput

impl<I> Iterator for InfallibleIter<I>
where
    I: Iterator<Item = char>,
{
    type Item = Result<char, Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(self.chars.next()?))
    }
}

impl<'a> IntoInput<InfallibleIter<Chars<'a>>, Infallible> for &'a String {
    fn into_input(self) -> InfallibleIter<Chars<'a>> {
        InfallibleIter { chars: self.chars() }
    }
}

impl<'a> IntoInput<InfallibleIter<Chars<'a>>, Infallible> for &'a str {
    fn into_input(self) -> InfallibleIter<Chars<'a>> {
        InfallibleIter { chars: self.chars() }
    }
}

impl<'a> IntoInput<InfallibleIter<Copied<Iter<'a, char>>>, Infallible> for &'a Vec<char> {
    fn into_input(self) -> InfallibleIter<Copied<Iter<'a, char>>> {
        InfallibleIter {
            chars: self.iter().copied(),
        }
    }
}

impl<'a> IntoInput<InfallibleIter<Copied<Iter<'a, char>>>, Infallible> for &'a [char] {
    fn into_input(self) -> InfallibleIter<Copied<Iter<'a, char>>> {
        InfallibleIter {
            chars: self.iter().copied(),
        }
    }
}

impl<'a> IntoInput<InfallibleIter<Copied<Iter<'a, char>>>, Infallible> for &'a mut [char] {
    fn into_input(self) -> InfallibleIter<Copied<Iter<'a, char>>> {
        InfallibleIter {
            chars: self.iter().copied(),
        }
    }
}

// Iterator<char> -> IntoInput

impl<'a, I> IntoInput<InfallibleIter<I>, Infallible> for I
where
    I: Iterator<Item = char>,
{
    fn into_input(self) -> InfallibleIter<I> {
        InfallibleIter { chars: self }
    }
}

// Iterator<Result<char, E>> -> IntoInput

impl<'a, I, E> IntoInput<I, E> for I
where
    I: Iterator<Item = Result<char, E>>,
{
    fn into_input(self) -> I {
        self
    }
}
