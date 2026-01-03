# Parser Implementation Guide

This document describes how the PGN parser works internally. For information about what patterns can be parsed and parsing modes, see [PGN_SPEC.md](../../docs/PGN_SPEC.md).

## Overview

The parser uses a three-phase pipeline:

```
Input String
     │
     ▼
┌─────────────────────────────────────┐
│ Phase 1: Lexer (lexer.rs)           │
│ - Regex-based tokenization (logos)  │
│ - Location tracking (line, column)  │
│ - Mode-agnostic                     │
└─────────────────────────────────────┘
     │ Vec<LocatedToken>
     ▼
┌─────────────────────────────────────┐
│ Phase 2: Token Post-Processing      │
│ (token_postprocess.rs)              │
│ - Lenient mode only                 │
│ - collapse_prose: merges            │
│   parenthetical refs into BareText  │
└─────────────────────────────────────┘
     │ Vec<LocatedToken>
     ▼
┌─────────────────────────────────────┐
│ Phase 3: Builder (builder.rs)       │
│ - State machine (ParseContext)      │
│ - Tree construction                 │
│ - Prose detection                   │
│ - Variation handling                │
└─────────────────────────────────────┘
     │
     ▼
  GameTree
```

**Key source files:**

- `token.rs` - Token type definitions using the `logos` crate
- `lexer.rs` - Phase 1: Pure tokenization with location tracking
- `token_postprocess.rs` - Phase 2: Token stream transformations
- `builder.rs` - Phase 3: State machine and tree construction
- `mod.rs` - Pipeline orchestration, public API

## Phase 1: Lexer

### Token Types (`token.rs`)

