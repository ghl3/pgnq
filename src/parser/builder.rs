//! Tree builder - constructs GameTree from token stream

use super::lexer::LocatedToken;
use super::token::Token;
use crate::error::{ErrorCode, ParseError, ParseMode, Result};
use crate::nag::Nag;
use crate::tree::{GameNode, GameResult, GameTree};
use std::path::PathBuf;

/// A path from the root to a specific node in the tree.
/// Each element is a child index at that level.
/// Empty path means root node.
type NodePath = Vec<usize>;

/// Parse context state machine for distinguishing real moves from prose references.
///
/// This handles Lichess-style baretext comments where move-like text (e.g., "f3")
/// can appear in prose and should not be parsed as actual chess moves.
#[derive(Debug, Clone)]
enum ParseContext {
    /// After a MoveNumber, expecting up to N moves.
    /// - After "1." expect 2 moves (White then Black)
    /// - After "1..." expect 1 move (Black only)
    ExpectingMoves { remaining: u8 },

    /// Between moves - not currently in prose.
    /// Moves here are real (Lichess puts continuation moves on their own lines).
    BetweenMoves,

    /// Inside prose text - move tokens should be treated as comment text.
    /// Entered when BareText is seen, exited on Newline.
    InProse,
}

/// Tracks the location where a variation started (for error reporting)
#[derive(Debug, Clone)]
struct VariationStart {
    line: usize,
    column: usize,
}

/// Mutable state used during tree building.
/// Encapsulates all the state that gets passed between token handlers.
struct BuilderState {
    /// Path from root to current position (indices at each level)
    current_path: NodePath,
    /// Stack for returning after variations end
    return_stack: Vec<(NodePath, u16, bool, ParseContext)>,
    /// Stack tracking where variations started (for error reporting)
    variation_starts: Vec<VariationStart>,
    /// Comment text waiting to be attached to next move
    pending_comment: String,
    /// NAGs waiting to be attached to next move
    pending_nags: Vec<Nag>,
    /// Current move number being parsed
    current_move_number: u16,
    /// Whether we expect the next move to be Black's
    expect_black: bool,
    /// Context for prose detection state machine
    context: ParseContext,
    /// Parsing mode (strict or lenient)
    mode: ParseMode,
    /// Source file path (for error messages)
    file: Option<PathBuf>,
    /// Original source text (for error context)
    source: Option<String>,
}

impl BuilderState {
    fn new(mode: ParseMode) -> Self {
        Self {
            current_path: Vec::new(),
            return_stack: Vec::new(),
            variation_starts: Vec::new(),
            pending_comment: String::new(),
            pending_nags: Vec::new(),
            current_move_number: 1,
            expect_black: false,
            context: ParseContext::BetweenMoves,
            mode,
            file: None,
            source: None,
        }
    }

    fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Create a parse error with source context
    fn make_error(
        &self,
        code: ErrorCode,
        message: &str,
        token: &LocatedToken,
    ) -> ParseError {
        let mut err = ParseError::new(code, message)
            .with_location(token.line, token.column)
            .with_span(token.len);

        if let Some(ref file) = self.file {
            err = err.with_file(file.clone());
        }

        if let Some(ref source) = self.source {
            let context = ParseError::create_context_from_source(
                source,
                token.line,
                token.column,
                token.len,
                message,
            );
            err = err.with_source_context(context);
        }

        err
    }

    /// Append text to the appropriate comment location (pending or current node)
    fn append_comment(&mut self, root: &mut GameNode, text: &str) {
        if self.current_path.is_empty() {
            if !self.pending_comment.is_empty() {
                self.pending_comment.push(' ');
            }
            self.pending_comment.push_str(text);
        } else {
            let current = root.navigate_path_mut(&self.current_path);
            if !current.comment.is_empty() {
                current.comment.push(' ');
            }
            current.comment.push_str(text);
        }
    }

    /// Add a NAG to the appropriate location (pending or current node)
    fn add_nag(&mut self, root: &mut GameNode, nag: Nag) {
        if self.current_path.is_empty() {
            self.pending_nags.push(nag);
        } else {
            let current = root.navigate_path_mut(&self.current_path);
            current.nags.push(nag);
        }
    }

