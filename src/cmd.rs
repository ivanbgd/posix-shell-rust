//! Command handlers

use crate::constants::{Args, COMMANDS};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::process::Command;

/// The output of a command
///
/// Contains fields `stdout` and `stderr` that hold the respective output data.
#[derive(Debug)]
pub struct Output {
    /// The data that the command wrote to `stdout`
    stdout: Vec<u8>,
    /// The data that the command wrote to `stderr`
    stderr: Vec<u8>,
}

impl Output {
    /// Constructs a new instance, filling the `stdout` and `stderr` fields.
    fn new(stdout: &[u8], stderr: &[u8]) -> Self {
        Self {
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }

    /// Gets the `stdout` and `stderr` data fields for reading.
    pub fn get(self) -> (Vec<u8>, Vec<u8>) {
        (self.stdout, self.stderr)
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Output {{ stdout: {:?}, stderr: {:?} }}",
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        )
    }
}

/// Handler for the `cd` builtin
pub fn handle_cd(arg: Args) -> Output {
    if !arg.is_empty() {
        let arg = &arg[0];
        let home = match env::var("HOME") {
            Ok(val) => val,
            Err(_) => {
                return Output::new(b"", b"HOME not found\n");
            }
        };
        let arg = if arg.eq(&"~") { home.as_str() } else { arg };

        if env::set_current_dir(arg).is_err() {
            return Output::new(
                b"",
                format!("cd: {arg}: No such file or directory\n").as_bytes(),
            );
        }
    };

    Output::new(b"", b"")
}

/// Handler for the `echo` builtin
pub fn handle_echo(args: Args) -> Output {
    Output::new(format!("{}\n", args.join(" ")).as_ref(), b"")
}

/// Handler for the `exit` builtin
pub fn handle_exit(arg: Args) -> Output {
    match arg.is_empty() {
        false => {
            let arg = &arg[0];
            match arg.trim().parse::<i32>() {
                Ok(exit_code) => std::process::exit(exit_code),
                Err(_) => Output::new(b"", format!("Invalid exit code: {arg}\n").as_bytes()),
            }
        }
        true => std::process::exit(0),
    }
}

/// Handler for the `pwd` builtin
pub fn handle_pwd(_arg: Args) -> Output {
    match env::current_dir() {
        Ok(pwd) => Output::new(format!("{}\n", pwd.display()).as_bytes(), b""),
        Err(err) => Output::new(b"", format!("{}\n", err).as_bytes()),
    }
}

/// Handler for the `type` builtin
///
/// Searches for executable files using the `PATH` environment variable.
///
/// Some commands, such as `echo`, can exist as both builtin commands and executable files.
/// In such cases, the type command identifies them as builtins.
pub fn handle_type(arg: Args) -> Output {
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
                    return Output::new(result.as_bytes(), b"");
                }
            }

            return Output::new(b"", format!("{arg}: not found\n").as_bytes());
        }
    };

    Output::new(result.as_bytes(), b"")
}

/// Runs external programs with arguments
///
/// External programs are located using the `PATH` environment variable.
pub fn run_program(exec: &str, args: Args) -> Output {
    // eprintln!("******* run_program() !!!!!!!"); // TODO remove
    let paths = get_paths();

    for path in paths {
        if path.join(exec).exists() {
            let output = match Command::new(exec).args(args).output() {
                Ok(output) => output,
                Err(err) => {
                    return Output::new(
                        b"",
                        format!(
                            "{err}: failed to execute command `{} {}'\n",
                            exec,
                            args.join(" ")
                        )
                        .as_ref(),
                    );
                }
            };
            return Output::new(&output.stdout, &output.stderr);
        }
    }

    Output::new(b"", format!("{exec}: command not found\n").as_ref())
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
