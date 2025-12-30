//! Roundtrip tests: parse -> serialize -> parse
//!
//! These tests verify that parsing a PGN and serializing it back produces
//! an equivalent game tree when parsed again.

#[macro_use]
mod common;

use common::{main_line_moves, parse_pgn};
use pgnq::parser::parse;
use pgnq::serializer::{to_pgn, OutputFormat, OutputOptions};
use pgnq::tree::{GameNode, GameTree};
use pgnq::Nag;

// ============================================================================
// Helper Functions
// ============================================================================

/// Compare two game trees for structural equality
fn trees_equal(t1: &GameTree, t2: &GameTree) -> bool {
    if t1.result != t2.result {
        eprintln!("Result mismatch: {:?} vs {:?}", t1.result, t2.result);
        return false;
    }
    nodes_equal(&t1.root, &t2.root)
}

/// Compare two nodes recursively
fn nodes_equal(n1: &GameNode, n2: &GameNode) -> bool {
    if !n1.is_root() && n1.san != n2.san {
        eprintln!("SAN mismatch: '{}' vs '{}'", n1.san, n2.san);
        return false;
    }

    if n1.children.len() != n2.children.len() {
        eprintln!(
            "Child count mismatch at '{}': {} vs {}",
            n1.san,
            n1.children.len(),
            n2.children.len()
        );
        return false;
    }

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
    let pgn = "1. e4 e5 2. Nf3 Nc6 1-0";
    let (t1, t2) = roundtrip(pgn);
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
    let pgn = "1. d4 d5 2. c4 e6 3. Nc3 Nf6 *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));
}

const LONG_GAME: &str = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7 11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6 15. Bg5 h6 16. Bd2 Bg7 17. a4 c5 18. d5 c4 19. b4 cxb3 20. Bxb3 *";

#[test]
fn test_roundtrip_long_game() {
    let (t1, t2) = roundtrip(LONG_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.main_line_length(), t2.main_line_length());
}

// ============================================================================
// Comments Roundtrip
// ============================================================================

const BRACE_COMMENTS: &str = r#"[Result "*"]
1. e4 {The King's Pawn opening} e5 {Symmetrical response} 2. Nf3 {Attacking the e5 pawn} Nc6 {Defending} *"#;

#[test]
fn test_roundtrip_brace_comments() {
    let (t1, t2) = roundtrip(BRACE_COMMENTS);
    assert!(trees_equal(&t1, &t2));
    assert!(trees_equal_with_comments(&t1, &t2));
}

const MULTILINE_COMMENT: &str = r#"[Result "*"]
1. e4 {This is a longer comment
that spans multiple lines
and contains various information} e5 *"#;

#[test]
fn test_roundtrip_multiline_comment() {
    let (t1, t2) = roundtrip(MULTILINE_COMMENT);
    assert!(trees_equal(&t1, &t2));
    // Whitespace is normalized, so comments may differ slightly
    assert_eq!(t1.count_comments(), t2.count_comments());
}

const UNICODE_COMMENT: &str = "1. e4 {Müller plays the King's pawn: ♔ → ♙e4} e5 {Карлсен responds symmetrically} *";

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

const SPECIAL_CHARS_COMMENT: &str = "1. e4 {Special chars: <>!@#$%^&*()_+-=[]|;':\",./<>?} e5 *";

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
    let pgn = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal_with_nags(&t1, &t2));
}

#[test]
fn test_roundtrip_numeric_nags() {
    let pgn = "1. e4 $1 e5 $2 2. Nf3 $3 Nc6 $4 3. Bb5 $5 a6 $6 *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal_with_nags(&t1, &t2));
}

#[test]
fn test_roundtrip_multiple_nags() {
    let pgn = "1. e4! $14 e5 $2 $17 2. Nf3 *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal_with_nags(&t1, &t2));
}

// ============================================================================
// Variation Roundtrip
// ============================================================================

#[test]
fn test_roundtrip_single_variation() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3 Nc6 *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_sibling_variations() {
    let pgn = "1. e4 e5 (1... c5 2. Nf3) (1... e6 2. d4) (1... d5 2. exd5) 2. Nf3 *";
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));

    // Verify variation count
    let e4_1 = t1.root.find_child("e4").unwrap();
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert_eq!(e4_1.children.len(), e4_2.children.len());
}

