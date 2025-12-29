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

    // Second: convert moves that appear inside prose to baretext
    // A move is "inside prose" if it directly follows a BareText token
    convert_prose_moves(tokens);
}

/// Convert move tokens that appear inside prose to baretext.
/// A move is "inside prose" if:
/// 1. It directly follows a BareText token, OR
/// 2. It follows a MoveNumber that follows BareText (e.g., "...but 6.Be3 is common")
///
/// This handles cases like "the pawn on f3 is weak" where f3 is
/// incorrectly tokenized as a move but is actually part of the prose.
fn convert_prose_moves(tokens: &mut Vec<Token>) {
    // First pass: mark move numbers that are in prose context
    let mut prose_move_numbers: Vec<usize> = Vec::new();
    for i in 0..tokens.len() {
        if matches!(&tokens[i], Token::MoveNumber(_)) && i > 0 {
            if matches!(&tokens[i - 1], Token::BareText(_)) {
                prose_move_numbers.push(i);
            }
        }
    }

    // Second pass: convert moves that follow baretext or prose move numbers
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].is_move() && i > 0 {
            let in_prose = matches!(&tokens[i - 1], Token::BareText(_))
                || (matches!(&tokens[i - 1], Token::MoveNumber(_))
                    && prose_move_numbers.contains(&(i - 1)));

            if in_prose {
                if let Some(san) = tokens[i].as_move() {
                    tokens[i] = Token::BareText(san.to_string());
                }
            }
        }
        i += 1;
    }

    // Third pass: convert prose move numbers to baretext
    for &idx in prose_move_numbers.iter().rev() {
        if let Token::MoveNumber(s) = &tokens[idx] {
            tokens[idx] = Token::BareText(s.clone());
        }
    }
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
    fn test_convert_prose_moves() {
        // "the pawn on f3" - f3 should be converted to baretext
        let tokens = tokenize("the pawn on f3 is weak");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 0, "f3 in prose should not be a move");

        // "1. e4" - e4 should remain a move
        let tokens = tokenize("1. e4");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 1, "e4 after move number should be a move");
    }

    #[test]
    fn test_move_number_in_prose() {
        // "6.Be3 is common" - both move number and move are prose references
        let tokens = tokenize("but 6.Be3 is the most common continuation");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 0, "6.Be3 in prose should not be a move");

        // But actual move line should work
        let tokens = tokenize("1. e4\n6. Be3");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 2, "both e4 and Be3 should be moves");
    }
}
