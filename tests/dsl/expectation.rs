//! Expected tree structures for test assertions
//!
//! Provides `TreeExpectation` and `NodeExpectation` builders for declaratively
//! specifying expected tree structures that can be compared against parsed PGN.
//!
//! # Design Philosophy
//!
//! Use a **path-based API** that avoids closures for cleaner, more readable tests.
//!
//! ```ignore
//! // Clean path-based API (preferred)
//! TreeExpectation::new()
//!     .line(&["e4", "e5", "Nf3", "Nc6"])
//!     .at(&["e4"]).nag(Nag::GOOD_MOVE).comment_contains("Opening")
//!     .at(&["e4", "e5"]).nag(Nag::POOR_MOVE)
//!     .at(&["e4"]).has_children(&["e5", "c5", "d5"])
//!     .build()
//!
//! // Simple variations without closures
//! TreeExpectation::new()
//!     .line(&["e4", "e5", "Nf3"])
//!     .at(&["e4"]).children(&["e5", "c5", "d5", "e6"])
//!     .build()
//! ```
//!
//! The closure-based API is still available for complex nested cases.

use super::matcher::{CountMatcher, NagMatcher, StringMatcher};
use pgnq::nag::Nag;
use pgnq::tree::GameResult;
use std::collections::HashMap;

/// Expectation for a complete game tree
#[derive(Debug, Clone)]
pub struct TreeExpectation {
    /// Expected headers (only specified headers are checked)
    pub headers: HashMap<String, StringMatcher>,
    /// Expected game result (None = any result)
    pub result: Option<GameResult>,
    /// Expected root node children
    pub root_children: Vec<ChildExpectation>,
    /// Expected main line moves (shorthand for linear games)
    pub expected_main_line: Option<Vec<String>>,
    /// Expected node count (None = any count)
    pub node_count: Option<CountMatcher>,
    /// Path-based expectations: (path, expectation) pairs
    pub path_expectations: Vec<(Vec<String>, NodeExpectation)>,
}

/// A child expectation with SAN and node expectation
#[derive(Debug, Clone)]
pub struct ChildExpectation {
    /// The expected SAN move
    pub san: String,
    /// Whether this is a variation (non-main-line child)
    pub is_variation: bool,
    /// Expectations for this node
    pub expectation: NodeExpectation,
}

