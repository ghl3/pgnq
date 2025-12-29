//! CLI module for pgnq

pub mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, Read};
use std::path::PathBuf;

/// A Unix-like CLI tool for querying and manipulating chess PGN files
#[derive(Parser)]
#[command(
    name = "pgnq",
    author,
    version,
    about = "Query and manipulate chess PGN files",
    long_about = "pgnq parses PGN files into tree structures, enabling powerful \
                  filtering, splitting, and transformation operations. Think of it as jq for chess."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Display information about a PGN file
    Info(commands::InfoArgs),

    /// Display the game tree visually
    Tree(commands::TreeArgs),

    /// Show detailed statistics about a PGN file
    Stats(commands::StatsArgs),

    /// Convert between PGN formats
    Convert(commands::ConvertArgs),

    /// Extract a subtree at a specific path
    Extract(commands::ExtractArgs),

    /// Split a PGN file at multiple node paths
    Split(commands::SplitArgs),

    /// Filter nodes matching specific criteria
    Filter(commands::FilterArgs),

    /// Merge multiple PGN files into a unified tree
    Merge(commands::MergeArgs),
}

/// Input source - either a file path or stdin
#[derive(Debug, Clone)]
pub enum InputSource {
    File(PathBuf),
    Stdin,
}

impl std::str::FromStr for InputSource {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(InputSource::Stdin)
        } else {
            Ok(InputSource::File(PathBuf::from(s)))
        }
    }
}

impl InputSource {
    /// Read the entire contents to a string
    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            InputSource::File(path) => std::fs::read_to_string(path),
            InputSource::Stdin => {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                Ok(buf)
            }
        }
    }

    /// Get the display name for this source
    pub fn display_name(&self) -> String {
        match self {
            InputSource::File(path) => path.display().to_string(),
            InputSource::Stdin => "<stdin>".to_string(),
        }
    }
}

/// Output format enum for CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum CliOutputFormat {
    #[default]
    Standard,
    Lichess,
    Minimal,
    Tree,
}

impl From<CliOutputFormat> for crate::serializer::OutputFormat {
    fn from(f: CliOutputFormat) -> Self {
        match f {
            CliOutputFormat::Standard => crate::serializer::OutputFormat::Standard,
            CliOutputFormat::Lichess => crate::serializer::OutputFormat::Lichess,
            CliOutputFormat::Minimal => crate::serializer::OutputFormat::Minimal,
            CliOutputFormat::Tree => crate::serializer::OutputFormat::Tree,
        }
    }
}

/// Run the CLI
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Info(args) => commands::info::run(args, cli.quiet),
        Command::Tree(args) => commands::tree::run(args, cli.quiet),
        Command::Stats(args) => commands::stats::run(args, cli.quiet),
        Command::Convert(args) => commands::convert::run(args, cli.quiet),
        Command::Extract(args) => commands::extract::run(args, cli.quiet),
        Command::Split(args) => commands::split::run(args, cli.quiet),
        Command::Filter(args) => commands::filter::run(args, cli.quiet),
        Command::Merge(args) => commands::merge::run(args, cli.quiet),
    }
}
