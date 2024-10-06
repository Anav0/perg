use colored::*;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;
use std::rc::Rc;
use std::{fmt, fs, io};

use crate::Args;

type RcMut<T> = Rc<RefCell<T>>;

pub const EPLISON: char = 'ε';
pub const CONCAT: char = '?';
pub const UNION: char = '+';
pub const KLEEN: char = '*';
pub const ANY_DIGIT: char = '#';
pub const ANY_ALPHANUMERIC: char = '=';
pub const ANY_OTHER_CHAR: char = '&';
pub const SLASH: char = '\\';
pub const CHAR_SET_START: char = '[';
pub const CHAR_SET_END: char = ']';
pub const GROUP_START: char = '(';
pub const GROUP_END: char = ')';

lazy_static! {
    pub static ref RESERVED_CHARS: HashSet<char> = {
        let mut m = HashSet::new();
        m.insert(EPLISON);
        m.insert(CONCAT);
        m.insert(UNION);
        m.insert(KLEEN);
        m.insert(ANY_DIGIT);
        m.insert(ANY_ALPHANUMERIC);
        m.insert(ANY_OTHER_CHAR);
        m.insert(SLASH);
        m.insert(GROUP_START);
        m.insert(GROUP_END);
        m.insert(CHAR_SET_END);
        m.insert(CHAR_SET_START);
        m
    };
    pub static ref CANNOT_CONCAT_PREV_CHAR: HashSet<char> = {
        let mut m = HashSet::new();
        m.insert(GROUP_START);
        m.insert(UNION);
        m.insert(CHAR_SET_START);
        m.insert(SLASH);
        m
    };
    pub static ref CANNOT_CONCAT_CURRENT_CHAR: HashSet<char> = {
        let mut m = HashSet::new();
        m.insert(CONCAT);
        m.insert(UNION);
        m.insert(KLEEN);
        m.insert(GROUP_END);
        m.insert(CHAR_SET_END);
        m
    };
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum StateKind {
    Normal,
    Failed,
    Initial,
    Final,
}

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub transitions: Vec<Transition>,
    pub kind: StateKind,
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
    pub fn new<S: Into<String>>(name: S, transitions: Vec<Transition>, kind: StateKind) -> Self {
        Self {
            name: name.into(),
            transitions,
            kind,
        }
    }

    pub fn add_transition(&mut self, on: char, to: &RcMut<State>) {
        let transition = Transition::new(on, Rc::clone(to));
        self.transitions.push(transition);
    }
}

#[derive(Clone, Debug)]
pub struct NfaOptions {
    pub ignore_case: bool,
}

impl Default for NfaOptions {
    fn default() -> Self {
        Self { ignore_case: false }
    }
}

