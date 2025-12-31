//! `pgnq stats` command - show detailed statistics

use crate::cli::InputSource;
use crate::error::ParseMode;
use crate::parser::parse_with_options;
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct StatsArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Show move frequency statistics
    #[arg(long)]
    pub move_stats: bool,

    /// Show comment statistics
    #[arg(long)]
    pub comment_stats: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: StatsArgs, _quiet: bool, mode: ParseMode) -> Result<()> {
    let content = args.input.read_to_string()?;
    let file_path = match &args.input {
        InputSource::File(p) => Some(p.clone()),
        InputSource::Stdin => None,
    };
    let tree = parse_with_options(&content, mode, file_path)?;

    // Collect NAG counts
    let mut nag_counts: HashMap<String, usize> = HashMap::new();
    for node in tree.root.iter_dfs() {
        for nag in &node.nags {
            *nag_counts.entry(nag.to_string()).or_insert(0) += 1;
        }
    }

    // Count variations
    let mut variation_count = 0;
    for node in tree.root.iter_dfs() {
        if node.has_variations() {
            variation_count += node.variations().len();
        }
    }

    let output_string = if args.json {
        let stats = serde_json::json!({
            "total_nodes": tree.count_nodes(),
            "leaf_nodes": tree.count_lines(),
            "commented_nodes": tree.count_comments(),
            "max_depth": tree.max_depth(),
            "main_line_length": tree.main_line_length(),
            "variation_count": variation_count,
            "nag_counts": nag_counts,
        });
        format!("{}\n", serde_json::to_string_pretty(&stats)?)
    } else {
        // Text output
        let mut out = String::new();
        out.push_str("Statistics:\n");
        out.push_str(&format!("  Total nodes: {}\n", tree.count_nodes()));
        out.push_str(&format!("  Leaf nodes (lines): {}\n", tree.count_lines()));
        out.push_str(&format!("  Commented nodes: {}\n", tree.count_comments()));
        out.push_str(&format!("  Max depth: {}\n", tree.max_depth()));
        out.push_str(&format!("  Main line length: {}\n", tree.main_line_length()));
        out.push_str(&format!("  Variations: {}\n", variation_count));

        if !nag_counts.is_empty() {
            out.push('\n');
            out.push_str("NAG distribution:\n");
            let mut nags: Vec<_> = nag_counts.iter().collect();
            nags.sort_by(|a, b| b.1.cmp(a.1));
            for (nag, count) in nags.iter().take(10) {
                out.push_str(&format!("  {}: {}\n", nag, count));
            }
        }

        out
    };

    // Write output
    if let Some(path) = args.output {
        fs::write(&path, &output_string)?;
    } else {
        io::stdout().write_all(output_string.as_bytes())?;
    }

    Ok(())
}
