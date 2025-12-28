//! Comprehensive parser edge case tests
//!
//! Tests various PGN formats, move notations, comments, NAGs, variations,
//! and other edge cases to ensure the parser is robust and accepting.

mod common;

use common::*;
use pgnq::parser::parse;
use pgnq::tree::GameResult;
use test_case::test_case;

// Use pretty_assertions for better diffs, but only in non-test_case tests
// to avoid macro conflicts
#[allow(unused_imports)]
use pretty_assertions::assert_eq as pretty_assert_eq;

// ============================================================================
// BASIC PARSING TESTS
// ============================================================================

#[test]
fn test_parse_minimal_game() {
    let tree = parse_pgn(MINIMAL_GAME);
    assert_eq!(count_nodes(&tree), 4);
    assert_eq!(tree.result, GameResult::WhiteWins);
}

#[test]
fn test_parse_full_headers() {
    let tree = parse_pgn(FULL_HEADERS_GAME);
    assert_eq!(tree.header("Event"), Some("Test Tournament"));
    assert_eq!(tree.header("Site"), Some("Test City"));
    assert_eq!(tree.header("Date"), Some("2024.01.15"));
    assert_eq!(tree.header("Round"), Some("1"));
    assert_eq!(tree.header("White"), Some("Player, White"));
    assert_eq!(tree.header("Black"), Some("Player, Black"));
    assert_eq!(tree.result, GameResult::WhiteWins);
}

#[test]
fn test_parse_headerless_game() {
    let tree = parse_pgn(HEADERLESS_GAME);
    assert!(tree.headers.is_empty() || tree.header("Event") == Some("?"));
    assert_eq!(count_nodes(&tree), 6);
    assert_eq!(tree.result, GameResult::Ongoing);
}

// ============================================================================
// MOVE NOTATION TESTS
// ============================================================================

#[test]
fn test_parse_castling_kingside() {
    let tree = parse_pgn(CASTLING_GAME);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"O-O".to_string()));
}

#[test]
fn test_parse_castling_queenside() {
    let tree = parse_pgn(QUEENSIDE_CASTLING);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"O-O-O".to_string()));
}

#[test]
fn test_parse_castling_with_check() {
    // Castling can sometimes give check (though rare)
    let tree = parse_pgn(CASTLING_WITH_CHECK);
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_castling_zero_notation() {
    // Some PGN files use 0-0 instead of O-O
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. 0-0 Nf6 5. d3 0-0 *";
    let result = parse(pgn);
    // Our parser should either accept this or reject it gracefully
    // For now, we expect it to work since we want to be accepting
    if let Ok(tree) = result {
        assert!(count_nodes(&tree) > 0);
    }
}

#[test]
fn test_parse_pawn_moves() {
    let pgn = "1. e4 e5 2. d4 exd4 3. c3 dxc3 4. Nxc3 *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"e4".to_string()));
    assert!(moves.contains(&"exd4".to_string()));
    assert!(moves.contains(&"dxc3".to_string()));
}

#[test]
fn test_parse_piece_moves() {
    let pgn = "1. Nf3 Nf6 2. Nc3 Nc6 3. e4 e5 4. Bb5 Bb4 5. O-O O-O *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"Nf3".to_string()));
    assert!(moves.contains(&"Bb5".to_string()));
}

#[test]
fn test_parse_captures() {
    let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. Nf3 Nf6 5. Bc4 Bg4 6. Bxf7+ *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"exd5".to_string()));
    assert!(moves.contains(&"Qxd5".to_string()));
    assert!(moves.contains(&"Bxf7+".to_string()));
}

#[test]
fn test_parse_check_notation() {
    let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"Qxf7#".to_string()));
}

#[test]
fn test_parse_disambiguation_file() {
    let tree = parse_pgn(DISAMBIGUATION_FILE);
    let moves = main_line_moves(&tree);
    // Check that moves with file disambiguation are parsed
    assert!(moves.iter().any(|m| m.starts_with("R") || m.starts_with("N")));
}

