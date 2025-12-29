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

    // ========================================================================
    // Constants Tests
    // ========================================================================

    #[test]
    fn test_nag_constants() {
        assert_eq!(Nag::GOOD_MOVE.0, 1);
        assert_eq!(Nag::POOR_MOVE.0, 2);
        assert_eq!(Nag::BRILLIANT_MOVE.0, 3);
        assert_eq!(Nag::BLUNDER.0, 4);
        assert_eq!(Nag::INTERESTING_MOVE.0, 5);
        assert_eq!(Nag::DUBIOUS_MOVE.0, 6);
        assert_eq!(Nag::EQUAL.0, 10);
        assert_eq!(Nag::WHITE_SLIGHT_ADVANTAGE.0, 14);
        assert_eq!(Nag::BLACK_SLIGHT_ADVANTAGE.0, 15);
        assert_eq!(Nag::WHITE_MODERATE_ADVANTAGE.0, 16);
        assert_eq!(Nag::BLACK_MODERATE_ADVANTAGE.0, 17);
        assert_eq!(Nag::WHITE_DECISIVE_ADVANTAGE.0, 18);
        assert_eq!(Nag::BLACK_DECISIVE_ADVANTAGE.0, 19);
    }

    // ========================================================================
    // to_symbol Tests
    // ========================================================================

    #[test]
    fn test_nag_symbols() {
        assert_eq!(Nag::GOOD_MOVE.to_symbol(), Some("!"));
        assert_eq!(Nag::BLUNDER.to_symbol(), Some("??"));
        assert_eq!(Nag(100).to_symbol(), None);
    }

    #[test]
    fn test_nag_symbol_move_annotations() {
        assert_eq!(Nag(1).to_symbol(), Some("!"));
        assert_eq!(Nag(2).to_symbol(), Some("?"));
        assert_eq!(Nag(3).to_symbol(), Some("!!"));
        assert_eq!(Nag(4).to_symbol(), Some("??"));
        assert_eq!(Nag(5).to_symbol(), Some("!?"));
        assert_eq!(Nag(6).to_symbol(), Some("?!"));
    }

    #[test]
    fn test_nag_symbol_positional() {
        assert_eq!(Nag(10).to_symbol(), Some("="));
        assert_eq!(Nag(14).to_symbol(), Some("+="));
        assert_eq!(Nag(15).to_symbol(), Some("=+"));
        assert_eq!(Nag(16).to_symbol(), Some("+/-"));
        assert_eq!(Nag(17).to_symbol(), Some("-/+"));
        assert_eq!(Nag(18).to_symbol(), Some("+-"));
        assert_eq!(Nag(19).to_symbol(), Some("-+"));
    }

    #[test]
    fn test_nag_symbol_no_symbol() {
        // NAGs without standard symbols
        assert_eq!(Nag(0).to_symbol(), None);
        assert_eq!(Nag(7).to_symbol(), None);
        assert_eq!(Nag(8).to_symbol(), None);
        assert_eq!(Nag(9).to_symbol(), None);
        assert_eq!(Nag(11).to_symbol(), None);
        assert_eq!(Nag(20).to_symbol(), None);
        assert_eq!(Nag(100).to_symbol(), None);
        assert_eq!(Nag(255).to_symbol(), None);
    }

    // ========================================================================
    // from_symbol Tests
    // ========================================================================

    #[test]
    fn test_from_symbol() {
        assert_eq!(Nag::from_symbol("!"), Some(Nag::GOOD_MOVE));
        assert_eq!(Nag::from_symbol("!!"), Some(Nag::BRILLIANT_MOVE));
        assert_eq!(Nag::from_symbol("+-"), Some(Nag::WHITE_DECISIVE_ADVANTAGE));
    }

    #[test]
    fn test_from_symbol_move_annotations() {
        assert_eq!(Nag::from_symbol("!"), Some(Nag(1)));
        assert_eq!(Nag::from_symbol("?"), Some(Nag(2)));
        assert_eq!(Nag::from_symbol("!!"), Some(Nag(3)));
        assert_eq!(Nag::from_symbol("??"), Some(Nag(4)));
        assert_eq!(Nag::from_symbol("!?"), Some(Nag(5)));
        assert_eq!(Nag::from_symbol("?!"), Some(Nag(6)));
    }

    #[test]
    fn test_from_symbol_positional() {
        assert_eq!(Nag::from_symbol("="), Some(Nag(10)));
        assert_eq!(Nag::from_symbol("+="), Some(Nag(14)));
        assert_eq!(Nag::from_symbol("=+"), Some(Nag(15)));
        assert_eq!(Nag::from_symbol("+/-"), Some(Nag(16)));
        assert_eq!(Nag::from_symbol("-/+"), Some(Nag(17)));
        assert_eq!(Nag::from_symbol("+-"), Some(Nag(18)));
        assert_eq!(Nag::from_symbol("-+"), Some(Nag(19)));
    }

    #[test]
    fn test_from_symbol_unicode() {
        // Unicode chess symbols
        assert_eq!(Nag::from_symbol("⩲"), Some(Nag(14))); // White slight advantage
        assert_eq!(Nag::from_symbol("⩱"), Some(Nag(15))); // Black slight advantage
        assert_eq!(Nag::from_symbol("±"), Some(Nag(16))); // White moderate advantage
        assert_eq!(Nag::from_symbol("∓"), Some(Nag(17))); // Black moderate advantage
    }

    #[test]
    fn test_from_symbol_invalid() {
        assert_eq!(Nag::from_symbol(""), None);
        assert_eq!(Nag::from_symbol("x"), None);
        assert_eq!(Nag::from_symbol("!!!"), None);
        assert_eq!(Nag::from_symbol("???"), None);
        assert_eq!(Nag::from_symbol("$1"), None); // Use from_dollar_notation
    }

    // ========================================================================
    // from_dollar_notation Tests
    // ========================================================================

    #[test]
    fn test_from_dollar() {
        assert_eq!(Nag::from_dollar_notation("$1"), Some(Nag(1)));
        assert_eq!(Nag::from_dollar_notation("$14"), Some(Nag(14)));
        assert_eq!(Nag::from_dollar_notation("14"), None);
    }

    #[test]
    fn test_from_dollar_all_values() {
        assert_eq!(Nag::from_dollar_notation("$0"), Some(Nag(0)));
        assert_eq!(Nag::from_dollar_notation("$1"), Some(Nag(1)));
        assert_eq!(Nag::from_dollar_notation("$100"), Some(Nag(100)));
        assert_eq!(Nag::from_dollar_notation("$255"), Some(Nag(255)));
    }

    #[test]
    fn test_from_dollar_boundary_values() {
        // u8 boundaries
        assert_eq!(Nag::from_dollar_notation("$0"), Some(Nag(0)));
        assert_eq!(Nag::from_dollar_notation("$255"), Some(Nag(255)));
    }

    #[test]
    fn test_from_dollar_overflow() {
        // Values > 255 should fail
        assert_eq!(Nag::from_dollar_notation("$256"), None);
        assert_eq!(Nag::from_dollar_notation("$1000"), None);
    }

    #[test]
    fn test_from_dollar_invalid() {
        assert_eq!(Nag::from_dollar_notation(""), None);
        assert_eq!(Nag::from_dollar_notation("$"), None);
        assert_eq!(Nag::from_dollar_notation("$-1"), None);
        assert_eq!(Nag::from_dollar_notation("$abc"), None);
        assert_eq!(Nag::from_dollar_notation("1"), None);
        assert_eq!(Nag::from_dollar_notation("$1.5"), None);
    }

    // ========================================================================
    // Display Tests
    // ========================================================================

    #[test]
    fn test_display_symbolic() {
        assert_eq!(format!("{}", Nag(1)), "!");
        assert_eq!(format!("{}", Nag(2)), "?");
        assert_eq!(format!("{}", Nag(3)), "!!");
        assert_eq!(format!("{}", Nag(4)), "??");
        assert_eq!(format!("{}", Nag(5)), "!?");
        assert_eq!(format!("{}", Nag(6)), "?!");
    }

    #[test]
    fn test_display_positional() {
        assert_eq!(format!("{}", Nag(10)), "=");
        assert_eq!(format!("{}", Nag(14)), "+=");
        assert_eq!(format!("{}", Nag(18)), "+-");
        assert_eq!(format!("{}", Nag(19)), "-+");
    }

    #[test]
    fn test_display_numeric() {
        // NAGs without symbols display as $N
        assert_eq!(format!("{}", Nag(0)), "$0");
        assert_eq!(format!("{}", Nag(7)), "$7");
        assert_eq!(format!("{}", Nag(100)), "$100");
        assert_eq!(format!("{}", Nag(255)), "$255");
    }

    // ========================================================================
    // Equality and Clone Tests
    // ========================================================================

    #[test]
    fn test_nag_equality() {
        assert_eq!(Nag(1), Nag(1));
        assert_eq!(Nag::GOOD_MOVE, Nag(1));
        assert_ne!(Nag(1), Nag(2));
    }

    #[test]
    fn test_nag_clone() {
        let nag = Nag(5);
        let cloned = nag;  // Copy
        assert_eq!(nag, cloned);
    }

    #[test]
    fn test_nag_copy() {
        let nag = Nag(10);
        let copied = nag;
        assert_eq!(nag, copied);
    }

    // ========================================================================
    // Hash Tests
    // ========================================================================

    #[test]
    fn test_nag_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Nag(1));
        set.insert(Nag(2));
        set.insert(Nag(1)); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Nag(1)));
        assert!(set.contains(&Nag(2)));
    }

    // ========================================================================
    // Roundtrip Tests
    // ========================================================================

    #[test]
    fn test_symbol_roundtrip() {
        let nags = [
            Nag::GOOD_MOVE,
            Nag::POOR_MOVE,
            Nag::BRILLIANT_MOVE,
            Nag::BLUNDER,
            Nag::INTERESTING_MOVE,
            Nag::DUBIOUS_MOVE,
        ];

        for nag in nags {
            let symbol = nag.to_symbol().unwrap();
            let parsed = Nag::from_symbol(symbol).unwrap();
            assert_eq!(nag, parsed);
        }
    }

    #[test]
    fn test_dollar_display_roundtrip() {
        // For NAGs without symbols, Display produces $N which should parse back
        for i in [0u8, 7, 8, 9, 11, 12, 13, 20, 100, 255] {
            let nag = Nag(i);
            let display = format!("{}", nag);
            let parsed = Nag::from_dollar_notation(&display).unwrap();
            assert_eq!(nag, parsed);
        }
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_nag_zero() {
        let nag = Nag(0);
        assert_eq!(nag.to_symbol(), None);
        assert_eq!(format!("{}", nag), "$0");
    }

    #[test]
    fn test_nag_max() {
        let nag = Nag(255);
        assert_eq!(nag.to_symbol(), None);
        assert_eq!(format!("{}", nag), "$255");
    }

    #[test]
    fn test_uncommon_nags() {
        // Test some uncommon but valid NAGs
        // $13 = unclear, $36 = with initiative, etc.
        for i in 20..=50 {
            let nag = Nag(i);
            // Should not panic
            let _ = nag.to_symbol();
            let _ = format!("{}", nag);
        }
    }
}
