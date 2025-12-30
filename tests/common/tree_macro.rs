//! Tree construction macro for building GameNode trees
//!
//! Provides the `game_tree!` macro for declaratively constructing GameNode trees
//! with a clean, readable syntax.
//!
//! # Syntax
//!
//! ```ignore
//! game_tree! {
//!     move_name (properties) { children }
//! }
//! ```
//!
//! Where:
//! - `move_name` is either an identifier (`e4`) or string literal (`"O-O"`)
//! - `(properties)` is optional: `(comment: "text")`, `(nag: GOOD_MOVE)`, etc.
//! - `{ children }` is optional: nested moves separated by commas
//!
//! # Examples
//!
//! ```ignore
//! // Simple linear game
//! let tree = game_tree! { e4 { e5 { Nf3 { Nc6 } } } };
//!
//! // With properties
//! let tree = game_tree! {
//!     e4 (comment: "King's Pawn", nag: GOOD_MOVE) {
//!         e5 { Nf3 }
//!     }
//! };
//!
//! // With variations (siblings)
//! let tree = game_tree! {
//!     e4 {
//!         e5 { Nf3 },
//!         c5 (comment: "Sicilian"),
//!         d5
//!     }
//! };
//!
//! // String literals for special moves
//! let tree = game_tree! { e4 { e5 { Nf3 { Nc6 { "O-O" } } } } };
//! ```

/// Create a GameNode tree using a declarative syntax
///
/// The macro produces a `GameNode::root()` with the specified children.
#[macro_export]
macro_rules! game_tree {
    // Empty tree
    () => {{
        pgnq::tree::GameNode::root()
    }};

    // === Single node patterns (no siblings) ===

    // Single identifier leaf
    ($name:ident) => {{
        let mut root = pgnq::tree::GameNode::root();
        root.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        root
    }};

    // Single string literal leaf
    ($name:literal) => {{
        let mut root = pgnq::tree::GameNode::root();
        root.add_child(pgnq::tree::GameNode::new($name));
        root
    }};

    // Identifier with properties (no children)
    ($name:ident ( $($key:ident : $val:tt),* $(,)? )) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        root
    }};

    // String literal with properties (no children)
    ($name:literal ( $($key:ident : $val:tt),* $(,)? )) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        root
    }};

    // Identifier with children (no properties)
    ($name:ident { $($inner:tt)* }) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $crate::game_tree!(@children child; $($inner)*);
        root
    }};

    // String literal with children (no properties)
    ($name:literal { $($inner:tt)* }) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new($name));
        $crate::game_tree!(@children child; $($inner)*);
        root
    }};

    // Identifier with properties AND children
    ($name:ident ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* }) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
        root
    }};

    // String literal with properties AND children
    ($name:literal ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* }) => {{
        let mut root = pgnq::tree::GameNode::root();
        let child = root.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
        root
    }};

    // === Internal: Process children ===

    // Empty children
    (@children $parent:ident;) => {};

    // Identifier with props and children, followed by comma and more
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* } , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // Identifier with props and children, trailing comma
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* } ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // Identifier with props and children (last item)
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* }) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // Identifier with props only, followed by comma and more
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? ) , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // Identifier with props only, trailing comma
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? ) ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
    }};

    // Identifier with props only (last item)
    (@children $parent:ident; $name:ident ( $($key:ident : $val:tt),* $(,)? )) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $($crate::game_tree!(@prop child, $key, $val);)*
    }};

    // Identifier with children only, followed by comma and more
    (@children $parent:ident; $name:ident { $($inner:tt)* } , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $crate::game_tree!(@children child; $($inner)*);
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // Identifier with children only, trailing comma
    (@children $parent:ident; $name:ident { $($inner:tt)* } ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // Identifier with children only (last item)
    (@children $parent:ident; $name:ident { $($inner:tt)* }) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // Identifier leaf, followed by comma and more
    (@children $parent:ident; $name:ident , $($rest:tt)+) => {{
        $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // Identifier leaf with trailing comma
    (@children $parent:ident; $name:ident ,) => {{
        $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
    }};

    // Identifier leaf (last item)
    (@children $parent:ident; $name:ident) => {{
        $parent.add_child(pgnq::tree::GameNode::new(stringify!($name)));
    }};

    // === String literal versions ===

    // String with props and children, followed by comma and more
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* } , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // String with props and children, trailing comma
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* } ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // String with props and children (last item)
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? ) { $($inner:tt)* }) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // String with props only, followed by comma and more
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? ) , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // String with props only, trailing comma
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? ) ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
    }};

    // String with props only (last item)
    (@children $parent:ident; $name:literal ( $($key:ident : $val:tt),* $(,)? )) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $($crate::game_tree!(@prop child, $key, $val);)*
    }};

    // String with children only, followed by comma and more
    (@children $parent:ident; $name:literal { $($inner:tt)* } , $($rest:tt)+) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $crate::game_tree!(@children child; $($inner)*);
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // String with children only, trailing comma
    (@children $parent:ident; $name:literal { $($inner:tt)* } ,) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // String with children only (last item)
    (@children $parent:ident; $name:literal { $($inner:tt)* }) => {{
        let child = $parent.add_child(pgnq::tree::GameNode::new($name));
        $crate::game_tree!(@children child; $($inner)*);
    }};

    // String leaf, followed by comma and more
    (@children $parent:ident; $name:literal , $($rest:tt)+) => {{
        $parent.add_child(pgnq::tree::GameNode::new($name));
        $crate::game_tree!(@children $parent; $($rest)+);
    }};

    // String leaf with trailing comma
    (@children $parent:ident; $name:literal ,) => {{
        $parent.add_child(pgnq::tree::GameNode::new($name));
    }};

    // String leaf (last item)
    (@children $parent:ident; $name:literal) => {{
        $parent.add_child(pgnq::tree::GameNode::new($name));
    }};

    // === Property handlers ===

    (@prop $node:ident, comment, $val:tt) => {
        $node.comment = ($val).to_string();
    };

    (@prop $node:ident, nag, $val:tt) => {
        $node.nags.push(pgnq::nag::Nag::$val);
    };

    (@prop $node:ident, nags, [$($nag:tt),* $(,)?]) => {
        $($node.nags.push(pgnq::nag::Nag::$nag);)*
    };
}

