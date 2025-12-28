# pgnq

A Unix-like command-line tool for querying and manipulating chess PGN files.

`pgnq` parses PGN (Portable Game Notation) files into tree structures, enabling powerful filtering, splitting, and transformation operations. Think of it as `jq` for chess games.

## Installation

```bash
cargo install pgnq
```

Or build from source:

```bash
git clone https://github.com/yourusername/pgnq
cd pgnq
cargo build --release
```

## Quick Start

```bash
# View info about a PGN file
pgnq info game.pgn

# Display the move tree
pgnq tree game.pgn

# Convert between formats
cat lichess_study.pgn | pgnq convert -F standard

# Extract a specific variation
pgnq extract game.pgn -p "e4/e5/Nf3/Nc6"

# Split a study into separate files
pgnq split study.pgn -s "e4/c5:sicilian" -s "e4/e5:open_game"
```

## Concepts

### The Game Tree

PGN files represent chess games as trees, not linear sequences. The main line is the primary sequence of moves, but variations (alternative moves) branch off at any point:

```
1. e4
├── 1... e5 (main line)
│   ├── 2. Nf3
│   │   └── 2... Nc6
│   └── 2. Bc4 (variation)
│       └── 2... Nf6
└── 1... c5 (variation: Sicilian)
    └── 2. Nf3
```

`pgnq` preserves this tree structure, allowing you to navigate, filter, and extract any part of it.

### Node Paths

Node paths specify locations in the game tree using a slash-separated syntax:

```
e4/e5/Nf3/Nc6          # Sequence of moves from root
e4/c5:1                # First variation after e4 (Sicilian)
e4/e5/Nf3:v2           # Second variation at Nf3
1.e4/1...e5/2.Nf3      # With move numbers (optional)
```

**Path Syntax:**
- `/` separates moves in the path
- `:N` or `:vN` selects the Nth variation (0 = main line)
- Move numbers are optional and ignored for matching

**Special Selectors:**
- `@root` - the root node
- `@end` - follow main line to the end
- `/**` - all descendants (glob)
- `/*` - direct children only

### Flexible PGN Parsing

`pgnq` uses a **liberal parser** that handles virtually any reasonable PGN:

| Feature | Supported Syntax |
|---------|------------------|
| **Brace comments** | `{ this is a comment }` |
| **Semicolon comments** | `; comment to end of line` |
| **Bare text comments** | Lines that don't look like moves become comments |
| **Mixed comments** | All styles can appear in the same file |
| **Variations** | `( )` parentheses, arbitrarily nested |
| **Headerless games** | Moves without the Seven Tag Roster |
| **Clock times** | `[%clk 1:30:00]` embedded in comments |
| **Evaluations** | `[%eval +0.45]` or `[%eval #3]` |
| **NAGs** | `$1`, `$2`, or symbols like `!`, `?`, `!!`, `??` |

**Philosophy**: If it looks like reasonable PGN, `pgnq` will parse it. No strict format enforcement.

## Commands

### `pgnq info`

Display information about a PGN file.

```bash
pgnq info game.pgn
pgnq info game.pgn --json
pgnq info game.pgn --headers-only
```

**Output:**
```
File: game.pgn

Headers:
  Event: World Championship
  White: Carlsen, Magnus
  Black: Nepomniachtchi, Ian
  Result: 1-0

Statistics:
  Nodes: 89
  Lines: 12
  Comments: 34
  Max depth: 45
```

### `pgnq tree`

Display the game tree visually.

```bash
pgnq tree game.pgn
pgnq tree game.pgn --depth 5
pgnq tree game.pgn --from-path "e4/e5"
pgnq tree game.pgn --show-comments --show-nags
pgnq tree game.pgn --ascii  # No Unicode box drawing
```

**Output:**
```
1. e4
├── 1... e5
│   ├── 2. Nf3
│   │   ├── 2... Nc6
│   │   │   └── 3. Bb5 {Ruy Lopez}
│   │   └── 2... Nf6 {Petrov Defense}
│   └── 2. Bc4
│       └── 2... Bc5
└── 1... c5 {Sicilian Defense}
    └── 2. Nf3
        └── ...
```

### `pgnq stats`

Show detailed statistics about a PGN file.

```bash
pgnq stats game.pgn
pgnq stats game.pgn --json
pgnq stats game.pgn --move-stats   # Move frequency
pgnq stats game.pgn --comment-stats # Comment analysis
```

