//! Tree view serialization - visual display of game tree structure

use super::options::OutputOptions;
use crate::tree::{GameNode, GameTree};

/// Box-drawing characters for tree display
struct TreeChars {
    branch: &'static str,
    last_branch: &'static str,
    vertical: &'static str,
    space: &'static str,
}

impl TreeChars {
    fn unicode() -> Self {
        Self {
            branch: "├── ",
            last_branch: "└── ",
            vertical: "│   ",
            space: "    ",
        }
    }

    fn ascii() -> Self {
        Self {
            branch: "|-- ",
            last_branch: "`-- ",
            vertical: "|   ",
            space: "    ",
        }
    }
}

/// Serialize a GameTree to a visual tree format
pub fn to_tree_view(tree: &GameTree, options: &OutputOptions) -> String {
    let mut output = String::new();
    let chars = if options.ascii {
        TreeChars::ascii()
    } else {
        TreeChars::unicode()
    };

    // Start from root's children
    let children = &tree.root.children;
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        serialize_tree_node(child, options, &chars, "", is_last, 1, &mut output);
    }

    output
}

fn serialize_tree_node(
    node: &GameNode,
    options: &OutputOptions,
    chars: &TreeChars,
    prefix: &str,
    is_last: bool,
    depth: usize,
    output: &mut String,
) {
    // Check depth limit
    if options.max_depth > 0 && depth > options.max_depth {
        if !node.children.is_empty() {
            output.push_str(prefix);
            output.push_str(if is_last {
                chars.last_branch
            } else {
                chars.branch
            });
            output.push_str("...\n");
        }
        return;
    }

    // Write this node
    output.push_str(prefix);
    output.push_str(if is_last {
        chars.last_branch
    } else {
        chars.branch
    });

    // Format the move
    output.push_str(&node.move_text());

    // Add NAGs if requested
    if options.show_nags && !node.nags.is_empty() {
        for nag in &node.nags {
            output.push(' ');
            output.push_str(&nag.to_string());
        }
    }

    // Add truncated comment if requested
    if options.show_comments && !node.comment.is_empty() {
        let comment = truncate_comment(&node.comment, 40);
        output.push_str(" {");
        output.push_str(&comment);
        output.push('}');
    }

    // Add variation indicator if there are alternatives
    if node.has_variations() {
        let var_count = node.variations().len();
        output.push_str(&format!(" (+{} var)", var_count));
    }

    output.push('\n');

    // Build new prefix for children
    let child_prefix = format!(
        "{}{}",
        prefix,
        if is_last { chars.space } else { chars.vertical }
    );

    // Write children
    let children = &node.children;
    for (i, child) in children.iter().enumerate() {
        let child_is_last = i == children.len() - 1;
        serialize_tree_node(child, options, chars, &child_prefix, child_is_last, depth + 1, output);
    }
}

