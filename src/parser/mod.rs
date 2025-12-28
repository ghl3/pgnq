//! PGN parser - tokenization and tree building

mod builder;
mod lexer;
mod token;

pub use builder::build_tree;
pub use lexer::tokenize;
pub use token::Token;

use crate::error::Result;
use crate::tree::GameTree;

/// Parse a PGN string into a GameTree
pub fn parse(input: &str) -> Result<GameTree> {
    let tokens = tokenize(input);
    build_tree(&tokens)
}

/// Parse multiple games from a PGN string
pub fn parse_all(input: &str) -> Result<Vec<GameTree>> {
    // Split on headers that appear after moves (indicating a new game)
    let mut games = Vec::new();
    let mut current_start = 0;
    let mut seen_moves = false; // Track if we've seen movetext after headers
    let mut current_pos = 0;

    for line in input.lines() {
        let line_len = line.len();
        let is_header = line.starts_with('[') && line.contains('"');
        let is_move_line = !line.trim().is_empty()
            && !is_header
            && !line.trim().starts_with(';'); // Not a semicolon comment

        if is_header && seen_moves {
            // New game starting - we've seen moves and now see a new header
            let game_text = &input[current_start..current_pos];
            if !game_text.trim().is_empty() {
                games.push(parse(game_text)?);
            }
            current_start = current_pos;
            seen_moves = false;
        } else if is_move_line {
            seen_moves = true;
        }

        // Move position past this line and its newline
        current_pos += line_len;
        if current_pos < input.len() {
            // Skip the newline character(s)
            if input[current_pos..].starts_with("\r\n") {
                current_pos += 2;
            } else if input[current_pos..].starts_with('\n') {
                current_pos += 1;
            }
        }
    }

    // Parse the last game
    let remaining = &input[current_start..];
    if !remaining.trim().is_empty() {
        games.push(parse(remaining)?);
    }

    if games.is_empty() {
        // Try parsing as a single game anyway
        games.push(parse(input)?);
    }

    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_game() {
        let pgn = r#"
[Event "Test"]
[White "Player 1"]
[Black "Player 2"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;
        let tree = parse(pgn).unwrap();
        assert_eq!(tree.header("Event"), Some("Test"));
        assert_eq!(tree.header("White"), Some("Player 1"));

        // Check the moves
        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.san, "e4");
        let e5 = e4.find_child("e5").unwrap();
        assert_eq!(e5.san, "e5");
    }

    #[test]
    fn test_parse_with_comments() {
        let pgn = "1. e4 {Best by test} e5 {Solid reply}";
        let tree = parse(pgn).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.comment.trim(), "Best by test");
    }

    #[test]
    fn test_parse_with_variations() {
        let pgn = "1. e4 e5 (1... c5 2. Nf3) 2. Nf3";
        let tree = parse(pgn).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.children.len(), 2); // e5 and c5

        let e5 = &e4.children[0];
        assert_eq!(e5.san, "e5");

        let c5 = &e4.children[1];
        assert_eq!(c5.san, "c5");
    }

    #[test]
    fn test_parse_headerless() {
        let pgn = "1. d4 Nf6 2. c4 e6";
        let tree = parse(pgn).unwrap();

        assert!(tree.headers.is_empty());
        let d4 = tree.root.find_child("d4").unwrap();
        assert_eq!(d4.san, "d4");
    }
}
