use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct InvalidPattern {}

#[derive(Debug)]
pub struct GlobErr {}

#[derive(Debug)]
pub struct Paths {}

impl Iterator for Paths {
    type Item = Result<PathBuf, GlobErr>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn glob(pattern: &str) -> Result<Paths, InvalidPattern> {
    todo!()
}
