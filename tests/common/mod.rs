//! Common test fixtures and helpers for pgnq integration tests

#![allow(dead_code)]

// ============================================================================
// SIMPLE PGN FIXTURES
// ============================================================================

/// Minimal valid PGN with just moves
pub const MINIMAL_GAME: &str = "1. e4 e5 2. Nf3 Nc6 1-0";

/// Game with complete Seven Tag Roster
pub const FULL_HEADERS_GAME: &str = r#"[Event "Test Tournament"]
[Site "Test City"]
[Date "2024.01.15"]
[Round "1"]
[White "Player, White"]
[Black "Player, Black"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0"#;

/// Headerless game (no tags at all)
pub const HEADERLESS_GAME: &str = "1. d4 d5 2. c4 e6 3. Nc3 Nf6 *";

/// Longer game (20+ moves)
pub const LONG_GAME: &str = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7 11. Nbd2 Bb7 12. Bc2 Re8 13. Nf1 Bf8 14. Ng3 g6 15. Bg5 h6 16. Bd2 Bg7 17. a4 c5 18. d5 c4 19. b4 cxb3 20. Bxb3 *";

/// Complete Seven Tag Roster (alias for tests)
pub const FULL_HEADERS: &str = FULL_HEADERS_GAME;

// ============================================================================
// MOVE NOTATION EDGE CASES
// ============================================================================

/// Castling moves (letter O, not zero)
pub const CASTLING_GAME: &str = r#"[Event "Castling Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O Nf6 5. d3 O-O 6. c3 d6 *"#;

/// Queenside castling
pub const QUEENSIDE_CASTLING: &str = "1. d4 d5 2. c4 e6 3. Nc3 Nf6 4. Bg5 Be7 5. e3 O-O 6. Nf3 Nbd7 7. Qc2 c6 8. O-O-O *";

/// Castling with check
pub const CASTLING_WITH_CHECK: &str = "1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Nd6 6. Bxc6 dxc6 7. dxe5 Nf5 8. Qxd8+ Kxd8 *";

/// Castling using zero notation (0-0) instead of letter O
pub const CASTLING_ZERO_NOTATION: &str = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. 0-0 Nf6 5. d3 0-0 *";

/// Piece disambiguation (file)
pub const DISAMBIGUATION_FILE: &str = "1. e4 e5 2. Nf3 Nc6 3. d4 exd4 4. Nxd4 Nf6 5. Nc3 Bb4 6. Nxc6 bxc6 7. Bd3 d5 8. exd5 cxd5 9. O-O O-O 10. Bg5 c6 11. Qf3 Be7 12. Rae1 *";

/// Piece disambiguation (rank)
pub const DISAMBIGUATION_RANK: &str = "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. Be2 e5 7. Nb3 Be7 8. O-O O-O 9. Be3 Be6 10. Nd5 Nxd5 11. exd5 Bf5 12. c4 Nd7 13. Rc1 Rc8 14. Qd2 f6 15. Rfd1 Bg6 16. Na5 b5 17. Nc6 Qb6 18. Nxe7+ Kh8 19. Bf3 R8c7 20. Nc6 R7xc6 *";

/// Disambiguation game (alias for roundtrip tests)
pub const DISAMBIGUATION_GAME: &str = DISAMBIGUATION_FILE;

/// Pawn promotion
pub const PAWN_PROMOTION: &str = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 c6 5. Nf3 Nf6 6. Bc4 Bf5 7. Bd2 e6 8. Qe2 Bb4 9. O-O-O Nbd7 10. Rhe1 O-O 11. a3 Bxc3 12. Bxc3 Qc7 13. Kb1 b5 14. Bd3 Bxd3 15. Qxd3 a5 16. Ne5 Nxe5 17. Rxe5 Nd7 18. Re2 a4 19. Qe3 Qa5 20. d5 exd5 21. Rxd5 Qa6 22. Bd4 Qe6 23. Qg5 Qg6 24. Qxg6 hxg6 25. Rd6 Ne5 26. Bxe5 Rfe8 27. Bd4 Re4 28. Bc3 Rae8 29. Rxc6 Re1+ 30. Bxe1 Rxe1+ 31. Ka2 Re8 32. Rc7 Kf8 33. Ra7 Re4 34. h3 Rc4 35. Kb1 Rxc2 36. Ra8+ Ke7 37. Ra7+ Kf6 38. Rxc2 b4 39. axb4 a3 40. bxa3 Ke5 41. Rc5+ Kd4 42. a4 f5 43. a5 g5 44. a6 f4 45. a7 f3 46. a8=Q *";

/// Pawn promotion to knight (underpromotion)
pub const UNDERPROMOTION: &str = "1. e4 e5 2. f4 exf4 3. Nf3 g5 4. h4 g4 5. Ne5 Nf6 6. d4 d6 7. Nd3 Nxe4 8. Bxf4 Qe7 9. Be2 Nc6 10. c3 Bf5 11. Qc2 O-O-O 12. O-O Bxd3 13. Bxd3 Nf6 14. b4 h5 15. a4 Bg7 16. b5 Ne5 17. dxe5 dxe5 18. Be3 Nd5 19. Bd2 Qd6 20. Bf5+ Kb8 21. Be4 Nc7 22. Rf5 Rhf8 23. Rxf8 Rxf8 24. Bf3 Qg6 25. a5 Qb1+ 26. Qxb1 f6 27. Qb4 Kc8 28. Qc5 Bf8 29. Ra4 Bxc5+ 30. Bxc5 Rd8 31. Bxc7 Kxc7 32. Rxg4 Rd1+ 33. Kf2 Rd2+ 34. Kf1 Kb8 35. Bc6 Rxg2 36. b6 axb6 37. axb6 c5 38. Bb5 f5 39. Rg8+ Kc8 40. Ba6 bxa6 41. Rxc8+ Kxc8 42. b7+ Kxb7 *";

/// Promotion game (alias for roundtrip tests)
pub const PROMOTION_GAME: &str = PAWN_PROMOTION;

/// Capture with pawn
pub const PAWN_CAPTURES: &str = "1. e4 d5 2. exd5 Qxd5 3. Nc3 Qa5 4. d4 Nf6 5. Nf3 Bf5 6. Bd3 Bxd3 7. Qxd3 c6 8. O-O e6 9. Re1 Be7 10. Bd2 Qc7 11. a3 Nbd7 12. b4 O-O 13. Ne5 Nxe5 14. dxe5 Nd5 15. Nxd5 cxd5 16. Qg3 Rac8 17. Rec1 Qc4 *";

// ============================================================================
// COMMENT STYLES
// ============================================================================

/// Standard brace comments
pub const BRACE_COMMENTS: &str = r#"[Event "Comment Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 {The King's Pawn opening} e5 {Symmetrical response} 2. Nf3 {Attacking the e5 pawn} Nc6 {Defending} *"#;

/// Semicolon comments (rest of line)
pub const SEMICOLON_COMMENTS: &str = r#"[Event "Semicolon Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 e5 ; Open game
2. Nf3 Nc6 ; Knight development
3. Bb5 *"#;

/// Multiline brace comment
pub const MULTILINE_COMMENT: &str = r#"[Event "Multiline Test"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 {This is a longer comment
that spans multiple lines
and contains various information} e5 *"#;

