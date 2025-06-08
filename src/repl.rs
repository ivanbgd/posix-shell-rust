//! REPL (Read-Eval-Print Loop)
//!
//! The main shell loop.
//!
//! [Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)

use crate::cmd::{handle_echo, handle_exit};
use std::io::{self, Write};
// use std::process::exit;

/// The main shell loop.
pub fn repl() {
    loop {
        // Print prompt
        print!("$ ");
        io::stdout().flush().expect("Flush failed");

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Read line failed");

        let cmd = input.trim();

        if cmd.is_empty() {
            continue;
        }

        parse_and_handle_cmd(cmd);

        // if cmd.starts_with("exit") {
        //     // if let Ok(code) = handle_exit(cmd.split_once(" ").unwrap_or(("exit", "0")).1) {
        //     if let Ok(code) = handle_exit(cmd) {
        //         exit(code);
        //     }
        // } else {
        //     println!("{}: command not found", cmd);
        // }
    }
}

/// Parses command and calls the appropriate command handler
fn parse_and_handle_cmd(cmd: &str) {
    let (cmd, args) = match cmd.split_once(" ") {
        Some((cmd, args)) => (cmd, Some(args)),
        None => (cmd, None),
    };

    let cmd = cmd.to_ascii_lowercase();

    match cmd.as_str() {
        "echo" => handle_echo(args),
        "exit" => handle_exit(args),
        cmd => println!("{}: command not found", cmd),
    }
}
