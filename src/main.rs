use re::regex_to_nfa;
use std::{env, io::stdin, process};

mod nfa;
mod re;

fn match_pattern(input_line: &str, raw_pattern: &str) -> bool {
    let nfa = regex_to_nfa(raw_pattern);

    nfa.find_match(input_line)
}

fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