#[test]
fn test_roundtrip_nested_variations() {
    let pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 (3. Bb5 g6)) 3. d4) 2. Nf3 Nc6 *"#;
    let (t1, t2) = roundtrip(pgn);
    assert!(trees_equal(&t1, &t2));
}

#[test]
fn test_roundtrip_deeply_nested_variations() {
    let pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 cxd4 (3... e6 4. Nxc6 (4. d5 Ne5) bxc6 5. e5) 4. Nxd4) 3. d4 cxd4 4. Nxd4) 2. Nf3 Nc6 (2... Nf6 3. Nxe5 d6 (3... Nxe4 4. Qe2 Qe7) 4. Nf3) 3. Bb5 *"#;
    let (t1, t2) = roundtrip(pgn);
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

const CASTLING_GAME: &str = r#"[Result "*"]
1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 O-O 6. c3 d6 *"#;

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

const CASTLING_ZERO_NOTATION: &str = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. 0-0 Nf6 5. d3 0-0 *";

#[test]
fn test_roundtrip_castling_zero_notation() {
    // Parser should normalize 0-0 to O-O
    let (t1, t2) = roundtrip(CASTLING_ZERO_NOTATION);
    assert!(trees_equal(&t1, &t2));
}

// ============================================================================
// Clock/Eval Annotations Roundtrip
// ============================================================================

const LICHESS_CLOCKS: &str = r#"[Result "1-0"]
1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} 2. Nf3 {[%clk 0:02:58]} Nc6 {[%clk 0:02:59]} 1-0"#;

#[test]
fn test_roundtrip_lichess_clocks() {
    let (t1, t2) = roundtrip(LICHESS_CLOCKS);
    assert!(trees_equal(&t1, &t2));

    // Verify clock comments preserved
    let e4_2 = t2.root.find_child("e4").unwrap();
    assert!(e4_2.comment.contains("%clk") || e4_2.comment.contains("clk"));
}

const EVAL_ANNOTATIONS: &str = r#"1. e4 {[%eval 0.25]} e5 {[%eval 0.20]} 2. Nf3 {[%eval 0.35]} Nc6 {[%eval 0.30]} *"#;

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

const ANNOTATED_GAME: &str = r#"[Event "World Championship"]
[ECO "D59"]
[Result "1-0"]

1. c4 {Fischer avoids 1.e4 for the first time in the match} e6 2. Nf3 d5 3. d4 Nf6 4. Nc3 Be7 5. Bg5 O-O 6. e3 h6 7. Bh4 b6 {The Tartakower Defense} 8. cxd5 Nxd5 9. Bxe7 Qxe7 10. Nxd5 exd5 11. Rc1 Be6 12. Qa4 c5 13. Qa3 Rc8 14. Bb5! {An excellent move, putting pressure on the queenside} (14. Be2 {was also possible} cxd4 15. Nxd4 Qb4) 14... a6 15. dxc5 bxc5 16. O-O Ra7 17. Be2 Nd7 18. Nd4! {A strong knight maneuver} Qf8 (18... Nf6 19. Nxe6 fxe6 20. Bg4 $14) 19. Nxe6 fxe6 20. e4! $1 {Opening up the position with Black's king exposed} d4 21. f4 Qe7 22. e5 Rb8 23. Bc4 Kh8 24. Qh3 Nf8 25. b3 a5 26. f5! exf5 27. Rxf5 Nh7 28. Rcf1 {White has a crushing attack} Qd8 29. Qg3 Re7 30. h4 Rbb7 31. e6! Rbc7 32. Qe5 Qe8 33. a4 Qd8 34. R1f2 Qe8 35. R2f3 Qd8 36. Bd3 Qe8 37. Qe4 Nf6 38. Rxf6! gxf6 39. Rxf6 Kg8 40. Bc4 Kh8 41. Qf4 1-0"#;

#[test]
fn test_roundtrip_standard_format() {
    let options = OutputOptions {
        format: OutputFormat::Standard,
        ..Default::default()
    };
    let (t1, t2, _) = roundtrip_with_options(ANNOTATED_GAME, &options);
    assert!(trees_equal(&t1, &t2));
}

const LICHESS_EXPORT: &str = r#"[Event "Rated Blitz game"]
[Site "https://lichess.org/abcd1234"]
[WhiteElo "1850"]
[BlackElo "1820"]
[ECO "C50"]
[Termination "Normal"]
[Result "1-0"]

1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} 2. Nf3 {[%clk 0:02:58]} Nc6 {[%clk 0:02:59]} 3. Bc4 {[%clk 0:02:55]} Bc5 {[%clk 0:02:57]} 4. O-O {[%clk 0:02:52]} Nf6 {[%clk 0:02:54]} 5. d3 {[%clk 0:02:50]} O-O {[%clk 0:02:51]} 1-0"#;

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

