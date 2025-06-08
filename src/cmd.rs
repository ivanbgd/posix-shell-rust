//! Command handlers

use crate::constants::COMMANDS;
use std::env;

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
pub fn handle_type(arg: Option<&str>) {
    if let Some(arg) = arg {
        if COMMANDS.contains(&arg.as_bytes()) {
            println!("{arg} is a shell builtin");
        } else {
            let key = "PATH";
            let path = match env::var(key) {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("{key} not found");
                    return;
                }
            };
            let paths = env::split_paths(&path);

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
