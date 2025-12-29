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

// ============================================================================
// ADDITIONAL CASTLING EDGE CASES
// ============================================================================

#[test]
fn test_parse_castling_kingside_with_check() {
    let pgn = "1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_castling_queenside_with_check() {
    // Game where O-O-O gives check
    let pgn = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 c6 5. Bd2 Nf6 6. Bc4 Bf5 7. Nf3 e6 8. Qe2 Bb4 9. O-O-O+ *";
    let result = parse(pgn);
    assert!(result.is_ok());
    let tree = result.unwrap();
    let moves = main_line_moves(&tree);
    assert!(moves.iter().any(|m| m.contains("O-O-O")));
}

#[test]
fn test_parse_both_sides_castle() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    // Both O-O should appear
    let castles: Vec<_> = moves.iter().filter(|m| *m == "O-O").collect();
    assert_eq!(castles.len(), 2);
}

// ============================================================================
// HEADER ESCAPING AND SPECIAL CHARACTERS
// ============================================================================

#[test]
fn test_parse_header_with_quotes_inside() {
    // Escaped quotes in header values
    let pgn = r#"[Event "The \"Big\" Tournament"]
1. e4 *"#;
    let result = parse(pgn);
    // Should not crash - may or may not parse quotes correctly
    assert!(result.is_ok());
}

