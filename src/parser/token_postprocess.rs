//! Phase 2: Token Post-Processing
//!
//! Transforms the token stream after lexing but before tree construction.
//! This phase handles patterns that are ambiguous at the lexer level.
//!
//! ## Transformations
//!
//! - [`collapse_prose`]: Converts parenthetical move references in prose to BareText

use super::lexer::LocatedToken;
use super::token::Token;
use crate::error::ParseMode;

/// Process tokens, applying mode-dependent transformations.
///
/// In Lenient mode, applies heuristics to handle ambiguous patterns.
/// In Strict mode, returns tokens unchanged.
pub fn process(mut tokens: Vec<LocatedToken>, mode: ParseMode) -> Vec<LocatedToken> {
    if mode == ParseMode::Lenient {
        collapse_prose(&mut tokens);
    }
    tokens
}

/// Collapse variation tokens into prose when they appear to be
/// parenthetical move references embedded in text.
///
/// # Pattern Recognized
///
/// ```text
/// BareText ... ( [MoveNumber?] Move{1-2} ) ... BareText
/// ```
///
/// # Conditions (all must be true)
///
/// 1. Preceded by `BareText` token
/// 2. No structural tokens (moves, headers) between `BareText` and `(`
/// 3. Variation contains at most 2 move tokens
/// 4. Followed immediately by `BareText` token
///
/// # Examples
///
/// **Collapsed (all conditions met):**
/// - `"the Petrosian (7.d5) is avoided"` → `(7.d5)` becomes BareText
/// - `"Italian (3.Bc4) or Spanish"` → `(3.Bc4)` becomes BareText
///
/// **NOT collapsed:**
/// - `"1. e4 (1... c5 2. Nf3)"` — preceded by move, not prose
/// - `"White plays (3.Bc4)"` — not followed by BareText
/// - `"sideline (1... c5 2. Nf3 d6)"` — contains 3 moves
fn collapse_prose(tokens: &mut Vec<LocatedToken>) {
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
        let followed_by_baretext = var_end_idx + 1 < tokens.len()
            && matches!(tokens[var_end_idx + 1].token, Token::BareText(_));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::tokenize;

    /// Count VariationStart tokens in the stream
    fn count_variation_starts(tokens: &[LocatedToken]) -> usize {
        tokens
            .iter()
            .filter(|t| matches!(t.token, Token::VariationStart))
            .count()
    }

    #[test]
    fn test_collapse_simple_parenthetical() {
        // "prose (7.d5) prose" → variation collapsed
        let input = "the Petrosian Variation (7.d5) is avoided";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        // Should have no variation tokens - collapsed to BareText
        assert_eq!(
            count_variation_starts(&processed),
            0,
            "Parenthetical reference should be collapsed"
        );

        // Should have BareText containing "(7.d5)"
        let has_collapsed = processed
            .iter()
            .any(|t| matches!(&t.token, Token::BareText(s) if s.contains("(7.d5)")));
        assert!(has_collapsed, "Collapsed variation should appear as BareText");
    }

    #[test]
    fn test_collapse_two_move_parenthetical() {
        // Two moves still qualifies for collapsing
        let input = "Italian (3.Bc4 Bc5) is solid";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        assert_eq!(
            count_variation_starts(&processed),
            0,
            "Two-move parenthetical should be collapsed"
        );
    }

    #[test]
    fn test_preserve_real_variation() {
        // "(1... c5 2. Nf3 d6)" with 3+ moves → NOT collapsed
        let input = "1. e4 (1... c5 2. Nf3 d6) e5";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        assert_eq!(
            count_variation_starts(&processed),
            1,
            "Real variation with 3+ moves should be preserved"
        );
    }

    #[test]
    fn test_no_collapse_without_trailing_prose() {
        // "plays (3.Bc4)" at end → NOT collapsed
        let input = "White plays (3.Bc4)";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        assert_eq!(
            count_variation_starts(&processed),
            1,
            "Variation not followed by prose should be preserved"
        );
    }

    #[test]
    fn test_no_collapse_after_move() {
        // "e4 (1... c5)" — preceded by move → NOT collapsed
        let input = "1. e4 (1... c5) e5";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        // The ( is preceded by PawnMove("e4"), not BareText
        assert_eq!(
            count_variation_starts(&processed),
            1,
            "Variation preceded by move should be preserved"
        );
    }

    #[test]
    fn test_nested_variations_handled() {
        // Nested variations should track depth correctly
        let input = "prose ((inner)) more";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        // Inner variation has no moves, outer should be processed
        // This tests that depth tracking works
        assert!(processed.len() > 0);
    }

    #[test]
    fn test_multiple_parenthetical_refs() {
        // "Ruy Lopez (3.Bb5) or Italian (3.Bc4)" → both collapsed
        let input = "Ruy Lopez (3.Bb5) or Italian (3.Bc4) here";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        assert_eq!(
            count_variation_starts(&processed),
            0,
            "Both parenthetical references should be collapsed"
        );
    }

    #[test]
    fn test_strict_mode_no_changes() {
        // In strict mode, no collapsing happens
        let input = "the Petrosian Variation (7.d5) is avoided";
        let tokens = tokenize(input);
        let before_count = count_variation_starts(&tokens);

        let processed = process(tokens, ParseMode::Strict);

        assert_eq!(
            count_variation_starts(&processed),
            before_count,
            "Strict mode should not modify tokens"
        );
    }

    #[test]
    fn test_preserves_token_locations() {
        // Collapsed tokens should have valid location info
        let input = "prose (d5) more";
        let tokens = tokenize(input);
        let processed = process(tokens, ParseMode::Lenient);

        for token in &processed {
            assert!(token.line >= 1, "Line should be 1-indexed");
            assert!(token.column >= 1, "Column should be 1-indexed");
        }
    }
}
