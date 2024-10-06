use std::collections::{HashMap, VecDeque};

use crate::nfa::{
    alphanumeric, concat, digits, kleen, negative_set_of_chars, set_of_chars, symbol, union,
    NfaOptions, CANNOT_CONCAT_CURRENT_CHAR, CANNOT_CONCAT_PREV_CHAR, CHAR_SET_END, CHAR_SET_START,
    CONCAT, GROUP_END, GROUP_START, KLEEN, NFA, SLASH, UNION,
};

fn insert_concat_symbol(regex: &str) -> String {
    let mut prev_symbol: Option<char> = None;
    let mut output: Vec<char> = vec![];
    let mut is_in_char_set = false;
    for c in regex.chars() {
        if c == CHAR_SET_START {
            is_in_char_set = true;
        }
        if c == CHAR_SET_END {
            is_in_char_set = false;
        }

        let can_concat = !is_in_char_set
            && !CANNOT_CONCAT_CURRENT_CHAR.contains(&c)
            && prev_symbol.is_some_and(|prev_c| !CANNOT_CONCAT_PREV_CHAR.contains(&prev_c));

        if can_concat {
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
    let precedence: HashMap<char, u8> = HashMap::from([
        (GROUP_START, 0),
        (GROUP_END, 0),
        (KLEEN, 4),
        (UNION, 2),
        (CONCAT, 3),
    ]);

    let regex = insert_concat_symbol(raw_regex);

    let mut is_in_char_set = false;
    for c in regex.chars() {
        match c {
            CHAR_SET_END => {
                is_in_char_set = false;
                output.push(c);
            }
            _ if is_in_char_set => {
                output.push(c);
            }
            KLEEN | UNION | CONCAT if !is_in_char_set => {
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
            CHAR_SET_START => {
                is_in_char_set = true;
                output.push(c);
            }

            GROUP_START => {
                operators.push_back(c);
            }
            GROUP_END => loop {
                let operator = operators
                    .pop_back()
                    .expect("No more symbols!, cannot find matching parenthesis");

                if operator == GROUP_START {
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

pub fn regex_to_nfa(regex: &str, options: &NfaOptions) -> NFA {
    let normalized = shunting_yard(regex);
    let mut nfa_queque: VecDeque<NFA> = VecDeque::new();
    let mut symbols = normalized.chars().peekable();
    let mut c = symbols.next();

    let mut is_in_char_group = false;
    let mut negation = false;
    let mut character_set: Vec<char> = vec![];
    while c.is_some() {
        match c.unwrap() {
            '^' if is_in_char_group => {
                negation = true;
            }
            '^' => {}
            CHAR_SET_END => {
                let nfa = if !negation {
                    set_of_chars(&character_set, options)
                } else {
                    negative_set_of_chars(&character_set, options)
                };
                nfa_queque.push_back(nfa);
                character_set.clear();
                is_in_char_group = false;
            }
            _ if is_in_char_group => {
                character_set.push(c.unwrap());
            }
            CHAR_SET_START => {
                is_in_char_group = true;
            }
            SLASH => {
                let next_symbol = symbols.peek().expect("Nothing follows '\' symbol");
                let nfa: Option<NFA> = match *next_symbol {
                    'd' => Some(digits()),
                    'w' => Some(alphanumeric(options)),
                    _ => None,
                };

                if nfa.is_some() {
                    nfa_queque.push_back(nfa.unwrap());
                    symbols.next();
                }
            }
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
                nfa_queque.push_back(symbol(c.unwrap(), options));
            }
        }

        c = symbols.next();
    }

    nfa_queque.pop_back().expect("No NFA to pop!")
}

#[cfg(test)]
mod tests {
    use crate::nfa::digits;

    use super::*;

    #[test]
    fn insert_concat_underscore() {
        assert_eq!("a?_?b", insert_concat_symbol("a_b"));
    }

    #[test]
    fn insert_concat_no_insert_needed() {
        assert_eq!("a", insert_concat_symbol("a"));
    }

    #[test]
    fn insert_concat_two_symbols() {
        assert_eq!("a?b", insert_concat_symbol("ab"));
    }

    #[test]
    fn insert_concat_ignore_char_sets() {
        assert_eq!("[abc]", insert_concat_symbol("[abc]"));
    }

    #[test]
    fn insert_concat_ignore_char_sets_and_nothing_else_1() {
        assert_eq!("[abc]?a", insert_concat_symbol("[abc]a"));
    }

    #[test]
    fn insert_concat_ignore_char_sets_and_nothing_else() {
        assert_eq!("[abc]?a+b", insert_concat_symbol("[abc]a+b"));
    }

    #[test]
    fn insert_concat_decimal() {
        assert_eq!("\\d", insert_concat_symbol("\\d"));
    }

    #[test]
    fn insert_concat_word() {
        assert_eq!("\\w", insert_concat_symbol("\\w"));
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
    fn shunting_yard_ignore_negative_character_groups() {
        let output = shunting_yard("[^abc]");
        assert_eq!(output, String::from("[^abc]"));
    }

    #[test]
    fn shunting_yard_ignore_negative_character_groups_and_nothing_else_1() {
        let output = shunting_yard("[^abc]a");
        assert_eq!(output, String::from("[^abc]a?"));
    }

    #[test]
    fn shunting_yard_ignore_character_groups() {
        let output = shunting_yard("[abc]");
        assert_eq!(output, String::from("[abc]"));
    }

    #[test]
    fn shunting_yard_ignore_character_groups_and_nothing_else_1() {
        let output = shunting_yard("[abc]a");
        assert_eq!(output, String::from("[abc]a?"));
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
    fn shunting_yard_concat_with_char_set() {
        let output = shunting_yard("[ab]c");
        assert_eq!(output, String::from("[ab]c?"));
    }

    #[test]
    fn shunting_yard_underscore() {
        let output = shunting_yard("a_b");
        assert_eq!(output, String::from("a_?b?"));
    }

    #[test]
    fn shunting_yard_long_concat() {
        let output = shunting_yard("abcdefghijk");
        assert_eq!(output, String::from("ab?c?d?e?f?g?h?i?j?k?"));
    }

    #[test]
    fn shunting_yard_concat() {
        let output = shunting_yard("ab");
        assert_eq!(output, String::from("ab?"));
    }

    #[test]
    fn shunting_yard_decimal() {
        let output = shunting_yard("\\d");
        assert_eq!(output, String::from("\\d"));
    }

    #[test]
    fn shunting_yard_word() {
        let output = shunting_yard("\\w");
        assert_eq!(output, String::from("\\w"));
    }

    #[test]
    fn shunting_yard_union() {
        let output = shunting_yard("a+b");
        assert_eq!(output, String::from("ab+"));
    }

    #[test]
    fn regex_to_nfa_negative_character_set() {
        let opt = NfaOptions::default();
        let nfa = negative_set_of_chars(&vec!['a', 'b'], &opt);
        let outcome = regex_to_nfa("[^ab]", &opt);

        let tests = vec!["a", "b", "c", "ab", "ac", "abc", "", "xyz"];
        for example in tests {
            println!("{}", example);
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_character_set() {
        let opt = NfaOptions::default();
        let nfa = set_of_chars(&vec!['a', 'b', 'c'], &opt);
        let outcome = regex_to_nfa("[abc]", &opt);

        let tests = vec!["a", "b", "c", "ab", "ac", "abc", "", "xyz"];
        for example in tests {
            println!("{}", example);
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_alphanumeric() {
        let opt = NfaOptions::default();
        let nfa = alphanumeric(&opt);
        let outcome = regex_to_nfa("\\w", &opt);

        let tests = vec!["0", "123", "aa", "", "a", "bb", "abababa"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_digits() {
        let opt = NfaOptions::default();
        let nfa = digits();
        let outcome = regex_to_nfa("\\d", &opt);

        let tests = vec!["0", "123", "aa", "", "a", "bb", "abababa"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_single_char_ignore_case() {
        let opt = NfaOptions { ignore_case: true };
        let nfa = symbol('a', &opt);
        let outcome = regex_to_nfa("a", &opt);

        let tests = vec!["aa", "", "a", "bb", "abababa", "A"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_single_char() {
        let opt = NfaOptions::default();
        let nfa = symbol('a', &opt);
        let outcome = regex_to_nfa("a", &opt);

        let tests = vec!["aa", "", "a", "bb", "abababa"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_ignore_case() {
        let opt = NfaOptions { ignore_case: true };
        let nfa = kleen(symbol('a', &opt));
        let outcome = regex_to_nfa("a*", &opt);

        let tests = vec!["a", "aa", "A", "aaa", "ab", "bbb"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_kleen() {
        let opt = NfaOptions::default();
        let nfa = kleen(symbol('a', &opt));
        let outcome = regex_to_nfa("a*", &opt);

        let tests = vec!["a", "aa", "aaa", "ab", "bbb"];
        for example in tests {
            assert_eq!(nfa.find_match(example), outcome.find_match(example));
        }
    }

    #[test]
    fn regex_to_nfa_complex_2() {
        let opt = NfaOptions::default();
        let outcome = regex_to_nfa("(0+11+10(00+1)*01)*", &opt);
        let nfa = kleen(union(
            symbol('0', &opt),
            union(
                concat(symbol('1', &opt), symbol('1', &opt)),
                concat(
                    concat(symbol('1', &opt), symbol('0', &opt)),
                    concat(
                        kleen(union(
                            concat(symbol('0', &opt), symbol('0', &opt)),
                            symbol('1', &opt),
                        )),
                        concat(symbol('0', &opt), symbol('1', &opt)),
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
        let opt = NfaOptions::default();
        let nfa = kleen(union(
            concat(symbol('a', &opt), symbol('b', &opt)),
            symbol('a', &opt),
        ));
        let outcome = regex_to_nfa("(ab+a)*", &opt);

        let tests = vec!["ab", "", "aa", "ababab", "bbbaaa"];
        for example in tests {
            let x = nfa.find_match(example);
            let y = outcome.find_match(example);
            assert_eq!(x, y);
        }
    }
}