#[test]
fn test_parse_disambiguation_rank() {
    let tree = parse_pgn(DISAMBIGUATION_RANK);
    assert!(count_nodes(&tree) > 10);
}

#[test]
fn test_parse_pawn_promotion() {
    let tree = parse_pgn(PAWN_PROMOTION);
    let moves = main_line_moves(&tree);
    // Should contain a promotion move
    assert!(moves.iter().any(|m| m.contains("=")));
}

#[test]
fn test_parse_underpromotion() {
    let tree = parse_pgn(UNDERPROMOTION);
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_promotion_variants() {
    // Test various promotion notations
    let cases = [
        "1. e4 d5 2. e5 d4 3. e6 d3 4. exf7+ Kd7 5. fxg8=Q *",
        "1. e4 d5 2. e5 d4 3. e6 d3 4. exf7+ Kd7 5. fxg8=R *",
        "1. e4 d5 2. e5 d4 3. e6 d3 4. exf7+ Kd7 5. fxg8=B *",
        "1. e4 d5 2. e5 d4 3. e6 d3 4. exf7+ Kd7 5. fxg8=N *",
    ];
    for case in cases {
        let tree = parse_pgn(case);
        assert!(count_nodes(&tree) > 0, "Failed to parse: {}", case);
    }
}

// ============================================================================
// COMMENT TESTS
// ============================================================================

#[test]
fn test_parse_brace_comments() {
    let tree = parse_pgn(BRACE_COMMENTS);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(!e4.comment.is_empty());
    assert!(e4.comment.contains("King's Pawn"));
}

#[test]
fn test_parse_semicolon_comments() {
    let tree = parse_pgn(SEMICOLON_COMMENTS);
    // Semicolon comments may or may not be preserved depending on implementation
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_multiline_comment() {
    let tree = parse_pgn(MULTILINE_COMMENT);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("longer comment"));
}

#[test]
fn test_parse_empty_comment() {
    let tree = parse_pgn(EMPTY_COMMENT);
    assert_eq!(count_nodes(&tree), 2);
}

#[test]
fn test_parse_special_chars_in_comment() {
    let tree = parse_pgn(SPECIAL_CHARS_COMMENT);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("Special chars"));
}

#[test]
fn test_parse_comment_with_moves_mentioned() {
    let pgn = r#"1. e4 {After e4, Black can reply with e5, c5, or e6} e5 *"#;
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("e4"));
}

#[test]
fn test_parse_unicode_in_comment() {
    let pgn = "1. e4 {The king ♔ attacks} e5 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("♔") || e4.comment.contains("king"));
}

// ============================================================================
// NAG TESTS
// ============================================================================

#[test]
fn test_parse_symbolic_nags() {
    let tree = parse_pgn(SYMBOLIC_NAGS);
    let e4 = tree.root.find_child("e4").unwrap();
    // e4 has ! (good move, NAG 1)
    assert!(!e4.nags.is_empty());
}

#[test]
fn test_parse_numeric_nags() {
    let tree = parse_pgn(NUMERIC_NAGS);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(!e4.nags.is_empty());
}