/// Empty comment
pub const EMPTY_COMMENT: &str = "1. e4 {} e5 *";

/// Comment with special characters
pub const SPECIAL_CHARS_COMMENT: &str = "1. e4 {Special chars: <>!@#$%^&*()_+-=[]|;':\",./<>?} e5 *";

/// Comment with unicode characters
pub const UNICODE_COMMENT: &str = "1. e4 {Müller plays the King's pawn: ♔ → ♙e4} e5 {Карлсен responds symmetrically} *";

// ============================================================================
// NAG (NUMERIC ANNOTATION GLYPHS)
// ============================================================================

/// Symbolic NAGs
pub const SYMBOLIC_NAGS: &str = "1. e4! e5? 2. Nf3!! Nc6?? 3. Bb5!? a6?! *";

/// Numeric NAGs
pub const NUMERIC_NAGS: &str = "1. e4 $1 e5 $2 2. Nf3 $3 Nc6 $4 3. Bb5 $5 a6 $6 *";

/// Positional NAGs
pub const POSITIONAL_NAGS: &str = "1. e4 $14 e5 $15 2. Nf3 $16 Nc6 $17 3. Bb5 $18 a6 $19 *";

/// Multiple NAGs on same move
pub const MULTIPLE_NAGS: &str = "1. e4! $14 e5 $2 $17 2. Nf3 *";

// ============================================================================
// VARIATIONS (RAV)
// ============================================================================

/// Simple single variation
pub const SINGLE_VARIATION: &str = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3 Nc6 *";

/// Multiple sibling variations at same point
pub const SIBLING_VARIATIONS: &str = "1. e4 e5 (1... c5 2. Nf3) (1... e6 2. d4) (1... d5 2. exd5) 2. Nf3 *";

