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
    use crate::nag::Nag;

    // ========================================================================
    // Root Node Tests
    // ========================================================================

    #[test]
    fn test_root_node() {
        let root = GameNode::root();
        assert!(root.is_root());
        assert!(root.san.is_empty());
    }

    #[test]
    fn test_root_default() {
        let root = GameNode::default();
        assert!(root.is_root());
    }

    #[test]
    fn test_non_root_node() {
        let node = GameNode::new("e4");
        assert!(!node.is_root());
        assert_eq!(node.san, "e4");
    }

    #[test]
    fn test_node_with_move_number_not_root() {
        let mut node = GameNode::new("");
        node.move_number = Some(1);
        // Empty san but with move number - not root
        assert!(!node.is_root());
    }

    // ========================================================================
    // Move Text Tests
    // ========================================================================

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
    fn test_move_text_no_move_number() {
        let node = GameNode::new("Nf3");
        assert_eq!(node.move_text(), "Nf3");
    }

    #[test]
    fn test_move_text_empty_san() {
        let root = GameNode::root();
        assert_eq!(root.move_text(), "");
    }

    #[test]
    fn test_move_text_high_move_number() {
        let mut node = GameNode::new("Kf1");
        node.move_number = Some(100);
        node.is_black = false;
        assert_eq!(node.move_text(), "100. Kf1");
    }

    #[test]
    fn test_move_text_castling() {
        let mut node = GameNode::new("O-O");
        node.move_number = Some(5);
        assert_eq!(node.move_text(), "5. O-O");

        node.san = "O-O-O".to_string();
        assert_eq!(node.move_text(), "5. O-O-O");
    }

    // ========================================================================
    // Find Child Tests
    // ========================================================================

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
    fn test_find_child_with_move_number() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));

        // Should normalize and find
        assert!(root.find_child("1. e4").is_some());
        assert!(root.find_child("e4").is_some());
    }

    #[test]
    fn test_find_child_mut() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));

        let e4 = root.find_child_mut("e4").unwrap();
        e4.comment = "Test comment".to_string();

        assert_eq!(root.find_child("e4").unwrap().comment, "Test comment");
    }

    #[test]
    fn test_find_child_empty_children() {
        let root = GameNode::root();
        assert!(root.find_child("e4").is_none());
    }

    // ========================================================================
    // Find Path Tests
    // ========================================================================

    #[test]
    fn test_find_path() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        let result = root.find_path(&["e4", "e5", "Nf3"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().san, "Nf3");
    }

    #[test]
    fn test_find_path_empty() {
        let root = GameNode::root();
        let result = root.find_path(&[]);
        assert!(result.is_some());
        assert!(result.unwrap().is_root());
    }

    #[test]
    fn test_find_path_nonexistent() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));

        let result = root.find_path(&["d4"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_path_partial_match() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let result = root.find_path(&["e4", "e5", "Nf3"]);
        assert!(result.is_none()); // Nf3 doesn't exist
    }

    #[test]
    fn test_find_path_mut() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let e5 = root.find_path_mut(&["e4", "e5"]).unwrap();
        e5.comment = "Modified".to_string();

        assert_eq!(root.find_path(&["e4", "e5"]).unwrap().comment, "Modified");
    }

    // ========================================================================
    // Main Line and Variations Tests
    // ========================================================================

    #[test]
    fn test_main_line() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));

        let main = e4.main_line();
        assert!(main.is_some());
        assert_eq!(main.unwrap().san, "e5");
    }

    #[test]
    fn test_main_line_empty() {
        let root = GameNode::root();
        assert!(root.main_line().is_none());
    }

    #[test]
    fn test_variations() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        e4.add_child(GameNode::new("e6"));

        let vars = e4.variations();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].san, "c5");
        assert_eq!(vars[1].san, "e6");
    }

    #[test]
    fn test_variations_single_child() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        assert!(e4.variations().is_empty());
    }

    #[test]
    fn test_has_variations() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        assert!(!e4.has_variations());

        e4.add_child(GameNode::new("c5"));
        assert!(e4.has_variations());
    }

    #[test]
    fn test_has_children() {
        let mut root = GameNode::root();
        assert!(!root.has_children());

        root.add_child(GameNode::new("e4"));
        assert!(root.has_children());
    }

    #[test]
    fn test_is_leaf() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));

        assert!(e4.is_leaf());
        e4.add_child(GameNode::new("e5"));
        assert!(!e4.is_leaf());
    }

    // ========================================================================
    // Counting Tests
    // ========================================================================

    #[test]
    fn test_count_nodes() {
        let mut root = GameNode::root();
        let child1 = root.add_child(GameNode::new("e4"));
        child1.add_child(GameNode::new("e5"));
        root.add_child(GameNode::new("d4"));

        // root + e4 + e5 + d4 = 4
        assert_eq!(root.count_nodes(), 4);
    }

    #[test]
    fn test_count_nodes_empty() {
        let root = GameNode::root();
        assert_eq!(root.count_nodes(), 1); // Just root
    }

    #[test]
    fn test_count_nodes_deep_tree() {
        let mut root = GameNode::root();
        let mut current = &mut root;
        for _ in 0..10 {
            current = current.add_child(GameNode::new("e4"));
        }
        assert_eq!(root.count_nodes(), 11); // root + 10 children
    }

    #[test]
    fn test_count_leaves() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        root.add_child(GameNode::new("d4"));

        // e5, c5, d4 are leaves
        assert_eq!(root.count_leaves(), 3);
    }

    #[test]
    fn test_count_leaves_single_line() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        assert_eq!(root.count_leaves(), 1);
    }

    #[test]
    fn test_count_comments() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.comment = "Opening move".to_string();
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.comment = "Reply".to_string();
        e4.add_child(GameNode::new("c5")); // No comment

        assert_eq!(root.count_comments(), 2);
    }

    #[test]
    fn test_count_comments_none() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));
        assert_eq!(root.count_comments(), 0);
    }

    // ========================================================================
    // Depth Tests
    // ========================================================================

    #[test]
    fn test_max_depth() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        let nf3 = e5.add_child(GameNode::new("Nf3"));
        nf3.add_child(GameNode::new("Nc6"));

        assert_eq!(root.max_depth(), 4);
    }

    #[test]
    fn test_max_depth_with_variations() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));
        // Shorter variation
        e4.add_child(GameNode::new("c5"));

        // Main line is deeper (4 vs 3)
        assert_eq!(root.max_depth(), 3);
    }

    #[test]
    fn test_max_depth_empty() {
        let root = GameNode::root();
        assert_eq!(root.max_depth(), 0);
    }

    #[test]
    fn test_main_line_length() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        assert_eq!(root.main_line_length(), 3);
    }

    #[test]
    fn test_main_line_length_with_variations() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));
        // Variation doesn't affect main line length
        e4.add_child(GameNode::new("c5"));

        assert_eq!(root.main_line_length(), 3);
    }

    // ========================================================================
    // Normalize SAN Tests
    // ========================================================================

    #[test]
    fn test_normalize_san() {
        assert_eq!(normalize_san("1. e4"), "e4");
        assert_eq!(normalize_san("1...e5"), "e5");
        assert_eq!(normalize_san("Nf3"), "Nf3");
        assert_eq!(normalize_san("O-O"), "O-O");
    }

    #[test]
    fn test_normalize_san_high_move_number() {
        assert_eq!(normalize_san("100. Kf1"), "Kf1");
        assert_eq!(normalize_san("50...Kf8"), "Kf8");
    }

    #[test]
    fn test_normalize_san_with_check() {
        assert_eq!(normalize_san("15. Qxf7+"), "Qxf7+");
        assert_eq!(normalize_san("20. Qxf7#"), "Qxf7#");
    }

    #[test]
    fn test_normalize_san_promotion() {
        assert_eq!(normalize_san("8. e8=Q+"), "e8=Q+");
    }

    #[test]
    fn test_normalize_san_whitespace() {
        assert_eq!(normalize_san("  e4  "), "e4");
        assert_eq!(normalize_san("  1. e4  "), "e4");
    }

    // ========================================================================
    // Add Child Tests
    // ========================================================================

    #[test]
    fn test_add_child() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.comment = "Test".to_string();

        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].comment, "Test");
    }

    #[test]
    fn test_add_multiple_children() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));
        root.add_child(GameNode::new("d4"));
        root.add_child(GameNode::new("c4"));

        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].san, "e4");
        assert_eq!(root.children[1].san, "d4");
        assert_eq!(root.children[2].san, "c4");
    }

    // ========================================================================
    // Deep Clone Tests
    // ========================================================================

    #[test]
    fn test_deep_clone() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.comment = "Opening".to_string();
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));

        let clone = root.deep_clone();

        assert_eq!(clone.children.len(), 1);
        assert_eq!(clone.children[0].san, "e4");
        assert_eq!(clone.children[0].comment, "Opening");
        assert_eq!(clone.children[0].children.len(), 2);
    }

    #[test]
    fn test_deep_clone_independence() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.comment = "Original".to_string();

        let mut clone = root.deep_clone();
        clone.children[0].comment = "Modified".to_string();

        // Original unchanged
        assert_eq!(root.children[0].comment, "Original");
        assert_eq!(clone.children[0].comment, "Modified");
    }

    #[test]
    fn test_deep_clone_empty() {
        let root = GameNode::root();
        let clone = root.deep_clone();
        assert!(clone.is_root());
        assert!(clone.children.is_empty());
    }

    // ========================================================================
    // NAG Tests
    // ========================================================================

    #[test]
    fn test_node_with_nags() {
        let mut node = GameNode::new("e4");
        node.nags.push(Nag(1)); // Good move
        node.nags.push(Nag(14)); // White has slight advantage

        assert_eq!(node.nags.len(), 2);
        assert_eq!(node.nags[0], Nag(1));
    }

    #[test]
    fn test_deep_clone_preserves_nags() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.nags.push(Nag(1));

        let clone = root.deep_clone();
        assert_eq!(clone.children[0].nags.len(), 1);
        assert_eq!(clone.children[0].nags[0], Nag(1));
    }
}
