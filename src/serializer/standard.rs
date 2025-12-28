//! Standard PGN format serialization

use super::options::OutputOptions;
use crate::tree::{GameNode, GameTree};

/// Serialize a GameTree to standard PGN format
pub fn to_standard(tree: &GameTree, options: &OutputOptions) -> String {
    let mut output = String::new();

    // Write headers
    if options.headers {
        // Write Seven Tag Roster first in order
        let str = tree.seven_tag_roster();
        output.push_str(&format!("[Event \"{}\"]\n", str.event));
        output.push_str(&format!("[Site \"{}\"]\n", str.site));
        output.push_str(&format!("[Date \"{}\"]\n", str.date));
        output.push_str(&format!("[Round \"{}\"]\n", str.round));
        output.push_str(&format!("[White \"{}\"]\n", str.white));
        output.push_str(&format!("[Black \"{}\"]\n", str.black));
        output.push_str(&format!("[Result \"{}\"]\n", str.result));

        // Write any additional headers
        for (key, value) in &tree.headers {
            if !is_str_header(key) {
                output.push_str(&format!("[{} \"{}\"]\n", key, value));
            }
        }
        output.push('\n');
    }

    // Write moves
    let movetext = serialize_node(&tree.root, options, 0, true);
    output.push_str(&movetext.trim());

    // Add result
    if options.result {
        output.push(' ');
        output.push_str(tree.result.as_str());
    }

    output.push('\n');
    output
}

fn is_str_header(key: &str) -> bool {
    matches!(
        key,
        "Event" | "Site" | "Date" | "Round" | "White" | "Black" | "Result"
    )
}

/// Serialize a node and its children to PGN movetext
fn serialize_node(
    node: &GameNode,
    options: &OutputOptions,
    depth: usize,
    force_move_number: bool,
) -> String {
    let mut output = String::new();

    // Check depth limit
    if options.max_depth > 0 && depth > options.max_depth {
        return output;
    }

    // Write this node's move (skip root)
    if !node.is_root() {
        output.push_str(&format_move(node, options, force_move_number));
    }

    // Write children with proper variation placement
    // In PGN, variations come after the move they branch from, but before
    // the continuation of the main line
    if !node.children.is_empty() {
        if let Some(main) = node.main_line() {
            // First, write the main line move (just this move, not descendants)
            output.push_str(&format_move(main, options, false));

            // Then write variations (alternatives to main line move)
            if options.variations && node.has_variations() {
                for var in node.variations() {
                    output.push_str("(");
                    output.push_str(&serialize_node(var, options, depth + 1, true));
                    output.push_str(") ");
                }
            }

            // Now continue with main line's children
            output.push_str(&serialize_children(main, options, depth + 1));
        }
    }

    output
}

/// Format a single move with its annotations (NAGs, comments)
fn format_move(node: &GameNode, options: &OutputOptions, force_move_number: bool) -> String {
    let mut output = String::new();

    if node.is_root() {
        return output;
    }

    // Write move number if needed
    let needs_number = force_move_number || !node.is_black;
    if needs_number {
        if let Some(num) = node.move_number {
            if node.is_black {
                output.push_str(&format!("{}... ", num));
            } else {
                output.push_str(&format!("{}. ", num));
            }
        }
    }

    // Write the move
    output.push_str(&node.san);

    // Write NAGs
    if options.nags {
        for nag in &node.nags {
            output.push_str(&format!(" {}", nag));
        }
    }

    // Write comment
    if options.comments && !node.comment.is_empty() {
        let comment = process_comment(&node.comment, options);
        if !comment.is_empty() {
            output.push_str(&format!(" {{ {} }}", comment));
        }
    }

    output.push(' ');
    output
}

/// Serialize just the children of a node (for continuing after variations)
fn serialize_children(node: &GameNode, options: &OutputOptions, depth: usize) -> String {
    let mut output = String::new();

    if options.max_depth > 0 && depth > options.max_depth {
        return output;
    }

    if !node.children.is_empty() {
        if let Some(main) = node.main_line() {
            // Write main line move
            output.push_str(&format_move(main, options, false));

            // Write variations
            if options.variations && node.has_variations() {
                for var in node.variations() {
                    output.push_str("(");
                    output.push_str(&serialize_node(var, options, depth + 1, true));
                    output.push_str(") ");
                }
            }

            // Continue with main line's children
            output.push_str(&serialize_children(main, options, depth + 1));
        }
    }

    output
}

