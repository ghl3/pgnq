//! `pgnq split` command - split a PGN file at multiple node paths

use crate::cli::{CliOutputFormat, InputSource};
use crate::error::ParseMode;
use crate::parser::parse_with_options;
use crate::serializer::{to_pgn, OutputOptions};
use crate::tree::{GameTree, NodePath};
use anyhow::Result;
use clap::Args;
use std::fs;
use std::path::PathBuf;

#[derive(Args)]
pub struct SplitArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Split specification: 'path:name' (can be repeated)
    #[arg(short = 's', long = "split", value_name = "PATH:NAME")]
    pub splits: Vec<String>,

    /// Output directory
    #[arg(short = 'o', long, default_value = "output")]
    pub output_dir: PathBuf,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "standard")]
    pub format: CliOutputFormat,

    /// Include path from root to split point
    #[arg(long)]
    pub include_prefix: bool,

    /// Show what would be created without writing
    #[arg(long)]
    pub dry_run: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: SplitArgs, quiet: bool, mode: ParseMode) -> Result<()> {
    let content = args.input.read_to_string()?;
    let file_path = match &args.input {
        InputSource::File(p) => Some(p.clone()),
        InputSource::Stdin => None,
    };
    let tree = parse_with_options(&content, mode, file_path)?;

    if args.splits.is_empty() {
        anyhow::bail!("No split specifications provided. Use -s 'path:name'");
    }

    // Create output directory if not dry run
    if !args.dry_run {
        fs::create_dir_all(&args.output_dir)?;
    }

    if !quiet {
        println!("Splitting {}...", args.input.display_name());
    }

    let options = OutputOptions {
        format: args.format.into(),
        headers: true,
        result: true,
        ..Default::default()
    };

    let mut success_count = 0;

    for split_spec in &args.splits {
        // Parse split spec "path:name"
        let (path_str, name) = parse_split_spec(split_spec)?;

        let path = NodePath::parse(&path_str)?;
        match path.resolve(&tree) {
            Some(node) => {
                // Create subtree
                let mut subtree = GameTree::new();
                subtree.root = node.deep_clone();
                subtree.result = tree.result.clone();
                subtree.headers = tree.headers.clone();

                let output = to_pgn(&subtree, &options);
                let node_count = node.count_nodes();
                let filename = format!("{}.pgn", name);
                let filepath = args.output_dir.join(&filename);

                if args.dry_run {
                    println!("  {} ({} nodes) [dry run]", filename, node_count);
                } else {
                    fs::write(&filepath, &output)?;
                    if !quiet {
                        println!("  {} ({} nodes) OK", filename, node_count);
                    }
                }
                success_count += 1;
            }
            None => {
                eprintln!("  {}: path not found - {}", name, path_str);
            }
        }
    }

    if !quiet {
        if args.dry_run {
            println!("\nWould create {} files in {}", success_count, args.output_dir.display());
        } else {
            println!("\nCreated {} files in {}", success_count, args.output_dir.display());
        }
    }

    Ok(())
}

/// Parse a split specification like "path:name" or "e4/e5:opening"
fn parse_split_spec(spec: &str) -> Result<(String, String)> {
    // Find the last colon (to allow colons in paths if needed)
    if let Some(colon_idx) = spec.rfind(':') {
        let path = spec[..colon_idx].to_string();
        let name = spec[colon_idx + 1..].to_string();
        if name.is_empty() {
            anyhow::bail!("Empty name in split spec: {}", spec);
        }
        Ok((path, name))
    } else {
        anyhow::bail!("Invalid split spec (missing ':name'): {}", spec);
    }
}
