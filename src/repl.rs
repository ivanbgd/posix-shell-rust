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

use crate::cmd::{run_program, Output};
use crate::constants::{
    Handler, COMMANDS, DEBUG, FAILED_FLUSH, FAILED_READ_LINE,
    FAILED_WRITE_TO_STDERR, FAILED_WRITE_TO_STDOUT, HANDLERS, PROMPT,
};
use crate::parse::{parse_input, Redirect};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Stderr, Stdout, Write};
use std::iter::zip;

/// The main shell loop.
pub fn repl() {
    get_debug();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        // Print prompt
        stdout.write_all(PROMPT).expect(FAILED_WRITE_TO_STDOUT);
        stdout.flush().expect(FAILED_FLUSH);

        // Wait for user input
        let mut input = String::new();
        stdin.read_line(&mut input).expect(FAILED_READ_LINE);

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        parse_input_and_handle_cmd(&mut stdout, &mut stderr, input);
    }
}

/// Parses user input and calls the appropriate command or program handler
fn parse_input_and_handle_cmd(stdout: &mut Stdout, stderr: &mut Stderr, input: &str) {
    let handlers = get_handlers();

    let (items, redirect) = match parse_input(input) {
        Ok(items) => items,
        Err(error) => {
            write!(stderr, "{error}").expect(FAILED_WRITE_TO_STDERR);
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

    if DEBUG.get().is_some_and(|&debug| debug) {
        eprintln!("cmd: {cmd:?}");
        eprintln!("args: {args:?}");
        eprintln!("redirect: {redirect:?}");
        eprintln!("output: {output}");
        eprintln!();
    }

    handle_redirect(stdout, stderr, redirect, output);

    stdout.flush().expect(FAILED_FLUSH);
    stderr.flush().expect(FAILED_FLUSH);
}

/// Handle redirection
fn handle_redirect(stdout: &mut Stdout, stderr: &mut Stderr, redirect: Redirect, output: Output) {
    match redirect {
        Redirect::None => {
            stdout
                .write_all(&output.stdout)
                .expect(FAILED_WRITE_TO_STDOUT);
            stderr
                .write_all(&output.stderr)
                .expect(FAILED_WRITE_TO_STDERR);
        }
        Redirect::Stdout(target) => {
            stderr
                .write_all(&output.stderr)
                .expect(FAILED_WRITE_TO_STDERR);
            write_redirected(&output.stdout, &target, false);
        }
        Redirect::Stderr(target) => {
            stdout
                .write_all(&output.stdout)
                .expect(FAILED_WRITE_TO_STDOUT);
            write_redirected(&output.stderr, &target, false);
        }
        Redirect::AppendStdout(target) => {
            stderr
                .write_all(&output.stderr)
                .expect(FAILED_WRITE_TO_STDERR);
            write_redirected(&output.stdout, &target, true);
        }
        Redirect::AppendStderr(target) => {
            stdout
                .write_all(&output.stdout)
                .expect(FAILED_WRITE_TO_STDOUT);
            write_redirected(&output.stderr, &target, true);
        }
        Redirect::CombinedStdout(target) => {
            write_redirected(&output.stdout, &target, false);
            write_redirected(&output.stderr, &target, true);
        }
        Redirect::AppendCombinedStdout(target) => {
            write_redirected(&output.stdout, &target, true);
            write_redirected(&output.stderr, &target, true);
        }
        Redirect::CombinedStderr(target) | Redirect::AppendCombinedStderr(target) => {
            stdout
                .write_all(&output.stdout)
                .expect(FAILED_WRITE_TO_STDOUT);
            stderr
                .write_all(&output.stderr)
                .expect(FAILED_WRITE_TO_STDERR);
            if !target.is_empty() {
                if let Err(err) = File::create(&target) {
                    eprintln!("{err}: Failed to create the file '{target}'");
                }
            }
        }
    }
}

/// Helper for writing redirected output to the given target file
///
/// # References
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
/// - [Appending Redirected Output](https://www.gnu.org/software/bash/manual/bash.html#Appending-Redirected-Output)
fn write_redirected(output: &[u8], target: &str, append: bool) {
    if !append {
        if let Err(err) = fs::write(target, output) {
            eprintln!("{err}: Failed to write to file '{target}'");
        }
    } else {
        let mut file = match OpenOptions::new().append(true).create(true).open(target) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{err}: Failed to open the file '{target}'");
                return;
            }
        };
        if let Err(err) = file.write_all(output) {
            eprintln!("{err}: Failed to append to file '{target}'");
        }
        if let Err(err) = file.flush() {
            eprintln!("{err}: Failed to flush the file '{target}'");
        };
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

/// Copies the value of the environment variable `DEBUG`, if it exists, to the global variable `DEBUG`,
/// and if it doesn't exist, sets the global variable `DEBUG` to `false`.
///
/// If the environment variable `DEBUG` exists, it must hold string `true` for the global variable `DEBUG`
/// to be set; otherwise, the global variable `DEBUG` will be reset.
fn get_debug() {
    let key: Option<&'static str> = option_env!("DEBUG");

    let debug: bool = if let Some(key) = key {
        key.trim().parse().unwrap_or_default()
    } else {
        false
    };

    DEBUG.get_or_init(|| debug);
}
