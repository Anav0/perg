use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::io;
use std::io::Write;
use std::ops::Range;
use std::ops::RangeBounds;
use std::process;
use std::rc::Rc;

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
        write!(f, "'{}' -> {}", self.on, (*self.to).borrow().name)
    }
}

pub struct State {
    pub name: String,
    pub transitions: Vec<Transition>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\"{}\"", self.name,)?;
        for trans in &self.transitions {
            writeln!(f, "\t\t{}", trans)?;
        }
        Ok(())
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
    pub fn get_states_on(&self, c: char) -> Vec<RcMut<State>> {
        let mut output = vec![];
        for trans in &self.transitions {
            if trans.on == c {
                output.push(Rc::clone(&trans.to));
            }
        }
        output
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
        let mut final_states_names = vec![];
        for state in &self.final_states {
            let inner_value = (**state).borrow();
            final_states_names.push(format!("{}", inner_value.name));
        }

        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Initial state: {}", (*self.initial_state).borrow().name)?;
        writeln!(f, "Final states: {}", final_states_names.join(", "))?;
        writeln!(f, "Transitions:")?;

        for state in &self.states {
            let inner_value = (**state).borrow();
            writeln!(f, "\t\"{}\"", inner_value.name)?;
            for trans in &inner_value.transitions {
                writeln!(f, "\t\t{}", trans)?;
            }
        }

        Ok(())
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
        let mut states_to_simulate: Vec<RcMut<State>> = vec![Rc::clone(&self.initial_state)];
        let mut states_to_append: Vec<RcMut<State>> = vec![];

        for c in text.chars() {
            for state in &states_to_simulate {
                let current_state = Rc::clone(&state);

                let current_state_borrowed = (*current_state).borrow();
                let mut any_character_transition: Option<&Transition> = None;

                let mut matches_given_char = false;
                for transition in &current_state_borrowed.transitions {
                    if transition.on == ANY_CHAR {
                        any_character_transition = Some(transition);
                    }

                    if transition.on == c {
                        matches_given_char = true;
                        let appended_state = Rc::clone(&transition.to);
                        let appended_state_borrow = (*appended_state).borrow();
                        let mut epsilon_states = appended_state_borrow.get_states_on(EPLISON);
                        states_to_append.append(&mut epsilon_states);
                        states_to_append.push(appended_state.clone());
                    }
                }
                if !matches_given_char && any_character_transition.is_some() {
                    states_to_append.push(Rc::clone(&any_character_transition.unwrap().to));
                }
            }
            states_to_simulate = states_to_append.clone();
            states_to_append.clear();
        }

        for final_state in &self.final_states {
            for state in &states_to_simulate {
                if Rc::ptr_eq(final_state, state) {
                    return true;
                }
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

pub fn concat(mut a: NFA, mut b: NFA) -> NFA {
    a.states.append(&mut b.states);

    for final_state in a.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, &b.initial_state);
    }
    a.final_states = b.final_states;

    a
}

// Constructs NFA for single symbol like 'a' or 'b'
// pub fn symbol<R: RangeBounds<usize>>(c: char, occurences: R) -> NFA {
//     let mut transitions: TransitionTable = BTreeMap::new();

//     let mut start = 0;
//     let mut end = 0;

//     match occurences.end_bound() {
//         std::ops::Bound::Included(v) => end = *v,
//         std::ops::Bound::Excluded(v) => end = *v,
//         std::ops::Bound::Unbounded => end = 0,
//     }

//     match occurences.start_bound() {
//         std::ops::Bound::Included(v) => start = *v,
//         std::ops::Bound::Excluded(v) => start = *v,
//         std::ops::Bound::Unbounded => start = 0,
//     }

//     if end == 0 {
//         //TODO: handle ay number of characters
//         todo!()
//     }

//     todo!()
// }

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

    // #[test]
    fn single_symbol() {
        let nfa = single('a');

        let tests = vec![
            ("a", true),
            ("aa", false),
            ("", false),
            ("aaa", false),
            ("aaaa", false),
            ("aaaaa", false),
            ("ba", false),
            ("bba", false),
            ("bbaa", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn two_symbols() {
        let nfa = concat(single('a'), single('b'));

        println!("{}", nfa);

        let tests = vec![
            ("ab", true),
            ("abb", false),
            ("a", false),
            ("b", false),
            ("", false),
            ("ba", false),
            ("bc", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("Test input: '{text}'");
            assert_eq!(result, expected);
        }
    }
}
