//! Lexer for PGN files - converts input text to token stream

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

/// Tokenize a PGN string into a vector of located tokens
///
/// This is a liberal tokenizer that handles multiple PGN formats:
/// - Standard PGN with {} comments
/// - Lichess format with bare text comments
/// - Mixed formats
pub fn tokenize(input: &str) -> Vec<LocatedToken> {
    let mut tokens: Vec<LocatedToken> = Vec::new();
    let lexer = Token::lexer(input);

    for (token_result, span) in lexer.spanned() {
        if let Ok(tok) = token_result {
            // Filter out bare text that looks like it's part of a move sequence
            // (this helps with the Lichess format detection)
            if let Token::BareText(ref text) = tok {
                let trimmed = text.trim();
                // Skip if it's empty or looks like a result
                if trimmed.is_empty() {
                    continue;
                }
                // Skip if it looks like a partial move number
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

    // Post-process: convert bare text to comments if we detect it's actually a comment
    // (lines that don't look like moves)
    post_process_tokens(&mut tokens);

    tokens
}

/// Tokenize without location info (for backward compatibility)
pub fn tokenize_simple(input: &str) -> Vec<Token> {
    tokenize(input).into_iter().map(|lt| lt.token).collect()
}

/// Detect and collapse variations that are actually parenthetical references in text.
/// Pattern: BareText ... ( MoveNumber? Move ) ... BareText
/// These are NOT real variations - they're references like "the Petrosian (7.d5)"
fn collapse_embedded_variations(tokens: &mut Vec<LocatedToken>) {
    let mut i = 0;
    while i < tokens.len() {
        // Look for BareText followed eventually by VariationStart
        if !matches!(tokens[i].token, Token::BareText(_)) {
            i += 1;
            continue;
        }

        // Find VariationStart after this BareText
        let var_start = (i + 1..tokens.len())
            .find(|&j| matches!(tokens[j].token, Token::VariationStart));

        let Some(var_start_idx) = var_start else {
            i += 1;
            continue;
        };

        // Check all tokens between i and var_start are BareText or Newline
        let all_baretext = (i + 1..var_start_idx)
            .all(|j| matches!(tokens[j].token, Token::BareText(_) | Token::Newline));
        if !all_baretext {
            i += 1;
            continue;
        }

        // Find matching VariationEnd (handle nesting)
        let mut depth = 0;
        let mut var_end_idx = None;
        for j in var_start_idx..tokens.len() {
            match &tokens[j].token {
                Token::VariationStart => depth += 1,
                Token::VariationEnd => {
                    depth -= 1;
                    if depth == 0 {
                        var_end_idx = Some(j);
                        break;
                    }
                }
                _ => {}
            }
        }

        let Some(var_end_idx) = var_end_idx else {
            i += 1;
            continue;
        };

        // Count move tokens inside the variation
        let move_count = (var_start_idx + 1..var_end_idx)
            .filter(|&j| tokens[j].token.is_move())
            .count();

        // If only 1-2 moves AND followed by BareText, it's likely a parenthetical reference
        if move_count > 2 {
            i += 1;
            continue; // Real variation with multiple moves
        }

        // Check if followed by BareText (not end of input, not another move)
        let followed_by_baretext =
            var_end_idx + 1 < tokens.len() && matches!(tokens[var_end_idx + 1].token, Token::BareText(_));

        if followed_by_baretext {
            // This is a parenthetical reference - collapse the variation into BareText
            let var_text = tokens[var_start_idx..=var_end_idx]
                .iter()
                .map(|lt| match &lt.token {
                    Token::VariationStart => "(".to_string(),
                    Token::VariationEnd => ")".to_string(),
                    Token::MoveNumber(s) => s.clone(),
                    Token::PawnMove(s) | Token::PieceMove(s) => s.clone(),
                    Token::CastleShort(s) | Token::CastleLong(s) => s.clone(),
                    Token::BareText(s) => s.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join("");

            // Get location from the start of the variation
            let loc = &tokens[var_start_idx];
            let new_token = LocatedToken::new(
                Token::BareText(var_text),
                loc.line,
                loc.column,
                loc.offset,
                tokens[var_end_idx].offset + tokens[var_end_idx].len - loc.offset,
            );

            // Replace variation tokens with single BareText
            tokens.splice(var_start_idx..=var_end_idx, std::iter::once(new_token));
            // Don't increment i - check again from same position
            continue;
        }

        i += 1;
    }
}

/// Post-process tokens to handle Lichess-style bare text comments
fn post_process_tokens(tokens: &mut Vec<LocatedToken>) {
    // Collapse embedded variations (parenthetical references in text)
    // These are patterns like "the Petrosian Variation (7.d5)" where (7.d5)
    // is NOT a real variation but a textual reference
    collapse_embedded_variations(tokens);

    // Note: Prose detection (distinguishing move references from real moves)
    // is now handled by the builder's state machine, not here in the lexer.
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
