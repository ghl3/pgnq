//! Node path parsing and resolution for navigating game trees

use crate::error::Error;
use crate::tree::{GameNode, GameTree};

/// A segment in a node path
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// A specific move, optionally with variation index
    Move {
        san: String,
        variation_index: Option<usize>,
    },
    /// Root node selector
    Root,
    /// Follow main line to end
    End,
    /// Select specific variation by index
    Variation(usize),
    /// All descendants (glob)
    AllDescendants,
    /// Direct children only
    DirectChildren,
}

/// A parsed node path for navigating game trees
#[derive(Debug, Clone)]
pub struct NodePath {
    pub segments: Vec<PathSegment>,
}

impl NodePath {
    /// Parse a path string like "e4/e5/Nf3" or "e4/c5:1"
    pub fn parse(s: &str) -> Result<Self, Error> {
        let s = s.trim();
        if s.is_empty() {
            return Err(Error::InvalidPath("empty path".to_string()));
        }

        // Handle special selectors
        if s == "@root" {
            return Ok(NodePath {
                segments: vec![PathSegment::Root],
            });
        }

        let mut segments = Vec::new();
        let parts: Vec<&str> = s.split('/').collect();

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Check for special selectors
            match part {
                "**" => {
                    segments.push(PathSegment::AllDescendants);
                    continue;
                }
                "*" => {
                    segments.push(PathSegment::DirectChildren);
                    continue;
                }
                "@end" => {
                    segments.push(PathSegment::End);
                    continue;
                }
                "@root" => {
                    segments.push(PathSegment::Root);
                    continue;
                }
                _ => {}
            }

            if let Some(rest) = part.strip_prefix("@var") {
                let idx: usize = rest
                    .trim()
                    .parse()
                    .map_err(|_| Error::InvalidPath(format!("invalid variation index: {}", part)))?;
                segments.push(PathSegment::Variation(idx));
                continue;
            }

            // Parse as move with optional variation suffix
            segments.push(parse_move_segment(part)?);
        }

        Ok(NodePath { segments })
    }

    /// Resolve this path against a game tree, returning the first matching node
    pub fn resolve<'a>(&self, tree: &'a GameTree) -> Option<&'a GameNode> {
        self.resolve_from(&tree.root)
    }

    /// Resolve this path starting from a specific node
    pub fn resolve_from<'a>(&self, start: &'a GameNode) -> Option<&'a GameNode> {
        let mut current = start;

        for segment in &self.segments {
            match segment {
                PathSegment::Root => {
                    // Can't go to root from here, just stay at current
                }
                PathSegment::Move {
                    san,
                    variation_index,
                } => {
                    if let Some(idx) = variation_index {
                        // Select specific variation by index
                        current = current.children.get(*idx)?;
                        // Verify the move matches
                        if !moves_match(&current.san, san) {
                            return None;
                        }
                    } else {
                        // Find child by move
                        current = current.find_child(san)?;
                    }
                }
                PathSegment::End => {
                    // Follow main line to end
                    while let Some(child) = current.main_line() {
                        current = child;
                    }
                }
                PathSegment::Variation(idx) => {
                    current = current.children.get(*idx)?;
                }
                PathSegment::AllDescendants | PathSegment::DirectChildren => {
                    // These are for collecting multiple nodes, not single resolution
                    // For single resolution, just return current
                }
            }
        }

        Some(current)
    }

    /// Resolve this path and return all matching nodes
    pub fn resolve_all<'a>(&self, tree: &'a GameTree) -> Vec<&'a GameNode> {
        self.resolve_all_from(&tree.root)
    }

    /// Resolve all matching nodes starting from a specific node
    pub fn resolve_all_from<'a>(&self, start: &'a GameNode) -> Vec<&'a GameNode> {
        let mut current = vec![start];

        for segment in &self.segments {
            current = match segment {
                PathSegment::Root => vec![start],
                PathSegment::Move {
                    san,
                    variation_index,
                } => {
                    current
                        .into_iter()
                        .filter_map(|node| {
                            if let Some(idx) = variation_index {
                                node.children.get(*idx).filter(|c| moves_match(&c.san, san))
                            } else {
                                node.find_child(san)
                            }
                        })
                        .collect()
                }
                PathSegment::End => {
                    current
                        .into_iter()
                        .map(|node| {
                            let mut n = node;
                            while let Some(child) = n.main_line() {
                                n = child;
                            }
                            n
                        })
                        .collect()
                }
                PathSegment::Variation(idx) => {
                    current
                        .into_iter()
                        .filter_map(|node| node.children.get(*idx))
                        .collect()
                }
                PathSegment::AllDescendants => {
                    // Collect all descendants
                    let mut all = Vec::new();
                    for node in current {
                        let mut stack = vec![node];
                        while let Some(n) = stack.pop() {
                            all.push(n);
                            stack.extend(n.children.iter());
                        }
                    }
                    all
                }
                PathSegment::DirectChildren => {
                    current
                        .into_iter()
                        .flat_map(|node| node.children.iter())
                        .collect()
                }
            };

            if current.is_empty() {
                break;
            }
        }

        current
    }
}

