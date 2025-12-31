//! `pgnq merge` command - merge multiple PGN files into unified tree

use crate::cli::{CliOutputFormat, InputSource};
use crate::error::ParseMode;
use crate::parser::parse_all_with_options;
use crate::serializer::{to_pgn, OutputOptions};
use crate::tree::{GameNode, GameResult, GameTree};
use anyhow::{bail, Result};
use clap::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct MergeArgs {
    /// Input PGN files (use '-' for stdin as first file only)
    #[arg(value_name = "FILE", required = true)]
    pub inputs: Vec<InputSource>,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short = 'F', long, value_enum, default_value = "standard")]
    pub format: CliOutputFormat,

    /// Strip all comments
    #[arg(long)]
    pub no_comments: bool,

    /// Remove NAG annotations
    #[arg(long)]
    pub no_nags: bool,

    /// Concatenate comments when merging duplicate moves
    #[arg(long)]
    pub concat_comments: bool,

    /// Wrap lines at N characters (0 = no wrapping)
    #[arg(long, default_value = "80")]
    pub line_width: usize,
}

/// Options for controlling merge behavior
#[derive(Default)]
pub struct MergeOptions {
    pub concat_comments: bool,
}

pub fn run(args: MergeArgs, _quiet: bool, mode: ParseMode) -> Result<()> {
    // Validate inputs
    let stdin_count = args.inputs.iter().filter(|i| matches!(i, InputSource::Stdin)).count();
    if stdin_count > 1 {
        bail!("stdin ('-') can only be specified once");
    }

    // Collect all games from all inputs
    let mut all_trees: Vec<GameTree> = Vec::new();

    for input in &args.inputs {
        let content = input.read_to_string()?;
        let file_path = match input {
            InputSource::File(p) => Some(p.clone()),
            InputSource::Stdin => None,
        };
        let trees = parse_all_with_options(&content, mode, file_path)?;
        all_trees.extend(trees);
    }

    if all_trees.is_empty() {
        bail!("no games found in input files");
    }

    // Merge all trees
    let merge_options = MergeOptions {
        concat_comments: args.concat_comments,
    };
    let merged = merge_trees(&all_trees, &merge_options);

    // Serialize output
    let options = OutputOptions {
        format: args.format.into(),
        headers: true,
        comments: !args.no_comments,
        variations: true,
        nags: !args.no_nags,
        strip_clocks: false,
        strip_evals: false,
        line_width: args.line_width,
        result: true,
        ..Default::default()
    };

    let output = to_pgn(&merged, &options);

    // Write output
    if let Some(path) = args.output {
        fs::write(&path, &output)?;
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}

/// Merge multiple game trees into a single unified tree
///
/// Uses BFS-style algorithm to merge level-by-level, grouping moves by
/// normalized SAN. First-seen move order is preserved (first = main line).
pub fn merge_trees(trees: &[GameTree], options: &MergeOptions) -> GameTree {
    let mut result = GameTree::new();

    // Use headers from first tree
    if let Some(first) = trees.first() {
        result.headers = first.headers.clone();
    }

    // Set result to ongoing since merged tree has multiple endpoints
    result.result = GameResult::Ongoing;

    // Collect all first-level children from all trees
    let initial_children: Vec<&GameNode> = trees
        .iter()
        .flat_map(|t| t.root.children.iter())
        .collect();

    if initial_children.is_empty() {
        return result;
    }

    // BFS work list: (path to target node as indices, source nodes to merge)
    let mut work_list: Vec<(Vec<usize>, Vec<&GameNode>)> = vec![(vec![], initial_children)];

    while let Some((target_path, source_nodes)) = work_list.pop() {
        // Group source nodes by normalized SAN
        let mut groups: IndexMap<String, Vec<&GameNode>> = IndexMap::new();
        for node in source_nodes {
            let key = normalize_san(&node.san);
            groups.entry(key).or_default().push(node);
        }

        // Get mutable reference to target node
        let target = get_node_mut_by_path(&mut result.root, &target_path);

        // Process each unique move (preserving insertion order)
        for (_san, nodes) in groups {
            // Create merged node from first instance
            let mut merged = GameNode::new(&nodes[0].san);
            merged.move_number = nodes[0].move_number;
            merged.is_black = nodes[0].is_black;

            // Merge comments
            if options.concat_comments {
                // Concatenate all non-empty comments
                let comments: Vec<&str> = nodes
                    .iter()
                    .filter(|n| !n.comment.is_empty())
                    .map(|n| n.comment.as_str())
                    .collect();
                if !comments.is_empty() {
                    merged.comment = comments.join(" ");
                }
            } else {
                // First-seen wins
                merged.comment = nodes
                    .iter()
                    .find(|n| !n.comment.is_empty())
                    .map(|n| n.comment.clone())
                    .unwrap_or_default();
            }

            // Merge NAGs (first-seen wins)
            merged.nags = nodes
                .iter()
                .find(|n| !n.nags.is_empty())
                .map(|n| n.nags.clone())
                .unwrap_or_default();

            // Add to target
            target.children.push(merged);
            let idx = target.children.len() - 1;

            // Collect all children from matching source nodes for next level
            let all_children: Vec<&GameNode> = nodes
                .iter()
                .flat_map(|n| n.children.iter())
                .collect();

            if !all_children.is_empty() {
                let mut new_path = target_path.clone();
                new_path.push(idx);
                work_list.push((new_path, all_children));
            }
        }
    }

    result
}

/// Navigate to a node given a path of child indices.
/// Uses GameNode::navigate_path_mut for centralized path navigation.
fn get_node_mut_by_path<'a>(root: &'a mut GameNode, path: &[usize]) -> &'a mut GameNode {
    root.navigate_path_mut(path)
}

