//! GameTree - represents a complete chess game with metadata

use super::GameNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a chess game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    Ongoing,
}

impl GameResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Ongoing => "*",
        }
    }

    pub fn parse(s: &str) -> Option<GameResult> {
        match s.trim() {
            "1-0" => Some(GameResult::WhiteWins),
            "0-1" => Some(GameResult::BlackWins),
            "1/2-1/2" => Some(GameResult::Draw),
            "*" => Some(GameResult::Ongoing),
            _ => None,
        }
    }
}

impl Default for GameResult {
    fn default() -> Self {
        GameResult::Ongoing
    }
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A complete game tree with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTree {
    /// PGN tag pairs (Event, Site, Date, etc.)
    pub headers: HashMap<String, String>,

    /// Root node of the game tree (empty move, children are first moves)
    pub root: GameNode,

    /// Game result
    pub result: GameResult,
}

impl GameTree {
    /// Create a new empty game tree
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            root: GameNode::root(),
            result: GameResult::Ongoing,
        }
    }

    /// Navigate from root following a path
    pub fn find_path(&self, path: &[&str]) -> Option<&GameNode> {
        self.root.find_path(path)
    }

    /// Navigate from root (mutable)
    pub fn find_path_mut(&mut self, path: &[&str]) -> Option<&mut GameNode> {
        self.root.find_path_mut(path)
    }

    /// Count total nodes (excluding root)
    pub fn count_nodes(&self) -> usize {
        self.root.count_nodes() - 1 // Exclude root
    }

    /// Count leaf nodes (distinct lines)
    pub fn count_lines(&self) -> usize {
        if self.root.children.is_empty() {
            0
        } else {
            self.root.count_leaves()
        }
    }

    /// Count nodes with comments
    pub fn count_comments(&self) -> usize {
        self.root.count_comments()
    }

    /// Get maximum tree depth
    pub fn max_depth(&self) -> usize {
        if self.root.children.is_empty() {
            0
        } else {
            self.root.max_depth()
        }
    }

    /// Get main line length
    pub fn main_line_length(&self) -> usize {
        self.root.main_line_length()
    }

    /// Get a header value
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    /// Set a header value
    pub fn set_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(key.into(), value.into());
    }

    /// Get the Seven Tag Roster values
    pub fn seven_tag_roster(&self) -> SevenTagRoster {
        SevenTagRoster {
            event: self.header("Event").unwrap_or("?").to_string(),
            site: self.header("Site").unwrap_or("?").to_string(),
            date: self.header("Date").unwrap_or("????.??.??").to_string(),
            round: self.header("Round").unwrap_or("?").to_string(),
            white: self.header("White").unwrap_or("?").to_string(),
            black: self.header("Black").unwrap_or("?").to_string(),
            result: self.result.as_str().to_string(),
        }
    }
}

impl Default for GameTree {
    fn default() -> Self {
        Self::new()
    }
}

