//! Command handlers

/// Handler for the `echo` command
pub fn handle_echo(arg: Option<&str>) {}

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