// Use centralized SAN normalization
use crate::tree::san::normalize as normalize_san;

/// Ordered hash map to preserve insertion order
/// This ensures first-seen moves become main line
struct IndexMap<K, V> {
    map: HashMap<K, usize>,
    entries: Vec<(K, V)>,
}

impl<K: std::hash::Hash + Eq + Clone, V> IndexMap<K, V> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            entries: Vec::new(),
        }
    }

    fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        if let Some(&idx) = self.map.get(&key) {
            Entry::Occupied(&mut self.entries[idx].1)
        } else {
            Entry::Vacant {
                map: self,
                key,
            }
        }
    }
}

impl<K, V> IntoIterator for IndexMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

enum Entry<'a, K, V> {
    Occupied(&'a mut V),
    Vacant { map: &'a mut IndexMap<K, V>, key: K },
}

impl<'a, K: std::hash::Hash + Eq + Clone, V: Default> Entry<'a, K, V> {
    fn or_default(self) -> &'a mut V {
        match self {
            Entry::Occupied(v) => v,
            Entry::Vacant { map, key } => {
                let idx = map.entries.len();
                map.map.insert(key.clone(), idx);
                map.entries.push((key, V::default()));
                &mut map.entries[idx].1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_merge_two_games_same_first_move() {
        let t1 = parse("1. e4 e5 2. Nf3").unwrap();
        let t2 = parse("1. e4 c5 2. Nf3").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        // e4 should have two children: e5 and c5
        let e4 = merged.root.find_child("e4").unwrap();
        assert_eq!(e4.children.len(), 2);
        assert_eq!(e4.children[0].san, "e5");
        assert_eq!(e4.children[1].san, "c5");
    }

    #[test]
    fn test_merge_different_first_moves() {
        let t1 = parse("1. e4 e5").unwrap();
        let t2 = parse("1. d4 d5").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        // Root should have two children: e4 and d4
        assert_eq!(merged.root.children.len(), 2);
        assert_eq!(merged.root.children[0].san, "e4");
        assert_eq!(merged.root.children[1].san, "d4");
    }

    #[test]
    fn test_merge_preserves_first_as_main_line() {
        let t1 = parse("1. e4 e5").unwrap();
        let t2 = parse("1. e4 c5").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        // First child should be e5 (main line from first game)
        let e4 = merged.root.find_child("e4").unwrap();
        assert_eq!(e4.children[0].san, "e5");
        assert_eq!(e4.children[1].san, "c5");
    }

    #[test]
    fn test_merge_comments_first_wins() {
        let t1 = parse("1. e4 {First comment} e5").unwrap();
        let t2 = parse("1. e4 {Second comment} c5").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        let e4 = merged.root.find_child("e4").unwrap();
        assert_eq!(e4.comment, "First comment");
    }

    #[test]
    fn test_merge_comments_concat() {
        let t1 = parse("1. e4 {First} e5").unwrap();
        let t2 = parse("1. e4 {Second} c5").unwrap();

        let options = MergeOptions { concat_comments: true };
        let merged = merge_trees(&[t1, t2], &options);

        let e4 = merged.root.find_child("e4").unwrap();
        assert_eq!(e4.comment, "First Second");
    }

    #[test]
    fn test_merge_deep_convergence() {
        // Two games that diverge then converge
        let t1 = parse("1. e4 e5 2. Nf3 Nc6 3. Bb5").unwrap();
        let t2 = parse("1. e4 e5 2. Nf3 Nc6 3. Bc4").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        // Navigate to Nc6
        let nc6 = merged.root.find_path(&["e4", "e5", "Nf3", "Nc6"]).unwrap();

        // Should have two children: Bb5 and Bc4
        assert_eq!(nc6.children.len(), 2);
        assert_eq!(nc6.children[0].san, "Bb5");
        assert_eq!(nc6.children[1].san, "Bc4");
    }

    #[test]
    fn test_merge_empty_trees() {
        let merged = merge_trees(&[], &MergeOptions::default());
        assert!(merged.root.children.is_empty());
    }

    #[test]
    fn test_merge_single_tree() {
        let t1 = parse("1. e4 e5 2. Nf3").unwrap();
        let merged = merge_trees(&[t1], &MergeOptions::default());

        assert_eq!(merged.root.children.len(), 1);
        assert_eq!(merged.root.find_path(&["e4", "e5", "Nf3"]).unwrap().san, "Nf3");
    }

    #[test]
    fn test_merge_preserves_nags() {
        let t1 = parse("1. e4! e5").unwrap();
        let t2 = parse("1. e4 c5").unwrap();

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        let e4 = merged.root.find_child("e4").unwrap();
        assert!(!e4.nags.is_empty());
    }

    #[test]
    fn test_merge_multiple_games_same_file() {
        // Simulating multiple games from one file
        let t1 = parse("1. e4 e5").unwrap();
        let t2 = parse("1. d4 d5").unwrap();
        let t3 = parse("1. e4 c5").unwrap();

        let merged = merge_trees(&[t1, t2, t3], &MergeOptions::default());

        // Root should have e4 and d4
        assert_eq!(merged.root.children.len(), 2);

        // e4 should have e5 and c5 as children
        let e4 = merged.root.find_child("e4").unwrap();
        assert_eq!(e4.children.len(), 2);
    }

    // SAN normalization tests moved to tree::san module

    #[test]
    fn test_merge_headers_from_first() {
        let mut t1 = parse("1. e4").unwrap();
        t1.headers.insert("Event".to_string(), "First Event".to_string());

        let mut t2 = parse("1. d4").unwrap();
        t2.headers.insert("Event".to_string(), "Second Event".to_string());

        let merged = merge_trees(&[t1, t2], &MergeOptions::default());

        assert_eq!(merged.headers.get("Event"), Some(&"First Event".to_string()));
    }

    #[test]
    fn test_merge_result_is_ongoing() {
        let mut t1 = parse("1. e4 e5").unwrap();
        t1.result = GameResult::WhiteWins;

        let merged = merge_trees(&[t1], &MergeOptions::default());

        // Merged trees always have ongoing result
        assert_eq!(merged.result, GameResult::Ongoing);
    }
}
