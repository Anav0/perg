use std::{
    collections::VecDeque,
    fs::{self, ReadDir},
    path::{Iter, Path, PathBuf, Component},
};

#[derive(Debug)]
pub struct GlobError {
    pub msg: String,
}

#[derive(Debug)]
pub enum PathEntry {
    File(PathBuf),
    Dir(ReadDir),
}

#[derive(Debug)]
pub struct Paths<'a> {
    pattern_chars: Vec<char>,
    components: Vec<&'a str>,
    path: &'a PathBuf,
    is_wildcard: bool,
    entries_to_process: VecDeque<PathEntry>,
}

pub fn to_lexical_absolute<P: AsRef<Path>>(p: P) -> std::io::Result<PathBuf> {
        let path = p.as_ref();
        let mut absolute = if path.is_absolute() {
            PathBuf::new()
        } else {
            std::env::current_dir()?
        };
        for component in path.components() {
            match component {
                Component::CurDir => {},
                Component::ParentDir => { absolute.pop(); },
                component @ _ => absolute.push(component.as_os_str()),
            }
        }
        Ok(absolute)
    }

impl<'a> Paths<'a> {
    pub fn matches(&self, path: &PathBuf) -> Result<bool, GlobError> {
        if !path.is_file() {
            panic!("Paths to dir are not yet supported");
        }

        let canon = to_lexical_absolute(path).unwrap();

        let path_chars: Vec<char> = canon.to_str().unwrap().chars().collect();

        self.matches_ex(0, &mut 0, &path_chars)
    }

    fn matches_ex(
        &self,
        mut pattern_idx: usize,
        text_idx: &mut usize,
        text: &Vec<char>,
    ) -> Result<bool, GlobError> {
        while pattern_idx < self.pattern_chars.len() && *text_idx < text.len() {
            match self.pattern_chars[pattern_idx] {
                '*' => {
                    if self
                        .matches_ex(pattern_idx + 1, text_idx, text)
                        .is_ok_and(|x| x)
                    {
                        return Ok(true);
                    }
                    *text_idx += 1;
                }
                '[' => {
                    pattern_idx += 1;
                    let mut matched = false;
                    while pattern_idx < self.pattern_chars.len()
                        && *text_idx < text.len()
                        && self.pattern_chars[pattern_idx] != ']'
                    {
                        if self.pattern_chars[pattern_idx] == text[*text_idx] {
                            matched = true;
                            *text_idx += 1;
                        }
                        pattern_idx += 1;
                    }

                    if !matched {
                        return Ok(false);
                    }

                    while self.pattern_chars[pattern_idx] != ']' {
                        pattern_idx += 1;
                    }

                    pattern_idx += 1;
                }
                ']' => {
                    //TODO: return err
                    panic!("Standalone ']' is not allowed!");
                }
                '?' => {
                    pattern_idx += 1;
                    *text_idx += 1;
                }
                _ => {
                    if self.pattern_chars[pattern_idx] != text[*text_idx] {
                        return Ok(false);
                    }
                    pattern_idx += 1;
                    *text_idx += 1;
                }
            }
        }

        let have_pattern_left = pattern_idx < self.pattern_chars.len();
        let have_text_left = *text_idx < text.len();

        if !have_pattern_left && !have_text_left {
            return Ok(true);
        }

        if have_text_left {
            if pattern_idx < self.pattern_chars.len() {
                while self.pattern_chars[pattern_idx] == '*' {
                    pattern_idx += 1;
                }
                if pattern_idx >= self.pattern_chars.len() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn new(pattern: &'a str, path: &'a PathBuf) -> Self {
        let is_wildcard = pattern.contains('*') || pattern.contains('?') || pattern.contains('[');
        let components: Vec<&str> = pattern.split('/').collect();

        let mut queque: VecDeque<PathEntry> = VecDeque::new();

        if path.is_file() {
            queque.push_back(PathEntry::File(path.clone()));
        }

        if path.is_dir() {
            let iter = fs::read_dir(path).expect(&format!(
                "Failed to read directory: '{}'",
                path.to_str().unwrap()
            ));
            queque.push_back(PathEntry::Dir(iter));
        }

        Self {
            pattern_chars: pattern.chars().collect(),
            is_wildcard,
            components,
            path,
            entries_to_process: queque,
        }
    }
}

impl<'a> Iterator for Paths<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        let mut to_append: VecDeque<PathEntry> = VecDeque::new();
        loop {
            let mut current_entry = self.entries_to_process.pop_back()?;
            match &mut current_entry {
                PathEntry::File(file_path) => match self.matches(file_path) {
                    Ok(matched) => {
                        if matched {
                            return Some(file_path.clone());
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err.msg);
                        return None;
                    }
                },
                PathEntry::Dir(dir_iter) => match dir_iter.next() {
                    Some(entry) => {
                        to_append.push_back(current_entry);
                        if let Ok(x) = entry {
                            let meta = x.metadata().expect("Cannot read metadata of: '{}'");
                            if meta.is_file() {
                                to_append.push_back(PathEntry::File(x.path()));
                            }
                            if meta.is_dir() {
                                let iter = fs::read_dir(x.path()).expect(&format!(
                                    "Failed to read directory: '{}'",
                                    x.path().to_str().unwrap()
                                ));
                                to_append.push_back(PathEntry::Dir(iter));
                            }
                        }
                    }
                    None => {}
                },
            }
            self.entries_to_process.append(&mut to_append);
        }
    }
}

pub fn glob<'a>(pattern: &'a str, path: &'a PathBuf) -> Result<Paths<'a>, GlobError> {
    if !path.exists() {
        return Err(GlobError {
            msg: format!("Path: '{}' does not exist!", path.to_str().unwrap()),
        });
    }

    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        match c {
            '[' => {
                if chars.find(|v| *v == ']').is_none() {
                    return Err(GlobError {
                        msg: format!("Invalid pattern, '[' needs a matching brace"),
                    });
                }
            }
            _ => {}
        }
    }

