//! Roundtrip tests: parse -> serialize -> parse
//!
//! These tests verify that parsing a PGN and serializing it back produces
//! an equivalent game tree when parsed again.

mod common;

use common::*;
use pgnq::parser::parse;
use pgnq::serializer::{to_pgn, OutputFormat, OutputOptions};
use pgnq::tree::{GameNode, GameTree};

// ============================================================================
// Helper Functions
// ============================================================================

/// Compare two game trees for structural equality
fn trees_equal(t1: &GameTree, t2: &GameTree) -> bool {
    // Compare results
    if t1.result != t2.result {
        eprintln!("Result mismatch: {:?} vs {:?}", t1.result, t2.result);
        return false;
    }

    // Compare tree structure
    nodes_equal(&t1.root, &t2.root)
}

/// Compare two nodes recursively
fn nodes_equal(n1: &GameNode, n2: &GameNode) -> bool {
    // Compare SAN (skip for root)
    if !n1.is_root() && n1.san != n2.san {
        eprintln!("SAN mismatch: '{}' vs '{}'", n1.san, n2.san);
        return false;
    }

    // Compare child count
    if n1.children.len() != n2.children.len() {
        eprintln!(
            "Child count mismatch at '{}': {} vs {}",
            n1.san,
            n1.children.len(),
            n2.children.len()
        );
        return false;
    }

    // Compare children
    for (c1, c2) in n1.children.iter().zip(n2.children.iter()) {
        if !nodes_equal(c1, c2) {
            return false;
        }
    }

    true
}

/// Compare trees including NAGs
fn trees_equal_with_nags(t1: &GameTree, t2: &GameTree) -> bool {
    if t1.result != t2.result {
        return false;
    }
    nodes_equal_with_nags(&t1.root, &t2.root)
}

fn nodes_equal_with_nags(n1: &GameNode, n2: &GameNode) -> bool {
    if !n1.is_root() {
        if n1.san != n2.san {
            return false;
        }
        if n1.nags != n2.nags {
            eprintln!("NAG mismatch at '{}': {:?} vs {:?}", n1.san, n1.nags, n2.nags);
            return false;
        }
    }

    if n1.children.len() != n2.children.len() {
        return false;
    }

    for (c1, c2) in n1.children.iter().zip(n2.children.iter()) {
        if !nodes_equal_with_nags(c1, c2) {
            return false;
        }
    }

    true
}

/// Compare trees including comments (normalized whitespace)
fn trees_equal_with_comments(t1: &GameTree, t2: &GameTree) -> bool {
    if t1.result != t2.result {
        return false;
    }
    nodes_equal_with_comments(&t1.root, &t2.root)
}

fn nodes_equal_with_comments(n1: &GameNode, n2: &GameNode) -> bool {
    if !n1.is_root() {
        if n1.san != n2.san {
            return false;
        }
        // Normalize whitespace for comment comparison
        let c1: String = n1.comment.split_whitespace().collect::<Vec<_>>().join(" ");
        let c2: String = n2.comment.split_whitespace().collect::<Vec<_>>().join(" ");
        if c1 != c2 {
            eprintln!("Comment mismatch at '{}': '{}' vs '{}'", n1.san, c1, c2);
            return false;
        }
    }

    if n1.children.len() != n2.children.len() {
        return false;
    }

    for (c1, c2) in n1.children.iter().zip(n2.children.iter()) {
        if !nodes_equal_with_comments(c1, c2) {
            return false;
        }
    }

    true
}

/// Perform roundtrip test with default options
fn roundtrip(pgn: &str) -> (GameTree, GameTree) {
    let tree1 = parse_pgn(pgn);
    let options = OutputOptions::default();
    let serialized = to_pgn(&tree1, &options);
    let tree2 = parse(&serialized).expect("Failed to parse roundtrip PGN");
    (tree1, tree2)
}

/// Perform roundtrip test with custom options
fn roundtrip_with_options(pgn: &str, options: &OutputOptions) -> (GameTree, GameTree, String) {
    let tree1 = parse_pgn(pgn);
    let serialized = to_pgn(&tree1, options);
    let tree2 = parse(&serialized).expect("Failed to parse roundtrip PGN");
    (tree1, tree2, serialized)
}

// ============================================================================
// Simple Game Roundtrips
// ============================================================================

