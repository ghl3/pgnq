//! Tests for the test DSL itself
//!
//! These tests verify that the DSL works correctly before using it
//! to test the parser.

mod common;
mod dsl;

use common::parse_pgn;
use dsl::*;
use pgnq::nag::Nag;
use pgnq::tree::GameResult;

// ============================================================================
// TreeExpectation Tests
// ============================================================================

#[test]
fn test_simple_main_line() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");

    let expected = TreeExpectation::new()
        .ongoing()
        .main_line(&["e4", "e5", "Nf3", "Nc6"]);

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_headers() {
    let tree = parse_pgn(
        r#"[Event "Test Tournament"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]

1. e4 e5 1-0"#,
    );

    let expected = TreeExpectation::new()
        .header("Event", "Test Tournament")
        .header("White", "Player1")
        .header("Black", "Player2")
        .white_wins();

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_result_variations() {
    // White wins
    let tree = parse_pgn("1. e4 e5 1-0");
    assert_tree_contains!(tree, TreeExpectation::new().white_wins());

    // Black wins
    let tree = parse_pgn("1. f3 e5 2. g4 Qh4# 0-1");
    assert_tree_contains!(tree, TreeExpectation::new().black_wins());

    // Draw
    let tree = parse_pgn("1. e4 e5 1/2-1/2");
    assert_tree_contains!(tree, TreeExpectation::new().draw());

    // Ongoing
    let tree = parse_pgn("1. e4 e5 *");
    assert_tree_contains!(tree, TreeExpectation::new().ongoing());
}

#[test]
fn test_node_count() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 *");
    assert_tree_contains!(tree, TreeExpectation::new().node_count(3));
}

// ============================================================================
// Inline Node Property Tests (New API)
// ============================================================================

