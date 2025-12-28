//! `pgnq convert` command - convert between PGN formats

use crate::cli::{CliOutputFormat, InputSource};
use crate::parser::parse;
use crate::serializer::{to_pgn, OutputOptions};
use anyhow::Result;
use clap::Args;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "standard")]
    pub format: CliOutputFormat,

    /// Omit PGN headers
    #[arg(long)]
    pub no_headers: bool,

    /// Strip all comments
    #[arg(long)]
    pub no_comments: bool,

    /// Keep main line only (no variations)
    #[arg(long)]
    pub no_variations: bool,

    /// Remove NAG annotations
    #[arg(long)]
    pub no_nags: bool,

    /// Remove clock times from comments
    #[arg(long)]
    pub strip_clocks: bool,

    /// Remove evaluations from comments
    #[arg(long)]
    pub strip_evals: bool,

    /// Wrap lines at N characters (0 = no wrapping)
    #[arg(long, default_value = "80")]
    pub line_width: usize,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: ConvertArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    let options = OutputOptions {
        format: args.format.into(),
        headers: !args.no_headers,
        comments: !args.no_comments,
        variations: !args.no_variations,
        nags: !args.no_nags,
        strip_clocks: args.strip_clocks,
        strip_evals: args.strip_evals,
        line_width: args.line_width,
        result: true,
        ..Default::default()
    };

    let output = to_pgn(&tree, &options);

    // Write output
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}
