//! Assertion macros for tree comparison
//!
//! Provides convenient macros for asserting tree structure in tests.

// ============================================================================
// GameNode comparison macros (for use with game_tree! macro)
// ============================================================================

/// Assert that actual GameTree contains the expected GameNode structure (subset matching)
///
/// This compares the root of the parsed GameTree against the expected GameNode tree.
/// The actual tree may have additional children/properties not in the expected tree.
///
/// # Example
/// ```ignore
/// let actual = parse_pgn("1. e4! e5 2. Nf3 *");
/// let expected = game_tree! { e4 (nag: GOOD_MOVE) { e5 { Nf3 } } };
/// assert_contains_tree!(actual, expected);
/// ```
#[macro_export]
macro_rules! assert_contains_tree {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::dsl::node_contains(&actual.root, expected);
        if result.is_mismatch() {
            panic!("{}", result.format_node_error(&actual.root, expected));
        }
    }};
    ($actual:expr, $expected:expr, $($arg:tt)+) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::dsl::node_contains(&actual.root, expected);
        if result.is_mismatch() {
            panic!("{}\n\nContext: {}", result.format_node_error(&actual.root, expected), format_args!($($arg)+));
        }
    }};
}

/// Assert that two GameNode trees match exactly
///
/// Both trees must have identical structure, properties, and children order.
///
/// # Example
/// ```ignore
/// let tree1 = game_tree! { e4 { e5 } };
/// let tree2 = game_tree! { e4 { e5 } };
/// assert_nodes_match!(tree1, tree2);
/// ```
#[macro_export]
macro_rules! assert_nodes_match {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::dsl::nodes_match(actual, expected);
        if result.is_mismatch() {
            panic!("{}", result.format_node_error(actual, expected));
        }
    }};
    ($actual:expr, $expected:expr, $($arg:tt)+) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::dsl::nodes_match(actual, expected);
        if result.is_mismatch() {
            panic!("{}\n\nContext: {}", result.format_node_error(actual, expected), format_args!($($arg)+));
        }
    }};
}

// ============================================================================
// TreeExpectation-based macros (original API)
// ============================================================================

/// Assert that a tree contains the expected structure (subset matching)
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 *");
/// assert_tree_contains!(tree, TreeExpectation::new().main_line(&["e4", "e5"]));
/// ```
#[macro_export]
macro_rules! assert_tree_contains {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual;
        let expected = $expected;
        let result = $crate::dsl::tree_contains(actual, &expected);
        if result.is_mismatch() {
            panic!("{}", result.format_error(actual));
        }
    }};
    ($actual:expr, $expected:expr, $($arg:tt)+) => {{
        let actual = &$actual;
        let expected = $expected;
        let result = $crate::dsl::tree_contains(actual, &expected);
        if result.is_mismatch() {
            panic!("{}\n\nContext: {}", result.format_error(actual), format_args!($($arg)+));
        }
    }};
}

/// Assert that two trees are exactly equal
///
/// # Example
/// ```ignore
/// let tree1 = parse_pgn("1. e4 e5 *");
/// let tree2 = parse_pgn("1. e4 e5 *");
/// assert_tree_eq!(tree1, tree2);
/// ```
#[macro_export]
macro_rules! assert_tree_eq {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::dsl::trees_equal(actual, expected);
        if result.is_mismatch() {
            panic!("{}", result.format_error(actual));
        }
    }};
}

/// Assert that a tree does NOT contain the expected structure
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 *");
/// assert_tree_not_contains!(tree, TreeExpectation::new().white_wins());
/// ```
#[macro_export]
macro_rules! assert_tree_not_contains {
    ($actual:expr, $expected:expr) => {{
        let actual = &$actual;
        let expected = $expected;
        let result = $crate::dsl::tree_contains(actual, &expected);
        if result.is_match() {
            panic!("Expected tree to NOT match expectation, but it did");
        }
    }};
}

/// Assert the main line of a tree matches expected moves
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");
/// assert_main_line!(tree, ["e4", "e5", "Nf3", "Nc6"]);
/// ```
#[macro_export]
macro_rules! assert_main_line {
    ($tree:expr, [$($move:literal),* $(,)?]) => {{
        let tree = &$tree;
        let expected_moves: Vec<&str> = vec![$($move),*];
        let actual_moves: Vec<String> = tree.root
            .iter_main_line()
            .skip(1)  // Skip root
            .map(|n| n.san.clone())
            .collect();

        let expected_len = expected_moves.len();
        let actual_len = actual_moves.len();

        if actual_len < expected_len {
            panic!(
                "Main line too short:\n  Expected: {:?}\n  Actual: {:?}",
                expected_moves, actual_moves
            );
        }

        for (i, expected) in expected_moves.iter().enumerate() {
            if actual_moves.get(i).map(|s| s.as_str()) != Some(*expected) {
                panic!(
                    "Main line mismatch at move {}:\n  Expected: {:?}\n  Actual: {:?}",
                    i + 1, expected_moves, actual_moves
                );
            }
        }
    }};
}

/// Assert that a tree has specific headers
///
/// # Example
/// ```ignore
/// let tree = parse_pgn(r#"[Event "Test"][White "Player1"]1. e4 *"#);
/// assert_headers!(tree, {
///     "Event" => "Test",
///     "White" => "Player1"
/// });
/// ```
#[macro_export]
macro_rules! assert_headers {
    ($tree:expr, { $($key:literal => $value:literal),* $(,)? }) => {{
        let tree = &$tree;
        $(
            let actual = tree.header($key);
            if actual != Some($value) {
                panic!(
                    "Header mismatch for {:?}:\n  Expected: {:?}\n  Actual: {:?}",
                    $key, $value, actual
                );
            }
        )*
    }};
}

