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

//! The Parser

use regex::Regex;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::{from_utf8_unchecked, FromStr};

use clval::{ClKey, ClVal, Date};
use error::{Error, ErrorKind};
use token::LexerToken;

#[derive(Default)]
/// The Parser
///
/// # Example
///
/// ```
/// extern crate clausewitz_parser;
///
/// use clausewitz_parser::Tokenizer;
/// use clausewitz_parser::Parser;
///
/// fn main() {
///     let tokenizer = Tokenizer::new(b"foo=bar");
///     let mut parser = Parser::new(tokenizer.tokenize());
///
///     let values = parser.parse().unwrap();
/// }
/// ```
pub struct Parser<'buf> {
    tokens: Vec<LexerToken<'buf>>,
    current_indent: u32,
    position: usize,
}

impl<'buf> Parser<'buf> {
    /// Construct a new `Parser`

    pub fn new(tokens: Vec<LexerToken<'buf>>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    /// Parse the provided [**LexerTokens**](../token/enum.LexerToken.html) into [**ClVals**](../clval/enum.ClVal.html)
    ///
    /// The returned `ClVal` is always a `Dict`
    pub fn parse(&mut self) -> Result<ClVal, Error> {
        let mut dict = HashMap::new();
        debug!("got {} tokens to parse", self.tokens.len());

        while self.position < self.tokens.len() {
            let key = self.parse_key()?;
            debug!("[parse] got key: {:?}", key);
            {
                let token = &self.tokens[self.position];
                debug!("[parse] next token: {:?}", token);
                // equals is optional for dicts
                if token.is_equals() {
                    self.position += 1;
                } else {
                    info!("expected equals, but found: {:?}", token);
                }
            }
            let value = self.parse_value()?;
            debug!("[parse] got value: {:?}", value);
            dict.insert(key, value);
        }

        Ok(ClVal::Dict(dict))
    }

    fn parse_key(&mut self) -> Result<ClKey, Error> {
        let token = &self.tokens[self.position];
        debug!("[key] pos: {} - token: {:?}", self.position, token);
        self.position += 1;
        let key = match token {
            // Quoted string:  QUOTE UNTYPED QUOTE
            LexerToken::Quote => {
                debug!("[key] quoted string pos: {} - token: {:?}", self.position, token);
                let token = &self.tokens[self.position];
                let s = self.parse_quoted_str(token.as_untyped()?);
                self.position += 2;
                debug!("[key] quoted string: {:?}", s);
                s
            }
            LexerToken::Untyped(b) => {
                debug!("[key] untyped");
                if let Ok(val) = self.parse_int(b) {
                    debug!("[key] int: {:?}", val);
                    return Ok(val);
                }
                if let Ok(val) = self.parse_date(b) {
                    debug!("[key] date: {:?}", val);
                    return Ok(val);
                }
                let val = self.parse_identifier(b);
                debug!("[key] identifier: {:?}", val);
                val
            }
            _ => bail!(ErrorKind::InvalidToken),
        };

        Ok(key)
    }

    fn parse_value(&mut self) -> Result<ClVal, Error> {
        let token = self.tokens[self.position].clone();
        debug!("[value] pos: {} - token: {:?}", self.position, token);
        self.position += 1;

        let value = match token {
            // Quoted string:  QUOTE UNTYPED QUOTE
            LexerToken::Quote => {
                let token = &self.tokens[self.position];
                debug!("[value] string token at {}: {:?}", self.position, token);
                let s = self.parse_quoted_str_v(token.as_untyped()?);
                self.position += 2;
                debug!("[value] quoted string: {:?}", s);
                s
            }
            LexerToken::Untyped(b) => {
                debug!("[value] untyped");
                if let Ok(val) = self.parse_int_v(b) {
                    debug!("[value] int: {:?}", val);
                    return Ok(val);
                }
                if let Ok(val) = self.parse_float(b) {
                    debug!("[value] float: {:?}", val);
                    return Ok(val);
                }
                if let Ok(val) = self.parse_bool(b) {
                    debug!("[value] bool: {:?}", val);
                    return Ok(val);
                }
                if let Ok(val) = self.parse_date_v(b) {
                    debug!("[value] date: {:?}", val);
                    return Ok(val);
                }
                let val = self.parse_identifier_v(b);
                debug!("[value] identifier: {:?}", val);
                val
            }
            // Lists / Dicts: LEFTCURLY [VALUE..][COMMA] RIGHTCURLY
            LexerToken::LeftCurly => {
                debug!("[value] collection");
                self.current_indent += 1;
                debug!("[value] indent now {}", self.current_indent);
                self.parse_collection()?
            }
            _ => bail!(ErrorKind::InvalidToken),
        };

        Ok(value)
    }

    fn parse_dict(&mut self) -> Result<ClVal, Error> {
        let mut dict = HashMap::new();

        while self.position < self.tokens.len() {
            let key = match self.parse_key() {
                Ok(key) => key,
                Err(e) => match e.kind() {
                    ErrorKind::InvalidToken => {
                        info!("[parse_dict] got an invalid token for key");
                        continue;
                    }
                    _ => bail!(e),
                },
            };
            debug!("[parse_dict] got key: {:?}", key);
            {
                let token = &self.tokens[self.position];
                debug!("[parse_dict] next token at {}: {:?}", self.position, token);
                // equals is optional for dicts
                if token.is_equals() {
                    self.position += 1;
                } else {
                    info!("expected equals, but found: {:?}", token);
                }
            }
            let value = match self.parse_value() {
                Ok(value) => value,
                Err(e) => match e.kind() {
                    ErrorKind::InvalidToken => {
                        info!("[parse_dict] got an invalid token for value: {:?}", self.tokens.get(self.position - 1));
                        continue;
                    }
                    _ => bail!(e),
                },
            };
            debug!("[parse_dict] got value: {:?}", value);
            dict.insert(key, value);
            if self.position >= self.tokens.len() {
                debug!("[parse_dict] reached EOF");
                break;
            }

            debug!("[parse_dict] next token at {}: {:?}", self.position, self.tokens[self.position]);
            // peek the next token
            match self.tokens[self.position] {
                // right curly -> end dict
                LexerToken::RightCurly => {
                    self.position += 1;
                    self.current_indent -= 1;
                    debug!("[parse_dict] got right curly -> return");
                    debug!("[parse_dict] indent now {}", self.current_indent);
                    break;
                }
                // Optional comma
                LexerToken::Comma => {
                    self.position += 1;
                }
                _ => {}
            }
        }

        Ok(ClVal::Dict(dict))
    }

    fn parse_list(&mut self, first: Option<ClVal>) -> Result<ClVal, Error> {
        let mut list = Vec::new();
        if let Some(first) = first {
            debug!("[parse_list] got first value: {:?}", first);
            list.push(first);
        }

        while self.position < self.tokens.len() {
            let value = match self.parse_value() {
                Ok(value) => value,
                Err(e) => match e.kind() {
                    ErrorKind::InvalidToken => {
                        info!("[parse_list] got an invalid token");
                        continue;
                    }
                    _ => bail!(e),
                },
            };
            debug!("[parse_list] got value: {:?}", value);
            list.push(value);
            if self.position >= self.tokens.len() {
                debug!("[parse_list] reached EOF");
                break;
            }

            debug!("[parse_list] next token at {}: {:?}", self.position, self.tokens[self.position]);
            // peek the next token
            match self.tokens[self.position] {
                // right curly -> end dict
                LexerToken::RightCurly => {
                    self.position += 1;
                    self.current_indent -= 1;
                    debug!("[parse_list] got right curly -> return");
                    debug!("[parse_list] indent now {}", self.current_indent);
                    break;
                }
                // optional comma
                LexerToken::Comma => {
                    self.position += 1;
                }
                _ => {}
            }
        }

        Ok(ClVal::List(list))
    }

    fn parse_collection(&mut self) -> Result<ClVal, Error> {
        // the first value for the list
        let first;
        let old_pos;
        // peek the next token and value to check if it's empty, a list or a dict
        let is_dict = {
            old_pos = self.position;
            debug!("[collection] next token: {:?}", self.tokens[self.position]);
            // check for empty collections
            if let LexerToken::RightCurly = self.tokens[self.position] {
                self.position += 1;
                return Ok(ClVal::List(Vec::new()));
            }
            // parse the next value
            let value = self.parse_value()?;
            debug!("[collection] next entry: {:?}", value);
            let token = &self.tokens[self.position];
            first = Some(value);

            debug!("[collection] next token: {:?}", token);
            // check if the next token after the value is an equals
            token.is_equals()
        };

        if is_dict {
            // reset the position for dicts, since the first parsed value must ne a ClKey
            // and we parsed a ClVal
            self.position = old_pos;
            debug!("[collection] dict");
            self.parse_dict()
        } else {
            debug!("[collection] list");
            self.parse_list(first)
        }
    }

    fn parse_identifier(&self, buf: &[u8]) -> ClKey {
        ClKey::Identifier(to_string(buf).to_string())
    }

    fn parse_identifier_v(&self, buf: &[u8]) -> ClVal {
        self.parse_identifier(buf).into()
    }

    fn parse_quoted_str(&self, buf: &[u8]) -> ClKey {
        ClKey::String(to_string(buf).to_string())
    }

    fn parse_quoted_str_v(&self, buf: &[u8]) -> ClVal {
        self.parse_quoted_str(buf).into()
    }

    fn parse_int(&self, buf: &[u8]) -> Result<ClKey, Error> {
        let int = buf_to_i32(buf)?;
        Ok(ClKey::Integer(int))
    }

    fn parse_int_v(&self, buf: &[u8]) -> Result<ClVal, Error> {
        self.parse_int(buf).map(|k| k.into())
    }

    fn parse_float(&self, buf: &[u8]) -> Result<ClVal, Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([+-]?)(\d*)\.(\d+)$").unwrap();
        }