impl From<&Args> for NfaOptions {
    fn from(value: &Args) -> Self {
        Self {
            ignore_case: value.ignore_case,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NFA {
    pub states: Vec<RcMut<State>>,
    pub initial_state: RcMut<State>,
    pub final_states: Vec<RcMut<State>>,
}

#[derive(Debug)]
pub struct Match {
    pub from: usize,
    pub to: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct FileMatch {
    pub file_path: Option<PathBuf>,
    pub matches: Vec<Match>,
}

impl FileMatch {
    pub fn print_matches(&self) {
        if self.matches.is_empty() {
            return;
        }

        if self.file_path.is_none() {
            return;
        }

        let path = self.file_path.as_ref().unwrap();
        let file = File::open(path).expect(&format!(
            "Failed to read file: '{}'",
            path.to_str().unwrap()
        ));

        println!("{}", path.to_str().unwrap().blue());
        let reader = io::BufReader::new(file);

        let lines: Vec<_> = reader.lines().collect();
        let max_match = self.matches.iter().max_by_key(|x| x.line);

        let line_number_col_size = if max_match.is_some() {
            max_match.unwrap().line.to_string().len()
        } else {
            1
        };

        for m in &self.matches {
            let err_msg = format!(
                "Failed to read line: '{}' from: '{}' line",
                m.line,
                path.to_str().unwrap(),
            );

            let line = lines[m.line].as_ref().expect(&err_msg);

            let before = &line[..m.from];
            let matched = &line[m.from..m.to];
            let after = &line[m.to..];
            println!(
                "{:<line_number_col_size$} {}{}{}",
                (m.line + 1).to_string().green(),
                before,
                matched.red(),
                after
            );
        }
    }
}

impl fmt::Display for NFA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_states_names = vec![];
        for state in &self.final_states {
            let inner_value = (**state).borrow();
            final_states_names.push(inner_value.name.to_string());
        }

        writeln!(f, "Number of states: {}", self.states.len())?;
        writeln!(f, "Initial state: {}", (*self.initial_state).borrow().name)?;
        writeln!(f, "Final states: {}", final_states_names.join(", "))?;
        writeln!(f, "Transitions:")?;

        for state in &self.states {
            let inner_value = (**state).borrow();
            writeln!(f, "\t\"{}\" ({:?})", inner_value.name, inner_value.kind)?;
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

    pub fn find_matches(&self, text: &str) -> Vec<Match> {
        if text.len() == 0 {
            return vec![];
        }

        let mut all_matches: Vec<Match> = vec![];
        let lines = text.split('\n');
        for (line_number, line) in lines.enumerate() {
            for (k, _) in line.char_indices() {
                let mut matches = self.find_matches_inner(&line[k..], k, line_number);
                if !matches.is_empty() {
                    all_matches.append(&mut matches);
                }
            }
        }
        all_matches
    }

    pub fn find_match(&self, text: &str) -> bool {
        if text.len() == 0 {
            return self.find_match_inner(text, 0);
        }

        for (k, _) in text.char_indices() {
            if self.find_match_inner(&text[k..], k) {
                return true;
            }
        }
        false
    }

    fn find_matches_inner(&self, text: &str, start_index: usize, line_number: usize) -> Vec<Match> {
        let mut matches = vec![];
        let mut states_for_curr_symbol: Vec<RcMut<State>> = vec![Rc::clone(&self.initial_state)];
        let mut states_for_next_symbol: Vec<RcMut<State>> = vec![];

        let mut final_index: Option<usize> = None;
        for (k, c) in text.char_indices() {
            let mut i = 0;
            while i < states_for_curr_symbol.len() {
                let current_state = Rc::clone(&states_for_curr_symbol[i]);

                let current_state_borrowed = (*current_state).borrow();

                match current_state_borrowed.kind {
                    StateKind::Final => {
                        final_index = Some(start_index + k);
                    }
                    _ => {}
                }

                let mut any_character_transition: Option<&Transition> = None;

                let mut matches_given_char = false;
                for transition in &current_state_borrowed.transitions {
                    if transition.on == EPLISON {
                        states_for_curr_symbol.push(Rc::clone(&transition.to));
                    }

                    if transition.on == ANY_OTHER_CHAR {
                        any_character_transition = Some(transition);
                    }

                    if transition.on == c
                        || (transition.on == ANY_DIGIT && c.is_numeric())
                        || (transition.on == ANY_ALPHANUMERIC && c.is_alphanumeric())
                    {
                        matches_given_char = true;
                        let appended_state = Rc::clone(&transition.to);
                        states_for_next_symbol.push(appended_state.clone());
                    }
                }

                if !matches_given_char && any_character_transition.is_some() {
                    states_for_next_symbol.push(Rc::clone(&any_character_transition.unwrap().to));
                }

                i += 1;
            }

            if final_index.is_some() {
                matches.push(Match {
                    from: start_index,
                    to: final_index.unwrap(),
                    line: line_number,
                });
                final_index = None;
            }

            states_for_curr_symbol = states_for_next_symbol.clone();
            states_for_next_symbol.clear();
        }

        let mut i = 0;
        while i < states_for_curr_symbol.len() {
            let state = Rc::clone(&states_for_curr_symbol[i]);
            let current_state = (*state).borrow();
            for transition in &current_state.transitions {
                if transition.on == EPLISON {
                    states_for_curr_symbol.push(Rc::clone(&transition.to));
                }
            }
            i += 1;
        }

        matches
    }

    fn find_match_inner(&self, text: &str, start_index: usize) -> bool {
        let mut states_for_curr_symbol: Vec<RcMut<State>> = vec![Rc::clone(&self.initial_state)];
        let mut states_for_next_symbol: Vec<RcMut<State>> = vec![];

        let mut final_index: Option<usize> = None;
        let mut k = 0;
        for c in text.chars() {
            let mut i = 0;
            while i < states_for_curr_symbol.len() {
                let current_state = Rc::clone(&states_for_curr_symbol[i]);

                let current_state_borrowed = (*current_state).borrow();

                match current_state_borrowed.kind {
                    StateKind::Final => {
                        final_index = Some(start_index + k);
                    }
                    _ => {}
                }

                let mut any_character_transition: Option<&Transition> = None;

                let mut matches_given_char = false;
                for transition in &current_state_borrowed.transitions {
                    if transition.on == EPLISON {
                        states_for_curr_symbol.push(Rc::clone(&transition.to));
                    }

                    if transition.on == ANY_OTHER_CHAR {
                        any_character_transition = Some(transition);
                    }

                    if transition.on == c
                        || (transition.on == ANY_DIGIT && c.is_numeric())
                        || (transition.on == ANY_ALPHANUMERIC && c.is_alphanumeric())
                    {
                        matches_given_char = true;
                        let appended_state = Rc::clone(&transition.to);
                        states_for_next_symbol.push(appended_state.clone());
                    }
                }

                if !matches_given_char && any_character_transition.is_some() {
                    states_for_next_symbol.push(Rc::clone(&any_character_transition.unwrap().to));
                }

                i += 1;
            }
            k += 1;

            if final_index.is_some() {
                println!(
                    "Found pattern in: '{}' from: '{}:{}'",
                    text,
                    start_index,
                    final_index.unwrap()
                );
                return true;
            }

            states_for_curr_symbol = states_for_next_symbol.clone();
            states_for_next_symbol.clear();
        }

        let mut i = 0;
        while i < states_for_curr_symbol.len() {
            let state = Rc::clone(&states_for_curr_symbol[i]);
            let current_state = (*state).borrow();
            for transition in &current_state.transitions {
                if transition.on == EPLISON {
                    states_for_curr_symbol.push(Rc::clone(&transition.to));
                }
            }
            i += 1;
        }

        for final_state in &self.final_states {
            for state in &states_for_curr_symbol {
                if Rc::ptr_eq(final_state, state) {
                    return true;
                }
            }
        }

        false
    }
}

pub fn negative_set_of_chars(chars: &Vec<char>, options: &NfaOptions) -> NFA {
    let initial_state = Rc::new(RefCell::new(State::new(
        format!("initial"),
        vec![],
        StateKind::Initial,
    )));
    let final_state = Rc::new(RefCell::new(State::new(
        format!("final"),
        vec![],
        StateKind::Final,
    )));
    let failed_state = Rc::new(RefCell::new(State::new(
        format!("failed"),
        vec![],
        StateKind::Failed,
    )));

    let states = vec![initial_state, final_state, failed_state];

    if options.ignore_case {
        for c in chars {
            states[0]
                .borrow_mut()
                .add_transition(naive_lowercase(*c), &states[2]);
            states[0]
                .borrow_mut()
                .add_transition(naive_uppercase(*c), &states[2]);
        }
    } else {
        for c in chars {
            states[0].borrow_mut().add_transition(*c, &states[2]);
        }
    }

    states[0]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[1]);

    let starting_state = Rc::clone(&states[0]);

    let final_states = vec![Rc::clone(&states[1])];

    NFA::new(states, starting_state, final_states)
}

pub fn set_of_chars(chars: &Vec<char>, options: &NfaOptions) -> NFA {
    let initial_state = Rc::new(RefCell::new(State::new(
        format!("initial"),
        vec![],
        StateKind::Initial,
    )));
    let final_state = Rc::new(RefCell::new(State::new(
        format!("final"),
        vec![],
        StateKind::Final,
    )));
    let failed_state = Rc::new(RefCell::new(State::new(
        format!("failed"),
        vec![],
        StateKind::Failed,
    )));

    let states = vec![initial_state, final_state, failed_state];

    if options.ignore_case {
        for c in chars {
            //From initial to final
            states[0]
                .borrow_mut()
                .add_transition(naive_uppercase(*c), &states[1]);
            states[0]
                .borrow_mut()
                .add_transition(naive_lowercase(*c), &states[1]);
        }
    } else {
        for c in chars {
            //From initial to final
            states[0].borrow_mut().add_transition(*c, &states[1]);
        }
    }

    //From initial to failed
    states[0]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[2]);
    //from final to failed
    states[1]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[2]);

    let starting_state = Rc::clone(&states[0]);

    let final_states = vec![Rc::clone(&states[1])];

    NFA::new(states, starting_state, final_states)
}

pub fn digits() -> NFA {
    let opt = NfaOptions { ignore_case: false };
    concat(symbol(ANY_DIGIT, &opt), kleen(symbol(ANY_DIGIT, &opt)))
}

pub fn alphanumeric(options: &NfaOptions) -> NFA {
    symbol(ANY_ALPHANUMERIC, options)
}

pub fn digit() -> NFA {
    let opt = NfaOptions { ignore_case: false };
    symbol(ANY_DIGIT, &opt)
}

fn naive_uppercase(c: char) -> char {
    c.to_uppercase().collect::<Vec<_>>()[0]
}

fn naive_lowercase(c: char) -> char {
    c.to_lowercase().collect::<Vec<_>>()[0]
}

pub fn symbol(c: char, options: &NfaOptions) -> NFA {
    let initial_state = Rc::new(RefCell::new(State::new(
        format!("initial_{c}"),
        vec![],
        StateKind::Initial,
    )));
    let final_state = Rc::new(RefCell::new(State::new(
        format!("final_{c}"),
        vec![],
        StateKind::Final,
    )));
    let failed_state = Rc::new(RefCell::new(State::new(
        format!("failed_{c}"),
        vec![],
        StateKind::Failed,
    )));

    let states = vec![initial_state, final_state, failed_state];

    //From initial to final
    //TODO: convert transitions so they ternsition on String not on char
    if options.ignore_case {
        states[0]
            .borrow_mut()
            .add_transition(naive_uppercase(c), &states[1]);
        states[0]
            .borrow_mut()
            .add_transition(naive_lowercase(c), &states[1]);
    } else {
        states[0].borrow_mut().add_transition(c, &states[1]);
    }
    //From initial to failed
    states[0]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[2]);
    //from final to failed
    states[1]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[2]);

    let starting_state = Rc::clone(&states[0]);

    let final_states = vec![Rc::clone(&states[1])];

    NFA::new(states, starting_state, final_states)
}