const FULL_HEADERS: &str = r#"[Event "Test Tournament"]
[Site "Test City"]
[Date "2024.01.15"]
[Round "1"]
[White "Player, White"]
[Black "Player, Black"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0"#;

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
    let nested_pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 (3. Bb5 g6)) 3. d4) 2. Nf3 Nc6 *"#;
    let (t1, t2, serialized) = roundtrip_with_options(nested_pgn, &options);

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
    let nags_pgn = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";
    let (t1, t2, serialized) = roundtrip_with_options(nags_pgn, &options);

    // NAGs should be absent from movetext
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

const WHITE_WINS: &str = r#"[Result "1-0"]
1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0"#;

#[test]
fn test_roundtrip_white_wins() {
    let (t1, t2) = roundtrip(WHITE_WINS);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

const BLACK_WINS: &str = r#"[Result "0-1"]
1. f3 e5 2. g4 Qh4# 0-1"#;

#[test]
fn test_roundtrip_black_wins() {
    let (t1, t2) = roundtrip(BLACK_WINS);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

const DRAW_GAME: &str = r#"[Result "1/2-1/2"]
1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nf3 Nxe4 1/2-1/2"#;

#[test]
fn test_roundtrip_draw() {
    let (t1, t2) = roundtrip(DRAW_GAME);
    assert!(trees_equal(&t1, &t2));
    assert_eq!(t1.result, t2.result);
}

const ONGOING_GAME: &str = r#"[Result "*"]
1. e4 e5 2. Nf3 Nc6 *"#;

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

const PROMOTION_GAME: &str = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 c6 5. Nf3 Nf6 6. Bc4 Bf5 7. Bd2 e6 8. Qe2 Bb4 9. O-O-O Nbd7 10. Rhe1 O-O 11. a3 Bxc3 12. Bxc3 Qc7 13. Kb1 b5 14. Bd3 Bxd3 15. Qxd3 a5 16. Ne5 Nxe5 17. Rxe5 Nd7 18. Re2 a4 19. Qe3 Qa5 20. d5 exd5 21. Rxd5 Qa6 22. Bd4 Qe6 23. Qg5 Qg6 24. Qxg6 hxg6 25. Rd6 Ne5 26. Bxe5 Rfe8 27. Bd4 Re4 28. Bc3 Rae8 29. Rxc6 Re1+ 30. Bxe1 Rxe1+ 31. Ka2 Re8 32. Rc7 Kf8 33. Ra7 Re4 34. h3 Rc4 35. Kb1 Rxc2 36. Ra8+ Ke7 37. Ra7+ Kf6 38. Rxc2 b4 39. axb4 a3 40. bxa3 Ke5 41. Rc5+ Kd4 42. a4 f5 43. a5 g5 44. a6 f4 45. a7 f3 46. a8=Q *";

#[test]
fn test_roundtrip_promotion() {
    let (t1, t2) = roundtrip(PROMOTION_GAME);
    assert!(trees_equal(&t1, &t2));

    let moves = main_line_moves(&t2);
    assert!(moves.iter().any(|m| m.contains('=')));
}

const DISAMBIGUATION_GAME: &str = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. Nxd4 Nf6 5. Nc3 Bb4 6. Nxc6 bxc6 7. Bd3 d5 8. exd5 cxd5 9. O-O O-O 10. Bg5 c6 11. Qf3 Be7 12. Rae1 *";

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
    let pgn = "1. e4 e5 2. Nf3 Nc6 1-0";
    let (t1, t2) = roundtrip(pgn);

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
    let nested_pgn = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 (3. Bb5 g6)) 3. d4) 2. Nf3 Nc6 *"#;

    let tree1 = parse_pgn(nested_pgn);
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

// ============================================================================
// Comprehensive NAG Roundtrip Tests (0-255)
// ============================================================================

