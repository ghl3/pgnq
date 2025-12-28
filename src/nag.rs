//! Numeric Annotation Glyphs (NAGs) for chess move annotations

use serde::{Deserialize, Serialize};
use std::fmt;

/// Numeric Annotation Glyph - standard chess annotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nag(pub u8);

impl Nag {
    // Common NAGs
    pub const GOOD_MOVE: Nag = Nag(1);
    pub const POOR_MOVE: Nag = Nag(2);
    pub const BRILLIANT_MOVE: Nag = Nag(3);
    pub const BLUNDER: Nag = Nag(4);
    pub const INTERESTING_MOVE: Nag = Nag(5);
    pub const DUBIOUS_MOVE: Nag = Nag(6);
    pub const EQUAL: Nag = Nag(10);
    pub const WHITE_SLIGHT_ADVANTAGE: Nag = Nag(14);
    pub const BLACK_SLIGHT_ADVANTAGE: Nag = Nag(15);
    pub const WHITE_MODERATE_ADVANTAGE: Nag = Nag(16);
    pub const BLACK_MODERATE_ADVANTAGE: Nag = Nag(17);
    pub const WHITE_DECISIVE_ADVANTAGE: Nag = Nag(18);
    pub const BLACK_DECISIVE_ADVANTAGE: Nag = Nag(19);

    /// Convert NAG to its symbolic representation
    pub fn to_symbol(self) -> Option<&'static str> {
        match self.0 {
            1 => Some("!"),
            2 => Some("?"),
            3 => Some("!!"),
            4 => Some("??"),
            5 => Some("!?"),
            6 => Some("?!"),
            10 => Some("="),
            14 => Some("+="),
            15 => Some("=+"),
            16 => Some("+/-"),
            17 => Some("-/+"),
            18 => Some("+-"),
            19 => Some("-+"),
            _ => None,
        }
    }

    /// Parse a NAG from a symbol string (!, ?, !!, etc.)
    pub fn from_symbol(s: &str) -> Option<Nag> {
        match s {
            "!!" => Some(Nag::BRILLIANT_MOVE),
            "??" => Some(Nag::BLUNDER),
            "!?" => Some(Nag::INTERESTING_MOVE),
            "?!" => Some(Nag::DUBIOUS_MOVE),
            "!" => Some(Nag::GOOD_MOVE),
            "?" => Some(Nag::POOR_MOVE),
            "=" => Some(Nag::EQUAL),
            "+=" | "⩲" => Some(Nag::WHITE_SLIGHT_ADVANTAGE),
            "=+" | "⩱" => Some(Nag::BLACK_SLIGHT_ADVANTAGE),
            "+/-" | "±" => Some(Nag::WHITE_MODERATE_ADVANTAGE),
            "-/+" | "∓" => Some(Nag::BLACK_MODERATE_ADVANTAGE),
            "+-" => Some(Nag::WHITE_DECISIVE_ADVANTAGE),
            "-+" => Some(Nag::BLACK_DECISIVE_ADVANTAGE),
            _ => None,
        }
    }

    /// Parse a NAG from $N notation
    pub fn from_dollar_notation(s: &str) -> Option<Nag> {
        if s.starts_with('$') {
            s[1..].parse::<u8>().ok().map(Nag)
        } else {
            None
        }
    }
}

impl fmt::Display for Nag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sym) = self.to_symbol() {
            write!(f, "{}", sym)
        } else {
            write!(f, "${}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nag_symbols() {
        assert_eq!(Nag::GOOD_MOVE.to_symbol(), Some("!"));
        assert_eq!(Nag::BLUNDER.to_symbol(), Some("??"));
        assert_eq!(Nag(100).to_symbol(), None);
    }

    #[test]
    fn test_from_symbol() {
        assert_eq!(Nag::from_symbol("!"), Some(Nag::GOOD_MOVE));
        assert_eq!(Nag::from_symbol("!!"), Some(Nag::BRILLIANT_MOVE));
        assert_eq!(Nag::from_symbol("+-"), Some(Nag::WHITE_DECISIVE_ADVANTAGE));
    }

    #[test]
    fn test_from_dollar() {
        assert_eq!(Nag::from_dollar_notation("$1"), Some(Nag(1)));
        assert_eq!(Nag::from_dollar_notation("$14"), Some(Nag(14)));
        assert_eq!(Nag::from_dollar_notation("14"), None);
    }
}
