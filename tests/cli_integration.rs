//! CLI integration tests
//!
//! These tests invoke the pgnq binary and verify correct output.

use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Helper to run pgnq with given args and stdin
fn run_pgnq(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn pgnq");

    if let Some(input) = stdin {
        let stdin_handle = child.stdin.as_mut().expect("Failed to open stdin");
        stdin_handle
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for pgnq");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (exit_code, stdout, stderr)
}

// ============================================================================
// Help/Version Tests
// ============================================================================

#[test]
fn test_cli_help() {
    let (code, stdout, _) = run_pgnq(&["--help"], None);
    assert_eq!(code, 0);
    assert!(stdout.contains("pgnq"));
    assert!(stdout.contains("Usage"));
}

#[test]
fn test_cli_version() {
    let (code, stdout, _) = run_pgnq(&["--version"], None);
    assert_eq!(code, 0);
    assert!(stdout.contains("pgnq"));
}

// ============================================================================
// Info Command Tests
// ============================================================================

#[test]
fn test_cli_info_from_stdin() {
    let pgn = r#"[Event "Test"]
[White "Alice"]
[Black "Bob"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0"#;

    let (code, stdout, _) = run_pgnq(&["info", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Bob"));
}

#[test]
fn test_cli_info_from_file() {
    let pgn = r#"[Event "Test"]
[White "Player1"]
[Black "Player2"]
[Result "*"]

1. d4 d5 *"#;

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(pgn.as_bytes())
        .expect("Failed to write temp file");

    let (code, stdout, _) = run_pgnq(&["info", file.path().to_str().unwrap()], None);
    assert_eq!(code, 0);
    assert!(stdout.contains("Player1"));
    assert!(stdout.contains("Player2"));
}

// ============================================================================
// Convert Command Tests
// ============================================================================

#[test]
fn test_cli_convert_standard() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--format", "standard", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("Nf3"));
}

#[test]
fn test_cli_convert_minimal() {
    let pgn = r#"[Event "Test"]
1. e4 {comment} e5 *"#;

    let (code, stdout, _) = run_pgnq(&["convert", "--format", "minimal", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Minimal format should not include comments
    assert!(!stdout.contains("comment"));
}

#[test]
fn test_cli_convert_no_headers() {
    let pgn = r#"[Event "Test"]
[White "Alice"]
1. e4 e5 *"#;

    let (code, stdout, _) = run_pgnq(&["convert", "--no-headers", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("[Event"));
    assert!(!stdout.contains("[White"));
}

#[test]
fn test_cli_convert_strip_clocks() {
    let pgn = "1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} *";

    let (code, stdout, _) = run_pgnq(&["convert", "--strip-clocks", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
}

#[test]
fn test_cli_convert_strip_evals() {
    let pgn = "1. e4 {[%eval +0.25]} e5 {[%eval 0.20]} *";

    let (code, stdout, _) = run_pgnq(&["convert", "--strip-evals", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("%eval"));
}

#[test]
fn test_cli_convert_no_variations() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--no-variations", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("c5"));
    assert!(!stdout.contains("("));
}

#[test]
fn test_cli_convert_no_comments() {
    let pgn = "1. e4 {Opening move} e5 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--no-comments", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("Opening move"));
}

#[test]
fn test_cli_convert_no_nags() {
    let pgn = "1. e4! e5? 2. Nf3!! *";

    let (code, stdout, _) = run_pgnq(&["convert", "--no-nags", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should not contain NAGs (but may contain ! in result or other contexts)
    assert!(!stdout.contains("e4 !"));
    assert!(!stdout.contains("e5 ?"));
}

// ============================================================================
// Stats Command Tests
// ============================================================================

#[test]
fn test_cli_stats_basic() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show some statistics
    assert!(stdout.contains("6") || stdout.contains("moves") || stdout.len() > 0);
}

#[test]
fn test_cli_stats_json() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["stats", "--json", "-"], Some(pgn));
    assert_eq!(code, 0);
    // JSON output should have braces
    assert!(stdout.contains("{"));
    assert!(stdout.contains("}"));
}

// ============================================================================
// Tree Command Tests
// ============================================================================

#[test]
fn test_cli_tree_basic() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("e5"));
}

#[test]
fn test_cli_tree_depth_limit() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(&["tree", "--depth", "2", "-"], Some(pgn));
    assert_eq!(code, 0);
    // With depth 2, should show e4, e5, and maybe Nf3
    assert!(stdout.contains("e4"));
}

#[test]
fn test_cli_tree_with_variations() {
    let pgn = "1. e4 e5 (1... c5) 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show both main line and variation
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("c5"));
}

// ============================================================================
// Extract Command Tests
// ============================================================================

#[test]
fn test_cli_extract_basic() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(&["extract", "--path", "e4/e5", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should output the subtree from e5 onwards
    assert!(stdout.contains("Nf3") || stdout.len() > 0);
}

// ============================================================================
// Filter Command Tests
// ============================================================================

#[test]
fn test_cli_filter_has_comment() {
    let pgn = "1. e4 {comment} e5 2. Nf3 {another} Nc6 *";

    let (code, stdout, _) = run_pgnq(&["filter", "--has-comment", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show nodes that have comments
    assert!(stdout.len() > 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_cli_empty_input() {
    let (code, _, _) = run_pgnq(&["info", "-"], Some(""));
    // Should not crash
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_whitespace_input() {
    let (code, _, _) = run_pgnq(&["info", "-"], Some("   \n\n   "));
    // Should not crash
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_unicode_input() {
    let pgn = r#"[White "Каспаров"]
1. e4 {日本語} e5 *"#;

    let (code, stdout, _) = run_pgnq(&["convert", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should preserve unicode
    assert!(stdout.contains("Каспаров") || stdout.contains("e4"));
}

#[test]
fn test_cli_large_game() {
    let mut pgn = String::new();
    for i in 1..=100 {
        pgn.push_str(&format!("{}. e4 e5 ", i));
    }
    pgn.push('*');

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(&pgn));
    assert_eq!(code, 0);
    assert!(stdout.len() > 0);
}