/// Test that all NAG values 0-255 roundtrip correctly through $N notation
#[test]
fn test_roundtrip_all_nag_values_dollar_notation() {
    for nag_value in 0u8..=255 {
        let pgn = format!("1. e4 ${} e5 *", nag_value);
        let tree1 = parse(&pgn).expect(&format!("Failed to parse NAG ${}", nag_value));

        let options = OutputOptions::default();
        let serialized = to_pgn(&tree1, &options);
        let tree2 = parse(&serialized).expect(&format!("Failed to roundtrip NAG ${}", nag_value));

        let e4_1 = tree1.root.find_child("e4").unwrap();
        let e4_2 = tree2.root.find_child("e4").unwrap();

        assert_eq!(
            e4_1.nags, e4_2.nags,
            "NAG ${} did not roundtrip correctly. Original: {:?}, After: {:?}",
            nag_value, e4_1.nags, e4_2.nags
        );
    }
}

/// Test symbolic NAGs roundtrip correctly (!, ?, !!, ??, !?, ?!)
#[test]
fn test_roundtrip_symbolic_nags_all() {
    let symbolic_tests = [
        ("1. e4! e5 *", Nag::GOOD_MOVE, "!"),
        ("1. e4? e5 *", Nag::POOR_MOVE, "?"),
        ("1. e4!! e5 *", Nag::BRILLIANT_MOVE, "!!"),
        ("1. e4?? e5 *", Nag::BLUNDER, "??"),
        ("1. e4!? e5 *", Nag::INTERESTING_MOVE, "!?"),
        ("1. e4?! e5 *", Nag::DUBIOUS_MOVE, "?!"),
    ];

    for (pgn, expected_nag, symbol) in symbolic_tests {
        let tree1 = parse(pgn).expect(&format!("Failed to parse {}", symbol));

        let options = OutputOptions::default();
        let serialized = to_pgn(&tree1, &options);
        let tree2 = parse(&serialized).expect(&format!("Failed to roundtrip {}", symbol));

        let e4_1 = tree1.root.find_child("e4").unwrap();
        let e4_2 = tree2.root.find_child("e4").unwrap();

        assert!(
            e4_1.nags.contains(&expected_nag),
            "Original should contain {} NAG", symbol
        );
        assert!(
            e4_2.nags.contains(&expected_nag),
            "Roundtrip should preserve {} NAG", symbol
        );
    }
}

/// Test positional evaluation NAGs roundtrip correctly
#[test]
fn test_roundtrip_positional_nags() {
    let positional_tests = [
        (Nag::EQUAL, "$10", "="),
        (Nag::WHITE_SLIGHT_ADVANTAGE, "$14", "+="),
        (Nag::BLACK_SLIGHT_ADVANTAGE, "$15", "=+"),
        (Nag::WHITE_MODERATE_ADVANTAGE, "$16", "+/-"),
        (Nag::BLACK_MODERATE_ADVANTAGE, "$17", "-/+"),
        (Nag::WHITE_DECISIVE_ADVANTAGE, "$18", "+-"),
        (Nag::BLACK_DECISIVE_ADVANTAGE, "$19", "-+"),
    ];

    for (expected_nag, dollar, symbol) in positional_tests {
        let pgn = format!("1. e4 {} e5 *", dollar);
        let tree1 = parse(&pgn).expect(&format!("Failed to parse {}", symbol));

        let options = OutputOptions::default();
        let serialized = to_pgn(&tree1, &options);
        let tree2 = parse(&serialized).expect(&format!("Failed to roundtrip {}", symbol));

        let e4_1 = tree1.root.find_child("e4").unwrap();
        let e4_2 = tree2.root.find_child("e4").unwrap();

        assert!(
            e4_1.nags.contains(&expected_nag),
            "Original should contain {} NAG ({})", symbol, dollar
        );
        assert!(
            e4_2.nags.contains(&expected_nag),
            "Roundtrip should preserve {} NAG ({})", symbol, dollar
        );
    }
}

