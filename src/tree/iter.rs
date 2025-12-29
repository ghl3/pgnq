//! Iterators for traversing game trees

use super::GameNode;

/// Depth-first iterator over tree nodes
pub struct DfsIter<'a> {
    stack: Vec<&'a GameNode>,
}

impl<'a> DfsIter<'a> {
    pub fn new(root: &'a GameNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for DfsIter<'a> {
    type Item = &'a GameNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse order for correct traversal
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

/// Iterator over main line only
pub struct MainLineIter<'a> {
    current: Option<&'a GameNode>,
}

impl<'a> MainLineIter<'a> {
    pub fn new(root: &'a GameNode) -> Self {
        Self {
            current: Some(root),
        }
    }
}

impl<'a> Iterator for MainLineIter<'a> {
    type Item = &'a GameNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.main_line();
        Some(node)
    }
}

/// Iterator with depth information
pub struct DepthIter<'a> {
    stack: Vec<(&'a GameNode, usize)>,
}

impl<'a> DepthIter<'a> {
    pub fn new(root: &'a GameNode) -> Self {
        Self {
            stack: vec![(root, 0)],
        }
    }
}

impl<'a> Iterator for DepthIter<'a> {
    type Item = (&'a GameNode, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, depth) = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push((child, depth + 1));
        }
        Some((node, depth))
    }
}

/// Iterator over leaf nodes only
pub struct LeavesIter<'a> {
    stack: Vec<&'a GameNode>,
}

impl<'a> LeavesIter<'a> {
    pub fn new(root: &'a GameNode) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for LeavesIter<'a> {
    type Item = &'a GameNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            if node.children.is_empty() {
                return Some(node);
            }
            self.stack.extend(node.children.iter().rev());
        }
        None
    }
}

