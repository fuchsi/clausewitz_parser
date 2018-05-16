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
extern crate clausewitz_parser;
extern crate encoding;
extern crate env_logger;

use clausewitz_parser::Parser;
use clausewitz_parser::Tokenizer;
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use std::env::args;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut args = args();
    args.next();
    let name = args.next().unwrap();
    println!("open: {}", name);
    let mut file = File::open(name).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let buf = if buf.starts_with(b"EU4txt") {
        &buf[6..]
    } else {
        &buf[..]
    };

    let encoded = WINDOWS_1252.decode(&buf, DecoderTrap::Strict).unwrap();

    let tokenizer = Tokenizer::new(encoded.as_bytes());
    let tokens = tokenizer.tokenize();
    let mut parser = Parser::new(tokens);
    let clvals = parser.parse().unwrap();

    println!("CL Values:\n{:#?}", clvals);
}
