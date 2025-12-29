# pgnq Test Suite

This directory contains integration and unit tests for the pgnq PGN parser and transformer.

## Testing Philosophy

The test suite uses a **DSL (Domain Specific Language)** for building expected tree structures and comparing them against parsed PGN. This approach:

1. **Separates parsing from assertions** - Build expected trees declaratively, then compare
2. **Enables comprehensive matching** - Verify entire tree structures, not just individual properties
3. **Supports partial matching** - Check only the properties you care about
4. **Provides clear error messages** - Shows exactly where trees differ

## Quick Start

```rust
use crate::common::*;
use crate::dsl::*;

#[test]
fn test_sicilian_defense() {
    let actual = parse_pgn("1. e4 c5 {Sicilian Defense} 2. Nf3 *");

    let expected = TreeExpectation::new()
        .result(GameResult::Ongoing)
        .root("e4", |n| n
            .child("c5", |n| n
                .comment_contains("Sicilian")
                .leaf("Nf3")
            )
        );

    assert_tree_contains!(actual, expected);
}
```

## DSL Components

### TreeExpectation - Building Expected Trees

```rust
// Simple main line (no properties to check on individual nodes)
TreeExpectation::new()
    .result(GameResult::WhiteWins)
    .main_line(&["e4", "e5", "Nf3", "Nc6"])

// With headers
TreeExpectation::new()
    .header("Event", "World Championship")
    .header("White", "Fischer")
    .result(GameResult::WhiteWins)
    .main_line(&["c4", "e6"])

// With inline node properties using closures
TreeExpectation::new()
    .root("e4", |n| n
        .nag(Nag::GOOD_MOVE)
        .comment_contains("King's Pawn")
        .child("e5", |n| n
            .comment("Symmetrical response")
        )
    )
```

### NodeExpectation - Inline Node Properties

Properties are declared **inline** with node creation using closures. This keeps everything together and makes trees easy to read:

```rust
TreeExpectation::new()
    .root("e4", |n| n
        .nag(Nag::GOOD_MOVE)              // Has specific NAG
        .comment("Opening move")           // Exact comment match
        .comment_contains("Opening")       // Partial comment match
        .has_child("e5")                   // Has child with this SAN
        .children_count(2)                 // Exact number of children
        .has_variations()                  // Has at least one variation
        .child("e5", |n| n                 // Continue to child
            .child("Nf3", |n| n)
        )
        .variation("c5", |n| n             // Add variation
            .comment("Sicilian")
        )
    )
```

### Comparison Macros

```rust
// Subset matching - actual may have extra properties
assert_tree_contains!(actual, expected);

// Exact equality - all properties must match
assert_tree_eq!(actual, expected);
```

## Deep Tree Examples

### Inline Closures (Recommended)

Properties and children are declared together, making trees easy to read:

```rust
let expected = TreeExpectation::new()
    .root("e4", |n| n
        .nag(Nag::GOOD_MOVE)
        .child("e5", |n| n
            .child("Nf3", |n| n
                .child("Nc6", |n| n
                    .has_child("Bb5")
                )
            )
        )
    );
```

### Variations with `variation()`

Mark non-main-line children with `variation()`:

```rust
let expected = TreeExpectation::new()
    .root("e4", |n| n
        .child("e5", |n| n)                        // main line
        .variation("c5", |n| n.comment("Sicilian")) // variation
        .variation("e6", |n| n.comment("French"))   // variation
    );
```

### Complex Nested Example

```rust
// Testing: 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6
//          6. Be3 (6. Bg5 e6) e5 7. Nb3 *
let expected = TreeExpectation::new()
    .root("e4", |n| n
        .child("c5", |n| n
            .child("Nf3", |n| n
                .child("d6", |n| n
                    .child("d4", |n| n
                        .child("cxd4", |n| n
                            .child("Nxd4", |n| n
                                .child("Nf6", |n| n
                                    .child("Nc3", |n| n
                                        .child("a6", |n| n
                                            .comment("The Najdorf")
                                            .child("Be3", |n| n
                                                .child("e5", |n| n
                                                    .leaf("Nb3")
                                                )
                                            )
                                            .variation("Bg5", |n| n
                                                .leaf("e6")
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    );
```

### Shorthand for Linear Games

For mostly linear games without annotations, use `main_line`:

```rust
let expected = TreeExpectation::new()
    .main_line(&["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4"]);
```

## Comparison Functions

### tree_contains

Subset matching. The actual tree may have additional properties:
- Extra headers are allowed
- Extra children are allowed
- Only specified NAGs/comments must match

```rust
// This passes even if actual has more moves after Nc6
let expected = TreeExpectation::new()
    .main_line(&["e4", "e5", "Nf3", "Nc6"]);

assert_tree_contains!(actual, expected);
```

### trees_equal

Full structural equality. Both trees must have identical:
- Headers (all keys and values)
- Result
- All nodes (SAN, comments, NAGs, children structure)

```rust
let result = trees_equal(&actual, &expected);
match result {
    CompareResult::Match => println!("Trees are identical"),
    CompareResult::Mismatch(diffs) => {
        for diff in diffs {
            println!("At {}: expected {}, got {}", diff.path, diff.expected, diff.actual);
        }
    }
}
```

## Error Messages

When assertions fail, you get detailed diff output:

```
Tree comparison failed:
  Location: root -> e4 -> e5 -> Nf3

  Expected: NAG GOOD_MOVE (!)
  Actual: No NAGs

  Actual tree structure:
    e4 [!] {Opening}
    └── e5
        └── Nf3       ← missing NAG
```

## Test Organization

```
tests/
├── README.md              # This file
├── common/
│   └── mod.rs             # Shared fixtures and helpers
├── dsl/
│   ├── mod.rs             # DSL module exports
│   ├── expectation.rs     # TreeExpectation, NodeExpectation
│   ├── comparison.rs      # Comparison logic
│   ├── matcher.rs         # String/NAG matchers
│   └── macros.rs          # assert_tree_* macros
├── dsl_tests.rs           # DSL validation tests
├── parser_edge_cases.rs   # Parser feature tests
├── roundtrip.rs           # Serialization tests
├── real_world_pgn.rs      # Format compatibility
├── cli_integration.rs     # CLI command tests
└── error_handling.rs      # Malformed input tests
```

## Helper Macros

For quick assertions without building full expectations:

```rust
// Main line assertion
assert_main_line!(tree, ["e4", "e5", "Nf3", "Nc6"]);

// Header assertions
assert_headers!(tree, {
    "Event" => "Test",
    "White" => "Player1"
});

// NAG at specific path
assert_has_nag!(tree, ["e4"], Nag::GOOD_MOVE);

// Comment at specific path
assert_comment_contains!(tree, ["e4"], "Opening");

// Node count
assert_node_count!(tree, 4);

// Result
assert_result!(tree, GameResult::WhiteWins);

// Children assertions
assert_has_children!(tree, ["e4"]);
assert_has_variations!(tree, ["e4"]);
assert_children_count!(tree, ["e4"], 3);
```

## Existing Helpers (in common/mod.rs)

```rust
// Parse PGN, panic on failure
let tree = parse_pgn("1. e4 e5 *");

// Parse with Result
let result = try_parse_pgn("1. e4 e5 *");

// Count nodes (excluding root)
let count = count_nodes(&tree);

// Get main line as Vec<String>
let moves = main_line_moves(&tree);

// CLI testing (fluent API)
pgnq("convert")
    .arg("--format=minimal")
    .stdin("1. e4 e5 *")
    .run()
    .success()
    .stdout_contains("e4");
```
