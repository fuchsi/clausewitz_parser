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

use clausewitz_parser::Tokenizer;
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

    let tokenizer = Tokenizer::new(&buf);
    let tokens = tokenizer.tokenize();

    println!("Tokens:\n{:#?}", tokens);
}