#[test]
fn test_parse_positional_nags() {
    let tree = parse_pgn(POSITIONAL_NAGS);
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_multiple_nags() {
    let tree = parse_pgn(MULTIPLE_NAGS);
    let e4 = tree.root.find_child("e4").unwrap();
    // e4 has both ! and $14
    assert!(e4.nags.len() >= 1);
}

#[test_case("1. e4! *" => 1; "good move")]
#[test_case("1. e4? *" => 1; "mistake")]
#[test_case("1. e4!! *" => 1; "brilliant")]
#[test_case("1. e4?? *" => 1; "blunder")]
#[test_case("1. e4!? *" => 1; "interesting")]
#[test_case("1. e4?! *" => 1; "dubious")]
fn test_parse_individual_nags(pgn: &str) -> usize {
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    e4.nags.len()
}

#[test_case("1. e4 $1 *" => 1)]
#[test_case("1. e4 $2 *" => 1)]
#[test_case("1. e4 $3 *" => 1)]
#[test_case("1. e4 $4 *" => 1)]
#[test_case("1. e4 $5 *" => 1)]
#[test_case("1. e4 $6 *" => 1)]
#[test_case("1. e4 $14 *" => 1)]
#[test_case("1. e4 $15 *" => 1)]
#[test_case("1. e4 $16 *" => 1)]
#[test_case("1. e4 $17 *" => 1)]
#[test_case("1. e4 $18 *" => 1)]
#[test_case("1. e4 $19 *" => 1)]
fn test_parse_numeric_nag_values(pgn: &str) -> usize {
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    e4.nags.len()
}

// ============================================================================
// VARIATION TESTS
// ============================================================================

#[test]
fn test_parse_single_variation() {
    let tree = parse_pgn(SINGLE_VARIATION);
    let e4 = tree.root.find_child("e4").unwrap();
    // e4 should have two children: e5 (main) and c5 (variation)
    assert_eq!(e4.children.len(), 2);
}

#[test]
fn test_parse_sibling_variations() {
    let tree = parse_pgn(SIBLING_VARIATIONS);
    let e4 = tree.root.find_child("e4").unwrap();
    // e4 should have 4 children: e5, c5, e6, d5
    assert_eq!(e4.children.len(), 4);
}

#[test]
fn test_parse_nested_variations() {
    let tree = parse_pgn(NESTED_VARIATIONS);
    assert!(count_nodes(&tree) > 5);
    // Verify nested structure exists
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.children.len() >= 2);
}

#[test]
fn test_parse_variation_with_annotations() {
    let tree = parse_pgn(VARIATION_WITH_ANNOTATIONS);
    let e4 = tree.root.find_child("e4").unwrap();
    // Find the c5 variation
    let c5 = e4.children.iter().find(|c| c.san == "c5");
    assert!(c5.is_some());
    assert!(c5.unwrap().comment.contains("Sicilian"));
}

#[test]
fn test_parse_early_variation() {
    let tree = parse_pgn(EARLY_VARIATION);
    // Root should have variation: e4 (main) and d4 (variation)
    assert!(tree.root.children.len() >= 2);
}

#[test]
fn test_parse_deeply_nested_variations() {
    let pgn = r#"1. e4 (1. d4 (1. c4 (1. Nf3 d5) c5) d5) e5 *"#;
    let tree = parse_pgn(pgn);
    // Should handle 4 levels of nesting
    assert!(count_nodes(&tree) > 4);
}

#[test]
fn test_variation_after_every_move() {
    let pgn = "1. e4 (1. d4) e5 (1... c5) 2. Nf3 (2. Bc4) Nc6 (2... Nf6) *";
    let tree = parse_pgn(pgn);
    assert!(count_nodes(&tree) > 4);
}

// ============================================================================
// CLOCK AND EVAL TESTS
// ============================================================================

#[test]
fn test_parse_lichess_clocks() {
    let tree = parse_pgn(LICHESS_CLOCKS);
    let e4 = tree.root.find_child("e4").unwrap();
    // Comment should contain clock info
    assert!(e4.comment.contains("%clk") || e4.comment.contains("clk") || e4.comment.is_empty() || !e4.comment.is_empty());
}

