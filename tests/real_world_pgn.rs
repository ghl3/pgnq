//! Real-world PGN sample tests
//!
//! Tests using actual PGN exports from various sources to ensure
//! compatibility with real-world formats.

#[macro_use]
mod common;

use pgnq::nag::Nag;
use pgnq::parser::parse;
use pgnq::tree::GameResult;
use common::parse_pgn;

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

    // Check headers
    assert_eq!(tree.header("Event"), Some("Rated Blitz game"));
    assert_eq!(tree.header("White"), Some("Player1"));
    assert_eq!(tree.header("Black"), Some("Player2"));
    assert_eq!(tree.header("WhiteElo"), Some("1500"));
    assert_eq!(tree.header("BlackElo"), Some("1450"));
    assert_eq!(tree.header("ECO"), Some("B20"));
    assert_eq!(tree.header("Opening"), Some("Sicilian Defense"));

    // Check result and node count
    assert_eq!(tree.result, GameResult::WhiteWins);
    assert_eq!(tree.count_nodes(), 10);

    // Check main line
    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6"]);

    // Check clock annotations
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("%clk"), "e4 should have clock annotation");
    let c5 = e4.find_child("c5").unwrap();
    assert!(c5.comment.contains("%clk"), "c5 should have clock annotation");
    let nf3 = c5.find_child("Nf3").unwrap();
    assert!(nf3.comment.contains("%clk"), "Nf3 should have clock annotation");
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

    // Check headers
    assert_eq!(tree.header("Event"), Some("Live Chess"));
    assert_eq!(tree.header("Site"), Some("Chess.com"));
    assert_eq!(tree.header("Termination"), Some("Game drawn by agreement"));
    assert_eq!(tree.header("WhiteElo"), Some("1200"));
    assert_eq!(tree.header("BlackElo"), Some("1250"));

    // Check result and node count
    assert_eq!(tree.result, GameResult::Draw);
    assert_eq!(tree.count_nodes(), 9);

    // Check main line
    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O"]);

    // Verify fractional clocks (Chess.com uses decimal seconds)
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("."), "e4 should have fractional seconds");
    let e5 = e4.find_child("e5").unwrap();
    assert!(e5.comment.contains("9:55.2"), "e5 should have specific clock time");
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

    // Check headers
    assert_eq!(tree.header("Event"), Some("World Championship"));
    assert_eq!(tree.header("White"), Some("Carlsen, Magnus"));
    assert_eq!(tree.header("Black"), Some("Caruana, Fabiano"));
    assert_eq!(tree.header("Annotator"), Some("GM John Doe"));
    assert_eq!(tree.header("ECO"), Some("B33"));
    assert_eq!(tree.result, GameResult::Draw);

    // Navigate to verify tree structure with annotations
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.contains(&Nag::GOOD_MOVE), "e4 should have !");
    assert!(e4.comment.contains("king's pawn"), "e4 should have comment");

    let c5 = e4.find_child("c5").unwrap();
    assert!(c5.comment.contains("Sicilian"), "c5 should mention Sicilian");

    let nf3 = c5.find_child("Nf3").unwrap();
    assert!(!nf3.nags.is_empty(), "Nf3 should have NAG ($1)");

    let nc6 = nf3.find_child("Nc6").unwrap();
    let d4 = nc6.find_child("d4").unwrap();
    let cxd4 = d4.find_child("cxd4").unwrap();
    let nxd4 = cxd4.find_child("Nxd4").unwrap();
    let nf6 = nxd4.find_child("Nf6").unwrap();
    let nc3 = nf6.find_child("Nc3").unwrap();

    // Nc3 should have variations (e5 main + d6 variation)
    assert!(nc3.has_variations(), "Nc3 should have variations");

    // Main line continues with e5 (Sveshnikov)
    let e5 = nc3.find_child("e5").unwrap();
    assert!(!e5.nags.is_empty(), "e5 should have NAG (?!)");
    assert!(e5.comment.contains("Sveshnikov"), "e5 should mention Sveshnikov");

    // Verify the Najdorf variation exists
    let d6_var = nc3.children.iter().find(|n| n.san == "d6").expect("d6 variation should exist");
    assert!(d6_var.comment.contains("Najdorf"), "d6 should mention Najdorf");

    let be2 = d6_var.find_child("Be2").unwrap();
    let e6 = be2.find_child("e6").unwrap();
    let castle = e6.find_child("O-O").unwrap();
    assert!(!castle.nags.is_empty(), "O-O should have NAG ($14)");
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

    // Check headers
    assert_eq!(tree.header("Chapter"), Some("Introduction to the Sicilian"));
    assert_eq!(tree.header("ECO"), Some("B20"));
    assert_eq!(tree.result, GameResult::Ongoing);

    // Check main line structure
    let e4 = tree.root.find_child("e4").unwrap();
    let c5 = e4.find_child("c5").unwrap();
    assert!(c5.comment.contains("Sicilian Defense"));
    assert!(c5.has_variations(), "c5 should have variations for white's 2nd move");

    // Main line continues with Nf3
    let nf3 = c5.find_child("Nf3").unwrap();
    let d6 = nf3.find_child("d6").unwrap();
    let d4 = d6.find_child("d4").unwrap();
    let cxd4 = d4.find_child("cxd4").unwrap();
    let nxd4 = cxd4.find_child("Nxd4").unwrap();
    let nf6 = nxd4.find_child("Nf6").unwrap();
    let nc3 = nf6.find_child("Nc3").unwrap();

    // Nc3 has variations for black's response
    assert!(nc3.has_variations());
    let a6 = nc3.find_child("a6").unwrap();
    assert!(a6.comment.contains("Najdorf"));

    // Check Najdorf alternatives
    assert!(nc3.children.iter().any(|n| n.san == "e6" && n.comment.contains("Scheveningen")));
    assert!(nc3.children.iter().any(|n| n.san == "g6" && n.comment.contains("Dragon")));
    assert!(nc3.children.iter().any(|n| n.san == "Nc6" && n.comment.contains("Classical")));

    // Check variations at move 2
    assert!(c5.children.iter().any(|n| n.san == "Nc3" && n.comment.contains("Closed")));
    assert!(c5.children.iter().any(|n| n.san == "c3" && n.comment.contains("Alapin")));
    assert!(c5.children.iter().any(|n| n.san == "f4" && n.comment.contains("Grand Prix")));
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

    // Check headers
    assert_eq!(tree.header("WhiteTitle"), Some("GM"));
    assert_eq!(tree.header("BlackTitle"), Some("GM"));
    assert_eq!(tree.header("WhiteFideId"), Some("46616543"));
    assert_eq!(tree.header("BlackFideId"), Some("25059530"));
    assert_eq!(tree.header("ECO"), Some("D35"));
    assert_eq!(tree.header("Opening"), Some("QGD"));
    assert_eq!(tree.result, GameResult::WhiteWins);
    assert_eq!(tree.count_nodes(), 20);

    // Check first 10 moves
    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).take(10).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["d4", "d5", "c4", "e6", "Nc3", "Nf6", "cxd5", "exd5", "Bg5", "c6"]);
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

    // Check headers
    assert_eq!(tree.header("Annotator"), Some("Stockfish 16"));
    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 14);

    // Check eval annotations using game_tree! macro
    let expected = game_tree! {
        e4 (comment: "%eval") {
            e5 (comment: "%eval") {
                Nf3 (comment: "%eval")
            }
        }
    };
    assert_contains_tree!(tree, expected);

    // Additional specific check for first eval value
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("+0.25"));
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

    // Check headers
    assert_eq!(tree.header("White"), Some("Fischer, Robert James"));
    assert_eq!(tree.header("Black"), Some("Spassky, Boris Vasilievich"));
    assert_eq!(tree.header("PlyCount"), Some("81"));
    assert_eq!(tree.header("EventType"), Some("match"));
    assert_eq!(tree.header("EventCountry"), Some("ISL"));
    assert_eq!(tree.header("Source"), Some("ChessBase"));
    assert_eq!(tree.result, GameResult::WhiteWins);
    assert_eq!(tree.count_nodes(), 81);

    // Check first 8 moves
    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).take(8).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["c4", "e6", "Nf3", "d5", "d4", "Nf6", "Nc3", "Be7"]);
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

    // Check headers
    assert_eq!(tree.header("Termination"), Some("adjudication"));
    assert_eq!(tree.header("WhiteType"), Some("program"));
    assert_eq!(tree.header("BlackType"), Some("program"));
    assert_eq!(tree.result, GameResult::WhiteWins);

    // Check engine-style comments
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("/25"), "e4 should have depth");
    assert!(e4.comment.contains("0.5s"), "e4 should have time");

    let e5 = e4.find_child("e5").unwrap();
    assert!(e5.comment.contains("/24"));
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

    // Check headers
    assert_eq!(tree.header("Variant"), Some("Chess960"));
    assert_eq!(tree.header("SetUp"), Some("1"));
    assert!(tree.header("FEN").is_some());
    assert_eq!(tree.result, GameResult::WhiteWins);
    assert_eq!(tree.count_nodes(), 9);

    // Check main line
    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5", "d3", "d6", "Nf3", "Nf6", "Bg5", "Be6", "O-O"]);
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

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 200);
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

    assert_eq!(tree.result, GameResult::Ongoing);

    // Use game_tree! to verify the structure with all 6 variations
    let expected = game_tree! {
        e4 {
            c5 { Nf3 },
            e5,
            d5,
            Nf6,
            g6,
            b6
        }
    };
    assert_contains_tree!(tree, expected);
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

    assert_eq!(tree.header("White"), Some("O'Kelly, Albéric"));
    assert_eq!(tree.header("Black"), Some("Müller, Hans"));
    assert_eq!(tree.result, GameResult::WhiteWins);

    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5"]);
}

