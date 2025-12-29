//! Real-world PGN sample tests
//!
//! Tests using actual PGN exports from various sources to ensure
//! compatibility with real-world formats.

mod common;

use common::parse_pgn;
use pgnq::parser::parse;

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

#[test]
fn test_lichess_game_parses() {
    let tree = parse(LICHESS_GAME).unwrap();
    assert_eq!(tree.header("Event"), Some("Rated Blitz game"));
    assert_eq!(tree.header("White"), Some("Player1"));
    assert_eq!(tree.header("WhiteElo"), Some("1500"));
}

#[test]
fn test_lichess_clocks_present() {
    let tree = parse(LICHESS_GAME).unwrap();
    // Find a node with clock annotation
    let has_clock = tree.root.iter_dfs().any(|node| node.comment.contains("%clk"));
    assert!(has_clock);
}

#[test]
fn test_lichess_move_count() {
    let tree = parse(LICHESS_GAME).unwrap();
    // 10 half-moves
    assert_eq!(tree.count_nodes(), 10);
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

#[test]
fn test_chesscom_game_parses() {
    let tree = parse(CHESSCOM_GAME).unwrap();
    assert_eq!(tree.header("Event"), Some("Live Chess"));
    assert_eq!(tree.header("Site"), Some("Chess.com"));
    assert_eq!(tree.header("Termination"), Some("Game drawn by agreement"));
}

#[test]
fn test_chesscom_fractional_clocks() {
    let tree = parse(CHESSCOM_GAME).unwrap();
    // Should handle fractional seconds in clocks
    let has_fractional = tree.root.iter_dfs().any(|node| node.comment.contains("."));
    assert!(has_fractional);
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

#[test]
fn test_annotated_game_parses() {
    let tree = parse(ANNOTATED_GAME).unwrap();
    assert_eq!(tree.header("Event"), Some("World Championship"));
    assert_eq!(tree.header("Annotator"), Some("GM John Doe"));
}

#[test]
fn test_annotated_game_has_comments() {
    let tree = parse(ANNOTATED_GAME).unwrap();
    assert!(tree.count_comments() > 0);
}

#[test]
fn test_annotated_game_has_nags() {
    let tree = parse(ANNOTATED_GAME).unwrap();
    // Should have NAGs from !, $1, ?!, $14, $5, $15
    let has_nags = tree.root.iter_dfs().any(|node| !node.nags.is_empty());
    assert!(has_nags);
}

#[test]
fn test_annotated_game_has_variations() {
    let tree = parse(ANNOTATED_GAME).unwrap();
    // Has a variation after 5... e5
    let has_variations = tree.root.iter_dfs().any(|node| node.has_variations());
    assert!(has_variations);
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

#[test]
fn test_lichess_study_parses() {
    let tree = parse(LICHESS_STUDY).unwrap();
    assert!(tree.header("Event").unwrap().contains("Opening Repertoire"));
    assert_eq!(tree.header("Chapter"), Some("Introduction to the Sicilian"));
}

#[test]
fn test_lichess_study_has_multiple_variations() {
    let tree = parse(LICHESS_STUDY).unwrap();
    // Count nodes with variations
    let variation_count = tree.root.iter_dfs().filter(|n| n.has_variations()).count();
    // Should have at least some variations
    assert!(variation_count >= 1);
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

#[test]
fn test_twic_game_parses() {
    let tree = parse(TWIC_GAME).unwrap();
    assert!(tree.header("Event").unwrap().contains("Tata Steel"));
    assert_eq!(tree.header("WhiteTitle"), Some("GM"));
    assert_eq!(tree.header("BlackTitle"), Some("GM"));
}

#[test]
fn test_twic_fide_ids_present() {
    let tree = parse(TWIC_GAME).unwrap();
    assert!(tree.header("WhiteFideId").is_some());
    assert!(tree.header("BlackFideId").is_some());
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

#[test]
fn test_stockfish_analysis_parses() {
    let tree = parse(STOCKFISH_ANALYSIS).unwrap();
    assert_eq!(tree.header("Annotator"), Some("Stockfish 16"));
}

#[test]
fn test_stockfish_evals_present() {
    let tree = parse(STOCKFISH_ANALYSIS).unwrap();
    let has_eval = tree.root.iter_dfs().any(|node| node.comment.contains("%eval"));
    assert!(has_eval);
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

#[test]
fn test_scid_game_parses() {
    let tree = parse(SCID_GAME).unwrap();
    assert!(tree.header("Event").unwrap().contains("World Championship"));
    assert_eq!(tree.header("PlyCount"), Some("81"));
}

#[test]
fn test_scid_extra_headers() {
    let tree = parse(SCID_GAME).unwrap();
    assert_eq!(tree.header("EventType"), Some("match"));
    assert_eq!(tree.header("EventCountry"), Some("ISL"));
    assert_eq!(tree.header("Source"), Some("ChessBase"));
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

#[test]
fn test_arena_game_parses() {
    let tree = parse(ARENA_GAME).unwrap();
    assert_eq!(tree.header("Termination"), Some("adjudication"));
    assert_eq!(tree.header("WhiteType"), Some("program"));
}

#[test]
fn test_arena_engine_comments() {
    let tree = parse(ARENA_GAME).unwrap();
    // Engine comments have depth/time info like "+0.25/25 0.5s"
    let has_engine_comment = tree.root.iter_dfs().any(|node| node.comment.contains("/"));
    assert!(has_engine_comment);
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

#[test]
fn test_chess960_game_parses() {
    let tree = parse(CHESS960_GAME).unwrap();
    assert_eq!(tree.header("Variant"), Some("Chess960"));
    assert_eq!(tree.header("SetUp"), Some("1"));
    assert!(tree.header("FEN").is_some());
}

// ============================================================================
// Edge Case: Very Long Game
// ============================================================================

#[test]
fn test_very_long_game() {
    // Create a game with 200 moves
    let mut pgn = r#"[Event "Test"]
[Result "*"]

"#.to_string();

    for i in 1..=200 {
        if i % 2 == 1 {
            pgn.push_str(&format!("{}. Nc3 ", (i + 1) / 2));
        } else {
            pgn.push_str("Nc6 ");
        }
    }
    pgn.push('*');

    let result = parse(&pgn);
    assert!(result.is_ok());
    let tree = result.unwrap();
    assert_eq!(tree.count_nodes(), 200);
}

// ============================================================================
// Edge Case: Deep Variations
// ============================================================================

#[test]
fn test_deeply_nested_variations() {
    let pgn = r#"[Event "Analysis"]
1. e4 c5 (1... e5 (1... d5 (1... Nf6 (1... g6 (1... b6))))) 2. Nf3 *"#;

    let tree = parse(pgn).unwrap();
    // Should parse successfully with nested variations
    // The structure is complex but should not panic
    assert!(tree.count_nodes() >= 5);
}

// ============================================================================
// Edge Case: Many Sibling Variations
// ============================================================================

#[test]
fn test_many_sibling_variations() {
    let pgn = r#"[Event "Analysis"]
1. e4 (1. d4) (1. c4) (1. Nf3) (1. g3) (1. b3) (1. f4) (1. e3) (1. d3) c5 *"#;

    let tree = parse(pgn).unwrap();
    // Root's first child (e4) should have many siblings
    assert!(tree.root.has_children());
}

// ============================================================================
// Roundtrip Tests with Real Games
// ============================================================================

#[test]
fn test_roundtrip_lichess_game() {
    use pgnq::serializer::{to_pgn, OutputFormat, OutputOptions};

    let tree1 = parse(LICHESS_GAME).unwrap();
    let output = to_pgn(&tree1, &OutputOptions::default());
    let tree2 = parse(&output).unwrap();

    // Move counts should match
    assert_eq!(tree1.count_nodes(), tree2.count_nodes());
}

#[test]
fn test_roundtrip_annotated_game() {
    use pgnq::serializer::{to_pgn, OutputOptions};

    let tree1 = parse(ANNOTATED_GAME).unwrap();
    let output = to_pgn(&tree1, &OutputOptions::default());
    let tree2 = parse(&output).unwrap();

    // Move counts should match
    assert_eq!(tree1.count_nodes(), tree2.count_nodes());
}
