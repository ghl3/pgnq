//! Token types for PGN parsing

use logos::Logos;

/// Token types produced by the lexer
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum Token {
    /// PGN header like [Event "Test"]
    #[regex(r#"\[[A-Za-z_][A-Za-z0-9_]*\s+"[^"]*"\]"#, |lex| lex.slice().to_string(), priority = 10)]
    Header(String),

    /// Move number like "1." or "1..."
    #[regex(r"[0-9]+\.+", |lex| lex.slice().to_string(), priority = 8)]
    MoveNumber(String),

    /// Standalone digits (not followed by dots) - these occur in prose as list markers like "1)" or "2)"
    #[regex(r"[0-9]+", |lex| lex.slice().to_string(), priority = 3)]
    Digit(String),

    /// Castling moves (must be before Move to have higher priority)
    /// Accepts both O-O-O (standard) and 0-0-0 (common variant with zeros)
    #[regex(r"[O0]-[O0]-[O0][+#]?", |lex| normalize_castling(lex.slice()), priority = 9)]
    CastleLong(String),

    /// Accepts both O-O (standard) and 0-0 (common variant with zeros)
    #[regex(r"[O0]-[O0][+#]?", |lex| normalize_castling(lex.slice()), priority = 9)]
    CastleShort(String),

    /// Chess move in SAN notation - piece moves
    #[regex(r"[KQRBN][a-h]?[1-8]?x?[a-h][1-8][+#]?", |lex| lex.slice().to_string(), priority = 7)]
    PieceMove(String),

    /// Pawn moves (e4, exd5, e8=Q, etc.)
    #[regex(r"[a-h]x?[a-h]?[1-8](=[QRBN])?[+#]?", |lex| lex.slice().to_string(), priority = 7)]
    PawnMove(String),

    /// Brace comment { text }
    #[regex(r"\{[^}]*\}", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    }, priority = 10)]
    BraceComment(String),

    /// Semicolon comment (to end of line)
    #[regex(r";[^\n]*", |lex| lex.slice()[1..].to_string(), priority = 10)]
    SemicolonComment(String),

    /// NAG like $1, $14
    #[regex(r"\$[0-9]+", |lex| lex.slice().to_string(), priority = 8)]
    Nag(String),

    /// Symbolic NAG like !!, ??, !?, ?!
    #[regex(r"!!", |lex| lex.slice().to_string(), priority = 6)]
    DoubleBang(String),

    #[regex(r"\?\?", |lex| lex.slice().to_string(), priority = 6)]
    DoubleQuestion(String),

    #[regex(r"!\?", |lex| lex.slice().to_string(), priority = 6)]
    BangQuestion(String),

    #[regex(r"\?!", |lex| lex.slice().to_string(), priority = 6)]
    QuestionBang(String),

    #[regex(r"!", |lex| lex.slice().to_string(), priority = 5)]
    Bang(String),

    #[regex(r"\?", |lex| lex.slice().to_string(), priority = 5)]
    Question(String),

    /// Positional assessment symbols
    #[regex(r"\+-", priority = 6)]
    WhiteWinning,

    #[regex(r"-\+", priority = 6)]
    BlackWinning,

    #[regex(r"\+/-|±", priority = 6)]
    WhiteBetter,

    #[regex(r"-/\+|∓", priority = 6)]
    BlackBetter,

    #[regex(r"\+=|⩲", priority = 6)]
    WhiteSlightlyBetter,

    #[regex(r"=\+|⩱", priority = 6)]
    BlackSlightlyBetter,

    /// Equal position symbol (standalone =)
    #[token("=", priority = 4)]
    Equal,

    /// Variation start
    #[token("(", priority = 10)]
    VariationStart,

    /// Variation end
    #[token(")", priority = 10)]
    VariationEnd,

    /// Game result
    #[regex(r"1-0", priority = 9)]
    WhiteWins,

    #[regex(r"0-1", priority = 9)]
    BlackWins,

    #[regex(r"1/2-1/2", priority = 9)]
    Draw,

    #[token("*", priority = 9)]
    Ongoing,

    /// Newline (significant for bare comment detection)
    #[regex(r"\n+", priority = 3)]
    Newline,

    /// Bare text that doesn't match other patterns (potential comment in Lichess format)
    /// Note: Does NOT include spaces, !, or ? - each word is a separate token
    /// Excludes ! and ? to avoid consuming NAGs attached to moves
    /// Includes trailing colons like "ideas:" which are common in prose
    #[regex(r"[A-Za-z][A-Za-z0-9,.'\-]*:?", |lex| lex.slice().to_string(), priority = 2)]
    BareText(String),

    /// Punctuation that can appear in prose (commas, semicolons, dashes, etc.)
    /// Not including ! or ? which are NAGs, or () which are variation markers
    /// Note: dash/hyphen needs special handling as it can be in moves (O-O) or prose
    #[regex(r"[,;\-]", |lex| lex.slice().to_string(), priority = 1)]
    Punctuation(String),
}

impl Token {
    /// Check if this token is a move
    pub fn is_move(&self) -> bool {
        matches!(
            self,
            Token::PieceMove(_)
                | Token::PawnMove(_)
                | Token::CastleLong(_)
                | Token::CastleShort(_)
        )
    }

