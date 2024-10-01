use clap::{command, Parser};
use nfa::FileMatch;
use re::regex_to_nfa;
use std::{env, fs, io::stdin, path::PathBuf, process};

mod nfa;
mod re;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', long)]
    ignore_case: bool,

    #[arg(short = 'f', long, default_value_t = String::new())]
    path_to_pattern_file: String,

    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    #[arg(short, long, default_value_t = false)]
    count: bool,

    #[arg(short = 'p')]
    pattern: String,

    #[arg(short = 'C', long, default_value_t = 1)]
    context: u8,

    #[arg(default_value_t = String::new())]
    input: String,
}

fn main() {
    let args = Args::parse();

    let mut input = args.input;
    if !args.path_to_pattern_file.is_empty() {
        input = fs::read_to_string(&args.path_to_pattern_file).expect("Failed to read input file");
    }

    let nfa = regex_to_nfa(&args.pattern);
    let matches = nfa.find_matches(&input);
    let file_match = FileMatch {
        file_path: Some(PathBuf::from(&args.path_to_pattern_file)),
        matches,
    };

    file_match.print_matches()
}
