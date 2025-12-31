# PGN Format Specification for pgnq

This document describes what PGN (Portable Game Notation) patterns pgnq can parse, which require heuristics, and which will produce errors.

---

## 1. Overview

pgnq parses PGN files with flexible format support:

- Files can mix `{brace comments}` and bare text comments
- Standard PGN and line-based formats can coexist in the same file
- **Default: Lenient mode** - applies heuristics for ambiguous patterns
- **Option: Strict mode** (`--strict`) - errors on any ambiguity

---

## 2. Three Pattern Categories

### Category 1: Unambiguous (Works in Both Modes)
Clear patterns that parse identically in strict and lenient modes.

### Category 2: Ambiguous (Requires Heuristics)
Patterns that could be interpreted multiple ways.
- **Lenient mode**: Applies heuristics to resolve
- **Strict mode**: Returns error with helpful message

### Category 3: Unparsable (Always Errors)
Fundamentally broken patterns that cannot be parsed.
Both modes return an error with context and suggestions.

---

## 3. Unambiguous Patterns (Category 1)

These patterns are clear and work in both modes:

### Standard PGN Elements
| Pattern | Meaning |
|---------|---------|
| `{brace comments}` | Always a comment |
| `(variations with moves)` | Always a variation |
| `1. e4` | Move number + move, always structural |
| `; text` | Semicolon to end of line is a comment |
| `!`, `?`, `$14` | NAGs after moves are annotations |
| `1-0`, `0-1`, `1/2-1/2`, `*` | Game result |

### Line-Based Bare Text
| First Token on Line | Classification |
|---------------------|----------------|
| `(` | Variation start |
| `)` | Variation end |
| Move number (e.g., `7.`) | Move line |
| Move (e.g., `Nf3`, `e5`) | Move line |
| Text word (e.g., `This`, `The`) | Comment line (entire line) |

### Mixed Format Example
This works in strict mode because each element is unambiguous:

```pgn
1. e4 {Standard comment} e5
This is a bare text comment on its own line.
2. Nf3 Nc6
(2... d6 {Philidor inside variation})
3. Bb5 *
```

---

## 4. Ambiguous Patterns (Category 2)

These require heuristics (lenient mode) or error (strict mode):

### Move-Like Token at Line Start

```
c7 is a great square for the knight.
```

- **Could be**: pawn move `c7` OR start of comment
- **Lenient**: If followed by non-move text, treat whole line as comment
- **Strict**: Error

### Parenthetical Move References in Prose

```
The Petrosian Variation (7.d5) leads to complex play.
```

- **Could be**: variation OR textual reference to a move
- **Lenient**: Collapse to comment if 1-2 moves surrounded by text
- **Strict**: Error

### List Markers That Look Like Variation Ends

```
Black has two plans: 1) push e5 2) play c5.
```

- **Could be**: `1)` closing a variation OR list marker in prose
- **Lenient**: Treat as comment text when in prose context
- **Strict**: Error

---

## 5. Unparsable Patterns (Category 3)

These always error in both modes:

### Unbalanced Parentheses

```
1. e4 (1... c5 2. Nf3
3. d4 *
```

Variation opened but never closed.

### Unclosed Brace Comment

```
1. e4 {This comment never closes
e5 2. Nf3 *
```

Brace opened but no matching `}`.

### Invalid Move Notation

```
1. e4 Xyz9 2. Nf3 *
```

`Xyz9` is not valid SAN (Standard Algebraic Notation).

### Unexpected Closing Parenthesis

```
1. e4 ) e5
```

Closing paren without an opening paren.

---

## 6. Parsing Rules

### Rule 1: Line Classification

The first token after a newline determines how the line is parsed:

| First Token | Line Type | Behavior |
|-------------|-----------|----------|
| `(` | Variation start | Opens new variation |
| `)` | Variation end | Closes current variation |
| Move number | Move line | Parse moves structurally |
| Move | Move line | Parse moves structurally |
| Text word | Comment line | ALL tokens until newline are comment |

### Rule 2: Comment Lines Are Complete

Once a line is classified as a comment, everything on that line is comment text. Move-like words, list markers, and parentheses are NOT interpreted structurally.

### Rule 3: Brace Comments Override Context

`{...}` is always a comment, anywhere it appears.

### Rule 4: Variations End at `)` or End of Move Line

- `)` at line start closes the variation
- `)` at end of a move line closes the variation

---

## 7. Error Messages

pgnq produces rich, informative error messages modeled on the Rust compiler style.

### Error Message Format

```
error[E002]: unclosed variation
  --> game.pgn:1:8
   |
 1 |   1. e4 (1... c5 2. Nf3
   |         ^---- variation starts here
 2 |   3. d4 *
   |         ^ expected ')' before end of game
   |
   = note: Variation was opened at line 1, column 8
   = help: Add ')' to close the variation, or remove the opening '('
```

### What's Included

| Component | Description |
|-----------|-------------|
| Error code | `E001`, `E002`, etc. for programmatic handling |
| File path | Which file contains the error |
| Line:column | Exact position of the problem |
| Source context | Surrounding lines with visual highlighting |
| Plain English | What went wrong |
| Suggestions | How to fix it |

### Color Scheme

- **Red**: The problematic token/pattern
- **Cyan**: Line numbers and file path
- **Yellow**: Secondary markers
- **Bold white**: Error title

### Error Codes

| Code | Error Type |
|------|------------|
| E001 | Unclosed brace comment |
| E002 | Unclosed variation (unbalanced parentheses) |
| E003 | Unexpected closing parenthesis |
| E004 | Unexpected token |
| E005 | Invalid move notation |
| E010 | Ambiguous: move-like token at line start (strict mode) |
| E011 | Ambiguous: parenthetical move reference (strict mode) |
| E012 | Ambiguous: list marker could be variation end (strict mode) |

---

## 8. Lenient Mode Heuristics

| Pattern | Heuristic Applied |
|---------|-------------------|
| Move-like at line start + text | Treat whole line as comment |
| `(move)` surrounded by text | Collapse to comment |
| List markers `1)` `2)` | Treat as text in comment context |

---

## 9. References

- [Official PGN Specification](http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm) - The original 1994 standard
- [Chess.com PGN Guide](https://www.chess.com/terms/chess-pgn)