pub fn union(mut a: NFA, mut b: NFA) -> NFA {
    a.states.append(&mut b.states);
    let new_inital_state = Rc::new(RefCell::new(State::new(
        "initial_n".to_string(),
        vec![],
        StateKind::Initial,
    )));
    {
        let mut new_initial_state_borrowed = (*new_inital_state).borrow_mut();
        new_initial_state_borrowed.add_transition(EPLISON, &a.initial_state);
        new_initial_state_borrowed.add_transition(EPLISON, &b.initial_state);
    }
    a.states.push(new_inital_state);
    a.initial_state = Rc::clone(&a.states[a.states.len() - 1]);

    let new_final_state = Rc::new(RefCell::new(State::new(
        "final_n",
        vec![],
        StateKind::Final,
    )));
    a.states.push(new_final_state);

    let new_final_state = &a.states[a.states.len() - 1];

    for final_state in &a.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, new_final_state);
        final_state_borrowed.kind = StateKind::Normal;
    }

    for final_state in &b.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, new_final_state);
        final_state_borrowed.kind = StateKind::Normal;
    }

    a.final_states.clear();

    a.final_states.push(Rc::clone(new_final_state));

    a
}

pub fn kleen(mut a: NFA) -> NFA {
    {
        let new_final_state = Rc::new(RefCell::new(State::new(
            "final_n",
            vec![],
            StateKind::Final,
        )));
        a.states.push(new_final_state);

        let new_final_state = a.states.last().unwrap();

        for final_state in &a.final_states {
            let mut final_state_borrowed = (*final_state).borrow_mut();
            final_state_borrowed.add_transition(EPLISON, new_final_state);
            final_state_borrowed.add_transition(EPLISON, &a.initial_state);
            final_state_borrowed.kind = StateKind::Normal;
        }
    }

    let new_inital_state = Rc::new(RefCell::new(State::new(
        "initial_n".to_string(),
        vec![],
        StateKind::Initial,
    )));
    {
        let mut new_initial_state_borrowed = (*new_inital_state).borrow_mut();
        new_initial_state_borrowed.add_transition(EPLISON, &a.initial_state);

        for final_state in &a.final_states {
            new_initial_state_borrowed.add_transition(EPLISON, final_state);
        }
    }
    a.states.push(new_inital_state);
    a.initial_state = Rc::clone(a.states.last().unwrap());
    a.final_states.clear();

    let new_final_state = &a.states[a.states.len() - 2];
    a.final_states.push(Rc::clone(new_final_state));

    a
}

