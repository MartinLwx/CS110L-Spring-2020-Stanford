use regex::Regex;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];

    let open_file = File::open(filename).expect("File is invalid");
    let reader = BufReader::new(open_file);
    // WARNING: the `.lines()` method will drop newline automatically
    let mut lines: Vec<String> = reader
        .lines()
        .map(|line| line.expect("Parse line failed"))
        .collect();

    // a simple solution, we only keep the alphabetic chars
    let non_alphabetic = Regex::new(r"[^a-zA-Z]+").unwrap();
    let mut words = 0;
    let mut chars = 0;
    for line in lines.iter_mut() {
        chars += line.len();
        let parsed_line = non_alphabetic.replace_all(line.as_str(), " ");
        words += parsed_line
            .split(' ')
            .filter(|x| x.len() > 0) // it may contains prefix/suffix whitespace
            .collect::<Vec<_>>()
            .len();
    }

    println!("The number of lines: {}", lines.len());
    println!("The number of words: {:?}", words);
    println!("The number of chars: {}", chars);
}
