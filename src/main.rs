use std::env;
use std::io;
use std::process;

enum Symbol {
    Number,
    Text(String),
}

struct Pattern {
    source: Vec<char>,
    index: usize,
    length: usize,
}

impl Pattern {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            index: 0,
            length: 0,
        }
    }
}

impl Iterator for Pattern {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer: Vec<char> = Vec::with_capacity(16);

        for i in self.index..self.source.len() {
            let char = self.source[i];
            self.index += 1;

            let symbol = match buffer.as_slice() {
                ['\\', 'd'] => Some(Symbol::Number),
                _ => None,
            };

            if symbol.is_some() {
                return symbol;
            }

            buffer.push(char);
        }

        None
    }
}

fn match_pattern(input_line: &str, raw_pattern: &str) -> bool {
    let mut pattern = Pattern::new(raw_pattern);

    for symbol in pattern {}

    false
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
