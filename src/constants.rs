//! # Constants, Globals and Types
//!
//! Constants, global variables and types used throughout the application

use crate::cmd::{handle_cd, handle_echo, handle_exit, handle_pwd, handle_type, Output};
use std::sync::OnceLock;

/// Allows debug printouts
pub static DEBUG: OnceLock<bool> = OnceLock::new();

/// Used for end-to-end testing from within a shell script
pub static TEST: OnceLock<bool> = OnceLock::new();

/// Number of supported shell commands
const NUM_CMDS: usize = 5;

/// Supported Shell commands
pub const COMMANDS: [&str; NUM_CMDS] = ["cd", "echo", "exit", "pwd", "type"];

/// Supported Shell command handlers
pub const HANDLERS: [Handler; NUM_CMDS] =
    [handle_cd, handle_echo, handle_exit, handle_pwd, handle_type];

/// The shell prompt
pub const PROMPT: &[u8] = b"$ ";

/// Error message for invalid input
pub const INVALID_INPUT_MSG: &str = "invalid input";

/// Expect message for a failed read line from `stdin`
pub const FAILED_READ_LINE: &str = "Read line failed";
/// Expect message for a failed write to `stdout`
pub const FAILED_WRITE_TO_STDOUT: &str = "Write to stdout failed";
/// Expect message for a failed write to `stderr`
pub const FAILED_WRITE_TO_STDERR: &str = "Write to stderr failed";
/// Expect message for a failed flush to stdout
pub const FAILED_FLUSH_TO_STDOUT: &str = "Flush to stdout failed";
/// Expect message for a failed flush to stderr
pub const FAILED_FLUSH_TO_STDERR: &str = "Flush to stderr failed";

/// Command-handlers' arguments type
pub type Args<'a> = &'a [&'a str];

/// Command-handlers' type
pub type Handler = fn(Args) -> Output;
