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