impl TreeExpectation {
    /// Create a new empty tree expectation
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            result: None,
            root_children: Vec::new(),
            expected_main_line: None,
            node_count: None,
            path_expectations: Vec::new(),
        }
    }

    /// Expect a specific header value (exact match)
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .insert(key.into(), StringMatcher::exact(value.into()));
        self
    }

    /// Expect a header with a custom matcher
    pub fn header_matches(mut self, key: impl Into<String>, matcher: StringMatcher) -> Self {
        self.headers.insert(key.into(), matcher);
        self
    }

    /// Expect a header to exist (any value)
    pub fn has_header(mut self, key: impl Into<String>) -> Self {
        self.headers.insert(key.into(), StringMatcher::any());
        self
    }

    /// Expect a specific game result
    pub fn result(mut self, result: GameResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Shorthand for white wins
    pub fn white_wins(self) -> Self {
        self.result(GameResult::WhiteWins)
    }

    /// Shorthand for black wins
    pub fn black_wins(self) -> Self {
        self.result(GameResult::BlackWins)
    }

    /// Shorthand for draw
    pub fn draw(self) -> Self {
        self.result(GameResult::Draw)
    }

    /// Shorthand for ongoing game
    pub fn ongoing(self) -> Self {
        self.result(GameResult::Ongoing)
    }

    /// Expect a specific main line (list of SANs)
    ///
    /// This is a shorthand for linear games without variations or annotations.
    /// For games with annotations or variations, use `root()` instead.
    pub fn main_line(mut self, moves: &[&str]) -> Self {
        self.expected_main_line = Some(moves.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add a root child with inline properties using a closure
    ///
    /// This is the primary way to build tree expectations. Properties are
    /// declared inline with the node, making trees easier to read.
    ///
    /// # Example
    /// ```ignore
    /// TreeExpectation::new()
    ///     .root("e4", |n| n
    ///         .nag(Nag::GOOD_MOVE)
    ///         .child("e5", |n| n
    ///             .comment("Symmetrical")
    ///         )
    ///     )
    /// ```
    pub fn root<F>(mut self, san: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(NodeExpectation) -> NodeExpectation,
    {
        self.root_children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation: f(NodeExpectation::new()),
        });
        self
    }

    /// Add a root variation (alternative first move)
    ///
    /// Use for games where you want to check a non-main-line first move.
    pub fn root_variation<F>(mut self, san: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(NodeExpectation) -> NodeExpectation,
    {
        self.root_children.push(ChildExpectation {
            san: san.into(),
            is_variation: true,
            expectation: f(NodeExpectation::new()),
        });
        self
    }

    /// Add a child at the root level (explicit NodeExpectation)
    ///
    /// Prefer `root()` with closure for most cases.
    pub fn with_root_child(mut self, san: impl Into<String>, expectation: NodeExpectation) -> Self {
        self.root_children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation,
        });
        self
    }

    /// Expect a specific node count
    pub fn node_count(mut self, count: usize) -> Self {
        self.node_count = Some(CountMatcher::exact(count));
        self
    }

    /// Expect at least this many nodes
    pub fn node_count_at_least(mut self, count: usize) -> Self {
        self.node_count = Some(CountMatcher::at_least(count));
        self
    }

    // ========================================================================
    // Path-based API (preferred - avoids closures)
    // ========================================================================

    /// Specify a linear main line path
    ///
    /// Creates expectations that this path exists in the tree.
    /// Use `.at()` to add properties at specific points.
    ///
    /// # Example
    /// ```ignore
    /// TreeExpectation::new()
    ///     .line(&["e4", "e5", "Nf3", "Nc6"])
    ///     .at(&["e4"]).nag(Nag::GOOD_MOVE)
    ///     .build()
    /// ```
    pub fn line(mut self, moves: &[&str]) -> Self {
        self.expected_main_line = Some(moves.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Start building expectations at a specific path
    ///
    /// Returns a PathBuilder for adding properties at this path.
    /// Chain multiple `.at()` calls to annotate different positions.
    ///
    /// # Example
    /// ```ignore
    /// TreeExpectation::new()
    ///     .line(&["e4", "e5", "Nf3"])
    ///     .at(&["e4"]).nag(Nag::GOOD_MOVE).comment_contains("Opening")
    ///     .at(&["e4", "e5"]).nag(Nag::POOR_MOVE)
    ///     .build()
    /// ```
    pub fn at(self, path: &[&str]) -> PathBuilder {
        PathBuilder::new(self, path)
    }
}

/// Builder for adding expectations at a specific path
///
/// Created by `TreeExpectation::at()`. Allows fluent chaining of
/// property expectations at a path, then continuing to other paths
/// or finishing with `.build()`.
pub struct PathBuilder {
    tree: TreeExpectation,
    path: Vec<String>,
    expectation: NodeExpectation,
}

impl PathBuilder {
    fn new(tree: TreeExpectation, path: &[&str]) -> Self {
        Self {
            tree,
            path: path.iter().map(|s| s.to_string()).collect(),
            expectation: NodeExpectation::new(),
        }
    }

    /// Expect a specific NAG at this path
    pub fn nag(mut self, nag: Nag) -> Self {
        self.expectation = self.expectation.nag(nag);
        self
    }

    /// Expect multiple NAGs at this path
    pub fn nags(mut self, nags: &[Nag]) -> Self {
        self.expectation = self.expectation.nags(nags);
        self
    }

    /// Expect at least one NAG at this path
    pub fn has_nag(mut self) -> Self {
        self.expectation = self.expectation.has_nag();
        self
    }

    /// Expect no NAGs at this path
    pub fn no_nags(mut self) -> Self {
        self.expectation = self.expectation.no_nags();
        self
    }

    /// Expect a specific comment at this path
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.expectation = self.expectation.comment(text);
        self
    }

    /// Expect comment containing substring at this path
    pub fn comment_contains(mut self, text: impl Into<String>) -> Self {
        self.expectation = self.expectation.comment_contains(text);
        self
    }

    /// Expect a non-empty comment at this path
    pub fn has_comment(mut self) -> Self {
        self.expectation = self.expectation.has_comment();
        self
    }

    /// Expect no comment at this path
    pub fn no_comment(mut self) -> Self {
        self.expectation = self.expectation.no_comment();
        self
    }

    /// Expect specific children at this path (by SAN)
    ///
    /// # Example
    /// ```ignore
    /// .at(&["e4"]).children(&["e5", "c5", "d5"])  // e4 has these children
    /// ```
    pub fn children(mut self, sans: &[&str]) -> Self {
        for san in sans {
            self.expectation.children.push(ChildExpectation {
                san: san.to_string(),
                is_variation: false,
                expectation: NodeExpectation::new(),
            });
        }
        self
    }

    /// Expect a specific number of children at this path
    pub fn children_count(mut self, count: usize) -> Self {
        self.expectation = self.expectation.children_count(count);
        self
    }

    /// Expect variations (more than one child) at this path
    pub fn has_variations(mut self) -> Self {
        self.expectation = self.expectation.has_variations();
        self
    }

    /// Expect no variations at this path
    pub fn no_variations(mut self) -> Self {
        self.expectation = self.expectation.no_variations();
        self
    }

    /// Continue to add expectations at another path
    ///
    /// Stores the current path's expectations and moves to a new path.
    pub fn at(mut self, path: &[&str]) -> Self {
        // Store current path expectations
        if !self.path.is_empty() {
            self.tree
                .path_expectations
                .push((self.path.clone(), self.expectation));
        }
        // Start new path
        Self {
            tree: self.tree,
            path: path.iter().map(|s| s.to_string()).collect(),
            expectation: NodeExpectation::new(),
        }
    }

    /// Finish building and return the TreeExpectation
    pub fn build(mut self) -> TreeExpectation {
        // Store final path expectations
        if !self.path.is_empty() {
            self.tree
                .path_expectations
                .push((self.path, self.expectation));
        }
        self.tree
    }
}

impl Default for TreeExpectation {
    fn default() -> Self {
        Self::new()
    }
}

/// Expectation for a single node in the game tree
#[derive(Debug, Clone, Default)]
pub struct NodeExpectation {
    /// Expected SAN (None = any SAN / don't check)
    pub san: Option<StringMatcher>,
    /// Expected comment (None = don't check comment)
    pub comment: Option<StringMatcher>,
    /// Expected NAGs (None = don't check NAGs)
    pub nags: Option<NagMatcher>,
    /// Expected children count (None = don't check)
    pub children_count: Option<CountMatcher>,
    /// Expected specific children
    pub children: Vec<ChildExpectation>,
    /// Must have variations (more than one child)
    pub must_have_variations: Option<bool>,
    /// Expected move number
    pub move_number: Option<u16>,
    /// Expected is_black flag
    pub is_black: Option<bool>,
}

impl NodeExpectation {
    /// Create a new empty node expectation
    pub fn new() -> Self {
        Self::default()
    }

    /// Expect a specific SAN move
    pub fn san(mut self, san: impl Into<String>) -> Self {
        self.san = Some(StringMatcher::exact(san.into()));
        self
    }

    /// Expect a specific comment (exact match)
    pub fn comment(mut self, text: impl Into<String>) -> Self {
        self.comment = Some(StringMatcher::exact(text.into()));
        self
    }

    /// Expect comment to contain substring
    pub fn comment_contains(mut self, substring: impl Into<String>) -> Self {
        self.comment = Some(StringMatcher::contains(substring.into()));
        self
    }

    /// Expect a non-empty comment
    pub fn has_comment(mut self) -> Self {
        self.comment = Some(StringMatcher::non_empty());
        self
    }

    /// Expect no comment
    pub fn no_comment(mut self) -> Self {
        self.comment = Some(StringMatcher::exact("".to_string()));
        self
    }

    /// Expect a specific NAG
    pub fn nag(mut self, nag: Nag) -> Self {
        self.nags = Some(NagMatcher::includes(nag));
        self
    }

    /// Expect multiple NAGs (all must be present)
    pub fn nags(mut self, nags: &[Nag]) -> Self {
        self.nags = Some(NagMatcher::includes_all(nags.to_vec()));
        self
    }

    /// Expect any of these NAGs
    pub fn nags_any(mut self, nags: &[Nag]) -> Self {
        self.nags = Some(NagMatcher::includes_any(nags.to_vec()));
        self
    }

    /// Expect at least one NAG (any NAG)
    pub fn has_nag(mut self) -> Self {
        self.nags = Some(NagMatcher::has_any());
        self
    }

    /// Expect no NAGs
    pub fn no_nags(mut self) -> Self {
        self.nags = Some(NagMatcher::none());
        self
    }

    /// Expect a specific number of children
    pub fn children_count(mut self, count: usize) -> Self {
        self.children_count = Some(CountMatcher::exact(count));
        self
    }

    /// Expect at least this many children
    pub fn children_at_least(mut self, count: usize) -> Self {
        self.children_count = Some(CountMatcher::at_least(count));
        self
    }

    /// Add a child node with inline properties using a closure
    ///
    /// This is the primary way to build node trees. Properties are declared
    /// inline with the node, making trees easier to read.
    ///
    /// # Example
    /// ```ignore
    /// NodeExpectation::new()
    ///     .child("e5", |n| n
    ///         .comment("Symmetrical response")
    ///         .child("Nf3", |n| n
    ///             .nag(Nag::GOOD_MOVE)
    ///         )
    ///     )
    /// ```
    pub fn child<F>(mut self, san: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(NodeExpectation) -> NodeExpectation,
    {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation: f(NodeExpectation::new()),
        });
        self
    }

    /// Add a variation with inline properties using a closure
    ///
    /// Use for non-main-line children (alternative moves).
    ///
    /// # Example
    /// ```ignore
    /// NodeExpectation::new()
    ///     .child("e5", |n| n)
    ///     .variation("c5", |n| n.comment("Sicilian Defense"))
    /// ```
    pub fn variation<F>(mut self, san: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(NodeExpectation) -> NodeExpectation,
    {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: true,
            expectation: f(NodeExpectation::new()),
        });
        self
    }

    /// Add an expected child (explicit NodeExpectation)
    ///
    /// Prefer `child()` with closure for most cases.
    pub fn with_child(mut self, san: impl Into<String>, expectation: NodeExpectation) -> Self {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation,
        });
        self
    }

    /// Add an expected variation (explicit NodeExpectation)
    ///
    /// Prefer `variation()` with closure for most cases.
    pub fn with_variation(
        mut self,
        san: impl Into<String>,
        expectation: NodeExpectation,
    ) -> Self {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: true,
            expectation,
        });
        self
    }

    /// Expect this node to have variations (more than one child)
    pub fn has_variations(mut self) -> Self {
        self.must_have_variations = Some(true);
        self
    }

    /// Expect this node to have no variations (at most one child)
    pub fn no_variations(mut self) -> Self {
        self.must_have_variations = Some(false);
        self
    }

    /// Expect a specific move number
    pub fn move_number(mut self, num: u16) -> Self {
        self.move_number = Some(num);
        self
    }

    /// Expect this to be black's move
    pub fn is_black(mut self) -> Self {
        self.is_black = Some(true);
        self
    }

    /// Expect this to be white's move
    pub fn is_white(mut self) -> Self {
        self.is_black = Some(false);
        self
    }

    /// Check if a child with the given SAN exists (no other expectations)
    ///
    /// Shorthand for `.child("san", |n| n)` when you only want to verify
    /// the child exists without checking its properties.
    pub fn has_child(mut self, san: impl Into<String>) -> Self {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation: NodeExpectation::new(),
        });
        self
    }

    /// Add a simple leaf child (no properties to check)
    ///
    /// Shorthand for `.child("san", |n| n)`.
    pub fn leaf(mut self, san: impl Into<String>) -> Self {
        self.children.push(ChildExpectation {
            san: san.into(),
            is_variation: false,
            expectation: NodeExpectation::new(),
        });
        self
    }
}

