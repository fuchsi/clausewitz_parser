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

//! Clausewitz Values
//!
//! The Clausewitz savegame format is at its core a fairly simple context-free grammar.
//! It may be described by the following ABNF specification:
//!
//!     /*
//!     key            = string / integer / date / identifier
//!     value          = string / integer / float / date / identifier / boolean
//!     key-value-pair = key equals (value / group)
//!     group          = open-group *((key-value-pair / value / group) [comma]) close-group
//!
//!     document       = [magic-number] *(key-value-pair [comma])
//!     */
//!

use error::*;
use std::collections::HashMap;
use std::str::FromStr;
use regex::Regex;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Date type
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// Construct a new `Date`
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self{year, month, day}
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static!{
            static ref RE: Regex = Regex::new(r"^(\d+)\.(\d{1,2})\.(\d{1,2})$").unwrap();
        };

        let caps = RE.captures(s).ok_or_else(|| "not a date")?;
        let year = caps[1].parse::<i32>()?;
        let month = caps[2].parse::<u8>()?;
        let day = caps[3].parse::<u8>()?;
        Ok(Self::new(year, month, day))
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "{}.{}.{}", self.year, self.month, self.day)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
/// Key types
pub enum ClKey {
    Integer(i32),
    String(String),
    Date(Date),
    Identifier(String),
}

