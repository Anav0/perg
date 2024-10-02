use bolg::glob;
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

    #[arg(short = 'g', long, default_value_t = String::new())]
    glob: String,

    #[arg()]
    path: String,
}

fn main() {
    let args = Args::parse();

    let nfa = regex_to_nfa(&args.pattern);

    for entry in glob(&args.glob).expect("Invalid glob pattern") {
        if let Ok(file_path) = entry {
            if let Ok(m) = fs::metadata(&file_path) {
                if m.is_file() {
                    let input = fs::read_to_string(&file_path).expect(&format!(
                        "Failed to read input file: '{}'",
                        file_path.to_str().unwrap()
                    ));
                    let matches = nfa.find_matches(&input);
                    let file_match = FileMatch {
                        file_path: Some(PathBuf::from(file_path)),
                        matches,
                    };
                    file_match.print_matches();
                }
            }
        }
    }
}
