use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;
use std::process;

type TransitionTable = HashMap<usize, HashMap<char, usize>>;

struct NFA {
    transitions: TransitionTable,
    initial_state: usize,
    final_states: HashSet<usize>,
}

impl NFA {
    pub fn new(
        initial_state: usize,
        final_states: HashSet<usize>,
        transitions: TransitionTable,
    ) -> Self {
        Self {
            transitions,
            initial_state,
            final_states,
        }
    }

    pub fn find_match(&self, text: &str) -> bool {
        let mut current_state = self.initial_state;
        for c in text.chars() {
            current_state = *self
                .transitions
                .get(&current_state)
                .unwrap()
                .get(&c)
                .unwrap();
        }
        self.final_states.contains(&current_state)
    }
}

pub fn and(a: &NFA, b: &NFA) -> NFA {
    todo!();
}

pub fn or(a: &NFA, b: &NFA) -> NFA {
    todo!();
}

enum Regex {
    Many,
    Number,
    Symbol(char),
}

fn match_pattern(input_line: &str, raw_pattern: &str) -> bool {
    // a*b*
    // Many(Symbol('a')) + Many(Symbol('b'))
    // a\da
    // Symbol('a') + Number + Symbol('a')
    // for symbol in raw_pattern {}

    false
}

fn main() {
    //
    let transitions: TransitionTable = HashMap::from([
        (0, HashMap::from([('a', 1), ('b', 0)])),
        (1, HashMap::from([('a', 1), ('b', 1)])),
    ]);

    let nfa = NFA::new(0, HashSet::from([1]), transitions);

    let tests = vec![("aa", true), ("a", true), ("b", false), ("bbb", false)];
    for (text, expected) in tests {
        let result = nfa.find_match(text);
        println!("nfa({text}) = {result} | {expected}");
    }

    assert!(nfa.find_match("a"));
    assert!(nfa.find_match("aa"));
    assert_eq!(nfa.find_match("ab"), false);

    return;

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
