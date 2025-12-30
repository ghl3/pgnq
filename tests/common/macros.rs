//! Assertion macros for tree comparison
//!
//! Provides convenient macros for asserting tree structure in tests.

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
        let result = $crate::common::node_contains(&actual.root, expected);
        if result.is_mismatch() {
            panic!("{}", result.format_node_error(&actual.root, expected));
        }
    }};
    ($actual:expr, $expected:expr, $($arg:tt)+) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::common::node_contains(&actual.root, expected);
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
        let result = $crate::common::nodes_match(actual, expected);
        if result.is_mismatch() {
            panic!("{}", result.format_node_error(actual, expected));
        }
    }};
    ($actual:expr, $expected:expr, $($arg:tt)+) => {{
        let actual = &$actual;
        let expected = &$expected;
        let result = $crate::common::nodes_match(actual, expected);
        if result.is_mismatch() {
            panic!("{}\n\nContext: {}", result.format_node_error(actual, expected), format_args!($($arg)+));
        }
    }};
}

/// Assert that GameTree headers match expected values (subset matching)
///
/// Verifies that specific headers exist and have the expected values.
/// Headers not specified in the assertion are not checked.
///
/// # Example
/// ```ignore
/// let tree = parse_pgn("[White \"Carlsen\"][Black \"Nepomniachtchi\"] 1. e4 *");
/// assert_headers!(tree, {
///     "White" => "Carlsen",
///     "Black" => "Nepomniachtchi",
/// });
/// ```
#[macro_export]
macro_rules! assert_headers {
    ($tree:expr, { $($key:literal => $value:literal),* $(,)? }) => {{
        let tree = &$tree;
        $(
            let actual = tree.headers.get($key);
            assert_eq!(
                actual.map(|s| s.as_str()),
                Some($value),
                "Header '{}' mismatch: expected {:?}, got {:?}",
                $key, $value, actual
            );
        )*
    }};
}

#[cfg(test)]
mod tests {
    use super::super::parse_pgn;

    #[test]
    fn test_assert_contains_tree_macro() {
        let tree = parse_pgn("1. e4 e5 *");
        let expected = crate::game_tree! { e4 { e5 } };
        crate::assert_contains_tree!(tree, expected);
    }

    #[test]
    fn test_assert_nodes_match_macro() {
        let tree1 = crate::game_tree! { e4 { e5 } };
        let tree2 = crate::game_tree! { e4 { e5 } };
        crate::assert_nodes_match!(tree1, tree2);
    }

    #[test]
    fn test_assert_headers_macro() {
        let tree = parse_pgn(r#"[White "Carlsen"][Black "Nepomniachtchi"] 1. e4 *"#);
        crate::assert_headers!(tree, {
            "White" => "Carlsen",
            "Black" => "Nepomniachtchi",
        });
    }
}
