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

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
        Fmt(::std::fmt::Error);
        ParseInt(::std::num::ParseIntError);
        ParseFloat(::std::num::ParseFloatError);
    }

    errors {
        NoToken {
            description("not a token")
            display("not a token")
        }
        InvalidValue(t: String) {
            description("invalid value type")
            display("invalid value type: {}", t)
        }
        InvalidToken
    }
}