/// Builder for creating a chain of moves (linear main line)
pub struct MainLineBuilder {
    moves: Vec<String>,
    annotations: HashMap<usize, NodeExpectation>,
}

impl MainLineBuilder {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            annotations: HashMap::new(),
        }
    }

    /// Add a move to the main line
    pub fn mv(mut self, san: impl Into<String>) -> Self {
        self.moves.push(san.into());
        self
    }

    /// Annotate the last move
    pub fn annotate(mut self, expectation: NodeExpectation) -> Self {
        if !self.moves.is_empty() {
            self.annotations.insert(self.moves.len() - 1, expectation);
        }
        self
    }

    /// Build a NodeExpectation chain from this main line
    pub fn build(self) -> NodeExpectation {
        self.build_from_index(0)
    }

    fn build_from_index(&self, index: usize) -> NodeExpectation {
        if index >= self.moves.len() {
            return NodeExpectation::new();
        }

        let mut node = self
            .annotations
            .get(&index)
            .cloned()
            .unwrap_or_else(NodeExpectation::new);

        if index + 1 < self.moves.len() {
            node = node.with_child(&self.moves[index + 1], self.build_from_index(index + 1));
        }

        node.san(&self.moves[index])
    }
}

impl Default for MainLineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_expectation_builder() {
        let exp = TreeExpectation::new()
            .header("Event", "Test")
            .white_wins()
            .main_line(&["e4", "e5", "Nf3"]);

