//! PGN parser - tokenization and tree building
//!
//! # Pipeline
//!
//! ```text
//! Input → Phase 1 (Lexer) → Phase 2 (Post-process) → Phase 3 (Builder) → GameTree
//! ```

mod builder;
mod lexer;
mod token;
mod token_postprocess;

pub use builder::build_tree;
pub use lexer::{tokenize, LocatedToken};
pub use token::Token;

use crate::error::{ParseMode, Result};
use crate::tree::GameTree;
use std::path::PathBuf;

/// Parse a PGN string into a GameTree
pub fn parse(input: &str) -> Result<GameTree> {
    parse_with_options(input, ParseMode::Lenient, None)
}

/// Parse a PGN string with specific options
pub fn parse_with_options(
    input: &str,
    mode: ParseMode,
    file: Option<PathBuf>,
) -> Result<GameTree> {
    // Phase 1: Tokenize
    let tokens = lexer::tokenize(input);

    // Phase 2: Post-process tokens
    let tokens = token_postprocess::process(tokens, mode);

    // Phase 3: Build tree
    builder::build_tree_with_options(&tokens, mode, file, Some(input))
}

/// Parse multiple games from a PGN string
pub fn parse_all(input: &str) -> Result<Vec<GameTree>> {
    parse_all_with_options(input, ParseMode::Lenient, None)
}

/// Parse multiple games with specific options
pub fn parse_all_with_options(
    input: &str,
    mode: ParseMode,
    file: Option<PathBuf>,
) -> Result<Vec<GameTree>> {
    // Split on headers that appear after moves (indicating a new game)
    let mut games = Vec::new();
    let mut current_start = 0;
    let mut seen_moves = false; // Track if we've seen movetext after headers
    let mut current_pos = 0;

    for line in input.lines() {
        let line_len = line.len();
        let is_header = line.starts_with('[') && line.contains('"');
        let is_move_line = !line.trim().is_empty()
            && !is_header
            && !line.trim().starts_with(';'); // Not a semicolon comment

        if is_header && seen_moves {
            // New game starting - we've seen moves and now see a new header
            let game_text = &input[current_start..current_pos];
            if !game_text.trim().is_empty() {
                games.push(parse_with_options(game_text, mode, file.clone())?);
            }
            current_start = current_pos;
            seen_moves = false;
        } else if is_move_line {
            seen_moves = true;
        }

        // Move position past this line and its newline
        current_pos += line_len;
        if current_pos < input.len() {
            // Skip the newline character(s)
            if input[current_pos..].starts_with("\r\n") {
                current_pos += 2;
            } else if input[current_pos..].starts_with('\n') {
                current_pos += 1;
            }
        }
    }

    // Parse the last game
    let remaining = &input[current_start..];
    if !remaining.trim().is_empty() {
        games.push(parse_with_options(remaining, mode, file)?);
    }

    if games.is_empty() {
        // Try parsing as a single game anyway
        games.push(parse(input)?);
    }

    Ok(games)
}

