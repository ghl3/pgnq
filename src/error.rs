//! Error types for pgnq

use colored::*;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for pgnq operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid path syntax: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Parse(#[from] ParseError),

    #[error("{0}")]
    Other(String),
}

/// Result type alias using our Error
pub type Result<T> = std::result::Result<T, Error>;

/// Error codes for parse errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// E001: Unclosed brace comment
    UnclosedBraceComment,
    /// E002: Unclosed variation (unbalanced parentheses)
    UnclosedVariation,
    /// E003: Unexpected closing parenthesis
    UnexpectedClosingParen,
    /// E004: Unexpected token
    UnexpectedToken,
    /// E005: Invalid move notation
    InvalidMoveNotation,
    /// E010: Ambiguous move-like token at line start (strict mode)
    AmbiguousMoveAtLineStart,
    /// E011: Ambiguous parenthetical move reference (strict mode)
    AmbiguousParentheticalMove,
    /// E012: Ambiguous list marker could be variation end (strict mode)
    AmbiguousListMarker,
}

impl ErrorCode {
    /// Get the numeric error code string (e.g., "E001")
    pub fn code(&self) -> &'static str {
        match self {
            ErrorCode::UnclosedBraceComment => "E001",
            ErrorCode::UnclosedVariation => "E002",
            ErrorCode::UnexpectedClosingParen => "E003",
            ErrorCode::UnexpectedToken => "E004",
            ErrorCode::InvalidMoveNotation => "E005",
            ErrorCode::AmbiguousMoveAtLineStart => "E010",
            ErrorCode::AmbiguousParentheticalMove => "E011",
            ErrorCode::AmbiguousListMarker => "E012",
        }
    }

    /// Check if this is an ambiguous error (only in strict mode)
    pub fn is_ambiguous(&self) -> bool {
        matches!(
            self,
            ErrorCode::AmbiguousMoveAtLineStart
                | ErrorCode::AmbiguousParentheticalMove
                | ErrorCode::AmbiguousListMarker
        )
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// A highlighted region in a source line
#[derive(Debug, Clone)]
pub struct Highlight {
    /// Column where highlight starts (0-indexed)
    pub start: usize,
    /// Length of the highlight
    pub len: usize,
    /// Label to display under the highlight
    pub label: String,
}

/// A source line with optional highlighting
#[derive(Debug, Clone)]
pub struct SourceLine {
    /// Line number (1-indexed)
    pub line_num: usize,
    /// The actual content of the line
    pub content: String,
    /// Optional highlight on this line
    pub highlight: Option<Highlight>,
}

/// Rich parse error with context and suggestions
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Error code (E001, E002, etc.)
    pub code: ErrorCode,
    /// Short error title
    pub message: String,
    /// Source file path (if known)
    pub file: Option<PathBuf>,
    /// Line number where error occurred (1-indexed)
    pub line: usize,
    /// Column number where error occurred (1-indexed)
    pub column: usize,
    /// Length of the problematic span
    pub span_len: usize,
    /// Surrounding source lines for context
    pub source_context: Vec<SourceLine>,
    /// Additional notes explaining the error
    pub notes: Vec<String>,
    /// Suggestions for how to fix the error
    pub help: Vec<String>,
}