The lexer uses the [logos](https://crates.io/crates/logos) crate for regex-based tokenization. Tokens are defined with the `#[derive(Logos)]` macro:

```rust
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]  // Skip whitespace
pub enum Token {
    #[regex(r#"\[[A-Za-z_][A-Za-z0-9_]*\s+"[^"]*"\]"#, priority = 10)]
    Header(String),

    #[regex(r"[0-9]+\.+", priority = 8)]
    MoveNumber(String),

    // ... etc
}
```

**Token categories:**

| Category     | Tokens                                          | Priority |
| ------------ | ----------------------------------------------- | -------- |
| Structure    | `Header`, `VariationStart`, `VariationEnd`      | 10       |
| Results      | `WhiteWins`, `BlackWins`, `Draw`, `Ongoing`     | 9        |
| Castling     | `CastleShort`, `CastleLong`                     | 9        |
| Move numbers | `MoveNumber`                                    | 8        |
| NAGs         | `Nag`, symbolic NAGs (`Bang`, `Question`, etc.) | 5-8      |
| Moves        | `PieceMove`, `PawnMove`                         | 7        |
| Comments     | `BraceComment`, `SemicolonComment`              | 10       |
| Prose        | `BareText`, `Digit`, `Punctuation`, `Newline`   | 1-3      |

Priority determines which pattern matches when multiple could apply. Higher priority wins.

### Location Tracking (`lexer.rs`)

Each token is wrapped with location information:

```rust
pub struct LocatedToken {
    pub token: Token,
    pub line: usize,      // 1-indexed
    pub column: usize,    // 1-indexed
    pub offset: usize,    // Byte offset in source
    pub len: usize,       // Token length in bytes
}
```

This enables precise error messages pointing to exact positions in the source.

## Phase 2: Token Post-Processing

After tokenization, the token stream can optionally be transformed to handle ambiguous patterns found in real-world PGN files. This post-processing is **only applied in lenient mode** (the default); in strict mode, tokens are passed through unchanged.

### `collapse_prose` (`token_postprocess.rs`)

This function scans for variation tokens (`VariationStart` ... `VariationEnd`) that appear to be parenthetical move references embedded within prose. When detected, it merges these tokens back into a single `BareText` token, preserving them as comment text rather than game tree structure.

**How it identifies parenthetical references:**

The key insight is that parenthetical move references are _surrounded by prose_. A real variation like `1. e4 e5 (1... c5) 2. Nf3` appears in structured movetext, while a reference like `"the Italian (3.Bc4) is solid"` appears mid-sentence.

The function checks four conditions:

1. **Preceded by prose** — A `BareText` token exists before the `(`, indicating we're in narrative text rather than structured movetext.

2. **No structural tokens between** — All tokens between that `BareText` and the `(` are also `BareText` or `Newline`. If a move or move number appears between the prose and the parenthesis (like `"after e4 (d5)"`), this isn't a simple parenthetical reference.

3. **Brief content** — The parentheses contain at most 2 move tokens. Real variations typically have more moves; parenthetical references are brief mentions like `(7.d5)` or `(3.Bc4 Bc5)`.

4. **Followed by prose** — A `BareText` token immediately follows the `)`. This confirms the parenthetical is embedded mid-sentence, not at line end or before movetext.

When all conditions are met, the variation tokens are replaced with a single `BareText` containing the original text (e.g., `"(7.d5)"`).

**Examples:**

_Merged (all conditions met):_

```
Input: "the Petrosian Variation (7.d5) is avoided"

Before: [..., BareText("Variation"), VariationStart, MoveNumber("7."),
         PawnMove("d5"), VariationEnd, BareText("is"), ...]

After:  [..., BareText("Variation"), BareText("(7.d5)"), BareText("is"), ...]
```

_Not merged (preceded by move, not prose):_

```
Input: "1. e4 e5 (1... c5 2. Nf3 d6) 2. Nf3"

The `(` is preceded by PawnMove("e5"), not BareText.
Also contains 3 moves, exceeding the limit.
→ Tokens unchanged; parsed as a real variation.
```

_Not merged (not followed by prose):_

```
Input: "White plays (3.Bc4)"

The `)` is followed by end-of-input, not BareText.
→ Tokens unchanged.
```

_Not merged (move intervenes before parenthesis):_

```
Input: "after e4 (d5) Black is fine"

PawnMove("e4") appears between BareText("after") and VariationStart.
→ Tokens unchanged.
```

---

**Note:** This post-processing only handles parenthetical references. Distinguishing move-like tokens _within_ prose (e.g., `"the pawn on f3 is weak"`) is handled later by the builder's state machine.

## Phase 3: Builder

### State Machine (`builder.rs`)

The builder uses a state machine to distinguish real moves from move-like text in prose:

```rust
enum ParseContext {
    ExpectingMoves { remaining: u8 },  // After move number, expect N moves
    BetweenMoves,                       // Default state
    InProse,                            // Move tokens are references, not real
}
```

**State transitions:**

```
                        MoveNumber("1.")
                              │
                              ▼
    ┌──────────────────► ExpectingMoves{2}
    │                         │
    │                    Move consumed
    │                         │
    │                         ▼
    │                   ExpectingMoves{1}
    │                         │
    │                    Move consumed
    │                         │
    │                         ▼
    │                    BetweenMoves ◄───── Newline ─────┐
    │                         │                           │
    │                    BareText or                      │
    │                    Punctuation                      │
    │                         │                           │
    │                         ▼                           │
    └────────────────────  InProse  ──────────────────────┘
```

**Key behaviors:**

- `BareText` → enters `InProse` context
- `Newline` → exits `InProse` context (returns to `BetweenMoves`)
- Move tokens in `InProse` context → appended as comment text, not added as tree nodes

### Builder State

The `BuilderState` struct tracks all parsing context:

```rust
struct BuilderState {
    current_path: NodePath,           // Path from root to current position
    return_stack: Vec<(...)>,         // Saved state for variation returns
    variation_starts: Vec<...>,       // For error reporting
    pending_comment: String,          // Comment awaiting attachment
    pending_nags: Vec<Nag>,           // NAGs awaiting attachment
    current_move_number: u16,
    expect_black: bool,               // Next move is Black's?
    context: ParseContext,            // State machine state
    mode: ParseMode,
    // ...
}
```

### Variation Handling

Variations use a stack-based approach:

1. **On `(`**: Save current state to `return_stack`, set `pending_variation_pop = true`
2. **First move in variation**: Determine if replacement or response:
   - If move color matches saved `expect_black` → **response** (child of current position)
   - If move color differs → **replacement** (sibling, pop path first)
3. **On `)`**: Restore saved state from `return_stack`

Example:

```
1. e4 e5 (1... c5 2. Nf3) 2. Bc4
         ▲
         │
After e5, expect_black=false. Variation starts.
First move is c5 (Black). Black ≠ saved expect_black (false)?
Yes → c5 is a REPLACEMENT for e5 (sibling under e4)
```

### Token Handlers

Each token type has a handler method in `BuilderState`:

| Token            | Handler                  | Key Logic                                                     |
| ---------------- | ------------------------ | ------------------------------------------------------------- |
| `Header`         | `handle_header`          | Parse key/value, store in `tree.headers`                      |
| `MoveNumber`     | `handle_move_number`     | Set `current_move_number`, enter `ExpectingMoves`             |
| `Digit`          | `handle_digit`           | Always treat as prose (list markers like "1)")                |
| `*Move`          | `handle_move`            | If `InProse`, append to comment; else add tree node           |
| `BareText`       | `handle_bare_text`       | Append to comment, enter `InProse`                            |
| `Punctuation`    | `handle_punctuation`     | Commas/semicolons enter prose; dashes ignored if not in prose |
| `VariationStart` | `handle_variation_start` | Push state to return stack                                    |
| `VariationEnd`   | `handle_variation_end`   | Pop state; error if unmatched (strict) or ignore (lenient)    |
| `Newline`        | `handle_newline`         | Exit `InProse` context                                        |

## Data Structures

### GameTree (`tree/game.rs`)

```rust
pub struct GameTree {
    pub headers: HashMap<String, String>,  // PGN headers
    pub root: GameNode,                     // Root node (empty move)
    pub result: GameResult,                 // 1-0, 0-1, 1/2-1/2, *
}
```

### GameNode (`tree/node.rs`)

```rust
pub struct GameNode {
    pub san: String,              // Move in SAN ("e4", "Nf3", "O-O")
    pub move_number: Option<u16>, // 1, 2, 3, ...
    pub is_black: bool,
    pub comment: String,
    pub nags: Vec<Nag>,
    pub children: Vec<GameNode>,  // First = main line, rest = variations
}
```

**Tree structure:**

- Root node has empty `san` and `move_number = None`
- First child is always the main line
- Additional children are alternative variations
- Navigation: `find_child("e4")`, `find_path(&["e4", "e5", "Nf3"])`

## Step-by-Step Walkthrough

Let's trace parsing this PGN:

```pgn
[Event "Test"]
1. e4 {Best move} e5 (1... c5 2. Nf3) 2. Nf3 *
```

### Step 1: Lexer Output

```
Token::Header("[Event \"Test\"]")     line=1, col=1
Token::MoveNumber("1.")               line=2, col=1
Token::PawnMove("e4")                 line=2, col=4
Token::BraceComment("Best move")      line=2, col=7
Token::PawnMove("e5")                 line=2, col=20
Token::VariationStart                 line=2, col=23
Token::MoveNumber("1...")             line=2, col=24
Token::PawnMove("c5")                 line=2, col=29
Token::MoveNumber("2.")               line=2, col=32
Token::PieceMove("Nf3")               line=2, col=35
Token::VariationEnd                   line=2, col=38
Token::MoveNumber("2.")               line=2, col=40
Token::PieceMove("Nf3")               line=2, col=43
Token::Ongoing                        line=2, col=47
```

### Step 2: Builder Processing

| Token                | State Before        | Action                                                     | State After         |
| -------------------- | ------------------- | ---------------------------------------------------------- | ------------------- |
| `Header`             | `BetweenMoves`      | Store "Event"="Test"                                       | `BetweenMoves`      |
| `MoveNumber("1.")`   | `BetweenMoves`      | Set move_num=1, expect_black=false                         | `ExpectingMoves{2}` |
| `PawnMove("e4")`     | `ExpectingMoves{2}` | Add node, path=[0], expect_black=true                      | `ExpectingMoves{1}` |
| `BraceComment`       | `ExpectingMoves{1}` | Attach "Best move" to e4                                   | `ExpectingMoves{1}` |
| `PawnMove("e5")`     | `ExpectingMoves{1}` | Add node as child of e4, path=[0,0]                        | `BetweenMoves`      |
| `VariationStart`     | `BetweenMoves`      | Push state (path=[0,0], expect_black=false)                | `BetweenMoves`      |
| `MoveNumber("1...")` | `BetweenMoves`      | Set move_num=1, expect_black=true                          | `ExpectingMoves{1}` |
| `PawnMove("c5")`     | `ExpectingMoves{1}` | Color=Black, saved=false → replacement, pop to [0], add c5 | `BetweenMoves`      |
| `MoveNumber("2.")`   | `BetweenMoves`      | Set move_num=2                                             | `ExpectingMoves{2}` |
| `PieceMove("Nf3")`   | `ExpectingMoves{2}` | Add as child of c5, path=[0,1,0]                           | `ExpectingMoves{1}` |
| `VariationEnd`       | `ExpectingMoves{1}` | Pop: restore path=[0,0], expect_black=false                | (restored)          |
| `MoveNumber("2.")`   | `BetweenMoves`      | Set move_num=2                                             | `ExpectingMoves{2}` |
| `PieceMove("Nf3")`   | `ExpectingMoves{2}` | Add as child of e5, path=[0,0,0]                           | `ExpectingMoves{1}` |
| `Ongoing`            | `ExpectingMoves{1}` | Set result=Ongoing                                         | `ExpectingMoves{1}` |

### Step 3: Final Tree Structure

```
root
└── e4 (comment: "Best move")
    ├── e5 (main line)
    │   └── Nf3
    └── c5 (variation)
        └── Nf3
```

## Error Handling

### Parse Modes

```rust
pub enum ParseMode {
    Lenient,  // Default: apply heuristics, liberal with input
    Strict,   // Error on ambiguous patterns
}
```

**Lenient mode differences:**

- Extra `)` silently ignored
- Embedded variations collapsed to text
- Parentheses in prose context treated as text

**Strict mode differences:**

- Errors on unmatched `)`
- Parses all parentheses literally as variations
- No embedded variation collapse

### Error Context

Errors include rich context for debugging:

```rust
pub struct ParseError {
    pub code: ErrorCode,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub span_len: usize,
    pub source_context: Vec<SourceLine>,  // Surrounding lines
    pub notes: Vec<String>,
    pub help: Vec<String>,
}
```

Error codes: `E001` (unclosed brace), `E002` (unclosed variation), `E003` (unexpected `)`)

## Public API

Entry points in `mod.rs`:

```rust
// Single game
pub fn parse(input: &str) -> Result<GameTree>
pub fn parse_with_options(input: &str, mode: ParseMode, file: Option<PathBuf>) -> Result<GameTree>

// Multiple games
pub fn parse_all(input: &str) -> Result<Vec<GameTree>>
pub fn parse_all_with_options(...) -> Result<Vec<GameTree>>

// Low-level access
pub fn tokenize(input: &str) -> Vec<LocatedToken>
pub fn build_tree(tokens: &[LocatedToken]) -> Result<GameTree>
```
