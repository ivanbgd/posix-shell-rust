//! REPL (Read-Eval-Print Loop)
//!
//! [Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)

use std::io::{self, Write};

pub fn repl() {
    loop {
        // Print prompt
        print!("$ ");
        io::stdout().flush().expect("Flush failed");

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Read line failed");
        if input.trim().is_empty() {
            break;
        }
        println!("{}: command not found", input.trim());
    }
}
