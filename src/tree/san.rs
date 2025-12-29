//! SAN (Standard Algebraic Notation) utilities for chess moves
//!
//! This module provides canonical functions for normalizing and comparing chess moves
//! in SAN format, handling move number prefixes and various notation styles.

/// Normalize a SAN move string by stripping move number prefixes.
///
/// This handles inputs like:
/// - "1. e4" -> "e4"
/// - "1... e5" -> "e5"
/// - "e4" -> "e4"
/// - "O-O" -> "O-O"
/// - "1. O-O-O" -> "O-O-O"
///
/// # Examples
/// ```
/// use pgnq::tree::san::normalize;
/// assert_eq!(normalize("1. e4"), "e4");
/// assert_eq!(normalize("15... Nf6"), "Nf6");
/// assert_eq!(normalize("O-O"), "O-O");
/// ```
pub fn normalize(san: &str) -> String {
    let s = san.trim();
    // Find first alphabetic char or O (for castling)
    // This strips any leading move number like "1." or "15..."
    if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == 'O') {
        s[idx..].to_string()
    } else {
        s.to_string()
    }
}

/// Check if two SAN moves match after normalization.
///
/// This compares moves ignoring move number prefixes, so "1. e4" matches "e4".
///
/// # Examples
/// ```
/// use pgnq::tree::san::moves_match;
/// assert!(moves_match("1. e4", "e4"));
/// assert!(moves_match("15... Nf6", "Nf6"));
/// assert!(!moves_match("e4", "d4"));
/// ```
pub fn moves_match(san1: &str, san2: &str) -> bool {
    normalize(san1) == normalize(san2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_with_move_number() {
        assert_eq!(normalize("1. e4"), "e4");
        assert_eq!(normalize("1... e5"), "e5");
        assert_eq!(normalize("15. Nf3"), "Nf3");
        assert_eq!(normalize("42... Qxd4"), "Qxd4");
    }

    #[test]
    fn test_normalize_without_move_number() {
        assert_eq!(normalize("e4"), "e4");
        assert_eq!(normalize("Nf3"), "Nf3");
        assert_eq!(normalize("Qxd4+"), "Qxd4+");
    }

    #[test]
    fn test_normalize_castling() {
        assert_eq!(normalize("O-O"), "O-O");
        assert_eq!(normalize("O-O-O"), "O-O-O");
        assert_eq!(normalize("1. O-O"), "O-O");
        assert_eq!(normalize("15... O-O-O"), "O-O-O");
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize("  e4  "), "e4");
        assert_eq!(normalize("  1. e4  "), "e4");
    }

    #[test]
    fn test_moves_match() {
        assert!(moves_match("1. e4", "e4"));
        assert!(moves_match("e4", "1. e4"));
        assert!(moves_match("15... Nf6", "Nf6"));
        assert!(moves_match("O-O", "1. O-O"));
    }

    #[test]
    fn test_moves_dont_match() {
        assert!(!moves_match("e4", "d4"));
        assert!(!moves_match("Nf3", "Nc3"));
        assert!(!moves_match("O-O", "O-O-O"));
    }
}
