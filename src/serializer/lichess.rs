//! Lichess study format serialization

use super::options::OutputOptions;
use crate::tree::{GameNode, GameTree};

/// Serialize a GameTree to Lichess study format
pub fn to_lichess(tree: &GameTree, options: &OutputOptions) -> String {
    let mut output = String::new();

    // Write headers (same as standard)
    if options.headers {
        let str = tree.seven_tag_roster();
        output.push_str(&format!("[Event \"{}\"]\n", str.event));
        output.push_str(&format!("[Site \"{}\"]\n", str.site));
        output.push_str(&format!("[Date \"{}\"]\n", str.date));
        output.push_str(&format!("[Round \"{}\"]\n", str.round));
        output.push_str(&format!("[White \"{}\"]\n", str.white));
        output.push_str(&format!("[Black \"{}\"]\n", str.black));
        output.push_str(&format!("[Result \"{}\"]\n", str.result));

        for (key, value) in &tree.headers {
            if !is_str_header(key) {
                output.push_str(&format!("[{} \"{}\"]\n", key, value));
            }
        }
        output.push('\n');
    }

    // Write moves in Lichess format (line-based)
    serialize_lichess_node(&tree.root, options, &mut output, 0);

    // Add result
    if options.result {
        output.push_str(tree.result.as_str());
        output.push('\n');
    }

    output
}

fn is_str_header(key: &str) -> bool {
    matches!(
        key,
        "Event" | "Site" | "Date" | "Round" | "White" | "Black" | "Result"
    )
}

/// Serialize a node in Lichess format (line-based with bare comments)
fn serialize_lichess_node(
    node: &GameNode,
    options: &OutputOptions,
    output: &mut String,
    depth: usize,
) {
    // Check depth limit
    if options.max_depth > 0 && depth > options.max_depth {
        return;
    }

    // Write this node's move (skip root)
    if !node.is_root() {
        // Write move with number
        if let Some(num) = node.move_number {
            if node.is_black {
                output.push_str(&format!("{}... ", num));
            } else {
                output.push_str(&format!("{}. ", num));
            }
        }
        output.push_str(&node.san);

        // Write NAGs on same line
        if options.nags {
            for nag in &node.nags {
                output.push_str(&format!(" {}", nag));
            }
        }
        output.push('\n');

        // Write comment on separate line (Lichess style)
        if options.comments && !node.comment.is_empty() {
            let comment = process_lichess_comment(&node.comment, options);
            if !comment.is_empty() {
                output.push_str(&comment);
                output.push('\n');
            }
        }
    }

    // Write children
    if !node.children.is_empty() {
        // Main line first
        if let Some(main) = node.main_line() {
            serialize_lichess_node(main, options, output, depth + 1);
        }

        // Variations
        if options.variations && node.has_variations() {
            for var in node.variations() {
                output.push_str("(\n");
                serialize_lichess_node(var, options, output, depth + 1);
                output.push_str(")\n");
            }
        }
    }
}

/// Process a comment for Lichess format
fn process_lichess_comment(comment: &str, options: &OutputOptions) -> String {
    let mut result = comment.to_string();

    if options.strip_clocks {
        result = remove_command(&result, "%clk");
        result = remove_command(&result, "%emt");
    }

    if options.strip_evals {
        result = remove_command(&result, "%eval");
    }

    result.trim().to_string()
}

/// Remove a PGN command like [%clk ...] from text
fn remove_command(text: &str, cmd: &str) -> String {
    let pattern = format!("[%{}", cmd);
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut skip_depth = 0;

    while let Some(c) = chars.next() {
        if c == '[' {
            let rest: String = chars.clone().take(pattern.len() - 1).collect();
            if format!("[{}", rest).starts_with(&pattern) {
                skip_depth = 1;
                for c2 in chars.by_ref() {
                    if c2 == '[' {
                        skip_depth += 1;
                    } else if c2 == ']' {
                        skip_depth -= 1;
                        if skip_depth == 0 {
                            break;
                        }
                    }
                }
                continue;
            }
        }
        if skip_depth == 0 {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_lichess_format() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_lichess(&tree, &options);

        // Each move should be on its own line
        assert!(pgn.contains("1. e4\n"));
        assert!(pgn.contains("1... e5\n"));
    }

    #[test]
    fn test_lichess_with_comments() {
        let tree = parse("1. e4 {Opening move} e5").unwrap();
        let options = OutputOptions {
            headers: false,
            result: false,
            ..Default::default()
        };
        let pgn = to_lichess(&tree, &options);

        // Comment should be on separate line without braces
        assert!(pgn.contains("Opening move"));
        assert!(!pgn.contains("{"));
    }
}
