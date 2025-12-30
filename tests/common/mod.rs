//! Test support module for pgnq
//!
//! Provides utilities for writing integration tests:
//! - `parse_pgn()` - Parse PGN strings in tests
//! - `game_tree!` - Build expected tree structures declaratively
//! - `assert_contains_tree!` - Assert tree contains expected structure
//! - CLI testing helpers in `cli` submodule

#![allow(dead_code)]

pub mod cli;
pub mod comparison;
#[macro_use]
pub mod macros;
#[macro_use]
pub mod tree_macro;

// Re-export comparison functions
pub use comparison::{node_contains, nodes_match, CompareResult, Difference};

// Re-export CLI helpers
pub use cli::{pgn_file, pgnq, CommandResult, PgnOutput, PgnqCommand};

use pgnq::parser::parse;
use pgnq::tree::GameTree;

/// Parse a PGN string and return the game tree, panicking on failure
pub fn parse_pgn(pgn: &str) -> GameTree {
    parse(pgn).expect("Failed to parse PGN")
}

/// Parse a PGN string and return Result
pub fn try_parse_pgn(pgn: &str) -> Result<GameTree, pgnq::error::Error> {
    parse(pgn)
}

/// Count total move nodes in a game tree (excluding root)
pub fn count_nodes(tree: &GameTree) -> usize {
    tree.count_nodes()
}

/// Get the main line as a vector of SAN moves (excluding root)
pub fn main_line_moves(tree: &GameTree) -> Vec<String> {
    tree.root
        .iter_main_line()
        .skip(1) // Skip root node
        .map(|node| node.san.clone())
        .collect()
}