/// Test unusual whitespace (tabs, multiple spaces, etc.)
#[test]
fn test_unusual_whitespace() {
    let pgn = "[Event \"Test\"]\n[Result \"*\"]\n\n1.  e4    e5\t2. Nf3\t\tNc6   3.  Bb5 *";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 5);

    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5", "Nf3", "Nc6", "Bb5"]);
}

/// Test Windows line endings (CRLF)
#[test]
fn test_windows_line_endings() {
    let pgn = "[Event \"Test\"]\r\n[Site \"?\"]\r\n[Result \"*\"]\r\n\r\n1. e4 e5 2. Nf3 *\r\n";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.header("Event"), Some("Test"));
    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 3);
}

/// Test missing space between move number and move
#[test]
fn test_no_space_after_move_number() {
    let pgn = "1.e4 e5 2.Nf3 Nc6 3.Bb5 *";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 5);

    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5", "Nf3", "Nc6", "Bb5"]);
}

/// Test continuation move number style (1... for black's move)
#[test]
fn test_continuation_move_numbers() {
    let pgn = "1. e4 1... e5 2. Nf3 2... Nc6 *";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 4);

    let main_line: Vec<_> = tree.root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line, vec!["e4", "e5", "Nf3", "Nc6"]);
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

    assert_eq!(tree.result, GameResult::Ongoing);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(!e4.comment.is_empty());
    assert!(e4.comment.contains("Brace"));
}

