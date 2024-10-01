use clap::{command, Parser};
use re::regex_to_nfa;
use std::{env, fs, io::stdin, process};

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

    #[arg(default_value_t = String::new())]
    input: String,
}

fn match_pattern(input_line: &str, raw_pattern: &str) -> bool {
    let nfa = regex_to_nfa(raw_pattern);

    nfa.find_match(input_line)
}

fn main() {
    let args = Args::parse();

    let mut input = args.input;
    if !args.path_to_pattern_file.is_empty() {
        input = fs::read_to_string(args.path_to_pattern_file).expect("Failed to read input file");
    }

    if match_pattern(&input, &args.pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
