//! Phase 1: Lexer
//!
//! Converts input text to a stream of located tokens.
//! This phase is mode-agnostic; all heuristics are applied in Phase 2.

use super::token::Token;
use logos::Logos;

/// A token with its location information
#[derive(Debug, Clone)]
pub struct LocatedToken {
    /// The token itself
    pub token: Token,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in source
    pub offset: usize,
    /// Length of the token in bytes
    pub len: usize,
}

impl LocatedToken {
    /// Create a new located token
    pub fn new(token: Token, line: usize, column: usize, offset: usize, len: usize) -> Self {
        Self {
            token,
            line,
            column,
            offset,
            len,
        }
    }

    /// Check if this is a move token
    pub fn is_move(&self) -> bool {
        self.token.is_move()
    }

    /// Check if this is a comment token
    pub fn is_comment(&self) -> bool {
        self.token.is_comment()
    }
}

/// Calculate line and column from byte offset
fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Tokenize a PGN string into located tokens.
///
/// This is pure tokenization with no heuristics applied.
/// Post-processing (if any) happens in Phase 2.
pub fn tokenize(input: &str) -> Vec<LocatedToken> {
    let mut tokens: Vec<LocatedToken> = Vec::new();
    let lexer = Token::lexer(input);

    for (token_result, span) in lexer.spanned() {
        if let Ok(tok) = token_result {
            // Filter out bare text that is empty or looks like a partial move number
            if let Token::BareText(ref text) = tok {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    continue;
                }
            }
            let (line, column) = offset_to_line_col(input, span.start);
            tokens.push(LocatedToken::new(
                tok,
                line,
                column,
                span.start,
                span.end - span.start,
            ));
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("1. e4 e5 2. Nf3");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 3); // e4, e5, Nf3
    }

    #[test]
    fn test_tokenize_with_headers() {
        let input = r#"[Event "Test"]
[White "Player"]

1. e4"#;
        let tokens = tokenize(input);
        let headers: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Header(_)))
            .collect();
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let tokens = tokenize("1. e4 {opening} e5 {response}");
        let comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::BraceComment(_)))
            .collect();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_tokenize_with_variations() {
        let tokens = tokenize("1. e4 (1. d4 d5) e5");
        assert!(tokens.iter().any(|t| matches!(t.token, Token::VariationStart)));
        assert!(tokens.iter().any(|t| matches!(t.token, Token::VariationEnd)));
    }

    #[test]
    fn test_tokenize_castling() {
        let tokens = tokenize("O-O O-O-O O-O+");
        let castles: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::CastleShort(_) | Token::CastleLong(_)))
            .collect();
        assert_eq!(castles.len(), 3);
    }

    #[test]
    fn test_token_locations() {
        let input = "1. e4\ne5";
        let tokens = tokenize(input);

        // Find e4 move
        let e4 = tokens.iter().find(|t| matches!(&t.token, Token::PawnMove(s) if s == "e4"));
        assert!(e4.is_some());
        let e4 = e4.unwrap();
        assert_eq!(e4.line, 1);
        assert_eq!(e4.column, 4); // "1. " is 3 chars, then "e4" starts at 4

        // Find e5 move
        let e5 = tokens.iter().find(|t| matches!(&t.token, Token::PawnMove(s) if s == "e5"));
        assert!(e5.is_some());
        let e5 = e5.unwrap();
        assert_eq!(e5.line, 2);
        assert_eq!(e5.column, 1);
    }

    #[test]
    fn test_multiline_locations() {
        let input = "1. e4 e5\n2. Nf3 Nc6\n3. Bb5";
        let tokens = tokenize(input);

        let bb5 = tokens.iter().find(|t| matches!(&t.token, Token::PieceMove(s) if s == "Bb5"));
        assert!(bb5.is_some());
        let bb5 = bb5.unwrap();
        assert_eq!(bb5.line, 3);
    }

    // Note: Tests for prose detection (distinguishing move references from real moves)
    // are in the parser/mod.rs tests, since that logic is now in the builder's state machine.
}