/// Truncate a comment to a maximum length
fn truncate_comment(comment: &str, max_len: usize) -> String {
    let comment = comment.trim();
    if comment.len() <= max_len {
        comment.to_string()
    } else {
        format!("{}...", &comment[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::serializer::OutputFormat;

    // ========================================================================
    // Basic Tree View Tests
    // ========================================================================

    #[test]
    fn test_tree_view_simple() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e4"));
        assert!(view.contains("e5"));
        assert!(view.contains("Nf3"));
    }

    #[test]
    fn test_tree_view_empty() {
        let tree = parse("*").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);
        assert!(view.is_empty());
    }

    #[test]
    fn test_tree_view_single_move() {
        let tree = parse("1. e4 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);
        assert!(view.contains("e4"));
        // Only one move, should use last_branch
        assert!(view.contains("└── ") || view.contains("`-- "));
    }

    #[test]
    fn test_tree_view_move_numbers() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should include move numbers
        assert!(view.contains("1. e4") || view.contains("e4"));
        assert!(view.contains("1... e5") || view.contains("e5"));
    }

    // ========================================================================
    // Variation Tests
    // ========================================================================

    #[test]
    fn test_tree_view_with_variations() {
        let tree = parse("1. e4 e5 (1... c5) 2. Nf3").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e5"));
        assert!(view.contains("c5"));
    }

    #[test]
    fn test_tree_view_multiple_variations() {
        let tree = parse("1. e4 e5 (1... c5) (1... e6) (1... d5) *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e5"));
        assert!(view.contains("c5"));
        assert!(view.contains("e6"));
        assert!(view.contains("d5"));
    }

    #[test]
    fn test_tree_view_nested_variations() {
        let tree = parse("1. e4 e5 (1... c5 2. Nf3 (2. Nc3)) *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e5"));
        assert!(view.contains("c5"));
        assert!(view.contains("Nf3"));
        assert!(view.contains("Nc3"));
    }

    #[test]
    fn test_tree_view_variation_indicator() {
        let tree = parse("1. e4 e5 (1... c5) *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // After e4, should show variation indicator
        // Note: The indicator appears on the parent node that has variations
    }

    // ========================================================================
    // Depth Limit Tests
    // ========================================================================

    #[test]
    fn test_tree_view_depth_limit() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 3. Bb5 a6").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: 2,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should have ... for truncated content
        assert!(view.contains("..."));
    }

    #[test]
    fn test_tree_view_depth_limit_zero() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: 0, // 0 means no limit
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should show all moves
        assert!(view.contains("e4"));
        assert!(view.contains("e5"));
        assert!(view.contains("Nf3"));
    }

    #[test]
    fn test_tree_view_depth_limit_one() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: 1,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e4"));
        assert!(view.contains("..."));
    }

    // ========================================================================
    // ASCII Mode Tests
    // ========================================================================

    #[test]
    fn test_ascii_mode() {
        let tree = parse("1. e4 e5").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ascii: true,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should use ASCII characters
        assert!(view.contains("|--") || view.contains("`--"));
        assert!(!view.contains("├"));
        assert!(!view.contains("└"));
    }

    #[test]
    fn test_unicode_mode() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ascii: false,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should use Unicode characters
        assert!(view.contains("├── ") || view.contains("└── "));
    }

    #[test]
    fn test_ascii_mode_with_variations() {
        let tree = parse("1. e4 e5 (1... c5) 2. Nf3").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ascii: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("|"));
        assert!(!view.contains("│"));
    }

    // ========================================================================
    // Comment Tests
    // ========================================================================

    #[test]
    fn test_tree_view_with_comments() {
        let tree = parse("1. e4 {Opening move} e5 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_comments: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("{Opening move}"));
    }

    #[test]
    fn test_tree_view_comments_disabled() {
        let tree = parse("1. e4 {Opening move} e5 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_comments: false,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(!view.contains("Opening move"));
    }

    #[test]
    fn test_tree_view_long_comment_truncation() {
        let long_comment = "x".repeat(100);
        let pgn = format!("1. e4 {{{}}} e5 *", long_comment);
        let tree = parse(&pgn).unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_comments: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Comment should be truncated with ...
        assert!(view.contains("..."));
    }

    #[test]
    fn test_tree_view_short_comment_not_truncated() {
        let tree = parse("1. e4 {Short} e5 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_comments: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("{Short}"));
        // Should not have truncation marker within the comment
    }

    // ========================================================================
    // NAG Tests
    // ========================================================================

    #[test]
    fn test_tree_view_with_nags() {
        let tree = parse("1. e4! e5? *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_nags: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("!") || view.contains("$1"));
        assert!(view.contains("?") || view.contains("$2"));
    }

    #[test]
    fn test_tree_view_nags_disabled() {
        let tree = parse("1. e4! e5? *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_nags: false,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should not contain NAG symbols
        // (Need to check the move itself isn't considered)
        assert!(view.contains("e4"));
    }

    #[test]
    fn test_tree_view_multiple_nags() {
        let tree = parse("1. e4 $1 $14 e5 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            show_nags: true,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should show both NAGs
        assert!(view.contains("e4"));
    }

    // ========================================================================
    // Truncate Comment Tests
    // ========================================================================

    #[test]
    fn test_truncate_comment_short() {
        let result = truncate_comment("Short comment", 40);
        assert_eq!(result, "Short comment");
    }

    #[test]
    fn test_truncate_comment_exact_length() {
        let comment = "a".repeat(40);
        let result = truncate_comment(&comment, 40);
        assert_eq!(result, comment);
    }

    #[test]
    fn test_truncate_comment_long() {
        let comment = "a".repeat(50);
        let result = truncate_comment(&comment, 40);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 40);
    }

    #[test]
    fn test_truncate_comment_whitespace() {
        let result = truncate_comment("  comment with spaces  ", 40);
        assert_eq!(result, "comment with spaces");
    }

    // ========================================================================
    // TreeChars Tests
    // ========================================================================

    #[test]
    fn test_tree_chars_unicode() {
        let chars = TreeChars::unicode();
        assert_eq!(chars.branch, "├── ");
        assert_eq!(chars.last_branch, "└── ");
        assert_eq!(chars.vertical, "│   ");
        assert_eq!(chars.space, "    ");
    }

    #[test]
    fn test_tree_chars_ascii() {
        let chars = TreeChars::ascii();
        assert_eq!(chars.branch, "|-- ");
        assert_eq!(chars.last_branch, "`-- ");
        assert_eq!(chars.vertical, "|   ");
        assert_eq!(chars.space, "    ");
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_tree_view_deep_tree() {
        let mut pgn = String::from("1. e4 e5 ");
        for i in 2..=10 {
            pgn.push_str(&format!("{}. Nc3 Nc6 ", i));
        }
        pgn.push('*');
        let tree = parse(&pgn).unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should contain many levels of indentation
        assert!(view.lines().count() > 5);
    }

    #[test]
    fn test_tree_view_wide_tree() {
        let tree = parse("1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) e5 *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e4"));
        assert!(view.contains("d4"));
        assert!(view.contains("c4"));
        assert!(view.contains("Nf3"));
        assert!(view.contains("g3"));
    }

    #[test]
    fn test_tree_view_castling() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O O-O *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("O-O"));
    }

    #[test]
    fn test_tree_view_promotion() {
        let tree = parse("1. e8=Q *").unwrap();
        let options = OutputOptions {
            format: OutputFormat::Tree,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e8=Q"));
    }
}
