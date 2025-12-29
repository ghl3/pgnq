//! Test DSL for pgnq
//!
//! This module provides a Domain Specific Language (DSL) for writing comprehensive
//! tests that compare parsed PGN trees against expected structures.
//!
//! # Philosophy
//!
//! The DSL separates tree construction from comparison:
//! 1. Build expected tree structures using `TreeExpectation` and `NodeExpectation`
//! 2. Compare parsed trees against expectations using comparison functions or macros
//!
//! This approach enables:
//! - Complete tree matching rather than fragmented property checks
//! - Partial/subset matching (only check specified properties)
//! - Clear error messages showing exactly where trees differ
//!
//! # Quick Start
//!
//! ```ignore
//! use crate::dsl::*;
//!
//! #[test]
//! fn test_example() {
//!     let actual = parse_pgn("1. e4 e5 2. Nf3 *");
//!
//!     let expected = TreeExpectation::new()
//!         .ongoing()
//!         .main_line(&["e4", "e5", "Nf3"]);
//!
//!     assert_tree_contains!(actual, expected);
//! }
//! ```
//!
//! # Components
//!
//! - [`TreeExpectation`] - Builder for expected tree structures
//! - [`NodeExpectation`] - Builder for expected node properties
//! - [`StringMatcher`] - Flexible string matching (exact, contains, starts_with, etc.)
//! - [`NagMatcher`] - NAG matching (exact, includes, none, etc.)
//! - [`tree_contains`] - Check if tree contains expected structure (subset matching)
//! - [`trees_equal`] - Check if two trees are exactly equal
//!
//! # Macros
//!
//! - `assert_tree_contains!` - Assert tree contains expected structure
//! - `assert_tree_eq!` - Assert two trees are exactly equal
//! - `assert_main_line!` - Assert main line matches expected moves
//! - `assert_headers!` - Assert headers match expected values
//! - `assert_has_nag!` - Assert node has specific NAG
//! - `assert_comment_contains!` - Assert comment contains text
//! - `assert_node_count!` - Assert node count
//! - `assert_result!` - Assert game result
//! - `assert_has_children!` - Assert node has children
//! - `assert_has_variations!` - Assert node has variations
//! - `assert_children_count!` - Assert exact children count

pub mod comparison;
pub mod expectation;
#[macro_use]
pub mod macros;
pub mod matcher;

// Re-export main types
pub use comparison::{tree_contains, trees_equal, CompareResult, Difference};
pub use expectation::{
    ChildExpectation, MainLineBuilder, NodeExpectation, PathBuilder, TreeExpectation,
};
pub use matcher::{CountMatcher, NagMatcher, StringMatcher};
