//! Tree builder - constructs GameTree from token stream

use super::token::Token;
use crate::error::Result;
use crate::nag::Nag;
use crate::tree::{GameNode, GameResult, GameTree};

/// Build a GameTree from a token stream
pub fn build_tree(tokens: &[Token]) -> Result<GameTree> {
    let mut tree = GameTree::new();

    // Track the path from root to current position for proper variation handling
    // path_stack[i] points to the node at depth i (path_stack[0] = root)
    let mut path_stack: Vec<*mut GameNode> = vec![&mut tree.root as *mut GameNode];

    // Stack for tracking where to return after a variation ends
    // Each entry is (path_depth, node_ptr) where we should restore to
    let mut return_stack: Vec<(usize, *mut GameNode)> = Vec::new();

    let mut pending_comment = String::new();
    let mut pending_nags: Vec<Nag> = Vec::new();
    let mut current_move_number: u16 = 1;
    let mut expect_black = false;

    for token in tokens {
        match token {
            Token::Header(header_str) => {
                if let Some((key, value)) = parse_header(header_str) {
                    if key == "Result" {
                        if let Some(result) = GameResult::parse(&value) {
                            tree.result = result;
                        }
                    }
                    tree.headers.insert(key, value);
                }
            }

            Token::MoveNumber(num_str) => {
                if let Some(num) = parse_move_number(num_str) {
                    current_move_number = num;
                    expect_black = num_str.contains("...");
                }
            }

            Token::PieceMove(san)
            | Token::PawnMove(san)
            | Token::CastleLong(san)
            | Token::CastleShort(san) => {
                let mut node = GameNode::new(san.clone());
                node.move_number = Some(current_move_number);
                node.is_black = expect_black;

                if !pending_comment.is_empty() {
                    node.comment = std::mem::take(&mut pending_comment);
                }
                node.nags = std::mem::take(&mut pending_nags);

                // Get parent (last node in path_stack) and add child
                let parent_ptr = *path_stack.last().unwrap();
                let parent = unsafe { &mut *parent_ptr };
                let new_node = parent.add_child(node);
                path_stack.push(new_node as *mut GameNode);

                if expect_black {
                    current_move_number += 1;
                    expect_black = false;
                } else {
                    expect_black = true;
                }
            }

            Token::BraceComment(text) | Token::SemicolonComment(text) => {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }

                // path_stack.len() == 1 means we're at root (only root in stack)
                if path_stack.len() == 1 {
                    if !pending_comment.is_empty() {
                        pending_comment.push(' ');
                    }
                    pending_comment.push_str(text);
                } else {
                    // Get the current move node (last in path_stack)
                    let current_ptr = *path_stack.last().unwrap();
                    let current = unsafe { &mut *current_ptr };
                    if !current.comment.is_empty() {
                        current.comment.push(' ');
                    }
                    current.comment.push_str(text);
                }
            }

            Token::BareText(text) => {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                // Bare text is treated as a comment in Lichess format
                if path_stack.len() == 1 {
                    if !pending_comment.is_empty() {
                        pending_comment.push(' ');
                    }
                    pending_comment.push_str(text);
                } else {
                    let current_ptr = *path_stack.last().unwrap();
                    let current = unsafe { &mut *current_ptr };
                    if !current.comment.is_empty() {
                        current.comment.push(' ');
                    }
                    current.comment.push_str(text);
                }
            }

            Token::Nag(nag_str) => {
                if let Some(nag) = Nag::from_dollar_notation(nag_str) {
                    if path_stack.len() == 1 {
                        pending_nags.push(nag);
                    } else {
                        let current_ptr = *path_stack.last().unwrap();
                        let current = unsafe { &mut *current_ptr };
                        current.nags.push(nag);
                    }
                }
            }

            // Handle all symbolic NAGs
            Token::DoubleBang(_)
            | Token::DoubleQuestion(_)
            | Token::BangQuestion(_)
            | Token::QuestionBang(_)
            | Token::Bang(_)
            | Token::Question(_) => {
                if let Some(nag_val) = token.as_nag_value() {
                    let nag = Nag(nag_val);
                    if path_stack.len() == 1 {
                        pending_nags.push(nag);
                    } else {
                        let current_ptr = *path_stack.last().unwrap();
                        let current = unsafe { &mut *current_ptr };
                        current.nags.push(nag);
                    }
                }
            }

            // Positional assessment NAGs
            Token::WhiteWinning
            | Token::BlackWinning
            | Token::WhiteBetter
            | Token::BlackBetter
            | Token::WhiteSlightlyBetter
            | Token::BlackSlightlyBetter => {
                if let Some(nag_val) = token.as_nag_value() {
                    let nag = Nag(nag_val);
                    if path_stack.len() == 1 {
                        pending_nags.push(nag);
                    } else {
                        let current_ptr = *path_stack.last().unwrap();
                        let current = unsafe { &mut *current_ptr };
                        current.nags.push(nag);
                    }
                }
            }

            Token::VariationStart => {
                // Variation is an alternative to the preceding move
                // Push current position to return to after variation ends
                if path_stack.len() > 1 {
                    let current_ptr = *path_stack.last().unwrap();
                    return_stack.push((path_stack.len(), current_ptr));
                    // Pop the current move to go back to its parent
                    // The variation's moves will be siblings of the current move
                    path_stack.pop();
                }
            }

            Token::VariationEnd => {
                // Restore to the position we saved at VariationStart
                if let Some((depth, node_ptr)) = return_stack.pop() {
                    // Truncate path to one less than saved depth, then push saved node
                    path_stack.truncate(depth - 1);
                    path_stack.push(node_ptr);
                }
            }

            Token::WhiteWins => {
                tree.result = GameResult::WhiteWins;
            }
            Token::BlackWins => {
                tree.result = GameResult::BlackWins;
            }
            Token::Draw => {
                tree.result = GameResult::Draw;
            }
            Token::Ongoing => {
                tree.result = GameResult::Ongoing;
            }

            Token::Newline => {
                // Newlines are generally ignored
            }
        }
    }

    Ok(tree)
}