        let caps = RE.captures(to_string(buf)).ok_or_else(|| "not a float")?;
        let sign = if &caps[1] == "-" { "-" } else { "" };
        let before_dot = &caps[2];
        let after_dot = &caps[3];

        let float = format!("{}{}.{}", sign, before_dot, after_dot).parse::<f32>()?;
        Ok(ClVal::Float(float))
    }

    fn parse_bool(&self, buf: &[u8]) -> Result<ClVal, Error> {
        match buf {
            b"yes" => Ok(ClVal::Bool(true)),
            b"no" => Ok(ClVal::Bool(false)),
            _ => bail!("not a bool"),
        }
    }

    fn parse_date(&self, buf: &[u8]) -> Result<ClKey, Error> {
        let s = to_string(buf);
        let date = Date::from_str(s)?;
        Ok(ClKey::Date(date))
    }

    fn parse_date_v(&self, buf: &[u8]) -> Result<ClVal, Error> {
        self.parse_date(buf).map(|k| k.into())
    }
}

fn to_string(b: &[u8]) -> &str {
    unsafe { from_utf8_unchecked(b) }
}
fn to_i32(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>()
}
fn buf_to_i32(s: &[u8]) -> Result<i32, ParseIntError> {
    to_i32(to_string(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::Tokenizer;

    fn untyped(buf: &[u8]) -> LexerToken {
        LexerToken::Untyped(buf)
    }

    fn equals() -> LexerToken<'static> {
        LexerToken::Equals
    }

    fn quote() -> LexerToken<'static> {
        LexerToken::Quote
    }

    fn comma() -> LexerToken<'static> {
        LexerToken::Comma
    }

    fn c_left() -> LexerToken<'static> {
        LexerToken::LeftCurly
    }

    fn c_right() -> LexerToken<'static> {
        LexerToken::RightCurly
    }

    fn key_id(k: &str) -> ClKey {
        ClKey::Identifier(k.to_string())
    }

    fn key_i(k: i32) -> ClKey {
        ClKey::Integer(k)
    }

    fn key_s(k: &str) -> ClKey {
        ClKey::String(k.to_string())
    }

    fn key_d(k: Date) -> ClKey {
        ClKey::Date(k)
    }

    fn val_id(v: &str) -> ClVal {
        ClVal::Identifier(v.to_string())
    }

    fn val_i(k: i32) -> ClVal {
        ClVal::Integer(k)
    }

    fn val_s(k: &str) -> ClVal {
        ClVal::String(k.to_string())
    }

    fn val_d(k: Date) -> ClVal {
        ClVal::Date(k)
    }

    fn val_f(k: f32) -> ClVal {
        ClVal::Float(k)
    }

    fn val_b(k: bool) -> ClVal {
        ClVal::Bool(k)
    }

    fn val_dict(v: HashMap<ClKey, ClVal>) -> ClVal {
        ClVal::Dict(v)
    }

    fn val_list(v: Vec<ClVal>) -> ClVal {
        ClVal::List(v)
    }

    #[test]
    fn test_parse() {
        let buf = include_bytes!("../examples/test");
        let tokenizer = Tokenizer::new(buf);
        let mut parser = Parser::new(tokenizer.tokenize());
        parser.parse().unwrap();
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = vec![untyped(b"key"), equals(), untyped(b"value")];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_id("key"), val_id("value"));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_string() {
        let tokens = vec![
            quote(),
            untyped(b"key"),
            quote(),
            equals(),
            quote(),
            untyped(b"value"),
            quote(),
        ];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_s("key"), val_s("value"));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_int() {
        let tokens = vec![untyped(b"12"), equals(), untyped(b"34")];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_i(12), val_i(34));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_date() {
        let tokens = vec![untyped(b"2018.5.16"), equals(), untyped(b"2018.05.17")];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_d(Date::new(2018, 5, 16)), val_d(Date::new(2018, 5, 17)));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_float() {
        let tokens = vec![untyped(b"key"), equals(), untyped(b"12.34")];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_id("key"), val_f(12.34));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_bool() {
        let tokens = vec![untyped(b"key"), equals(), untyped(b"yes")];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_id("key"), val_b(true));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_list() {
        let tokens = vec![
            untyped(b"key"),
            equals(),
            c_left(),
            untyped(b"1"),
            untyped(b"2"),
            comma(),
            untyped(b"3"),
            c_right(),
        ];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        dict.insert(key_id("key"), val_list(vec![val_i(1), val_i(2), val_i(3)]));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }

    #[test]
    fn test_parse_dict() {
        let tokens = vec![
            untyped(b"key"),
            equals(),
            c_left(),
            untyped(b"key1"),
            equals(),
            untyped(b"val1"),
            comma(),
            untyped(b"key2"),
            equals(),
            untyped(b"val2"),
            c_right(),
        ];
        let mut parser = Parser::new(tokens);
        let mut dict = HashMap::new();
        let mut dict2 = HashMap::new();
        dict2.insert(key_id("key1"), val_id("val1"));
        dict2.insert(key_id("key2"), val_id("val2"));
        dict.insert(key_id("key"), val_dict(dict2));
        assert_eq!(parser.parse().unwrap(), val_dict(dict));
    }
}
