use std::convert::Infallible;
use std::str::Chars;

mod lex;
mod parse;
mod numeral;
mod ring_buf;

pub mod error;
pub mod config;
pub mod input;