**JSON Output:**
```json
{
  "total_nodes": 1247,
  "leaf_nodes": 89,
  "commented_nodes": 342,
  "max_depth": 45,
  "main_line_length": 28,
  "variation_count": 156,
  "nag_counts": {
    "!": 45,
    "?": 23,
    "!!": 12
  }
}
```

### `pgnq convert`

Convert between PGN formats with optional transformations.

```bash
# Format conversion
pgnq convert game.pgn -F lichess -o output.pgn
cat game.pgn | pgnq convert -F standard > clean.pgn

# Strip metadata
pgnq convert game.pgn --strip-clocks --strip-evals

# Remove elements
pgnq convert game.pgn --no-comments
pgnq convert game.pgn --no-variations
pgnq convert game.pgn --no-nags
pgnq convert game.pgn --no-headers

# Combine options
pgnq convert game.pgn -F standard --no-comments --strip-clocks
```

**Options:**
| Flag | Description |
|------|-------------|
| `-F, --format` | Output format: `standard`, `lichess`, `minimal` |
| `-o, --output` | Output file (default: stdout) |
| `--no-headers` | Omit PGN headers |
| `--no-comments` | Strip all comments |
| `--no-variations` | Keep main line only |
| `--no-nags` | Remove NAG annotations |
| `--strip-clocks` | Remove `[%clk]` from comments |
| `--strip-evals` | Remove `[%eval]` from comments |
| `--line-width N` | Wrap lines at N characters (default: 80) |

### `pgnq extract`

Extract a subtree starting at a specific path.

```bash
pgnq extract game.pgn -p "e4/c5/Nf3/d6"
pgnq extract game.pgn -p "e4/e5/Nf3" --with-prefix
pgnq extract game.pgn -p "e4/c5:1" -o sicilian.pgn
```

**Options:**
| Flag | Description |
|------|-------------|
| `-p, --path` | Node path to extract (required) |
| `-o, --output` | Output file (default: stdout) |
| `-F, --format` | Output format |
| `--with-prefix` | Include moves from root to extraction point |
| `--with-headers` | Include headers from original file |

### `pgnq split`

Split a PGN file at multiple node paths.

```bash
pgnq split study.pgn \
  -s "e4/c5/Nf3/d6:najdorf" \
  -s "e4/c5/Nf3/Nc6/Bb5:rossolimo" \
  -s "e4/c5/Nc3:closed" \
  -o chapters/

pgnq split game.pgn -s "e4:white_openings" --include-prefix
pgnq split game.pgn -s "e4/e5/@end:endgame" --dry-run
```

**Options:**
| Flag | Description |
|------|-------------|
| `-s, --split PATH:NAME` | Split spec (can be repeated) |
| `-o, --output-dir` | Output directory (default: `output/`) |
| `-F, --format` | Output format for split files |
| `--include-prefix` | Include path from root to split point |
| `--dry-run` | Show what would be created without writing |

**Output:**
```
Splitting study.pgn...
  najdorf.pgn       (245 nodes) OK
  rossolimo.pgn     (189 nodes) OK
  closed.pgn        (312 nodes) OK

Created 3 files in chapters/
```

### `pgnq filter`

Filter nodes matching specific criteria.

```bash
# By path pattern
pgnq filter game.pgn -p "e4/e5/**"
pgnq filter game.pgn -p "e4/*"

# By properties
pgnq filter game.pgn --has-comment
pgnq filter game.pgn --has-nag "!"
pgnq filter game.pgn --min-depth 10 --max-depth 20

# Main line only
pgnq filter game.pgn --main-line

# Invert filter
pgnq filter game.pgn --has-comment --invert
```

**Options:**
| Flag | Description |
|------|-------------|
| `-p, --path` | Path pattern to match |
| `--has-comment` | Nodes with comments |
| `--has-nag NAG` | Nodes with specific NAG |
| `--min-depth N` | Minimum tree depth |
| `--max-depth N` | Maximum tree depth |
| `--main-line` | Main line nodes only |
| `--invert` | Invert the filter |
| `-F, --format` | Output format |

## Piping and Composition

`pgnq` is designed for Unix pipelines:

```bash
# Chain operations
cat game.pgn | pgnq convert -F standard | pgnq filter --main-line

# Process multiple files
for f in *.pgn; do pgnq stats "$f" --json; done | jq -s '.'

# Extract and convert in one pipeline
pgnq extract study.pgn -p "e4/c5" | pgnq convert -F lichess > sicilian.pgn

# Use with other tools
pgnq tree game.pgn | head -20
pgnq stats game.pgn --json | jq '.total_nodes'
```

## PGN Format Reference

### Headers (Tag Pairs)

```
[Event "World Championship"]
[Site "Dubai"]
[Date "2021.12.03"]
[Round "6"]
[White "Carlsen, Magnus"]
[Black "Nepomniachtchi, Ian"]
[Result "1-0"]
```

The Seven Tag Roster (Event, Site, Date, Round, White, Black, Result) is standard. Additional tags are preserved.

### Comments

**Standard format (brace comments):**
```
1. e4 {Best by test} e5 2. Nf3 {Attacking the e5 pawn}
```

**Lichess format (line comments):**
```
1. e4
Best by test
1... e5
2. Nf3
Attacking the e5 pawn
```

**Semicolon comments (end of line):**
```
1. e4 e5 ; Open game
2. Nf3
```

### Variations (RAVs)

Recursive Annotation Variations use parentheses:

```
1. e4 e5 (1... c5 {Sicilian} 2. Nf3) 2. Nf3 Nc6 (2... Nf6 {Petrov})
```

Variations can nest arbitrarily deep.

### NAGs (Numeric Annotation Glyphs)

| Symbol | NAG | Meaning |
|--------|-----|---------|
| `!` | `$1` | Good move |
| `?` | `$2` | Mistake |
| `!!` | `$3` | Brilliant move |
| `??` | `$4` | Blunder |
| `!?` | `$5` | Interesting move |
| `?!` | `$6` | Dubious move |
| `=` | `$10` | Equal position |
| `+=` | `$14` | White slightly better |
| `=+` | `$15` | Black slightly better |
| `+-` | `$18` | White winning |
| `-+` | `$19` | Black winning |

### Clock and Evaluation Commands

Embedded in comments using `[%command value]` syntax:

```
1. e4 {[%clk 1:29:45]} e5 {[%clk 1:29:30] [%eval +0.15]}
2. Nf3 {[%emt 0:00:12] [%eval +0.22]}
```

| Command | Format | Description |
|---------|--------|-------------|
| `%clk` | `h:mm:ss` | Clock time remaining |
| `%emt` | `h:mm:ss` | Elapsed move time |
| `%eval` | `+0.00` or `#N` | Position evaluation |

## Multi-Game Files

PGN files can contain multiple games, separated by blank lines. Use `--game N` to select a specific game (1-indexed), or process all games:

```bash
# Info on specific game
pgnq info tournament.pgn --game 5

# Process all games
pgnq stats tournament.pgn --all --json

# Extract game 3
pgnq convert tournament.pgn --game 3 -o game3.pgn
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Parse error |
| 2 | Path not found |
| 3 | Invalid arguments |
| 4 | I/O error |

## Examples

### Opening Repertoire Management

```bash
# Split a study into chapter files
pgnq split repertoire.pgn \
  -s "e4/e5/Nf3/Nc6/Bb5:ruy_lopez" \
  -s "e4/e5/Nf3/Nc6/Bc4:italian" \
  -s "e4/c5:sicilian" \
  -o openings/

# Count lines in each opening
for f in openings/*.pgn; do
  echo "$f: $(pgnq stats "$f" --json | jq '.leaf_nodes') lines"
done
```

### Clean Up Annotated Games

```bash
# Remove engine analysis, keep human comments
pgnq convert annotated.pgn --strip-evals --strip-clocks -o clean.pgn

# Create minimal PGN for sharing
pgnq convert game.pgn --no-variations --no-comments -F minimal
```

### Analyze Study Structure

```bash
# Find all heavily annotated positions
pgnq filter study.pgn --has-comment --has-nag "!" | pgnq tree

# Get statistics on variation depth
pgnq stats study.pgn --json | jq '{
  total: .total_nodes,
  mainline: .main_line_length,
  avg_depth: (.total_nodes / .leaf_nodes)
}'
```

### Convert Between Platforms

```bash
# Lichess study to standard PGN
pgnq convert lichess_export.pgn -F standard -o chessbase.pgn

# Standard PGN to Lichess format
cat chessbase.pgn | pgnq convert -F lichess > lichess_import.pgn
```

## License

MIT
