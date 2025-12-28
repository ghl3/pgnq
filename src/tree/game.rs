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

    #[test]
    fn test_game_result_parse() {
        assert_eq!(GameResult::parse("1-0"), Some(GameResult::WhiteWins));
        assert_eq!(GameResult::parse("0-1"), Some(GameResult::BlackWins));
        assert_eq!(GameResult::parse("1/2-1/2"), Some(GameResult::Draw));
        assert_eq!(GameResult::parse("*"), Some(GameResult::Ongoing));
        assert_eq!(GameResult::parse("invalid"), None);
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
}
