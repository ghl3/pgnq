//! CLI testing helpers
//!
//! Provides a fluent API for testing pgnq CLI commands.

#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Result from running a pgnq command with fluent assertion methods
#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResult {
    /// Assert the command succeeded (exit code 0)
    pub fn success(&self) -> &Self {
        assert_eq!(
            self.exit_code, 0,
            "Expected success, got exit code {}.\nStderr: {}",
            self.exit_code, self.stderr
        );
        self
    }

    /// Assert the command failed (exit code != 0)
    pub fn failure(&self) -> &Self {
        assert_ne!(
            self.exit_code, 0,
            "Expected failure, got success.\nStdout: {}",
            self.stdout
        );
        self
    }

    /// Assert stdout contains a string
    pub fn stdout_contains(&self, text: &str) -> &Self {
        assert!(
            self.stdout.contains(text),
            "Expected stdout to contain {:?}\nActual stdout:\n{}",
            text,
            self.stdout
        );
        self
    }

    /// Assert stdout does NOT contain a string
    pub fn stdout_not_contains(&self, text: &str) -> &Self {
        assert!(
            !self.stdout.contains(text),
            "Expected stdout to NOT contain {:?}\nActual stdout:\n{}",
            text,
            self.stdout
        );
        self
    }

    /// Assert stderr contains a string
    pub fn stderr_contains(&self, text: &str) -> &Self {
        assert!(
            self.stderr.contains(text),
            "Expected stderr to contain {:?}\nActual stderr:\n{}",
            text,
            self.stderr
        );
        self
    }

    /// Get the PGN output for further assertions
    pub fn pgn(&self) -> PgnOutput<'_> {
        PgnOutput::new(&self.stdout)
    }
}

/// Builder for running pgnq commands with fluent API
pub struct PgnqCommand {
    args: Vec<String>,
    stdin: Option<String>,
}

impl PgnqCommand {
    /// Start building a new pgnq command
    pub fn new(subcommand: &str) -> Self {
        Self {
            args: vec![subcommand.to_string()],
            stdin: None,
        }
    }

    /// Add an argument
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add multiple arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    /// Set stdin input (PGN content) - automatically adds "-" to args
    pub fn stdin(mut self, pgn: &str) -> Self {
        self.stdin = Some(pgn.to_string());
        self.args.push("-".to_string());
        self
    }

    /// Run the command and return the result
    pub fn run(self) -> CommandResult {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--quiet", "--"]);
        cmd.args(&self.args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn pgnq");

        if let Some(input) = &self.stdin {
            let stdin_handle = child.stdin.as_mut().expect("Failed to open stdin");
            stdin_handle
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait for pgnq");

        CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

/// Wrapper for PGN output with assertion helpers
pub struct PgnOutput<'a> {
    content: &'a str,
}

impl<'a> PgnOutput<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    /// Assert the PGN contains a specific move
    pub fn has_move(&self, san: &str) -> &Self {
        assert!(
            self.content.contains(san),
            "Expected PGN to contain move {:?}\nActual PGN:\n{}",
            san,
            self.content
        );
        self
    }

    /// Assert the PGN does NOT contain a specific move
    pub fn not_has_move(&self, san: &str) -> &Self {
        assert!(
            !self.content.contains(san),
            "Expected PGN to NOT contain move {:?}\nActual PGN:\n{}",
            san,
            self.content
        );
        self
    }

    /// Assert the PGN contains a variation (has parentheses)
    pub fn has_variation(&self) -> &Self {
        assert!(
            self.content.contains('('),
            "Expected PGN to contain variations\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN has no variations
    pub fn no_variations(&self) -> &Self {
        assert!(
            !self.content.contains('('),
            "Expected PGN to NOT contain variations\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN contains a comment with specific text
    pub fn has_comment(&self, text: &str) -> &Self {
        assert!(
            self.content.contains(text) && self.content.contains('{'),
            "Expected PGN to contain comment {:?}\nActual PGN:\n{}",
            text,
            self.content
        );
        self
    }

    /// Assert the PGN has no comments (no braces)
    pub fn no_comments(&self) -> &Self {
        assert!(
            !self.content.contains('{'),
            "Expected PGN to NOT contain comments\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN contains a specific NAG
    pub fn has_nag(&self, nag: &str) -> &Self {
        assert!(
            self.content.contains(nag),
            "Expected PGN to contain NAG {:?}\nActual PGN:\n{}",
            nag,
            self.content
        );
        self
    }

    /// Assert the PGN has a specific header value
    pub fn has_header(&self, key: &str, value: &str) -> &Self {
        let pattern = format!("[{} \"{}\"]", key, value);
        assert!(
            self.content.contains(&pattern),
            "Expected PGN to contain header [{} {:?}]\nActual PGN:\n{}",
            key,
            value,
            self.content
        );
        self
    }

    /// Assert the PGN result
    pub fn has_result(&self, result: &str) -> &Self {
        assert!(
            self.content.contains(result),
            "Expected PGN to contain result {:?}\nActual PGN:\n{}",
            result,
            self.content
        );
        self
    }

    /// Assert the main line contains all specified moves
    pub fn main_line_has(&self, moves: &[&str]) -> &Self {
        for m in moves {
            assert!(
                self.content.contains(m),
                "Expected main line to contain {:?}\nActual PGN:\n{}",
                m,
                self.content
            );
        }
        self
    }

    /// Get raw content for custom assertions
    pub fn raw(&self) -> &str {
        self.content
    }
}

/// Convenience function to start building a pgnq command
pub fn pgnq(subcommand: &str) -> PgnqCommand {
    PgnqCommand::new(subcommand)
}

/// Create a temp file with PGN content
pub fn pgn_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write PGN");
    file.flush().expect("Failed to flush");
    file
}
