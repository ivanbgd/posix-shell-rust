//! Command handlers

use crate::constants::COMMANDS;
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Handler for the `echo` command
pub fn handle_echo(arg: Option<&str>) {
    if let Some(arg) = arg {
        println!("{arg}");
    };
}

/// Handler for the `exit` command
pub fn handle_exit(arg: Option<&str>) {
    match arg {
        Some(arg) => match arg.trim().parse::<i32>() {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(_) => eprintln!("Invalid exit code: {arg}"),
        },
        None => std::process::exit(0),
    }
}

/// Handler for the `type` command
///
/// Searches for executable files using
/// the [PATH](https://en.wikipedia.org/wiki/PATH_(variable)) environment variable.
///
/// Some commands, such as `echo`, can exist as both builtin commands and executable files.
/// In such cases, the type command identifies them as builtins.
pub fn handle_type(arg: Option<&str>) {
    if let Some(arg) = arg {
        if COMMANDS.contains(&arg.as_bytes()) {
            println!("{arg} is a shell builtin");
        } else {
            let paths = get_paths();

            for path in paths {
                if path.join(arg).exists() {
                    println!("{arg} is {}", path.join(arg).display());
                    return;
                }
            }

            println!("{arg}: not found");
        }
    };
}

/// Runs external programs with arguments
///
/// External programs are located using
/// the [PATH](https://en.wikipedia.org/wiki/PATH_(variable)) environment variable.
pub fn run_program(exec: &str, args: Option<&str>) {
    let args = args.unwrap_or_default();
    let args = args.split_ascii_whitespace();

    let paths = get_paths();

    for path in paths {
        if path.join(exec).exists() {
            let output = Command::new(exec)
                .args(args)
                .output()
                .expect("Failed to execute command");
            print!("{}", String::from_utf8(output.stdout).unwrap());
            return;
        }
    }

    println!("{exec}: command not found");
}

/// A helper function which extracts directories from
/// the [PATH](https://en.wikipedia.org/wiki/PATH_(variable)) environment variable.
fn get_paths() -> Vec<PathBuf> {
    let key = "PATH";

    let path = match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            eprintln!("{key} not found");
            return vec![];
        }
    };

    let paths = env::split_paths(&path);

    paths.collect()
}
