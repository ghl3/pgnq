//! Real-world PGN sample tests
//!
//! Tests using actual PGN exports from various sources to ensure
//! compatibility with real-world formats.
//!
//! These tests validate complete tree structures using the DSL,
//! not just scattered property checks.

mod common;
mod dsl;

use common::parse_pgn;
use dsl::*;
use pgnq::nag::Nag;
use pgnq::parser::parse;
use pgnq::tree::GameResult;

// ============================================================================
// Lichess Export Format
// ============================================================================

const LICHESS_GAME: &str = r#"[Event "Rated Blitz game"]
[Site "https://lichess.org/abcd1234"]
[Date "2024.01.15"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[UTCDate "2024.01.15"]
[UTCTime "12:00:00"]
[WhiteElo "1500"]
[BlackElo "1450"]
[WhiteRatingDiff "+8"]
[BlackRatingDiff "-8"]
[Variant "Standard"]
[TimeControl "180+0"]
[ECO "B20"]
[Opening "Sicilian Defense"]
[Termination "Normal"]

1. e4 { [%clk 0:03:00] } c5 { [%clk 0:03:00] } 2. Nf3 { [%clk 0:02:58] } d6 { [%clk 0:02:58] } 3. d4 { [%clk 0:02:55] } cxd4 { [%clk 0:02:55] } 4. Nxd4 { [%clk 0:02:53] } Nf6 { [%clk 0:02:52] } 5. Nc3 { [%clk 0:02:50] } a6 { [%clk 0:02:48] } 1-0
"#;

/// Comprehensive test for Lichess game format
/// Validates: headers, result, main line moves, clock annotations
#[test]
fn test_lichess_game_complete() {
    let tree = parse(LICHESS_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "Rated Blitz game")
        .header("White", "Player1")
        .header("Black", "Player2")
        .header("WhiteElo", "1500")
        .header("BlackElo", "1450")
        .header("ECO", "B20")
        .header("Opening", "Sicilian Defense")
        .white_wins()
        .node_count(10)
        .main_line(&["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6"])
        .root("e4", |n| n
            .comment_contains("%clk")
            .child("c5", |n| n
                .comment_contains("%clk")
                .child("Nf3", |n| n
                    .comment_contains("%clk")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Chess.com Export Format
// ============================================================================

const CHESSCOM_GAME: &str = r#"[Event "Live Chess"]
[Site "Chess.com"]
[Date "2024.01.15"]
[Round "-"]
[White "User1"]
[Black "User2"]
[Result "1/2-1/2"]
[CurrentPosition "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"]
[Timezone "UTC"]
[ECO "B00"]
[ECOUrl "https://www.chess.com/openings/Kings-Pawn-Opening"]
[WhiteElo "1200"]
[BlackElo "1250"]
[TimeControl "600"]
[Termination "Game drawn by agreement"]
[StartTime "10:00:00"]
[EndDate "2024.01.15"]
[EndTime "10:15:00"]
[Link "https://www.chess.com/game/live/12345678"]

1. e4 {[%clk 0:09:58.3]} e5 {[%clk 0:09:55.2]} 2. Nf3 {[%clk 0:09:50.1]} Nc6 {[%clk 0:09:48.7]} 3. Bb5 {[%clk 0:09:45.5]} a6 {[%clk 0:09:40.2]} 4. Ba4 {[%clk 0:09:38.1]} Nf6 {[%clk 0:09:35.8]} 5. O-O {[%clk 0:09:30.5]} 1/2-1/2
"#;

/// Comprehensive test for Chess.com game format
/// Validates: headers, result, fractional clocks, Ruy Lopez opening
#[test]
fn test_chesscom_game_complete() {
    let tree = parse(CHESSCOM_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "Live Chess")
        .header("Site", "Chess.com")
        .header("Termination", "Game drawn by agreement")
        .header("WhiteElo", "1200")
        .header("BlackElo", "1250")
        .draw()
        .node_count(9)
        .main_line(&["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O"])
        // Verify fractional clocks (Chess.com uses decimal seconds)
        .root("e4", |n| n
            .comment_contains(".") // Fractional seconds
            .child("e5", |n| n
                .comment_contains("9:55.2")
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Annotated Game Format
// ============================================================================

const ANNOTATED_GAME: &str = r#"[Event "World Championship"]
[Site "London"]
[Date "2018.11.28"]
[Round "12"]
[White "Carlsen, Magnus"]
[Black "Caruana, Fabiano"]
[Result "1/2-1/2"]
[WhiteElo "2835"]
[BlackElo "2832"]
[ECO "B33"]
[Opening "Sicilian Defense: Sveshnikov Variation"]
[Annotator "GM John Doe"]

1. e4! { The king's pawn opening. White stakes a claim in the center. }
1... c5 { The Sicilian Defense - the most popular response to 1.e4 at the top level. }
2. Nf3 $1 { Developing with tempo. }
2... Nc6 { Black develops naturally. }
3. d4 { Opening the center. }
3... cxd4
4. Nxd4 Nf6
5. Nc3 e5?! { The Sveshnikov! A double-edged choice. }
(5... d6 { The Najdorf would be a safer choice. } 6. Be2 e6 7. O-O $14)
6. Ndb5 d6
7. Bg5 a6
8. Na3 b5 $5 { An interesting pawn sacrifice. }
9. Bxf6 gxf6 $15 { Black's structure is damaged but compensation exists. }
1/2-1/2
"#;

/// Comprehensive test for annotated game with comments, NAGs, and variations
/// Validates complete tree structure including the Najdorf variation
#[test]
fn test_annotated_game_complete() {
    let tree = parse(ANNOTATED_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "World Championship")
        .header("White", "Carlsen, Magnus")
        .header("Black", "Caruana, Fabiano")
        .header("Annotator", "GM John Doe")
        .header("ECO", "B33")
        .draw()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE) // !
            .comment_contains("king's pawn")
            .child("c5", |n| n
                .comment_contains("Sicilian")
                .child("Nf3", |n| n
                    .has_nag() // $1
                    .child("Nc6", |n| n
                        .child("d4", |n| n
                            .child("cxd4", |n| n
                                .child("Nxd4", |n| n
                                    .child("Nf6", |n| n
                                        .child("Nc3", |n| n
                                            .has_variations() // e5 main + d6 variation
                                            .child("e5", |n| n
                                                .has_nag() // ?!
                                                .comment_contains("Sveshnikov")
                                            )
                                            // Verify the Najdorf variation exists
                                            .variation("d6", |n| n
                                                .comment_contains("Najdorf")
                                                .child("Be2", |n| n
                                                    .child("e6", |n| n
                                                        .child("O-O", |n| n
                                                            .has_nag() // $14
                                                        )
                                                    )
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Lichess Study Format
// ============================================================================

const LICHESS_STUDY: &str = r#"[Event "Opening Repertoire: Sicilian Defense"]
[Site "https://lichess.org/study/abcd1234"]
[Result "*"]
[Variant "Standard"]
[ECO "B20"]
[Opening "Sicilian Defense"]
[Annotator "https://lichess.org/@/StudyAuthor"]
[Chapter "Introduction to the Sicilian"]

1. e4 c5 { The Sicilian Defense is Black's most popular and successful response to 1.e4. It leads to asymmetrical positions with chances for both sides. } 2. Nf3 (2. Nc3 { The Closed Sicilian - a solid alternative. } 2... Nc6 3. g3 g6 4. Bg2 Bg7 5. d3 d6) (2. c3 { The Alapin Variation - trying to build a strong center. } 2... Nf6 3. e5 Nd5 4. d4 cxd4 5. cxd4) (2. f4 { The Grand Prix Attack - aggressive but risky. } 2... d5 3. exd5 Nf6) 2... d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 { The Najdorf Variation - the most complex and popular. } (5... e6 { The Scheveningen - solid and flexible. }) (5... g6 { The Dragon - sharp and tactical. }) (5... Nc6 { The Classical - developing naturally. }) *
"#;

/// Comprehensive test for Lichess study with multiple variations at each point
/// Validates: study headers, main line, and all variation branches
#[test]
fn test_lichess_study_complete() {
    let tree = parse(LICHESS_STUDY).unwrap();

    let expected = TreeExpectation::new()
        .header("Chapter", "Introduction to the Sicilian")
        .header("ECO", "B20")
        .ongoing()
        .root("e4", |n| n
            .child("c5", |n| n
                .comment_contains("Sicilian Defense")
                .has_variations() // Nf3 main + Nc3, c3, f4 variations
                // Main line continues with Nf3
                .child("Nf3", |n| n
                    .child("d6", |n| n
                        .child("d4", |n| n
                            .child("cxd4", |n| n
                                .child("Nxd4", |n| n
                                    .child("Nf6", |n| n
                                        .child("Nc3", |n| n
                                            .has_variations()
                                            .child("a6", |n| n
                                                .comment_contains("Najdorf")
                                            )
                                            .variation("e6", |n| n
                                                .comment_contains("Scheveningen")
                                            )
                                            .variation("g6", |n| n
                                                .comment_contains("Dragon")
                                            )
                                            .variation("Nc6", |n| n
                                                .comment_contains("Classical")
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
                // Verify the Closed Sicilian variation (sibling of Nf3)
                .variation("Nc3", |n| n
                    .comment_contains("Closed Sicilian")
                    .child("Nc6", |n| n
                        .child("g3", |n| n)
                    )
                )
                // Verify the Alapin variation
                .variation("c3", |n| n
                    .comment_contains("Alapin")
                )
                // Verify the Grand Prix variation
                .variation("f4", |n| n
                    .comment_contains("Grand Prix")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// TWIC (The Week In Chess) Format
// ============================================================================

const TWIC_GAME: &str = r#"[Event "Tata Steel Masters 2024"]
[Site "Wijk aan Zee NED"]
[Date "2024.01.20"]
[Round "6.1"]
[White "Gukesh D"]
[Black "Praggnanandhaa R"]
[Result "1-0"]
[WhiteTitle "GM"]
[BlackTitle "GM"]
[WhiteElo "2725"]
[BlackElo "2743"]
[ECO "D35"]
[Opening "QGD"]
[Variation "Exchange, positional line, 5...c6"]
[WhiteFideId "46616543"]
[BlackFideId "25059530"]
[EventDate "2024.01.13"]

1. d4 d5 2. c4 e6 3. Nc3 Nf6 4. cxd5 exd5 5. Bg5 c6 6. e3 Be7 7. Bd3 Nbd7 8. Nge2 O-O 9. Qc2 Re8 10. O-O Nf8 1-0
"#;

/// Test for TWIC format with FIDE IDs and tournament metadata
#[test]
fn test_twic_game_complete() {
    let tree = parse(TWIC_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("WhiteTitle", "GM")
        .header("BlackTitle", "GM")
        .header("WhiteFideId", "46616543")
        .header("BlackFideId", "25059530")
        .header("ECO", "D35")
        .header("Opening", "QGD")
        .white_wins()
        .node_count(20)
        .main_line(&["d4", "d5", "c4", "e6", "Nc3", "Nf6", "cxd5", "exd5", "Bg5", "c6"]);

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Stockfish Analysis Format
// ============================================================================

const STOCKFISH_ANALYSIS: &str = r#"[Event "Analysis"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]
[Annotator "Stockfish 16"]

1. e4 { [%eval +0.25] } e5 { [%eval +0.30] } 2. Nf3 { [%eval +0.28] } Nc6 { [%eval +0.32] } 3. Bb5 { [%eval +0.25] } a6 { [%eval +0.35] } 4. Ba4 { [%eval +0.30] } Nf6 { [%eval +0.28] } 5. O-O { [%eval +0.45] } Be7 { [%eval +0.40] } 6. Re1 { [%eval +0.38] } b5 { [%eval +0.42] } 7. Bb3 { [%eval +0.35] } O-O { [%eval +0.40] } *
"#;

/// Test for Stockfish analysis with eval annotations
#[test]
fn test_stockfish_analysis_complete() {
    let tree = parse(STOCKFISH_ANALYSIS).unwrap();

    let expected = TreeExpectation::new()
        .header("Annotator", "Stockfish 16")
        .ongoing()
        .node_count(14)
        .root("e4", |n| n
            .comment_contains("%eval")
            .comment_contains("+0.25")
            .child("e5", |n| n
                .comment_contains("%eval")
                .child("Nf3", |n| n
                    .comment_contains("%eval")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Scid Database Export
// ============================================================================

const SCID_GAME: &str = r#"[Event "World Championship Match"]
[Site "Reykjavik ISL"]
[Date "1972.07.23"]
[Round "6"]
[White "Fischer, Robert James"]
[Black "Spassky, Boris Vasilievich"]
[Result "1-0"]
[ECO "D59"]
[WhiteElo "2785"]
[BlackElo "2660"]
[PlyCount "81"]
[EventDate "1972.07.11"]
[EventType "match"]
[EventRounds "21"]
[EventCountry "ISL"]
[Source "ChessBase"]
[SourceDate "1998.11.10"]

1. c4 e6 2. Nf3 d5 3. d4 Nf6 4. Nc3 Be7 5. Bg5 O-O 6. e3 h6 7. Bh4 b6 8. cxd5 Nxd5 9. Bxe7 Qxe7 10. Nxd5 exd5 11. Rc1 Be6 12. Qa4 c5 13. Qa3 Rc8 14. Bb5 a6 15. dxc5 bxc5 16. O-O Ra7 17. Be2 Nd7 18. Nd4 Qf8 19. Nxe6 fxe6 20. e4 d4 21. f4 Qe7 22. e5 Rb8 23. Bc4 Kh8 24. Qh3 Nf8 25. b3 a5 26. f5 exf5 27. Rxf5 Nh7 28. Rcf1 Qd8 29. Qg3 Re7 30. h4 Rbb7 31. e6 Rbc7 32. Qe5 Qe8 33. a4 Qd8 34. R1f2 Qe8 35. R2f3 Qd8 36. Bd3 Qe8 37. Qe4 Nf6 38. Rxf6 gxf6 39. Rxf6 Kg8 40. Bc4 Kh8 41. Qf4 1-0
"#;

/// Test for Scid/ChessBase export with extensive metadata
#[test]
fn test_scid_game_complete() {
    let tree = parse(SCID_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("White", "Fischer, Robert James")
        .header("Black", "Spassky, Boris Vasilievich")
        .header("PlyCount", "81")
        .header("EventType", "match")
        .header("EventCountry", "ISL")
        .header("Source", "ChessBase")
        .white_wins()
        .node_count(81)
        .main_line(&["c4", "e6", "Nf3", "d5", "d4", "Nf6", "Nc3", "Be7"]);

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Arena Export Format
// ============================================================================

const ARENA_GAME: &str = r#"[Event "Computer Chess Game"]
[Site "Computer"]
[Date "2024.01.15"]
[Round "1"]
[White "Stockfish 16"]
[Black "Komodo Dragon 3"]
[Result "1-0"]
[TimeControl "40/300:0+0"]
[Time "12:00:00"]
[PlyCount "120"]
[Termination "adjudication"]
[WhiteType "program"]
[BlackType "program"]

1. e4 {+0.25/25 0.5s} e5 {+0.30/24 0.4s} 2. Nf3 {+0.28/26 0.6s} Nc6 {+0.32/25 0.5s} 3. Bb5 {+0.30/27 0.7s} 1-0
"#;

/// Test for Arena engine match format with depth/time annotations
#[test]
fn test_arena_game_complete() {
    let tree = parse(ARENA_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("Termination", "adjudication")
        .header("WhiteType", "program")
        .header("BlackType", "program")
        .white_wins()
        .root("e4", |n| n
            // Engine format: +0.25/25 0.5s (eval/depth time)
            .comment_contains("/25") // depth
            .comment_contains("0.5s") // time
            .child("e5", |n| n
                .comment_contains("/24")
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// PGN with Chess960/FRC
// ============================================================================

const CHESS960_GAME: &str = r#"[Event "Chess960"]
[Site "?"]
[Date "2024.01.15"]
[Round "1"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[Variant "Chess960"]
[SetUp "1"]
[FEN "brkqnrnb/pppppppp/8/8/8/8/PPPPPPPP/BRKQNRNB w FCfc - 0 1"]

1. e4 e5 2. d3 d6 3. Nf3 Nf6 4. Bg5 Be6 5. O-O 1-0
"#;

/// Test for Chess960 format with custom starting position
#[test]
fn test_chess960_game_complete() {
    let tree = parse(CHESS960_GAME).unwrap();

    let expected = TreeExpectation::new()
        .header("Variant", "Chess960")
        .header("SetUp", "1")
        .has_header("FEN")
        .white_wins()
        .node_count(9)
        .main_line(&["e4", "e5", "d3", "d6", "Nf3", "Nf6", "Bg5", "Be6", "O-O"]);

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Edge Case: Very Long Game
// ============================================================================

#[test]
fn test_very_long_game() {
    // Create a game with 200 moves
    let mut pgn = r#"[Event "Test"]
[Result "*"]

"#
    .to_string();

    for i in 1..=200 {
        if i % 2 == 1 {
            pgn.push_str(&format!("{}. Nc3 ", (i + 1) / 2));
        } else {
            pgn.push_str("Nc6 ");
        }
    }
    pgn.push('*');

    let tree = parse(&pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .node_count(200);

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Edge Case: Deep Variations (Textually Nested)
// ============================================================================

#[test]
fn test_deeply_nested_variations() {
    // The PGN has textually nested variations, but semantically they're all
    // alternatives for Black's first move (siblings of c5 under e4)
    let pgn = r#"[Event "Analysis"]
1. e4 c5 (1... e5 (1... d5 (1... Nf6 (1... g6 (1... b6))))) 2. Nf3 *"#;

    let tree = parse(pgn).unwrap();

    // All variations are siblings - alternatives for Black's 1st move
    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .has_variations()
            .children_count(6) // c5, e5, d5, Nf6, g6, b6
            .child("c5", |n| n
                .leaf("Nf3")
            )
            .variation("e5", |n| n)
            .variation("d5", |n| n)
            .variation("Nf6", |n| n)
            .variation("g6", |n| n)
            .variation("b6", |n| n)
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Edge Case: Many Sibling Variations
// ============================================================================

#[test]
fn test_many_sibling_variations() {
    let pgn = r#"[Event "Analysis"]
1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) (1. b3) (1. f4) (1. e3) (1. d3) c5 *"#;

    let tree = parse(pgn).unwrap();

    // Root should have 9 children: e4 main + 8 variations
    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .children_count(9) // c5 plus 8 variation siblings of e4... wait no
        );

    // Actually root.children has e4, d4, c4, etc. as siblings
    // Let me check: the e4 node should have c5 as child
    // The variations 1. d4, 1. c4 etc are siblings OF e4, not children of e4
    assert!(tree.root.children.len() >= 9, "Expected at least 9 first moves, got {}", tree.root.children.len());
}

// ============================================================================
// Roundtrip Tests with Real Games
// ============================================================================

#[test]
fn test_roundtrip_lichess_game() {
    use pgnq::serializer::{to_pgn, OutputOptions};

    let tree1 = parse(LICHESS_GAME).unwrap();
    let output = to_pgn(&tree1, &OutputOptions::default());
    let tree2 = parse(&output).unwrap();

    // Both trees should have same structure
    assert_eq!(tree1.count_nodes(), tree2.count_nodes());
    assert_eq!(tree1.result, tree2.result);
}

#[test]
fn test_roundtrip_annotated_game() {
    use pgnq::serializer::{to_pgn, OutputOptions};

    let tree1 = parse(ANNOTATED_GAME).unwrap();
    let output = to_pgn(&tree1, &OutputOptions::default());
    let tree2 = parse(&output).unwrap();

    // Both trees should have same structure
    assert_eq!(tree1.count_nodes(), tree2.count_nodes());
    assert_eq!(tree1.result, tree2.result);
}

#[test]
fn test_roundtrip_study_preserves_variations() {
    use pgnq::serializer::{to_pgn, OutputOptions};

    let tree1 = parse(LICHESS_STUDY).unwrap();
    let output = to_pgn(&tree1, &OutputOptions::default());
    let tree2 = parse(&output).unwrap();

    // Count variation nodes in both
    let var_count1 = tree1.root.iter_dfs().filter(|n| n.has_variations()).count();
    let var_count2 = tree2.root.iter_dfs().filter(|n| n.has_variations()).count();

    assert_eq!(var_count1, var_count2, "Variation count should be preserved");
}

// ============================================================================
// Additional Edge Cases for Parser Robustness
// ============================================================================

/// Test header with special characters (common in ChessBase exports)
#[test]
fn test_special_characters_in_headers() {
    let pgn = r#"[Event "Tata Steel A Group"]
[Site "Wijk aan Zee"]
[White "O'Kelly, Albéric"]
[Black "Müller, Hans"]
[Result "1-0"]

1. e4 e5 1-0"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .header("White", "O'Kelly, Albéric")
        .header("Black", "Müller, Hans")
        .white_wins()
        .main_line(&["e4", "e5"]);

    assert_tree_contains!(tree, expected);
}

/// Test unusual whitespace (tabs, multiple spaces, etc.)
#[test]
fn test_unusual_whitespace() {
    let pgn = "[Event \"Test\"]\n[Result \"*\"]\n\n1.  e4    e5\t2. Nf3\t\tNc6   3.  Bb5 *";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .node_count(5)
        .main_line(&["e4", "e5", "Nf3", "Nc6", "Bb5"]);

    assert_tree_contains!(tree, expected);
}

/// Test Windows line endings (CRLF)
#[test]
fn test_windows_line_endings() {
    let pgn = "[Event \"Test\"]\r\n[Site \"?\"]\r\n[Result \"*\"]\r\n\r\n1. e4 e5 2. Nf3 *\r\n";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "Test")
        .ongoing()
        .node_count(3);

    assert_tree_contains!(tree, expected);
}

/// Test missing space between move number and move
#[test]
fn test_no_space_after_move_number() {
    let pgn = "1.e4 e5 2.Nf3 Nc6 3.Bb5 *";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .node_count(5)
        .main_line(&["e4", "e5", "Nf3", "Nc6", "Bb5"]);

    assert_tree_contains!(tree, expected);
}

/// Test continuation move number style (1... for black's move)
#[test]
fn test_continuation_move_numbers() {
    let pgn = "1. e4 1... e5 2. Nf3 2... Nc6 *";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .node_count(4)
        .main_line(&["e4", "e5", "Nf3", "Nc6"]);

    assert_tree_contains!(tree, expected);
}

/// Test mixed comment styles in same game
#[test]
fn test_mixed_comment_styles() {
    let pgn = r#"[Event "Test"]
[Result "*"]

1. e4 {Brace comment} e5 ; Semicolon comment
2. Nf3 {Another brace} Nc6 ; More semicolon
*"#;

    let tree = parse(pgn).unwrap();

    // Should have comments on multiple moves (at least brace comments)
    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .has_comment()
            .comment_contains("Brace")
        );

    assert_tree_contains!(tree, expected);
}

/// Test NAGs mixed with comments
#[test]
fn test_nags_with_comments() {
    let pgn = "1. e4! {Great move!} e5? {Dubious} 2. Nf3!! {Brilliant} Nc6?? {Blunder} *";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .comment_contains("Great")
            .child("e5", |n| n
                .nag(Nag::POOR_MOVE)
                .comment_contains("Dubious")
                .child("Nf3", |n| n
                    .nag(Nag::BRILLIANT_MOVE)
                    .child("Nc6", |n| n
                        .nag(Nag::BLUNDER)
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test empty header values
#[test]
fn test_empty_header_values() {
    let pgn = r#"[Event ""]
[Site ""]
[White ""]
[Black ""]
[Result "*"]

1. e4 *"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "")
        .ongoing()
        .node_count(1);

    assert_tree_contains!(tree, expected);
}

/// Test very long header value
#[test]
fn test_long_header_value() {
    let long_name = "A".repeat(500);
    let pgn = format!(
        r#"[Event "{}"]
[Result "*"]

1. e4 *"#,
        long_name
    );

    let tree = parse(&pgn).unwrap();
    assert_eq!(tree.header("Event").unwrap().len(), 500);
}

/// Test game with no moves (headers only)
#[test]
fn test_headers_only() {
    let pgn = r#"[Event "Unplayed"]
[Site "?"]
[Date "2024.01.01"]
[Round "1"]
[White "Player1"]
[Black "Player2"]
[Result "*"]

*"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "Unplayed")
        .ongoing()
        .node_count(0);

    assert_tree_contains!(tree, expected);
}

/// Test multiple games - use parse_all to get all games
#[test]
fn test_multiple_games_parse_all() {
    use pgnq::parser::parse_all;

    let pgn = r#"[Event "Game 1"]
[Result "1-0"]

1. e4 e5 1-0

[Event "Game 2"]
[Result "0-1"]

1. d4 d5 0-1"#;

    // parse_all returns all games
    let trees = parse_all(pgn).unwrap();
    assert_eq!(trees.len(), 2);

    let expected1 = TreeExpectation::new()
        .header("Event", "Game 1")
        .white_wins()
        .main_line(&["e4", "e5"]);

    let expected2 = TreeExpectation::new()
        .header("Event", "Game 2")
        .black_wins()
        .main_line(&["d4", "d5"]);

    assert_tree_contains!(trees[0], expected1);
    assert_tree_contains!(trees[1], expected2);
}

/// Test PGN with BOM (Byte Order Mark)
#[test]
fn test_pgn_with_bom() {
    let pgn = "\u{FEFF}[Event \"Test\"]\n[Result \"*\"]\n\n1. e4 *";

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .node_count(1);

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Coverage Gap Tests: Combined Features
// ============================================================================

/// Test combining comments, NAGs, and variations on the same move
#[test]
fn test_combined_features_same_move() {
    let pgn = r#"1. e4! {A great opening move} e5 (1... c5! {The Sicilian}) (1... e6 {The French}) 2. Nf3 *"#;
    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .comment_contains("great opening")
            .has_variations()
            .children_count(3) // e5, c5, e6
            .child("e5", |n| n.leaf("Nf3"))
            .variation("c5", |n| n
                .nag(Nag::GOOD_MOVE)
                .comment_contains("Sicilian")
            )
            .variation("e6", |n| n
                .comment_contains("French")
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test deep tree with annotations at multiple depths
#[test]
fn test_deep_tree_with_annotations() {
    let pgn = r#"1. e4! {Start} e5? 2. Nf3!! {Attack} Nc6?? 3. Bb5!? {Ruy Lopez} a6?! 4. Ba4 Nf6 5. O-O {Castle early} Be7 *"#;
    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .has_comment()
            .child("e5", |n| n
                .nag(Nag::POOR_MOVE)
                .child("Nf3", |n| n
                    .nag(Nag::BRILLIANT_MOVE)
                    .has_comment()
                    .child("Nc6", |n| n
                        .nag(Nag::BLUNDER)
                        .child("Bb5", |n| n
                            .nag(Nag::INTERESTING_MOVE)
                            .comment_contains("Ruy Lopez")
                            .child("a6", |n| n
                                .nag(Nag::DUBIOUS_MOVE)
                                .child("Ba4", |n| n
                                    .child("Nf6", |n| n
                                        .child("O-O", |n| n
                                            .has_comment()
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test variations with their own sub-variations and annotations
#[test]
fn test_nested_variations_with_annotations() {
    let pgn = r#"1. e4 e5 (1... c5! {Sicilian} 2. Nf3 (2. Nc3!? {Closed}) d6 3. d4 cxd4!) 2. Nf3 Nc6 *"#;
    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .has_variations()
            .child("e5", |n| n
                .leaf("Nf3")
            )
            .variation("c5", |n| n
                .nag(Nag::GOOD_MOVE)
                .comment_contains("Sicilian")
                .has_variations() // 2. Nf3 and 2. Nc3
                .child("Nf3", |n| n
                    .child("d6", |n| n
                        .child("d4", |n| n
                            .child("cxd4", |n| n
                                .nag(Nag::GOOD_MOVE)
                            )
                        )
                    )
                )
                .variation("Nc3", |n| n
                    .nag(Nag::INTERESTING_MOVE)
                    .comment_contains("Closed")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test evaluation comments combined with clock annotations
#[test]
fn test_combined_eval_and_clock() {
    let pgn = r#"1. e4 {[%eval 0.25] [%clk 0:03:00]} e5 {[%eval 0.30] [%clk 0:02:58]} 2. Nf3 {[%eval 0.28] [%clk 0:02:55]} *"#;
    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .comment_contains("%eval")
            .comment_contains("%clk")
            .child("e5", |n| n
                .comment_contains("%eval")
                .comment_contains("%clk")
                .child("Nf3", |n| n
                    .comment_contains("%eval")
                    .comment_contains("%clk")
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test game with every type of annotation
#[test]
fn test_all_annotation_types() {
    let pgn = r#"[Event "Comprehensive Test"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[ECO "C50"]
[Annotator "Engine"]

1. e4! {Best move [%eval +0.35] [%clk 0:05:00]} e5? $17 2. Nf3!! {Developing [%clk 0:04:58]}
(2. Bc4!? {Italian setup} Nc6)
Nc6 3. Bc4 {Italian Game} Bc5 4. c3 Nf6 5. d4! exd4 $14 6. cxd4 Bb4+?! 1-0"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .header("Event", "Comprehensive Test")
        .header("ECO", "C50")
        .header("Annotator", "Engine")
        .white_wins()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .comment_contains("%eval")
            .comment_contains("%clk")
            .child("e5", |n| n
                .nag(Nag::POOR_MOVE)
                .has_nag() // $17
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test very long main line with annotations throughout
#[test]
fn test_long_game_with_annotations() {
    let pgn = r#"1. e4! e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez} a6 4. Ba4 Nf6 5. O-O Be7
6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 {Preventing ...Bg4} Nb8 10. d4 Nbd7
11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6 15. Bg5 h6 16. Bd2 Bg7
17. a4 c5 18. d5! {Space advantage} c4! 19. b4! cxb3 $14 20. Bxb3 *"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .child("e5", |n| n
                .child("Nf3", |n| n
                    .child("Nc6", |n| n
                        .child("Bb5", |n| n
                            .comment_contains("Ruy Lopez")
                        )
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
    // Verify node count is substantial
    assert!(tree.count_nodes() >= 38, "Should have at least 38 moves");
}

/// Test multiple sibling variations each with their own annotations
#[test]
fn test_many_annotated_variations() {
    let pgn = r#"1. e4 c5 (1... e5! {King's Pawn Game}) (1... e6 {French Defense} 2. d4 d5)
(1... c6 {Caro-Kann} 2. d4 d5!) (1... d5!? {Scandinavian} 2. exd5 Qxd5)
(1... g6 {Modern Defense}) (1... Nf6 {Alekhine's Defense}) 2. Nf3 d6 *"#;

    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .has_variations()
            .child("c5", |n| n
                .child("Nf3", |n| n
                    .leaf("d6")
                )
            )
            .variation("e5", |n| n
                .nag(Nag::GOOD_MOVE)
                .comment_contains("King's Pawn")
            )
            .variation("e6", |n| n
                .comment_contains("French")
            )
            .variation("c6", |n| n
                .comment_contains("Caro-Kann")
            )
            .variation("d5", |n| n
                .nag(Nag::INTERESTING_MOVE)
                .comment_contains("Scandinavian")
            )
            .variation("g6", |n| n
                .comment_contains("Modern")
            )
            .variation("Nf6", |n| n
                .comment_contains("Alekhine")
            )
        );

    assert_tree_contains!(tree, expected);
}

/// Test numeric NAGs (positional evaluations) combined with symbolic NAGs
#[test]
fn test_numeric_and_symbolic_nags() {
    let pgn = "1. e4! $14 e5? $17 2. Nf3!! $18 Nc6?? $19 *";
    let tree = parse(pgn).unwrap();

    let expected = TreeExpectation::new()
        .ongoing()
        .root("e4", |n| n
            .nag(Nag::GOOD_MOVE)
            .nag(Nag::WHITE_SLIGHT_ADVANTAGE)
            .child("e5", |n| n
                .nag(Nag::POOR_MOVE)
                .nag(Nag::BLACK_MODERATE_ADVANTAGE)
                .child("Nf3", |n| n
                    .nag(Nag::BRILLIANT_MOVE)
                    .nag(Nag::WHITE_DECISIVE_ADVANTAGE)
                    .child("Nc6", |n| n
                        .nag(Nag::BLUNDER)
                        .nag(Nag::BLACK_DECISIVE_ADVANTAGE)
                    )
                )
            )
        );

    assert_tree_contains!(tree, expected);
}

// ============================================================================
// Stress Tests
// ============================================================================

/// Test parsing a game with maximum practical nesting depth
#[test]
fn test_stress_deep_variation_nesting() {
    // 10 levels of nested variations - all are Black's alternatives at move 1
    let pgn = r#"1. e4 e5 (1... c5 (1... d5 (1... e6 (1... c6 (1... d6 (1... g6 (1... b6 (1... Nf6 (1... Nc6))))))))) 2. Nf3 *"#;
    let tree = parse(pgn).unwrap();

    // Should parse without stack overflow
    assert!(tree.count_nodes() >= 3, "Should have at least e4, e5, Nf3");

    // Root has e4 as its only child
    assert_eq!(tree.root.children.len(), 1);

    // e4 should have many children (e5 + all the nested variations which are siblings)
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.children.len() >= 2, "e4 should have e5 and variation alternatives");
}

/// Test parsing a game with many moves (100+ half-moves)
#[test]
fn test_stress_long_game() {
    let mut pgn = String::from("[Event \"Long Game\"]\n\n");
    for i in 1..=60 {
        pgn.push_str(&format!("{}. Nc3 Nc6 ", i));
    }
    pgn.push('*');

    let tree = parse(&pgn).unwrap();
    assert_eq!(tree.count_nodes(), 120);
}

/// Test parsing a game with many variations at each point
#[test]
fn test_stress_many_variations_per_move() {
    let pgn = r#"1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) (1. b3) (1. f4) (1. e3) (1. d3) (1. c3) (1. Nc3)
    e5 (1... c5) (1... e6) (1... c6) (1... d6) (1... g6) (1... d5) (1... Nf6) (1... Nc6)
    2. Nf3 *"#;

    let tree = parse(pgn).unwrap();

    // First move should have many alternatives
    assert!(tree.root.children.len() >= 10);
    // e4's replies should have many alternatives
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.children.len() >= 8);
}

/// Test game with annotations on every move
#[test]
fn test_stress_heavily_annotated() {
    let pgn = r#"1. e4! {Opening} e5? {Passive} 2. Nf3!! {Developing} Nc6?? {Error}
    3. Bb5!? {Lopez} a6?! {Defense} 4. Ba4! {Retreat} Nf6? {Counter}
    5. O-O!! {Castle} Be7?? {Develop} 6. Re1!? {Central} b5?! {Push} *"#;

    let tree = parse(pgn).unwrap();

    // Every move should have annotations
    for node in tree.root.iter_main_line().skip(1) {
        assert!(
            !node.nags.is_empty() || !node.comment.is_empty(),
            "Move {} should have annotation",
            node.san
        );
    }
}
