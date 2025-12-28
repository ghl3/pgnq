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

    #[test]
    fn test_parse_simple_path() {
        let path = NodePath::parse("e4/e5/Nf3").unwrap();
        assert_eq!(path.segments.len(), 3);
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
    fn test_strip_move_number() {
        assert_eq!(strip_move_number("1. e4"), "e4");
        assert_eq!(strip_move_number("1...e5"), "e5");
        assert_eq!(strip_move_number("Nf3"), "Nf3");
        assert_eq!(strip_move_number("O-O"), "O-O");
    }

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
}