    /// Handle a header token
    fn handle_header(&self, tree: &mut GameTree, header_str: &str) {
        if let Some((key, value)) = parse_header(header_str) {
            if key == "Result" {
                if let Some(result) = GameResult::parse(&value) {
                    tree.result = result;
                }
            }
            tree.headers.insert(key, value);
        }
    }

    /// Handle a move number token
    fn handle_move_number(&mut self, root: &mut GameNode, num_str: &str) {
        if matches!(self.context, ParseContext::InProse) {
            // In prose context, treat move numbers as comment text
            self.append_comment(root, num_str);
        } else if let Some(num) = parse_move_number(num_str) {
            self.current_move_number = num;
            self.expect_black = num_str.contains("...");
            // After a move number, we expect real moves
            self.context = ParseContext::ExpectingMoves {
                remaining: if self.expect_black { 1 } else { 2 },
            };
        }
    }

    /// Handle a chess move token (piece move, pawn move, or castling)
    fn handle_move(&mut self, root: &mut GameNode, san: &str) {
        if matches!(self.context, ParseContext::InProse) {
            // In prose context, treat as comment text
            self.append_comment(root, san);
        } else {
            // Real move - add to tree
            let mut node = GameNode::new(san);
            node.move_number = Some(self.current_move_number);
            node.is_black = self.expect_black;

            if !self.pending_comment.is_empty() {
                node.comment = std::mem::take(&mut self.pending_comment);
            }
            node.nags = std::mem::take(&mut self.pending_nags);

            // Navigate to parent and add child
            let parent = root.navigate_path_mut(&self.current_path);
            parent.children.push(node);
            let new_child_idx = parent.children.len() - 1;
            self.current_path.push(new_child_idx);

            // Update move tracking
            if self.expect_black {
                self.current_move_number += 1;
                self.expect_black = false;
            } else {
                self.expect_black = true;
            }

            // Update context after consuming a move
            if let ParseContext::ExpectingMoves { remaining } = self.context {
                self.context = if remaining > 1 {
                    ParseContext::ExpectingMoves {
                        remaining: remaining - 1,
                    }
                } else {
                    ParseContext::BetweenMoves
                };
            }
        }
    }

    /// Handle a comment token (brace or semicolon style)
    fn handle_comment(&mut self, root: &mut GameNode, text: &str) {
        let text = text.trim();
        if !text.is_empty() {
            self.append_comment(root, text);
        }
    }

    /// Handle bare text (Lichess-style inline comment)
    fn handle_bare_text(&mut self, root: &mut GameNode, text: &str) {
        let text = text.trim();
        if !text.is_empty() {
            self.append_comment(root, text);
            // Enter prose context - subsequent move tokens are references
            self.context = ParseContext::InProse;
        }
    }

    /// Handle start of a variation
    fn handle_variation_start(&mut self, token: &LocatedToken) {
        // Track where this variation started for error reporting
        self.variation_starts.push(VariationStart {
            line: token.line,
            column: token.column,
        });

        // Save current state to restore after variation ends
        self.return_stack.push((
            self.current_path.clone(),
            self.current_move_number,
            self.expect_black,
            self.context.clone(),
        ));

        if !self.current_path.is_empty() {
            // Go back to parent - variation moves are siblings of current move
            self.current_path.pop();
        }
        // Root-level variations create alternative first moves

        // Variations start fresh
        self.context = ParseContext::BetweenMoves;
    }

    /// Handle end of a variation
    fn handle_variation_end(&mut self, _token: &LocatedToken) -> std::result::Result<(), ParseError> {
        // Pop variation start tracking
        self.variation_starts.pop();

        // Restore saved state. Extra closing parens are silently ignored
        // (follows "be liberal with what you accept" principle)
        if let Some((saved_path, saved_move_num, saved_expect_black, saved_context)) =
            self.return_stack.pop()
        {
            self.current_path = saved_path;
            self.current_move_number = saved_move_num;
            self.expect_black = saved_expect_black;
            self.context = saved_context;
        }
        // Note: In lenient mode, we silently ignore unmatched closing parens
        // In strict mode, this should be an error, but we're being liberal here
        Ok(())
    }

