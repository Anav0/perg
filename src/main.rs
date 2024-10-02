use clap::{command, Parser};
use lazy_static::lazy_static;
use nfa::{FileMatch, NFA};
use re::regex_to_nfa;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

mod nfa;
mod re;

//TODO: determin if file is a text file by checking its contants
lazy_static! {
    pub static ref ALLOWED_EXT: HashSet<String> = {
        let mut m = HashSet::new();
        for ext in ["txt", "rs", "cpp", "hpp", "h", "json", "xml", "java", "py"] {
            m.insert(ext.to_string());
        }
        m
    };
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    ignore_case: bool,

    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    #[arg(short, long, default_value_t = false)]
    count: bool,

    #[arg(short = 'p')]
    pattern: String,

    #[arg(short = 'C', long, default_value_t = 1)]
    context: u8,

    #[arg()]
    path: String,
}

fn walk_and_print_matches(path: &Path, nfa: &mut NFA) {
    if let Ok(m) = fs::metadata(path) {
        if m.is_file() {
            if !path
                .extension()
                .is_some_and(|ext| ALLOWED_EXT.contains(ext.to_str().unwrap()))
            {
                return;
            }

            let input = fs::read_to_string(path).expect(&format!(
                "Failed to read input file: '{}'",
                path.to_str().unwrap()
            ));
            let matches = nfa.find_matches(&input);
            let file_match = FileMatch {
                file_path: Some(PathBuf::from(path)),
                matches,
            };
            file_match.print_matches();
        }

        if m.is_dir() {
            for file in fs::read_dir(path).expect("Failed to read files in provided path") {
                if let Ok(entry) = file {
                    walk_and_print_matches(&entry.path(), nfa);
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut nfa = regex_to_nfa(&args.pattern);

    walk_and_print_matches(Path::new(&args.path), &mut nfa);
}
