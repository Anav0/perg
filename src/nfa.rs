use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

type RcMut<T> = Rc<RefCell<T>>;

pub const EPLISON: char = 'ε';
pub const CONCAT: char = '?';
pub const UNION: char = '+';
pub const KLEEN: char = '*';
pub const ANY_CHAR: char = '&';

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
        let mut states_for_curr_symbol: Vec<RcMut<State>> = vec![Rc::clone(&self.initial_state)];
        let mut states_for_next_symbol: Vec<RcMut<State>> = vec![];

        for c in text.chars() {
            let mut i = 0;
            while i < states_for_curr_symbol.len() {
                let current_state = Rc::clone(&states_for_curr_symbol[i]);

                let current_state_borrowed = (*current_state).borrow();
                let mut any_character_transition: Option<&Transition> = None;

                let mut matches_given_char = false;
                for transition in &current_state_borrowed.transitions {
                    if transition.on == EPLISON {
                        states_for_curr_symbol.push(Rc::clone(&transition.to));
                    }

                    if transition.on == ANY_CHAR {
                        any_character_transition = Some(transition);
                    }

                    if transition.on == c {
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

pub fn symbol(c: char) -> NFA {
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

pub fn union(mut a: NFA, mut b: NFA) -> NFA {
    a.states.append(&mut b.states);
    let new_inital_state = Rc::new(RefCell::new(State::new("initial_n".to_string(), vec![])));
    {
        let mut new_initial_state_borrowed = (*new_inital_state).borrow_mut();
        new_initial_state_borrowed.add_transition(EPLISON, &a.initial_state);
        new_initial_state_borrowed.add_transition(EPLISON, &b.initial_state);
    }
    a.states.push(new_inital_state);
    a.initial_state = Rc::clone(&a.states[a.states.len() - 1]);

    let new_final_state = Rc::new(RefCell::new(State::new("final_n", vec![])));
    a.states.push(new_final_state);

    let new_final_state = &a.states[a.states.len() - 1];

    for final_state in &a.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, new_final_state);
    }

    for final_state in &b.final_states {
        let mut final_state_borrowed = (*final_state).borrow_mut();
        final_state_borrowed.add_transition(EPLISON, new_final_state);
    }

    a.final_states.clear();

    a.final_states.push(Rc::clone(new_final_state));

    a
}

pub fn kleen(mut a: NFA) -> NFA {
    {
        let new_final_state = Rc::new(RefCell::new(State::new("final_n", vec![])));
        a.states.push(new_final_state);

        let new_final_state = a.states.last().unwrap();

        for final_state in &a.final_states {
            let mut final_state_borrowed = (*final_state).borrow_mut();
            final_state_borrowed.add_transition(EPLISON, new_final_state);
            final_state_borrowed.add_transition(EPLISON, &a.initial_state);
        }
    }

    let new_inital_state = Rc::new(RefCell::new(State::new("initial_n".to_string(), vec![])));
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
    }
    a.final_states = b.final_states;

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_match_all_operators() {
        let nfa = kleen(union(
            symbol('0'),
            union(
                concat(symbol('1'), symbol('1')),
                concat(
                    concat(symbol('1'), symbol('0')),
                    concat(
                        kleen(union(concat(symbol('0'), symbol('0')), symbol('1'))),
                        concat(symbol('0'), symbol('1')),
                    ),
                ),
            ),
        ));
        let tests = vec![
            ("11", true),
            ("100", false),
            ("101", false),
            ("110", true),
            ("1", false),
            ("100001", true),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_single_symbol() {
        let nfa = symbol('a');

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
    fn find_match_two_symbols() {
        let nfa = concat(symbol('a'), symbol('b'));

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
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn find_match_four_symbols() {
        let nfa = concat(concat(symbol('a'), symbol('b')), symbol('c'));

        let tests = vec![
            ("abc", true),
            ("abcc", false),
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
            ("", true),
            ("a", true),
            ("aa", true),
            ("aaa", true),
            ("c", false),
            ("ab", false),
            ("b", false),
            ("bbbbb", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
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
            ("ab", false),
            ("aa", false),
            ("bb", false),
            ("", false),
            ("aab", false),
            ("baa", false),
        ];

        for (text, expected) in tests {
            let result = nfa.find_match(text);
            assert_eq!(result, expected);
        }
    }
}
