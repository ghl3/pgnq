//! `pgnq stats` command - show detailed statistics

use crate::cli::InputSource;
use crate::parser::parse;
use anyhow::Result;
use clap::Args;
use std::collections::HashMap;

#[derive(Args)]
pub struct StatsArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

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

pub fn run(args: StatsArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

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

    if args.json {
        let stats = serde_json::json!({
            "total_nodes": tree.count_nodes(),
            "leaf_nodes": tree.count_lines(),
            "commented_nodes": tree.count_comments(),
            "max_depth": tree.max_depth(),
            "main_line_length": tree.main_line_length(),
            "variation_count": variation_count,
            "nag_counts": nag_counts,
        });
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    // Text output
    println!("Statistics:");
    println!("  Total nodes: {}", tree.count_nodes());
    println!("  Leaf nodes (lines): {}", tree.count_lines());
    println!("  Commented nodes: {}", tree.count_comments());
    println!("  Max depth: {}", tree.max_depth());
    println!("  Main line length: {}", tree.main_line_length());
    println!("  Variations: {}", variation_count);

    if !nag_counts.is_empty() {
        println!();
        println!("NAG distribution:");
        let mut nags: Vec<_> = nag_counts.iter().collect();
        nags.sort_by(|a, b| b.1.cmp(a.1));
        for (nag, count) in nags.iter().take(10) {
            println!("  {}: {}", nag, count);
        }
    }

    Ok(())
}
