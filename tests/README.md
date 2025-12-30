# pgnq Test Suite

This directory contains integration and unit tests for the pgnq PGN parser and transformer.

## Testing Philosophy

The test suite uses simple, direct assertions for testing. For complex tree structure assertions, the `game_tree!` macro provides a clean declarative syntax.

## Quick Start

```rust
mod common;

use common::parse_pgn;
use pgnq::nag::Nag;

#[test]
fn test_ruy_lopez() {
    let tree = parse_pgn("1. e4! e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez} *");

    // Check result
    assert_eq!(tree.result, GameResult::Ongoing);

    // Check nodes
    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.nags.contains(&Nag::GOOD_MOVE));

    let bb5 = e4.find_path(&["e5", "Nf3", "Nc6", "Bb5"]).unwrap();
    assert!(bb5.comment.contains("Ruy Lopez"));
}
```

## game_tree! Macro

For testing tree structure with multiple nodes, use the `game_tree!` macro:

```rust
#[test]
fn test_variations() {
    let tree = parse_pgn("1. e4 e5 (1... c5) (1... d5) *");

    let expected = game_tree! {
        e4 {
            e5,
            c5,
            d5
        }
    };
    assert_contains_tree!(tree, expected);
}

#[test]
fn test_nags() {
    let tree = parse_pgn("1. e4! e5? *");

    let expected = game_tree! {
        e4 (nag: GOOD_MOVE) {
            e5 (nag: POOR_MOVE)
        }
    };
    assert_contains_tree!(tree, expected);
}
```

### game_tree! Syntax

```rust
game_tree! {
    move_name (properties) { children }
}
```

Where:
- `move_name` is either an identifier (`e4`) or string literal (`"O-O"`)
- `(properties)` is optional: `(comment: "text")`, `(nag: GOOD_MOVE)`, etc.
- `{ children }` is optional: nested moves separated by commas

### Examples

```rust
// Simple linear game
let tree = game_tree! { e4 { e5 { Nf3 { Nc6 } } } };

// With properties
let tree = game_tree! {
    e4 (comment: "King's Pawn", nag: GOOD_MOVE) {
        e5 { Nf3 }
    }
};

// Multiple NAGs on one move
let tree = game_tree! {
    e4 (nags: [GOOD_MOVE, WHITE_SLIGHT_ADVANTAGE]) {
        e5 (nags: [POOR_MOVE, BLACK_MODERATE_ADVANTAGE])
    }
};

// With variations (siblings)
let tree = game_tree! {
    e4 {
        e5 { Nf3 },
        c5 (comment: "Sicilian"),
        d5
    }
};

// String literals for special moves
let tree = game_tree! { e4 { e5 { Nf3 { Nc6 { "O-O" } } } } };
```

## Assertion Macros

```rust
// Subset matching - actual may have extra children/NAGs not in expected
// Comments and NAGs in expected must match exactly when specified
assert_contains_tree!(actual, expected);

// Exact equality - all properties must match exactly
assert_nodes_match!(actual_node, expected_node);

// Header verification - only checks specified headers
assert_headers!(tree, {
    "White" => "Carlsen",
    "Event" => "Championship",
});
```

### Matching Behavior

`assert_contains_tree!` uses **subset matching**:
- All nodes in `expected` must exist in `actual` at the same positions
- `actual` may have additional children not in `expected`
- Empty properties in `expected` are not checked
- **Non-empty properties must match exactly** (comments, NAGs)

This means you must use the full comment text:
```rust
// CORRECT - full comment text
e4 (comment: "The King's Pawn opening")

// WRONG - partial comment won't match
e4 (comment: "King's Pawn")
```

## Test Organization

```
tests/
├── README.md              # This file
├── common/
│   ├── mod.rs             # Test utilities: parse_pgn, count_nodes, etc.
│   ├── tree_macro.rs      # game_tree! macro
│   ├── comparison.rs      # node_contains, nodes_match
│   ├── macros.rs          # assert_contains_tree!, assert_nodes_match!, assert_headers!
│   └── cli.rs             # CLI testing helpers
├── parser_edge_cases.rs   # Parser feature tests
├── roundtrip.rs           # Serialization tests
├── real_world_pgn.rs      # Format compatibility
├── cli_integration.rs     # CLI command tests
└── error_handling.rs      # Malformed input tests
```

## Helper Functions (in common/mod.rs)

```rust
// Parse PGN, panic on failure
let tree = parse_pgn("1. e4 e5 *");

// Parse with Result
let result = try_parse_pgn("1. e4 e5 *");

// Count nodes (excluding root)
let count = count_nodes(&tree);

// Get main line as Vec<String>
let moves = main_line_moves(&tree);
```

## CLI Testing (in common/cli.rs)

```rust
// Fluent API for CLI testing
pgnq("convert")
    .arg("--format=minimal")
    .stdin("1. e4 e5 *")
    .run()
    .success()
    .stdout_contains("e4");
```

## When to Use What

**Use single assertions for:**
- Testing a single property (one comment, one NAG, one header)
- Checking counts or simple predicates
- When the test is about one specific thing

**Use `game_tree!` macro for:**
- Testing tree structure with multiple nodes
- When a single assertion replaces many individual checks
- When testing variations or complex hierarchies

## Inline PGN Constants

Tests should use inline PGN strings when possible for readability:

```rust
#[test]
fn test_brace_comments() {
    let tree = parse_pgn(r#"
[Event "Test"]
1. e4 {King's Pawn} e5 {Best reply} *
"#);

    let e4 = tree.root.find_child("e4").unwrap();
    assert!(e4.comment.contains("King's Pawn"));
}
```

For real-world PGN examples, see `real_world_pgn.rs`.
