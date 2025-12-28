//! GameNode - represents a single position/move in the game tree

use crate::nag::Nag;
use serde::{Deserialize, Serialize};

/// A single node in the game tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameNode {
    /// The move in SAN notation (e.g., "e4", "Nf3", "O-O")
    /// Empty string for root node
    pub san: String,

    /// Move number (1, 2, 3, ...)
    pub move_number: Option<u16>,

    /// True if this is Black's move
    pub is_black: bool,

    /// Human-readable comment/annotation
    pub comment: String,

    /// Numeric Annotation Glyphs
    pub nags: Vec<Nag>,

    /// Child nodes (first child is main line, rest are variations)
    pub children: Vec<GameNode>,
}

impl GameNode {
    /// Create a new empty root node
    pub fn root() -> Self {
        Self {
            san: String::new(),
            move_number: None,
            is_black: false,
            comment: String::new(),
            nags: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create a new move node
    pub fn new(san: impl Into<String>) -> Self {
        Self {
            san: san.into(),
            move_number: None,
            is_black: false,
            comment: String::new(),
            nags: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Check if this is the root node
    pub fn is_root(&self) -> bool {
        self.san.is_empty() && self.move_number.is_none()
    }

    /// Get the first (main line) child
    pub fn main_line(&self) -> Option<&GameNode> {
        self.children.first()
    }

    /// Get mutable reference to main line child
    pub fn main_line_mut(&mut self) -> Option<&mut GameNode> {
        self.children.first_mut()
    }

    /// Get variation children (all except first)
    pub fn variations(&self) -> &[GameNode] {
        if self.children.len() > 1 {
            &self.children[1..]
        } else {
            &[]
        }
    }

    /// Check if node has alternative variations
    pub fn has_variations(&self) -> bool {
        self.children.len() > 1
    }

    /// Check if node has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Format as "1. e4" or "1... e5"
    pub fn move_text(&self) -> String {
        if self.san.is_empty() {
            return String::new();
        }
        match self.move_number {
            Some(n) if self.is_black => format!("{}... {}", n, self.san),
            Some(n) => format!("{}. {}", n, self.san),
            None => self.san.clone(),
        }
    }

    /// Find a child by SAN move (normalized comparison)
    pub fn find_child(&self, san: &str) -> Option<&GameNode> {
        let normalized = normalize_san(san);
        self.children
            .iter()
            .find(|c| normalize_san(&c.san) == normalized)
    }

    /// Find a child by SAN move (mutable)
    pub fn find_child_mut(&mut self, san: &str) -> Option<&mut GameNode> {
        let normalized = normalize_san(san);
        self.children
            .iter_mut()
            .find(|c| normalize_san(&c.san) == normalized)
    }

    /// Navigate to a node following a path of moves
    pub fn find_path(&self, path: &[&str]) -> Option<&GameNode> {
        let mut current = self;
        for san in path {
            current = current.find_child(san)?;
        }
        Some(current)
    }

    /// Navigate to a node following a path of moves (mutable)
    pub fn find_path_mut(&mut self, path: &[&str]) -> Option<&mut GameNode> {
        let mut current = self;
        for san in path {
            current = current.find_child_mut(san)?;
        }
        Some(current)
    }

    /// Count all nodes in this subtree (iterative to avoid stack overflow)
    pub fn count_nodes(&self) -> usize {
        let mut count = 0;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            count += 1;
            stack.extend(node.children.iter());
        }
        count
    }

    /// Count leaf nodes (lines) in this subtree
    pub fn count_leaves(&self) -> usize {
        let mut count = 0;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if node.children.is_empty() {
                count += 1;
            } else {
                stack.extend(node.children.iter());
            }
        }
        count
    }

    /// Count nodes with comments
    pub fn count_comments(&self) -> usize {
        let mut count = 0;
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if !node.comment.is_empty() {
                count += 1;
            }
            stack.extend(node.children.iter());
        }
        count
    }

    /// Get maximum depth of this subtree
    pub fn max_depth(&self) -> usize {
        let mut max = 0;
        let mut stack = vec![(self, 0usize)];
        while let Some((node, depth)) = stack.pop() {
            max = max.max(depth);
            for child in &node.children {
                stack.push((child, depth + 1));
            }
        }
        max
    }

    /// Get the length of the main line from this node
    pub fn main_line_length(&self) -> usize {
        let mut length = 0;
        let mut current = self;
        while let Some(child) = current.main_line() {
            length += 1;
            current = child;
        }
        length
    }

    /// Add a child node and return mutable reference to it
    pub fn add_child(&mut self, child: GameNode) -> &mut GameNode {
        self.children.push(child);
        self.children.last_mut().unwrap()
    }

    /// Create a deep copy of this subtree
    pub fn deep_clone(&self) -> Self {
        // Use iterative approach to handle deep trees
        let mut result = GameNode {
            san: self.san.clone(),
            move_number: self.move_number,
            is_black: self.is_black,
            comment: self.comment.clone(),
            nags: self.nags.clone(),
            children: Vec::with_capacity(self.children.len()),
        };

        // Stack of (source node, target parent pointer, child index to process)
        let mut stack: Vec<(&GameNode, *mut GameNode, usize)> = vec![];

        // Initialize with children of self
        for (i, child) in self.children.iter().enumerate().rev() {
            stack.push((child, &mut result as *mut GameNode, i));
        }

        while let Some((source, parent_ptr, _idx)) = stack.pop() {
            let child_copy = GameNode {
                san: source.san.clone(),
                move_number: source.move_number,
                is_black: source.is_black,
                comment: source.comment.clone(),
                nags: source.nags.clone(),
                children: Vec::with_capacity(source.children.len()),
            };

            // SAFETY: We control the parent pointer lifetime
            let parent = unsafe { &mut *parent_ptr };
            parent.children.push(child_copy);
            let new_child_ptr = parent.children.last_mut().unwrap() as *mut GameNode;

            // Add children to process
            for (i, grandchild) in source.children.iter().enumerate().rev() {
                stack.push((grandchild, new_child_ptr, i));
            }
        }

        result
    }
}

impl Default for GameNode {
    fn default() -> Self {
        Self::root()
    }
}

/// Normalize SAN for comparison (strip move numbers, whitespace)
fn normalize_san(san: &str) -> String {
    let s = san.trim();
    // Strip move number prefix like "1." or "1..."
    if let Some(idx) = s.find(|c: char| c.is_ascii_alphabetic() || c == 'O') {
        s[idx..].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_node() {
        let root = GameNode::root();
        assert!(root.is_root());
        assert!(root.san.is_empty());
    }

    #[test]
    fn test_move_text() {
        let mut node = GameNode::new("e4");
        node.move_number = Some(1);
        node.is_black = false;
        assert_eq!(node.move_text(), "1. e4");

        node.is_black = true;
        node.san = "e5".to_string();
        assert_eq!(node.move_text(), "1... e5");
    }

    #[test]
    fn test_find_child() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));
        root.add_child(GameNode::new("d4"));

        assert!(root.find_child("e4").is_some());
        assert!(root.find_child("d4").is_some());
        assert!(root.find_child("c4").is_none());
    }

    #[test]
    fn test_normalize_san() {
        assert_eq!(normalize_san("1. e4"), "e4");
        assert_eq!(normalize_san("1...e5"), "e5");
        assert_eq!(normalize_san("Nf3"), "Nf3");
        assert_eq!(normalize_san("O-O"), "O-O");
    }

    #[test]
    fn test_count_nodes() {
        let mut root = GameNode::root();
        let child1 = root.add_child(GameNode::new("e4"));
        child1.add_child(GameNode::new("e5"));
        root.add_child(GameNode::new("d4"));

        // root + e4 + e5 + d4 = 4
        assert_eq!(root.count_nodes(), 4);
    }
}