pub fn concat(mut a: NFA, mut b: NFA) -> NFA {
    a.states.append(&mut b.states);

    for final_state in a.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, &b.initial_state);
        final_state_borrowed.kind = StateKind::Normal;
    }
    a.final_states = b.final_states;

    a
}

#[cfg(test)]
mod tests {
    use crate::re::regex_to_nfa;

    use super::*;

    #[test]
    fn find_match_negative_characters_set() {
        let opt = NfaOptions::default();
        let nfa = negative_set_of_chars(&vec!['a', 'b'], &opt);

        let tests = vec![
            ("apple", true),
            ("banana", true),
            ("ccc", true),
            ("bbb", false),
            ("aaa", false),
        ];

        for (text, expected) in tests {
            println!("{text} {expected}");
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }
    #[test]
    fn find_match_alphanumeric() {
        let opt = NfaOptions::default();
        let nfa = alphanumeric(&opt);

        let tests = vec![
            ("", false),
            ("0", true),
            ("1", true),
            ("11231231321312", true),
            ("123", true),
            ("999", true),
            ("9", true),
            ("a", true),
            ("aa", true),
            ("aaa", true),
            ("śćźż", true),
        ];

        for (text, expected) in tests {
            println!("{text} {expected}");
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }
    #[test]
    fn find_match_digits() {
        let nfa = digits();

        let tests = vec![
            ("", false),
            ("0", true),
            ("1", true),
            ("11231231321312", true),
            ("123", true),
            ("999", true),
            ("9", true),
            ("a", false),
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
            println!("{text} {expected}");
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn ala_test() {
        let opt = NfaOptions::default();
        let nfa = symbol('b', &opt);
        nfa.find_match("ali baba ali baba");
    }

    #[test]
    fn ala_test_2() {
        let opt = NfaOptions::default();
        let nfa = concat(symbol('a', &opt), symbol('b', &opt));
        nfa.find_match("Co za baba");
        //-------------------0123456789
    }

    #[test]
    fn find_match_complex_3() {
        let opt = NfaOptions::default();
        let nfa = regex_to_nfa("\\d\\dabc", &opt);

        let tests = vec![
            ("01abc", true),
            ("abc01abc", true),
            ("12313", false),
            ("abc", false),
            ("awjdnakjd", false),
            ("", false),
        ];

        for (text, expected) in tests {
            println!("{text} {expected}");
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_digit() {
        let nfa = digit();

        let tests = vec![
            ("0", true),
            ("1", true),
            ("9", true),
            ("a", false),
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
            println!("{text} {expected}");
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_character_sets() {
        let opt = NfaOptions::default();
        let nfa = regex_to_nfa("[abc]", &opt);

        let tests = vec![
            ("a", true),
            ("b", true),
            ("c", true),
            ("cc", true),
            ("bb", true),
            ("aa", true),
            ("", false),
            ("x", false),
            ("xa", true),
            ("xb", true),
            ("xc", true),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("'{}' expected '{}'", text, expected);
            assert_eq!(result, expected);
        }
    }
    #[test]
    fn find_match_first_symbol() {
        let opt = NfaOptions::default();
        let nfa = symbol('d', &opt);

        let tests = vec![
            ("", false),
            (" ", false),
            ("dog", true),
            ("dom", true),
            ("aa", false),
            ("", false),
            ("aaa", false),
            ("aaaa", false),
            ("aaaaa", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("'{}' expected '{}'", text, expected);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_single_symbol_ignore_case() {
        let opt = NfaOptions { ignore_case: true };
        let nfa = symbol('a', &opt);

        let tests = vec![
            ("", false),
            (" ", false),
            ("a", true),
            ("A", true),
            ("AA", true),
            ("aa", true),
            ("", false),
            ("aaa", true),
            ("aaaa", true),
            ("aaaaa", true),
            ("ba", true),
            ("bba", true),
            ("bbaa", true),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("'{}' expected '{}'", text, expected);
            assert_eq!(result, expected);
        }
    }
    #[test]
    fn find_match_single_symbol() {
        let opt = NfaOptions::default();
        let nfa = symbol('a', &opt);

        let tests = vec![
            ("", false),
            (" ", false),
            ("a", true),
            ("aa", true),
            ("", false),
            ("aaa", true),
            ("aaaa", true),
            ("aaaaa", true),
            ("ba", true),
            ("bba", true),
            ("bbaa", true),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("'{}' expected '{}'", text, expected);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_two_symbols() {
        let opt = NfaOptions::default();
        let nfa = concat(symbol('a', &opt), symbol('b', &opt));

        let tests = vec![
            ("", false),
            (" ", false),
            ("ab", true),
            ("abb", true),
            ("a", false),
            ("b", false),
            ("", false),
            ("ba", false),
            ("bc", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_four_symbols() {
        let opt = NfaOptions::default();
        let nfa = concat(
            concat(symbol('a', &opt), symbol('b', &opt)),
            symbol('c', &opt),
        );

        let tests = vec![
            ("abc", true),
            ("abcc", true),
            ("c", false),
            ("cc", false),
            ("abb", false),
            ("a", false),
            ("b", false),
            ("", false),
            ("ba", false),
            ("bc", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_concat_concat() {
        let opt = NfaOptions::default();
        //abcc
        let nfa = concat(
            concat(symbol('a', &opt), symbol('b', &opt)),
            concat(symbol('c', &opt), symbol('c', &opt)),
        );

        let tests = vec![
            ("abcc", true),
            ("abc", false),
            ("c", false),
            ("cc", false),
            ("abb", false),
            ("a", false),
            ("b", false),
            ("", false),
            ("ba", false),
            ("bc", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn construction_kleen_test() {
        let opt = NfaOptions::default();
        let nfa = kleen(symbol('a', &opt));

        let tests = vec![
            ("c", false),
            ("", true),
            ("a", true),
            ("aa", true),
            ("aaa", true),
            ("ab", false),
            ("b", false),
            ("bbbbb", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!(
                "Input: '{}' expected: '{}', result: '{}'",
                text, expected, result
            );
            assert_eq!(result, expected);
        }
    }
    #[test]
    fn construction_union_test() {
        let opt = NfaOptions::default();
        let nfa = union(symbol('a', &opt), symbol('b', &opt));

        let tests = vec![
            ("a", true),
            ("b", true),
            ("c", false),
            ("ab", true),
            ("aa", true),
            ("bb", true),
            ("", false),
            ("aab", true),
            ("baa", true),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            println!("'{}' expected '{}'", text, expected);
            assert_eq!(result, expected);
        }
    }
}