/// Process a comment, optionally stripping clock/eval data
fn process_comment(comment: &str, options: &OutputOptions) -> String {
    let mut result = comment.to_string();

    if options.strip_clocks {
        // Remove [%clk ...] and [%emt ...]
        result = remove_command(&result, "clk");
        result = remove_command(&result, "emt");
    }

    if options.strip_evals {
        // Remove [%eval ...]
        result = remove_command(&result, "eval");
    }

    // Clean up extra whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Remove a PGN command like [%clk ...] from text
fn remove_command(text: &str, cmd: &str) -> String {
    let pattern = format!("[%{}", cmd);
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            // Check if this is our command
            let rest: String = chars.clone().take(pattern.len() - 1).collect();
            let check = format!("[{}", rest);
            if check.starts_with(&pattern) {
                // Skip until closing ]
                for c2 in chars.by_ref() {
                    if c2 == ']' {
                        break;
                    }
                }
                continue;
            }
        }
        result.push(c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    // ========================================================================
    // Basic Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_simple_game() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("e4"));
        assert!(pgn.contains("e5"));
        assert!(pgn.contains("Nf3"));
    }

    #[test]
    fn test_serialize_move_numbers() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 3. Bb5").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("1. e4"));
        assert!(pgn.contains("2. Nf3"));
        assert!(pgn.contains("3. Bb5"));
    }

    #[test]
    fn test_serialize_preserves_castling() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 O-O").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("O-O"));
    }

    // ========================================================================
    // Header Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_with_headers() {
        let tree = parse(
            r#"[Event "Test"][White "Alice"][Black "Bob"][Result "1-0"]
            1. e4 e5 1-0"#,
        )
        .unwrap();
        let options = OutputOptions::default();
        let pgn = to_standard(&tree, &options);

        assert!(pgn.contains("[Event \"Test\"]"));
        assert!(pgn.contains("[White \"Alice\"]"));
        assert!(pgn.contains("[Black \"Bob\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));
    }

    #[test]
    fn test_serialize_without_headers() {
        let tree = parse(
            r#"[Event "Test"][White "Alice"]
            1. e4 e5"#,
        )
        .unwrap();
        let options = OutputOptions {
            headers: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);

        assert!(!pgn.contains("[Event"));
        assert!(!pgn.contains("[White"));
    }

    #[test]
    fn test_serialize_seven_tag_roster_order() {
        let tree = parse(
            r#"[White "Alice"][Event "Test"][Date "2024.01.01"]
            1. e4"#,
        )
        .unwrap();
        let options = OutputOptions::default();
        let pgn = to_standard(&tree, &options);

        // Seven Tag Roster should come in order: Event, Site, Date, Round, White, Black, Result
        let event_pos = pgn.find("[Event").unwrap();
        let site_pos = pgn.find("[Site").unwrap();
        let date_pos = pgn.find("[Date").unwrap();
        let white_pos = pgn.find("[White").unwrap();

        assert!(event_pos < site_pos);
        assert!(site_pos < date_pos);
        assert!(date_pos < white_pos);
    }

    // ========================================================================
    // Comment Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_with_comments() {
        let tree = parse("1. e4 {Best!} e5").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("{ Best! }"));
    }

    #[test]
    fn test_serialize_without_comments() {
        let tree = parse("1. e4 {Best!} e5 {Good reply}").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            comments: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(!pgn.contains("{"));
        assert!(!pgn.contains("}"));
    }

    #[test]
    fn test_strip_clocks() {
        let comment = "Good move [%clk 1:30:00] indeed";
        let options = OutputOptions {
            strip_clocks: true,
            ..Default::default()
        };
        let result = process_comment(comment, &options);
        assert!(!result.contains("%clk"));
        assert!(result.contains("Good move"));
        assert!(result.contains("indeed"));
    }

    #[test]
    fn test_strip_emt() {
        let comment = "Played quickly [%emt 0:00:03] here";
        let options = OutputOptions {
            strip_clocks: true,
            ..Default::default()
        };
        let result = process_comment(comment, &options);
        assert!(!result.contains("%emt"));
        assert!(result.contains("Played quickly"));
    }

    #[test]
    fn test_strip_evals() {
        let comment = "Interesting [%eval +0.45] position";
        let options = OutputOptions {
            strip_evals: true,
            ..Default::default()
        };
        let result = process_comment(comment, &options);
        assert!(!result.contains("%eval"));
        assert!(result.contains("Interesting"));
        assert!(result.contains("position"));
    }

    #[test]
    fn test_strip_clocks_and_evals() {
        let comment = "[%clk 1:00:00] [%eval +0.5] Great move";
        let options = OutputOptions {
            strip_clocks: true,
            strip_evals: true,
            ..Default::default()
        };
        let result = process_comment(comment, &options);
        assert!(!result.contains("%clk"));
        assert!(!result.contains("%eval"));
        assert!(result.contains("Great move"));
    }

    #[test]
    fn test_preserve_clocks_when_not_stripped() {
        let comment = "Move [%clk 1:30:00] here";
        let options = OutputOptions {
            strip_clocks: false,
            ..Default::default()
        };
        let result = process_comment(comment, &options);
        assert!(result.contains("%clk"));
    }

    // ========================================================================
    // NAG Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_nags() {
        let tree = parse("1. e4! e5? 2. Nf3!! Nc6??").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("e4 !"));
        assert!(pgn.contains("e5 ?"));
        assert!(pgn.contains("Nf3 !!"));
        assert!(pgn.contains("Nc6 ??"));
    }

    #[test]
    fn test_serialize_without_nags() {
        let tree = parse("1. e4! e5? 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            nags: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(!pgn.contains("!"));
        assert!(!pgn.contains("?"));
    }

    #[test]
    fn test_serialize_numeric_nags() {
        let tree = parse("1. e4 $1 e5 $2").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        // Numeric NAGs are output as symbolic when possible
        assert!(pgn.contains("!") || pgn.contains("$1"));
    }

    // ========================================================================
    // Variation Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_single_variation() {
        let tree = parse("1. e4 e5 (1... c5 2. Nf3) 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("(1... c5"));
        assert!(pgn.contains(")"));
    }

    #[test]
    fn test_serialize_without_variations() {
        let tree = parse("1. e4 e5 (1... c5 2. Nf3) 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            variations: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(!pgn.contains("("));
        assert!(!pgn.contains("c5"));
    }

    #[test]
    fn test_serialize_nested_variations() {
        let tree = parse("1. e4 e5 (1... c5 2. Nf3 (2. d4)) 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        // Should have two levels of parentheses
        assert!(pgn.contains("(1... c5"));
        assert!(pgn.contains("(2. d4"));
    }

    #[test]
    fn test_serialize_sibling_variations() {
        let tree = parse("1. e4 e5 (1... c5) (1... d5) 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.contains("c5"));
        assert!(pgn.contains("d5"));
    }

    // ========================================================================
    // Result Serialization Tests
    // ========================================================================

    #[test]
    fn test_serialize_white_wins() {
        let tree = parse("1. e4 e5 1-0").unwrap();
        let options = OutputOptions {
            headers: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.ends_with("1-0\n"));
    }

    #[test]
    fn test_serialize_black_wins() {
        let tree = parse("1. e4 e5 0-1").unwrap();
        let options = OutputOptions {
            headers: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.ends_with("0-1\n"));
    }

    #[test]
    fn test_serialize_draw() {
        let tree = parse("1. e4 e5 1/2-1/2").unwrap();
        let options = OutputOptions {
            headers: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(pgn.ends_with("1/2-1/2\n"));
    }

    #[test]
    fn test_serialize_without_result() {
        let tree = parse("1. e4 e5 1-0").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_standard(&tree, &options);
        assert!(!pgn.contains("1-0"));
    }

    // ========================================================================
    // Format Move Helper Tests
    // ========================================================================

    #[test]
    fn test_format_move_white() {
        let mut node = GameNode::new("e4");
        node.move_number = Some(1);
        node.is_black = false;
        let options = OutputOptions::default();
        let text = format_move(&node, &options, false);
        assert!(text.contains("1. e4"));
    }

    #[test]
    fn test_format_move_black() {
        let mut node = GameNode::new("e5");
        node.move_number = Some(1);
        node.is_black = true;
        let options = OutputOptions::default();

        // Without forced move number
        let text = format_move(&node, &options, false);
        assert!(!text.contains("1..."));

        // With forced move number
        let text_forced = format_move(&node, &options, true);
        assert!(text_forced.contains("1... e5"));
    }

    #[test]
    fn test_format_move_with_nag() {
        let mut node = GameNode::new("e4");
        node.move_number = Some(1);
        node.is_black = false;
        node.nags.push(crate::nag::Nag::GOOD_MOVE);
        let options = OutputOptions::default();
        let text = format_move(&node, &options, false);
        assert!(text.contains("!"));
    }

    #[test]
    fn test_format_move_with_comment() {
        let mut node = GameNode::new("e4");
        node.move_number = Some(1);
        node.is_black = false;
        node.comment = "Opening move".to_string();
        let options = OutputOptions::default();
        let text = format_move(&node, &options, false);
        assert!(text.contains("{ Opening move }"));
    }

    // ========================================================================
    // Remove Command Helper Tests
    // ========================================================================

    #[test]
    fn test_remove_command_basic() {
        let text = "Hello [%clk 1:00:00] World";
        let result = remove_command(text, "clk");
        assert_eq!(result.trim(), "Hello  World");
    }

    #[test]
    fn test_remove_command_at_start() {
        let text = "[%eval +0.5] Good position";
        let result = remove_command(text, "eval");
        assert_eq!(result.trim(), "Good position");
    }

    #[test]
    fn test_remove_command_at_end() {
        let text = "Good move [%clk 0:30:00]";
        let result = remove_command(text, "clk");
        assert_eq!(result.trim(), "Good move");
    }

    #[test]
    fn test_remove_command_multiple() {
        let text = "[%clk 1:00:00] move [%clk 0:30:00]";
        let result = remove_command(text, "clk");
        assert!(!result.contains("%clk"));
    }

    #[test]
    fn test_remove_command_preserves_other() {
        let text = "[%clk 1:00:00] [%eval +0.5]";
        let result = remove_command(text, "clk");
        assert!(!result.contains("%clk"));
        assert!(result.contains("%eval"));
    }
}
