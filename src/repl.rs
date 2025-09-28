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
    Handler, COMMANDS, DEBUG, FAILED_FLUSH_TO_STDERR, FAILED_FLUSH_TO_STDOUT,
    FAILED_READ_LINE, FAILED_WRITE_TO_STDERR, FAILED_WRITE_TO_STDOUT, HANDLERS, PROMPT, TEST,
};
use crate::parse::{parse_input, RedirectionMode, Redirections};
use crate::test_to_break_or_continue;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Stderr, Stdout, Write};
use std::iter::zip;
use std::path::PathBuf;
use std::{fs, mem};

/// The main shell loop.
pub fn repl() {
    get_debug();
    get_test();
    eprintln!("DEBUG = {:?}, TEST = {:?}", DEBUG, TEST); // todo rem

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        // Print prompt
        stdout.write_all(PROMPT).expect(FAILED_WRITE_TO_STDOUT);
        stdout.flush().expect(FAILED_FLUSH_TO_STDOUT);

        // Wait for user input
        let mut input = String::new();
        stdin.read_line(&mut input).expect(FAILED_READ_LINE);

        let input = input.trim();

        if input.is_empty() {
            test_to_break_or_continue!();
            // test_to_break_or_continue!(TEST.get().is_some_and(|&test| test));
            // todo rem
            // if TEST.get().is_some_and(|&test| test) {
            //     break;
            // } else {
            //     continue;
            // }
            // continue;
        }

        parse_input_and_handle_cmds(&mut stdout, &mut stderr, input);
    }
}