impl ParseError {
    /// Create a new parse error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            file: None,
            line: 1,
            column: 1,
            span_len: 1,
            source_context: Vec::new(),
            notes: Vec::new(),
            help: Vec::new(),
        }
    }

    /// Set the file path
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the location (line and column, both 1-indexed)
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    /// Set the span length
    pub fn with_span(mut self, len: usize) -> Self {
        self.span_len = len;
        self
    }

    /// Add source context lines
    pub fn with_source_context(mut self, lines: Vec<SourceLine>) -> Self {
        self.source_context = lines;
        self
    }

    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add a help suggestion
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    /// Create source context from source text around a given line
    pub fn create_context_from_source(
        source: &str,
        error_line: usize,
        error_column: usize,
        span_len: usize,
        label: &str,
    ) -> Vec<SourceLine> {
        let lines: Vec<&str> = source.lines().collect();
        let mut context = Vec::new();

        // Show line before if exists
        if error_line > 1 && error_line - 2 < lines.len() {
            context.push(SourceLine {
                line_num: error_line - 1,
                content: lines[error_line - 2].to_string(),
                highlight: None,
            });
        }

        // Error line with highlight
        if error_line > 0 && error_line - 1 < lines.len() {
            context.push(SourceLine {
                line_num: error_line,
                content: lines[error_line - 1].to_string(),
                highlight: Some(Highlight {
                    start: error_column.saturating_sub(1),
                    len: span_len,
                    label: label.to_string(),
                }),
            });
        }

        // Show line after if exists
        if error_line < lines.len() {
            context.push(SourceLine {
                line_num: error_line + 1,
                content: lines[error_line].to_string(),
                highlight: None,
            });
        }

        context
    }
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error header: error[E001]: message
        write!(
            f,
            "{}{}{}{} {}",
            "error".red().bold(),
            "[".bold(),
            self.code.code().red().bold(),
            "]".bold(),
            self.message.bold()
        )?;
        writeln!(f)?;

        // Location: --> file:line:column
        let location = if let Some(ref file) = self.file {
            format!("{}:{}:{}", file.display(), self.line, self.column)
        } else {
            format!("<input>:{}:{}", self.line, self.column)
        };
        writeln!(f, "  {} {}", "-->".cyan().bold(), location.cyan())?;

        // Source context
        if !self.source_context.is_empty() {
            // Calculate width needed for line numbers
            let max_line = self.source_context.iter().map(|l| l.line_num).max().unwrap_or(0);
            let line_width = max_line.to_string().len();

            writeln!(f, "{:width$} {}", "", "|".cyan().bold(), width = line_width + 1)?;

            for source_line in &self.source_context {
                // Print line number and content
                writeln!(
                    f,
                    "{:>width$} {} {}",
                    source_line.line_num.to_string().cyan().bold(),
                    "|".cyan().bold(),
                    source_line.content,
                    width = line_width
                )?;

                // Print highlight if present
                if let Some(ref hl) = source_line.highlight {
                    let spaces = " ".repeat(hl.start);
                    let carets = "^".repeat(hl.len.max(1));
                    writeln!(
                        f,
                        "{:width$} {} {}{}",
                        "",
                        "|".cyan().bold(),
                        spaces,
                        format!("{} {}", carets, hl.label).red(),
                        width = line_width
                    )?;
                }
            }

            writeln!(f, "{:width$} {}", "", "|".cyan().bold(), width = line_width + 1)?;
        }

        // Notes
        for note in &self.notes {
            writeln!(f, "   {} {}: {}", "=".cyan().bold(), "note".bold(), note)?;
        }

        // Help suggestions
        for help in &self.help {
            writeln!(f, "   {} {}: {}", "=".cyan().bold(), "help".green().bold(), help)?;
        }

        Ok(())
    }
}

/// Parsing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Lenient mode: apply heuristics for ambiguous patterns (default)
    #[default]
    Lenient,
    /// Strict mode: error on any ambiguous pattern
    Strict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::UnclosedBraceComment.code(), "E001");
        assert_eq!(ErrorCode::UnclosedVariation.code(), "E002");
        assert_eq!(ErrorCode::AmbiguousMoveAtLineStart.code(), "E010");
    }

    #[test]
    fn test_ambiguous_check() {
        assert!(!ErrorCode::UnclosedBraceComment.is_ambiguous());
        assert!(!ErrorCode::UnclosedVariation.is_ambiguous());
        assert!(ErrorCode::AmbiguousMoveAtLineStart.is_ambiguous());
        assert!(ErrorCode::AmbiguousParentheticalMove.is_ambiguous());
    }

    #[test]
    fn test_parse_error_builder() {
        let err = ParseError::new(ErrorCode::UnclosedVariation, "unclosed variation")
            .with_file("test.pgn")
            .with_location(1, 8)
            .with_span(5)
            .with_note("Variation was opened but never closed")
            .with_help("Add ')' to close the variation");

        assert_eq!(err.code, ErrorCode::UnclosedVariation);
        assert_eq!(err.message, "unclosed variation");
        assert_eq!(err.file, Some(PathBuf::from("test.pgn")));
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 8);
        assert_eq!(err.span_len, 5);
        assert_eq!(err.notes.len(), 1);
        assert_eq!(err.help.len(), 1);
    }

    #[test]
    fn test_create_context_from_source() {
        let source = "1. e4 e5\n2. Nf3 (Nc6\n3. Bb5 *";
        let context = ParseError::create_context_from_source(source, 2, 7, 4, "variation starts here");

        assert_eq!(context.len(), 3);
        assert_eq!(context[0].line_num, 1);
        assert_eq!(context[1].line_num, 2);
        assert!(context[1].highlight.is_some());
        assert_eq!(context[2].line_num, 3);
    }

    #[test]
    fn test_parse_error_display() {
        let source = "1. e4 (1... c5\n2. Nf3 *";
        let context = ParseError::create_context_from_source(source, 1, 7, 8, "variation starts here");

        let err = ParseError::new(ErrorCode::UnclosedVariation, "unclosed variation")
            .with_file("game.pgn")
            .with_location(1, 7)
            .with_span(8)
            .with_source_context(context)
            .with_note("Variation was opened at line 1, column 7")
            .with_help("Add ')' to close the variation, or remove the opening '('");

        let output = err.to_string();
        assert!(output.contains("E002"));
        assert!(output.contains("unclosed variation"));
        assert!(output.contains("game.pgn:1:7"));
        assert!(output.contains("variation starts here"));
    }

    #[test]
    fn test_parse_mode_default() {
        assert_eq!(ParseMode::default(), ParseMode::Lenient);
    }
}
