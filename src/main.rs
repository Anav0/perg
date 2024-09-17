use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;
use std::ops::Range;
use std::ops::RangeBounds;
use std::process;

type TransitionTable = HashMap<usize, HashMap<char, usize>>;
const ANY_OTHER_SYMBOL: char = '$';

pub struct NFA {
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
            let have_transition_for_this_char = self
                .transitions
                .get(&current_state)
                .unwrap()
                .contains_key(&c);

            let mut transition_symbol = c;
            if !have_transition_for_this_char {
                transition_symbol = ANY_OTHER_SYMBOL;
            }

            current_state = *self
                .transitions
                .get(&current_state)
                .unwrap()
                .get(&transition_symbol)
                .unwrap();
        }
        self.final_states.contains(&current_state)
    }
}

pub fn symbol<R: RangeBounds<usize>>(c: char, occurences: R) -> NFA {
    let mut transitions: TransitionTable = HashMap::new();

    let mut start = 0;
    let mut end = 0;

    match occurences.end_bound() {
        std::ops::Bound::Included(v) => end = *v,
        std::ops::Bound::Excluded(v) => end = *v,
        std::ops::Bound::Unbounded => end = 0,
    }

    match occurences.start_bound() {
        std::ops::Bound::Included(v) => start = *v,
        std::ops::Bound::Excluded(v) => start = *v,
        std::ops::Bound::Unbounded => start = 0,
    }

    if end == 0 {
        //TODO: handle ay number of characters
        todo!()
    }

    let mut final_states = HashSet::new();

    let failure_state_index = end + 1;

    for i in 0..=end {
        println!("{i}");
        transitions.entry(i).or_insert(HashMap::from([
            (c, i + 1),
            (ANY_OTHER_SYMBOL, failure_state_index),
        ]));

        if occurences.contains(&i) {
            final_states.insert(i);
        }
    }

    // Add failure state
    transitions.insert(
        failure_state_index,
        HashMap::from([(ANY_OTHER_SYMBOL, failure_state_index)]),
    );

    println!("Transition: {:?}", transitions);
    println!("Final states: {:?}", final_states);

    NFA::new(0, final_states, transitions)
}

impl std::ops::BitAnd<NFA> for NFA {
    type Output = NFA;

    fn bitand(self, rhs: NFA) -> Self::Output {
        todo!()
    }
}

impl std::ops::BitOr<NFA> for NFA {
    type Output = NFA;

    fn bitor(self, rhs: NFA) -> Self::Output {
        todo!()
    }
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
    let nfa = symbol('a', 2..=4);

    let tests = vec![
        ("a", false),
        ("aa", true),
        ("aaa", true),
        ("aaaa", true),
        ("aaaaa", false),
        ("ba", false),
    ];
    for (text, expected) in tests {
        let result = nfa.find_match(text);
        println!("nfa({text}) = {result} | {expected}");
    }

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
