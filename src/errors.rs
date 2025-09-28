//! # Errors
//!
//! Error types and helper functions used in the library

use crate::constants::INVALID_INPUT_MSG;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Invalid input error
///
/// Contains the reason for the error.
///
/// # Examples
/// - `echo \`
/// - `echo \\\`
/// - `echo '`
/// - `echo '''`
/// - `echo "`
/// - `echo """`
/// - `echo '\''`
/// - `echo test >>> file`
/// - `echo test &>& file`
/// - `echo test >&& file`
#[derive(Debug, PartialEq)]
pub struct InvalidInputError {
    pub reason: String,
}

impl Error for InvalidInputError {}

impl Display for InvalidInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{INVALID_INPUT_MSG}: {}", self.reason)
    }
}

impl From<&str> for InvalidInputError {
    fn from(value: &str) -> Self {
        Self {
            reason: value.to_string(),
        }
    }
}

/// Error output type from builtin handlers or from external programs
#[derive(Debug)]
pub struct OutputError {
    pub reason: String,
}

impl Error for OutputError {}

impl Display for OutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<&str> for OutputError {
    fn from(value: &str) -> Self {
        Self {
            reason: value.to_string(),
        }
    }
}

impl From<String> for OutputError {
    fn from(value: String) -> Self {
        Self { reason: value }
    }
}
