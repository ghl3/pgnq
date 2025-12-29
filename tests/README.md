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
fn test_ruy_lopez() {
    let actual = parse_pgn("1. e4! e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez} *");

    // Path-based API - clean and readable
    let expected = TreeExpectation::new()
        .ongoing()
        .line(&["e4", "e5", "Nf3", "Nc6", "Bb5"])
        .at(&["e4"]).nag(Nag::GOOD_MOVE)
        .at(&["e4", "e5", "Nf3", "Nc6", "Bb5"]).comment_contains("Ruy Lopez")
        .build();

    assert_tree_contains!(actual, expected);
}
```

## DSL Components

### Path-based API (Recommended)

The path-based API avoids nested closures and provides a flat, readable structure:

```rust
// Annotate nodes at specific paths
TreeExpectation::new()
    .ongoing()
    .line(&["e4", "e5", "Nf3", "Nc6"])          // Main line exists
    .at(&["e4"]).nag(Nag::GOOD_MOVE)             // e4 has good move NAG
    .at(&["e4", "e5"]).comment_contains("reply") // e5 has comment
    .at(&["e4", "e5", "Nf3"]).has_nag()          // Nf3 has some NAG
    .build()

// Specify children/variations at a path
TreeExpectation::new()
    .at(&["e4"]).children(&["e5", "c5", "e6", "d5"]).has_variations()
    .at(&["e4", "c5"]).comment_contains("Sicilian")
    .build()
```

### TreeExpectation - Building Expected Trees

```rust
// Simple main line
TreeExpectation::new()
    .white_wins()
    .main_line(&["e4", "e5", "Nf3", "Nc6"])

// With headers
TreeExpectation::new()
    .header("Event", "World Championship")
    .header("White", "Fischer")
    .white_wins()
    .line(&["c4", "e6"])
    .build()

// Deep annotations without pyramid of doom
TreeExpectation::new()
    .ongoing()
    .line(&["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4"])
    .at(&["e4"]).nag(Nag::GOOD_MOVE)
    .at(&["e4", "e5", "Nf3", "Nc6", "Bb5"]).comment_contains("Ruy Lopez")
    .at(&["e4", "e5", "Nf3", "Nc6", "Bb5", "a6"]).comment_contains("Morphy")
    .build()
```

### Comparison Macros

```rust
// Subset matching - actual may have extra properties
assert_tree_contains!(actual, expected);

// Exact equality - all properties must match
assert_tree_eq!(actual, expected);
```

## Path-based API Examples

### Simple Annotations

```rust
let expected = TreeExpectation::new()
    .ongoing()
    .line(&["e4", "e5", "Nf3", "Nc6"])
    .at(&["e4"]).nag(Nag::GOOD_MOVE).comment_contains("Opening")
    .at(&["e4", "e5"]).nag(Nag::POOR_MOVE)
    .build();
```

### Variations

```rust
// Check that e4 has multiple children (variations)
let expected = TreeExpectation::new()
    .at(&["e4"]).children(&["e5", "c5", "e6", "d5"]).has_variations()
    .build();
```

### Deep Trees Without Pyramid of Doom

```rust
// Old closure style (avoid for deep trees):
TreeExpectation::new()
    .root("e4", |n| n
        .nag(Nag::GOOD_MOVE)
        .child("e5", |n| n
            .child("Nf3", |n| n
                .child("Nc6", |n| n
                    .child("Bb5", |n| n
                        .comment_contains("Ruy Lopez")
                    )
                )
            )
        )
    )

// New path style (preferred):
TreeExpectation::new()
    .line(&["e4", "e5", "Nf3", "Nc6", "Bb5"])
    .at(&["e4"]).nag(Nag::GOOD_MOVE)
    .at(&["e4", "e5", "Nf3", "Nc6", "Bb5"]).comment_contains("Ruy Lopez")
    .build()
```

## Closure API (Legacy)

The closure-based API is still available for complex cases where you need to build nested structures inline:

### Inline Closures

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
