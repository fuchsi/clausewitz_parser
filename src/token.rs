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

//! The Tokenizer

use error::{Error, ErrorKind};
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, Clone)]
/// The lexer tokens
pub enum LexerToken<'buf> {
    Equals,
    Quote,
    LeftCurly,
    RightCurly,
    LeftParanthesis,
    RightParanthesis,
    Comment,
    Comma,
    Untyped(&'buf [u8]),
}

impl<'buf> LexerToken<'buf> {
    pub fn as_untyped(&self) -> Result<&[u8], Error> {
        if let LexerToken::Untyped(buf) = self {
            Ok(buf)
        } else {
            bail!("not an untyped token")
        }
    }

    pub fn is_equals(&self) -> bool {
        if let LexerToken::Equals = self {
            true
        } else {
            false
        }
    }

    pub fn is_left_curly(&self) -> bool {
        if let LexerToken::LeftCurly = self {
            true
        } else {
            false
        }
    }

    pub fn is_right_curly(&self) -> bool {
        if let LexerToken::RightCurly = self {
            true
        } else {
            false
        }
    }
}

impl<'buf> TryFrom<&'buf u8> for LexerToken<'buf> {
    type Error = Error;

    fn try_from(chr: &u8) -> Result<Self, Error> {
        match *chr {
            b'=' => Ok(LexerToken::Equals),
            b'"' => Ok(LexerToken::Quote),
            b'{' => Ok(LexerToken::LeftCurly),
            b'}' => Ok(LexerToken::RightCurly),
            b'(' => Ok(LexerToken::LeftParanthesis),
            b')' => Ok(LexerToken::RightParanthesis),
            b'#' => Ok(LexerToken::Comment),
            b',' => Ok(LexerToken::Comma),
            _ => bail!(ErrorKind::NoToken),
        }
    }
}

fn is_whitespace(chr: &u8) -> bool {
    match *chr {
        b' ' => true,
        b'\n' => true,
        b'\r' => true,
        b'\t' => true,
        _ => false,
    }
}

