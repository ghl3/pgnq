//! Error types for pgnq

use thiserror::Error;

/// Main error type for pgnq operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid path syntax: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

/// Result type alias using our Error
pub type Result<T> = std::result::Result<T, Error>;