#[test]
fn test_nag_expectation() {
    let tree = parse_pgn("1. e4! e5? 2. Nf3!! *");

    // NAGs declared inline with node creation
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .child("e5", |n| n
                .nag(Nag::POOR_MOVE)
                .child("Nf3", |n| n
                    .nag(Nag::BRILLIANT_MOVE)
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_comment_expectation() {
    let tree = parse_pgn("1. e4 {The King's Pawn opening} e5 {Symmetrical response} *");

    // Comments declared inline with node creation
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .comment_contains("King's Pawn")
            .child("e5", |n| n
                .comment_contains("Symmetrical")
            )
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_children_expectation() {
    let tree = parse_pgn("1. e4 e5 (1... c5) (1... e6) 2. Nf3 *");

    let expected = TreeExpectation::new()
        .root("e4", |n| n.children_count(3)); // e5, c5, e6

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_has_variations_expectation() {
    let tree = parse_pgn("1. e4 e5 (1... c5) 2. Nf3 *");

    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .has_variations()
            .child("e5", |n| n.no_variations()) // e5 has no sibling variations (Nf3 is child)
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_nested_children() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");

    // Deep tree with inline expectations
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .has_child("e5")
            .child("e5", |n| n
                .has_child("Nf3")
                .child("Nf3", |n| n
                    .has_child("Nc6")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Deep Tree Tests
// ============================================================================

#[test]
fn test_deep_nested_expectation() {
    let tree = parse_pgn("1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 *");

    // Deep tree with inline closures - much cleaner than old with_child
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .child("c5", |n| n
                .child("Nf3", |n| n
                    .child("d6", |n| n
                        .child("d4", |n| n
                            .child("cxd4", |n| n
                                .child("Nxd4", |n| n
                                    .has_child("Nf6")
                                )
                            )
                        )
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_variation_with_annotations() {
    let tree = parse_pgn("1. e4 e5 (1... c5 {Sicilian Defense} 2. Nf3!) 2. Nf3 *");

    // Variations and annotations all inline
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .children_count(2) // e5 and c5
            .child("e5", |n| n)
            .variation("c5", |n| n
                .comment_contains("Sicilian")
                .child("Nf3", |n| n
                    .has_nag()
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Assertion Macro Tests
// ============================================================================

#[test]
fn test_assert_main_line_macro() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 3. Bb5 *");
    assert_main_line!(tree, ["e4", "e5", "Nf3", "Nc6", "Bb5"]);
}

#[test]
fn test_assert_headers_macro() {
    let tree = parse_pgn(
        r#"[Event "Test"]
[White "Alice"]
[Black "Bob"]
[Result "*"]

1. e4 *"#,
    );

    assert_headers!(tree, {
        "Event" => "Test",
        "White" => "Alice",
        "Black" => "Bob"
    });
}

#[test]
fn test_assert_has_nag_macro() {
    let tree = parse_pgn("1. e4! e5 2. Nf3 *");
    assert_has_nag!(tree, ["e4"], Nag::GOOD_MOVE);
}

#[test]
fn test_assert_comment_contains_macro() {
    let tree = parse_pgn("1. e4 {The best move} e5 *");
    assert_comment_contains!(tree, ["e4"], "best");
}

#[test]
fn test_assert_node_count_macro() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 *");
    assert_node_count!(tree, 4);
}

#[test]
fn test_assert_result_macro() {
    let tree = parse_pgn("1. e4 e5 1-0");
    assert_result!(tree, GameResult::WhiteWins);
}

#[test]
fn test_assert_has_children_macro() {
    let tree = parse_pgn("1. e4 e5 *");
    assert_has_children!(tree, ["e4"]);
}

#[test]
fn test_assert_has_variations_macro() {
    let tree = parse_pgn("1. e4 e5 (1... c5) *");
    assert_has_variations!(tree, ["e4"]);
}

#[test]
fn test_assert_children_count_macro() {
    let tree = parse_pgn("1. e4 e5 (1... c5) (1... e6) *");
    assert_children_count!(tree, ["e4"], 3);
}

// ============================================================================
// Mismatch Detection Tests
// ============================================================================

#[test]
fn test_mismatch_detection_result() {
    let tree = parse_pgn("1. e4 e5 *");
    let expected = TreeExpectation::new().white_wins();

    let result = tree_contains(&tree, &expected);
    assert!(result.is_mismatch());

    let diffs = result.differences().unwrap();
    assert!(diffs.iter().any(|d| d.path.contains("result")));
}

#[test]
fn test_mismatch_detection_header() {
    let tree = parse_pgn(
        r#"[Event "Tournament"]
1. e4 *"#,
    );
    let expected = TreeExpectation::new().header("Event", "Different");

    let result = tree_contains(&tree, &expected);
    assert!(result.is_mismatch());
}

#[test]
fn test_mismatch_detection_main_line() {
    let tree = parse_pgn("1. e4 e5 *");
    let expected = TreeExpectation::new().main_line(&["e4", "c5"]); // Wrong second move

    let result = tree_contains(&tree, &expected);
    assert!(result.is_mismatch());
}

#[test]
fn test_mismatch_detection_missing_node() {
    let tree = parse_pgn("1. e4 e5 *");
    // Try to find Nf3 which doesn't exist
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .child("e5", |n| n
                .child("Nf3", |n| n.has_nag())
            )
        );

    let result = tree_contains(&tree, &expected);
    assert!(result.is_mismatch());
}

// ============================================================================
// StringMatcher Tests
// ============================================================================

#[test]
fn test_string_matcher_contains() {
    let tree = parse_pgn("1. e4 {The King's Pawn opening leads to open games} *");

    let expected = TreeExpectation::new()
        .root("e4", |n| n.comment_contains("open games"));

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_string_matcher_has_comment() {
    let tree = parse_pgn("1. e4 {Something} e5 *");

    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .has_comment()
            .child("e5", |n| n.no_comment())
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_tree() {
    let tree = parse_pgn("*");
    let expected = TreeExpectation::new().ongoing().node_count(0);
    assert_tree_contains!(tree, expected);
}

#[test]
fn test_single_move() {
    let tree = parse_pgn("1. e4 *");
    let expected = TreeExpectation::new().main_line(&["e4"]).node_count(1);
    assert_tree_contains!(tree, expected);
}

#[test]
fn test_castling_moves() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 O-O *");
    assert_tree_contains!(
        tree,
        TreeExpectation::new().main_line(&["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "O-O"])
    );
}

#[test]
fn test_promotion() {
    let tree = parse_pgn("1. e4 d5 2. exd5 c6 3. dxc6 Nxc6 4. d4 e5 5. d5 Nb4 6. a3 Na6 7. d6 Bxd6 8. Qxd6 Qxd6 9. Bb5+ Qd7 10. Bxd7+ Kxd7 11. c4 Nf6 12. b4 a5 13. b5 Nc5 14. Nc3 a4 15. Nf3 e4 16. Nd4 e3 17. f3 e2 18. Kd2 e1=Q+ *");

    let moves = tree.root.iter_main_line().skip(1).map(|n| &n.san).collect::<Vec<_>>();
    assert!(moves.iter().any(|m| m.contains("=")));
}

// ============================================================================
// New API Demonstration Tests
// ============================================================================

#[test]
fn test_complex_game_inline() {
    // A complex game with comments, NAGs, and variations
    let tree = parse_pgn(r#"
        1. e4 {King's Pawn} e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez}
        (3. Bc4 {Italian Game})
        a6 4. Ba4 Nf6 *
    "#);

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .comment_contains("King's Pawn")
            .child("e5", |n| n
                .child("Nf3", |n| n
                    .child("Nc6", |n| n
                        // Main line continues with Bb5
                        .child("Bb5", |n| n
                            .comment_contains("Ruy Lopez")
                            .child("a6", |n| n
                                .has_child("Ba4")
                            )
                        )
                        // Variation with Italian Game
                        .variation("Bc4", |n| n
                            .comment_contains("Italian")
                        )
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_leaf_shorthand() {
    let tree = parse_pgn("1. e4 e5 2. Nf3 *");

    // Using leaf() for simple nodes without properties
    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .child("e5", |n| n
                .leaf("Nf3")
            )
        );

    assert_tree_contains!(tree, expected);
}

#[test]
fn test_multiple_variations() {
    let tree = parse_pgn("1. e4 e5 (1... c5) (1... e6) (1... d5) *");

    let expected = TreeExpectation::new()
        .root("e4", |n| n
            .children_count(4) // e5, c5, e6, d5
            .child("e5", |n| n)
            .variation("c5", |n| n)
            .variation("e6", |n| n)
            .variation("d5", |n| n)
        );

    assert_tree_contains!(tree, expected);
}