    /// Handle newline (exits prose context)
    fn handle_newline(&mut self) {
        if matches!(self.context, ParseContext::InProse) {
            self.context = ParseContext::BetweenMoves;
        }
    }

    /// Check for unclosed variations at end of parsing
    fn check_unclosed_variations(&self) -> std::result::Result<(), ParseError> {
        if let Some(start) = self.variation_starts.first() {
            let mut err = ParseError::new(ErrorCode::UnclosedVariation, "unclosed variation")
                .with_location(start.line, start.column)
                .with_note(format!(
                    "Variation was opened at line {}, column {}",
                    start.line, start.column
                ))
                .with_help("Add ')' to close the variation, or remove the opening '('");

            if let Some(ref file) = self.file {
                err = err.with_file(file.clone());
            }

            if let Some(ref source) = self.source {
                let context = ParseError::create_context_from_source(
                    source,
                    start.line,
                    start.column,
                    1,
                    "variation starts here",
                );
                err = err.with_source_context(context);
            }

            return Err(err);
        }
        Ok(())
    }
}

/// Build a GameTree from a token stream
pub fn build_tree(tokens: &[LocatedToken]) -> Result<GameTree> {
    build_tree_with_options(tokens, ParseMode::Lenient, None, None)
}

/// Build a GameTree with specific options
pub fn build_tree_with_options(
    tokens: &[LocatedToken],
    mode: ParseMode,
    file: Option<PathBuf>,
    source: Option<&str>,
) -> Result<GameTree> {
    let mut tree = GameTree::new();
    let mut state = BuilderState::new(mode);

    if let Some(f) = file {
        state = state.with_file(f);
    }
    if let Some(s) = source {
        state = state.with_source(s);
    }

    for token in tokens {
        match &token.token {
            Token::Header(header_str) => {
                state.handle_header(&mut tree, header_str);
            }

            Token::MoveNumber(num_str) => {
                state.handle_move_number(&mut tree.root, num_str);
            }

            Token::PieceMove(san)
            | Token::PawnMove(san)
            | Token::CastleLong(san)
            | Token::CastleShort(san) => {
                state.handle_move(&mut tree.root, san);
            }

            Token::BraceComment(text) | Token::SemicolonComment(text) => {
                state.handle_comment(&mut tree.root, text);
            }

            Token::BareText(text) => {
                state.handle_bare_text(&mut tree.root, text);
            }

            Token::Nag(nag_str) => {
                if let Some(nag) = Nag::from_dollar_notation(nag_str) {
                    state.add_nag(&mut tree.root, nag);
                }
            }

            // Symbolic NAGs (!, ?, !!, ??, !?, ?!)
            Token::DoubleBang(_)
            | Token::DoubleQuestion(_)
            | Token::BangQuestion(_)
            | Token::QuestionBang(_)
            | Token::Bang(_)
            | Token::Question(_) => {
                if let Some(nag_val) = token.token.as_nag_value() {
                    state.add_nag(&mut tree.root, Nag::new(nag_val));
                }
            }

            // Positional assessment NAGs (=, +=, =+, +/-, -/+, +-, -+)
            Token::Equal
            | Token::WhiteWinning
            | Token::BlackWinning
            | Token::WhiteBetter
            | Token::BlackBetter
            | Token::WhiteSlightlyBetter
            | Token::BlackSlightlyBetter => {
                if let Some(nag_val) = token.token.as_nag_value() {
                    state.add_nag(&mut tree.root, Nag::new(nag_val));
                }
            }

            Token::VariationStart => {
                state.handle_variation_start(token);
            }

            Token::VariationEnd => {
                state.handle_variation_end(token)?;
            }

            Token::WhiteWins => tree.result = GameResult::WhiteWins,
            Token::BlackWins => tree.result = GameResult::BlackWins,
            Token::Draw => tree.result = GameResult::Draw,
            Token::Ongoing => tree.result = GameResult::Ongoing,

            Token::Newline => {
                state.handle_newline();
            }
        }
    }

    // Check for unclosed variations
    state.check_unclosed_variations()?;

    Ok(tree)
}

