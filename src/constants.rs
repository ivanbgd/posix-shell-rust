//! # Constants and Types
//!
//! Constants and types used throughout the application

use crate::cmd::{handle_cd, handle_echo, handle_exit, handle_pwd, handle_type};

/// Allows debug printouts
pub const DEBUG: bool = true;

/// Number of supported shell commands
const NUM_CMDS: usize = 5;

/// Supported Shell commands
pub const COMMANDS: [&str; NUM_CMDS] = ["cd", "echo", "exit", "pwd", "type"];

/// Supported Shell command handlers
pub const HANDLERS: [Handler; NUM_CMDS] =
    [handle_cd, handle_echo, handle_exit, handle_pwd, handle_type];

/// The shell prompt
pub const PROMPT: &str = "$ ";

/// An error message for invalid input
pub const INVALID_INPUT_MSG: &str = "invalid input";

/// Command-handlers' arguments type
pub type Args<'a> = &'a [&'a str];

/// Command-handlers' type
pub type Handler = fn(Args) -> ();
