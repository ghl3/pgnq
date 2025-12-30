//! Tree comparison logic for test assertions
//!
//! Provides functions to compare actual `GameTree` instances against
//! expected `GameNode` trees built with the `game_tree!` macro.

use pgnq::tree::san::normalize as normalize_san;
use pgnq::tree::GameNode;
use std::fmt::Write;

/// Result of a tree comparison
#[derive(Debug, Clone)]
pub enum CompareResult {
    /// Trees match the expectation
    Match,
    /// Trees don't match, with list of differences
    Mismatch(Vec<Difference>),
}

impl CompareResult {
    /// Check if this is a match
    pub fn is_match(&self) -> bool {
        matches!(self, CompareResult::Match)
    }

    /// Check if this is a mismatch
    pub fn is_mismatch(&self) -> bool {
        matches!(self, CompareResult::Mismatch(_))
    }

    /// Get differences if mismatch
    pub fn differences(&self) -> Option<&[Difference]> {
        match self {
            CompareResult::Match => None,
            CompareResult::Mismatch(diffs) => Some(diffs),
        }
    }

    /// Format error for node comparison
    pub fn format_node_error(&self, actual: &GameNode, expected: &GameNode) -> String {
        match self {
            CompareResult::Match => "Nodes match".to_string(),
            CompareResult::Mismatch(diffs) => {
                let mut output = String::new();
                writeln!(output, "Node comparison failed:").unwrap();
                writeln!(output).unwrap();

                for diff in diffs {
                    writeln!(output, "  Location: {}", diff.path).unwrap();
                    writeln!(output, "  Expected: {}", diff.expected).unwrap();
                    writeln!(output, "  Actual: {}", diff.actual).unwrap();
                    writeln!(output).unwrap();
                }

                // Add tree visualizations
                writeln!(output, "Expected structure:").unwrap();
                writeln!(output, "{}", format_node_preview(expected, 5)).unwrap();
                writeln!(output).unwrap();
                writeln!(output, "Actual structure:").unwrap();
                writeln!(output, "{}", format_node_preview(actual, 5)).unwrap();

                output
            }
        }
    }
}

/// A single difference between expected and actual
#[derive(Debug, Clone)]
pub struct Difference {
    /// Path to the differing element (e.g., "root -> e4 -> e5")
    pub path: String,
    /// What was expected
    pub expected: String,
    /// What was actually found
    pub actual: String,
}

impl Difference {
    fn new(path: impl Into<String>, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

/// Check if actual node tree contains expected structure (subset matching)
///
/// This performs a subset match where:
/// - All nodes in `expected` must exist in `actual` at the same positions
/// - `actual` may have additional children not in `expected`
/// - Empty properties in `expected` are not checked (e.g., empty comment = don't check comment)
/// - Non-empty properties in `expected` must match exactly
///
/// # Example
/// ```ignore
/// let actual = parse_pgn("1. e4! e5 2. Nf3 *");
/// let expected = game_tree! { e4 (nag: GOOD_MOVE) { e5 { Nf3 } } };
/// assert!(node_contains(&actual.root, &expected).is_match());
/// ```
pub fn node_contains(actual: &GameNode, expected: &GameNode) -> CompareResult {
    let mut diffs = Vec::new();
    check_node_contains_recursive(actual, expected, "root", &mut diffs);
    if diffs.is_empty() {
        CompareResult::Match
    } else {
        CompareResult::Mismatch(diffs)
    }
}

fn check_node_contains_recursive(
    actual: &GameNode,
    expected: &GameNode,
    path: &str,
    diffs: &mut Vec<Difference>,
) {
    // Check SAN (only if expected has a non-empty SAN)
    if !expected.san.is_empty() {
        let actual_normalized = normalize_san(&actual.san);
        let expected_normalized = normalize_san(&expected.san);
        if actual_normalized != expected_normalized {
            diffs.push(Difference::new(
                format!("{} -> san", path),
                format!("{:?}", expected.san),
                format!("{:?}", actual.san),
            ));
        }
    }

    // Check comment (only if expected has a non-empty comment)
    // Uses exact matching - actual comment must equal expected text
    if !expected.comment.is_empty() && actual.comment != expected.comment {
        diffs.push(Difference::new(
            format!("{} -> comment", path),
            format!("{:?}", expected.comment),
            format!("{:?}", actual.comment),
        ));
    }

    // Check NAGs (only if expected has NAGs)
    if !expected.nags.is_empty() {
        // All expected NAGs must be present in actual
        for nag in &expected.nags {
            if !actual.nags.contains(nag) {
                diffs.push(Difference::new(
                    format!("{} -> nags", path),
                    format!("contains {:?}", nag),
                    format!("{:?}", actual.nags),
                ));
            }
        }
    }

    // Check children - each expected child must exist in actual
    for expected_child in &expected.children {
        let child_path = if path == "root" {
            expected_child.san.clone()
        } else {
            format!("{} -> {}", path, expected_child.san)
        };

        match find_child_by_san(actual, &expected_child.san) {
            Some(actual_child) => {
                check_node_contains_recursive(actual_child, expected_child, &child_path, diffs);
            }
            None => {
                diffs.push(Difference::new(
                    child_path,
                    "child to exist",
                    format!(
                        "child not found (available: {:?})",
                        actual.children.iter().map(|c| &c.san).collect::<Vec<_>>()
                    ),
                ));
            }
        }
    }
}

/// Check if two node trees match exactly
///
/// This performs an exact match where:
/// - Both trees must have identical structure
/// - All properties must match exactly (even empty ones)
/// - Children must appear in the same order
///
/// # Example
/// ```ignore
/// let tree1 = game_tree! { e4 { e5 } };
/// let tree2 = game_tree! { e4 { e5 } };
/// assert!(nodes_match(&tree1, &tree2).is_match());
/// ```
pub fn nodes_match(actual: &GameNode, expected: &GameNode) -> CompareResult {
    let mut diffs = Vec::new();
    check_nodes_match_recursive(actual, expected, "root", &mut diffs);
    if diffs.is_empty() {
        CompareResult::Match
    } else {
        CompareResult::Mismatch(diffs)
    }
}

fn check_nodes_match_recursive(
    actual: &GameNode,
    expected: &GameNode,
    path: &str,
    diffs: &mut Vec<Difference>,
) {
    // Check SAN
    let actual_normalized = normalize_san(&actual.san);
    let expected_normalized = normalize_san(&expected.san);
    if actual_normalized != expected_normalized {
        diffs.push(Difference::new(
            format!("{} -> san", path),
            format!("{:?}", expected.san),
            format!("{:?}", actual.san),
        ));
    }

    // Check comment
    if actual.comment != expected.comment {
        diffs.push(Difference::new(
            format!("{} -> comment", path),
            format!("{:?}", expected.comment),
            format!("{:?}", actual.comment),
        ));
    }

    // Check NAGs (exact match)
    if actual.nags != expected.nags {
        diffs.push(Difference::new(
            format!("{} -> nags", path),
            format!("{:?}", expected.nags),
            format!("{:?}", actual.nags),
        ));
    }

    // Check children count
    if actual.children.len() != expected.children.len() {
        diffs.push(Difference::new(
            format!("{} -> children_count", path),
            expected.children.len().to_string(),
            actual.children.len().to_string(),
        ));
    }

    // Check each child in order
    let min_children = actual.children.len().min(expected.children.len());
    for i in 0..min_children {
        let child_path = format!("{} -> {}", path, expected.children[i].san);
        check_nodes_match_recursive(&actual.children[i], &expected.children[i], &child_path, diffs);
    }
}

/// Find a child node by SAN, using normalized comparison
fn find_child_by_san<'a>(parent: &'a GameNode, san: &str) -> Option<&'a GameNode> {
    let normalized = normalize_san(san);
    parent
        .children
        .iter()
        .find(|c| normalize_san(&c.san) == normalized)
}