/// Deeply nested variations (3+ levels)
pub const NESTED_VARIATIONS: &str = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 (3. Bb5 g6)) 3. d4) 2. Nf3 Nc6 *"#;

/// Very deeply nested variations (4+ levels for stress testing)
pub const DEEPLY_NESTED: &str = r#"1. e4 e5 (1... c5 2. Nf3 d6 (2... Nc6 3. d4 cxd4 (3... e6 4. Nxc6 (4. d5 Ne5) bxc6 5. e5) 4. Nxd4) 3. d4 cxd4 4. Nxd4) 2. Nf3 Nc6 (2... Nf6 3. Nxe5 d6 (3... Nxe4 4. Qe2 Qe7) 4. Nf3) 3. Bb5 *"#;

/// Variation with comments and NAGs
pub const VARIATION_WITH_ANNOTATIONS: &str = "1. e4 e5 (1... c5 {Sicilian Defense} 2. Nf3! d6) 2. Nf3 *";

/// Variation immediately after opening move
pub const EARLY_VARIATION: &str = "1. e4 (1. d4 d5 2. c4) e5 2. Nf3 *";

// ============================================================================
// CLOCK AND EVAL ANNOTATIONS
// ============================================================================

/// Lichess clock format
pub const LICHESS_CLOCKS: &str = r#"[Event "Rated Blitz"]
[Site "https://lichess.org"]
[Date "2024.01.15"]
[Round "?"]
[White "Player1"]
[Black "Player2"]
[Result "1-0"]
[TimeControl "180+0"]

1. e4 {[%clk 0:03:00]} e5 {[%clk 0:03:00]} 2. Nf3 {[%clk 0:02:58]} Nc6 {[%clk 0:02:59]} 1-0"#;

/// Evaluation annotations
pub const EVAL_ANNOTATIONS: &str = r#"1. e4 {[%eval 0.25]} e5 {[%eval 0.20]} 2. Nf3 {[%eval 0.35]} Nc6 {[%eval 0.30]} *"#;

/// Mate evaluation
pub const MATE_EVAL: &str = r#"1. f3 e5 2. g4 {[%eval #-1]} Qh4# {[%eval #0]} 0-1"#;

/// Combined clock and eval
pub const CLOCK_AND_EVAL: &str = r#"1. e4 {[%clk 0:03:00] [%eval 0.25]} e5 {[%clk 0:03:00] [%eval 0.20]} *"#;

/// Elapsed move time (EMT)
pub const EMT_ANNOTATIONS: &str = r#"1. e4 {[%emt 0:00:05]} e5 {[%emt 0:00:03]} 2. Nf3 {[%emt 0:00:02]} *"#;

// ============================================================================
// GAME TERMINATION
// ============================================================================

/// White wins
pub const WHITE_WINS: &str = r#"[Result "1-0"]

1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0"#;

/// Black wins
pub const BLACK_WINS: &str = r#"[Result "0-1"]

1. f3 e5 2. g4 Qh4# 0-1"#;

/// Draw
pub const DRAW_GAME: &str = r#"[Result "1/2-1/2"]

1. e4 e5 2. Nf3 Nf6 3. Nxe5 d6 4. Nf3 Nxe4 1/2-1/2"#;

/// Ongoing/unknown result
pub const ONGOING_GAME: &str = r#"[Result "*"]

1. e4 e5 2. Nf3 Nc6 *"#;

// ============================================================================
// MULTI-GAME FILES
// ============================================================================

/// Two games in one file
pub const TWO_GAMES: &str = r#"[Event "Game 1"]
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

/// Three games with varying completeness
pub const THREE_GAMES: &str = r#"[Event "Complete Game"]
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

// ============================================================================
// HEADER EDGE CASES
// ============================================================================

/// Unicode in player names
pub const UNICODE_HEADERS: &str = r#"[Event "International"]
[Site "München, Germany"]
[Date "2024.01.15"]
[Round "1"]
[White "Müller, Hans"]
[Black "Карлсен, Магнус"]
[Result "*"]

1. e4 e5 *"#;

