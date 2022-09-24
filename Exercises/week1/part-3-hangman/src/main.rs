// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 10;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn print_per_guess(so_far: &String, current_guess: &String, remains: u32) {
    println!("The word so far is {}", so_far);
    println!("You have guessed the following letters: {}", current_guess);
    println!("You have {} guesses left", remains);
    print!("Please guess a letter: ");
    io::stdout().flush().unwrap();
}

fn update_so_far(so_far: &mut String, target: &String, ch: char) -> String {
    let so_far_chs: Vec<char> = so_far.chars().collect();
    let target_chs: Vec<char> = target.chars().collect();
    let mut mut_so_far = String::new();
    for pos in 0..target.len() {
        if target_chs[pos] == ch {
            mut_so_far.push(ch);
        } else {
            mut_so_far.push(so_far_chs[pos]);
        }
    }
    mut_so_far
}

fn main() {
    // Your code here! :)
    let secret_word = pick_a_random_word();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);
    let mut so_far = "-".repeat(secret_word.len());
    let mut current_guess = String::new();
    let mut try_cnt = 0;

    println!("Welcome to Guess the Word!");

    // gaming logic
    while try_cnt < NUM_INCORRECT_GUESSES {
        print_per_guess(&so_far, &current_guess, NUM_INCORRECT_GUESSES - try_cnt);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error!");
        let ch = input.to_string().chars().next().unwrap();
        current_guess.push(ch);
        let mut_so_far = update_so_far(&mut so_far, &secret_word, ch);
        if mut_so_far == so_far {
            println!("Sorry, that letter is not in the word")
        }
        so_far = mut_so_far;
        if so_far == secret_word {
            break;
        }
        println!();
        try_cnt += 1;
    }

    if so_far != secret_word {
        println!("Sorry, you ran out of guesses!");
    } else {
        println!(
            "Congratulations! you guessed the secret word: {}",
            secret_word
        );
    }
}