/// Parses user input and calls the appropriate command or program handler
fn parse_input_and_handle_cmds(stdout: &mut Stdout, stderr: &mut Stderr, input: &str) {
    let handlers = get_handlers();

    let (items, redirections) = match parse_input(input) {
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

    let mut output = match handlers.get(cmd) {
        Some(&handler) => handler(args),
        None => run_program(cmd, args),
    };

    // mem::swap(&mut output.stdout, &mut output.stderr); // TODO

    if DEBUG.get().is_some_and(|&debug| debug) {
        eprintln!("cmd: {cmd:?}");
        eprintln!("args: {args:?}");
        eprintln!("redirections: {redirections:?}");
        eprintln!("output: {output}");
        eprintln!();
    }

    // TODO rem
    // for redirect in redirections {
    //     handle_redirection(stdout, stderr, redirect, &output);
    // }

    handle_redirections(stdout, stderr, redirections, output);

    stdout.flush().expect(FAILED_FLUSH_TO_STDOUT);
    stderr.flush().expect(FAILED_FLUSH_TO_STDERR);
}

/// Handle redirections
fn handle_redirections(
    stdout: &mut Stdout,
    stderr: &mut Stderr,
    redirections: Redirections,
    output: Output,
) {
    let (mut stdout_redir, mut stderr_redir) = (redirections.stdout, redirections.stderr);
    let (mut stdout_data, mut stderr_data) = output.get();

    eprintln!(
        "01a stdout_redir: {:?}, stderr_redir: {:?}",
        &stdout_redir, &stderr_redir
    );
    eprintln!(
        "01b stdout_data: {:?}, stderr_data: {:?}",
        String::from_utf8_lossy(&stdout_data),
        String::from_utf8_lossy(&stderr_data)
    );

    let mut stdout_targets = stdout_redir.clone().paths; // todo rem clone()
    let mut stderr_targets = stderr_redir.clone().paths; // todo rem clone()

    // if stdout_redir.kind != RedirectionMode::None
    //     && !stdout_targets.is_empty()
    //     && stdout_targets
    //         .last()
    //         .expect("Expected last stdout target")
    //         .clone()
    //         .into_os_string()
    //         .is_empty()
    // if stdout_redir.kind != RedirectionMode::None && stdout_targets.is_empty() {
    if stdout_redir.kind != RedirectionMode::None
        && (stdout_targets.is_empty()
            || !stdout_targets.is_empty()
                && stdout_targets
                    .last()
                    .expect("Expected last stdout target")
                    .clone()
                    .into_os_string()
                    .is_empty())
    {
        // TODO: Should I also swap here?!
        eprintln!("here: stdout_redir");
        stdout_redir.kind = RedirectionMode::None;
    }

    // if stderr_redir.kind != RedirectionMode::None
    //     && !stderr_targets.is_empty()
    //     && stderr_targets
    //         .last()
    //         .expect("Expected last stderr target")
    //         .clone()
    //         .into_os_string()
    //         .is_empty()
    // if stderr_redir.kind != RedirectionMode::None && stderr_targets.is_empty() {
    if stderr_redir.kind != RedirectionMode::None
        && (stderr_targets.is_empty()
            || !stderr_targets.is_empty()
                && stderr_targets
                    .last()
                    .expect("Expected last stderr target")
                    .clone()
                    .into_os_string()
                    .is_empty())
    {
        // TODO: This is probably good, so don't touch it.
        eprintln!("Swap stderr_redir!");
        mem::swap(&mut stdout_data, &mut stderr_data);
        mem::swap(&mut stdout_targets, &mut stderr_targets);
        stdout_redir.kind = stderr_redir.kind;
        stderr_redir.kind = RedirectionMode::None;
    }

    eprintln!("02a stdout_redir: {stdout_redir:?}, stderr_redir: {stderr_redir:?}");
    eprintln!(
        "02b stdout_data: {:?}, stderr_data: {:?}",
        String::from_utf8_lossy(&stdout_data),
        String::from_utf8_lossy(&stderr_data)
    );

    stdout_targets = stdout_redir.paths;
    stderr_targets = stderr_redir.paths;

    eprintln!("STDOUT");
    // let stdout_targets = stdout_redir.paths.expect("Expected targets"); todo remove
    // let stdout_targets = stdout_redir.paths; //.unwrap_or_default(); // todo clean up
    // eprintln!("* {stdout_targets:?}, {}", stdout_targets.len()); // todo remove

    // Truncate all but the last target file to zero size.
    for target in &stdout_targets[..stdout_targets.len().saturating_sub(1)] {
        eprintln!("HERE 1a");
        eprintln!("* {}", target.display()); // todo remove
        if target.clone().into_os_string().is_empty() {
            eprintln!("HERE 2a");
            continue;
        }
        if let Err(err) = File::create(target) {
            eprintln!("{err}: Failed to create the file '{}'", &target.display());
        }
    }
    // eprintln!("--------"); // todo remove

    match stdout_redir.kind {
        RedirectionMode::None => stdout
            .write_all(&stdout_data)
            .expect(FAILED_WRITE_TO_STDOUT),
        RedirectionMode::Overwrite => {
            // let last_stdout_target = stdout_targets.last().expect("Expected last stdout target");
            // write_redirected(&stdout_data, last_stdout_target, false);

            if let Some(last_stdout_target) = stdout_targets.last() {
                write_redirected(&stdout_data, last_stdout_target, false);
            }
        }
        RedirectionMode::Append => {
            // let last_stdout_target = stdout_targets.last().expect("Expected last stdout target");
            // write_redirected(&stdout_data, last_stdout_target, true);

            if let Some(last_stdout_target) = stdout_targets.last() {
                write_redirected(&stdout_data, last_stdout_target, true);
            }
        }
    }

    eprintln!("STDERR");
    // let stderr_targets = stderr_redir.paths.expect("Expected targets"); todo remove
    // let stderr_targets = stderr_redir.paths; //.unwrap_or_default(); // todo clean up

    // Truncate all but the last target file to zero size.
    for target in &stderr_targets[..stderr_targets.len().saturating_sub(1)] {
        eprintln!("HERE 1b");
        eprintln!("* {}", target.display()); // todo remove
        if target.clone().into_os_string().is_empty() {
            eprintln!("HERE 2b");
            continue;
        }
        if let Err(err) = File::create(target) {
            eprintln!("{err}: Failed to create the file '{}'", &target.display());
        }
    }

    // eprintln!("HERE 3: {:?}", stderr_targets.last().expect("Expected a target"));
    // let last_stderr_target = stderr_targets.last().expect("Expected last stderr target");
    // stderr
    //     .write_all(&stderr_data)
    //     .expect(FAILED_WRITE_TO_STDERR);

    match stderr_redir.kind {
        RedirectionMode::None => stderr
            .write_all(&stderr_data)
            .expect(FAILED_WRITE_TO_STDERR),
        RedirectionMode::Overwrite => {
            // let last_stderr_target = stderr_targets.last().expect("Expected last stderr target");
            // write_redirected(&stderr_data, last_stderr_target, false);

            if let Some(last_stderr_target) = stderr_targets.last() {
                write_redirected(&stderr_data, last_stderr_target, false);
            }
        }
        RedirectionMode::Append => {
            // let last_stderr_target = stderr_targets.last().expect("Expected last stderr target");
            // write_redirected(&stderr_data, last_stderr_target, true);

            if let Some(last_stderr_target) = stderr_targets.last() {
                write_redirected(&stderr_data, last_stderr_target, true);
            }
        }
    }

    // TODO remove
    // match redirect {
    //     Redirect::None => {
    //         stdout
    //             .write_all(&output.stdout)
    //             .expect(FAILED_WRITE_TO_STDOUT);
    //         stderr
    //             .write_all(&output.stderr)
    //             .expect(FAILED_WRITE_TO_STDERR);
    //     }
    //     Redirect::Stdout(target) => {
    //         stderr
    //             .write_all(&output.stderr)
    //             .expect(FAILED_WRITE_TO_STDERR);
    //         write_redirected(&output.stdout, &target, false);
    //     }
    //     Redirect::Stderr(target) => {
    //         stdout
    //             .write_all(&output.stdout)
    //             .expect(FAILED_WRITE_TO_STDOUT);
    //         write_redirected(&output.stderr, &target, false);
    //     }
    //     Redirect::AppendStdout(target) => {
    //         stderr
    //             .write_all(&output.stderr)
    //             .expect(FAILED_WRITE_TO_STDERR);
    //         write_redirected(&output.stdout, &target, true);
    //     }
    //     Redirect::AppendStderr(target) => {
    //         stdout
    //             .write_all(&output.stdout)
    //             .expect(FAILED_WRITE_TO_STDOUT);
    //         write_redirected(&output.stderr, &target, true);
    //     }
    //     Redirect::CombinedStdout(target) => {
    //         write_redirected(&output.stdout, &target, false);
    //         write_redirected(&output.stderr, &target, true);
    //     }
    //     Redirect::AppendCombinedStdout(target) => {
    //         write_redirected(&output.stdout, &target, true);
    //         write_redirected(&output.stderr, &target, true);
    //     }
    //     Redirect::CombinedStderr(target) | Redirect::AppendCombinedStderr(target) => {
    //         stdout
    //             .write_all(&output.stdout)
    //             .expect(FAILED_WRITE_TO_STDOUT);
    //         stderr
    //             .write_all(&output.stderr)
    //             .expect(FAILED_WRITE_TO_STDERR);
    //         if !target.is_empty() {
    //             if let Err(err) = File::create(&target) {
    //                 eprintln!("{err}: Failed to create the file '{target}'");
    //             }
    //         }
    //     }
    // }
}

/// Helper for writing redirected output to the given target file
///
/// # References
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
/// - [Appending Redirected Output](https://www.gnu.org/software/bash/manual/bash.html#Appending-Redirected-Output)
fn write_redirected(output: &[u8], target: &PathBuf, append: bool) {
    if !append {
        if let Err(err) = fs::write(target, output) {
            eprintln!("{err}: Failed to write to file '{}'", target.display());
        }
    } else {
        let mut file = match OpenOptions::new().append(true).create(true).open(target) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{err}: Failed to open the file '{}'", target.display());
                return;
            }
        };
        if let Err(err) = file.write_all(output) {
            eprintln!("{err}: Failed to append to file '{}'", target.display());
        }
        if let Err(err) = file.flush() {
            eprintln!("{err}: Failed to flush the file '{}'", target.display());
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

/// Copies the value of the environment variable `DEBUG`, if it exists, to the global variable [`DEBUG`],
/// and if it doesn't exist, sets the global variable [`DEBUG`] to `false`.
///
/// If the environment variable `DEBUG` exists, it must hold string `true` for the global variable [`DEBUG`]
/// to be set; otherwise, the global variable [`DEBUG`] will be reset.
///
/// This is only considered during **compile time**, and **not** during run time.
fn get_debug() {
    let key: Option<&'static str> = option_env!("DEBUG");

    let debug: bool = if let Some(key) = key {
        key.trim().parse().unwrap_or_default()
    } else {
        false
    };

    DEBUG.get_or_init(|| debug);
}

/// Copies the value of the environment variable `TEST`, if it exists, to the global variable [`TEST`],
/// and if it doesn't exist, sets the global variable [`TEST`] to `false`.
///
/// If the environment variable `TEST` exists, it must hold string `true` for the global variable [`TEST`]
/// to be set; otherwise, the global variable [`TEST`] will be reset.
///
/// This is only considered during **compile time**, and **not** during run time.
fn get_test() {
    let key: Option<&'static str> = option_env!("TEST");

    let debug: bool = if let Some(key) = key {
        key.trim().parse().unwrap_or_default()
    } else {
        false
    };

    TEST.get_or_init(|| debug);
}
