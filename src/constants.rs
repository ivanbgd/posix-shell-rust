//! # Constants, Globals and Types
//!
//! Constants, global variables and types used throughout the application

use crate::cmd::{handle_cd, handle_echo, handle_exit, handle_pwd, handle_type};
use crate::errors::OutputError;
use std::sync::OnceLock;

/// Allows debug printouts
pub static DEBUG: OnceLock<bool> = OnceLock::new();

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
pub type Handler = fn(Args) -> Result<String, OutputError>;
