//! `pgnq info` command - display information about a PGN file

use crate::cli::InputSource;
use crate::error::ParseMode;
use crate::parser::parse_with_options;
use anyhow::Result;
use clap::Args;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct InfoArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

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

pub fn run(args: InfoArgs, _quiet: bool, mode: ParseMode) -> Result<()> {
    let content = args.input.read_to_string()?;
    let file_path = match &args.input {
        InputSource::File(p) => Some(p.clone()),
        InputSource::Stdin => None,
    };
    let tree = parse_with_options(&content, mode, file_path)?;

    let output_string = if args.json {
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
        format!("{}\n", serde_json::to_string_pretty(&info)?)
    } else {
        // Text output
        let mut out = String::new();
        out.push_str(&format!("File: {}\n", args.input.display_name()));
        out.push('\n');

        out.push_str("Headers:\n");
        let str = tree.seven_tag_roster();
        out.push_str(&format!("  Event: {}\n", str.event));
        out.push_str(&format!("  Site: {}\n", str.site));
        out.push_str(&format!("  Date: {}\n", str.date));
        out.push_str(&format!("  Round: {}\n", str.round));
        out.push_str(&format!("  White: {}\n", str.white));
        out.push_str(&format!("  Black: {}\n", str.black));
        out.push_str(&format!("  Result: {}\n", str.result));

        // Print additional headers
        for (key, value) in &tree.headers {
            if !is_str_header(key) {
                out.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        if !args.headers_only {
            out.push('\n');
            out.push_str("Statistics:\n");
            out.push_str(&format!("  Nodes: {}\n", tree.count_nodes()));
            out.push_str(&format!("  Lines: {}\n", tree.count_lines()));
            out.push_str(&format!("  Comments: {}\n", tree.count_comments()));
            out.push_str(&format!("  Max depth: {}\n", tree.max_depth()));
            out.push_str(&format!("  Main line length: {}\n", tree.main_line_length()));
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

fn is_str_header(key: &str) -> bool {
    matches!(
        key,
        "Event" | "Site" | "Date" | "Round" | "White" | "Black" | "Result"
    )
}
