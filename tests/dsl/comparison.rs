//! Tree comparison logic for test assertions
//!
//! Provides functions to compare actual `GameTree` instances against
//! `TreeExpectation` specifications, with detailed diff output on mismatch.

use super::expectation::{ChildExpectation, NodeExpectation, TreeExpectation};
use super::matcher::NagMatcher;
use pgnq::tree::{GameNode, GameTree};
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

    /// Format as a detailed error message
    pub fn format_error(&self, actual: &GameTree) -> String {
        match self {
            CompareResult::Match => "Trees match".to_string(),
            CompareResult::Mismatch(diffs) => {
                let mut output = String::new();
                writeln!(output, "Tree comparison failed:").unwrap();
                writeln!(output).unwrap();

                for diff in diffs {
                    writeln!(output, "  Location: {}", diff.path).unwrap();
                    writeln!(output, "  Expected: {}", diff.expected).unwrap();
                    writeln!(output, "  Actual: {}", diff.actual).unwrap();
                    writeln!(output).unwrap();
                }

                // Add tree visualization for context
                writeln!(output, "Actual tree structure:").unwrap();
                writeln!(output, "{}", format_tree_preview(actual, 3)).unwrap();

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

/// Compare a tree against an expectation (subset matching)
///
/// The actual tree may have additional properties not specified in the expectation.
/// Only the properties specified in the expectation are checked.
pub fn tree_contains(actual: &GameTree, expected: &TreeExpectation) -> CompareResult {
    let mut diffs = Vec::new();

    // Check headers
    for (key, matcher) in &expected.headers {
        let actual_value = actual.header(key).unwrap_or("");
        if !matcher.matches(actual_value) {
            diffs.push(Difference::new(
                format!("header[{}]", key),
                matcher.describe(),
                format!("{:?}", actual_value),
            ));
        }
    }

    // Check result
    if let Some(expected_result) = &expected.result {
        if actual.result != *expected_result {
            diffs.push(Difference::new(
                "result",
                expected_result.as_str(),
                actual.result.as_str(),
            ));
        }
    }

    // Check node count
    if let Some(count_matcher) = &expected.node_count {
        let actual_count = actual.count_nodes();
        if !count_matcher.matches(actual_count) {
            diffs.push(Difference::new(
                "node_count",
                count_matcher.describe(),
                actual_count.to_string(),
            ));
        }
    }

    // Check main line
    if let Some(expected_main_line) = &expected.expected_main_line {
        let actual_main_line = get_main_line_sans(actual);
        if !main_line_matches(&actual_main_line, expected_main_line) {
            diffs.push(Difference::new(
                "main_line",
                format!("{:?}", expected_main_line),
                format!("{:?}", actual_main_line),
            ));
        }
    }

    // Check root children
    for child_exp in &expected.root_children {
        check_child_expectation(&actual.root, child_exp, "root", &mut diffs);
    }

    if diffs.is_empty() {
        CompareResult::Match
    } else {
        CompareResult::Mismatch(diffs)
    }
}

/// Compare trees for exact equality
///
/// Both trees must have identical structure, headers, and all node properties.
pub fn trees_equal(actual: &GameTree, expected: &GameTree) -> CompareResult {
    let mut diffs = Vec::new();

    // Compare headers
    for (key, value) in &expected.headers {
        match actual.header(key) {
            Some(actual_value) if actual_value == value => {}
            Some(actual_value) => {
                diffs.push(Difference::new(
                    format!("header[{}]", key),
                    format!("{:?}", value),
                    format!("{:?}", actual_value),
                ));
            }
            None => {
                diffs.push(Difference::new(
                    format!("header[{}]", key),
                    format!("{:?}", value),
                    "missing",
                ));
            }
        }
    }

    // Check for extra headers in actual
    for key in actual.headers.keys() {
        if !expected.headers.contains_key(key) {
            diffs.push(Difference::new(
                format!("header[{}]", key),
                "missing",
                format!("{:?}", actual.header(key).unwrap_or("")),
            ));
        }
    }

    // Compare result
    if actual.result != expected.result {
        diffs.push(Difference::new(
            "result",
            expected.result.as_str(),
            actual.result.as_str(),
        ));
    }

    // Compare tree structure
    compare_nodes_equal(&actual.root, &expected.root, "root", &mut diffs);

    if diffs.is_empty() {
        CompareResult::Match
    } else {
        CompareResult::Mismatch(diffs)
    }
}

// Helper: check a node against an expectation
fn check_node_expectation(
    actual: &GameNode,
    expected: &NodeExpectation,
    path: &str,
    diffs: &mut Vec<Difference>,
) {
    // Check SAN
    if let Some(san_matcher) = &expected.san {
        if !san_matcher.matches(&actual.san) {
            diffs.push(Difference::new(
                format!("{} -> san", path),
                san_matcher.describe(),
                format!("{:?}", actual.san),
            ));
        }
    }

    // Check comment
    if let Some(comment_matcher) = &expected.comment {
        if !comment_matcher.matches(&actual.comment) {
            diffs.push(Difference::new(
                format!("{} -> comment", path),
                comment_matcher.describe(),
                format!("{:?}", actual.comment),
            ));
        }
    }

    // Check NAGs
    if let Some(nag_matcher) = &expected.nags {
        if !nag_matcher.matches(&actual.nags) {
            diffs.push(Difference::new(
                format!("{} -> nags", path),
                nag_matcher.describe(),
                NagMatcher::describe_actual(&actual.nags),
            ));
        }
    }

    // Check children count
    if let Some(count_matcher) = &expected.children_count {
        if !count_matcher.matches(actual.children.len()) {
            diffs.push(Difference::new(
                format!("{} -> children_count", path),
                count_matcher.describe(),
                actual.children.len().to_string(),
            ));
        }
    }

    // Check must_have_variations
    if let Some(must_have) = expected.must_have_variations {
        let has_variations = actual.children.len() > 1;
        if has_variations != must_have {
            diffs.push(Difference::new(
                format!("{} -> variations", path),
                if must_have {
                    "has variations"
                } else {
                    "no variations"
                },
                if has_variations {
                    "has variations"
                } else {
                    "no variations"
                },
            ));
        }
    }

    // Check move number
    if let Some(expected_num) = expected.move_number {
        match actual.move_number {
            Some(actual_num) if actual_num == expected_num => {}
            Some(actual_num) => {
                diffs.push(Difference::new(
                    format!("{} -> move_number", path),
                    expected_num.to_string(),
                    actual_num.to_string(),
                ));
            }
            None => {
                diffs.push(Difference::new(
                    format!("{} -> move_number", path),
                    expected_num.to_string(),
                    "none",
                ));
            }
        }
    }

    // Check is_black
    if let Some(expected_black) = expected.is_black {
        if actual.is_black != expected_black {
            diffs.push(Difference::new(
                format!("{} -> is_black", path),
                expected_black.to_string(),
                actual.is_black.to_string(),
            ));
        }
    }

    // Check expected children
    for child_exp in &expected.children {
        check_child_expectation(actual, child_exp, path, diffs);
    }
}

// Helper: check a child expectation
fn check_child_expectation(
    parent: &GameNode,
    child_exp: &ChildExpectation,
    parent_path: &str,
    diffs: &mut Vec<Difference>,
) {
    let child_path = format!("{} -> {}", parent_path, child_exp.san);

    match parent.find_child(&child_exp.san) {
        Some(child) => {
            check_node_expectation(child, &child_exp.expectation, &child_path, diffs);
        }
        None => {
            diffs.push(Difference::new(
                child_path,
                "child to exist",
                format!(
                    "child not found (available: {:?})",
                    parent.children.iter().map(|c| &c.san).collect::<Vec<_>>()
                ),
            ));
        }
    }
}

// Helper: compare nodes for exact equality
fn compare_nodes_equal(
    actual: &GameNode,
    expected: &GameNode,
    path: &str,
    diffs: &mut Vec<Difference>,
) {
    // Compare SAN
    if actual.san != expected.san {
        diffs.push(Difference::new(
            format!("{} -> san", path),
            format!("{:?}", expected.san),
            format!("{:?}", actual.san),
        ));
    }

    // Compare comment
    if actual.comment != expected.comment {
        diffs.push(Difference::new(
            format!("{} -> comment", path),
            format!("{:?}", expected.comment),
            format!("{:?}", actual.comment),
        ));
    }

    // Compare NAGs
    if actual.nags != expected.nags {
        diffs.push(Difference::new(
            format!("{} -> nags", path),
            format!("{:?}", expected.nags),
            format!("{:?}", actual.nags),
        ));
    }

    // Compare children count
    if actual.children.len() != expected.children.len() {
        diffs.push(Difference::new(
            format!("{} -> children_count", path),
            expected.children.len().to_string(),
            actual.children.len().to_string(),
        ));
    }

    // Compare each child
    let min_children = actual.children.len().min(expected.children.len());
    for i in 0..min_children {
        let child_path = format!("{} -> {}", path, expected.children[i].san);
        compare_nodes_equal(&actual.children[i], &expected.children[i], &child_path, diffs);
    }
}

// Helper: get main line as SANs
fn get_main_line_sans(tree: &GameTree) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = &tree.root;
    while let Some(child) = current.main_line() {
        result.push(child.san.clone());
        current = child;
    }
    result
}

// Helper: check if main line matches expectation
fn main_line_matches(actual: &[String], expected: &[String]) -> bool {
    if actual.len() < expected.len() {
        return false;
    }
    for (i, exp) in expected.iter().enumerate() {
        if actual.get(i) != Some(exp) {
            return false;
        }
    }
    true
}

// Helper: format a path for display
fn format_path(path: &[&str]) -> String {
    if path.is_empty() {
        "root".to_string()
    } else {
        format!("root -> {}", path.join(" -> "))
    }
}

// Helper: format tree preview for error messages
fn format_tree_preview(tree: &GameTree, max_depth: usize) -> String {
    let mut output = String::new();
    format_node_preview(&tree.root, "", true, max_depth, 0, &mut output);
    output
}

fn format_node_preview(
    node: &GameNode,
    prefix: &str,
    is_last: bool,
    max_depth: usize,
    depth: usize,
    output: &mut String,
) {
    if depth > max_depth {
        if !node.children.is_empty() {
            writeln!(output, "{}└── ...", prefix).unwrap();
        }
        return;
    }

    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

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

    let children_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == children_count - 1;
        let new_prefix = if node.san.is_empty() {
            prefix.to_string()
        } else {
            child_prefix.clone()
        };
        format_node_preview(child, &new_prefix, is_last_child, max_depth, depth + 1, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgnq::parser::parse;

    fn parse_pgn(pgn: &str) -> GameTree {
        parse(pgn).expect("Failed to parse PGN")
    }

    #[test]
    fn test_tree_contains_simple() {
        let actual = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");
        let expected = TreeExpectation::new()
            .ongoing()
            .main_line(&["e4", "e5", "Nf3", "Nc6"]);

        let result = tree_contains(&actual, &expected);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_tree_contains_with_headers() {
        let actual = parse_pgn(
            r#"[Event "Test"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. e4 e5 1-0"#,
        );

        let expected = TreeExpectation::new()
            .header("Event", "Test")
            .header("White", "Player1")
            .white_wins();

        let result = tree_contains(&actual, &expected);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_tree_contains_node_expectation() {
        let actual = parse_pgn("1. e4! {Good move} e5 2. Nf3 *");

        let expected = TreeExpectation::new().root("e4", |n| {
            n.nag(pgnq::nag::Nag::GOOD_MOVE)
                .comment_contains("Good")
                .has_child("e5")
        });

        let result = tree_contains(&actual, &expected);
        assert!(result.is_match(), "{:?}", result);
    }

    #[test]
    fn test_tree_contains_mismatch() {
        let actual = parse_pgn("1. e4 e5 *");

        let expected = TreeExpectation::new().white_wins(); // Wrong result

        let result = tree_contains(&actual, &expected);
        assert!(result.is_mismatch());
    }

    #[test]
    fn test_format_error() {
        let actual = parse_pgn("1. e4 e5 *");
        let expected = TreeExpectation::new().white_wins();

        let result = tree_contains(&actual, &expected);
        let error_msg = result.format_error(&actual);

        assert!(error_msg.contains("Tree comparison failed"));
        assert!(error_msg.contains("result"));
    }
}