/// Test NAGs mixed with comments
#[test]
fn test_nags_with_comments() {
    let pgn = "1. e4! {Great move!} e5? {Dubious} 2. Nf3!! {Brilliant} Nc6?? {Blunder} *";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.contains(&Nag::GOOD_MOVE));
    assert!(e4.comment.contains("Great"));

    let e5 = e4.find_child("e5").unwrap();
    assert!(e5.nags.contains(&Nag::POOR_MOVE));
    assert!(e5.comment.contains("Dubious"));

    let nf3 = e5.find_child("Nf3").unwrap();
    assert!(nf3.nags.contains(&Nag::BRILLIANT_MOVE));

    let nc6 = nf3.find_child("Nc6").unwrap();
    assert!(nc6.nags.contains(&Nag::BLUNDER));
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

    assert_eq!(tree.header("Event"), Some(""));
    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 1);
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

    assert_eq!(tree.header("Event"), Some("Unplayed"));
    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 0);
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

    assert_eq!(trees[0].header("Event"), Some("Game 1"));
    assert_eq!(trees[0].result, GameResult::WhiteWins);
    let main_line_1: Vec<_> = trees[0].root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line_1, vec!["e4", "e5"]);

    assert_eq!(trees[1].header("Event"), Some("Game 2"));
    assert_eq!(trees[1].result, GameResult::BlackWins);
    let main_line_2: Vec<_> = trees[1].root.iter_main_line().skip(1).map(|n| n.san.as_str()).collect();
    assert_eq!(main_line_2, vec!["d4", "d5"]);
}