/// Parse a header string like [Event "Test Game"] into (key, value)
/// Handles escaped quotes within values (e.g., [Key "value with \"quotes\""])
fn parse_header(header: &str) -> Option<(String, String)> {
    let header = header.trim();
    if !header.starts_with('[') || !header.ends_with(']') {
        return None;
    }

    let inner = &header[1..header.len() - 1];

    // Find the opening quote
    let quote_start = inner.find('"')?;
    let key = inner[..quote_start].trim().to_string();

    if key.is_empty() {
        return None;
    }

    // Parse the value, handling escaped quotes
    let value_part = &inner[quote_start + 1..];
    let mut value = String::new();
    let mut chars = value_part.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Handle escape sequences
                if let Some(&next) = chars.peek() {
                    match next {
                        '"' | '\\' => {
                            value.push(chars.next().unwrap());
                        }
                        _ => {
                            // Unknown escape, keep the backslash
                            value.push('\\');
                        }
                    }
                } else {
                    value.push('\\');
                }
            }
            '"' => {
                // End of value - closing quote found
                return Some((key, value));
            }
            _ => {
                value.push(c);
            }
        }
    }

    // No closing quote found - try to be lenient and return what we have
    if !value.is_empty() {
        Some((key, value))
    } else {
        None
    }
}