/// Alias for parse_all (backward compatibility)
pub fn parse_games(input: &str) -> Result<Vec<GameTree>> {
    parse_all(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_game() {
        let pgn = r#"
[Event "Test"]
[White "Player 1"]
[Black "Player 2"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;
        let tree = parse(pgn).unwrap();
        assert_eq!(tree.header("Event"), Some("Test"));
        assert_eq!(tree.header("White"), Some("Player 1"));

        // Check the moves
        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.san, "e4");
        let e5 = e4.find_child("e5").unwrap();
        assert_eq!(e5.san, "e5");
    }

    #[test]
    fn test_parse_with_comments() {
        let pgn = "1. e4 {Best by test} e5 {Solid reply}";
        let tree = parse(pgn).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.comment.trim(), "Best by test");
    }

    #[test]
    fn test_parse_with_variations() {
        let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3";
        let tree = parse(pgn).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.children.len(), 2); // e5 and c5

        let e5 = &e4.children[0];
        assert_eq!(e5.san, "e5");

        let c5 = &e4.children[1];
        assert_eq!(c5.san, "c5");
    }

    #[test]
    fn test_parse_headerless() {
        let pgn = "1. d4 Nf6 2. c4 e6";
        let tree = parse(pgn).unwrap();

        assert!(tree.headers.is_empty());
        let d4 = tree.root.find_child("d4").unwrap();
        assert_eq!(d4.san, "d4");
    }

    // ========================================================================
    // Baretext Parser Reproducer Tests
    // ========================================================================

    #[test]
    fn test_baretext_ignores_parenthetical_move_mention() {
        let pgn = r#"1. e4 e5
This avoids the Petrosian Variation (7.d5) which is problematic.
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 3 moves: e4, e5, Nf3
        // Should NOT have a "d5" node - the "(7.d5)" is a textual reference
        assert_eq!(tree.count_nodes(), 3, "Expected 3 nodes (e4, e5, Nf3), got {}", tree.count_nodes());
    }

    #[test]
    fn test_multiple_parenthetical_references() {
        let pgn = r#"1. e4 e5
White can play the Ruy Lopez (3.Bb5) or Italian (3.Bc4) here.
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 3 moves, not 5
        assert_eq!(tree.count_nodes(), 3, "Expected 3 nodes, got {}", tree.count_nodes());
    }

    #[test]
    fn test_real_variation_preserved() {
        let pgn = r#"1. e4 e5 (1... c5 2. Nf3) 2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 5 moves: e4, e5, c5, Nf3 (in var), Nf3 (main)
        // The variation has multiple moves, so it's NOT a parenthetical reference
        assert_eq!(tree.count_nodes(), 5, "Expected 5 nodes, got {}", tree.count_nodes());
        // e4 should have a variation
        let e4 = tree.root.find_child("e4").unwrap();
        assert!(e4.has_variations(), "e4 should have variations");
    }

    #[test]
    fn test_lichess_baretext_with_references() {
        let pgn = r#"1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2
This move order avoids the Petrosian Variation (7.d5) and the Exchange Variation.
Na6
The knight develops to the rim.
7. O-O *"#;
        let tree = parse(pgn).unwrap();
        // Main line: d4, Nf6, c4, g6, Nc3, Bg7, e4, d6, Nf3, O-O, Be2, Na6, O-O = 13 moves
        // No spurious d5 variation
        assert_eq!(tree.count_nodes(), 13, "Expected 13 nodes, got {}", tree.count_nodes());
    }

    #[test]
    fn test_stats_consistency() {
        let pgn = "1. e4 e5 (1... c5) 2. Nf3 *";
        let tree = parse(pgn).unwrap();
        let count = tree.count_nodes();
        // Verify count is stable across multiple calls
        assert_eq!(tree.count_nodes(), count);
        assert_eq!(tree.count_nodes(), count);
    }

    // ========================================================================
    // Prose-Embedded Move Detection Tests
    // ========================================================================

    #[test]
    fn test_baretext_move_reference_not_parsed_as_move() {
        let pgn = "1. e4 e5\nThe pawn on f3 is weak.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // Should have 3 moves: e4, e5, Nf3
        // "f3" in prose should NOT create a move node
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes (e4, e5, Nf3), got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_move_on_own_line_is_parsed() {
        let pgn = "1. e4\nGreat opening!\ne5\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // e5 on its own line should be parsed as a move
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes (e4, e5, Nf3), got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_multiple_embedded_moves_in_prose() {
        let pgn = "1. e4 e5\nWhite can play Be2 or Nf3 here.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // "Be2 or Nf3" in prose should NOT create move nodes
        // Only real moves: e4, e5, Nf3
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes (e4, e5, Nf3), got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_piece_name_in_prose_not_parsed_as_move() {
        let pgn = "1. e4 e5\nThe Knight is a tricky piece.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // "Knight" should not be parsed as a move
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    // ========================================================================
    // State Machine Edge Case Tests
    // ========================================================================

    #[test]
    fn test_transposition_reference_not_parsed_as_moves() {
        // This pattern was incorrectly parsed by the old heuristic
        let pgn = r#"1. e4 e5 2. Nf3
Transposition to 1.d4 Nf6 2.c4 g6 3.Nc3 Bg7.
3. Bc4 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 4 moves: e4, e5, Nf3, Bc4
        // The transposition reference should NOT create 6 additional nodes
        assert_eq!(
            tree.count_nodes(),
            4,
            "Expected 4 nodes, got {} (transposition parsed as moves?)",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_year_not_parsed_as_move_number() {
        // Years like "2025." should not be parsed as move numbers
        let pgn = r#"1. e4 e5
Smith-Jones 2025. In this game White played aggressively.
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 3 moves, not treating "2025." as a move number
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {} (year parsed as move number?)",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_move_number_in_prose_not_parsed() {
        // Move numbers mentioned in prose should stay as prose
        let pgn = r#"1. e4 e5
After 10. cxb5 Black has good counterplay.
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // "10. cxb5" is prose, not a real move
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_variation_after_prose_parses_correctly() {
        // Variation starting after a prose line should parse as real moves
        let pgn = r#"1. e4 e5
White has several options here.
(1. d4 d5 2. c4)
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Main: e4, e5, Nf3 = 3
        // Variation: d4, d5, c4 = 3
        // Total = 6
        assert_eq!(
            tree.count_nodes(),
            6,
            "Expected 6 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_prose_after_variation_filters_moves() {
        // Prose after a variation ends should still filter move references
        let pgn = r#"1. e4 e5 (1... c5)
This is similar to the Sicilian with Nf3 ideas.
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Main: e4, e5, Nf3 = 3
        // Variation: c5 = 1
        // "Nf3" in prose should NOT be parsed
        assert_eq!(
            tree.count_nodes(),
            4,
            "Expected 4 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_context_restored_after_variation() {
        // After exiting a variation, context should be restored properly
        let pgn = r#"1. e4
Opening comment with e5 reference.
(1. d4 d5)
Still in prose mode, Nf3 is mentioned.
e5
2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Main: e4, e5, Nf3 = 3
        // Variation: d4, d5 = 2
        // "e5" and "Nf3" in prose are NOT moves, but "e5" on own line IS
        assert_eq!(
            tree.count_nodes(),
            5,
            "Expected 5 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_brace_comment_does_not_enter_prose_mode() {
        // Brace comments are explicitly delimited, shouldn't affect state
        let pgn = "1. e4 {with e5 as reply} e5 2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // All moves should parse normally
        assert_eq!(tree.count_nodes(), 3);
        let e4 = tree.root.find_child("e4").unwrap();
        assert!(e4.comment.contains("e5"));
    }

    #[test]
    fn test_move_sequence_in_prose_line() {
        // Multiple moves on same line as prose = all prose
        let pgn = r#"1. e4 e5
The line 2.Nf3 Nc6 3.Bb5 is the Ruy Lopez.
2. Bc4 *"#;
        let tree = parse(pgn).unwrap();
        // Should have 3 moves: e4, e5, Bc4
        // "2.Nf3 Nc6 3.Bb5" is prose
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_newline_resets_prose_context() {
        // After newline, moves on own line should be real
        let pgn = "1. e4\nThis is prose with Nf3.\ne5\nMore prose.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // e4, e5 (on own line), Nf3 (after move number) = 3 real moves
        assert_eq!(tree.count_nodes(), 3);
    }

    #[test]
    fn test_nested_variation_with_prose() {
        // Nested variations should each maintain their own context
        let pgn = r#"1. e4 e5 (1... c5
The Sicilian with d6 ideas.
2. Nf3 (2. Nc3 Nc6)) 2. Nf3 *"#;
        let tree = parse(pgn).unwrap();
        // Main: e4, e5, Nf3 = 3
        // First var: c5, Nf3 = 2
        // Nested var: Nc3, Nc6 = 2
        // Total = 7
        // "d6" in prose should not be parsed
        assert_eq!(
            tree.count_nodes(),
            7,
            "Expected 7 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_castling_in_prose_not_parsed() {
        // Castling notation in prose should not be parsed as moves
        let pgn = "1. e4 e5\nWhite should castle O-O soon.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // "O-O" in prose should NOT be a move
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_long_transposition_sequence() {
        // Real pattern from Saemisch file: long transposition reference
        let pgn = r#"1. d4 Nf6 2. c4 g6
(2... e6 3. Nc3 Bb4
Transposition to 4.e4 d6 5.f3 O-O 6.Be3 c5 7.Nge2.
)
3. Nc3 *"#;
        let tree = parse(pgn).unwrap();
        // Main: d4, Nf6, c4, g6, Nc3 = 5
        // Variation: e6, Nc3, Bb4 = 3
        // Transposition text should NOT add nodes
        assert_eq!(
            tree.count_nodes(),
            8,
            "Expected 8 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_prose_with_check_notation() {
        // Check notation in prose should not be parsed
        let pgn = "1. e4 e5\nAfter Bb5+ the king must move.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_prose_with_capture_notation() {
        // Capture notation in prose should not be parsed
        let pgn = "1. e4 e5\nThe exchange exd5 is thematic.\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        assert_eq!(
            tree.count_nodes(),
            3,
            "Expected 3 nodes, got {}",
            tree.count_nodes()
        );
    }

    #[test]
    fn test_comment_then_move_on_same_line() {
        // Move immediately after prose on same line = prose (no newline reset)
        let pgn = "1. e4 e5\nGood move e5\n2. Nf3 *";
        let tree = parse(pgn).unwrap();
        // "e5" after "Good move" is prose, but there's already e5 from "1. e4 e5"
        assert_eq!(tree.count_nodes(), 3);
    }

    #[test]
    fn test_multiple_games_prose_isolation() {
        // Prose context should not leak between games
        let pgn = r#"[Event "Game 1"]
1. e4 e5
Prose about Nf3.
*

[Event "Game 2"]
1. d4 Nf3 *"#;
        let games = parse_all(pgn).unwrap();
        assert_eq!(games.len(), 2);
        // Game 1: e4, e5 = 2 (Nf3 is prose)
        assert_eq!(games[0].count_nodes(), 2);
        // Game 2: d4, Nf3 = 2 (Nf3 is real move after move number)
        assert_eq!(games[1].count_nodes(), 2);
    }

    // ========================================================================
    // Error Reporting Tests
    // ========================================================================

    #[test]
    fn test_unclosed_variation_error_with_context() {
        let pgn = "1. e4 (1... c5\n2. Nf3 *";
        let result = parse_with_options(pgn, ParseMode::Lenient, Some("test.pgn".into()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("E002"), "Should have error code E002");
        assert!(err_str.contains("unclosed variation"), "Should mention unclosed variation");
    }

    #[test]
    fn test_parse_mode_default_is_lenient() {
        // Verify parse() uses lenient mode by default
        let pgn = "1. e4 e5 2. Nf3 *";
        let result = parse(pgn);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Strict vs Lenient Mode Tests
    // ========================================================================

    #[test]
    fn test_strict_mode_errors_on_unmatched_closing_paren() {
        // Extra closing paren should error in strict mode
        let pgn = "1. e4 e5) 2. Nf3 *";
        let result = parse_with_options(pgn, ParseMode::Strict, None);
        assert!(result.is_err(), "Strict mode should error on unmatched )");
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("E003"), "Should have error code E003");
    }

    #[test]
    fn test_lenient_mode_ignores_unmatched_closing_paren() {
        // Extra closing paren should be silently ignored in lenient mode
        let pgn = "1. e4 e5) 2. Nf3 *";
        let result = parse_with_options(pgn, ParseMode::Lenient, None);
        assert!(result.is_ok(), "Lenient mode should ignore unmatched )");
        let tree = result.unwrap();
        assert_eq!(tree.count_nodes(), 3);
    }

    #[test]
    fn test_strict_mode_parses_embedded_variation_literally() {
        // In strict mode, parenthetical references are parsed as real variations
        let pgn = r#"1. e4 e5
This avoids the Petrosian Variation (7.d5) which is problematic.
2. Nf3 *"#;
        let result = parse_with_options(pgn, ParseMode::Strict, None);
        assert!(result.is_ok());
        let tree = result.unwrap();
        // Strict mode parses (7.d5) as a real variation, so we get more nodes
        // e4, e5, Nf3 (main) + d5 (from variation) = 4
        assert!(
            tree.count_nodes() > 3,
            "Strict mode should parse (7.d5) as a real variation"
        );
    }

    #[test]
    fn test_lenient_mode_collapses_embedded_variation() {
        // In lenient mode, short parenthetical references are collapsed to text
        let pgn = r#"1. e4 e5
This avoids the Petrosian Variation (7.d5) which is problematic.
2. Nf3 *"#;
        let result = parse_with_options(pgn, ParseMode::Lenient, None);
        assert!(result.is_ok());
        let tree = result.unwrap();
        // Lenient mode collapses (7.d5) into the comment text
        assert_eq!(
            tree.count_nodes(),
            3,
            "Lenient mode should collapse (7.d5) to text"
        );
    }

    #[test]
    fn test_strict_mode_multiple_unmatched_parens() {
        // Multiple unmatched parens should error on the first one in strict mode
        let pgn = "1. e4)) e5 *";
        let result = parse_with_options(pgn, ParseMode::Strict, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_lenient_mode_multiple_unmatched_parens() {
        // Multiple unmatched parens should all be ignored in lenient mode
        let pgn = "1. e4)) e5))) 2. Nf3 *";
        let result = parse_with_options(pgn, ParseMode::Lenient, None);
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.count_nodes(), 3);
    }

    #[test]
    fn test_strict_mode_valid_variation_works() {
        // Properly balanced variations should work in strict mode
        let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Bc4 *";
        let result = parse_with_options(pgn, ParseMode::Strict, None);
        assert!(result.is_ok());
        let tree = result.unwrap();
        // e4, e5, Bc4 (main) + c5, Nf3 (variation) = 5
        assert_eq!(tree.count_nodes(), 5);
    }

    #[test]
    fn test_both_modes_error_on_unclosed_variation() {
        // Unclosed variations should error in both modes
        let pgn = "1. e4 (1... c5 *";

        let strict_result = parse_with_options(pgn, ParseMode::Strict, None);
        assert!(strict_result.is_err(), "Strict mode should error on unclosed variation");

        let lenient_result = parse_with_options(pgn, ParseMode::Lenient, None);
        assert!(lenient_result.is_err(), "Lenient mode should also error on unclosed variation");
    }

}
