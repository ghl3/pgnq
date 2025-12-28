//! `pgnq extract` command - extract a subtree at a specific path

use crate::cli::{CliOutputFormat, InputSource};
use crate::parser::parse;
use crate::serializer::{to_pgn, OutputOptions};
use crate::tree::{GameTree, NodePath};
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
    subtree.root = node.deep_clone();
    subtree.result = tree.result.clone();

    if args.with_headers {
        subtree.headers = tree.headers.clone();
    }

    // TODO: Handle with_prefix by collecting moves from root to node

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
