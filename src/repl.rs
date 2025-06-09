//! REPL (Read-Eval-Print Loop)
//!
//! The main shell loop.
//!
//! Takes user input, parses it and calls the appropriate command or program handlers.
//!
//! # References
//!
//! - [REPL @ Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)
//! - Enclosing characters in single quotes preserves the literal value of each character within the quotes.
//!   [Single quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes)
//! - Enclosing characters in double quotes preserves the literal value of each character within the quotes except `\`.
//!   The backslash retains its special meaning when followed by `\`, `$`, `"` or newline.
//!   [Double quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes)

use crate::cmd::{handle_cd, handle_echo, handle_exit, handle_pwd, handle_type, run_program};
use crate::constants::STACK_SIZE;
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

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        parse_input_and_handle_cmd(input);
    }
}

/// Parses user input and calls the appropriate command or program handler
fn parse_input_and_handle_cmd(input: &str) {
    let mut items: Vec<String> = Vec::new();
    let mut item = String::new();
    let mut stack = [0u8; STACK_SIZE];
    let mut idx = 0usize;

    for ch in input.chars() {
        if ch.is_ascii_whitespace() {
            // Quoted text should keep all its whitespace characters, but unquoted text should not.
            // It should decrease several consecutive whitespace characters to a single space.
            if stack[idx.saturating_sub(1)] == b'\'' || stack[idx.saturating_sub(1)] == b'"' {
                item.push(ch);
            } else {
                if !item.is_empty() {
                    items.push(item.to_string());
                }
                item.clear();
            }
        } else if ch.eq(&'\'') {
            if stack[idx.saturating_sub(1)] == b'\'' {
                stack[idx.saturating_sub(1)] = 0;
                idx -= 1;
            } else {
                stack[idx.saturating_sub(1)] = ch as u8;
                idx += 1;
            }
        } else if ch.eq(&'"') {
        } else {
            item.push(ch);
        }
    }
    items.push(item.trim().to_string());

    if idx != 0 {
        eprintln!("Unmatched quotes: {stack:?}");
        return;
    }

    let cmd = &items[0];
    let args = &items[1..];

    match cmd.trim() {
        "cd" => handle_cd(args),
        "echo" => handle_echo(args),
        "exit" => handle_exit(args),
        "pwd" => handle_pwd(),
        "type" => handle_type(args),
        exec => run_program(exec, args),
    }
}
