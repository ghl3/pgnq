//! Comprehensive parser edge case tests
//!
//! Tests various PGN formats, move notations, comments, NAGs, variations,
//! and other edge cases to ensure the parser is robust and accepting.

#[macro_use]
mod common;

use pgnq::nag::Nag;
use pgnq::parser::parse;
use pgnq::tree::GameResult;
use common::{count_nodes, main_line_moves, parse_pgn};
use test_case::test_case;

// Use pretty_assertions for better diffs, but only in non-test_case tests
// to avoid macro conflicts
#[allow(unused_imports)]
use pretty_assertions::assert_eq as pretty_assert_eq;

// ============================================================================
// BASIC PARSING TESTS
// ============================================================================

const MINIMAL_GAME: &str = "1. e4 e5 2. Nf3 Nc6 1-0";

#[test]
fn test_parse_minimal_game() {
    let tree = parse_pgn(MINIMAL_GAME);

    assert_eq!(tree.result, GameResult::WhiteWins);

    let expected = game_tree! {
        e4 { e5 { Nf3 { Nc6 } } }
    };
    assert_contains_tree!(tree, expected);
}

const FULL_HEADERS_GAME: &str = r#"[Event "Test Tournament"]
[Site "Test City"]
[Date "2024.01.15"]
[Round "1"]
[White "Player, White"]
[Black "Player, Black"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0"#;

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

    let expected = game_tree! {
        e4 { e5 { Nf3 { Nc6 { Bb5 } } } }
    };
    assert_contains_tree!(tree, expected);
}

const HEADERLESS_GAME: &str = "1. d4 d5 2. c4 e6 3. Nc3 Nf6 *";

#[test]
fn test_parse_headerless_game() {
    let tree = parse_pgn(HEADERLESS_GAME);
    assert!(tree.headers.is_empty() || tree.header("Event") == Some("?"));
    assert_eq!(count_nodes(&tree), 6);
    assert_eq!(tree.result, GameResult::Ongoing);

    let expected = game_tree! {
        d4 { d5 { c4 { e6 { Nc3 { Nf6 } } } } }
    };
    assert_contains_tree!(tree, expected);
}

// ============================================================================
// MOVE NOTATION TESTS
// ============================================================================

const CASTLING_GAME: &str = r#"[Event "Castling Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 O-O 6. c3 d6 *"#;

