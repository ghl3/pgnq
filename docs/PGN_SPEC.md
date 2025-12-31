# PGN Format Support in pgnq

This document describes the PGN (Portable Game Notation) format variants that pgnq can parse. PGN is the standard text format for recording chess games.

---

## Supported Format Types

pgnq supports several PGN format variants. These are conceptual groupings based on how the PGN is structured.

### 1. Standard PGN Format

The classic format used by most chess software. Comments are wrapped in `{braces}`, variations use `(parentheses)`.

```pgn
[Event "World Championship"]
[White "Carlsen"]
[Black "Caruana"]
[Result "1/2-1/2"]

1. e4 {The King's Pawn opening} e5 2. Nf3 Nc6 3. Bb5 {The Ruy Lopez} a6
(3... Nf6 {Berlin Defense}) 4. Ba4 Nf6 5. O-O 1/2-1/2
```

**Characteristics:**
- Comments in `{braces}` or after `;` (semicolon to end of line)
- Variations in `(parentheses)`
- Moves, comments, and variations can be on the same line
- Common in exports from: Chess.com, Lichess games, ChessBase, SCID

**Status:** Fully supported

---

### 2. Line-Based Format with Inline Comments

A variation where each move appears on its own line, with comments interspersed between moves.

```pgn
1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2
Welcome to the main line!
Na6
This knight move might look strange at first, but it has a clear purpose.
7. O-O
We continue with castling.
e5 *
```

**Characteristics:**
- Moves appear on their own lines (or at the start of lines)
- Comments appear on subsequent lines as plain text (no braces needed)
- Each line is either a "move line" or a "comment line"
- Variations still use `(parentheses)`

**Status:** Supported with line-based parsing rules (see below)

---

### 3. Line-Based Format with Variations

Combines the line-based structure with nested variations.

```pgn
1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 d6 5. Nf3 O-O 6. Be2 Na6 7. O-O
(7. Nd2
This move prevents the typical knight jump to c5.
c5
Black switches to a Benoni-style setup for two reasons: 1) the knight goes to c7
2) White's knight is stuck on d2.
8. d5
)
e5 *
```

**Characteristics:**
- Combines line-based structure with variations
- `(` at the start of a line begins a variation
- `)` on its own line (or after moves) ends a variation
- Comment lines can contain list markers like `1)` or `2)` without breaking the variation

**Status:** Supported with line-based parsing rules

---

### 4. Compact Single-Line Format

Everything on one or few lines, common in database exports.

```pgn
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 1-0
```

**Characteristics:**
- Minimal formatting, no comments
- All moves on a single line or few lines
- Common in large database exports

**Status:** Fully supported

---

### 5. Annotated Format with NAGs

Games with move quality annotations.

```pgn
1. e4! {Excellent opening move} e5 2. Nf3 Nc6 3. Bc4?! {The Italian - playable but not the best}
Nf6! 4. Ng5?? {A well-known blunder} d5! 5. exd5 Na5 $17 {Black is much better} *
```

**Characteristics:**
- NAGs (Numeric Annotation Glyphs) like `$17` or symbols like `!`, `?`, `!!`, `??`
- Common symbol meanings:
  - `!` = good move
  - `?` = poor move
  - `!!` = brilliant
  - `??` = blunder
  - `!?` = interesting
  - `?!` = dubious

**Status:** Fully supported (symbols converted to NAGs internally)

---

## Parsing Rules for Line-Based Formats

When parsing line-based PGN (formats 2 and 3 above), pgnq uses these rules:

### Line Classification

Each line is classified by its first token:

| First token on line | Classification |
|---------------------|----------------|
| `(` | Start of variation |
| `)` | End of variation |
| Move number (like `7.` or `12...`) | Move line |
| Move (like `Nf3`, `e5`, `O-O`) | Move line |
| Regular text (like `This`, `The`, `Black`) | Comment line |

### Comment Lines

When a line starts with regular text (not a move), everything on that line is treated as comment:
- Move-like words are NOT interpreted as moves
- List markers like `1)` or `2)` are NOT interpreted as variation endings
- Parentheses are NOT interpreted structurally

This allows natural prose to include chess notation without confusion:

```
This makes sense for two reasons: 1) the knight goes to c7 2) White is stuck.
After the exchange on e5, Black gets active play.
The move ...Nf6 is the main alternative.
```

### Move Lines

When a line starts with a move or move number:
- Process moves structurally
- If followed by text, switch to comment mode until end of line
- A `)` at the end of a move line closes the current variation

---

## Patterns That Parse Correctly

### List markers in comments
```
This is good for two reasons: 1) first reason 2) second reason.
```
The `1)` and `2)` are part of the comment, not variation endings.

### Move references in comments
```
After the typical ...e7-e5 advance, Black gets counterplay.
The plan involving 7.d5 is thematic.
```
Move-like text in comments is preserved as text.

### Variations ending after moves
```
8. d5 Nc7 9. O-O a6)
```
The `)` closes the variation even though it's on a line with moves.

### Nested variations
```
(7. Nd2 c5
(8. dxc5 Nxc5)
8. d5)
```
Inner and outer variations are correctly tracked.

---

## Patterns That May Not Parse Correctly

### Ambiguous: Move at start of comment
```
c7 is a great square for the knight.
```
`c7` looks like a pawn move. Currently parsed as a move, not comment.

**Workaround:** Rephrase to start with non-move text: `The c7 square is great for the knight.`

### Ambiguous: Single move on its own line
```
Interesting position.
a3
More commentary.
```
Is `a3` a move or a label? Currently parsed as a move.

### Unsupported: Unbalanced parentheses in comments
```
This is a comment (with an unclosed paren
8. d5
```
May incorrectly start a variation.

**Workaround:** Ensure parentheses in comments are balanced, or use brace comments.

---

## Format Detection

pgnq attempts to detect the format automatically:

| Indicator | Inferred Format |
|-----------|-----------------|
| All comments in `{braces}` | Standard PGN |
| Text on lines between moves | Line-based format |
| No comments at all | Compact format |

You can also specify the format explicitly if auto-detection fails.

---

## References

- [Official PGN Specification](http://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm) - The original 1994 standard
- [Chess.com PGN Guide](https://www.chess.com/terms/chess-pgn)
