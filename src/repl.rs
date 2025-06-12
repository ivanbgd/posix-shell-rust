//! REPL (Read-Eval-Print Loop)
//!
//! The main shell loop.
//!
//! Takes user input, parses it and calls the appropriate command or program handlers.
//!
//! # References
//!
//! - [REPL @ Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)
//! - [Bash Reference Manual](https://www.gnu.org/software/bash/manual/html_node/)

use crate::cmd::run_program;
use crate::constants::{Handler, COMMANDS, DEBUG, HANDLERS, PROMPT};
use crate::parse::parse_input;
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::zip;

/// The main shell loop.
pub fn repl() {
    loop {
        // Print prompt
        print!("{PROMPT}");
        io::stdout().flush().expect("Flush failed");

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Read line failed");

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        parse_input_and_handle_cmd(input);
    }
}

/// Parses user input and calls the appropriate command or program handler
fn parse_input_and_handle_cmd(input: &str) {
    let handlers = get_handlers();

    let items = match parse_input(input) {
        Ok(items) => items,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let items = items
        .iter()
        .map(|item| item.as_str())
        .collect::<Vec<&str>>();

    let cmd = items[0].trim();
    let args = if items.len() > 1 { &items[1..] } else { &[] };

    if DEBUG {
        eprintln!("cmd: {cmd:?}");
        eprintln!("args: {args:?}");
    }

    match handlers.get(cmd) {
        Some(&handler) => handler(args),
        None => run_program(cmd, args),
    }
}

/// Builds a table of command handlers and returns it
fn get_handlers<'a>() -> HashMap<&'a str, Handler> {
    let pairs: [(&str, Handler); COMMANDS.len()] = zip(COMMANDS, HANDLERS)
        .collect::<Vec<_>>()
        .try_into()
        .expect("Failed to convert vector to array");
    HashMap::from(pairs)
}
