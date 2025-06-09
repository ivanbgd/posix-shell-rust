//! Command handlers

use crate::constants::{Args, COMMANDS};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Handler for the `cd` command
pub fn handle_cd(arg: Args) {
    if !arg.is_empty() {
        let arg = &arg[0];
        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => {
                eprintln!("HOME not found");
                return;
            }
        };
        let arg = if arg.eq(&"~") { home.as_str() } else { arg };

        if env::set_current_dir(arg).is_err() {
            println!("cd: {arg}: No such file or directory");
        }
    };
}

/// Handler for the `echo` command
pub fn handle_echo(args: Args) {
    if !args.is_empty() {
        let args = args.join(" ");
        println!("{args}");
    };
}

/// Handler for the `exit` command
pub fn handle_exit(arg: Args) {
    match arg.is_empty() {
        false => {
            let arg = &arg[0];
            match arg.trim().parse::<i32>() {
                Ok(exit_code) => std::process::exit(exit_code),
                Err(_) => eprintln!("Invalid exit code: {arg}"),
            }
        }
        true => std::process::exit(0),
    }
}

/// Handler for the `pwd` command
pub fn handle_pwd(_arg: Args) {
    match env::current_dir() {
        Ok(pwd) => println!("{}", pwd.display()),
        Err(err) => eprintln!("{err}"),
    }
}

/// Handler for the `type` command
///
/// Searches for executable files using the `PATH` environment variable.
///
/// Some commands, such as `echo`, can exist as both builtin commands and executable files.
/// In such cases, the type command identifies them as builtins.
pub fn handle_type(arg: Args) {
    if !arg.is_empty() {
        let arg = arg[0];
        if COMMANDS.contains(&arg) {
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
/// External programs are located using the `PATH` environment variable.
pub fn run_program(exec: &str, args: Args) {
    let args = args
        .iter()
        .map(|arg| arg.trim_matches('\''))
        .collect::<Vec<_>>();

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