/// Very long header value
pub const LONG_HEADER: &str = r#"[Event "This is an extremely long event name that goes on and on to test how the parser handles very long header values in PGN files"]
[Site "?"]
[Date "????.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 *"#;

/// Custom/non-standard tags
pub const CUSTOM_TAGS: &str = r#"[Event "Test"]
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

/// Unknown date parts
pub const PARTIAL_DATE: &str = r#"[Event "?"]
[Site "?"]
[Date "2024.??.??"]
[Round "?"]
[White "?"]
[Black "?"]
[Result "*"]

1. e4 *"#;

// ============================================================================
// REAL-WORLD EXAMPLES
// ============================================================================

/// Lichess-style game export
pub const LICHESS_EXPORT: &str = r#"[Event "Rated Blitz game"]
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

/// Annotated game with deep analysis
pub const ANNOTATED_GAME: &str = r#"[Event "World Championship"]
[Site "Reykjavik ISL"]
[Date "1972.07.23"]
[Round "6"]
[White "Fischer, Robert J."]
[Black "Spassky, Boris V."]
[Result "1-0"]
[ECO "D59"]

1. c4 {Fischer avoids 1.e4 for the first time in the match} e6 2. Nf3 d5 3. d4 Nf6 4. Nc3 Be7 5. Bg5 O-O 6. e3 h6 7. Bh4 b6 {The Tartakower Defense} 8. cxd5 Nxd5 9. Bxe7 Qxe7 10. Nxd5 exd5 11. Rc1 Be6 12. Qa4 c5 13. Qa3 Rc8 14. Bb5! {An excellent move, putting pressure on the queenside} (14. Be2 {was also possible} cxd4 15. Nxd4 Qb4) 14... a6 15. dxc5 bxc5 16. O-O Ra7 17. Be2 Nd7 18. Nd4! {A strong knight maneuver} Qf8 (18... Nf6 19. Nxe6 fxe6 20. Bg4 $14) 19. Nxe6 fxe6 20. e4! $1 {Opening up the position with Black's king exposed} d4 21. f4 Qe7 22. e5 Rb8 23. Bc4 Kh8 24. Qh3 Nf8 25. b3 a5 26. f5! exf5 27. Rxf5 Nh7 28. Rcf1 {White has a crushing attack} Qd8 29. Qg3 Re7 30. h4 Rbb7 31. e6! Rbc7 32. Qe5 Qe8 33. a4 Qd8 34. R1f2 Qe8 35. R2f3 Qd8 36. Bd3 Qe8 37. Qe4 Nf6 38. Rxf6! gxf6 39. Rxf6 Kg8 40. Bc4 Kh8 41. Qf4 1-0"#;

/// Lichess study chapter with variations
pub const LICHESS_STUDY: &str = r#"[Event "Opening Repertoire: Sicilian"]
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

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

use pgnq::parser::parse;
use pgnq::tree::GameTree;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Parse a PGN string and return the game tree, panicking on failure
pub fn parse_pgn(pgn: &str) -> GameTree {
    parse(pgn).expect("Failed to parse PGN")
}

/// Parse a PGN string and return Result
pub fn try_parse_pgn(pgn: &str) -> Result<GameTree, pgnq::error::Error> {
    parse(pgn)
}

/// Count total move nodes in a game tree (excluding root)
pub fn count_nodes(tree: &GameTree) -> usize {
    tree.root.count_nodes().saturating_sub(1)
}

/// Get the main line as a vector of SAN moves (excluding root)
pub fn main_line_moves(tree: &GameTree) -> Vec<String> {
    tree.root
        .iter_main_line()
        .skip(1) // Skip root node
        .map(|node| node.san.clone())
        .collect()
}

// ============================================================================
// FLUENT CLI TESTING API
// ============================================================================

/// Result from running a pgnq command with fluent assertion methods
#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CommandResult {
    /// Assert the command succeeded (exit code 0)
    pub fn success(&self) -> &Self {
        assert_eq!(
            self.exit_code, 0,
            "Expected success, got exit code {}.\nStderr: {}",
            self.exit_code, self.stderr
        );
        self
    }

    /// Assert the command failed (exit code != 0)
    pub fn failure(&self) -> &Self {
        assert_ne!(
            self.exit_code, 0,
            "Expected failure, got success.\nStdout: {}",
            self.stdout
        );
        self
    }

    /// Assert stdout contains a string
    pub fn stdout_contains(&self, text: &str) -> &Self {
        assert!(
            self.stdout.contains(text),
            "Expected stdout to contain {:?}\nActual stdout:\n{}",
            text,
            self.stdout
        );
        self
    }

    /// Assert stdout does NOT contain a string
    pub fn stdout_not_contains(&self, text: &str) -> &Self {
        assert!(
            !self.stdout.contains(text),
            "Expected stdout to NOT contain {:?}\nActual stdout:\n{}",
            text,
            self.stdout
        );
        self
    }

    /// Assert stderr contains a string
    pub fn stderr_contains(&self, text: &str) -> &Self {
        assert!(
            self.stderr.contains(text),
            "Expected stderr to contain {:?}\nActual stderr:\n{}",
            text,
            self.stderr
        );
        self
    }

    /// Get the PGN output for further assertions
    pub fn pgn(&self) -> PgnOutput<'_> {
        PgnOutput::new(&self.stdout)
    }
}