    /// Check if this token is a comment
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            Token::BraceComment(_) | Token::SemicolonComment(_) | Token::BareText(_)
        )
    }

    /// Check if this token is a NAG
    pub fn is_nag(&self) -> bool {
        matches!(
            self,
            Token::Nag(_)
                | Token::DoubleBang(_)
                | Token::DoubleQuestion(_)
                | Token::BangQuestion(_)
                | Token::QuestionBang(_)
                | Token::Bang(_)
                | Token::Question(_)
                | Token::Equal
                | Token::WhiteWinning
                | Token::BlackWinning
                | Token::WhiteBetter
                | Token::BlackBetter
                | Token::WhiteSlightlyBetter
                | Token::BlackSlightlyBetter
        )
    }

    /// Check if this is a result token
    pub fn is_result(&self) -> bool {
        matches!(
            self,
            Token::WhiteWins | Token::BlackWins | Token::Draw | Token::Ongoing
        )
    }

    /// Get the move string if this is a move token
    pub fn as_move(&self) -> Option<&str> {
        match self {
            Token::PieceMove(s)
            | Token::PawnMove(s)
            | Token::CastleLong(s)
            | Token::CastleShort(s) => Some(s),
            _ => None,
        }
    }

    /// Get the comment text if this is a comment token
    pub fn as_comment(&self) -> Option<&str> {
        match self {
            Token::BraceComment(s) | Token::SemicolonComment(s) | Token::BareText(s) => Some(s),
            _ => None,
        }
    }

    /// Get NAG value
    pub fn as_nag_value(&self) -> Option<u8> {
        match self {
            Token::Nag(s) => s.strip_prefix('$').and_then(|n| n.parse().ok()),
            Token::Bang(_) => Some(1),
            Token::Question(_) => Some(2),
            Token::DoubleBang(_) => Some(3),
            Token::DoubleQuestion(_) => Some(4),
            Token::BangQuestion(_) => Some(5),
            Token::QuestionBang(_) => Some(6),
            Token::Equal => Some(10),
            Token::WhiteSlightlyBetter => Some(14),
            Token::BlackSlightlyBetter => Some(15),
            Token::WhiteBetter => Some(16),
            Token::BlackBetter => Some(17),
            Token::WhiteWinning => Some(18),
            Token::BlackWinning => Some(19),
            _ => None,
        }
    }
}

/// Normalize castling notation: convert 0-0 (zeros) to O-O (letters)
/// This ensures consistent output regardless of input format
fn normalize_castling(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '0' => result.push('O'),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Token::lexer(input).filter_map(|r| r.ok()).collect()
    }

    #[test]
    fn test_header_tokens() {
        let tokens = tokenize(r#"[Event "Test Game"]"#);
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Header(s) if s.contains("Event")));
    }

    #[test]
    fn test_move_tokens() {
        let tokens = tokenize("e4 Nf3 Bxe5 O-O O-O-O e8=Q+");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn test_comment_tokens() {
        let tokens = tokenize("e4 {This is a comment} e5");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::BraceComment(s) if s == "This is a comment")));
    }

    #[test]
    fn test_nag_tokens() {
        let tokens = tokenize("e4! Nf3?? $14");
        let nags: Vec<_> = tokens.iter().filter(|t| t.is_nag()).collect();
        assert_eq!(nags.len(), 3);
    }

    #[test]
    fn test_variation_tokens() {
        let tokens = tokenize("e4 (d4 d5) e5");
        assert!(tokens.iter().any(|t| matches!(t, Token::VariationStart)));
        assert!(tokens.iter().any(|t| matches!(t, Token::VariationEnd)));
    }

    #[test]
    fn test_castling_with_zeros() {
        // Test that 0-0 (zeros) is normalized to O-O (letters)
        let tokens = tokenize("e4 e5 0-0");
        assert!(tokens.iter().any(|t| matches!(t, Token::CastleShort(s) if s == "O-O")));

        let tokens = tokenize("d4 d5 0-0-0");
        assert!(tokens.iter().any(|t| matches!(t, Token::CastleLong(s) if s == "O-O-O")));
    }

    #[test]
    fn test_castling_with_zeros_and_check() {
        let tokens = tokenize("0-0+");
        assert!(tokens.iter().any(|t| matches!(t, Token::CastleShort(s) if s == "O-O+")));

        let tokens = tokenize("0-0-0#");
        assert!(tokens.iter().any(|t| matches!(t, Token::CastleLong(s) if s == "O-O-O#")));
    }

    #[test]
    fn test_castling_standard_notation() {
        // Ensure standard O-O notation still works
        let tokens = tokenize("O-O O-O-O");
        let moves: Vec<_> = tokens.iter().filter(|t| t.is_move()).collect();
        assert_eq!(moves.len(), 2);
        assert!(matches!(moves[0], Token::CastleShort(s) if s == "O-O"));
        assert!(matches!(moves[1], Token::CastleLong(s) if s == "O-O-O"));
    }

    #[test]
    fn test_normalize_castling() {
        assert_eq!(normalize_castling("0-0"), "O-O");
        assert_eq!(normalize_castling("0-0-0"), "O-O-O");
        assert_eq!(normalize_castling("O-O"), "O-O");
        assert_eq!(normalize_castling("O-O-O"), "O-O-O");
        assert_eq!(normalize_castling("0-0+"), "O-O+");
        assert_eq!(normalize_castling("0-0-0#"), "O-O-O#");
    }
}