/// Assert that a node at a path has a specific NAG
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4! e5 *");
/// assert_has_nag!(tree, ["e4"], Nag::GOOD_MOVE);
/// ```
#[macro_export]
macro_rules! assert_has_nag {
    ($tree:expr, [$($path:literal),*], $nag:expr) => {{
        let tree = &$tree;
        let path: &[&str] = &[$($path),*];
        let node = tree.find_path(path).expect(&format!("Node not found at path {:?}", path));
        let nag = $nag;
        if !node.nags.contains(&nag) {
            panic!(
                "NAG not found at {:?}:\n  Expected: {:?}\n  Actual NAGs: {:?}",
                path, nag, node.nags
            );
        }
    }};
}

/// Assert that a node at a path has a comment containing text
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 {Good opening} e5 *");
/// assert_comment_contains!(tree, ["e4"], "opening");
/// ```
#[macro_export]
macro_rules! assert_comment_contains {
    ($tree:expr, [$($path:literal),*], $text:literal) => {{
        let tree = &$tree;
        let path: &[&str] = &[$($path),*];
        let node = tree.find_path(path).expect(&format!("Node not found at path {:?}", path));
        if !node.comment.contains($text) {
            panic!(
                "Comment does not contain {:?} at {:?}:\n  Actual comment: {:?}",
                $text, path, node.comment
            );
        }
    }};
}

/// Assert the node count of a tree
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 2. Nf3 *");
/// assert_node_count!(tree, 3);
/// ```
#[macro_export]
macro_rules! assert_node_count {
    ($tree:expr, $count:expr) => {{
        let tree = &$tree;
        let actual = tree.count_nodes();
        let expected = $count;
        if actual != expected {
            panic!(
                "Node count mismatch:\n  Expected: {}\n  Actual: {}",
                expected, actual
            );
        }
    }};
}

/// Assert a tree has a specific result
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 1-0");
/// assert_result!(tree, GameResult::WhiteWins);
/// ```
#[macro_export]
macro_rules! assert_result {
    ($tree:expr, $result:expr) => {{
        let tree = &$tree;
        let expected = $result;
        if tree.result != expected {
            panic!(
                "Result mismatch:\n  Expected: {:?}\n  Actual: {:?}",
                expected, tree.result
            );
        }
    }};
}

/// Assert that a node has children (is not a leaf)
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 *");
/// assert_has_children!(tree, ["e4"]);
/// ```
#[macro_export]
macro_rules! assert_has_children {
    ($tree:expr, [$($path:literal),*]) => {{
        let tree = &$tree;
        let path: &[&str] = &[$($path),*];
        let node = tree.find_path(path).expect(&format!("Node not found at path {:?}", path));
        if node.children.is_empty() {
            panic!("Node at {:?} has no children", path);
        }
    }};
}

/// Assert that a node has variations (more than one child)
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 (1... c5) *");
/// assert_has_variations!(tree, ["e4"]);
/// ```
#[macro_export]
macro_rules! assert_has_variations {
    ($tree:expr, [$($path:literal),*]) => {{
        let tree = &$tree;
        let path: &[&str] = &[$($path),*];
        let node = tree.find_path(path).expect(&format!("Node not found at path {:?}", path));
        if node.children.len() <= 1 {
            panic!(
                "Node at {:?} has no variations (children: {})",
                path, node.children.len()
            );
        }
    }};
}

/// Assert exact children count at a path
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("1. e4 e5 (1... c5) (1... e6) *");
/// assert_children_count!(tree, ["e4"], 3);
/// ```
#[macro_export]
macro_rules! assert_children_count {
    ($tree:expr, [$($path:literal),*], $count:expr) => {{
        let tree = &$tree;
        let path: &[&str] = &[$($path),*];
        let node = tree.find_path(path).expect(&format!("Node not found at path {:?}", path));
        let actual = node.children.len();
        let expected = $count;
        if actual != expected {
            panic!(
                "Children count mismatch at {:?}:\n  Expected: {}\n  Actual: {} (children: {:?})",
                path, expected, actual,
                node.children.iter().map(|c| &c.san).collect::<Vec<_>>()
            );
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use pgnq::parser::parse;

    fn parse_pgn(pgn: &str) -> pgnq::tree::GameTree {
        parse(pgn).expect("Failed to parse PGN")
    }

    #[test]
    fn test_assert_tree_contains_macro() {
        let tree = parse_pgn("1. e4 e5 *");
        assert_tree_contains!(
            tree,
            TreeExpectation::new()
                .ongoing()
                .main_line(&["e4", "e5"])
        );
    }

    #[test]
    fn test_assert_main_line_macro() {
        let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");
        assert_main_line!(tree, ["e4", "e5", "Nf3", "Nc6"]);
    }

    #[test]
    fn test_assert_headers_macro() {
        let tree = parse_pgn(
            r#"[Event "Test"]
[White "Player1"]
[Result "*"]

1. e4 *"#,
        );
        assert_headers!(tree, {
            "Event" => "Test",
            "White" => "Player1"
        });
    }

    #[test]
    fn test_assert_node_count_macro() {
        let tree = parse_pgn("1. e4 e5 2. Nf3 *");
        assert_node_count!(tree, 3);
    }

    #[test]
    fn test_assert_result_macro() {
        let tree = parse_pgn("1. e4 e5 1-0");
        assert_result!(tree, pgnq::tree::GameResult::WhiteWins);
    }
}
