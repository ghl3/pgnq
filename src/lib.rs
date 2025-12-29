//! pgnq - A Unix-like CLI tool for querying and manipulating chess PGN files
//!
//! This library provides parsing, manipulation, and serialization of PGN (Portable Game Notation)
//! files, treating them as tree structures rather than linear sequences.

pub mod cli;
pub mod error;
pub mod nag;
pub mod parser;
pub mod serializer;
pub mod tree;

pub use error::{Error, Result};
pub use nag::Nag;
pub use parser::{parse, parse_all};
pub use tree::{GameNode, GameResult, GameTree};
