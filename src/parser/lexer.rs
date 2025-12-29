//! Lexer for PGN files - converts input text to token stream

use super::token::Token;
use logos::Logos;

/// Tokenize a PGN string into a vector of tokens
///
/// This is a liberal tokenizer that handles multiple PGN formats:
/// - Standard PGN with {} comments
/// - Lichess format with bare text comments
/// - Mixed formats
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let lexer = Token::lexer(input);

    for token in lexer {
        if let Ok(tok) = token {
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
            tokens.push(tok);
        }
    }

    // Post-process: convert bare text to comments if we detect it's actually a comment
    // (lines that don't look like moves)
    post_process_tokens(&mut tokens);

    tokens
}

/// Detect and collapse variations that are actually parenthetical references in text.
/// Pattern: BareText ... ( MoveNumber? Move ) ... BareText
/// These are NOT real variations - they're references like "the Petrosian (7.d5)"
fn collapse_embedded_variations(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        // Look for BareText followed eventually by VariationStart
        if !matches!(tokens[i], Token::BareText(_)) {
            i += 1;
            continue;
        }

        // Find VariationStart after this BareText
        let var_start = (i + 1..tokens.len()).find(|&j| matches!(tokens[j], Token::VariationStart));

        let Some(var_start_idx) = var_start else {
            i += 1;
            continue;
        };

        // Check all tokens between i and var_start are BareText or Newline
        let all_baretext = (i + 1..var_start_idx)
            .all(|j| matches!(tokens[j], Token::BareText(_) | Token::Newline));
        if !all_baretext {
            i += 1;
            continue;
        }

        // Find matching VariationEnd (handle nesting)
        let mut depth = 0;
        let mut var_end_idx = None;
        for j in var_start_idx..tokens.len() {
            match &tokens[j] {
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
            .filter(|&j| tokens[j].is_move())
            .count();

        // If only 1-2 moves AND followed by BareText, it's likely a parenthetical reference
        if move_count > 2 {
            i += 1;
            continue; // Real variation with multiple moves
        }

        // Check if followed by BareText (not end of input, not another move)
        let followed_by_baretext = var_end_idx + 1 < tokens.len()
            && matches!(tokens[var_end_idx + 1], Token::BareText(_));

        if followed_by_baretext {
            // This is a parenthetical reference - collapse the variation into BareText
            let var_text = tokens[var_start_idx..=var_end_idx]
                .iter()
                .map(|t| match t {
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

            // Replace variation tokens with single BareText
            tokens.splice(
                var_start_idx..=var_end_idx,
                std::iter::once(Token::BareText(var_text)),
            );
            // Don't increment i - check again from same position
            continue;
        }

        i += 1;
    }
}

/// Post-process tokens to handle Lichess-style bare text comments
fn post_process_tokens(tokens: &mut Vec<Token>) {
    // First: collapse embedded variations (parenthetical references in text)
    // These are patterns like "the Petrosian Variation (7.d5)" where (7.d5)
    // is NOT a real variation but a textual reference
    collapse_embedded_variations(tokens);

    // In Lichess format, bare text lines that don't look like moves are comments
    // We detect this by checking if bare text appears after a move and before the next move

    let mut i = 0;
    while i < tokens.len() {
        if let Token::BareText(text) = &tokens[i] {
            let trimmed = text.trim();

            // Check if this looks like a move line we missed
            if looks_like_move_line(trimmed) {
                // Try to extract moves from it
                let sub_tokens: Vec<Token> = Token::lexer(trimmed)
                    .filter_map(|r| r.ok())
                    .collect();

                if !sub_tokens.is_empty()
                    && sub_tokens.iter().any(|t| t.is_move())
                {
                    // Replace bare text with extracted tokens
                    tokens.splice(i..=i, sub_tokens);
                    continue;
                }
            }

            // It's a genuine comment - convert it to a brace comment internally
            // (This unifies handling in the builder)
        }
        i += 1;
    }
}

/// Check if text looks like it contains chess moves
fn looks_like_move_line(text: &str) -> bool {
    // Quick heuristics for move-like text
    let text = text.trim();

    // Check for move number patterns
    if text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return true;
    }

    // Check for piece letters at start
    if text.starts_with(['K', 'Q', 'R', 'B', 'N', 'O']) {
        return true;
    }

    // Check for pawn move patterns (a-h followed by digit or x)
    if let Some(first) = text.chars().next() {
        if ('a'..='h').contains(&first) {
            if let Some(second) = text.chars().nth(1) {
                if second.is_ascii_digit() || second == 'x' {
                    return true;
                }
            }
        }
    }

    false
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
            .filter(|t| matches!(t, Token::Header(_)))
            .collect();
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let tokens = tokenize("1. e4 {opening} e5 {response}");
        let comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::BraceComment(_)))
            .collect();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_tokenize_with_variations() {
        let tokens = tokenize("1. e4 (1. d4 d5) e5");
        assert!(tokens.iter().any(|t| matches!(t, Token::VariationStart)));
        assert!(tokens.iter().any(|t| matches!(t, Token::VariationEnd)));
    }

    #[test]
    fn test_tokenize_castling() {
        let tokens = tokenize("O-O O-O-O O-O+");
        let castles: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t, Token::CastleShort(_) | Token::CastleLong(_)))
            .collect();
        assert_eq!(castles.len(), 3);
    }

    #[test]
    fn test_looks_like_move_line() {
        assert!(looks_like_move_line("1. e4"));
        assert!(looks_like_move_line("Nf3"));
        assert!(looks_like_move_line("e4"));
        assert!(looks_like_move_line("O-O"));
        assert!(!looks_like_move_line("This is a comment"));
        assert!(!looks_like_move_line("The knight moves here"));
    }
}