/// Builder for running pgnq commands with fluent API
pub struct PgnqCommand {
    args: Vec<String>,
    stdin: Option<String>,
}

impl PgnqCommand {
    /// Start building a new pgnq command
    pub fn new(subcommand: &str) -> Self {
        Self {
            args: vec![subcommand.to_string()],
            stdin: None,
        }
    }

    /// Add an argument
    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add multiple arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    /// Set stdin input (PGN content) - automatically adds "-" to args
    pub fn stdin(mut self, pgn: &str) -> Self {
        self.stdin = Some(pgn.to_string());
        self.args.push("-".to_string());
        self
    }

    /// Run the command and return the result
    pub fn run(self) -> CommandResult {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--quiet", "--"]);
        cmd.args(&self.args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn pgnq");

        if let Some(input) = &self.stdin {
            let stdin_handle = child.stdin.as_mut().expect("Failed to open stdin");
            stdin_handle
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait for pgnq");

        CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

/// Wrapper for PGN output with assertion helpers
pub struct PgnOutput<'a> {
    content: &'a str,
}

impl<'a> PgnOutput<'a> {
    pub fn new(content: &'a str) -> Self {
        Self { content }
    }

    /// Assert the PGN contains a specific move
    pub fn has_move(&self, san: &str) -> &Self {
        assert!(
            self.content.contains(san),
            "Expected PGN to contain move {:?}\nActual PGN:\n{}",
            san,
            self.content
        );
        self
    }

    /// Assert the PGN does NOT contain a specific move
    pub fn not_has_move(&self, san: &str) -> &Self {
        assert!(
            !self.content.contains(san),
            "Expected PGN to NOT contain move {:?}\nActual PGN:\n{}",
            san,
            self.content
        );
        self
    }

    /// Assert the PGN contains a variation (has parentheses)
    pub fn has_variation(&self) -> &Self {
        assert!(
            self.content.contains('('),
            "Expected PGN to contain variations\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN has no variations
    pub fn no_variations(&self) -> &Self {
        assert!(
            !self.content.contains('('),
            "Expected PGN to NOT contain variations\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN contains a comment with specific text
    pub fn has_comment(&self, text: &str) -> &Self {
        assert!(
            self.content.contains(text) && self.content.contains('{'),
            "Expected PGN to contain comment {:?}\nActual PGN:\n{}",
            text,
            self.content
        );
        self
    }

    /// Assert the PGN has no comments (no braces)
    pub fn no_comments(&self) -> &Self {
        assert!(
            !self.content.contains('{'),
            "Expected PGN to NOT contain comments\nActual PGN:\n{}",
            self.content
        );
        self
    }

    /// Assert the PGN contains a specific NAG
    pub fn has_nag(&self, nag: &str) -> &Self {
        assert!(
            self.content.contains(nag),
            "Expected PGN to contain NAG {:?}\nActual PGN:\n{}",
            nag,
            self.content
        );
        self
    }

    /// Assert the PGN has a specific header value
    pub fn has_header(&self, key: &str, value: &str) -> &Self {
        let pattern = format!("[{} \"{}\"]", key, value);
        assert!(
            self.content.contains(&pattern),
            "Expected PGN to contain header [{} {:?}]\nActual PGN:\n{}",
            key,
            value,
            self.content
        );
        self
    }

    /// Assert the PGN result
    pub fn has_result(&self, result: &str) -> &Self {
        assert!(
            self.content.contains(result),
            "Expected PGN to contain result {:?}\nActual PGN:\n{}",
            result,
            self.content
        );
        self
    }

    /// Assert the main line contains all specified moves
    pub fn main_line_has(&self, moves: &[&str]) -> &Self {
        for m in moves {
            assert!(
                self.content.contains(m),
                "Expected main line to contain {:?}\nActual PGN:\n{}",
                m,
                self.content
            );
        }
        self
    }

    /// Get raw content for custom assertions
    pub fn raw(&self) -> &str {
        self.content
    }
}

/// Convenience function to start building a pgnq command
pub fn pgnq(subcommand: &str) -> PgnqCommand {
    PgnqCommand::new(subcommand)
}

/// Create a temp file with PGN content
pub fn pgn_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write PGN");
    file.flush().expect("Failed to flush");
    file
}