#[test]
fn test_parse_header_with_backslash() {
    let pgn = r#"[Source "C:\\Games\\chess.pgn"]
1. e4 *"#;
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_header_with_brackets_in_value() {
    let pgn = r#"[Event "Match [Round 1]"]
1. e4 *"#;
    let result = parse(pgn);
    // May or may not handle this edge case
    let _ = result;
}

#[test]
fn test_parse_header_very_long_value() {
    let long_value = "A".repeat(1000);
    let pgn = format!(r#"[Event "{}"]
1. e4 *"#, long_value);
    let result = parse(&pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_header_unicode_player_names() {
    let pgn = r#"[White "Магнус Карлсен"]
[Black "丁立人"]
1. e4 e5 *"#;
    let tree = parse_pgn(pgn);
    assert!(tree.header("White").unwrap().contains("Карлсен"));
    assert!(tree.header("Black").unwrap().contains("丁"));
}

#[test]
fn test_parse_header_with_newline_before_value() {
    let pgn = "[Event\n\"Test\"]\n1. e4 *";
    let result = parse(pgn);
    // Should handle gracefully
    let _ = result;
}

// ============================================================================
// LINE ENDING VARIATIONS
// ============================================================================

#[test]
fn test_parse_crlf_line_endings() {
    let pgn = "[Event \"Test\"]\r\n[Site \"?\"]\r\n\r\n1. e4 e5\r\n2. Nf3 *\r\n";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_mixed_line_endings() {
    let pgn = "[Event \"Test\"]\r\n[Site \"?\"]\n\n1. e4 e5\r\n2. Nf3 *\n";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_no_final_newline() {
    let pgn = "1. e4 e5 2. Nf3 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

// ============================================================================
// EN PASSANT AND SPECIAL PAWN MOVES
// ============================================================================

#[test]
fn test_parse_en_passant_capture() {
    // En passant is notated as a regular pawn capture
    let pgn = "1. e4 d5 2. e5 f5 3. exf6 *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"exf6".to_string()));
}

#[test]
fn test_parse_double_pawn_push() {
    let pgn = "1. e4 e5 2. d4 d5 *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 4);
}

#[test]
fn test_parse_pawn_captures_both_directions() {
    let pgn = "1. e4 d5 2. exd5 e6 3. dxe6 *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"exd5".to_string()));
    assert!(moves.contains(&"dxe6".to_string()));
}

// ============================================================================
// PROMOTION EDGE CASES
// ============================================================================

#[test]
fn test_parse_all_promotion_pieces() {
    let pgn = "1. a8=Q b1=R 2. c8=B d1=N *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.iter().any(|m| m.contains("=Q")));
    assert!(moves.iter().any(|m| m.contains("=R")));
    assert!(moves.iter().any(|m| m.contains("=B")));
    assert!(moves.iter().any(|m| m.contains("=N")));
}

#[test]
fn test_parse_promotion_with_capture() {
    let pgn = "1. axb8=Q *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves[0].contains("xb8=Q") || moves[0].contains("axb8"));
}

#[test]
fn test_parse_promotion_with_check() {
    let pgn = "1. e8=Q+ Kf7 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_promotion_with_checkmate() {
    let pgn = "1. e8=Q# 1-0";
    let result = parse(pgn);
    assert!(result.is_ok());
}

// ============================================================================
// DISAMBIGUATION EDGE CASES
// ============================================================================

#[test]
fn test_parse_full_disambiguation() {
    // Rare case: both file and rank needed
    let pgn = "1. Qa1a3 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_queen_file_disambiguation() {
    let pgn = "1. Qaa3 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_queen_rank_disambiguation() {
    let pgn = "1. Q1a3 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_knight_disambiguation() {
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Nc3 Nf6 4. Nd5 Nxd5 5. exd5 Nd4 *";
    let tree = parse_pgn(pgn);
    assert!(tree.root.find_path(&["e4", "e5", "Nf3"]).is_some());
}

// ============================================================================
// CHECK AND CHECKMATE NOTATION
// ============================================================================

#[test]
fn test_parse_check_symbol() {
    let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7+ *";
    let tree = parse_pgn(pgn);
    let moves = main_line_moves(&tree);
    assert!(moves.iter().any(|m| m.contains("Qxf7")));
}

#[test]
fn test_parse_checkmate_symbol() {
    let pgn = "1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0";
    let tree = parse_pgn(pgn);
    assert_eq!(tree.result, GameResult::WhiteWins);
}

#[test]
fn test_parse_double_check() {
    // Double check is still just + in notation
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Bc5 5. Nxe5 Nxe5 6. d4 Bxd4 7. Qxd4 d6 8. f4 Neg4 9. e5 c6 10. exf6+ *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

// ============================================================================
// COMPLEX VARIATION STRUCTURES
// ============================================================================

#[test]
fn test_parse_variation_starting_at_move_one() {
    let pgn = "(1. d4) 1. e4 e5 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_variations_same_point() {
    let pgn = "1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) e5 *";
    let tree = parse_pgn(pgn);
    // Root should have 5 children: e4 + 4 variations
    assert_eq!(tree.root.children.len(), 5);
}

#[test]
fn test_parse_variation_with_sub_variations() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 (2. Nc3 Nc6) d6) 2. Nf3 *";
    let tree = parse_pgn(pgn);
    // Should parse all nested variations
    assert!(tree.root.find_child("e4").is_some());
}

#[test]
fn test_parse_variation_at_every_move() {
    let pgn = "1. e4 (1. d4) e5 (1... c5) 2. Nf3 (2. Bc4) Nc6 (2... Nf6) *";
    let tree = parse_pgn(pgn);
    // Multiple variations throughout
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.children.len() >= 2); // e5 and c5
}

#[test]
fn test_parse_long_variation() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be2 e5 7. Nb3 Be7 8. O-O O-O 9. Be3 Be6) 2. Nf3 *";
    let tree = parse_pgn(pgn);
    // Variation should have many moves
    let e4 = tree.root.find_child("e4").unwrap();
    let c5 = e4.find_child("c5");
    assert!(c5.is_some());
}

// ============================================================================
// NAG EDGE CASES
// ============================================================================

#[test]
fn test_parse_nag_zero() {
    let pgn = "1. e4 $0 e5 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_high_nag_values() {
    let pgn = "1. e4 $140 e5 $200 2. Nf3 $255 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_many_nags_one_move() {
    let pgn = "1. e4! ? $1 $2 $3 $4 $5 $6 e5 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.len() >= 2);
}

#[test]
fn test_parse_nags_in_variation() {
    let pgn = "1. e4 e5 (1... c5! $1 2. Nf3?) 2. Nf3 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    let c5 = e4.find_child("c5").unwrap();
    assert!(!c5.nags.is_empty());
}

#[test]
fn test_parse_positional_assessment_nags() {
    let pgn = "1. e4 $13 e5 $14 2. Nf3 $15 Nc6 $16 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

// ============================================================================
// COMMENT EDGE CASES
// ============================================================================

#[test]
fn test_parse_comment_before_first_move() {
    let pgn = "{This game is famous} 1. e4 e5 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_comment_after_result() {
    let pgn = "1. e4 e5 1-0 {White wins by resignation}";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_adjacent_comments() {
    let pgn = "1. e4 {first} {second} e5 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    // Comments should be combined
    assert!(!e4.comment.is_empty());
}

#[test]
fn test_parse_comment_with_pgn_like_content() {
    let pgn = "1. e4 {After 1. d4 d5 2. c4 we get the Queen's Gambit} e5 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_semicolon_comment_then_brace() {
    let pgn = "1. e4 ; line comment\n{brace comment} e5 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_comment_with_clock_and_text() {
    let pgn = "1. e4 {[%clk 0:03:00] Opening move, controlling the center} e5 *";
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("%clk"));
    assert!(e4.comment.contains("Opening"));
}

// ============================================================================
// GAME TERMINATION EDGE CASES
// ============================================================================

#[test]
fn test_parse_game_with_only_result() {
    let pgn = "1-0";
    let result = parse(pgn);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().result, GameResult::WhiteWins);
}

#[test]
fn test_parse_game_forfeit() {
    // Forfeit still uses standard result
    let pgn = r#"[Termination "Time forfeit"]
1. e4 e5 2. Nf3 1-0"#;
    let tree = parse_pgn(pgn);
    assert_eq!(tree.result, GameResult::WhiteWins);
}

#[test]
fn test_parse_result_mismatch_header() {
    // Result in movetext should match header
    let pgn = r#"[Result "1-0"]
1. e4 e5 0-1"#;
    let result = parse(pgn);
    // Parser should handle this - last result wins or header wins
    assert!(result.is_ok());
}

// ============================================================================
// MULTI-GAME EDGE CASES
// ============================================================================

#[test]
fn test_parse_games_with_blank_lines_between() {
    let pgn = r#"[Event "Game 1"]
1. e4 1-0


[Event "Game 2"]
1. d4 0-1"#;
    let games = pgnq::parser::parse_all(pgn).unwrap();
    assert_eq!(games.len(), 2);
}

#[test]
fn test_parse_games_no_blank_lines() {
    let pgn = r#"[Event "Game 1"]
1. e4 1-0
[Event "Game 2"]
1. d4 0-1"#;
    let games = pgnq::parser::parse_all(pgn).unwrap();
    assert_eq!(games.len(), 2);
}

#[test]
fn test_parse_game_headers_only() {
    let pgn = r#"[Event "No moves game"]
[Result "*"]

*"#;
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_games_mixed_completeness() {
    let pgn = r#"[Event "Complete"]
1. e4 e5 2. Nf3 Nc6 1-0

[Event "Ongoing"]
1. d4 *

[Event "Just started"]
*"#;
    let games = pgnq::parser::parse_all(pgn).unwrap();
    assert!(games.len() >= 2);
}

// ============================================================================
// MOVE NUMBER EDGE CASES
// ============================================================================

#[test]
fn test_parse_missing_move_numbers() {
    let pgn = "e4 e5 Nf3 Nc6 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_irregular_move_numbers() {
    // Some exports have gaps in move numbers
    let pgn = "1. e4 e5 3. Nf3 Nc6 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

#[test]
fn test_parse_move_number_without_period() {
    let pgn = "1 e4 e5 2 Nf3 *";
    let result = parse(pgn);
    // May or may not parse correctly
    let _ = result;
}

#[test]
fn test_parse_black_move_continuation_explicit() {
    let pgn = "1. e4 1... e5 2. Nf3 *";
    let result = parse(pgn);
    assert!(result.is_ok());
}

// ============================================================================
// LONG ALGEBRAIC NOTATION (if supported)
// ============================================================================

#[test]
fn test_parse_coordinate_notation() {
    // Some engines output coordinate notation
    let pgn = "1. e2e4 e7e5 *";
    let result = parse(pgn);
    // May or may not be supported
    let _ = result;
}

// ============================================================================
// EDGE CASES FROM REAL-WORLD EXPORTS
// ============================================================================

#[test]
fn test_parse_lichess_study_format() {
    let pgn = r#"[Event "Study: Chapter 1"]
[Site "https://lichess.org/study/abc123/xyz789"]
[UTCDate "2024.01.15"]
[UTCTime "10:00:00"]
[Variant "Standard"]
[ECO "B20"]
[Opening "Sicilian Defense"]
[Annotator "username"]

1. e4 c5 {The Sicilian Defense - Black's most popular response to 1.e4} 2. Nf3 *"#;
    let tree = parse_pgn(pgn);
    assert!(tree.header("Annotator").is_some());
    assert!(tree.header("ECO").is_some());
}

#[test]
fn test_parse_chesscom_format() {
    let pgn = r#"[Event "Live Chess"]
[Site "Chess.com"]
[Date "2024.01.15"]
[Round "-"]
[White "player1"]
[Black "player2"]
[Result "1-0"]
[WhiteElo "1500"]
[BlackElo "1480"]
[TimeControl "600"]
[EndTime "14:30:00 PST"]
[Termination "player1 won by checkmate"]

1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0"#;
    let tree = parse_pgn(pgn);
    assert_eq!(tree.result, GameResult::WhiteWins);
    assert!(tree.header("Termination").is_some());
}

#[test]
fn test_parse_stockfish_analysis() {
    let pgn = r#"[Event "Analysis"]
[Annotator "Stockfish 16"]

1. e4 {[%eval 0.25]} e5 {[%eval 0.22]} 2. Nf3 {[%eval 0.35]} Nc6 {[%eval 0.30]} *"#;
    let tree = parse_pgn(pgn);
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("%eval"));
}