    let paths = Paths::new(pattern, path);

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_returns_error_on_invalid_pattern() {
        let x = PathBuf::from("..\\..\\test_files");
        let result = glob("*.[abc", &x);

        assert!(result.is_err());
    }

    #[test]
    fn glob_matches_given_extentions() {
        let result: Vec<PathBuf> = glob("*.[abc]", &PathBuf::from("..\\..\\test_files"))
            .unwrap()
            .into_iter()
            .collect();

        let result_string: Vec<&str> = result.iter().map(|p| p.to_str().unwrap()).collect();

        assert_eq!(
            result_string,
            vec![
                "..\\..\\test_files\\ext\\file.a",
                "..\\..\\test_files\\ext\\file.b",
                "..\\..\\test_files\\ext\\file.c"
            ]
        );
    }

    #[test]
    fn glob_exact_match() {
        let result: Vec<PathBuf> = glob("f.h", &PathBuf::from("..\\..\\test_files"))
            .unwrap()
            .into_iter()
            .collect();

        let result_string: Vec<&str> = result.iter().map(|p| p.to_str().unwrap()).collect();

        assert_eq!(result_string, vec!["..\\..\\test_files\\nested\\f.h"]);
    }

    #[test]
    fn glob_question_mark_skipes_two_chars() {
        let result: Vec<PathBuf> = glob("a??a", &PathBuf::from("..\\..\\test_files"))
            .unwrap()
            .into_iter()
            .collect();

        let result_string: Vec<&str> = result.iter().map(|p| p.to_str().unwrap()).collect();

        assert_eq!(
            result_string,
            vec!["..\\..\\test_files\\abba", "..\\..\\test_files\\acca"]
        );
    }

    #[test]
    fn glob_question_mark_skipes_one_chars() {
        let result: Vec<PathBuf> = glob("a????", &PathBuf::from("..\\..\\test_files"))
            .unwrap()
            .into_iter()
            .collect();

        let result_string: Vec<&str> = result.iter().map(|p| p.to_str().unwrap()).collect();

        assert_eq!(result_string, vec!["..\\..\\test_files\\a.txt"]);
    }

    #[test]
    fn glob_print_only_h_files() {
        let result: Vec<PathBuf> = glob("*.h", &PathBuf::from("..\\..\\test_files"))
            .unwrap()
            .into_iter()
            .collect();

        let result_string: Vec<&str> = result.iter().map(|p| p.to_str().unwrap()).collect();
        assert_eq!(result_string, vec!["..\\..\\test_files\\nested\\f.h"]);
    }
}
