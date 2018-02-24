// MIT License
//
// Copyright (C) 2015-2018 Steve Sprang
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// This is an implementation of the Malbolge interpreter in Rust.
/// It's basically a translation of the original C version found here:
/// http://www.lscheffer.com/malbolge_interp.html
///
/// For more information about Malbolge:
///     http://en.wikipedia.org/wiki/Malbolge
///     http://www.lscheffer.com/malbolge_spec.html
///

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

static XLAT1: &[u8] = b"+b(29e*j1VMEKLyC})8&m#~W>qxdRp0wkrUo[D7,XTcA\"lI\
                        .v%{gJh4G\\-=O@5`_3i<?Z';FNQuY]szf$!BS/|t:Pn6^Ha";

static XLAT2: &[u8] = b"5z]&gqtyfr$(we4{WP)H-Zn,[%\\3dL+Q;>U!pJS72FhOA1C\
                        B6v^=I_0/8|jsb9m<.TVac`uY*MK'X~xDl}REokN:#?G\"i@";

const MAX_MEMORY: usize = 59049; // == 3^10

// u16 would work here, but this saves a bunch of casting
type Memory = [usize; MAX_MEMORY];

////////////////////////////////////////////////////////////////////////////////
// InitError
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum InitError {
    InvalidChar(char, usize),
    SourceTooShort,
    SourceTooLong,
}

use InitError::*;

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidChar(c, loc) =>
                write!(f, "Invalid character in source program: '{}' \
                           at location: {:#X}", c, loc),
            SourceTooShort => write!(f, "Source program is too short."),
            SourceTooLong => write!(f, "Source program is too long."),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} FILE", args[0]);
        return;
    }

    match load(&args[1]) {
        Ok(contents) => run(contents),
        Err(e) => println!("{}", e),
    }
}

////////////////////////////////////////////////////////////////////////////////
// File Handling
////////////////////////////////////////////////////////////////////////////////

fn load(filename: &str) -> std::io::Result<Vec<u8>> {
    let path = Path::new(filename);
    let mut file = File::open(path)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}

////////////////////////////////////////////////////////////////////////////////
// Interpreter Core
////////////////////////////////////////////////////////////////////////////////

fn run(contents: Vec<u8>) {
    let mut mem = [0; MAX_MEMORY];

    match init(contents, &mut mem) {
        Ok(_) => execute(&mut mem),
        Err(why) => println!("Could not initialize memory.\n{}", why),
    }
}

fn init(input: Vec<u8>, mem: &mut Memory) -> Result<usize, InitError> {
    let mut i = 0;
    let valid = "ji*p</vo";

    for (loc, &b) in input.iter().enumerate() {
        if (b as char).is_whitespace() {
            continue;
        }

        if is_printable(b as usize) {
            let index = (b as usize - 33 + i) % 94;
            let test = XLAT1[index] as char;

            if !valid.contains(&test.to_string()) {
                return Err(InvalidChar(b as char, loc));
            }
        }

        if i >= MAX_MEMORY {
            return Err(SourceTooLong);
        }

        mem[i] = b as usize;
        i += 1;
    }

    if i < 2 {
        // the C version does not check for this case
        return Err(SourceTooShort);
    }

    // fill in the rest of memory
    for n in i..MAX_MEMORY {
        mem[n] = crazy_op(mem[n - 1], mem[n - 2]);
    }

    Ok(MAX_MEMORY)
}

fn execute(mem: &mut Memory) {
    let mut r_a = 0;
    let mut r_c = 0;
    let mut r_d = 0;
    let mut input = std::io::stdin();

    while is_printable(mem[r_c]) {
        let index = (mem[r_c] - 33 + r_c) % 94;
        let op = XLAT1[index] as char;

        match op {
            'j' => r_d = mem[r_d],
            'i' => r_c = mem[r_d],
            '*' => {
                r_a = tri_rotate(mem[r_d]);
                mem[r_d] = r_a;
            }
            'p' => {
                r_a = crazy_op(r_a, mem[r_d]);
                mem[r_d] = r_a;
            }
            '<' => print!("{}", r_a as u8 as char),
            '/' => {
                let mut buf = [0u8];
                let result = input.read(&mut buf);

                match result {
                    Ok(cnt) => {
                        if cnt == 1 {
                            // read a byte
                            r_a = buf[0] as usize;
                        } else if cnt == 0 {
                            // EOF
                            r_a = MAX_MEMORY - 1;
                        }
                    }
                    Err(e) => println!("{}", e),
                }
            }
            'v' => return,
            _ => { /* no op */ }
        }

        let index = mem[r_c] - 33;
        mem[r_c] = XLAT2[index] as usize;
        r_c = (r_c + 1) % MAX_MEMORY;
        r_d = (r_d + 1) % MAX_MEMORY;
    }
}

////////////////////////////////////////////////////////////////////////////////
// Interpreter Functions
////////////////////////////////////////////////////////////////////////////////

#[inline]
fn is_printable(c: usize) -> bool {
    32 < c && c < 127
}

#[inline]
fn tri_rotate(x: usize) -> usize {
    // shift right and move the rightmost trit to the front
    let (q, r) = (x / 3, x % 3);
    q + r * 19683 // 3^9 == 19683
}

#[inline]
fn crazy_op(x: usize, y: usize) -> usize {
    static P9: [usize; 5] = [1, 9, 81, 729, 6561];
    static O: [[usize; 9]; 9] = [
        [4, 3, 3, 1, 0, 0, 1, 0, 0],
        [4, 3, 5, 1, 0, 2, 1, 0, 2],
        [5, 5, 4, 2, 2, 1, 2, 2, 1],
        [4, 3, 3, 1, 0, 0, 7, 6, 6],
        [4, 3, 5, 1, 0, 2, 7, 6, 8],
        [5, 5, 4, 2, 2, 1, 8, 8, 7],
        [7, 6, 6, 7, 6, 6, 4, 3, 3],
        [7, 6, 8, 7, 6, 8, 4, 3, 5],
        [8, 8, 7, 8, 8, 7, 5, 5, 4],
    ];

    (0..5).fold(0, |sum, i| sum + O[y / P9[i] % 9][x / P9[i] % 9] * P9[i])
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::tri_rotate;

    #[test]
    fn rotate_test() {
        let input = 17;
        let rotated = (0..10).fold(input, |prev, _| tri_rotate(prev));
        assert_eq!(input, rotated);
    }
}
