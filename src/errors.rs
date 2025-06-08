//! # Errors
//!
//! Error types and helper functions used in the library and application

use std::num::ParseIntError;
use thiserror::Error;

/// Application errors
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    ReplError(#[from] ReplError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors related to working with [`crate::repl`]
#[derive(Debug, Error)]
pub enum ReplError {
    #[error(transparent)]
    CmdError(#[from] CmdError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors related to working with [`crate::cmd`]
#[derive(Debug, Error)]
pub enum CmdError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error("{0}: command not found")]
    InvalidCommand(String),
}
