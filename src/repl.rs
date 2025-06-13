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
use crate::parse::{parse_input, Redirect};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
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

    let (items, target) = match parse_input(input) {
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

    let output = match handlers.get(cmd) {
        Some(&handler) => handler(args),
        None => run_program(cmd, args),
    };

    if DEBUG {
        eprintln!("cmd: {cmd:?}");
        eprintln!("args: {args:?}");
        eprintln!("target: {target:?}");
        eprintln!("output: {output:?}");
        eprintln!();
    }

    let is_output_ok = output.is_ok();
    let output = output.unwrap_or_else(|err| err.to_string());

    match target {
        Redirect::None => match is_output_ok {
            true => print!("{output}"),
            false => eprint!("{output}"),
        },
        Redirect::Stdout(target) | Redirect::Stderr(target) => redirect(&output, &target, false),
        Redirect::AppendStdout(target) | Redirect::AppendStderr(target) => {
            redirect(&output, &target, true)
        }
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

/// Helper for writing redirected output to the given target file
///
/// # References
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
/// - [Appending Redirected Output](https://www.gnu.org/software/bash/manual/bash.html#Appending-Redirected-Output)
fn redirect(output: &str, target: &str, append: bool) {
    if !append {
        if let Err(err) = fs::write(target, output) {
            eprintln!("{err}: Failed to write to file '{target}'");
        }
    } else {
        let mut file = match OpenOptions::new().append(true).open(target) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{err}: Failed to open the file '{target}'");
                return;
            }
        };
        if let Err(err) = write!(file, "{}", output) {
            eprintln!("{err}: Failed to write to file '{target}'");
        }
    }
}
