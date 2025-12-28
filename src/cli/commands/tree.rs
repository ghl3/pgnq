//! `pgnq tree` command - display the game tree visually

use crate::cli::InputSource;
use crate::parser::parse;
use crate::serializer::{to_tree_view, OutputFormat, OutputOptions};
use crate::tree::NodePath;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct TreeArgs {
    /// Input PGN file (use '-' for stdin)
    #[arg(value_name = "FILE", default_value = "-")]
    pub input: InputSource,

    /// Maximum depth to display
    #[arg(short = 'd', long, default_value = "10")]
    pub depth: usize,

    /// Start from specific path
    #[arg(short = 'p', long)]
    pub from_path: Option<String>,

    /// Show comments (truncated)
    #[arg(long)]
    pub show_comments: bool,

    /// Show NAGs
    #[arg(long)]
    pub show_nags: bool,

    /// Use ASCII characters only (no Unicode box drawing)
    #[arg(long)]
    pub ascii: bool,

    /// Select specific game (1-indexed)
    #[arg(long)]
    pub game: Option<usize>,
}

pub fn run(args: TreeArgs, _quiet: bool) -> Result<()> {
    let content = args.input.read_to_string()?;
    let tree = parse(&content)?;

    let options = OutputOptions {
        format: OutputFormat::Tree,
        max_depth: args.depth,
        ascii: args.ascii,
        show_comments: args.show_comments,
        show_nags: args.show_nags,
        ..Default::default()
    };

    // If a path is specified, start from that node
    if let Some(path_str) = &args.from_path {
        let path = NodePath::parse(path_str)?;
        if let Some(node) = path.resolve(&tree) {
            // Create a temporary tree rooted at this node
            let mut subtree = crate::tree::GameTree::new();
            subtree.root = node.deep_clone();
            let output = to_tree_view(&subtree, &options);
            print!("{}", output);
            return Ok(());
        } else {
            anyhow::bail!("Path not found: {}", path_str);
        }
    }

    let output = to_tree_view(&tree, &options);
    print!("{}", output);

    Ok(())
}