/// Format a node tree for display (helper for error messages)
fn format_node_preview(node: &GameNode, max_depth: usize) -> String {
    let mut output = String::new();
    format_node_preview_helper(node, "", true, max_depth, 0, &mut output);
    output
}

fn format_node_preview_helper(
    node: &GameNode,
    prefix: &str,
    is_last: bool,
    max_depth: usize,
    depth: usize,
    output: &mut String,
) {
    if depth > max_depth {
        if !node.children.is_empty() {
            writeln!(output, "{}...", prefix).unwrap();
        }
        return;
    }

    let connector = if is_last { "" } else { "" };
    let child_prefix = format!("{}  ", prefix);

    if !node.san.is_empty() {
        let mut line = node.san.clone();
        if !node.nags.is_empty() {
            let nag_str: Vec<_> = node.nags.iter().map(|n| n.to_string()).collect();
            line.push_str(&format!(" {}", nag_str.join(" ")));
        }
        if !node.comment.is_empty() {
            let comment_preview = if node.comment.len() > 30 {
                format!("{}...", &node.comment[..27])
            } else {
                node.comment.clone()
            };
            line.push_str(&format!(" {{{}}}", comment_preview));
        }
        writeln!(output, "{}{}{}", prefix, connector, line).unwrap();
    }

    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        format_node_preview_helper(child, &child_prefix, is_last_child, max_depth, depth + 1, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_contains_simple() {
        let actual = crate::game_tree! { e4 { e5 { Nf3 } } };
        let expected = crate::game_tree! { e4 { e5 } };

        let result = node_contains(&actual, &expected);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_node_contains_with_nags() {
        let mut actual = crate::game_tree! { e4 { e5 } };
        actual.children[0].nags.push(pgnq::nag::Nag::GOOD_MOVE);

        let expected = crate::game_tree! { e4 (nag: GOOD_MOVE) };

        let result = node_contains(&actual, &expected);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_node_contains_missing_child() {
        let actual = crate::game_tree! { e4 };
        let expected = crate::game_tree! { e4 { e5 } };

        let result = node_contains(&actual, &expected);
        assert!(result.is_mismatch());
    }

    #[test]
    fn test_nodes_match_simple() {
        let tree1 = crate::game_tree! { e4 { e5 } };
        let tree2 = crate::game_tree! { e4 { e5 } };

        let result = nodes_match(&tree1, &tree2);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_nodes_match_different_children() {
        let tree1 = crate::game_tree! { e4 { e5 } };
        let tree2 = crate::game_tree! { e4 { d5 } };

        let result = nodes_match(&tree1, &tree2);
        assert!(result.is_mismatch());
    }
}
