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

    #[test]
    fn test_dfs_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_dfs().map(|n| n.san.as_str()).collect();
        // root, e4, e5, c5, d4
        assert_eq!(moves, vec!["", "e4", "e5", "c5", "d4"]);
    }

    #[test]
    fn test_main_line_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_main_line().map(|n| n.san.as_str()).collect();
        // root, e4, e5 (main line)
        assert_eq!(moves, vec!["", "e4", "e5"]);
    }

    #[test]
    fn test_leaves_iter() {
        let tree = build_test_tree();
        let moves: Vec<&str> = tree.iter_leaves().map(|n| n.san.as_str()).collect();
        // e5, c5, d4 are leaves
        assert_eq!(moves, vec!["e5", "c5", "d4"]);
    }
}