impl GameNode {
    /// Iterate over all nodes depth-first
    pub fn iter_dfs(&self) -> DfsIter<'_> {
        DfsIter::new(self)
    }

    /// Iterate over main line only
    pub fn iter_main_line(&self) -> MainLineIter<'_> {
        MainLineIter::new(self)
    }

    /// Iterate with depth information
    pub fn iter_with_depth(&self) -> DepthIter<'_> {
        DepthIter::new(self)
    }

    /// Iterate over leaf nodes only
    pub fn iter_leaves(&self) -> LeavesIter<'_> {
        LeavesIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_tree() -> GameNode {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        root.add_child(GameNode::new("d4"));
        root
    }

    fn build_deep_tree() -> GameNode {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        let nf3 = e5.add_child(GameNode::new("Nf3"));
        let nc6 = nf3.add_child(GameNode::new("Nc6"));
        nc6.add_child(GameNode::new("Bb5"));
        root
    }

    fn build_wide_tree() -> GameNode {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));
        root.add_child(GameNode::new("d4"));
        root.add_child(GameNode::new("c4"));
        root.add_child(GameNode::new("Nf3"));
        root.add_child(GameNode::new("g3"));
        root
    }

    // ========================================================================
    // DFS Iterator Tests
    // ========================================================================

    #[test]
    fn test_dfs_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();
        // root, e4, e5, c5, d4
        assert_eq!(moves, vec!["", "e4", "e5", "c5", "d4"]);
    }

    #[test]
    fn test_dfs_iter_empty() {
        let root = GameNode::root();
        let moves: Vec<&str> = root.iter_dfs().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec![""]);
    }

    #[test]
    fn test_dfs_iter_single_child() {
        let mut root = GameNode::root();
        root.add_child(GameNode::new("e4"));
        let moves: Vec<&str> = root.iter_dfs().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["", "e4"]);
    }

    #[test]
    fn test_dfs_iter_deep() {
        let tree = build_deep_tree();
        let moves: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["", "e4", "e5", "Nf3", "Nc6", "Bb5"]);
    }

    #[test]
    fn test_dfs_iter_wide() {
        let tree = build_wide_tree();
        let moves: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();
        // DFS visits in order: root, then children left to right
        assert_eq!(moves, vec!["", "e4", "d4", "c4", "Nf3", "g3"]);
    }

    #[test]
    fn test_dfs_iter_count() {
        let tree = build_test_tree();
        assert_eq!(tree.iter_dfs().count(), 5);
    }

    #[test]
    fn test_dfs_iter_from_child() {
        let tree = build_test_tree();
        let e4 = tree.find_child("e4").unwrap();
        let moves: Vec<&str> = e4.iter_dfs().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["e4", "e5", "c5"]);
    }

    // ========================================================================
    // Main Line Iterator Tests
    // ========================================================================

    #[test]
    fn test_main_line_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_main_line().map(|n| n.san.as_str()).collect();
        // root, e4, e5 (main line)
        assert_eq!(moves, vec!["", "e4", "e5"]);
    }

    #[test]
    fn test_main_line_iter_empty() {
        let root = GameNode::root();
        let moves: Vec<&str> = root.iter_main_line().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec![""]);
    }

    #[test]
    fn test_main_line_iter_deep() {
        let tree = build_deep_tree();
        let moves: Vec<&str> = tree.iter_main_line().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["", "e4", "e5", "Nf3", "Nc6", "Bb5"]);
    }

    #[test]
    fn test_main_line_iter_ignores_variations() {
        let tree = build_test_tree();
        // Main line should be root -> e4 -> e5, not c5
        let moves: Vec<&str> = tree.iter_main_line().map(|n| n.san.as_str()).collect();
        assert!(!moves.contains(&"c5"));
        assert!(!moves.contains(&"d4"));
    }

    #[test]
    fn test_main_line_iter_from_child() {
        let tree = build_test_tree();
        let e4 = tree.find_child("e4").unwrap();
        let moves: Vec<&str> = e4.iter_main_line().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["e4", "e5"]);
    }

    #[test]
    fn test_main_line_iter_length() {
        let tree = build_deep_tree();
        assert_eq!(tree.iter_main_line().count(), 6);
    }

    // ========================================================================
    // Leaves Iterator Tests
    // ========================================================================

    #[test]
    fn test_leaves_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_leaves().map(|n| n.san.as_str()).collect();
        // e5, c5, d4 are leaves
        assert_eq!(moves, vec!["e5", "c5", "d4"]);
    }

    #[test]
    fn test_leaves_iter_empty() {
        let root = GameNode::root();
        let moves: Vec<&str> = root.iter_leaves().map(|n| n.san.as_str()).collect();
        // Root itself is a leaf when empty
        assert_eq!(moves, vec![""]);
    }

    #[test]
    fn test_leaves_iter_single_line() {
        let tree = build_deep_tree();
        let moves: Vec<&str> = tree.iter_leaves().map(|n| n.san.as_str()).collect();
        // Only Bb5 is a leaf
        assert_eq!(moves, vec!["Bb5"]);
    }

    #[test]
    fn test_leaves_iter_wide() {
        let tree = build_wide_tree();
        let moves: Vec<&str> = tree.iter_leaves().map(|n| n.san.as_str()).collect();
        // All children are leaves
        assert_eq!(moves.len(), 5);
    }

    #[test]
    fn test_leaves_iter_count() {
        let tree = build_test_tree();
        assert_eq!(tree.iter_leaves().count(), 3);
    }

    #[test]
    fn test_leaves_iter_from_child() {
        let tree = build_test_tree();
        let e4 = tree.find_child("e4").unwrap();
        let moves: Vec<&str> = e4.iter_leaves().map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["e5", "c5"]);
    }

    // ========================================================================
    // Depth Iterator Tests
    // ========================================================================

    #[test]
    fn test_depth_iter() {
        let tree = build_test_tree();
        let depths: Vec<(String, usize)> = tree
            .iter_with_depth()
            .map(|(n, d)| (n.san.clone(), d))
            .collect();

        assert!(depths.contains(&("".to_string(), 0)));
        assert!(depths.contains(&("e4".to_string(), 1)));
        assert!(depths.contains(&("e5".to_string(), 2)));
        assert!(depths.contains(&("c5".to_string(), 2)));
        assert!(depths.contains(&("d4".to_string(), 1)));
    }

    #[test]
    fn test_depth_iter_empty() {
        let root = GameNode::root();
        let depths: Vec<usize> = root.iter_with_depth().map(|(_, d)| d).collect();
        assert_eq!(depths, vec![0]);
    }

    #[test]
    fn test_depth_iter_deep() {
        let tree = build_deep_tree();
        let depths: Vec<usize> = tree.iter_with_depth().map(|(_, d)| d).collect();
        assert_eq!(depths, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_depth_iter_max_depth() {
        let tree = build_deep_tree();
        let max_depth = tree.iter_with_depth().map(|(_, d)| d).max().unwrap();
        assert_eq!(max_depth, 5);
    }

    #[test]
    fn test_depth_iter_from_child() {
        let tree = build_test_tree();
        let e4 = tree.find_child("e4").unwrap();
        let depths: Vec<usize> = e4.iter_with_depth().map(|(_, d)| d).collect();
        // Starting from e4, depths are 0, 1, 1
        assert_eq!(depths, vec![0, 1, 1]);
    }

    // ========================================================================
    // Iterator Trait Tests
    // ========================================================================

    #[test]
    fn test_iter_chain() {
        let tree = build_test_tree();
        let all: Vec<&str> = tree
            .iter_main_line()
            .chain(tree.iter_leaves())
            .map(|n| n.san.as_str())
            .collect();
        // Main line + leaves (with some overlap)
        assert!(all.contains(&"e4"));
        assert!(all.contains(&"e5"));
        assert!(all.contains(&"c5"));
    }

    #[test]
    fn test_iter_filter() {
        let mut tree = build_test_tree();
        tree.find_child_mut("e4").unwrap().comment = "Has comment".to_string();

        let with_comments: Vec<&str> = tree
            .iter_dfs()
            .filter(|n| !n.comment.is_empty())
            .map(|n| n.san.as_str())
            .collect();

        assert_eq!(with_comments, vec!["e4"]);
    }

    #[test]
    fn test_iter_skip() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_dfs().skip(1).map(|n| n.san.as_str()).collect();
        // Skip root
        assert!(!moves.contains(&""));
        assert_eq!(moves, vec!["e4", "e5", "c5", "d4"]);
    }

    #[test]
    fn test_iter_take() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_dfs().take(3).map(|n| n.san.as_str()).collect();
        assert_eq!(moves, vec!["", "e4", "e5"]);
    }

    #[test]
    fn test_iter_find() {
        let tree = build_test_tree();
        let found = tree.iter_dfs().find(|n| n.san == "c5");
        assert!(found.is_some());
        assert_eq!(found.unwrap().san, "c5");
    }

    #[test]
    fn test_iter_any() {
        let tree = build_test_tree();
        assert!(tree.iter_dfs().any(|n| n.san == "e4"));
        assert!(!tree.iter_dfs().any(|n| n.san == "Nf3"));
    }

    #[test]
    fn test_iter_all() {
        let tree = build_test_tree();
        // All non-root nodes have non-empty san
        assert!(tree.iter_dfs().skip(1).all(|n| !n.san.is_empty()));
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_iter_after_mutation() {
        let mut root = GameNode::root();
        let e4 = root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        // Count before
        let count_before = root.iter_dfs().count();

        // Mutate
        root.add_child(GameNode::new("d4"));

        // Count after
        let count_after = root.iter_dfs().count();

        assert_eq!(count_before, 3);
        assert_eq!(count_after, 4);
    }

    #[test]
    fn test_iter_multiple_times() {
        let tree = build_test_tree();

        let first: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();
        let second: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();

        assert_eq!(first, second);
    }
}
