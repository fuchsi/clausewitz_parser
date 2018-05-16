/*
 * clausewitz_parser, a Clausewitz file parser
 * Copyright (C) 2018 Daniel Müller
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
#![feature(try_from)]

#[macro_use]
extern crate error_chain;
extern crate regex;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

pub use clval::{ClKey, ClVal};
pub use error::{Error, ErrorKind};
pub use parser::Parser;
pub use token::{LexerToken, Tokenizer};

pub mod clval;
mod error;
pub mod parser;
pub mod token;

/// Parse a buffer of bytes into [**ClVals**](clval/enum.ClVal.html)
///
/// The returned `ClVal` is always a `Dict`
///
/// # Example
/// ```
/// extern crate clausewitz_parser;
///
/// use clausewitz_parser::parse;
///
/// fn main() {
///     let values = parse(b"foo=bar").unwrap();
/// }
/// ```
pub fn parse(buf: &[u8]) -> Result<ClVal, Error> {
    let tokenizer = Tokenizer::new(buf);
    let mut parser = Parser::new(tokenizer.tokenize());
    parser.parse()
}
