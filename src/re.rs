use std::collections::{HashMap, VecDeque};

use crate::nfa::{concat, kleen, symbol, union, CONCAT, KLEEN, NFA, UNION};

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

pub fn regex_to_nfa(regex: &str) -> NFA {
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