/// Parse a move number string like "1." or "15..." into the number
fn parse_move_number(s: &str) -> Option<u16> {
    let num_part: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_part.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    #[test]
    fn test_parse_header() {
        let (key, value) = parse_header(r#"[Event "Test Game"]"#).unwrap();
        assert_eq!(key, "Event");
        assert_eq!(value, "Test Game");
    }

    #[test]
    fn test_parse_move_number() {
        assert_eq!(parse_move_number("1."), Some(1));
        assert_eq!(parse_move_number("15..."), Some(15));
        assert_eq!(parse_move_number("42."), Some(42));
    }

    #[test]
    fn test_build_simple_game() {
        let tokens = tokenize("1. e4 e5 2. Nf3 Nc6");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.move_number, Some(1));
        assert!(!e4.is_black);

        let e5 = e4.find_child("e5").unwrap();
        assert_eq!(e5.move_number, Some(1));
        assert!(e5.is_black);
    }

    #[test]
    fn test_build_with_headers() {
        let tokens = tokenize(r#"[Event "Test"] [White "Alice"] 1. e4"#);
        let tree = build_tree(&tokens).unwrap();

        assert_eq!(tree.header("Event"), Some("Test"));
        assert_eq!(tree.header("White"), Some("Alice"));
    }

    #[test]
    fn test_build_with_comments() {
        let tokens = tokenize("1. e4 {Best move!} e5");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.comment.trim(), "Best move!");
    }

    #[test]
    fn test_build_with_nags() {
        let tokens = tokenize("1. e4! e5?? 2. Nf3 $14");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert!(e4.nags.contains(&Nag::GOOD_MOVE));

        let e5 = e4.find_child("e5").unwrap();
        assert!(e5.nags.contains(&Nag::BLUNDER));
    }

    #[test]
    fn test_parse_header_with_escaped_quotes() {
        let (key, value) = parse_header(r#"[Event "Test \"quoted\" Game"]"#).unwrap();
        assert_eq!(key, "Event");
        assert_eq!(value, r#"Test "quoted" Game"#);
    }

    #[test]
    fn test_parse_header_with_escaped_backslash() {
        let (key, value) = parse_header(r#"[Site "C:\\Games\\Chess"]"#).unwrap();
        assert_eq!(key, "Site");
        assert_eq!(value, r#"C:\Games\Chess"#);
    }

    #[test]
    fn test_parse_header_empty_value() {
        let (key, value) = parse_header(r#"[Event ""]"#).unwrap();
        assert_eq!(key, "Event");
        assert_eq!(value, "");
    }

    #[test]
    fn test_parse_header_unicode() {
        let (key, value) = parse_header(r#"[White "Каспаров"]"#).unwrap();
        assert_eq!(key, "White");
        assert_eq!(value, "Каспаров");
    }

    #[test]
    fn test_parse_header_missing_closing_bracket() {
        // Should return None for malformed headers
        assert!(parse_header(r#"[Event "Test""#).is_none());
    }

    #[test]
    fn test_parse_header_missing_key() {
        assert!(parse_header(r#"["value"]"#).is_none());
    }

    #[test]
    fn test_build_root_level_variation() {
        // Root-level variations should create alternative first moves
        let tokens = tokenize("(1. d4 d5) 1. e4 e5 *");
        let tree = build_tree(&tokens).unwrap();

        // Both d4 and e4 should be children of root
        assert_eq!(tree.root.children.len(), 2);

        // First variation: d4
        let d4 = tree.root.find_child("d4");
        assert!(d4.is_some(), "d4 should be a child of root");
        let d4 = d4.unwrap();
        assert!(d4.find_child("d5").is_some(), "d5 should follow d4");

        // Main line: e4
        let e4 = tree.root.find_child("e4");
        assert!(e4.is_some(), "e4 should be a child of root");
        let e4 = e4.unwrap();
        assert!(e4.find_child("e5").is_some(), "e5 should follow e4");
    }

    #[test]
    fn test_build_multiple_root_level_variations() {
        let tokens = tokenize("(1. d4) (1. c4) 1. e4 *");
        let tree = build_tree(&tokens).unwrap();

        // All three should be children of root
        assert_eq!(tree.root.children.len(), 3);
        assert!(tree.root.find_child("d4").is_some());
        assert!(tree.root.find_child("c4").is_some());
        assert!(tree.root.find_child("e4").is_some());
    }

    #[test]
    fn test_extra_closing_parens_handled_gracefully() {
        // Extra closing parens should be silently ignored (liberal input handling)
        let tokens = tokenize("1. e4 e5) 2. Nf3 *");
        let tree = build_tree(&tokens).unwrap();

        // The game should parse normally, ignoring the extra )
        let e4 = tree.root.find_child("e4").unwrap();
        let e5 = e4.find_child("e5").unwrap();
        assert!(e5.find_child("Nf3").is_some());
    }

    #[test]
    fn test_multiple_extra_closing_parens() {
        // Multiple extra closing parens should all be ignored
        let tokens = tokenize("1. e4)) e5))) 2. Nf3 *");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        let e5 = e4.find_child("e5").unwrap();
        assert!(e5.find_child("Nf3").is_some());
    }

    #[test]
    fn test_unclosed_variation_error() {
        // Unclosed variations should now produce an error
        let tokens = tokenize("1. e4 (1. d4 d5 2. Nf3 *");
        let result = build_tree(&tokens);

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Check it's a parse error
        if let crate::error::Error::Parse(parse_err) = err {
            assert_eq!(parse_err.code, ErrorCode::UnclosedVariation);
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_mixed_unbalanced_parens() {
        // Complex case: balanced variation with extra closing paren after it
        // 1. e4 e5 (2. Nf3)) 2. Bc4 *
        // The (2. Nf3) is a variation from e4 (alternative to e5), the extra ) is ignored
        let tokens = tokenize("1. e4 e5 (2. Nf3) 2. Bc4 *");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        // e5 is the main line continuation
        let e5 = e4.find_child("e5").unwrap();
        // Nf3 is a variation from e4 (sibling of e5)
        assert!(
            e4.find_child("Nf3").is_some(),
            "Nf3 should be a variation from e4"
        );
        // Bc4 continues the main line after e5 (extra ) was ignored)
        assert!(e5.find_child("Bc4").is_some(), "Bc4 should continue after e5");
    }

    #[test]
    fn test_error_has_source_context() {
        let source = "1. e4 (1... c5\n2. Nf3 *";
        let tokens = tokenize(source);
        let result = build_tree_with_options(&tokens, ParseMode::Lenient, Some("test.pgn".into()), Some(source));

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let crate::error::Error::Parse(parse_err) = err {
            assert!(!parse_err.source_context.is_empty());
            assert_eq!(parse_err.file, Some(PathBuf::from("test.pgn")));
        } else {
            panic!("Expected ParseError");
        }
    }
}
