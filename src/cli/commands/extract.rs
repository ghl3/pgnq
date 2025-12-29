//! `pgnq extract` command - extract a subtree at a specific path

use crate::cli::{CliOutputFormat, InputSource};
use crate::parser::parse;
use crate::serializer::{to_pgn, OutputOptions};
use crate::tree::{GameNode, GameTree, NodePath, PathSegment};
use anyhow::Result;
use clap::Args;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct ExtractArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Node path to extract
    #[arg(short = 'p', long, required = true)]
    pub path: String,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "standard")]
    pub format: CliOutputFormat,

    /// Include moves from root to extraction point
    #[arg(long)]
    pub with_prefix: bool,

    /// Include headers from original file
    #[arg(long)]
    pub with_headers: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: ExtractArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    let path = NodePath::parse(&args.path)?;
    let node = path
        .resolve(&tree)
        .ok_or_else(|| anyhow::anyhow!("Path not found: {}", args.path))?;

    // Create a new tree with the extracted subtree
    let mut subtree = GameTree::new();

    if args.with_prefix {
        // Build the path from root to the target node, then attach the subtree
        subtree.root = build_prefix_tree(&tree.root, &path, node)?;
    } else {
        subtree.root = node.deep_clone();
    }

    subtree.result = tree.result.clone();

    if args.with_headers {
        subtree.headers = tree.headers.clone();
    }

    let options = OutputOptions {
        format: args.format.into(),
        headers: args.with_headers,
        result: true,
        ..Default::default()
    };

    let output = to_pgn(&subtree, &options);

    // Write output
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}

/// Build a tree with the path from root to target, with target's subtree attached
fn build_prefix_tree(
    root: &GameNode,
    path: &NodePath,
    target: &GameNode,
) -> Result<GameNode> {
    let mut new_root = GameNode::root();
    let mut current_source = root;
    let mut current_target = &mut new_root;

    for segment in &path.segments {
        match segment {
            PathSegment::Move { san, variation_index } => {
                // Find the child in source
                let child = if let Some(idx) = variation_index {
                    current_source.children.get(*idx)
                } else {
                    current_source.find_child(san)
                };

                if let Some(child) = child {
                    // Check if this is the target node
                    if std::ptr::eq(child, target) {
                        // This is the target - clone the entire subtree
                        let cloned = child.deep_clone();
                        current_target.children.push(cloned);
                    } else {
                        // Clone just this node (without children yet)
                        let mut node_copy = GameNode::new(&child.san);
                        node_copy.move_number = child.move_number;
                        node_copy.is_black = child.is_black;
                        node_copy.comment = child.comment.clone();
                        node_copy.nags = child.nags.clone();
                        // Don't copy children - we'll add the next path node as child

                        current_target.children.push(node_copy);
                        current_target = current_target.children.last_mut().unwrap();
                        current_source = child;
                    }
                }
            }
            PathSegment::End => {
                // Follow main line to end, cloning each node
                while let Some(child) = current_source.main_line() {
                    if std::ptr::eq(child, target) {
                        // Target found - clone entire subtree
                        let cloned = child.deep_clone();
                        current_target.children.push(cloned);
                        break;
                    } else {
                        let mut node_copy = GameNode::new(&child.san);
                        node_copy.move_number = child.move_number;
                        node_copy.is_black = child.is_black;
                        node_copy.comment = child.comment.clone();
                        node_copy.nags = child.nags.clone();

                        current_target.children.push(node_copy);
                        current_target = current_target.children.last_mut().unwrap();
                        current_source = child;
                    }
                }
            }
            PathSegment::Variation(idx) => {
                if let Some(child) = current_source.children.get(*idx) {
                    if std::ptr::eq(child, target) {
                        let cloned = child.deep_clone();
                        current_target.children.push(cloned);
                    } else {
                        let mut node_copy = GameNode::new(&child.san);
                        node_copy.move_number = child.move_number;
                        node_copy.is_black = child.is_black;
                        node_copy.comment = child.comment.clone();
                        node_copy.nags = child.nags.clone();

                        current_target.children.push(node_copy);
                        current_target = current_target.children.last_mut().unwrap();
                        current_source = child;
                    }
                }
            }
            PathSegment::Root | PathSegment::AllDescendants | PathSegment::DirectChildren => {
                // These don't affect the prefix path
            }
        }
    }

    Ok(new_root)
}
