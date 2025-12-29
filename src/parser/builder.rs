//! Tree builder - constructs GameTree from token stream

use super::token::Token;
use crate::error::Result;
use crate::nag::Nag;
use crate::tree::{GameNode, GameResult, GameTree};

/// A path from the root to a specific node in the tree.
/// Each element is a child index at that level.
/// Empty path means root node.
type NodePath = Vec<usize>;

/// Build a GameTree from a token stream
pub fn build_tree(tokens: &[Token]) -> Result<GameTree> {
    let mut tree = GameTree::new();

    // Track the path from root to current position using indices instead of raw pointers.
    // This avoids undefined behavior from Vec reallocation invalidating pointers.
    // path[i] is the child index at depth i (path[0] is index of first move under root)
    let mut current_path: NodePath = Vec::new();

    // Stack for tracking where to return after a variation ends
    // Each entry is (path_to_node, move_number, expect_black) to restore
    let mut return_stack: Vec<(NodePath, u16, bool)> = Vec::new();

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

                // Navigate to parent node and add child
                let parent = get_node_mut(&mut tree.root, &current_path);
                parent.children.push(node);
                let new_child_idx = parent.children.len() - 1;
                current_path.push(new_child_idx);

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

                // current_path.is_empty() means we're at root (no moves yet)
                if current_path.is_empty() {
                    if !pending_comment.is_empty() {
                        pending_comment.push(' ');
                    }
                    pending_comment.push_str(text);
                } else {
                    // Get the current move node
                    let current = get_node_mut(&mut tree.root, &current_path);
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
                if current_path.is_empty() {
                    if !pending_comment.is_empty() {
                        pending_comment.push(' ');
                    }
                    pending_comment.push_str(text);
                } else {
                    let current = get_node_mut(&mut tree.root, &current_path);
                    if !current.comment.is_empty() {
                        current.comment.push(' ');
                    }
                    current.comment.push_str(text);
                }
            }

            Token::Nag(nag_str) => {
                if let Some(nag) = Nag::from_dollar_notation(nag_str) {
                    if current_path.is_empty() {
                        pending_nags.push(nag);
                    } else {
                        let current = get_node_mut(&mut tree.root, &current_path);
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
                    if current_path.is_empty() {
                        pending_nags.push(nag);
                    } else {
                        let current = get_node_mut(&mut tree.root, &current_path);
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
                    if current_path.is_empty() {
                        pending_nags.push(nag);
                    } else {
                        let current = get_node_mut(&mut tree.root, &current_path);
                        current.nags.push(nag);
                    }
                }
            }

            Token::VariationStart => {
                // Variation is an alternative to the preceding move
                // Push current position AND move tracking state to restore later
                if !current_path.is_empty() {
                    return_stack.push((
                        current_path.clone(),
                        current_move_number,
                        expect_black,
                    ));
                    // Pop the current move to go back to its parent
                    // The variation's moves will be siblings of the current move
                    current_path.pop();
                }
            }

            Token::VariationEnd => {
                // Restore to the position we saved at VariationStart
                if let Some((saved_path, saved_move_num, saved_expect_black)) =
                    return_stack.pop()
                {
                    current_path = saved_path;
                    current_move_number = saved_move_num;
                    expect_black = saved_expect_black;
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

/// Navigate to a node given a path of child indices
fn get_node_mut<'a>(root: &'a mut GameNode, path: &[usize]) -> &'a mut GameNode {
    let mut current = root;
    for &idx in path {
        current = &mut current.children[idx];
    }
    current
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