#[test]
fn test_roundtrip_minimal_game() {
    let (t1, t2) = roundtrip(MINIMAL_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

#[test]
fn test_roundtrip_game_with_headers() {
    let pgn = r#"[Event "Test Event"]
[Site "Test Site"]
[Date "2024.01.15"]
[Round "1"]
[White "Player 1"]
[Black "Player 2"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0"#;

    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));

    // Verify headers preserved
    assert_eq!(t2.header("Event"), Some("Test Event"));
    assert_eq!(t2.header("White"), Some("Player 1"));
    assert_eq!(t2.header("Black"), Some("Player 2"));
}

#[test]
fn test_roundtrip_headerless_game() {
    let (t1, t2) = roundtrip(HEADERLESS_GAME);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_long_game() {
    let (t1, t2) = roundtrip(LONG_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.main_line_length(), t2.main_line_length());
}

// ============================================================================
// Comments Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_brace_comments() {
    let (t1, t2) = roundtrip(BRACE_COMMENTS);
    assert!(trees_equal(&t1, &t2));
    assert!(trees_equal_with_comments(&t1, &t2));
}

#[test]
fn test_roundtrip_multiline_comment() {
    let (t1, t2) = roundtrip(MULTILINE_COMMENT);
    assert!(trees_equal(&t1, &t2));
    // Whitespace is normalized, so comments may differ slightly
    assert_eq!(t1.count_comments(), t2.count_comments());
}

#[test]
fn test_roundtrip_unicode_comment() {
    let (t1, t2) = roundtrip(UNICODE_COMMENT);
    assert!(trees_equal(&t1, &t2));

    // Verify unicode preserved
    let e4_1 = t1.root.find_child("e4").unwrap();
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert!(e4_1.comment.contains("Müller"));
    assert!(e4_2.comment.contains("Müller"));
}

#[test]
fn test_roundtrip_special_chars_comment() {
    let (t1, t2) = roundtrip(SPECIAL_CHARS_COMMENT);
    assert!(trees_equal(&t1, &t2));
}

// ============================================================================
// NAG Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_symbolic_nags() {
    let (t1, t2) = roundtrip(SYMBOLIC_NAGS);
    assert!(trees_equal_with_nags(&t1, &t2));
}

#[test]
fn test_roundtrip_numeric_nags() {
    let (t1, t2) = roundtrip(NUMERIC_NAGS);
    assert!(trees_equal_with_nags(&t1, &t2));
}

#[test]
fn test_roundtrip_multiple_nags() {
    let (t1, t2) = roundtrip(MULTIPLE_NAGS);
    assert!(trees_equal_with_nags(&t1, &t2));
}

// ============================================================================
// Variation Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_single_variation() {
    let (t1, t2) = roundtrip(SINGLE_VARIATION);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_sibling_variations() {
    let (t1, t2) = roundtrip(SIBLING_VARIATIONS);
    assert!(trees_equal(&t1, &t2));

    // Verify variation count
    let e4_1 = t1.root.find_child("e4").unwrap();
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert_eq!(e4_1.children.len(), e4_2.children.len());
}

#[test]
fn test_roundtrip_nested_variations() {
    let (t1, t2) = roundtrip(NESTED_VARIATIONS);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_deeply_nested_variations() {
    let (t1, t2) = roundtrip(DEEPLY_NESTED);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.max_depth(), t2.max_depth());
}

#[test]
fn test_roundtrip_variation_with_annotations() {
    let pgn = "1. e4 (1. d4 {Queen's pawn} Nf6!) e5 2. Nf3";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));

    // Verify NAGs in variation
    let d4 = t1.root.find_child("d4").unwrap();
    let nf6 = d4.find_child("Nf6").unwrap();
    assert!(!nf6.nags.is_empty());
}

// ============================================================================
// Castling Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_castling() {
    let (t1, t2) = roundtrip(CASTLING_GAME);
    assert!(trees_equal(&t1, &t2));

    // Verify castling moves preserved
    let moves1 = main_line_moves(&t1);
    let moves2 = main_line_moves(&t2);
    assert_eq!(moves1, moves2);
    assert!(moves1.iter().any(|m| m == "O-O" || m == "O-O-O"));
}

#[test]
fn test_roundtrip_castling_zero_notation() {
    // Parser should normalize 0-0 to O-O
    let (t1, t2) = roundtrip(CASTLING_ZERO_NOTATION);
    assert!(trees_equal(&t1, &t2));
}

