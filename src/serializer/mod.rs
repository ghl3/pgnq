//! PGN serialization - converting GameTree back to PGN text

mod lichess;
mod options;
mod standard;
mod tree_view;

pub use lichess::to_lichess;
pub use options::{OutputFormat, OutputOptions};
pub use standard::to_standard;
pub use tree_view::to_tree_view;

use crate::tree::GameTree;

/// Serialize a GameTree to PGN format
pub fn to_pgn(tree: &GameTree, options: &OutputOptions) -> String {
    match options.format {
        OutputFormat::Standard => to_standard(tree, options),
        OutputFormat::Lichess => to_lichess(tree, options),
        OutputFormat::Tree => to_tree_view(tree, options),
        OutputFormat::Minimal => to_minimal(tree, options),
    }
}

/// Minimal output - just moves, no annotations
fn to_minimal(tree: &GameTree, options: &OutputOptions) -> String {
    let mut output = String::new();

    // Write headers if requested
    if options.headers {
        for (key, value) in &tree.headers {
            output.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        if !tree.headers.is_empty() {
            output.push('\n');
        }
    }

    // Write moves (main line only)
    let mut current = &tree.root;
    let mut move_num = 1;
    let mut is_black = false;

    while let Some(child) = current.main_line() {
        if !is_black {
            output.push_str(&format!("{}. ", move_num));
        }
        output.push_str(&child.san);
        output.push(' ');

        if is_black {
            move_num += 1;
        }
        is_black = !is_black;
        current = child;
    }

    // Add result
    if options.result {
        output.push_str(tree.result.as_str());
    }

    output.trim().to_string()
}
