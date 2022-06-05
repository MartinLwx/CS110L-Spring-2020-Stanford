use std::io;
use std::io::*;
use parity_wordlist::random_phrase;

fn print_per_guess(so_far: &String, current_guess: &String, remains: i32) {
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
    // use parity_wordlist for generating a random word
    let target = random_phrase(1);
    let mut so_far = "-".repeat(target.len());
    let mut current_guess = String::new();
    // the word can be very long, so I increase the remains
    let mut remains = 10;

    println!("Welcome to Guess the Word!");
    
    // gaming logic
    while remains != 0 {
        print_per_guess(&so_far, &current_guess, remains);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error!"); 
        let ch = input.to_string().chars().next().unwrap();
        current_guess.push(ch);
        let mut_so_far = update_so_far(&mut so_far, &target, ch);
        if mut_so_far == so_far {
            println!("Sorry, that letter is not in the word")
        }
        so_far = mut_so_far;
        if so_far == target {
            break;
        }
        println!();
        remains -= 1;
    }

    if so_far != target {
        println!("Sorry, you ran out of guesses!");
    } else {
        println!("Congratulations you guessed the secret word: {}", target);
    }
}