/// The tokenizer
pub struct Tokenizer<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Tokenizer<'buf> {
    /// Constructs a new `Tokenizer`
    pub fn new(buf: &'buf [u8]) -> Self {
        Self { buf }
    }

    /// Tokenize the provided buffer
    pub fn tokenize(&self) -> Vec<LexerToken> {
        let mut untyped_start = None;
        let mut in_quote = false;
        let mut in_comment = false;
        let mut tokens = Vec::with_capacity(4096);

        for (pos, chr) in self.buf.iter().enumerate() {
            // if in a comment, advance until newline
            if in_comment {
                if chr == &b'\n' {
                    in_comment = false;
                }
                continue;
            }
            // Read a character and test to see if it is a token.
            let token = LexerToken::try_from(chr);
            match token {
                Ok(t) => {
                    if in_quote {
                        // If token is a quote, advance until closing quote
                        if let LexerToken::Quote = t {
                            debug!("got new token: {:?}", t);
                            if untyped_start.is_some() && pos != 0 {
                                debug!(
                                    "push untyped to list: {}",
                                    String::from_utf8_lossy(&self.buf[untyped_start.unwrap()..pos])
                                );
                                let untyped = LexerToken::Untyped(&self.buf[untyped_start.take().unwrap()..pos]);
                                tokens.push(untyped);
                            } else {
                                // push an empty string
                                tokens.push(LexerToken::Untyped(b""));
                            }
                        } else {
                            continue;
                        }
                    } else {
                        debug!("got new token: {:?}", t);
                        // got a new token, push the last untyped to the list
                        if untyped_start.is_some() && pos != 0 {
                            debug!(
                                "push untyped to list: {}",
                                String::from_utf8_lossy(&self.buf[untyped_start.unwrap()..pos])
                            );
                            let untyped = LexerToken::Untyped(&self.buf[untyped_start.take().unwrap()..pos]);
                            tokens.push(untyped);
                        }
                    }

                    if let LexerToken::Quote = t {
                        in_quote = !in_quote;
                        debug!("in quote now: {}", in_quote);
                    } else if let LexerToken::Comment = t {
                        in_comment = true;
                    }
                    tokens.push(t)
                }
                Err(_) => {
                    // ignore every whitespace as long as we're not in a quoted string
                    if !in_quote && is_whitespace(chr) {
                        debug!("got whitespace");
                        if untyped_start.is_some() {
                            debug!(
                                "push untyped to list: {}",
                                String::from_utf8_lossy(&self.buf[untyped_start.unwrap()..pos])
                            );
                            let untyped = LexerToken::Untyped(&self.buf[untyped_start.take().unwrap()..pos]);
                            tokens.push(untyped);
                        }
                    } else if untyped_start.is_none() {
                        // All characters until whitespace or a token is considered untyped
                        untyped_start = Some(pos);
                    }
                }
            }
        }

        // End of Input. If the last token is untyped append the remaining bytes
        if untyped_start.is_some() {
            debug!(
                "EOF. Push remaining untyped: {}",
                String::from_utf8_lossy(&self.buf[untyped_start.unwrap()..])
            );
            let untyped = LexerToken::Untyped(&self.buf[untyped_start.take().unwrap()..]);
            tokens.push(untyped);
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_whitespace() {
        assert!(is_whitespace(&b' '));
        assert!(is_whitespace(&b'\t'));
        assert!(is_whitespace(&b'\r'));
        assert!(is_whitespace(&b'\n'));
        for chr in b'a'..b'z' {
            assert!(!is_whitespace(&chr));
        }
        for chr in b'A'..b'Z' {
            assert!(!is_whitespace(&chr));
        }
        for chr in b'0'..b'9' {
            assert!(!is_whitespace(&chr));
        }
    }

    #[test]
    fn test_lexer_tokens() {
        assert_eq!(LexerToken::try_from(&b'=').unwrap(), LexerToken::Equals);
        assert_eq!(LexerToken::try_from(&b'{').unwrap(), LexerToken::LeftCurly);
        assert_eq!(LexerToken::try_from(&b'}').unwrap(), LexerToken::RightCurly);
        assert_eq!(LexerToken::try_from(&b'(').unwrap(), LexerToken::LeftParanthesis);
        assert_eq!(LexerToken::try_from(&b')').unwrap(), LexerToken::RightParanthesis);
        assert_eq!(LexerToken::try_from(&b'"').unwrap(), LexerToken::Quote);
        assert_eq!(LexerToken::try_from(&b'#').unwrap(), LexerToken::Comment);
        assert_eq!(LexerToken::try_from(&b',').unwrap(), LexerToken::Comma);
        assert_eq!(LexerToken::try_from(&b'z').unwrap_err().to_string(), "not a token");
    }

    #[test]
    fn test_tokenizer() {
        let buf = b"date=1597.1.1";
        let tokenizer = Tokenizer::new(buf);
        assert_eq!(
            tokenizer.tokenize(),
            vec![
                LexerToken::Untyped(b"date"),
                LexerToken::Equals,
                LexerToken::Untyped(b"1597.1.1"),
            ]
        );

        let buf = b"player = \"AAA\"";
        let tokenizer = Tokenizer::new(buf);
        assert_eq!(
            tokenizer.tokenize(),
            vec![
                LexerToken::Untyped(b"player"),
                LexerToken::Equals,
                LexerToken::Quote,
                LexerToken::Untyped(b"AAA"),
                LexerToken::Quote,
            ]
        );

        let buf = b"player = \"AAA\"";
        let tokenizer = Tokenizer::new(buf);
        assert_eq!(
            tokenizer.tokenize(),
            vec![
                LexerToken::Untyped(b"player"),
                LexerToken::Equals,
                LexerToken::Quote,
                LexerToken::Untyped(b"AAA"),
                LexerToken::Quote,
            ]
        );

        let buf = b"save_game=\"autosave.eu4\"";
        let tokenizer = Tokenizer::new(buf);
        assert_eq!(
            tokenizer.tokenize(),
            vec![
                LexerToken::Untyped(b"save_game"),
                LexerToken::Equals,
                LexerToken::Quote,
                LexerToken::Untyped(b"autosave.eu4"),
                LexerToken::Quote,
            ]
        );

        let buf = b"dlc=\"Rights of Man\"";
        let tokenizer = Tokenizer::new(buf);
        assert_eq!(
            tokenizer.tokenize(),
            vec![
                LexerToken::Untyped(b"dlc"),
                LexerToken::Equals,
                LexerToken::Quote,
                LexerToken::Untyped(b"Rights of Man"),
                LexerToken::Quote,
            ]
        );
    }
}
