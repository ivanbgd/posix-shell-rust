//! # Constants
//!
//! Constants used throughout the application

/// Supported Shell commands
pub const COMMANDS: [&[u8]; 5] = [b"cd", b"echo", b"exit", b"pwd", b"type"];

/// Stack size in bytes
pub const STACK_SIZE: usize = 32;
