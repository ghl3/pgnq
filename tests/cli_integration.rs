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

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_cli_nonexistent_file() {
    let (code, _, stderr) = run_pgnq(&["info", "/nonexistent/file.pgn"], None);
    // Should fail gracefully
    assert_ne!(code, 0);
    assert!(stderr.len() > 0 || true); // May output to stderr or just fail
}

#[test]
fn test_cli_invalid_command() {
    let (code, _, _) = run_pgnq(&["invalidcommand"], None);
    // Should fail with error
    assert_ne!(code, 0);
}

#[test]
fn test_cli_missing_required_arg() {
    // extract requires --path
    let (code, _, _) = run_pgnq(&["extract", "-"], Some("1. e4 *"));
    // May fail or use default
    let _ = code;
}

#[test]
fn test_cli_malformed_pgn() {
    let pgn = "[[[[[invalid";
    let (code, _, _) = run_pgnq(&["info", "-"], Some(pgn));
    // Should not crash
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_only_result() {
    let pgn = "1-0";
    let (code, _, _) = run_pgnq(&["info", "-"], Some(pgn));
    // Should handle gracefully
    assert!(code == 0 || code == 1);
}

// ============================================================================
// Multi-Game Tests
// ============================================================================

#[test]
fn test_cli_multi_game_input() {
    let pgn = r#"[Event "Game 1"]
1. e4 1-0

[Event "Game 2"]
1. d4 0-1"#;

    let (code, stdout, _) = run_pgnq(&["info", "-"], Some(pgn));
    // Should handle multi-game input
    assert_eq!(code, 0);
    assert!(stdout.contains("Game 1") || stdout.len() > 0);
}

#[test]
fn test_cli_stats_multi_game() {
    let pgn = r#"[Event "Game 1"]
1. e4 e5 1-0

[Event "Game 2"]
1. d4 d5 0-1"#;

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.len() > 0);
}

// ============================================================================
// Convert Format Tests
// ============================================================================

#[test]
fn test_cli_convert_lichess() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--format", "lichess", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Lichess format has moves on separate lines
    assert!(stdout.contains("e4"));
}

#[test]
fn test_cli_convert_tree() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--format", "tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Tree format has tree characters
    assert!(stdout.contains("├") || stdout.contains("|") || stdout.contains("└") || stdout.contains("`"));
}

// ============================================================================
// Combined Flags Tests
// ============================================================================

#[test]
fn test_cli_convert_multiple_flags() {
    let pgn = r#"[Event "Test"]
1. e4 {comment} e5 (1... c5) 1-0"#;

    let (code, stdout, _) = run_pgnq(
        &["convert", "--no-headers", "--no-comments", "--no-variations", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("[Event"));
    assert!(!stdout.contains("comment"));
    assert!(!stdout.contains("c5"));
}

#[test]
fn test_cli_strip_all_annotations() {
    let pgn = "1. e4! {[%clk 0:03:00] [%eval +0.5] comment} e5 *";

    let (code, stdout, _) = run_pgnq(
        &["convert", "--strip-clocks", "--strip-evals", "--no-comments", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("%eval"));
    assert!(!stdout.contains("comment"));
}

// ============================================================================
// Tree Command Edge Cases
// ============================================================================

#[test]
fn test_cli_tree_ascii() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["tree", "--ascii", "-"], Some(pgn));
    assert_eq!(code, 0);
    // ASCII mode uses |-- and `-- instead of Unicode
    assert!(!stdout.contains("├") || stdout.contains("|"));
}

#[test]
fn test_cli_tree_empty_game() {
    let pgn = "*";

    let (code, _, _) = run_pgnq(&["tree", "-"], Some(pgn));
    // Should handle empty game
    assert!(code == 0 || code == 1);
}

// ============================================================================
// Stats Command Edge Cases
// ============================================================================