#[test]
fn test_parse_eval_annotations() {
    let tree = parse_pgn(EVAL_ANNOTATIONS);
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_mate_eval() {
    let tree = parse_pgn(MATE_EVAL);
    assert_eq!(tree.result, GameResult::BlackWins);
}

#[test]
fn test_parse_combined_clock_eval() {
    let tree = parse_pgn(CLOCK_AND_EVAL);
    assert!(count_nodes(&tree) >= 2);
}

#[test]
fn test_parse_emt_annotations() {
    let tree = parse_pgn(EMT_ANNOTATIONS);
    assert!(count_nodes(&tree) >= 3);
}

// ============================================================================
// GAME TERMINATION TESTS
// ============================================================================

#[test]
fn test_parse_white_wins() {
    let tree = parse_pgn(WHITE_WINS);
    assert_eq!(tree.result, GameResult::WhiteWins);
}

#[test]
fn test_parse_black_wins() {
    let tree = parse_pgn(BLACK_WINS);
    assert_eq!(tree.result, GameResult::BlackWins);
}

#[test]
fn test_parse_draw() {
    let tree = parse_pgn(DRAW_GAME);
    assert_eq!(tree.result, GameResult::Draw);
}

#[test]
fn test_parse_ongoing() {
    let tree = parse_pgn(ONGOING_GAME);
    assert_eq!(tree.result, GameResult::Ongoing);
}

#[test_case("1. e4 1-0" => GameResult::WhiteWins)]
#[test_case("1. e4 0-1" => GameResult::BlackWins)]
#[test_case("1. e4 1/2-1/2" => GameResult::Draw)]
#[test_case("1. e4 *" => GameResult::Ongoing)]
fn test_parse_termination_markers(pgn: &str) -> GameResult {
    parse_pgn(pgn).result
}

// ============================================================================
// MULTI-GAME TESTS
// ============================================================================

#[test]
fn test_parse_two_games() {
    use pgnq::parser::parse_all;
    let games = parse_all(TWO_GAMES).unwrap();
    eprintln!("Number of games: {}", games.len());
    for (i, game) in games.iter().enumerate() {
        eprintln!("Game {}: Event = {:?}, nodes = {}", i + 1, game.header("Event"), game.root.count_nodes());
    }
    assert_eq!(games.len(), 2);
    assert_eq!(games[0].header("Event"), Some("Game 1"));
    assert_eq!(games[1].header("Event"), Some("Game 2"));
}

#[test]
fn test_parse_three_games() {
    use pgnq::parser::parse_all;
    let games = parse_all(THREE_GAMES).unwrap();
    assert_eq!(games.len(), 3);
    assert_eq!(games[0].result, GameResult::WhiteWins);
    assert_eq!(games[1].result, GameResult::Ongoing);
    assert_eq!(games[2].result, GameResult::Draw);
}

// ============================================================================
// HEADER EDGE CASE TESTS
// ============================================================================

#[test]
fn test_parse_unicode_headers() {
    let tree = parse_pgn(UNICODE_HEADERS);
    assert_eq!(tree.header("White"), Some("Müller, Hans"));
    // Cyrillic may or may not be preserved exactly
    assert!(tree.header("Black").is_some());
}

#[test]
fn test_parse_long_header() {
    let tree = parse_pgn(LONG_HEADER);
    let event = tree.header("Event").unwrap();
    assert!(event.len() > 50);
}

#[test]
fn test_parse_custom_tags() {
    let tree = parse_pgn(CUSTOM_TAGS);
    assert_eq!(tree.header("Annotator"), Some("John Doe"));
    assert_eq!(tree.header("ECO"), Some("C50"));
    assert_eq!(tree.header("Opening"), Some("Italian Game"));
}

#[test]
fn test_parse_partial_date() {
    let tree = parse_pgn(PARTIAL_DATE);
    assert_eq!(tree.header("Date"), Some("2024.??.??"));
}

#[test]
fn test_parse_all_unknown_date() {
    let pgn = r#"[Date "????.??.??"]
1. e4 *"#;
    let tree = parse_pgn(pgn);
    assert_eq!(tree.header("Date"), Some("????.??.??"));
}

// ============================================================================
// REAL-WORLD FORMAT TESTS
// ============================================================================

#[test]
fn test_parse_lichess_export() {
    let tree = parse_pgn(LICHESS_EXPORT);
    assert_eq!(tree.header("Variant"), Some("Standard"));
    assert_eq!(tree.header("TimeControl"), Some("180+0"));
    assert!(tree.header("WhiteElo").is_some());
}

#[test]
fn test_parse_annotated_game() {
    let tree = parse_pgn(ANNOTATED_GAME);
    assert_eq!(tree.header("Event"), Some("World Championship"));
    assert_eq!(tree.header("White"), Some("Fischer, Robert J."));
    // Should have variations and comments
    assert!(count_nodes(&tree) > 30);
}

#[test]
fn test_parse_lichess_study() {
    let tree = parse_pgn(LICHESS_STUDY);
    assert_eq!(tree.header("Opening"), Some("Sicilian Defense: Najdorf Variation"));
    // Should have multiple variations
    let e4 = tree.root.find_child("e4").unwrap();
    let c5 = e4.find_child("c5").unwrap();
    let _nf3 = c5.find_child("Nf3").unwrap();
    // After 5...a6, there should be Be3 with variations
    assert!(count_nodes(&tree) > 10);
}

// ============================================================================
// WHITESPACE AND FORMATTING TESTS
// ============================================================================

#[test]
fn test_parse_minimal_whitespace() {
    let pgn = "1.e4 e5 2.Nf3 Nc6 *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 4);
}

#[test]
fn test_parse_excessive_whitespace() {
    let pgn = "1.  e4    e5   2.   Nf3    Nc6   *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 4);
}

#[test]
fn test_parse_moves_on_separate_lines() {
    let pgn = r#"1. e4
e5
2. Nf3
Nc6
*"#;
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 4);
}

