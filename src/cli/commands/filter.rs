//! `pgnq filter` command - filter nodes matching criteria

use crate::cli::{CliOutputFormat, InputSource};
use crate::nag::Nag;
use crate::parser::parse;
use crate::serializer::{to_pgn, OutputOptions};
use crate::tree::{GameNode, GameTree, NodePath};
use anyhow::Result;
use clap::Args;
use std::io::{self, Write};

#[derive(Args)]
pub struct FilterArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Path pattern to match
    #[arg(short = 'p', long)]
    pub path: Option<String>,

    /// Filter nodes with comments
    #[arg(long)]
    pub has_comment: bool,

    /// Filter nodes with specific NAG
    #[arg(long)]
    pub has_nag: Option<String>,

    /// Minimum tree depth
    #[arg(long)]
    pub min_depth: Option<usize>,

    /// Maximum tree depth
    #[arg(long)]
    pub max_depth: Option<usize>,

    /// Main line nodes only
    #[arg(long)]
    pub main_line: bool,

    /// Invert the filter
    #[arg(long)]
    pub invert: bool,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "standard")]
    pub format: CliOutputFormat,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: FilterArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    // If --main-line is specified, just output the main line
    if args.main_line {
        let options = OutputOptions {
            format: args.format.into(),
            variations: false,
            result: true,
            ..Default::default()
        };
        let output = to_pgn(&tree, &options);
        io::stdout().write_all(output.as_bytes())?;
        return Ok(());
    }

    // If a path is specified, filter to that subtree
    if let Some(path_str) = &args.path {
        let path = NodePath::parse(path_str)?;
        let nodes = path.resolve_all(&tree);

        if nodes.is_empty() {
            anyhow::bail!("No nodes match path: {}", path_str);
        }

        // Create a new tree with matching nodes
        // For now, just take the first match and output it
        if let Some(node) = nodes.first() {
            let mut subtree = GameTree::new();
            subtree.root = (*node).deep_clone();
            subtree.headers = tree.headers.clone();
            subtree.result = tree.result.clone();

            let options = OutputOptions {
                format: args.format.into(),
                headers: true,
                result: true,
                ..Default::default()
            };
            let output = to_pgn(&subtree, &options);
            io::stdout().write_all(output.as_bytes())?;
        }
        return Ok(());
    }

    // Filter by criteria
    let nag_filter = args.has_nag.as_ref().and_then(|s| {
        Nag::from_symbol(s).or_else(|| Nag::from_dollar_notation(s))
    });

    // Build a filtered tree
    let mut filtered = GameTree::new();
    filtered.headers = tree.headers.clone();
    filtered.result = tree.result.clone();

    // Clone the tree but only include matching nodes
    filter_node(&tree.root, &mut filtered.root, &args, nag_filter, 0);

    let options = OutputOptions {
        format: args.format.into(),
        headers: true,
        result: true,
        ..Default::default()
    };
    let output = to_pgn(&filtered, &options);
    io::stdout().write_all(output.as_bytes())?;

    Ok(())
}

fn filter_node(
    source: &GameNode,
    target: &mut GameNode,
    args: &FilterArgs,
    nag_filter: Option<Nag>,
    depth: usize,
) {
    for child in &source.children {
        let matches = node_matches(child, args, nag_filter, depth + 1);
        let include = if args.invert { !matches } else { matches };

        if include {
            let mut new_child = GameNode::new(&child.san);
            new_child.move_number = child.move_number;
            new_child.is_black = child.is_black;
            new_child.comment = child.comment.clone();
            new_child.nags = child.nags.clone();

            // Recursively filter children
            filter_node(child, &mut new_child, args, nag_filter, depth + 1);

            target.children.push(new_child);
        } else {
            // Even if this node doesn't match, check children
            filter_node(child, target, args, nag_filter, depth + 1);
        }
    }
}

fn node_matches(node: &GameNode, args: &FilterArgs, nag_filter: Option<Nag>, depth: usize) -> bool {
    // Check depth constraints
    if let Some(min) = args.min_depth {
        if depth < min {
            return false;
        }
    }
    if let Some(max) = args.max_depth {
        if depth > max {
            return false;
        }
    }

    // Check has_comment
    if args.has_comment && node.comment.is_empty() {
        return false;
    }

    // Check has_nag
    if let Some(nag) = nag_filter {
        if !node.nags.contains(&nag) {
            return false;
        }
    }

    true
}
