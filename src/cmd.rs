//! Command handlers

use crate::constants::{Args, COMMANDS};
use crate::errors::OutputError;
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Handler for the `cd` builtin
pub fn handle_cd(arg: Args) -> Result<String, OutputError> {
    if !arg.is_empty() {
        let arg = &arg[0];
        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => {
                return Err("HOME not found\n".into());
            }
        };
        let arg = if arg.eq(&"~") { home.as_str() } else { arg };

        if env::set_current_dir(arg).is_err() {
            return Err(format!("cd: {arg}: No such file or directory\n").into());
        }
    };

    Ok("".to_string())
}

/// Handler for the `echo` builtin
pub fn handle_echo(args: Args) -> Result<String, OutputError> {
    Ok(format!("{}\n", args.join(" ")))
}

/// Handler for the `exit` builtin
pub fn handle_exit(arg: Args) -> Result<String, OutputError> {
    match arg.is_empty() {
        false => {
            let arg = &arg[0];
            match arg.trim().parse::<i32>() {
                Ok(exit_code) => std::process::exit(exit_code),
                Err(_) => Err(format!("Invalid exit code: {arg}\n").into()),
            }
        }
        true => std::process::exit(0),
    }
}

/// Handler for the `pwd` builtin
pub fn handle_pwd(_arg: Args) -> Result<String, OutputError> {
    match env::current_dir() {
        Ok(pwd) => Ok(format!("{}\n", pwd.display())),
        Err(err) => Err(format!("{}\n", err).into()),
    }
}

/// Handler for the `type` builtin
///
/// Searches for executable files using the `PATH` environment variable.
///
/// Some commands, such as `echo`, can exist as both builtin commands and executable files.
/// In such cases, the type command identifies them as builtins.
pub fn handle_type(arg: Args) -> Result<String, OutputError> {
    let mut result = "\n".to_string();

    if !arg.is_empty() {
        let arg = arg[0];
        if COMMANDS.contains(&arg) {
            result = format!("{arg} is a shell builtin\n");
        } else {
            let paths = get_paths();

            for path in paths {
                if path.join(arg).exists() {
                    result = format!("{arg} is {}\n", path.join(arg).display());
                    return Ok(result);
                }
            }

            return Err(format!("{arg}: not found\n").into());
        }
    };

    Ok(result)
}

/// Runs external programs with arguments
///
/// External programs are located using the `PATH` environment variable.
pub fn run_program(exec: &str, args: Args) -> Result<String, OutputError> {
    let paths = get_paths();

    for path in paths {
        if path.join(exec).exists() {
            let output = match Command::new(exec).args(args).output() {
                Ok(output) => output,
                Err(err) => {
                    return Err(format!(
                        "{err}: failed to execute command `{} {}'\n",
                        exec,
                        args.join(" ")
                    )
                    .into());
                }
            };
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8(output.stderr).unwrap());
            }
            if !output.stdout.is_empty() {
                return Ok(String::from_utf8(output.stdout).unwrap().to_string());
            }
        }
    }

    Err(format!("{exec}: command not found\n").into())
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
