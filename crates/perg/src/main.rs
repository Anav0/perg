use bolg::glob;
use clap::{command, Parser};
use futures::executor::{block_on, ThreadPool};
use futures::future::join_all;
use futures::task::SpawnExt;
use lazy_static::lazy_static;
use nfa::{FileMatch, NfaOptions, NFA};
use re::regex_to_nfa;
use std::{collections::HashSet, fs, path::PathBuf};

mod misc;
mod nfa;
mod re;

macro_rules! debug_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}

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

#[derive(Clone, Parser, Debug)]
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
    context: u32,

    #[arg(short = 'g', long, default_values_t = Vec::<String>::new(), num_args=0..)]
    glob: Vec<String>,

    #[arg()]
    path: String,
}

async fn find_matches_in_files(chunk: Vec<PathBuf>, args: Args, options: NfaOptions) -> Vec<FileMatch> {
    let nfa = regex_to_nfa(&args.pattern, &options);
    let mut output: Vec<FileMatch> = vec![];
    for file_path in chunk {
        if let Ok(m) = fs::metadata(&file_path) {
            if m.is_dir() {
                continue;
            }
            let input = fs::read_to_string(&file_path).expect(&format!(
                "Failed to read input file: '{}'",
                file_path.to_str().unwrap()
            ));
            let matches = nfa.find_matches(&input);
            let file_match = FileMatch {
                file_path: Some(PathBuf::from(file_path)),
                matches,
            };
            output.push(file_match);
        }
    }
    output
}

fn main() {
    let executor = ThreadPool::new().unwrap();
    let args = Args::parse();

    let path = PathBuf::from(&args.path);

    let options = NfaOptions::from(&args);

    let number_of_available_threads =
        std::thread::available_parallelism().expect("Cannot determin number of CPU cores");

    let mut files = vec![];
    for pattern in &args.glob {
        let mut matched_files = glob(pattern, &path)
            .expect("Cannot perform glob search")
            .collect::<Vec<_>>();
        files.append(&mut matched_files);
    }

    let mut chunk_size = files.len() / number_of_available_threads;

    if files.len() < number_of_available_threads.get() {
        chunk_size = files.len();
    }

    if chunk_size == 0 {
        return;
    }

    debug_println!(
        "Threads: {}, Files matched: {}, Chunk size: {}",
        number_of_available_threads,
        files.len(),
        chunk_size
    );

    let mut handles = vec![];
    for chunk in files.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let fut = find_matches_in_files(chunk, args.clone(), options.clone());
        let handle = executor.spawn_with_handle(fut).expect("Failed to spawn thread");
        handles.push(handle);
    }

    let results = block_on(join_all(handles));

    for matches in results {
        if args.count {
            for m in matches {
                m.print_count();
            }
        } else {
            for m in matches {
                m.print_matches(&options);
            }
        }
    }
}
