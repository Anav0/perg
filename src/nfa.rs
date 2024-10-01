use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

type RcMut<T> = Rc<RefCell<T>>;

pub const EPLISON: char = 'ε';
pub const CONCAT: char = '?';
pub const UNION: char = '+';
pub const KLEEN: char = '*';
pub const ANY_DIGIT: char = '#';
pub const ANY_ALPHANUMERIC: char = '=';
pub const ANY_OTHER_CHAR: char = '&';

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

    pub fn find_match(&self, text: &str) -> bool {
        if text.len() == 0 {
            return self.find_match_inner(text, 0);
        }

        for k in 0..text.len() {
            if self.find_match_inner(&text[k..], k) {
                return true;
            }
        }
        false
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

pub fn negative_set_of_chars(chars: &Vec<char>) -> NFA {
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

    for c in chars {
        states[0].borrow_mut().add_transition(*c, &states[2]);
    }

    states[0]
        .borrow_mut()
        .add_transition(ANY_OTHER_CHAR, &states[1]);

    let starting_state = Rc::clone(&states[0]);

    let final_states = vec![Rc::clone(&states[1])];

    NFA::new(states, starting_state, final_states)
}

pub fn set_of_chars(chars: &Vec<char>) -> NFA {
    /*
    if chars.len() <= 0 {
        panic!("Needs at least one char");
    }

    let mut nfa = symbol(chars[0]);

    for i in 1..chars.len() {
        nfa = union(nfa, symbol(chars[i]));
    }

    nfa
    */

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

    for c in chars {
        //From initial to final
        states[0].borrow_mut().add_transition(*c, &states[1]);
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
    concat(symbol(ANY_DIGIT), kleen(symbol(ANY_DIGIT)))
}

pub fn alphanumeric() -> NFA {
    symbol(ANY_ALPHANUMERIC)
}

pub fn digit() -> NFA {
    symbol(ANY_DIGIT)
}

pub fn symbol(c: char) -> NFA {
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
    states[0].borrow_mut().add_transition(c, &states[1]);
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
        let nfa = negative_set_of_chars(&vec!['a', 'b']);

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
        let nfa = alphanumeric();

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
        let nfa = symbol('b');
        nfa.find_match("ali baba ali baba");
    }

    #[test]
    fn ala_test_2() {
        let nfa = concat(symbol('a'), symbol('b'));
        println!("{}", nfa);
        nfa.find_match("Co za baba");
        //-------------------0123456789
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
        let nfa = regex_to_nfa("[abc]");

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
        let nfa = symbol('d');

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
    fn find_match_single_symbol() {
        let nfa = symbol('a');

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
        let nfa = concat(symbol('a'), symbol('b'));

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
        let nfa = concat(concat(symbol('a'), symbol('b')), symbol('c'));

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
        //abcc
        let nfa = concat(
            concat(symbol('a'), symbol('b')),
            concat(symbol('c'), symbol('c')),
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
        let nfa = kleen(symbol('a'));

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
        let nfa = union(symbol('a'), symbol('b'));

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