/// Test multiple NAGs on a single move roundtrip correctly
#[test]
fn test_roundtrip_multiple_nags_per_move() {
    // Move with both a move assessment and positional evaluation
    let pgn = "1. e4! $14 e5 *";
    let tree1 = parse(pgn).unwrap();

    let options = OutputOptions::default();
    let serialized = to_pgn(&tree1, &options);
    let tree2 = parse(&serialized).unwrap();

    let e4_1 = tree1.root.find_child("e4").unwrap();
    let e4_2 = tree2.root.find_child("e4").unwrap();

    assert_eq!(e4_1.nags.len(), 2, "Original should have 2 NAGs");
    assert_eq!(e4_2.nags.len(), 2, "Roundtrip should preserve 2 NAGs");
    assert!(e4_1.nags.contains(&Nag::GOOD_MOVE));
    assert!(e4_1.nags.contains(&Nag::WHITE_SLIGHT_ADVANTAGE));
    assert!(e4_2.nags.contains(&Nag::GOOD_MOVE));
    assert!(e4_2.nags.contains(&Nag::WHITE_SLIGHT_ADVANTAGE));
}

/// Test NAGs on multiple moves roundtrip correctly
#[test]
fn test_roundtrip_nags_on_multiple_moves() {
    let pgn = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";
    let tree1 = parse(pgn).unwrap();

    let options = OutputOptions::default();
    let serialized = to_pgn(&tree1, &options);
    let tree2 = parse(&serialized).unwrap();

    // Verify complete structure with all NAGs preserved
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
    assert_contains_tree!(tree2, expected);
}

/// Test NAGs inside variations roundtrip correctly
#[test]
fn test_roundtrip_nags_in_variations() {
    let pgn = "1. e4 e5 (1... c5! {Sicilian} 2. Nf3 $14) 2. Nf3 *";
    let tree1 = parse(pgn).unwrap();

    let options = OutputOptions::default();
    let serialized = to_pgn(&tree1, &options);
    let tree2 = parse(&serialized).unwrap();

    // Verify structure with NAGs in variations preserved
    let expected = game_tree! {
        e4 {
            e5 { Nf3 },
            c5 (comment: "Sicilian", nag: GOOD_MOVE) {
                Nf3 (nag: WHITE_SLIGHT_ADVANTAGE)
            }
        }
    };
    assert_contains_tree!(tree2, expected);
}

/// Test uncommon NAG values (those without symbols) roundtrip
#[test]
fn test_roundtrip_uncommon_nags() {
    // Test a selection of uncommon NAG values that don't have symbols
    let uncommon_nags = [0, 7, 8, 9, 11, 12, 13, 20, 21, 22, 50, 100, 150, 200, 254, 255];

    for &nag_value in &uncommon_nags {
        let pgn = format!("1. e4 ${} e5 *", nag_value);
        let tree1 = parse(&pgn).expect(&format!("Failed to parse NAG ${}", nag_value));

        let options = OutputOptions::default();
        let serialized = to_pgn(&tree1, &options);

        // Verify the NAG appears in serialized output as $N
        assert!(
            serialized.contains(&format!("${}", nag_value)),
            "Serialized output should contain ${}", nag_value
        );

        let tree2 = parse(&serialized).expect(&format!("Failed to roundtrip NAG ${}", nag_value));

        let e4_1 = tree1.root.find_child("e4").unwrap();
        let e4_2 = tree2.root.find_child("e4").unwrap();

        assert_eq!(
            e4_1.nags, e4_2.nags,
            "NAG ${} did not roundtrip correctly", nag_value
        );
    }
}

/// Test NAG boundary values specifically
#[test]
fn test_roundtrip_nag_boundaries() {
    // Test 0, 1, 254, 255 specifically as boundary values
    let boundaries = [
        (0, "$0"),
        (1, "!"),      // Symbolic
        (254, "$254"),
        (255, "$255"),
    ];

    for (nag_value, expected_display) in boundaries {
        let pgn = format!("1. e4 ${} e5 *", nag_value);
        let tree1 = parse(&pgn).unwrap();

        let options = OutputOptions::default();
        let serialized = to_pgn(&tree1, &options);

        // NAG 1 displays as "!" symbol, others as $N
        if nag_value == 1 {
            assert!(
                serialized.contains("e4!") || serialized.contains("e4 !"),
                "NAG 1 should serialize as ! symbol"
            );
        }

        let tree2 = parse(&serialized).unwrap();

        let _e4_1 = tree1.root.find_child("e4").unwrap();
        let e4_2 = tree2.root.find_child("e4").unwrap();

        let expected_nag = Nag::new(nag_value);
        assert!(
            e4_2.nags.contains(&expected_nag),
            "NAG {} ({}) should roundtrip correctly", nag_value, expected_display
        );
    }
}
