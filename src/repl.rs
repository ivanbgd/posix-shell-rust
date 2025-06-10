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
//!   [Single Quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes)
//! - Enclosing characters in double quotes preserves the literal value of each character within the quotes except `\`.
//!   The backslash retains its special meaning when followed by `\`, `$`, `"` or newline.
//!   [Double Quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes)
//! - A non-quoted backslash `\` is treated as an escape character.
//!   It preserves the literal value of the next character.
//!   [Escape Character](https://www.gnu.org/software/bash/manual/bash.html#Escape-Character)

use crate::cmd::run_program;
use crate::constants::{Handler, COMMANDS, HANDLERS, STACK_SIZE};
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::zip;

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
    let handlers = get_handlers();

    let items = parse_input(input);
    let items = items
        .iter()
        .map(|item| item.as_str())
        .collect::<Vec<&str>>();

    let cmd = items[0].trim();
    let args = if items.len() > 1 { &items[1..] } else { &[] };

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

/// Parses user input and returns parsed items
///
/// An item can be more than a single word if it was quoted in the input.
///
/// Conversely, two or more words from the input can be merged into a single word if they were separated
/// only by a matching pair of quotes in the input.
///
/// # Examples
///
/// ```shell
/// $   echo  hi   there,   'hello   world'  'hi''"there"'  "and""again"  "hello   world,   it's   me"   bye   bye
/// hi there, hello   world hi"there" andagain hello   world,   it's   me bye bye
/// ```
fn parse_input(input: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    let mut item = String::new();
    let mut stack = [0u8; STACK_SIZE];
    let mut idx = 0usize;

    for ch in input.chars() {
        // eprintln!("{ch} {:?} {idx} {item}", &stack[..16]);
        if ch.is_ascii_whitespace() {
            // Quoted text should keep all its whitespace characters, but unquoted text should not.
            // It should reduce several consecutive whitespace characters to a single space.
            if stack[idx.saturating_sub(1)] == b'\'' || stack[idx.saturating_sub(1)] == b'"' {
                item.push(ch);
            } else {
                if !item.is_empty() {
                    items.push(item.to_string());
                }
                item.clear();
            }
        } else if ch.eq(&'\'') {
            if stack[idx.saturating_sub(1)] == b'"' {
                item.push(ch);
            } else if stack[idx.saturating_sub(1)] == b'\'' {
                stack[idx.saturating_sub(1)] = 0;
                idx -= 1;
            } else {
                stack[idx.saturating_sub(1)] = ch as u8;
                idx += 1;
            }
        } else if ch.eq(&'"') {
            if stack[idx.saturating_sub(1)] == b'\'' {
                item.push(ch);
            } else if stack[idx.saturating_sub(1)] == b'"' {
                stack[idx.saturating_sub(1)] = 0;
                idx -= 1;
            } else {
                stack[idx.saturating_sub(1)] = ch as u8;
                idx += 1;
            }
        } else {
            item.push(ch);
        }
    }
    items.push(item.trim().to_string());

    items
}
