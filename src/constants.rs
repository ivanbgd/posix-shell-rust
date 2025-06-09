//! # Constants and Types
//!
//! Constants and types used throughout the application

use crate::cmd::{handle_cd, handle_echo, handle_exit, handle_pwd, handle_type};

/// Number of supported shell commands
const NUM_CMDS: usize = 5;

/// Supported Shell commands
pub const COMMANDS: [&str; NUM_CMDS] = ["cd", "echo", "exit", "pwd", "type"];

/// Supported Shell command handlers
pub const HANDLERS: [Handler; NUM_CMDS] =
    [handle_cd, handle_echo, handle_exit, handle_pwd, handle_type];

/// Stack size in bytes
pub const STACK_SIZE: usize = 32;

/// Command-handlers' arguments type
pub type Args<'a> = &'a [String];

/// Command-handlers' type
pub type Handler = fn(Args) -> ();
