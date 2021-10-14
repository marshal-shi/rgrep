use std::fmt::Debug;
use std::path::{Path, PathBuf};

use clap::{App, Arg};
use glob::glob;
use regex::Regex;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt};

fn contains(line: &str, r: Regex) -> bool {
    r.is_match(line)
}

async fn read_lines<P>(
    filename: P,
) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path> + Debug,
{
    let file = File::open(filename).await?;
    Ok(io::BufReader::new(file).lines())
}

async fn find_matches<P>(word: &str, fname: P)
where
    P: AsRef<Path> + Debug,
{
    println!("{:?}:", fname);
    let re = Regex::new(word).expect("Cannot regex");
    let mut lines = read_lines(fname).await.unwrap();
    let mut line_count: u32 = 1;
    while let Some(line) = lines.next_line().await.unwrap() {
        if contains(&line, re.clone()) {
            let rs = re.replace_all(&line, "\x1b[0;31m$0\x1b[0m");
            println!("  {}: {}", line_count, rs);
        }
        line_count += 1;
    }
}

fn find_all_files(fname: &str) -> Vec<Box<PathBuf>> {
    glob(fname)
        .unwrap()
        .into_iter()
        .filter(|entry| entry.is_ok())
        .map(|entry| Box::new(entry.unwrap()))
        .collect::<Vec<_>>()
}

#[tokio::main]
async fn main() {
    let matches = App::new("RGREP")
        .arg(Arg::with_name("WORD").required(true).index(1))
        .arg(Arg::with_name("FILE").required(true).index(2))
        .get_matches();

    let word = matches.value_of("WORD").unwrap();
    let fname = matches.value_of("FILE").unwrap();
    println!("{} {}", word, fname);

    let paths = find_all_files(fname);
    for path in paths {
        find_matches(word, *path).await
    }
}
