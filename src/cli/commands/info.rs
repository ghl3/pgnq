//! `pgnq info` command - display information about a PGN file

use crate::cli::InputSource;
use crate::parser::parse;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InfoArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Show headers only
    #[arg(long)]
    pub headers_only: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: InfoArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    if args.json {
        let info = serde_json::json!({
            "file": args.input.display_name(),
            "headers": tree.headers,
            "statistics": {
                "nodes": tree.count_nodes(),
                "lines": tree.count_lines(),
                "comments": tree.count_comments(),
                "max_depth": tree.max_depth(),
                "main_line_length": tree.main_line_length(),
            },
            "result": tree.result.as_str(),
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    // Text output
    println!("File: {}", args.input.display_name());
    println!();

    println!("Headers:");
    let str = tree.seven_tag_roster();
    println!("  Event: {}", str.event);
    println!("  Site: {}", str.site);
    println!("  Date: {}", str.date);
    println!("  Round: {}", str.round);
    println!("  White: {}", str.white);
    println!("  Black: {}", str.black);
    println!("  Result: {}", str.result);

    // Print additional headers
    for (key, value) in &tree.headers {
        if !is_str_header(key) {
            println!("  {}: {}", key, value);
        }
    }

    if !args.headers_only {
        println!();
        println!("Statistics:");
        println!("  Nodes: {}", tree.count_nodes());
        println!("  Lines: {}", tree.count_lines());
        println!("  Comments: {}", tree.count_comments());
        println!("  Max depth: {}", tree.max_depth());
        println!("  Main line length: {}", tree.main_line_length());
    }

    Ok(())
}

fn is_str_header(key: &str) -> bool {
    matches!(
        key,
        "Event" | "Site" | "Date" | "Round" | "White" | "Black" | "Result"
    )
}
