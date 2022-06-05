use std::io;
use std::io::*;


fn read_from_user() -> Vec<String> {
    let mut shopping_list = Vec::new();
    loop {
        print!("Enter an item to add to the list: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.to_lowercase() == "done\n" {
                    break;
                }
                shopping_list.push(input.strip_suffix("\n").unwrap().to_string());
            },
            Err(e) => println!("Something goes wront: {}", e),
        }
    }
    shopping_list
}
        
fn print_items(shopping_list: &Vec<String>) {
    println!("Remember to buy:");
    for item in shopping_list {
        println!("* {}", item);
    }
}

fn main() {
    let shopping_list = read_from_user();
    print_items(&shopping_list);
}