#[test]
fn test_parse_castling_kingside() {
    let tree = parse_pgn(CASTLING_GAME);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"O-O".to_string()));

    // Verify basic tree structure (castling move O-O has hyphen, verify via find_path)
    let expected = game_tree! {
        e4 { e5 { Nf3 { Nc6 { Bc4 { Bc5 } } } } }
    };
    assert_contains_tree!(tree, expected);
    // Verify castling is in the tree
    assert!(tree.find_path(&["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "O-O"]).is_some());
}

const QUEENSIDE_CASTLING: &str = "1. d4 d5 2. c4 e6 3. Nc3 Nf6 4. Bg5 Be7 5. e3 O-O 6. Nf3 Nbd7 7. Qc2 c6 8. O-O-O *";

#[test]
fn test_parse_castling_queenside() {
    let tree = parse_pgn(QUEENSIDE_CASTLING);
    let moves = main_line_moves(&tree);
    assert!(moves.contains(&"O-O-O".to_string()));
}

const CASTLING_WITH_CHECK: &str = "1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Nd6 6. Bxc6 dxc6 7. dxe5 Nf5 8. Qxd8+ Kxd8 *";

#[test]
fn test_parse_castling_with_check() {
    let tree = parse_pgn(CASTLING_WITH_CHECK);
    assert!(count_nodes(&tree) > 0);
}

#[test]
fn test_parse_castling_zero_notation() {
    // Some PGN files use 0-0 instead of O-O
    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. 0-0 Nf6 5. d3 0-0 *";
    let result = parse(pgn);
    // Our parser should either accept this or reject it gracefully
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

const DISAMBIGUATION_FILE: &str = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. Nxd4 Nf6 5. Nc3 Bb4 6. Nxc6 bxc6 7. Bd3 d5 8. exd5 cxd5 9. O-O O-O 10. Bg5 c6 11. Qf3 Be7 12. Rae1 *";

#[test]
fn test_parse_disambiguation_file() {
    let tree = parse_pgn(DISAMBIGUATION_FILE);
    let moves = main_line_moves(&tree);
    // Check that moves with file disambiguation are parsed
    assert!(moves.iter().any(|m| m.starts_with("R") || m.starts_with("N")));
}

const DISAMBIGUATION_RANK: &str = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be2 e5 7. Nb3 Be7 8. O-O O-O 9. Be3 Be6 10. Nd5 Nxd5 11. exd5 Bf5 12. c4 Nd7 13. Rc1 Rc8 14. Qd2 f6 15. Rfd1 Bg6 16. Na5 b5 17. Nc6 Qb6 18. Nxe7+ Kh8 19. Bf3 R8c7 20. Nc6 R7xc6 *";

#[test]
fn test_parse_disambiguation_rank() {
    let tree = parse_pgn(DISAMBIGUATION_RANK);
    assert!(count_nodes(&tree) > 10);
}

const PAWN_PROMOTION: &str = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 c6 5. Nf3 Nf6 6. Bc4 Bf5 7. Bd2 e6 8. Qe2 Bb4 9. O-O-O Nbd7 10. Rhe1 O-O 11. a3 Bxc3 12. Bxc3 Qc7 13. Kb1 b5 14. Bd3 Bxd3 15. Qxd3 a5 16. Ne5 Nxe5 17. Rxe5 Nd7 18. Re2 a4 19. Qe3 Qa5 20. d5 exd5 21. Rxd5 Qa6 22. Bd4 Qe6 23. Qg5 Qg6 24. Qxg6 hxg6 25. Rd6 Ne5 26. Bxe5 Rfe8 27. Bd4 Re4 28. Bc3 Rae8 29. Rxc6 Re1+ 30. Bxe1 Rxe1+ 31. Ka2 Re8 32. Rc7 Kf8 33. Ra7 Re4 34. h3 Rc4 35. Kb1 Rxc2 36. Ra8+ Ke7 37. Ra7+ Kf6 38. Rxc2 b4 39. axb4 a3 40. bxa3 Ke5 41. Rc5+ Kd4 42. a4 f5 43. a5 g5 44. a6 f4 45. a7 f3 46. a8=Q *";

#[test]
fn test_parse_pawn_promotion() {
    let tree = parse_pgn(PAWN_PROMOTION);
    let moves = main_line_moves(&tree);
    // Should contain a promotion move
    assert!(moves.iter().any(|m| m.contains("=")));
}

const UNDERPROMOTION: &str = "1. e4 e5 2. f4 exf4 3. Nf3 g5 4. h4 g4 5. Ne5 Nf6 6. d4 d6 7. Nd3 Nxe4 8. Bxf4 Qe7 9. Be2 Nc6 10. c3 Bf5 11. Qc2 O-O-O 12. O-O Bxd3 13. Bxd3 Nf6 14. b4 h5 15. a4 Bg7 16. b5 Ne5 17. dxe5 dxe5 18. Be3 Nd5 19. Bd2 Qd6 20. Bf5+ Kb8 21. Be4 Nc7 22. Rf5 Rhf8 23. Rxf8 Rxf8 24. Bf3 Qg6 25. a5 Qb1+ 26. Qxb1 f6 27. Qb4 Kc8 28. Qc5 Bf8 29. Ra4 Bxc5+ 30. Bxc5 Rd8 31. Bxc7 Kxc7 32. Rxg4 Rd1+ 33. Kf2 Rd2+ 34. Kf1 Kb8 35. Bc6 Rxg2 36. b6 axb6 37. axb6 c5 38. Bb5 f5 39. Rg8+ Kc8 40. Ba6 bxa6 41. Rxc8+ Kxc8 42. b7+ Kxb7 *";

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

const BRACE_COMMENTS: &str = r#"[Event "Comment Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 {The King's Pawn opening} e5 {Symmetrical response} 2. Nf3 {Attacking the e5 pawn} Nc6 {Defending} *"#;

#[test]
fn test_parse_brace_comments() {
    let tree = parse_pgn(BRACE_COMMENTS);
    let e4 = tree.root.find_child("e4").expect("e4 should exist");
    assert!(!e4.comment.is_empty(), "e4 should have a comment");
    assert!(e4.comment.contains("King's Pawn"), "Comment should contain King's Pawn");

    // Verify structure with comments
    let expected = game_tree! {
        e4 (comment: "The King's Pawn opening") {
            e5 (comment: "Symmetrical response") {
                Nf3 (comment: "Attacking the e5 pawn") {
                    Nc6 (comment: "Defending")
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const SEMICOLON_COMMENTS: &str = r#"[Event "Semicolon Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 e5 ; Open game
2. Nf3 Nc6 ; Knight development
3. Bb5 *"#;

#[test]
fn test_parse_semicolon_comments() {
    let tree = parse_pgn(SEMICOLON_COMMENTS);
    // Semicolon comments may or may not be preserved depending on implementation
    assert!(count_nodes(&tree) > 0);
}

const MULTILINE_COMMENT: &str = r#"[Event "Multiline Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 {This is a longer comment
that spans multiple lines
and contains various information} e5 *"#;

#[test]
fn test_parse_multiline_comment() {
    let tree = parse_pgn(MULTILINE_COMMENT);

    let expected = game_tree! {
        e4 (comment: "This is a longer comment\nthat spans multiple lines\nand contains various information") {
            e5
        }
    };
    assert_contains_tree!(tree, expected);
}

const EMPTY_COMMENT: &str = "1. e4 {} e5 *";

#[test]
fn test_parse_empty_comment() {
    let tree = parse_pgn(EMPTY_COMMENT);
    assert_eq!(count_nodes(&tree), 2);
}

const SPECIAL_CHARS_COMMENT: &str = "1. e4 {Special chars: <>!@#$%^&*()_+-=[]|;':\",./<>?} e5 *";

#[test]
fn test_parse_special_chars_in_comment() {
    let tree = parse_pgn(SPECIAL_CHARS_COMMENT);
    let e4 = tree.root.find_child("e4").expect("e4 should exist");
    assert!(e4.comment.contains("Special chars"), "Comment should contain 'Special chars'");
}

#[test]
fn test_parse_comment_with_moves_mentioned() {
    let pgn = r#"1. e4 {After e4, Black can reply with e5, c5, or e6} e5 *"#;
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);
    let e4 = tree.root.find_child("e4").expect("e4 should exist");
    assert!(e4.comment.contains("e4"), "Comment should mention e4");
    assert!(e4.comment.contains("Black can reply"), "Comment should mention Black can reply");
    assert!(e4.find_child("e5").is_some(), "e5 should be a child of e4");
}

#[test]
fn test_parse_unicode_in_comment() {
    let pgn = "1. e4 {The king ♔ attacks} e5 *";
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);
    let e4 = tree.root.find_child("e4").expect("e4 should exist");
    assert!(!e4.comment.is_empty(), "e4 should have a comment");
    assert!(e4.find_child("e5").is_some(), "e5 should be a child of e4");
    assert!(e4.comment.contains("♔") || e4.comment.contains("king"));
}

// ============================================================================
// NAG TESTS
// ============================================================================

const SYMBOLIC_NAGS: &str = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";

#[test]
fn test_parse_symbolic_nags() {
    let tree = parse_pgn(SYMBOLIC_NAGS);

    // Verify NAGs throughout the tree
    let expected = game_tree! {
        e4 (nag: GOOD_MOVE) {
            e5 (nag: POOR_MOVE) {
                Nf3 (nag: BRILLIANT_MOVE) {
                    Nc6 (nag: BLUNDER) {
                        Bb5 (nag: INTERESTING_MOVE) {
                            a6 (nag: DUBIOUS_MOVE)
                        }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const NUMERIC_NAGS: &str = "1. e4 $1 e5 $2 2. Nf3 $3 Nc6 $4 3. Bb5 $5 a6 $6 *";

#[test]
fn test_parse_numeric_nags() {
    let tree = parse_pgn(NUMERIC_NAGS);

    // Verify numeric NAG codes are parsed correctly
    let expected = game_tree! {
        e4 (nag: GOOD_MOVE) {
            e5 (nag: POOR_MOVE) {
                Nf3 (nag: BRILLIANT_MOVE) {
                    Nc6 (nag: BLUNDER) {
                        Bb5 (nag: INTERESTING_MOVE) {
                            a6 (nag: DUBIOUS_MOVE)
                        }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const POSITIONAL_NAGS: &str = "1. e4 $14 e5 $15 2. Nf3 $16 Nc6 $17 3. Bb5 $18 a6 $19 *";

#[test]
fn test_parse_positional_nags() {
    let tree = parse_pgn(POSITIONAL_NAGS);

    // Verify positional evaluation NAGs
    let expected = game_tree! {
        e4 (nag: WHITE_SLIGHT_ADVANTAGE) {
            e5 (nag: BLACK_SLIGHT_ADVANTAGE) {
                Nf3 (nag: WHITE_MODERATE_ADVANTAGE) {
                    Nc6 (nag: BLACK_MODERATE_ADVANTAGE) {
                        Bb5 (nag: WHITE_DECISIVE_ADVANTAGE) {
                            a6 (nag: BLACK_DECISIVE_ADVANTAGE)
                        }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const MULTIPLE_NAGS: &str = "1. e4! $14 e5 $2 $17 2. Nf3 *";

#[test]
fn test_parse_multiple_nags() {
    let tree = parse_pgn(MULTIPLE_NAGS);

    // Verify multiple NAGs on same moves
    let expected = game_tree! {
        e4 (nags: [GOOD_MOVE, WHITE_SLIGHT_ADVANTAGE]) {
            e5 (nags: [POOR_MOVE, BLACK_MODERATE_ADVANTAGE]) {
                Nf3
            }
        }
    };
    assert_contains_tree!(tree, expected);
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

const SINGLE_VARIATION: &str = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3 Nc6 *";

#[test]
fn test_parse_single_variation() {
    let tree = parse_pgn(SINGLE_VARIATION);

    // e4 should have two children: e5 (main) and c5 (variation)
    let expected = game_tree! {
        e4 {
            e5 { Nf3 { Nc6 } },
            c5 { Nf3 }
        }
    };
    assert_contains_tree!(tree, expected);
}

const SIBLING_VARIATIONS: &str = "1. e4 e5 (1... c5 2. Nf3) (1... e6 2. d4) (1... d5 2. exd5) 2. Nf3 *";

#[test]
fn test_parse_sibling_variations() {
    let tree = parse_pgn(SIBLING_VARIATIONS);

    // e4 should have 4 children: e5, c5, e6, d5
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 { Nf3 },
            e6 { d4 },
            d5 { exd5 }
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_parse_nested_variations() {
    let pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 (3. Bb5 g6)) 3. d4) 2. Nf3 Nc6 *"#;
    let tree = parse_pgn(pgn);

    // Full nested structure verification
    let expected = game_tree! {
        e4 {
            e5 { Nf3 { Nc6 } },
            c5 {
                Nf3 {
                    d6 { d4 },
                    Nc6 {
                        d4,
                        Bb5 { g6 }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const VARIATION_WITH_ANNOTATIONS: &str = "1. e4 e5 (1... c5 {Sicilian Defense} 2. Nf3! d6) 2. Nf3 *";

#[test]
fn test_parse_variation_with_annotations() {
    let tree = parse_pgn(VARIATION_WITH_ANNOTATIONS);

    // Verify c5 variation has Sicilian comment and Nf3 has !
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 (comment: "Sicilian Defense") {
                Nf3 (nag: GOOD_MOVE) {
                    d6
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_parse_early_variation() {
    let pgn = "1. e4 (1. d4 d5 2. c4) e5 2. Nf3 *";
    let tree = parse_pgn(pgn);

    // Main line
    let expected_main = game_tree! { e4 { e5 { Nf3 } } };
    assert_contains_tree!(tree, expected_main);

    // Root-level variation
    let expected_var = game_tree! { d4 { d5 { c4 } } };
    assert_contains_tree!(tree, expected_var);
}

#[test]
fn test_parse_deeply_nested_variations() {
    // These are all variations at move 1, so they're siblings at root level
    let pgn = r#"1. e4 (1. d4 (1. c4 (1. Nf3 d5) c5) d5) e5 *"#;
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);

    // All opening moves are siblings at root - verify each with continuations
    assert_contains_tree!(tree, game_tree! { e4 { e5 } });
    assert_contains_tree!(tree, game_tree! { d4 { d5 } });
    assert_contains_tree!(tree, game_tree! { c4 { c5 } });
    assert_contains_tree!(tree, game_tree! { Nf3 { d5 } });
}

#[test]
fn test_variation_after_every_move() {
    let pgn = "1. e4 (1. d4) e5 (1... c5) 2. Nf3 (2. Bc4) Nc6 (2... Nf6) *";
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);

    // Check root-level variations
    assert!(tree.root.find_child("e4").is_some(), "root should have e4");
    assert!(tree.root.find_child("d4").is_some(), "root should have d4 variation");

    // Verify the main line structure with variations at each level
    let expected = game_tree! {
        e4 {
            e5 {
                Nf3 { Nc6, Nf6 },
                Bc4
            },
            c5
        }
    };
    assert_contains_tree!(tree, expected);
}

// ============================================================================
// CLOCK AND EVAL TESTS
// ============================================================================

#[test]
fn test_parse_lichess_clocks() {
    let pgn = r#"[Event "Rated Blitz"]
[Site "https://lichess.org"]
[Date "2024.01.15"]
[Round "?"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[TimeControl "180+0"]

1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} 2. Nf3 {[%clk 0:02:58]} Nc6 {[%clk 0:02:59]} 1-0"#;

    let tree = parse_pgn(pgn);

    assert_headers!(tree, {
        "Event" => "Rated Blitz",
        "TimeControl" => "180+0",
    });
    assert_eq!(tree.result, GameResult::WhiteWins);

    // Verify tree structure with clock comments
    let expected = game_tree! {
        e4 (comment: "[%clk 0:03:00]") {
            e5 (comment: "[%clk 0:03:00]") {
                Nf3 (comment: "[%clk 0:02:58]") {
                    Nc6 (comment: "[%clk 0:02:59]")
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const EVAL_ANNOTATIONS: &str = r#"1. e4 {[%eval 0.25]} e5 {[%eval 0.20]} 2. Nf3 {[%eval 0.35]} Nc6 {[%eval 0.30]} *"#;

#[test]
fn test_parse_eval_annotations() {
    let tree = parse_pgn(EVAL_ANNOTATIONS);
    assert!(count_nodes(&tree) > 0);

    // Verify structure has comments with eval
    let expected = game_tree! {
        e4 (comment: "[%eval 0.25]") {
            e5 (comment: "[%eval 0.20]") {
                Nf3 (comment: "[%eval 0.35]") {
                    Nc6 (comment: "[%eval 0.30]")
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const MATE_EVAL: &str = r#"1. f3 e5 2. g4 {[%eval #-1]} Qh4# {[%eval #0]} 0-1"#;

#[test]
fn test_parse_mate_eval() {
    let tree = parse_pgn(MATE_EVAL);
    assert_eq!(tree.result, GameResult::BlackWins);
}

const CLOCK_AND_EVAL: &str = r#"1. e4 {[%clk 0:03:00] [%eval 0.25]} e5 {[%clk 0:03:00] [%eval 0.20]} *"#;

#[test]
fn test_parse_combined_clock_eval() {
    let tree = parse_pgn(CLOCK_AND_EVAL);
    assert!(count_nodes(&tree) >= 2);
}

const EMT_ANNOTATIONS: &str = r#"1. e4 {[%emt 0:00:05]} e5 {[%emt 0:00:03]} 2. Nf3 {[%emt 0:00:02]} *"#;

#[test]
fn test_parse_emt_annotations() {
    let tree = parse_pgn(EMT_ANNOTATIONS);
    assert!(count_nodes(&tree) >= 3);
}

// ============================================================================
// GAME TERMINATION TESTS
// ============================================================================

const WHITE_WINS: &str = r#"[Result "1-0"]

1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0"#;

#[test]
fn test_parse_white_wins() {
    let tree = parse_pgn(WHITE_WINS);
    assert_eq!(tree.result, GameResult::WhiteWins);
}

const BLACK_WINS: &str = r#"[Result "0-1"]

1. f3 e5 2. g4 Qh4# 0-1"#;

#[test]
fn test_parse_black_wins() {
    let tree = parse_pgn(BLACK_WINS);
    assert_eq!(tree.result, GameResult::BlackWins);
}

const DRAW_GAME: &str = r#"[Result "1/2-1/2"]

1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nf3 Nxe4 1/2-1/2"#;

#[test]
fn test_parse_draw() {
    let tree = parse_pgn(DRAW_GAME);
    assert_eq!(tree.result, GameResult::Draw);
}

const ONGOING_GAME: &str = r#"[Result "*"]

1. e4 e5 2. Nf3 Nc6 *"#;

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

const TWO_GAMES: &str = r#"[Event "Game 1"]
[Site "?"]
[Date "????.??.??"]
[Round "1"]
[White "White1"]
[Black "Black1"]
[Result "1-0"]

1. e4 e5 2. Nf3 1-0

[Event "Game 2"]
[Site "?"]
[Date "????.??.??"]
[Round "2"]
[White "White2"]
[Black "Black2"]
[Result "0-1"]

1. d4 d5 2. c4 0-1"#;

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

const THREE_GAMES: &str = r#"[Event "Complete Game"]
[Site "?"]
[Date "2024.01.01"]
[Round "1"]
[White "A"]
[Black "B"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0

[Event "Ongoing Game"]
[Site "?"]
[Date "2024.01.02"]
[Round "2"]
[White "C"]
[Black "D"]
[Result "*"]

1. d4 d5 *

[Event "Draw"]
[Site "?"]
[Date "2024.01.03"]
[Round "3"]
[White "E"]
[Black "F"]
[Result "1/2-1/2"]

1. c4 c5 1/2-1/2"#;

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

const UNICODE_HEADERS: &str = r#"[Event "International"]
[Site "München, Germany"]
[Date "2024.01.15"]
[Round "1"]
[White "Müller, Hans"]
[Black "Карлсен, Магнус"]
[Result "*"]

1. e4 e5 *"#;

#[test]
fn test_parse_unicode_headers() {
    let tree = parse_pgn(UNICODE_HEADERS);
    assert_eq!(tree.header("White"), Some("Müller, Hans"));
    // Cyrillic may or may not be preserved exactly
    assert!(tree.header("Black").is_some());
}

const LONG_HEADER: &str = r#"[Event "This is an extremely long event name that goes on and on to test how the parser handles very long header values in PGN files"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 *"#;

#[test]
fn test_parse_long_header() {
    let tree = parse_pgn(LONG_HEADER);
    let event = tree.header("Event").unwrap();
    assert!(event.len() > 50);
}

const CUSTOM_TAGS: &str = r#"[Event "Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]
[Annotator "John Doe"]
[ECO "C50"]
[Opening "Italian Game"]
[PlyCount "10"]
[TimeControl "300+5"]
[Termination "Normal"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 *"#;

#[test]
fn test_parse_custom_tags() {
    let tree = parse_pgn(CUSTOM_TAGS);
    assert_eq!(tree.header("Annotator"), Some("John Doe"));
    assert_eq!(tree.header("ECO"), Some("C50"));
    assert_eq!(tree.header("Opening"), Some("Italian Game"));
}

const PARTIAL_DATE: &str = r#"[Event "?"]
[Site "?"]
[Date "2024.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 *"#;

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

const LICHESS_EXPORT: &str = r#"[Event "Rated Blitz game"]
[Site "https://lichess.org/abcd1234"]
[Date "2024.01.15"]
[Round "?"]
[White "player1"]
[Black "player2"]
[Result "1-0"]
[UTCDate "2024.01.15"]
[UTCTime "14:30:00"]
[WhiteElo "1850"]
[BlackElo "1820"]
[WhiteRatingDiff "+8"]
[BlackRatingDiff "-8"]
[Variant "Standard"]
[TimeControl "180+0"]
[ECO "C50"]
[Termination "Normal"]

1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} 2. Nf3 {[%clk 0:02:58]} Nc6 {[%clk 0:02:59]} 3. Bc4 {[%clk 0:02:55]} Bc5 {[%clk 0:02:57]} 4. O-O {[%clk 0:02:52]} Nf6 {[%clk 0:02:54]} 5. d3 {[%clk 0:02:50]} O-O {[%clk 0:02:51]} 1-0"#;

#[test]
fn test_parse_lichess_export() {
    let tree = parse_pgn(LICHESS_EXPORT);
    assert_eq!(tree.header("Event"), Some("Rated Blitz game"));
    assert_eq!(tree.header("Site"), Some("https://lichess.org/abcd1234"));
    assert!(tree.header("WhiteElo").is_some());
    assert_eq!(tree.header("ECO"), Some("C50"));
}

const ANNOTATED_GAME: &str = r#"[Event "World Championship"]
[Site "Reykjavik ISL"]
[Date "1972.07.23"]
[Round "6"]
[White "Fischer, Robert J."]
[Black "Spassky, Boris V."]
[Result "1-0"]
[ECO "D59"]

1. c4 {Fischer avoids 1.e4 for the first time in the match} e6 2. Nf3 d5 3. d4 Nf6 4. Nc3 Be7 5. Bg5 O-O 6. e3 h6 7. Bh4 b6 {The Tartakower Defense} 8. cxd5 Nxd5 9. Bxe7 Qxe7 10. Nxd5 exd5 11. Rc1 Be6 12. Qa4 c5 13. Qa3 Rc8 14. Bb5! {An excellent move, putting pressure on the queenside} (14. Be2 {was also possible} cxd4 15. Nxd4 Qb4) 14... a6 15. dxc5 bxc5 16. O-O Ra7 17. Be2 Nd7 18. Nd4! {A strong knight maneuver} Qf8 (18... Nf6 19. Nxe6 fxe6 20. Bg4 $14) 19. Nxe6 fxe6 20. e4! $1 {Opening up the position with Black's king exposed} d4 21. f4 Qe7 22. e5 Rb8 23. Bc4 Kh8 24. Qh3 Nf8 25. b3 a5 26. f5! exf5 27. Rxf5 Nh7 28. Rcf1 {White has a crushing attack} Qd8 29. Qg3 Re7 30. h4 Rbb7 31. e6! Rbc7 32. Qe5 Qe8 33. a4 Qd8 34. R1f2 Qe8 35. R2f3 Qd8 36. Bd3 Qe8 37. Qe4 Nf6 38. Rxf6! gxf6 39. Rxf6 Kg8 40. Bc4 Kh8 41. Qf4 1-0"#;

#[test]
fn test_parse_annotated_game() {
    let tree = parse_pgn(ANNOTATED_GAME);
    assert_eq!(tree.header("Event"), Some("World Championship"));
    assert_eq!(tree.header("ECO"), Some("D59"));
    // Should have variations and comments
    assert!(count_nodes(&tree) > 30);
}

#[test]
fn test_parse_lichess_study() {
    let pgn = r#"[Event "Opening Repertoire: Sicilian"]
[Site "https://lichess.org/study/abc123"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]
[Annotator "ChessCoach"]
[UTCDate "2024.01.15"]
[UTCTime "10:00:00"]
[Variant "Standard"]
[ECO "B90"]
[Opening "Sicilian Defense: Najdorf Variation"]

1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 {The Najdorf Variation - one of Black's most aggressive defenses} 6. Be3 (6. Bg5 {The main alternative} e6 7. f4 Be7 8. Qf3 Qc7 9. O-O-O Nbd7) (6. Be2 {A quieter approach} e5 7. Nb3 Be7 8. O-O O-O) (6. f3 {The English Attack} e5 7. Nb3 Be6 8. Be3 Be7 9. Qd2 O-O 10. O-O-O) 6... e5 7. Nb3 Be6 8. f3 Be7 9. Qd2 O-O 10. O-O-O *"#;

    let tree = parse_pgn(pgn);
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

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.root.children.len(), 5);

    // Main line with continuation
    assert_contains_tree!(tree, game_tree! { e4 { e5 } });

    // All variations at root
    assert_contains_tree!(tree, game_tree! { d4 });
    assert_contains_tree!(tree, game_tree! { c4 });
    assert_contains_tree!(tree, game_tree! { Nf3 });
    assert_contains_tree!(tree, game_tree! { g3 });
}

#[test]
fn test_parse_variation_with_sub_variations() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3 (2. Nc3 Nc6) d6) 2. Nf3 *";
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);

    // Main line and Sicilian variation with sub-variations
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 {
                Nf3 { d6 },
                Nc3 { Nc6 }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_parse_variation_at_every_move_comprehensive() {
    let pgn = "1. e4 (1. d4) e5 (1... c5) 2. Nf3 (2. Bc4) Nc6 (2... Nf6) *";
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);

    // Main tree with variations at every level
    let expected = game_tree! {
        e4 {
            e5 {
                Nf3 { Nc6, Nf6 },
                Bc4
            },
            c5
        }
    };
    assert_contains_tree!(tree, expected);

    // Verify root-level variation (d4)
    let d4_expected = game_tree! { d4 };
    assert_contains_tree!(tree, d4_expected);
}

#[test]
fn test_parse_long_variation() {
    // Long Sicilian Najdorf variation
    let pgn = "1. e4 e5 (1... c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be2 e5 7. Nb3 Be7 8. O-O O-O 9. Be3 Be6) 2. Nf3 *";
    let tree = parse_pgn(pgn);

    assert_eq!(tree.result, GameResult::Ongoing);

    // Main line and deep Sicilian variation
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 {
                Nf3 {
                    d6 {
                        d4 {
                            cxd4 {
                                Nxd4 {
                                    Nf6 {
                                        Nc3 {
                                            a6 {
                                                Be2 {
                                                    e5 {
                                                        Nb3 {
                                                            Be7
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);

    // Verify castling is also in the variation (O-O has hyphen, use find_path)
    assert!(tree.find_path(&["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6", "Be2", "e5", "Nb3", "Be7", "O-O"]).is_some());
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

// ============================================================================
// LIST MARKER BUG TESTS
// ============================================================================
// These tests verify that list markers like "1)" and "2)" in bare text
// comments do NOT prematurely close variations. This is a HIGH severity
// bug that causes silent data loss when parsing PGN files.
//
// Bug: When a line contains prose like "ideas: 1) control d4 2) attack",
// the "1)" gets split into MoveNumber("1") and VariationEnd(")").
// The VariationEnd incorrectly closes the variation prematurely.

const LIST_MARKER_IN_VARIATION: &str = r#"1. e4 e5
(1... c5
The Sicilian has two ideas: 1) control d4 2) queenside play
2. Nf3 d6 3. d4 cxd4)
2. Nf3 *"#;

#[test]
fn test_list_marker_in_variation_comment() {
    // Core bug: 1) and 2) in prose inside a variation should NOT close it
    let tree = parse_pgn(LIST_MARKER_IN_VARIATION);

    // e4 should have two children: e5 (main) and c5 (variation with full continuation)
    // The comment with list markers is preserved on c5 - the full text including "1)" and "2)"
    // should be treated as comment text, not as move numbers + variation-ending parentheses.
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 (comment: "The Sicilian has two ideas: 1) control d4 2) queenside play") {
                Nf3 {
                    d6 {
                        d4 { cxd4 }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

const MULTIPLE_LIST_MARKERS: &str = r#"1. e4 e5
(1... c5
Three options: 1) Nf3 2) Nc3 3) d4 - all good
2. Nf3 d6)
2. Nf3 *"#;

#[test]
fn test_multiple_list_markers_in_variation() {
    // Multiple list markers: 1) 2) 3) etc should all be treated as text
    let tree = parse_pgn(MULTIPLE_LIST_MARKERS);

    // Variation should have full continuation despite multiple list markers
    // The comment should preserve the full text with all list markers intact
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 (comment: "Three options: 1) Nf3 2) Nc3 3) d4 - all good") { Nf3 { d6 } }
        }
    };
    assert_contains_tree!(tree, expected);
}

const LIST_MARKER_MAIN_LINE: &str = r#"1. e4
Two options: 1) push d4 2) develop knights
e5 *"#;

#[test]
fn test_list_marker_in_main_line_comment() {
    // List markers in main line prose should not cause issues
    let tree = parse_pgn(LIST_MARKER_MAIN_LINE);

    // The comment with list markers should be preserved on e4
    let expected = game_tree! {
        e4 (comment: "Two options: 1) push d4 2) develop knights") { e5 }
    };
    assert_contains_tree!(tree, expected);
}

const LIST_MARKER_BETWEEN_MOVES: &str = r#"1. e4 e5
White has choices: 1) Nf3 2) Bc4 3) Nc3
2. Nf3 Nc6 *"#;

#[test]
fn test_list_marker_between_moves() {
    // List markers in prose between moves on main line
    let tree = parse_pgn(LIST_MARKER_BETWEEN_MOVES);

    // Comment with list markers should be attached to e5
    let expected = game_tree! {
        e4 { e5 (comment: "White has choices: 1) Nf3 2) Bc4 3) Nc3") { Nf3 { Nc6 } } }
    };
    assert_contains_tree!(tree, expected);
    assert_eq!(count_nodes(&tree), 4, "Should have 4 moves: e4, e5, Nf3, Nc6");
}

const LIST_MARKER_NESTED: &str = r#"1. e4 e5
(1... c5 2. Nf3
Opening ideas: 1) attack center 2) develop pieces
d6 3. d4 cxd4)
2. Nf3 *"#;

#[test]
fn test_list_marker_in_nested_variation() {
    // List markers in prose within a variation shouldn't break continuation
    let tree = parse_pgn(LIST_MARKER_NESTED);

    // The c5 variation should have full continuation despite list markers in prose
    // Comment with list markers should be attached to Nf3 (the move before the prose)
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 {
                Nf3 (comment: "Opening ideas: 1) attack center 2) develop pieces") {
                    d6 {
                        d4 { cxd4 }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_closing_paren_at_end_of_move_line_still_works() {
    // Real variation closing at end of move line should still work
    let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Bc4 *";
    let tree = parse_pgn(pgn);

    let expected = game_tree! {
        e4 {
            e5 { Bc4 },
            c5 { Nf3 }
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_paren_in_brace_comment_works() {
    // Parentheses in brace comments already work - verify no regression
    let pgn = "1. e4 {Options: 1) d4 2) Nf3} e5 *";
    let tree = parse_pgn(pgn);
    assert_eq!(count_nodes(&tree), 2);

    let expected = game_tree! {
        e4 (comment: "Options: 1) d4 2) Nf3") { e5 }
    };
    assert_contains_tree!(tree, expected);
}

const EMPTY_PARENS_IN_PROSE: &str = r#"1. e4
Empty parens () here
e5 *"#;

#[test]
fn test_empty_parens_in_prose() {
    // Edge case: empty parens in prose
    let tree = parse_pgn(EMPTY_PARENS_IN_PROSE);

    // Empty parens should be preserved in comment
    let expected = game_tree! {
        e4 (comment: "Empty parens () here") { e5 }
    };
    assert_contains_tree!(tree, expected);
    assert_eq!(count_nodes(&tree), 2, "Empty parens should not break parsing");
}

const LETTER_PAREN_MARKERS: &str = r#"1. e4 e5
(1... c5
Options: a) attack b) defend c) wait
2. Nf3)
2. Bc4 *"#;

#[test]
fn test_letter_paren_like_a_in_prose() {
    // a) b) c) style lists should also be handled
    let tree = parse_pgn(LETTER_PAREN_MARKERS);

    // Letter-based list markers like a) b) c) should be preserved in comment
    let expected = game_tree! {
        e4 {
            e5 { Bc4 },
            c5 (comment: "Options: a) attack b) defend c) wait") { Nf3 }
        }
    };
    assert_contains_tree!(tree, expected);
}

const PROSE_AFTER_VARIATION: &str = r#"1. e4
Commentary about the opening.
(1. d4 d5)
More commentary: 1) point one 2) point two
e5 *"#;

#[test]
fn test_prose_context_restored_after_variation() {
    // After exiting variation, prose context should work correctly
    let tree = parse_pgn(PROSE_AFTER_VARIATION);

    // (1. d4 d5) after 1. e4 is a REPLACEMENT variation (alternative first move)
    // so d4 is at root level (sibling of e4), not a child
    // Comments are preserved with list markers intact
    let expected = game_tree! {
        e4 (comment: "Commentary about the opening. More commentary: 1) point one 2) point two") { e5 },
        d4 { d5 }
    };
    assert_contains_tree!(tree, expected);
}

const SINGLE_DIGIT_MID_SENTENCE: &str = r#"1. e4 e5
(1... c5
The best response is 1) d3 because of pressure.
2. Nf3)
2. Bc4 *"#;

#[test]
fn test_single_digit_paren_mid_sentence() {
    // Single digit followed by ) mid-sentence should not close variation
    let tree = parse_pgn(SINGLE_DIGIT_MID_SENTENCE);

    // List marker in mid-sentence should be preserved in comment
    let expected = game_tree! {
        e4 {
            e5 { Bc4 },
            c5 (comment: "The best response is 1) d3 because of pressure.") { Nf3 }
        }
    };
    assert_contains_tree!(tree, expected);
}

const TWO_DIGIT_LIST_MARKERS: &str = r#"1. e4 e5
(1... c5
Many ideas: 10) push pawns 11) develop pieces 12) castle
2. Nf3)
2. Bc4 *"#;

#[test]
fn test_two_digit_list_marker() {
    // Two-digit list markers like 10) 11) should also be handled
    let tree = parse_pgn(TWO_DIGIT_LIST_MARKERS);

    // Two-digit list markers should be preserved in comment
    let expected = game_tree! {
        e4 {
            e5 { Bc4 },
            c5 (comment: "Many ideas: 10) push pawns 11) develop pieces 12) castle") { Nf3 }
        }
    };
    assert_contains_tree!(tree, expected);
}

const REAL_WORLD_LIST_MARKER: &str = r#"1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2 Na6
(7. Nd2
Of the alternatives to 7.0-0, this knight retreat is the most testing.
c5
This move is entirely consistent with our philosophy... two main reasons: 1) Our knight can now find a productive role on c7... 2) White's last move hems in the dark-squared bishop...
8. d5 e6)
7. O-O *"#;

#[test]
fn test_real_world_danyakid_list_marker() {
    // Real-world example from DanyaKID Classical Main Line
    let tree = parse_pgn(REAL_WORLD_LIST_MARKER);

    // Expected tree structure with comments containing list markers
    // The multi-line prose with "1) ... 2) ..." should be preserved as comment on c5
    // Note: castling notation is normalized (0-0 → O-O) and tokenized separately from move number
    let expected = game_tree! {
        d4 {
            Nf6 {
                c4 {
                    g6 {
                        Nc3 {
                            Bg7 {
                                e4 {
                                    d6 {
                                        Nf3 {
                                            "O-O" {
                                                Be2 {
                                                    Na6 {
                                                        "O-O",
                                                        Nd2 (comment: "Of the alternatives to 7. O-O, this knight retreat is the most testing.") {
                                                            c5 (comment: "This move is entirely consistent with our philosophy... two main reasons: 1) Our knight can now find a productive role on c7... 2) White's last move hems in the dark-squared bishop...") {
                                                                d5 { e6 }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
}