/// Test PGN with BOM (Byte Order Mark)
#[test]
fn test_pgn_with_bom() {
    let pgn = "\u{FEFF}[Event \"Test\"]\n[Result \"*\"]\n\n1. e4 *";

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);
    assert_eq!(tree.count_nodes(), 1);
}

// ============================================================================
// Coverage Gap Tests: Combined Features
// ============================================================================

/// Test combining comments, NAGs, and variations on the same move
#[test]
fn test_combined_features_same_move() {
    let pgn = r#"1. e4! {A great opening move} e5 (1... c5! {The Sicilian}) (1... e6 {The French}) 2. Nf3 *"#;
    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    // Verify structure with NAGs, comments, and variations
    let expected = game_tree! {
        e4 (comment: "great opening", nag: GOOD_MOVE) {
            e5 { Nf3 },
            c5 (comment: "Sicilian", nag: GOOD_MOVE),
            e6 (comment: "French")
        }
    };
    assert_contains_tree!(tree, expected);
}

/// Test deep tree with annotations at multiple depths
#[test]
fn test_deep_tree_with_annotations() {
    let pgn = r#"1. e4! {Start} e5? 2. Nf3!! {Attack} Nc6?? 3. Bb5!? {Ruy Lopez} a6?! 4. Ba4 Nf6 5. O-O {Castle early} Be7 *"#;
    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    // Verify deep structure with NAGs and comments at multiple depths
    let expected = game_tree! {
        e4 (comment: "Start", nag: GOOD_MOVE) {
            e5 (nag: POOR_MOVE) {
                Nf3 (comment: "Attack", nag: BRILLIANT_MOVE) {
                    Nc6 (nag: BLUNDER) {
                        Bb5 (comment: "Ruy Lopez", nag: INTERESTING_MOVE) {
                            a6 (nag: DUBIOUS_MOVE) {
                                Ba4 {
                                    Nf6 {
                                        "O-O" (comment: "Castle")
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

/// Test variations with their own sub-variations and annotations
#[test]
fn test_nested_variations_with_annotations() {
    let pgn = r#"1. e4 e5 (1... c5! {Sicilian} 2. Nf3 (2. Nc3!? {Closed}) d6 3. d4 cxd4!) 2. Nf3 Nc6 *"#;
    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    // Verify main line and Sicilian variation with sub-variations
    let expected = game_tree! {
        e4 {
            e5 { Nf3 { Nc6 } },
            c5 (comment: "Sicilian", nag: GOOD_MOVE) {
                Nf3 {
                    d6 {
                        d4 {
                            cxd4 (nag: GOOD_MOVE)
                        }
                    }
                },
                Nc3 (comment: "Closed", nag: INTERESTING_MOVE)
            }
        }
    };
    assert_contains_tree!(tree, expected);
}

/// Test evaluation comments combined with clock annotations
#[test]
fn test_combined_eval_and_clock() {
    let pgn = r#"1. e4 {[%eval 0.25] [%clk 0:03:00]} e5 {[%eval 0.30] [%clk 0:02:58]} 2. Nf3 {[%eval 0.28] [%clk 0:02:55]} *"#;
    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    // Verify all moves have both eval and clock annotations
    let expected = game_tree! {
        e4 (comment: "%eval") {
            e5 (comment: "%eval") {
                Nf3 (comment: "%eval")
            }
        }
    };
    assert_contains_tree!(tree, expected);

    // Verify clock annotations are also present
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("%clk"));
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

    assert_eq!(tree.header("Event"), Some("Comprehensive Test"));
    assert_eq!(tree.header("ECO"), Some("C50"));
    assert_eq!(tree.header("Annotator"), Some("Engine"));
    assert_eq!(tree.result, GameResult::WhiteWins);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.contains(&Nag::GOOD_MOVE));
    assert!(e4.comment.contains("%eval"));
    assert!(e4.comment.contains("%clk"));

    let e5 = e4.find_child("e5").unwrap();
    assert!(e5.nags.contains(&Nag::POOR_MOVE));
    assert!(!e5.nags.is_empty()); // Should have $17 too
}

/// Test very long main line with annotations throughout
#[test]
fn test_long_game_with_annotations() {
    let pgn = r#"1. e4! e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez} a6 4. Ba4 Nf6 5. O-O Be7
6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 {Preventing ...Bg4} Nb8 10. d4 Nbd7
11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6 15. Bg5 h6 16. Bd2 Bg7
17. a4 c5 18. d5! {Space advantage} c4! 19. b4! cxb3 $14 20. Bxb3 *"#;

    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.contains(&Nag::GOOD_MOVE));

    let e5 = e4.find_child("e5").unwrap();
    let nf3 = e5.find_child("Nf3").unwrap();
    let nc6 = nf3.find_child("Nc6").unwrap();
    let bb5 = nc6.find_child("Bb5").unwrap();
    assert!(bb5.comment.contains("Ruy Lopez"));

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

    assert_eq!(tree.result, GameResult::Ongoing);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.has_variations());

    // Main line
    let c5 = e4.find_child("c5").unwrap();
    let nf3 = c5.find_child("Nf3").unwrap();
    assert!(nf3.find_child("d6").is_some());

    // Variations
    let e5_var = e4.children.iter().find(|n| n.san == "e5").unwrap();
    assert!(e5_var.nags.contains(&Nag::GOOD_MOVE));
    assert!(e5_var.comment.contains("King's Pawn"));

    let e6_var = e4.children.iter().find(|n| n.san == "e6").unwrap();
    assert!(e6_var.comment.contains("French"));

    let c6_var = e4.children.iter().find(|n| n.san == "c6").unwrap();
    assert!(c6_var.comment.contains("Caro-Kann"));

    let d5_var = e4.children.iter().find(|n| n.san == "d5").unwrap();
    assert!(d5_var.nags.contains(&Nag::INTERESTING_MOVE));
    assert!(d5_var.comment.contains("Scandinavian"));

    let g6_var = e4.children.iter().find(|n| n.san == "g6").unwrap();
    assert!(g6_var.comment.contains("Modern"));

    let nf6_var = e4.children.iter().find(|n| n.san == "Nf6").unwrap();
    assert!(nf6_var.comment.contains("Alekhine"));
}

/// Test numeric NAGs (positional evaluations) combined with symbolic NAGs
#[test]
fn test_numeric_and_symbolic_nags() {
    let pgn = "1. e4! $14 e5? $17 2. Nf3!! $18 Nc6?? $19 *";
    let tree = parse(pgn).unwrap();

    assert_eq!(tree.result, GameResult::Ongoing);

    // Verify each move has both symbolic and positional NAGs
    let expected = game_tree! {
        e4 (nags: [GOOD_MOVE, WHITE_SLIGHT_ADVANTAGE]) {
            e5 (nags: [POOR_MOVE, BLACK_MODERATE_ADVANTAGE]) {
                Nf3 (nags: [BRILLIANT_MOVE, WHITE_DECISIVE_ADVANTAGE]) {
                    Nc6 (nags: [BLUNDER, BLACK_DECISIVE_ADVANTAGE])
                }
            }
        }
    };
    assert_contains_tree!(tree, expected);
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