/// The Seven Tag Roster required by PGN standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SevenTagRoster {
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
    pub result: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // GameResult Tests
    // ========================================================================

    #[test]
    fn test_game_result_parse() {
        assert_eq!(GameResult::parse("1-0"), Some(GameResult::WhiteWins));
        assert_eq!(GameResult::parse("0-1"), Some(GameResult::BlackWins));
        assert_eq!(GameResult::parse("1/2-1/2"), Some(GameResult::Draw));
        assert_eq!(GameResult::parse("*"), Some(GameResult::Ongoing));
        assert_eq!(GameResult::parse("invalid"), None);
    }

    #[test]
    fn test_game_result_parse_with_whitespace() {
        assert_eq!(GameResult::parse("  1-0  "), Some(GameResult::WhiteWins));
        assert_eq!(GameResult::parse("\n*\n"), Some(GameResult::Ongoing));
    }

    #[test]
    fn test_game_result_as_str() {
        assert_eq!(GameResult::WhiteWins.as_str(), "1-0");
        assert_eq!(GameResult::BlackWins.as_str(), "0-1");
        assert_eq!(GameResult::Draw.as_str(), "1/2-1/2");
        assert_eq!(GameResult::Ongoing.as_str(), "*");
    }

    #[test]
    fn test_game_result_display() {
        assert_eq!(format!("{}", GameResult::WhiteWins), "1-0");
        assert_eq!(format!("{}", GameResult::BlackWins), "0-1");
        assert_eq!(format!("{}", GameResult::Draw), "1/2-1/2");
        assert_eq!(format!("{}", GameResult::Ongoing), "*");
    }

    #[test]
    fn test_game_result_default() {
        assert_eq!(GameResult::default(), GameResult::Ongoing);
    }

    #[test]
    fn test_game_result_equality() {
        assert_eq!(GameResult::WhiteWins, GameResult::WhiteWins);
        assert_ne!(GameResult::WhiteWins, GameResult::BlackWins);
    }

    #[test]
    fn test_game_result_clone() {
        let result = GameResult::Draw;
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ========================================================================
    // GameTree Basic Tests
    // ========================================================================

    #[test]
    fn test_game_tree_new() {
        let tree = GameTree::new();
        assert!(tree.root.is_root());
        assert!(tree.headers.is_empty());
        assert_eq!(tree.result, GameResult::Ongoing);
    }

    #[test]
    fn test_game_tree_default() {
        let tree = GameTree::default();
        assert!(tree.root.is_root());
    }

    #[test]
    fn test_game_tree_headers() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "Test Event");
        tree.set_header("White", "Player 1");

        assert_eq!(tree.header("Event"), Some("Test Event"));
        assert_eq!(tree.header("White"), Some("Player 1"));
        assert_eq!(tree.header("Black"), None);
    }

    #[test]
    fn test_game_tree_header_overwrite() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "First");
        tree.set_header("Event", "Second");

        assert_eq!(tree.header("Event"), Some("Second"));
    }

    #[test]
    fn test_game_tree_header_into_conversion() {
        let mut tree = GameTree::new();
        tree.set_header(String::from("Event"), String::from("Test"));
        assert_eq!(tree.header("Event"), Some("Test"));
    }

    // ========================================================================
    // Seven Tag Roster Tests
    // ========================================================================

    #[test]
    fn test_seven_tag_roster() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "World Championship");
        tree.set_header("White", "Carlsen");
        tree.result = GameResult::WhiteWins;

        let str = tree.seven_tag_roster();
        assert_eq!(str.event, "World Championship");
        assert_eq!(str.white, "Carlsen");
        assert_eq!(str.black, "?"); // Default
        assert_eq!(str.result, "1-0");
    }

    #[test]
    fn test_seven_tag_roster_defaults() {
        let tree = GameTree::new();
        let str = tree.seven_tag_roster();

        assert_eq!(str.event, "?");
        assert_eq!(str.site, "?");
        assert_eq!(str.date, "????.??.??");
        assert_eq!(str.round, "?");
        assert_eq!(str.white, "?");
        assert_eq!(str.black, "?");
        assert_eq!(str.result, "*");
    }

    #[test]
    fn test_seven_tag_roster_complete() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "Test Event");
        tree.set_header("Site", "Test Site");
        tree.set_header("Date", "2024.01.15");
        tree.set_header("Round", "1");
        tree.set_header("White", "Alice");
        tree.set_header("Black", "Bob");
        tree.result = GameResult::Draw;

        let str = tree.seven_tag_roster();
        assert_eq!(str.event, "Test Event");
        assert_eq!(str.site, "Test Site");
        assert_eq!(str.date, "2024.01.15");
        assert_eq!(str.round, "1");
        assert_eq!(str.white, "Alice");
        assert_eq!(str.black, "Bob");
        assert_eq!(str.result, "1/2-1/2");
    }

    // ========================================================================
    // Find Path Tests
    // ========================================================================

    #[test]
    fn test_find_path() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        let result = tree.find_path(&["e4", "e5", "Nf3"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().san, "Nf3");
    }

    #[test]
    fn test_find_path_empty() {
        let tree = GameTree::new();
        let result = tree.find_path(&[]);
        assert!(result.is_some());
        assert!(result.unwrap().is_root());
    }

    #[test]
    fn test_find_path_nonexistent() {
        let tree = GameTree::new();
        let result = tree.find_path(&["e4"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_path_mut() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        let e5 = tree.find_path_mut(&["e4", "e5"]).unwrap();
        e5.comment = "Modified".to_string();

        assert_eq!(tree.find_path(&["e4", "e5"]).unwrap().comment, "Modified");
    }

    // ========================================================================
    // Counting Tests
    // ========================================================================

    #[test]
    fn test_count_nodes() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));

        // Excludes root
        assert_eq!(tree.count_nodes(), 2);
    }

    #[test]
    fn test_count_nodes_empty() {
        let tree = GameTree::new();
        assert_eq!(tree.count_nodes(), 0);
    }

    #[test]
    fn test_count_lines() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.add_child(GameNode::new("e5"));
        e4.add_child(GameNode::new("c5"));
        tree.root.add_child(GameNode::new("d4"));

        // 3 leaf nodes = 3 lines
        assert_eq!(tree.count_lines(), 3);
    }

    #[test]
    fn test_count_lines_empty() {
        let tree = GameTree::new();
        assert_eq!(tree.count_lines(), 0);
    }

    #[test]
    fn test_count_lines_single() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        assert_eq!(tree.count_lines(), 1);
    }

    #[test]
    fn test_count_comments() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        e4.comment = "Opening".to_string();
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.comment = "Reply".to_string();

        assert_eq!(tree.count_comments(), 2);
    }

    #[test]
    fn test_count_comments_empty() {
        let tree = GameTree::new();
        assert_eq!(tree.count_comments(), 0);
    }

    // ========================================================================
    // Depth Tests
    // ========================================================================

    #[test]
    fn test_max_depth() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        assert_eq!(tree.max_depth(), 3);
    }

    #[test]
    fn test_max_depth_empty() {
        let tree = GameTree::new();
        assert_eq!(tree.max_depth(), 0);
    }

    #[test]
    fn test_main_line_length() {
        let mut tree = GameTree::new();
        let e4 = tree.root.add_child(GameNode::new("e4"));
        let e5 = e4.add_child(GameNode::new("e5"));
        e5.add_child(GameNode::new("Nf3"));

        assert_eq!(tree.main_line_length(), 3);
    }

    #[test]
    fn test_main_line_length_empty() {
        let tree = GameTree::new();
        assert_eq!(tree.main_line_length(), 0);
    }

    // ========================================================================
    // Clone Tests
    // ========================================================================

    #[test]
    fn test_game_tree_clone() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "Test");
        tree.result = GameResult::WhiteWins;
        tree.root.add_child(GameNode::new("e4"));

        let cloned = tree.clone();
        assert_eq!(cloned.header("Event"), Some("Test"));
        assert_eq!(cloned.result, GameResult::WhiteWins);
        assert_eq!(cloned.root.children.len(), 1);
    }

    #[test]
    fn test_game_tree_clone_independence() {
        let mut tree = GameTree::new();
        tree.set_header("Event", "Original");

        let mut cloned = tree.clone();
        cloned.set_header("Event", "Modified");

        assert_eq!(tree.header("Event"), Some("Original"));
        assert_eq!(cloned.header("Event"), Some("Modified"));
    }
}
