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
    }

    // Write children
    if !node.children.is_empty() {
        // Main line
        if let Some(main) = node.main_line() {
            output.push_str(&serialize_node(main, options, depth + 1, false));
        }

        // Variations
        if options.variations && node.has_variations() {
            for var in node.variations() {
                // Variations are enclosed in parentheses
                output.push_str("( ");
                output.push_str(&serialize_node(var, options, depth + 1, true));
                output.push_str(") ");
            }
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
}
