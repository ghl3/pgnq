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

    #[test]
    fn test_tree_view_simple() {
        let tree = parse("1. e4 e5 2. Nf3").unwrap();
        let options = OutputOptions {
            format: crate::serializer::OutputFormat::Tree,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e4"));
        assert!(view.contains("e5"));
        assert!(view.contains("Nf3"));
    }

    #[test]
    fn test_tree_view_with_variations() {
        let tree = parse("1. e4 e5 (1... c5) 2. Nf3").unwrap();
        let options = OutputOptions {
            format: crate::serializer::OutputFormat::Tree,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        assert!(view.contains("e5"));
        assert!(view.contains("c5"));
    }

    #[test]
    fn test_tree_view_depth_limit() {
        let tree = parse("1. e4 e5 2. Nf3 Nc6 3. Bb5 a6").unwrap();
        let options = OutputOptions {
            format: crate::serializer::OutputFormat::Tree,
            max_depth: 2,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should have ... for truncated content
        assert!(view.contains("..."));
    }

    #[test]
    fn test_ascii_mode() {
        let tree = parse("1. e4 e5").unwrap();
        let options = OutputOptions {
            format: crate::serializer::OutputFormat::Tree,
            ascii: true,
            max_depth: 10,
            ..Default::default()
        };
        let view = to_tree_view(&tree, &options);

        // Should use ASCII characters
        assert!(view.contains("|--") || view.contains("`--"));
    }
}
