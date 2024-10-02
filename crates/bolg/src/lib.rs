use std::{
    collections::VecDeque,
    ffi::OsStr,
    fs::{self, ReadDir},
    path::{Iter, Path, PathBuf},
};

#[derive(Debug)]
pub struct InvalidPattern {
    pub msg: String,
}

#[derive(Debug)]
pub struct GlobErr {}

#[derive(Debug)]
pub enum PathEntry {
    File(PathBuf),
    Dir(ReadDir),
}

#[derive(Debug)]
pub struct Paths<'a> {
    pattern: &'a str,
    components: Vec<&'a str>,
    path: &'a PathBuf,
    is_wildcard: bool,
    entries_to_process: VecDeque<PathEntry>,
}

impl<'a> Paths<'a> {
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

        println!("{:?}", queque);
        Self {
            pattern,
            is_wildcard,
            components,
            path,
            entries_to_process: queque,
        }
    }
}

impl<'a> Iterator for Paths<'a> {
    type Item = Result<PathBuf, GlobErr>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pop_last = false;
        let mut next_item: Option<Self::Item> = None;
        loop {
            let current_entry = self.entries_to_process.back_mut();

            if current_entry.is_none() {
                return None;
            }

            let current_entry = current_entry.unwrap();

            match current_entry {
                PathEntry::File(file_path) => {
                    pop_last = true;
                    next_item = Some(Ok(file_path.clone()));
                    break;
                }
                PathEntry::Dir(dir_iter) => match dir_iter.next() {
                    Some(entry) => {
                        if let Ok(x) = entry {
                            let meta = x.metadata().expect("Cannot read metadata of: '{}'");

                            if meta.is_file() {
                                self.entries_to_process.push_back(PathEntry::File(x.path()));
                            }
                            if meta.is_dir() {
                                let iter = fs::read_dir(x.path()).expect(&format!(
                                    "Failed to read directory: '{}'",
                                    x.path().to_str().unwrap()
                                ));
                                self.entries_to_process.push_back(PathEntry::Dir(iter));
                            }
                        }
                    }
                    None => {
                        self.entries_to_process.pop_back().unwrap();
                    }
                },
            }
        }
        if pop_last {
            self.entries_to_process.pop_back();
        }

        next_item
    }
}

pub fn glob<'a>(pattern: &'a str, path: &'a PathBuf) -> Result<Paths<'a>, InvalidPattern> {
    if !path.exists() {
        return Err(InvalidPattern {
            msg: format!("Path: '{}' does not exist!", path.to_str().unwrap()),
        });
    }

    let paths = Paths::new(pattern, path);

    Ok(paths)
}
