use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt;
use std::io;
use std::process;
use std::rc::Rc;

type RcMut<T> = Rc<RefCell<T>>;

const EPLISON: char = 'ε';
const CONCAT: char = '?';
const UNION: char = '+';
const KLEEN: char = '*';
const ANY_CHAR: char = '&';

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

pub fn empty() -> NFA {
    symbol(EPLISON)
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
    a.initial_state = Rc::clone(&a.states.last().unwrap());
    a.final_states.clear();

    let new_final_state = &a.states[a.states.len() - 2];
    a.final_states.push(Rc::clone(&new_final_state));

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

fn regex_to_nfa(regex: &str) -> NFA {
    let normalized = shunting_yard(regex);
    let mut nfa_queque: VecDeque<NFA> = VecDeque::new();
    for c in normalized.chars() {
        match c {
            KLEEN => {
                let a = nfa_queque
                    .pop_back()
                    .expect("Not enough NFA to star operation");

                nfa_queque.push_back(kleen(a));
            }
            CONCAT => {
                let b = nfa_queque
                    .pop_back()
                    .expect("Not enough NFA to perform concatenation");
                let a = nfa_queque
                    .pop_back()
                    .expect("Not enough NFA to perform concatenation");
                nfa_queque.push_back(concat(a, b));
            }
            UNION => {
                let b = nfa_queque
                    .pop_back()
                    .expect("Not enough NFA to perform union");
                let a = nfa_queque
                    .pop_back()
                    .expect("Not enough NFA to perform union");
                nfa_queque.push_back(union(a, b));
            }
            _ => {
                nfa_queque.push_back(symbol(c));
            }
        }
    }

    nfa_queque.pop_back().expect("No NFA to pop!")
}

fn match_pattern(input_line: &str, raw_pattern: &str) -> bool {
    let nfa = regex_to_nfa(raw_pattern);

    false
}

fn insert_concat_symbol(regex: &str) -> String {
    let mut prev_symbol: Option<char> = None;
    let mut output: Vec<char> = vec![];
    for c in regex.chars() {
        let can_concat = c == '(' || c.is_alphanumeric();
        let should_concat = can_concat
            && prev_symbol
                .is_some_and(|prev_c| prev_c.is_alphanumeric() || prev_c == KLEEN || prev_c == ')');

        if should_concat {
            output.push(CONCAT);
        }
        output.push(c);
        prev_symbol = Some(c);
    }

    output.into_iter().collect()
}

fn shunting_yard(raw_regex: &str) -> String {
    let mut operators = VecDeque::new();
    let mut output = Vec::new();
    let precedence: HashMap<char, u8> =
        HashMap::from([('(', 0), (')', 0), (KLEEN, 4), (UNION, 2), (CONCAT, 3)]);

    let regex = insert_concat_symbol(raw_regex);

    for c in regex.chars() {
        match c {
            KLEEN | UNION | CONCAT => {
                if operators.is_empty() {
                    operators.push_back(c);
                } else {
                    loop {
                        let top_operator = operators.pop_back();

                        if top_operator.is_none() {
                            break;
                        }

                        let top_operator = top_operator.unwrap();

                        if precedence.get(&top_operator).unwrap() >= precedence.get(&c).unwrap() {
                            output.push(top_operator);
                        } else {
                            operators.push_back(top_operator);
                            break;
                        }
                    }

                    operators.push_back(c);
                }
            }
            '(' => {
                operators.push_back(c);
            }
            ')' => loop {
                let operator = operators
                    .pop_back()
                    .expect("No more symbols!, cannot find matching parenthesis");

                if operator == '(' {
                    break;
                }

                output.push(operator);
            },
            _ => {
                output.push(c);
            }
        };
    }

    while !operators.is_empty() {
        let operator = operators.pop_back().unwrap();
        output.push(operator);
    }

    output.into_iter().collect()
}

fn main() {
    if env::args().nth(1).unwrap() != "-E" {
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
    fn insert_concat_no_insert_needed() {
        assert_eq!("a", insert_concat_symbol("a"));
    }

    #[test]
    fn insert_concat_two_symbols() {
        assert_eq!("a?b", insert_concat_symbol("ab"));
    }

    #[test]
    fn insert_concat_complex() {
        assert_eq!("a?(a+b)*?b", insert_concat_symbol("a(a+b)*b"));
    }

    #[test]
    fn shunting_yard_empty_input() {
        let output = shunting_yard("");
        assert_eq!(output, String::from(""));
    }

    #[test]
    fn shunting_yard_concat_of_groups() {
        let output = shunting_yard("(ab)(ab)");
        assert_eq!(output, String::from("ab?ab??"));
    }

    #[test]
    fn shunting_yard_complex_example() {
        let output = shunting_yard("a(a+b)*b");
        assert_eq!(output, String::from("aab+*?b?"));
    }

    #[test]
    fn shunting_yard_concat() {
        let output = shunting_yard("ab");
        assert_eq!(output, String::from("ab?"));
    }

    #[test]
    fn shunting_yard_union() {
        let output = shunting_yard("a+b");
        assert_eq!(output, String::from("ab+"));
    }

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

    #[test]
    fn regex_to_nfa_single_char() {
        let nfa = symbol('a');
        let outcome = regex_to_nfa("a");

        let tests = vec!["aa", "", "a", "bb", "abababa"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_kleen() {
        let nfa = kleen(symbol('a'));
        let outcome = regex_to_nfa("a*");

        let tests = vec!["a", "aa", "aaa", "ab", "bbb"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_complex_2() {
        let outcome = regex_to_nfa("(0+11+10(00+1)*01)*");
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
        let tests = vec!["11", "100", "101", "110", "1", "100001"];
        for example in tests {
            let x = nfa.find_match(example);
            let y = outcome.find_match(example);
            assert_eq!(x, y);
        }
    }

    #[test]
    fn regex_to_nfa_complex() {
        let nfa = kleen(union(concat(symbol('a'), symbol('b')), symbol('a')));
        let outcome = regex_to_nfa("(ab+a)*");

        let tests = vec!["ab", "", "aa", "ababab", "bbbaaa"];
        for example in tests {
            let x = nfa.find_match(example);
            let y = outcome.find_match(example);
            assert_eq!(x, y);
        }
    }
}
