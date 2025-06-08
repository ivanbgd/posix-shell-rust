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

// /// Handler for the `exit` command
// pub fn handle_exit(arg: Option<&str>) -> Result<i32, CmdError> {
//     if cmd.eq("exit") {
//         return Ok(0);
//     }
//
//     let (cmd, arg) = cmd.split_once(" ").expect("Split failed");
//
//     if !cmd.eq("exit") {
//         return Err(CmdError::InvalidCommand(cmd.to_string()));
//     }
//
//     let res = match arg.trim().parse::<i32>() {
//         Ok(res) => res,
//         Err(err) => {
//             eprintln!("Invalid exit code: {arg}");
//             Err(CmdError::ParseIntError(err))?
//         }
//     };
//
//     Ok(res)
// }
//