        assert_eq!(exp.result, Some(GameResult::WhiteWins));
        assert_eq!(
            exp.expected_main_line,
            Some(vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string()])
        );
    }

    #[test]
    fn test_tree_with_root_closure() {
        let exp = TreeExpectation::new()
            .ongoing()
            .root("e4", |n| n
                .nag(Nag::GOOD_MOVE)
                .child("e5", |n| n
                    .comment_contains("response")
                )
            );

        assert_eq!(exp.result, Some(GameResult::Ongoing));
        assert_eq!(exp.root_children.len(), 1);
        assert_eq!(exp.root_children[0].san, "e4");
    }

    #[test]
    fn test_node_expectation_builder() {
        let exp = NodeExpectation::new()
            .san("e4")
            .nag(Nag::GOOD_MOVE)
            .comment_contains("Opening")
            .children_count(2);

        assert!(exp.san.is_some());
        assert!(exp.nags.is_some());
        assert!(exp.comment.is_some());
        assert!(exp.children_count.is_some());
    }

    #[test]
    fn test_node_with_closure_children() {
        let exp = NodeExpectation::new()
            .san("e4")
            .child("e5", |n| n.san("e5"))
            .variation("c5", |n| n.san("c5").comment("Sicilian"));

        assert_eq!(exp.children.len(), 2);
        assert!(!exp.children[0].is_variation);
        assert!(exp.children[1].is_variation);
    }

    #[test]
    fn test_deep_nested_tree() {
        // Build a deep tree inline
        let exp = TreeExpectation::new()
            .root("e4", |n| n
                .nag(Nag::GOOD_MOVE)
                .child("c5", |n| n
                    .comment("Sicilian")
                    .child("Nf3", |n| n
                        .child("d6", |n| n
                            .child("d4", |n| n)
                        )
                    )
                )
            );

        assert_eq!(exp.root_children.len(), 1);
        let e4 = &exp.root_children[0];
        assert_eq!(e4.san, "e4");
        assert_eq!(e4.expectation.children.len(), 1);
        let c5 = &e4.expectation.children[0];
        assert_eq!(c5.san, "c5");
    }

    #[test]
    fn test_leaf_shorthand() {
        let exp = NodeExpectation::new()
            .leaf("e5")
            .leaf("c5");

        assert_eq!(exp.children.len(), 2);
    }

    #[test]
    fn test_main_line_builder() {
        let node = MainLineBuilder::new()
            .mv("e4")
            .mv("e5")
            .mv("Nf3")
            .annotate(NodeExpectation::new().nag(Nag::GOOD_MOVE))
            .build();

        assert!(node.san.is_some());
    }

    // ========================================================================
    // Path-based API tests
    // ========================================================================

    #[test]
    fn test_path_based_simple() {
        let exp = TreeExpectation::new()
            .line(&["e4", "e5", "Nf3"])
            .at(&["e4"]).nag(Nag::GOOD_MOVE)
            .build();

        assert_eq!(
            exp.expected_main_line,
            Some(vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string()])
        );
        assert_eq!(exp.path_expectations.len(), 1);
        assert_eq!(exp.path_expectations[0].0, vec!["e4".to_string()]);
    }

    #[test]
    fn test_path_based_multiple_paths() {
        let exp = TreeExpectation::new()
            .line(&["e4", "e5", "Nf3", "Nc6"])
            .at(&["e4"]).nag(Nag::GOOD_MOVE).comment_contains("Opening")
            .at(&["e4", "e5"]).nag(Nag::POOR_MOVE)
            .at(&["e4", "e5", "Nf3"]).has_comment()
            .build();

        assert_eq!(exp.path_expectations.len(), 3);
        assert_eq!(exp.path_expectations[0].0, vec!["e4".to_string()]);
        assert_eq!(
            exp.path_expectations[1].0,
            vec!["e4".to_string(), "e5".to_string()]
        );
        assert_eq!(
            exp.path_expectations[2].0,
            vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string()]
        );
    }

    #[test]
    fn test_path_based_with_children() {
        let exp = TreeExpectation::new()
            .line(&["e4", "e5"])
            .at(&["e4"]).children(&["e5", "c5", "d5"]).has_variations()
            .build();

        assert_eq!(exp.path_expectations.len(), 1);
        let (path, node_exp) = &exp.path_expectations[0];
        assert_eq!(path, &vec!["e4".to_string()]);
        assert_eq!(node_exp.children.len(), 3);
        assert!(node_exp.must_have_variations == Some(true));
    }

    #[test]
    fn test_path_based_replaces_closure_style() {
        // Old closure style:
        // .root("e4", |n| n
        //     .nag(Nag::GOOD_MOVE)
        //     .child("e5", |n| n
        //         .nag(Nag::POOR_MOVE)
        //         .child("Nf3", |n| n.has_comment())
        //     )
        // )
        //
        // New path style:
        let exp = TreeExpectation::new()
            .line(&["e4", "e5", "Nf3"])
            .at(&["e4"]).nag(Nag::GOOD_MOVE)
            .at(&["e4", "e5"]).nag(Nag::POOR_MOVE)
            .at(&["e4", "e5", "Nf3"]).has_comment()
            .build();

        // Much cleaner! No pyramid of doom.
        assert_eq!(exp.path_expectations.len(), 3);
    }

    #[test]
    fn test_path_based_variations() {
        // Old style:
        // .variation("e5", |n| n)
        // .variation("d5", |n| n)
        // .variation("Nf6", |n| n)
        //
        // New style:
        let exp = TreeExpectation::new()
            .at(&["e4"]).children(&["e5", "c5", "d5", "e6", "Nf6"])
            .build();

        assert_eq!(exp.path_expectations.len(), 1);
        assert_eq!(exp.path_expectations[0].1.children.len(), 5);
    }
}