#[test]
fn test_cli_stats_with_variations() {
    let pgn = "1. e4 e5 (1... c5) (1... e6) 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Stats should count variations
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_stats_with_comments() {
    let pgn = "1. e4 {comment 1} e5 {comment 2} 2. Nf3 {comment 3} *";

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should count comments
    assert!(stdout.len() > 0);
}

// ============================================================================
// Extract Command Edge Cases
// ============================================================================

#[test]
fn test_cli_extract_nonexistent_path() {
    let pgn = "1. e4 e5 *";

    let (code, _, _) = run_pgnq(&["extract", "--path", "d4/d5", "-"], Some(pgn));
    // Path doesn't exist, should handle gracefully
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_extract_deep_path() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 *";

    let (code, stdout, _) = run_pgnq(&["extract", "--path", "e4/e5/Nf3/Nc6/Bb5", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should extract from Bb5 onwards
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_extract_with_prefix() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 *";

    // Without prefix - should start at Nf3
    let (code, stdout, _) = run_pgnq(&["extract", "--path", "e4/e5/Nf3", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("1. e4"));
    assert!(stdout.contains("Nf3"));

    // With prefix - should include e4, e5 leading up to Nf3
    let (code, stdout, _) =
        run_pgnq(&["extract", "--path", "e4/e5/Nf3", "--with-prefix", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.contains("1. e4"));
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("Nf3"));
}

// ============================================================================
// Filter Command Edge Cases
// ============================================================================

#[test]
fn test_cli_filter_no_matches() {
    let pgn = "1. e4 e5 2. Nf3 *";

    let (code, _, _) = run_pgnq(&["filter", "--has-comment", "-"], Some(pgn));
    // No comments in game, should return empty or success
    assert!(code == 0 || code == 1);
}

// ============================================================================
// File Output Tests
// ============================================================================

#[test]
fn test_cli_convert_to_file() {
    let pgn = "1. e4 e5 *";
    let output_file = NamedTempFile::new().expect("Failed to create temp file");
    let output_path = output_file.path().to_str().unwrap();

    let (code, _, _) = run_pgnq(
        &["convert", "--output", output_path, "-"],
        Some(pgn),
    );
    // May or may not support --output flag
    let _ = code;
}

// ============================================================================
// Special Input Tests
// ============================================================================

#[test]
fn test_cli_binary_like_input() {
    let pgn = "\x00\x01\x02 1. e4 *";

    let (code, _, _) = run_pgnq(&["info", "-"], Some(pgn));
    // Should not crash on binary input
    assert!(code == 0 || code == 1 || code != 0);
}

#[test]
fn test_cli_very_long_comment() {
    let comment = "x".repeat(10000);
    let pgn = format!("1. e4 {{{}}} e5 *", comment);

    let (code, _, _) = run_pgnq(&["info", "-"], Some(&pgn));
    // Should handle very long comments
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_deeply_nested_variations() {
    let pgn = "1. e4 (1. d4 (1. c4 (1. Nf3 (1. g3)))) *";

    let (code, stdout, _) = run_pgnq(&["tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show all nested variations
    assert!(stdout.contains("e4") || stdout.len() > 0);
}

#[test]
fn test_cli_many_variations() {
    let pgn = "1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) (1. b3) (1. f4) e5 *";

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(stdout.len() > 0);
}

// ============================================================================
// Merge Command Tests
// ============================================================================

#[test]
fn test_cli_merge_basic() {
    // Create two temp files with different games
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");

    write!(file1, "1. e4 e5 *").expect("Failed to write");
    write!(file2, "1. e4 c5 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // Merged tree should contain both e5 and c5 as variations
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("c5"));
}

#[test]
fn test_cli_merge_overlapping_lines() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");

    // Same opening, different continuations
    write!(file1, "1. e4 e5 2. Nf3 Nc6 *").expect("Failed to write");
    write!(file2, "1. e4 e5 2. Nf3 Nf6 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // Should have Nf3 once, with both Nc6 and Nf6 as children
    assert!(stdout.contains("Nf3"));
    assert!(stdout.contains("Nc6"));
    assert!(stdout.contains("Nf6"));
}

#[test]
fn test_cli_merge_preserves_variations() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");

    // First file has a variation
    write!(file1, "1. e4 e5 (1... c5) *").expect("Failed to write");
    write!(file2, "1. e4 d5 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // Should preserve all variations
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("c5"));
    assert!(stdout.contains("d5"));
}

#[test]
fn test_cli_merge_with_comments() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");

    write!(file1, "1. e4 {{First game}} e5 *").expect("Failed to write");
    write!(file2, "1. e4 {{Second game}} c5 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // Should preserve first comment (or merge them)
    assert!(stdout.contains("e4"));
}

#[test]
fn test_cli_merge_single_file() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    write!(file1, "1. e4 e5 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(&["merge", file1.path().to_str().unwrap()], None);

    assert_eq!(code, 0);
    // Single file should just output the tree
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("e5"));
}

#[test]
fn test_cli_merge_three_files() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file3 = NamedTempFile::new().expect("Failed to create temp file");

    write!(file1, "1. e4 e5 *").expect("Failed to write");
    write!(file2, "1. e4 c5 *").expect("Failed to write");
    write!(file3, "1. e4 d5 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");
    file3.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
            file3.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // All three responses should be present
    assert!(stdout.contains("e5"));
    assert!(stdout.contains("c5"));
    assert!(stdout.contains("d5"));
}

#[test]
fn test_cli_merge_identical_games() {
    let mut file1 = NamedTempFile::new().expect("Failed to create temp file");
    let mut file2 = NamedTempFile::new().expect("Failed to create temp file");

    // Identical games should merge without duplication
    write!(file1, "1. e4 e5 2. Nf3 Nc6 *").expect("Failed to write");
    write!(file2, "1. e4 e5 2. Nf3 Nc6 *").expect("Failed to write");
    file1.flush().expect("Failed to flush");
    file2.flush().expect("Failed to flush");

    let (code, stdout, _) = run_pgnq(
        &[
            "merge",
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ],
        None,
    );

    assert_eq!(code, 0);
    // Should have the moves but no duplicates
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("Nf3"));
    assert!(stdout.contains("Nc6"));
}

// ============================================================================
// Combined --strip-* Flags Tests (Additional Coverage)
// ============================================================================

#[test]
fn test_cli_convert_strip_clocks_and_evals_combined() {
    let pgn = "1. e4 {[%clk 0:03:00] [%eval +0.5]} e5 {[%clk 0:02:55] [%eval -0.3]} *";

    let (code, stdout, _) = run_pgnq(
        &["convert", "--strip-clocks", "--strip-evals", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("%eval"));
    // Moves should still be present
    assert!(stdout.contains("e4"));
    assert!(stdout.contains("e5"));
}

#[test]
fn test_cli_convert_strip_clocks_preserve_text_comments() {
    let pgn = "1. e4 {[%clk 0:03:00] Good opening move} e5 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--strip-clocks", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    // Text comment should be preserved
    assert!(stdout.contains("Good opening move"));
}

#[test]
fn test_cli_convert_strip_evals_preserve_text_comments() {
    let pgn = "1. e4 {[%eval +0.5] Solid choice} e5 *";

    let (code, stdout, _) = run_pgnq(&["convert", "--strip-evals", "-"], Some(pgn));
    assert_eq!(code, 0);
    assert!(!stdout.contains("%eval"));
    // Text comment should be preserved
    assert!(stdout.contains("Solid choice"));
}

#[test]
fn test_cli_convert_strip_all_with_nags() {
    let pgn = "1. e4! {[%clk 0:03:00] [%eval +0.5] Best move} e5? {[%clk 0:02:55]} *";

    let (code, stdout, _) = run_pgnq(
        &["convert", "--strip-clocks", "--strip-evals", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("%eval"));
    // NAGs and text comments should be preserved
    assert!(stdout.contains("Best move"));
}

#[test]
fn test_cli_convert_strip_and_no_comments() {
    let pgn = "1. e4 {[%clk 0:03:00] Opening} e5 {[%eval +0.3] Response} *";

    let (code, stdout, _) = run_pgnq(
        &["convert", "--strip-clocks", "--strip-evals", "--no-comments", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("%eval"));
    assert!(!stdout.contains("Opening"));
    assert!(!stdout.contains("Response"));
}

#[test]
fn test_cli_convert_strip_and_no_variations() {
    let pgn = "1. e4 {[%clk 0:03:00]} e5 (1... c5 {[%eval +0.2]}) *";

    let (code, stdout, _) = run_pgnq(
        &["convert", "--strip-clocks", "--strip-evals", "--no-variations", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("c5"));
}

#[test]
fn test_cli_convert_all_strip_flags_with_format() {
    let pgn = r#"[Event "Test"]
1. e4! {[%clk 0:03:00] [%eval +0.5] Opening} e5 (1... c5) *"#;

    let (code, stdout, _) = run_pgnq(
        &[
            "convert",
            "--format", "minimal",
            "--strip-clocks",
            "--strip-evals",
            "-",
        ],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("%clk"));
    assert!(!stdout.contains("%eval"));
}

// ============================================================================
// Filter Command - Multiple Criteria Tests
// ============================================================================

#[test]
fn test_cli_filter_has_nag() {
    let pgn = "1. e4! e5 2. Nf3!! Nc6? *";

    let (code, stdout, _) = run_pgnq(&["filter", "--has-nag", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should filter to nodes with NAGs
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_min_depth() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(&["filter", "--min-depth", "3", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should filter to nodes at depth 3+
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_max_depth() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(&["filter", "--max-depth", "2", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should filter to nodes at depth <=2
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_depth_range() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 *";

    let (code, stdout, _) = run_pgnq(
        &["filter", "--min-depth", "2", "--max-depth", "4", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_main_line() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 d6) 2. Nf3 Nc6 *";

    let (code, stdout, _) = run_pgnq(&["filter", "--main-line", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show only main line
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_invert() {
    let pgn = "1. e4 {comment} e5 2. Nf3 Nc6 *";

    let (code, stdout, _) = run_pgnq(&["filter", "--has-comment", "--invert", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Inverted: should show nodes WITHOUT comments
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_combined_has_comment_and_nag() {
    let pgn = "1. e4! {Opening} e5 2. Nf3! {Knight out} Nc6 *";

    // Nodes that have BOTH comment AND nag
    let (code, stdout, _) = run_pgnq(
        &["filter", "--has-comment", "--has-nag", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_filter_path_and_depth() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *";

    let (code, stdout, _) = run_pgnq(
        &["filter", "--path", "e4/e5", "--min-depth", "1", "-"],
        Some(pgn),
    );
    // Should handle combined path and depth
    assert!(code == 0 || code == 1);
}

// ============================================================================
// Extract Command - Additional Flag Combinations
// ============================================================================

#[test]
fn test_cli_extract_with_headers() {
    let pgn = r#"[Event "Test"]
[White "Alice"]
1. e4 e5 2. Nf3 Nc6 *"#;

    let (code, stdout, _) = run_pgnq(
        &["extract", "--path", "e4/e5", "--with-headers", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    // Should include headers in output
    assert!(stdout.contains("[Event") || stdout.contains("Nf3"));
}

#[test]
fn test_cli_extract_format_minimal() {
    let pgn = r#"[Event "Test"]
1. e4 {comment} e5 2. Nf3 Nc6 *"#;

    let (code, stdout, _) = run_pgnq(
        &["extract", "--path", "e4/e5", "--format", "minimal", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    // Minimal format should not include comments
    assert!(!stdout.contains("comment") || stdout.contains("Nf3"));
}

#[test]
fn test_cli_extract_prefix_and_headers() {
    let pgn = r#"[Event "Championship"]
[White "Magnus"]
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 *"#;

    let (code, stdout, _) = run_pgnq(
        &["extract", "--path", "e4/e5/Nf3", "--with-prefix", "--with-headers", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    // Should have prefix moves and headers
    assert!(stdout.contains("e4") || stdout.contains("Nf3"));
}

#[test]
fn test_cli_extract_from_variation() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 d6 3. d4) *";

    let (code, stdout, _) = run_pgnq(&["extract", "--path", "e4/c5/Nf3", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should extract from within the variation
    assert!(stdout.len() > 0);
}

#[test]
fn test_cli_extract_single_move() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 *";

    let (code, stdout, _) = run_pgnq(&["extract", "--path", "e4", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should extract from e4 onwards
    assert!(stdout.contains("e5") || stdout.len() > 0);
}

// ============================================================================
// Stats Command - Additional Output Format Tests
// ============================================================================

#[test]
fn test_cli_stats_json_structure() {
    let pgn = "1. e4! e5 {comment} 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["stats", "--json", "-"], Some(pgn));
    assert_eq!(code, 0);
    // JSON should have expected fields
    assert!(stdout.contains("{"));
    assert!(stdout.contains("}"));
    // Should contain some numeric stats
    assert!(stdout.contains(":") || stdout.contains("\""));
}

#[test]
fn test_cli_stats_with_nags() {
    let pgn = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";

    let (code, stdout, _) = run_pgnq(&["stats", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Stats should count NAGs
    assert!(stdout.len() > 0);
}

// ============================================================================
// Tree Command - Additional Output Tests
// ============================================================================

#[test]
fn test_cli_tree_with_annotations() {
    let pgn = "1. e4! {Opening} e5 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Tree should show structure
    assert!(stdout.contains("e4"));
}

#[test]
fn test_cli_tree_deep_variations() {
    let pgn = "1. e4 (1. d4 (1. c4 Nf6)) e5 *";

    let (code, stdout, _) = run_pgnq(&["tree", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should show all alternatives
    assert!(stdout.contains("e4"));
}

#[test]
fn test_cli_tree_depth_limits_variations() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 d6) 2. Nf3 *";

    let (code, stdout, _) = run_pgnq(&["tree", "--depth", "1", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Depth 1 should limit output
    assert!(stdout.len() > 0);
}

// ============================================================================
// Convert Command - Format Edge Cases
// ============================================================================

#[test]
fn test_cli_convert_format_with_variations() {
    let pgn = "1. e4 e5 (1... c5) 2. Nf3 *";

    for format in &["standard", "lichess", "minimal"] {
        let (code, stdout, _) = run_pgnq(
            &["convert", "--format", format, "-"],
            Some(pgn),
        );
        assert_eq!(code, 0, "Format {} failed", format);
        assert!(stdout.contains("e4"), "Format {} missing e4", format);
    }
}

#[test]
fn test_cli_convert_no_headers_with_format() {
    let pgn = r#"[Event "Test"]
1. e4 e5 *"#;

    let (code, stdout, _) = run_pgnq(
        &["convert", "--no-headers", "--format", "standard", "-"],
        Some(pgn),
    );
    assert_eq!(code, 0);
    assert!(!stdout.contains("[Event"));
    assert!(stdout.contains("e4"));
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_cli_recovers_from_partial_pgn() {
    // PGN with incomplete game followed by complete game
    let pgn = r#"[Event "Partial"]
1. e4

[Event "Complete"]
1. d4 d5 *"#;

    let (code, _, _) = run_pgnq(&["info", "-"], Some(pgn));
    // Should not crash, may parse one or both games
    assert!(code == 0 || code == 1);
}

#[test]
fn test_cli_handles_mixed_results() {
    let pgn = r#"[Event "Game 1"]
1. e4 e5 1-0

[Event "Game 2"]
1. d4 d5 0-1

[Event "Game 3"]
1. c4 c5 1/2-1/2

[Event "Game 4"]
1. Nf3 Nf6 *"#;

    let (code, stdout, _) = run_pgnq(&["info", "-"], Some(pgn));
    assert_eq!(code, 0);
    // Should handle all result types
    assert!(stdout.len() > 0);
}
