//! `pgnq tree` command - display the game tree visually

use crate::cli::InputSource;
use crate::parser::parse;
use crate::serializer::{to_tree_view, OutputFormat, OutputOptions};
use crate::tree::{GameNode, NodePath};
use anyhow::Result;
use clap::Args;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct TreeArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Maximum depth to display
    #[arg(short = 'd', long, default_value = "10")]
    pub depth: usize,

    /// Start from specific path
    #[arg(short = 'p', long)]
    pub from_path: Option<String>,

    /// Show comments (truncated)
    #[arg(long)]
    pub show_comments: bool,

    /// Show NAGs
    #[arg(long)]
    pub show_nags: bool,

    /// Use ASCII characters only (no Unicode box drawing)
    #[arg(long)]
    pub ascii: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: TreeArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    // Determine the root node to display
    let display_root = if let Some(path_str) = &args.from_path {
        let path = NodePath::parse(path_str)?;
        if let Some(node) = path.resolve(&tree) {
            node.deep_clone()
        } else {
            anyhow::bail!("Path not found: {}", path_str);
        }
    } else {
        tree.root.clone()
    };

    let output_string = if args.json {
        let json_tree = node_to_json(&display_root, args.depth, 0);
        serde_json::to_string_pretty(&json_tree)?
    } else {
        let options = OutputOptions {
            format: OutputFormat::Tree,
            max_depth: args.depth,
            ascii: args.ascii,
            show_comments: args.show_comments,
            show_nags: args.show_nags,
            ..Default::default()
        };

        // If a path is specified, use the subtree
        if args.from_path.is_some() {
            let mut subtree = crate::tree::GameTree::new();
            subtree.root = display_root;
            to_tree_view(&subtree, &options)
        } else {
            to_tree_view(&tree, &options)
        }
    };

    // Write output
    if let Some(path) = args.output {
        fs::write(&path, &output_string)?;
    } else {
        io::stdout().write_all(output_string.as_bytes())?;
    }

    Ok(())
}

/// Convert a game node to JSON representation recursively
fn node_to_json(node: &GameNode, max_depth: usize, current_depth: usize) -> serde_json::Value {
    if node.san.is_empty() {
        // Root node - just output children
        let children: Vec<_> = node
            .children
            .iter()
            .map(|child| node_to_json(child, max_depth, current_depth))
            .collect();
        json!({
            "root": true,
            "children": children
        })
    } else {
        let mut obj = json!({
            "san": node.san,
        });

        if let Some(move_num) = node.move_number {
            obj["move_number"] = json!(move_num);
            obj["is_black"] = json!(node.is_black);
        }

        if !node.comment.is_empty() {
            obj["comment"] = json!(node.comment);
        }

        if !node.nags.is_empty() {
            let nag_strs: Vec<String> = node.nags.iter().map(|n| n.to_string()).collect();
            obj["nags"] = json!(nag_strs);
        }

        // Only include children if within depth limit
        if current_depth < max_depth && !node.children.is_empty() {
            let children: Vec<_> = node
                .children
                .iter()
                .map(|child| node_to_json(child, max_depth, current_depth + 1))
                .collect();
            obj["children"] = json!(children);
        }

        obj
    }
}
