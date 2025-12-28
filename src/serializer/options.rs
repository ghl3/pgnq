//! Output options for PGN serialization

use serde::{Deserialize, Serialize};

/// Output format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OutputFormat {
    #[default]
    Standard,
    Lichess,
    Tree,
    Minimal,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" | "std" => Ok(OutputFormat::Standard),
            "lichess" | "lic" => Ok(OutputFormat::Lichess),
            "tree" => Ok(OutputFormat::Tree),
            "minimal" | "min" => Ok(OutputFormat::Minimal),
            _ => Err(format!("unknown format: {}", s)),
        }
    }
}

/// Options for controlling output
#[derive(Debug, Clone)]
pub struct OutputOptions {
    /// Output format
    pub format: OutputFormat,

    /// Include PGN headers
    pub headers: bool,

    /// Include comments
    pub comments: bool,

    /// Include variations (RAVs)
    pub variations: bool,

    /// Include NAG annotations
    pub nags: bool,

    /// Strip clock times from comments
    pub strip_clocks: bool,

    /// Strip evaluations from comments
    pub strip_evals: bool,

    /// Maximum line width (0 = no wrapping)
    pub line_width: usize,

    /// Indentation for tree view
    pub indent: usize,

    /// Maximum depth to output (0 = unlimited)
    pub max_depth: usize,

    /// Include result termination marker
    pub result: bool,

    /// Use ASCII characters only (no Unicode box drawing)
    pub ascii: bool,

    /// Show comments in tree view
    pub show_comments: bool,

    /// Show NAGs in tree view
    pub show_nags: bool,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Standard,
            headers: true,
            comments: true,
            variations: true,
            nags: true,
            strip_clocks: false,
            strip_evals: false,
            line_width: 80,
            indent: 2,
            max_depth: 0,
            result: true,
            ascii: false,
            show_comments: false,
            show_nags: false,
        }
    }
}

impl OutputOptions {
    /// Create options for minimal output
    pub fn minimal() -> Self {
        Self {
            format: OutputFormat::Minimal,
            headers: false,
            comments: false,
            variations: false,
            nags: false,
            strip_clocks: true,
            strip_evals: true,
            line_width: 0,
            indent: 0,
            max_depth: 0,
            result: true,
            ascii: false,
            show_comments: false,
            show_nags: false,
        }
    }

    /// Create options for tree view
    pub fn tree() -> Self {
        Self {
            format: OutputFormat::Tree,
            line_width: 0,
            max_depth: 5,
            ..Default::default()
        }
    }

    /// Create options for Lichess format
    pub fn lichess() -> Self {
        Self {
            format: OutputFormat::Lichess,
            ..Default::default()
        }
    }
}
