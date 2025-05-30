//! A POSIX-Compliant Shell (CLI) Implementation in Rust

use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().expect("Flush failed");

    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Read line failed");
    println!("{}: command not found", input.trim())
}
