//! Error handling tests
//!
//! Tests for graceful error handling and recovery from malformed input.

mod common;

use common::try_parse_pgn;
use pgnq::parser::parse;

// ============================================================================
// Malformed Input Tests
// ============================================================================

#[test]
fn test_parse_empty_input() {
    // Empty input should still produce a valid (empty) tree
    let result = parse("");
    assert!(result.is_ok());
    let tree = result.unwrap();
    assert!(tree.root.children.is_empty());
}

#[test]
fn test_parse_whitespace_only() {
    let result = parse("   \n\n   \t   ");
    assert!(result.is_ok());
    let tree = result.unwrap();
    assert!(tree.root.children.is_empty());
}

#[test]
fn test_parse_only_newlines() {
    let result = parse("\n\n\n");
    assert!(result.is_ok());
}

// ============================================================================
// Header Edge Cases
// ============================================================================

#[test]
fn test_parse_unclosed_header() {
    // Unclosed header - parser should handle gracefully
    let result = parse(r#"[Event "Test"#);
    // May fail or succeed depending on parser leniency
    // We're being accepting, so it should not panic
    let _ = result; // Either Ok or Err is acceptable, but no panic
}

#[test]
fn test_parse_header_missing_value() {
    let result = parse("[Event]");
    // Should not panic
    let _ = result;
}

#[test]
fn test_parse_header_empty_value() {
    let result = parse(r#"[Event ""]"#);
    assert!(result.is_ok());
}

#[test]
fn test_parse_header_no_quotes() {
    let result = parse("[Event Test]");
    // Should not panic
    let _ = result;
}

#[test]
fn test_parse_malformed_header_bracket() {
    let result = parse("[Event \"Test\" extra]");
    // Should not panic
    let _ = result;
}

// ============================================================================
// Move Parsing Edge Cases
// ============================================================================

#[test]
fn test_parse_invalid_move_notation() {
    // Invalid move that doesn't match any pattern
    let result = parse("1. xyz123");
    // Parser should skip unknown tokens gracefully
    let _ = result;
}

#[test]
fn test_parse_move_without_number() {
    let result = parse("e4 e5 Nf3 Nc6");
    assert!(result.is_ok());
}

#[test]
fn test_parse_double_move_number() {
    let result = parse("1. 1. e4");
    // Should not panic
    let _ = result;
}

#[test]
fn test_parse_very_high_move_number() {
    let result = parse("999. e4");
    assert!(result.is_ok());
}

// ============================================================================
// Variation Edge Cases
// ============================================================================

#[test]
fn test_parse_unbalanced_open_paren() {
    let result = parse("1. e4 (1... c5 e5");
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_parse_unbalanced_close_paren() {
    let result = parse("1. e4 e5) Nf3");
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_parse_empty_variation() {
    let result = parse("1. e4 () e5");
    assert!(result.is_ok());
}

#[test]
fn test_parse_nested_empty_variations() {
    let result = parse("1. e4 (()) e5");
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_parse_variation_at_start() {
    let result = parse("(1. e4) 1. d4");
    // Should handle gracefully
    let _ = result;
}

// ============================================================================
// Comment Edge Cases
// ============================================================================

#[test]
fn test_parse_unclosed_brace_comment() {
    let result = parse("1. e4 {This comment is never closed e5");
    // Should handle gracefully - might consume rest as comment
    let _ = result;
}

#[test]
fn test_parse_nested_braces_in_comment() {
    let result = parse("1. e4 {outer {inner} still outer} e5");
    assert!(result.is_ok());
}

#[test]
fn test_parse_special_chars_in_comment() {
    let result = parse("1. e4 {Special: <>[]{}!@#$%^&*()} e5");
    assert!(result.is_ok());
}

#[test]
fn test_parse_very_long_comment() {
    let long_comment = "x".repeat(10000);
    let pgn = format!("1. e4 {{{}}} e5", long_comment);
    let result = parse(&pgn);
    assert!(result.is_ok());
}

// ============================================================================
// NAG Edge Cases
// ============================================================================

#[test]
fn test_parse_invalid_nag_number() {
    let result = parse("1. e4 $999999");
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_parse_nag_zero() {
    let result = parse("1. e4 $0");
    // $0 is valid but unusual
    let _ = result;
}

#[test]
fn test_parse_many_nags_on_one_move() {
    let result = parse("1. e4! ? !! ?? !? ?! $1 $2 $3 $4");
    assert!(result.is_ok());
}

// ============================================================================
// Result Marker Edge Cases
// ============================================================================

#[test]
fn test_parse_result_at_start() {
    let result = parse("1-0");
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_results() {
    let result = parse("1. e4 1-0 0-1 1/2-1/2");
    // Last result should win
    assert!(result.is_ok());
}

#[test]
fn test_parse_result_in_middle() {
    let result = parse("1. e4 1-0 e5 2. Nf3");
    // Should handle gracefully
    let _ = result;
}

// ============================================================================
// Mixed Edge Cases
// ============================================================================

#[test]
fn test_parse_random_garbage() {
    let result = parse("!@#$%^&*()_+{}|:\"<>?");
    // Should not panic
    let _ = result;
}

#[test]
fn test_parse_binary_like_input() {
    let result = parse("\x00\x01\x02\x03");
    // Should not panic
    let _ = result;
}

#[test]
fn test_parse_very_deep_nesting() {
    let mut pgn = "1. e4".to_string();
    for i in 0..50 {
        pgn.push_str(&format!(" ({}. d4", i + 1));
    }
    for _ in 0..50 {
        pgn.push(')');
    }
    let result = parse(&pgn);
    // Should handle deep nesting without stack overflow
    assert!(result.is_ok());
}

#[test]
fn test_parse_very_long_game() {
    let mut pgn = String::new();
    for i in 1..=500 {
        if i % 2 == 1 {
            pgn.push_str(&format!("{}. e4 ", (i + 1) / 2));
        } else {
            pgn.push_str("e5 ");
        }
    }
    let result = parse(&pgn);
    assert!(result.is_ok());
}

// ============================================================================
// Unicode Edge Cases
// ============================================================================

#[test]
fn test_parse_unicode_in_headers() {
    let result = parse(r#"[White "Каспаров"] 1. e4"#);
    assert!(result.is_ok());
}

#[test]
fn test_parse_unicode_in_comments() {
    let result = parse("1. e4 {日本語コメント} e5");
    assert!(result.is_ok());
}

#[test]
fn test_parse_emoji_in_comment() {
    let result = parse("1. e4 {Great move! 🎉} e5");
    assert!(result.is_ok());
}

// ============================================================================
// Successful Parse Verification
// ============================================================================

#[test]
fn test_result_is_ok_for_valid_games() {
    let valid_games = vec![
        "1. e4 e5 2. Nf3 Nc6 1-0",
        "1. d4 d5 2. c4 *",
        "[Event \"Test\"] 1. e4",
        "1. e4 {comment} e5",
        "1. e4! e5? 2. Nf3",
        "1. e4 e5 (1... c5) 2. Nf3",
    ];

    for pgn in valid_games {
        let result = try_parse_pgn(pgn);
        assert!(result.is_ok(), "Failed to parse: {}", pgn);
    }
}