impl ClKey {
    pub fn as_i32(&self) -> Result<&i32> {
        if let ClKey::Integer(ref int) = self {
            Ok(int)
        } else {
            bail!(ErrorKind::InvalidValue("integer".to_string()))
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        if let ClKey::String(ref string) = self {
            Ok(string)
        } else {
            bail!(ErrorKind::InvalidValue("string".to_string()))
        }
    }

    pub fn as_identifier(&self) -> Result<&str> {
        if let ClKey::Identifier(ref string) = self {
            Ok(string)
        } else {
            bail!(ErrorKind::InvalidValue("identifier".to_string()))
        }
    }

    pub fn as_date(&self) -> Result<&Date> {
        if let ClKey::Date(ref date) = self {
            Ok(date)
        } else {
            bail!(ErrorKind::InvalidValue("identifier".to_string()))
        }
    }
}

impl Into<ClVal> for ClKey {
    fn into(self) -> ClVal {
        match self {
            ClKey::Integer(i) => ClVal::Integer(i),
            ClKey::String(s) => ClVal::String(s),
            ClKey::Date(d) => ClVal::Date(d),
            ClKey::Identifier(i) => ClVal::Identifier(i),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Value types
pub enum ClVal {
    Integer(i32),
    Float(f32),
    String(String),
    Date(Date),
    Bool(bool),
    Identifier(String),
    List(Vec<ClVal>),
    Dict(HashMap<ClKey, ClVal>),
}

impl ClVal {
    pub fn as_i32(&self) -> Result<&i32> {
        if let ClVal::Integer(ref int) = self {
            Ok(int)
        } else {
            bail!(ErrorKind::InvalidValue("integer".to_string()))
        }
    }

    pub fn as_f32(&self) -> Result<&f32> {
        if let ClVal::Float(ref float) = self {
            Ok(float)
        } else {
            bail!(ErrorKind::InvalidValue("float".to_string()))
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        if let ClVal::String(ref string) = self {
            Ok(string)
        } else {
            bail!(ErrorKind::InvalidValue("string".to_string()))
        }
    }

    pub fn as_bool(&self) -> Result<&bool> {
        if let ClVal::Bool(ref bool) = self {
            Ok(bool)
        } else {
            bail!(ErrorKind::InvalidValue("bool".to_string()))
        }
    }

    pub fn as_list(&self) -> Result<&Vec<ClVal>> {
        if let ClVal::List(ref list) = self {
            Ok(list)
        } else {
            bail!(ErrorKind::InvalidValue("list".to_string()))
        }
    }

    pub fn as_dict(&self) -> Result<&HashMap<ClKey, ClVal>> {
        if let ClVal::Dict(ref dict) = self {
            Ok(dict)
        } else {
            bail!(ErrorKind::InvalidValue("dict".to_string()))
        }
    }

    pub fn as_identifier(&self) -> Result<&str> {
        if let ClVal::Identifier(ref string) = self {
            Ok(string)
        } else {
            bail!(ErrorKind::InvalidValue("identifier".to_string()))
        }
    }

    pub fn as_date(&self) -> Result<&Date> {
        if let ClVal::Date(ref date) = self {
            Ok(date)
        } else {
            bail!(ErrorKind::InvalidValue("identifier".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_i32() {
        let val = ClVal::Integer(42);
        assert_eq!(val.as_i32().unwrap(), &42i32);
        let val = ClVal::Bool(true);
        assert_eq!(val.as_i32().unwrap_err().to_string(), "invalid value type: integer");
    }

    #[test]
    fn test_as_f32() {
        let val = ClVal::Float(13.37);
        assert_eq!(val.as_f32().unwrap(), &13.37f32);
        let val = ClVal::Bool(true);
        assert_eq!(val.as_f32().unwrap_err().to_string(), "invalid value type: float");
    }

    #[test]
    fn test_as_string() {
        let val = ClVal::String("test".to_string());
        assert_eq!(val.as_string().unwrap(), "test");
        let val = ClVal::Bool(true);
        assert_eq!(val.as_string().unwrap_err().to_string(), "invalid value type: string");
    }

    #[test]
    fn test_as_bool() {
        let val = ClVal::Bool(true);
        assert_eq!(val.as_bool().unwrap(), &true);
        let val = ClVal::Integer(1);
        assert_eq!(val.as_bool().unwrap_err().to_string(), "invalid value type: bool");
    }

    #[test]
    fn test_as_list() {
        let list = vec![ClVal::Integer(42)];
        let val = ClVal::List(list.clone());
        assert_eq!(val.as_list().unwrap(), &list);
        let val = ClVal::Bool(true);
        assert_eq!(val.as_list().unwrap_err().to_string(), "invalid value type: list");
    }

    #[test]
    fn test_as_dict() {
        let mut dict = HashMap::new();
        dict.insert(ClKey::String("test".to_string()), ClVal::Integer(42));
        let val = ClVal::Dict(dict.clone());
        assert_eq!(val.as_dict().unwrap(), &dict);
        let val = ClVal::Bool(true);
        assert_eq!(val.as_dict().unwrap_err().to_string(), "invalid value type: dict");
    }

    #[test]
    fn test_as_identifier() {
        let val = ClVal::Identifier("test".to_string());
        assert_eq!(val.as_identifier().unwrap(), "test");
        let val = ClVal::Bool(true);
        assert_eq!(
            val.as_identifier().unwrap_err().to_string(),
            "invalid value type: identifier"
        );
    }

    #[test]
    fn test_as_date() {
        let val = ClVal::Date(Date::new(2018, 5, 16));
        assert_eq!(val.as_date().unwrap(), &Date::new(2018, 5, 16));
        let val = ClVal::Integer(111);
        assert_eq!(
            val.as_identifier().unwrap_err().to_string(),
            "invalid value type: identifier"
        );
    }

    #[test]
    fn test_key_as_i32() {
        let val = ClKey::Integer(42);
        assert_eq!(val.as_i32().unwrap(), &42i32);
        let val = ClKey::String("test".to_string());
        assert_eq!(val.as_i32().unwrap_err().to_string(), "invalid value type: integer");
    }

    #[test]
    fn test_key_as_string() {
        let val = ClKey::String("test".to_string());
        assert_eq!(val.as_string().unwrap(), "test");
        let val = ClKey::Integer(111);
        assert_eq!(val.as_string().unwrap_err().to_string(), "invalid value type: string");
    }

    #[test]
    fn test_key_as_identifier() {
        let val = ClKey::Identifier("test".to_string());
        assert_eq!(val.as_identifier().unwrap(), "test");
        let val = ClKey::Integer(111);
        assert_eq!(
            val.as_identifier().unwrap_err().to_string(),
            "invalid value type: identifier"
        );
    }

    #[test]
    fn test_key_as_date() {
        let val = ClKey::Date(Date::new(2018, 5, 16));
        assert_eq!(val.as_date().unwrap(), &Date::new(2018, 5, 16));
        let val = ClKey::Integer(111);
        assert_eq!(
            val.as_identifier().unwrap_err().to_string(),
            "invalid value type: identifier"
        );
    }

    #[test]
    fn test_parse_date() {
        let s = "2018.5.16";
        let date = Date::new(2018, 5, 16);
        assert_eq!(Date::from_str(s).unwrap(), date);
        let s = "2018.05.16";
        assert_eq!(Date::from_str(s).unwrap(), date);
    }

    #[test]
    fn test_parse_date_error() {
        assert_eq!(Date::from_str("123.45").unwrap_err().to_string(), "not a date");
    }

    #[test]
    fn test_parse_date_error2() {
        assert_eq!(Date::from_str("clearly not a date").unwrap_err().to_string(), "not a date");
    }
}