#[cfg(test)]
mod tests {
    use pgnq::nag::Nag;

    #[test]
    fn test_empty_tree() {
        let tree = game_tree! {};
        assert!(tree.is_root());
        assert!(tree.children.is_empty());
    }

    #[test]
    fn test_single_leaf() {
        let tree = game_tree! { e4 };
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].san, "e4");
    }

    #[test]
    fn test_simple_linear() {
        let tree = game_tree! { e4 { e5 { Nf3 { Nc6 } } } };
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].children[0].san, "e5");
        assert_eq!(tree.children[0].children[0].children[0].san, "Nf3");
        assert_eq!(tree.children[0].children[0].children[0].children[0].san, "Nc6");
    }

    #[test]
    fn test_variations() {
        let tree = game_tree! {
            e4 {
                e5,
                c5,
                d5
            }
        };
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].children.len(), 3);
        assert_eq!(tree.children[0].children[0].san, "e5");
        assert_eq!(tree.children[0].children[1].san, "c5");
        assert_eq!(tree.children[0].children[2].san, "d5");
    }

    #[test]
    fn test_comment_property() {
        let tree = game_tree! {
            e4 (comment: "King's Pawn")
        };
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].comment, "King's Pawn");
    }

    #[test]
    fn test_nag_property() {
        let tree = game_tree! {
            e4 (nag: GOOD_MOVE)
        };
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].nags, vec![Nag::GOOD_MOVE]);
    }

    #[test]
    fn test_multiple_nags() {
        let tree = game_tree! {
            e4 (nags: [GOOD_MOVE, WHITE_SLIGHT_ADVANTAGE])
        };
        assert_eq!(tree.children[0].nags.len(), 2);
        assert_eq!(tree.children[0].nags[0], Nag::GOOD_MOVE);
        assert_eq!(tree.children[0].nags[1], Nag::WHITE_SLIGHT_ADVANTAGE);
    }

    #[test]
    fn test_multiple_properties() {
        let tree = game_tree! {
            e4 (comment: "Best by test", nag: GOOD_MOVE) {
                e5
            }
        };
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].comment, "Best by test");
        assert_eq!(tree.children[0].nags, vec![Nag::GOOD_MOVE]);
        assert_eq!(tree.children[0].children[0].san, "e5");
    }

    #[test]
    fn test_string_literal_move() {
        let tree = game_tree! { "O-O" };
        assert_eq!(tree.children[0].san, "O-O");
    }

    #[test]
    fn test_string_literal_with_children() {
        let tree = game_tree! {
            e4 { e5 { Nf3 { Nc6 { "O-O" { d6 } } } } }
        };
        let castling = tree
            .find_path(&["e4", "e5", "Nf3", "Nc6", "O-O"])
            .expect("Should find castling");
        assert_eq!(castling.san, "O-O");
        assert_eq!(castling.children[0].san, "d6");
    }

    #[test]
    fn test_string_literal_with_properties() {
        let tree = game_tree! {
            "O-O" (comment: "Castles kingside", nag: GOOD_MOVE)
        };
        assert_eq!(tree.children[0].san, "O-O");
        assert_eq!(tree.children[0].comment, "Castles kingside");
        assert_eq!(tree.children[0].nags, vec![Nag::GOOD_MOVE]);
    }

    #[test]
    fn test_deep_tree() {
        let tree = game_tree! {
            e4 { e5 { Nf3 { Nc6 { Bb5 { a6 { Ba4 { Nf6 { "O-O" { Be7 } } } } } } } } }
        };
        assert_eq!(tree.max_depth(), 10);
    }

    #[test]
    fn test_complex_tree_with_variations_and_properties() {
        let tree = game_tree! {
            e4 (comment: "King's Pawn") {
                e5 (comment: "Open Game") { Nf3 },
                c5 (comment: "Sicilian Defense", nag: GOOD_MOVE) {
                    Nf3 { d6 { d4 } },
                    Nc3 (comment: "Closed Sicilian")
                },
                e6 (comment: "French Defense")
            }
        };

        // Verify structure
        assert_eq!(tree.children[0].san, "e4");
        assert_eq!(tree.children[0].comment, "King's Pawn");
        assert_eq!(tree.children[0].children.len(), 3);

        // e5 line
        let e5 = &tree.children[0].children[0];
        assert_eq!(e5.san, "e5");
        assert_eq!(e5.comment, "Open Game");

        // c5 line (Sicilian)
        let c5 = &tree.children[0].children[1];
        assert_eq!(c5.san, "c5");
        assert_eq!(c5.comment, "Sicilian Defense");
        assert_eq!(c5.nags, vec![Nag::GOOD_MOVE]);
        assert_eq!(c5.children.len(), 2);

        // e6 line (French)
        let e6 = &tree.children[0].children[2];
        assert_eq!(e6.san, "e6");
        assert_eq!(e6.comment, "French Defense");
    }

    #[test]
    fn test_trailing_comma_in_children() {
        let tree = game_tree! {
            e4 {
                e5,
                c5,
            }
        };
        assert_eq!(tree.children[0].children.len(), 2);
    }

    #[test]
    fn test_trailing_comma_in_properties() {
        let tree = game_tree! {
            e4 (comment: "test", nag: GOOD_MOVE,)
        };
        assert_eq!(tree.children[0].comment, "test");
        assert_eq!(tree.children[0].nags, vec![Nag::GOOD_MOVE]);
    }
}
