use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::io;
use std::ops::Range;
use std::ops::RangeBounds;
use std::process;
use std::rc::Rc;

type TransitionTable = BTreeMap<usize, BTreeMap<char, usize>>;
type RcMut<T> = Rc<RefCell<T>>;

const EPLISON: char = '$';
const ANY_CHAR: char = '&';

pub struct Transition {
    pub on: char,
    pub to: RcMut<State>,
}

impl Transition {
    pub fn new(on: char, to: RcMut<State>) -> Self {
        Self { on, to }
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}' -> {}", self.on, (*self.to).borrow())
    }
}

pub struct State {
    pub name: String,
    pub transitions: Vec<Transition>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = vec![];

        for trans in &self.transitions {
            output.push(format!("\t{}", trans));
        }

        write!(f, "\"{}\"\n{}", self.name, output.join("\n"))
    }
}

impl State {
    pub fn new<S: Into<String>>(name: S, transitions: Vec<Transition>) -> Self {
        Self {
            name: name.into(),
            transitions,
        }
    }

    pub fn add_transition(&mut self, on: char, to: &RcMut<State>) {
        let transition = Transition::new(on, Rc::clone(to));
        self.transitions.push(transition);
    }
}

#[derive(Clone)]
pub struct NFA {
    pub states: Vec<RcMut<State>>,
    pub initial_state: RcMut<State>,
    pub final_states: Vec<RcMut<State>>,
}

impl fmt::Display for NFA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = vec![];
        for state in &self.states {
            let inner_value = (**state).borrow();
            output.push(format!("{}", inner_value));
        }
        write!(f, "{}", output.join("\n"))
    }
}

impl NFA {
    pub fn new(
        states: Vec<RcMut<State>>,
        initial_state: RcMut<State>,
        final_states: Vec<RcMut<State>>,
    ) -> Self {
        Self {
            states,
            initial_state,
            final_states,
        }
    }

    pub fn find_match(&self, text: &str) -> bool {
        let mut current_state = Rc::clone(&self.initial_state);

        //TODO: check if current_state is copy of initial, they must not be pointing to the same memory
        for c in text.chars() {
            let mut next_state: Option<RcMut<State>> = None;
            let mut matched = false;
            let mut any_char_transition: Option<&Transition> = None;
            {
                let current_state_borrowed = (*current_state).borrow();
                for transition in &current_state_borrowed.transitions {
                    if transition.on == ANY_CHAR {
                        any_char_transition = Some(transition);
                    }
                    if transition.on == c {
                        next_state = Some(Rc::clone(&transition.to));
                        matched = true;
                        break;
                    }
                }

                if !matched && any_char_transition.is_some() {
                    let transition = any_char_transition.unwrap();
                    next_state = Some(Rc::clone(&transition.to));
                }
            }

            if next_state.is_some() {
                current_state = Rc::clone(&next_state.unwrap());
            }
        }

        for potential_final_state in &self.final_states {
            if Rc::ptr_eq(&potential_final_state, &current_state) {
                return true;
            }
        }

        false
    }
}

pub fn single(c: char) -> NFA {
    let initial_state = Rc::new(RefCell::new(State::new(format!("initial_{c}"), vec![])));
    let final_state = Rc::new(RefCell::new(State::new(format!("final_{c}"), vec![])));
    let failed_state = Rc::new(RefCell::new(State::new(format!("failed_{c}"), vec![])));

    let states = vec![initial_state, final_state, failed_state];

    //From initial to final
    states[0].borrow_mut().add_transition(c, &states[1]);
    //From initial to failed
    states[0].borrow_mut().add_transition(ANY_CHAR, &states[2]);
    //from final to failed
    states[1].borrow_mut().add_transition(ANY_CHAR, &states[2]);

    let starting_state = Rc::clone(&states[0]);

    let final_states = vec![Rc::clone(&states[1])];

    NFA::new(states, starting_state, final_states)
}

pub fn empty() -> NFA {
    single(EPLISON)
}

pub fn union(a: &NFA, b: &NFA) -> NFA {
    todo!()
}

// Constructs NFA for single symbol like 'a' or 'b'
pub fn symbol<R: RangeBounds<usize>>(c: char, occurences: R) -> NFA {
    let mut transitions: TransitionTable = BTreeMap::new();

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

    todo!()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_symbol() {
        let nfa = single('a');

        println!("{}", nfa);

        let tests = vec![
            ("a", true),
            ("aa", false),
            ("aaa", false),
            ("aaaa", false),
            ("aaaaa", false),
            ("ba", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("Test input: '{text}'");
            assert_eq!(result, expected);
        }
    }
}