/// Parse a move segment like "e4", "1.e4", "e4:1", "Nf3:v2"
fn parse_move_segment(s: &str) -> Result<PathSegment, Error> {
    // Check for variation suffix like ":1" or ":v1"
    let (move_part, var_idx) = if let Some(colon_idx) = s.rfind(':') {
        let var_str = &s[colon_idx + 1..];
        let var_str = var_str.strip_prefix('v').unwrap_or(var_str);
        let idx: usize = var_str
            .parse()
            .map_err(|_| Error::InvalidPath(format!("invalid variation index in: {}", s)))?;
        (&s[..colon_idx], Some(idx))
    } else {
        (s, None)
    };

    // Strip move number prefix if present
    let san = strip_move_number(move_part);

    Ok(PathSegment::Move {
        san,
        variation_index: var_idx,
    })
}

/// Strip move number prefix like "1." or "1..." from a move
fn strip_move_number(s: &str) -> String {
    let s = s.trim();
    // Find first alphabetic char or O (for castling)
    if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == 'O') {
        s[idx..].to_string()
    } else {
        s.to_string()
    }
}

/// Check if two moves match (normalized comparison)
fn moves_match(san1: &str, san2: &str) -> bool {
    strip_move_number(san1) == strip_move_number(san2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Basic Path Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_path() {
        let path = NodePath::parse("e4/e5/Nf3").unwrap();
        assert_eq!(path.segments.len(), 3);
    }

    #[test]
    fn test_parse_single_move() {
        let path = NodePath::parse("e4").unwrap();
        assert_eq!(path.segments.len(), 1);
        match &path.segments[0] {
            PathSegment::Move { san, variation_index } => {
                assert_eq!(san, "e4");
                assert_eq!(*variation_index, None);
            }
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_parse_long_path() {
        let path = NodePath::parse("e4/e5/Nf3/Nc6/Bb5/a6/Ba4/Nf6/O-O/Be7").unwrap();
        assert_eq!(path.segments.len(), 10);
    }

    #[test]
    fn test_parse_with_variation() {
        let path = NodePath::parse("e4/c5:1").unwrap();
        assert_eq!(path.segments.len(), 2);
        match &path.segments[1] {
            PathSegment::Move {
                san,
                variation_index,
            } => {
                assert_eq!(san, "c5");
                assert_eq!(*variation_index, Some(1));
            }
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_parse_variation_with_v_prefix() {
        let path = NodePath::parse("e4/c5:v2").unwrap();
        match &path.segments[1] {
            PathSegment::Move { variation_index, .. } => {
                assert_eq!(*variation_index, Some(2));
            }
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_parse_castling_in_path() {
        let path = NodePath::parse("e4/e5/Nf3/Nc6/Bb5/a6/Ba4/Nf6/O-O").unwrap();
        assert_eq!(path.segments.len(), 9);
        match &path.segments[8] {
            PathSegment::Move { san, .. } => assert_eq!(san, "O-O"),
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_parse_queenside_castling() {
        let path = NodePath::parse("e4/e5/O-O-O").unwrap();
        match &path.segments[2] {
            PathSegment::Move { san, .. } => assert_eq!(san, "O-O-O"),
            _ => panic!("expected Move segment"),
        }
    }

    // ========================================================================
    // Special Selector Tests
    // ========================================================================

    #[test]
    fn test_parse_special_selectors() {
        let path = NodePath::parse("@root").unwrap();
        assert_eq!(path.segments, vec![PathSegment::Root]);

        let path = NodePath::parse("e4/@end").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[1], PathSegment::End);

        let path = NodePath::parse("e4/**").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[1], PathSegment::AllDescendants);
    }

    #[test]
    fn test_parse_direct_children_selector() {
        let path = NodePath::parse("e4/*").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[1], PathSegment::DirectChildren);
    }

    #[test]
    fn test_parse_variation_selector() {
        let path = NodePath::parse("e4/@var1").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[1], PathSegment::Variation(1));
    }

    #[test]
    fn test_parse_root_in_middle() {
        let path = NodePath::parse("e4/@root/d4").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[1], PathSegment::Root);
    }

    #[test]
    fn test_parse_multiple_special_selectors() {
        let path = NodePath::parse("e4/**/@end").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[1], PathSegment::AllDescendants);
        assert_eq!(path.segments[2], PathSegment::End);
    }

    // ========================================================================
    // Error Case Tests
    // ========================================================================

    #[test]
    fn test_parse_empty_path_fails() {
        let result = NodePath::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only_fails() {
        let result = NodePath::parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_variation_index() {
        let result = NodePath::parse("e4/c5:abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_var_selector() {
        let result = NodePath::parse("e4/@varXYZ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_negative_variation_index() {
        // Negative numbers should fail to parse as usize
        let result = NodePath::parse("e4/c5:-1");
        assert!(result.is_err());
    }

    // ========================================================================
    // Move Number Stripping Tests
    // ========================================================================

    #[test]
    fn test_strip_move_number() {
        assert_eq!(strip_move_number("1. e4"), "e4");
        assert_eq!(strip_move_number("1...e5"), "e5");
        assert_eq!(strip_move_number("Nf3"), "Nf3");
        assert_eq!(strip_move_number("O-O"), "O-O");
    }

    #[test]
    fn test_strip_move_number_various_formats() {
        assert_eq!(strip_move_number("1.e4"), "e4");
        assert_eq!(strip_move_number("1. e4"), "e4");
        assert_eq!(strip_move_number("1 e4"), "e4");
        assert_eq!(strip_move_number("10. e4"), "e4");
        assert_eq!(strip_move_number("100. e4"), "e4");
        assert_eq!(strip_move_number("10...e5"), "e5");
    }

    #[test]
    fn test_strip_move_number_castling() {
        assert_eq!(strip_move_number("5. O-O"), "O-O");
        assert_eq!(strip_move_number("10. O-O-O"), "O-O-O");
    }

    #[test]
    fn test_strip_move_number_with_check() {
        assert_eq!(strip_move_number("15. Qxf7+"), "Qxf7+");
        assert_eq!(strip_move_number("20. Qxf7#"), "Qxf7#");
    }

    #[test]
    fn test_moves_match() {
        assert!(moves_match("e4", "e4"));
        assert!(moves_match("1. e4", "e4"));
        assert!(moves_match("e4", "1. e4"));
        assert!(moves_match("1. e4", "1. e4"));
        assert!(moves_match("1...e5", "e5"));
        assert!(!moves_match("e4", "e5"));
    }

    // ========================================================================
    // Path Resolution Tests
    // ========================================================================

    #[test]
    fn test_resolve_path() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));

        let path = NodePath::parse("e4/e5").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "e5");

        // Test variation selector
        let path = NodePath::parse("e4/c5:1").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "c5");
    }

    #[test]
    fn test_resolve_nonexistent_path() {
        let mut tree = GameTree::new();
        tree.root.add_child(GameNode::new("e4"));

        let path = NodePath::parse("d4").unwrap();
        let result = path.resolve(&tree);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_partial_path() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let path = NodePath::parse("e4/e5/Nf3").unwrap();
        let result = path.resolve(&tree);
        assert!(result.is_none()); // Nf3 doesn't exist
    }

    #[test]
    fn test_resolve_end_selector() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        let nf3 = e5.add_child(GameNode::new("Nf3"));
        nf3.add_child(GameNode::new("Nc6"));

        let path = NodePath::parse("e4/@end").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "Nc6");
    }

    #[test]
    fn test_resolve_variation_by_index() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        e4.add_child(GameNode::new("e6"));

        let path = NodePath::parse("e4/@var0").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "e5");

        let path = NodePath::parse("e4/@var1").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "c5");

        let path = NodePath::parse("e4/@var2").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "e6");
    }

    #[test]
    fn test_resolve_variation_out_of_bounds() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let path = NodePath::parse("e4/@var5").unwrap();
        let result = path.resolve(&tree);
        assert!(result.is_none());
    }

    // ========================================================================
    // Resolve All Tests
    // ========================================================================

    #[test]
    fn test_resolve_all_direct_children() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        e4.add_child(GameNode::new("e6"));

        let path = NodePath::parse("e4/*").unwrap();
        let nodes = path.resolve_all(&tree);
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn test_resolve_all_descendants() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        let nf3 = e5.add_child(GameNode::new("Nf3"));
        nf3.add_child(GameNode::new("Nc6"));

        let path = NodePath::parse("e4/**").unwrap();
        let nodes = path.resolve_all(&tree);
        // Should include e4 and all its descendants
        assert_eq!(nodes.len(), 4); // e4, e5, Nf3, Nc6
    }

    #[test]
    fn test_resolve_all_with_variations() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));
        let c5 = e4.add_child(GameNode::new("c5"));
        c5.add_child(GameNode::new("Nf3"));

        let path = NodePath::parse("e4/**").unwrap();
        let nodes = path.resolve_all(&tree);
        // e4, e5, Nf3, c5, Nf3
        assert_eq!(nodes.len(), 5);
    }

    #[test]
    fn test_resolve_all_empty_result() {
        let tree = GameTree::new();

        let path = NodePath::parse("e4/*").unwrap();
        let nodes = path.resolve_all(&tree);
        assert!(nodes.is_empty());
    }

    // ========================================================================
    // Complex Path Tests
    // ========================================================================

    #[test]
    fn test_resolve_complex_path() {
        use crate::parser::parse;
        // Use parser to build tree to avoid borrow issues
        let tree = parse("1. e4 e5 (1... c5 2. Nf3 d6) 2. Nf3 Nc6 *").unwrap();

        // Navigate to sicilian variation
        let path = NodePath::parse("e4/c5:1/Nf3/d6").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "d6");
    }

    #[test]
    fn test_resolve_path_with_whitespace() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let path = NodePath::parse("  e4 / e5  ").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "e5");
    }

    #[test]
    fn test_resolve_path_empty_segments() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        // Empty segments should be skipped
        let path = NodePath::parse("e4//e5").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert_eq!(node.san, "e5");
    }

    // ========================================================================
    // PathSegment Equality Tests
    // ========================================================================

    #[test]
    fn test_path_segment_equality() {
        assert_eq!(PathSegment::Root, PathSegment::Root);
        assert_eq!(PathSegment::End, PathSegment::End);
        assert_eq!(PathSegment::AllDescendants, PathSegment::AllDescendants);
        assert_eq!(PathSegment::DirectChildren, PathSegment::DirectChildren);
        assert_eq!(PathSegment::Variation(1), PathSegment::Variation(1));
        assert_ne!(PathSegment::Variation(1), PathSegment::Variation(2));
    }

    #[test]
    fn test_path_segment_move_equality() {
        let move1 = PathSegment::Move {
            san: "e4".to_string(),
            variation_index: None,
        };
        let move2 = PathSegment::Move {
            san: "e4".to_string(),
            variation_index: None,
        };
        let move3 = PathSegment::Move {
            san: "e5".to_string(),
            variation_index: None,
        };
        assert_eq!(move1, move2);
        assert_ne!(move1, move3);
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_resolve_from_non_root() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        let nf3 = e5.add_child(GameNode::new("Nf3"));
        nf3.add_child(GameNode::new("Nc6"));

        // Resolve starting from e5
        let path = NodePath::parse("Nf3/Nc6").unwrap();
        let node = path.resolve_from(e5).unwrap();
        assert_eq!(node.san, "Nc6");
    }

    #[test]
    fn test_resolve_all_from_non_root() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));
        e5.add_child(GameNode::new("Bc4"));

        // Get all children of e5
        let path = NodePath::parse("*").unwrap();
        let nodes = path.resolve_all_from(e5);
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_path_with_promotion() {
        let path = NodePath::parse("e8=Q").unwrap();
        match &path.segments[0] {
            PathSegment::Move { san, .. } => assert_eq!(san, "e8=Q"),
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_path_with_check() {
        let path = NodePath::parse("Qxf7+/Ke8").unwrap();
        assert_eq!(path.segments.len(), 2);
        match &path.segments[0] {
            PathSegment::Move { san, .. } => assert_eq!(san, "Qxf7+"),
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_path_with_checkmate() {
        let path = NodePath::parse("Qxf7#").unwrap();
        match &path.segments[0] {
            PathSegment::Move { san, .. } => assert_eq!(san, "Qxf7#"),
            _ => panic!("expected Move segment"),
        }
    }

    #[test]
    fn test_resolve_mismatched_variation_index() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));

        // Index 0 should be e5, not c5
        let path = NodePath::parse("e4/c5:0").unwrap();
        let result = path.resolve(&tree);
        assert!(result.is_none()); // c5 is at index 1, not 0
    }

    #[test]
    fn test_end_on_empty_tree() {
        let tree = GameTree::new();

        let path = NodePath::parse("@end").unwrap();
        let node = path.resolve(&tree).unwrap();
        assert!(node.is_root());
    }

    #[test]
    fn test_all_descendants_from_root() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        let path = NodePath::parse("**").unwrap();
        let nodes = path.resolve_all(&tree);
        // Root + e4 + e5 + Nf3
        assert_eq!(nodes.len(), 4);
    }
}