/// Parse a header string like [Event "Test Game"] into (key, value)
fn parse_header(header: &str) -> Option<(String, String)> {
    let header = header.trim();
    if !header.starts_with('[') || !header.ends_with(']') {
        return None;
    }

    let inner = &header[1..header.len() - 1];
    let quote_start = inner.find('"')?;
    let quote_end = inner.rfind('"')?;

    if quote_start >= quote_end {
        return None;
    }

    let key = inner[..quote_start].trim().to_string();
    let value = inner[quote_start + 1..quote_end].to_string();

    Some((key, value))
}

/// Parse a move number string like "1." or "15..." into the number
fn parse_move_number(s: &str) -> Option<u16> {
    let num_part: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_part.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenize;

    #[test]
    fn test_parse_header() {
        let (key, value) = parse_header(r#"[Event "Test Game"]"#).unwrap();
        assert_eq!(key, "Event");
        assert_eq!(value, "Test Game");
    }

    #[test]
    fn test_parse_move_number() {
        assert_eq!(parse_move_number("1."), Some(1));
        assert_eq!(parse_move_number("15..."), Some(15));
        assert_eq!(parse_move_number("42."), Some(42));
    }

    #[test]
    fn test_build_simple_game() {
        let tokens = tokenize("1. e4 e5 2. Nf3 Nc6");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.move_number, Some(1));
        assert!(!e4.is_black);

        let e5 = e4.find_child("e5").unwrap();
        assert_eq!(e5.move_number, Some(1));
        assert!(e5.is_black);
    }

    #[test]
    fn test_build_with_headers() {
        let tokens = tokenize(r#"[Event "Test"] [White "Alice"] 1. e4"#);
        let tree = build_tree(&tokens).unwrap();

        assert_eq!(tree.header("Event"), Some("Test"));
        assert_eq!(tree.header("White"), Some("Alice"));
    }

    #[test]
    fn test_build_with_comments() {
        let tokens = tokenize("1. e4 {Best move!} e5");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert_eq!(e4.comment.trim(), "Best move!");
    }

    #[test]
    fn test_build_with_nags() {
        let tokens = tokenize("1. e4! e5?? 2. Nf3 $14");
        let tree = build_tree(&tokens).unwrap();

        let e4 = tree.root.find_child("e4").unwrap();
        assert!(e4.nags.contains(&Nag::GOOD_MOVE));

        let e5 = e4.find_child("e5").unwrap();
        assert!(e5.nags.contains(&Nag::BLUNDER));
    }
}