// ============================================================================
// Clock/Eval Annotations Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_lichess_clocks() {
    let (t1, t2) = roundtrip(LICHESS_CLOCKS);
    assert!(trees_equal(&t1, &t2));

    // Verify clock comments preserved
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert!(e4_2.comment.contains("%clk") || e4_2.comment.contains("clk"));
}

#[test]
fn test_roundtrip_eval_annotations() {
    let (t1, t2) = roundtrip(EVAL_ANNOTATIONS);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_strip_clocks() {
    let options = OutputOptions {
        strip_clocks: true,
        ..Default::default()
    };
    let (t1, t2, serialized) = roundtrip_with_options(LICHESS_CLOCKS, &options);

    // Moves should still match
    assert!(trees_equal(&t1, &t2));

    // Clock data should be stripped
    assert!(!serialized.contains("%clk"));
}

#[test]
fn test_roundtrip_strip_evals() {
    let options = OutputOptions {
        strip_evals: true,
        ..Default::default()
    };
    let (t1, t2, serialized) = roundtrip_with_options(EVAL_ANNOTATIONS, &options);

    assert!(trees_equal(&t1, &t2));
    assert!(!serialized.contains("%eval"));
}

// ============================================================================
// Output Format Roundtrips
// ============================================================================

#[test]
fn test_roundtrip_standard_format() {
    let options = OutputOptions {
        format: OutputFormat::Standard,
        ..Default::default()
    };
    let (t1, t2, _) = roundtrip_with_options(ANNOTATED_GAME, &options);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_lichess_format() {
    let options = OutputOptions::lichess();
    let (t1, t2, _) = roundtrip_with_options(LICHESS_EXPORT, &options);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_minimal_format() {
    let options = OutputOptions::minimal();
    let tree1 = parse_pgn(ANNOTATED_GAME);
    let serialized = to_pgn(&tree1, &options);
    let tree2 = parse(&serialized).expect("Failed to parse");

    // Minimal format strips variations, so just compare main line
    let moves1 = main_line_moves(&tree1);
    let moves2 = main_line_moves(&tree2);
    assert_eq!(moves1, moves2);
}

// ============================================================================
// Output Option Variations
// ============================================================================

#[test]
fn test_roundtrip_no_headers() {
    let options = OutputOptions {
        headers: false,
        ..Default::default()
    };
    let (t1, t2, serialized) = roundtrip_with_options(FULL_HEADERS, &options);

    // Moves should match
    assert!(trees_equal(&t1, &t2));

    // Headers should be absent from output
    assert!(!serialized.contains("[Event"));
    assert!(!serialized.contains("[White"));
}

#[test]
fn test_roundtrip_no_comments() {
    let options = OutputOptions {
        comments: false,
        ..Default::default()
    };
    let (_, t2, serialized) = roundtrip_with_options(BRACE_COMMENTS, &options);

    // Comments should be absent
    assert!(!serialized.contains("{"));
    assert_eq!(t2.count_comments(), 0);
}

#[test]
fn test_roundtrip_no_variations() {
    let options = OutputOptions {
        variations: false,
        ..Default::default()
    };
    let (t1, t2, serialized) = roundtrip_with_options(NESTED_VARIATIONS, &options);

    // Variations should be absent
    assert!(!serialized.contains("("));

    // Main line preserved
    let moves1 = main_line_moves(&t1);
    let moves2 = main_line_moves(&t2);
    assert_eq!(moves1, moves2);
}

#[test]
fn test_roundtrip_no_nags() {
    let options = OutputOptions {
        nags: false,
        headers: false, // Disable headers to avoid "?" in site/date
        ..Default::default()
    };
    let (t1, t2, serialized) = roundtrip_with_options(SYMBOLIC_NAGS, &options);

    // NAGs should be absent from movetext (check for patterns like "e4!" or "$1")
    // Note: We can't just check for ! or ? since they might appear in other contexts
    assert!(!serialized.contains("e4!"));
    assert!(!serialized.contains("e5?"));
    assert!(!serialized.contains("Nf3!!"));
    assert!(!serialized.contains("Nc6??"));
    assert!(!serialized.contains("$"));

    // Moves still match
    assert!(trees_equal(&t1, &t2));
}

// ============================================================================
// Game Results Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_white_wins() {
    let (t1, t2) = roundtrip(WHITE_WINS);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

#[test]
fn test_roundtrip_black_wins() {
    let (t1, t2) = roundtrip(BLACK_WINS);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

#[test]
fn test_roundtrip_draw() {
    let (t1, t2) = roundtrip(DRAW_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

#[test]
fn test_roundtrip_ongoing() {
    let (t1, t2) = roundtrip(ONGOING_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

// ============================================================================
// Real-World PGN Roundtrips
// ============================================================================

#[test]
fn test_roundtrip_lichess_export() {
    let (t1, t2) = roundtrip(LICHESS_EXPORT);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_annotated_game() {
    let (t1, t2) = roundtrip(ANNOTATED_GAME);
    assert!(trees_equal(&t1, &t2));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_roundtrip_promotion() {
    let (t1, t2) = roundtrip(PROMOTION_GAME);
    assert!(trees_equal(&t1, &t2));

    let moves = main_line_moves(&t2);
    assert!(moves.iter().any(|m| m.contains('=')));
}

#[test]
fn test_roundtrip_disambiguation() {
    let (t1, t2) = roundtrip(DISAMBIGUATION_GAME);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_all_move_types() {
    // Comprehensive game with many move types
    let pgn = r#"1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7
6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6
15. Bg5 h6 16. Bd2 Bg7 17. a4 c5 18. d5 c4 19. b4 cxb3
20. Bxb3 Nc5 21. Bc2 Qc7 22. Rc1 Rac8 23. Qe2 Nfd7 *"#;

    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.main_line_length(), t2.main_line_length());
}

#[test]
fn test_roundtrip_many_variations() {
    let pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6) (1... e6 2. d4 d5) (1... c6 2. d4 d5)
2. Nf3 Nc6 (2... Nf6 3. Nxe5) 3. Bb5 a6 (3... Nf6) (3... f5) 4. Ba4 *"#;

    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));

    // Verify variation count at e4
    let e4_1 = t1.root.find_child("e4").unwrap();
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert_eq!(e4_1.children.len(), e4_2.children.len());
}

#[test]
fn test_roundtrip_preserves_move_order() {
    let (t1, t2) = roundtrip(MINIMAL_GAME);

    let moves1 = main_line_moves(&t1);
    let moves2 = main_line_moves(&t2);

    assert_eq!(moves1, moves2);
    for (m1, m2) in moves1.iter().zip(moves2.iter()) {
        assert_eq!(m1, m2);
    }
}

#[test]
fn test_roundtrip_complex_annotated() {
    let pgn = r#"[Event "Test"]

1. e4! {The king's pawn!} e5? (1... c5! {Sicilian} 2. Nf3 d6 $14)
2. Nf3!! {Developing with tempo} Nc6?!
3. Bb5 $1 {The Ruy Lopez} a6 {Morphy Defense} 4. Ba4 Nf6 5. O-O *"#;

    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));
    assert!(trees_equal_with_nags(&t1, &t2));
}

// ============================================================================
// Stability Tests (serialize twice)
// ============================================================================

#[test]
fn test_double_roundtrip_stable() {
    let tree1 = parse_pgn(ANNOTATED_GAME);
    let options = OutputOptions::default();

    // First roundtrip
    let pgn2 = to_pgn(&tree1, &options);
    let tree2 = parse(&pgn2).unwrap();

    // Second roundtrip
    let pgn3 = to_pgn(&tree2, &options);
    let tree3 = parse(&pgn3).unwrap();

    // Output should be identical after first roundtrip
    assert_eq!(pgn2, pgn3);
    assert!(trees_equal(&tree2, &tree3));
}

#[test]
fn test_triple_roundtrip_stable() {
    let options = OutputOptions::default();

    let tree1 = parse_pgn(NESTED_VARIATIONS);
    let pgn2 = to_pgn(&tree1, &options);
    let tree2 = parse(&pgn2).unwrap();
    let pgn3 = to_pgn(&tree2, &options);
    let tree3 = parse(&pgn3).unwrap();
    let pgn4 = to_pgn(&tree3, &options);

    // All subsequent roundtrips should be identical
    assert_eq!(pgn2, pgn3);
    assert_eq!(pgn3, pgn4);
    assert!(trees_equal(&tree2, &tree3));
}