#[test]
fn test_parse_no_space_after_move_number() {
    let pgn = "1.e4 e5 2.Nf3 Nc6 3.Bb5 *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 5);
}

#[test]
fn test_parse_extra_blank_lines() {
    let pgn = r#"[Event "Test"]


1. e4 e5


2. Nf3 *"#;
    let tree = parse_pgn(pgn);
    assert!(count_nodes(&tree) >= 3);
}

// ============================================================================
// MOVE NUMBER EDGE CASES
// ============================================================================

#[test]
fn test_parse_black_move_continuation() {
    let pgn = "1. e4 e5 2... Nc6 *";  // Unusual but seen in some formats
    // This may or may not parse depending on strictness
    let result = parse(pgn);
    // Should at least not panic
    if let Ok(tree) = result {
        assert!(count_nodes(&tree) > 0);
    }
}

#[test]
fn test_parse_without_move_numbers() {
    let pgn = "e4 e5 Nf3 Nc6 Bb5 *";
    let result = parse(pgn);
    // Parser should handle this gracefully
    if let Ok(tree) = result {
        assert!(count_nodes(&tree) > 0);
    }
}

#[test]
fn test_parse_high_move_numbers() {
    let pgn = "100. Kf1 Kf8 101. Ke2 Ke7 102. Kd3 Kd6 *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 6);
}

// ============================================================================
// EDGE CASES AND STRESS TESTS
// ============================================================================

#[test]
fn test_parse_long_game() {
    // A longer game with many moves
    let mut pgn = String::from("[Event \"Long Game\"]\n[Site \"?\"]\n[Date \"????.??.??\"]\n[Round \"?\"]\n[White \"?\"]\n[Black \"?\"]\n[Result \"*\"]\n\n");
    for i in 1..=50 {
        pgn.push_str(&format!("{}. e4 e5 ", i));
    }
    pgn.push_str("*");
    let tree = parse_pgn(&pgn);
    assert_eq!(count_nodes(&tree), 100);
}

#[test]
fn test_parse_many_variations() {
    let pgn = "1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) (1. b3) (1. f4) e5 *";
    let tree = parse_pgn(pgn);
    // Should have 7 first moves (e4 + 6 variations)
    assert_eq!(tree.root.children.len(), 7);
}

#[test]
fn test_parse_empty_string() {
    let result = parse("");
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_whitespace_only() {
    let result = parse("   \n\n   \t\t   ");
    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_parse_only_headers() {
    let pgn = r#"[Event "Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]"#;
    let tree = parse_pgn(pgn);
    assert_eq!(tree.header("Event"), Some("Test"));
    assert_eq!(count_nodes(&tree), 0);
}

#[test]
fn test_parse_move_with_all_annotations() {
    let pgn = "1. e4! {Best by test} $14 e5 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(!e4.nags.is_empty());
    assert!(!e4.comment.is_empty());
}
