//! REPL (Read-Eval-Print Loop)
//!
//! The main shell loop.
//!
//! [Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)

use crate::cmd::{handle_echo, handle_exit, handle_type, run_program};
use std::io::{self, Write};

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
    }
}

/// Parses command and calls the appropriate command or program handler
fn parse_and_handle_cmd(cmd: &str) {
    let (cmd, args) = match cmd.split_once(" ") {
        Some((cmd, args)) => (cmd, Some(args)),
        None => (cmd, None),
    };

    match cmd {
        "echo" => handle_echo(args),
        "exit" => handle_exit(args),
        "type" => handle_type(args),
        exec => run_program(exec, args),
    }
}
