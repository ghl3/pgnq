# pgnq

A Unix-like command-line tool for querying and manipulating chess PGN files.

`pgnq` parses PGN (Portable Game Notation) files into tree structures, enabling powerful filtering, splitting, and transformation operations. Think of it as `jq` for chess games.

**Flexible parsing**: Works with PGN exports from Lichess, Chess.com, ChessBase, and other sources. Handles brace comments, semicolon comments, bare text annotations, clock times, evaluations, and nested variations—no strict format enforcement.

## Commands

| Command | Description |
|---------|-------------|
| [`info`](#pgnq-info) | Display file metadata and statistics |
| [`tree`](#pgnq-tree) | Visualize the game tree |
| [`stats`](#pgnq-stats) | Detailed statistics in JSON format |
| [`convert`](#pgnq-convert) | Transform between PGN formats |
| [`extract`](#pgnq-extract) | Extract a subtree at a specific path |
| [`split`](#pgnq-split) | Split into multiple files at node paths |
| [`filter`](#pgnq-filter) | Filter nodes by criteria |
| [`merge`](#pgnq-merge) | Combine multiple files into one tree |

## Examples

### Opening Repertoire Management

```bash
# Merge separate opening files into one repertoire
pgnq merge sicilian.pgn french.pgn caro_kann.pgn -o black_vs_e4.pgn

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

# Build a complete repertoire from multiple sources
pgnq merge openings/*.pgn | pgnq convert -F lichess -o complete_repertoire.pgn
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

## Installation

```bash
cargo install pgnq
```

Or build from source:

```bash
git clone https://github.com/ghl3/pgnq
cd pgnq
cargo build --release
```

## Command Reference

### Path Syntax

Several commands (`extract`, `split`, `filter`) use node paths to specify locations in the game tree:

```
e4/e5/Nf3/Nc6          # Sequence of moves from root
e4/c5:1                # First variation after e4 (Sicilian)
e4/e5/Nf3:v2           # Second variation at Nf3
1.e4/1...e5/2.Nf3      # With move numbers (optional, ignored for matching)
```

**Syntax:**
- `/` separates moves in the path
- `:N` or `:vN` selects the Nth variation (0 = main line)

**Special Selectors:**
- `@root` - the root node
- `@end` - follow main line to the end
- `/**` - all descendants (glob)
- `/*` - direct children only

### `pgnq info`

Display metadata and summary statistics about a PGN file. Shows the Seven Tag Roster headers (Event, Site, Date, Round, White, Black, Result) plus any additional headers, along with basic tree statistics like node count, number of lines (leaf nodes), comment count, and maximum depth.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format for programmatic use |
| `--headers-only` | Show only headers, skip statistics |
| `--game N` | Select a specific game from a multi-game file (1-indexed) |

**Examples:**

```bash
# Basic info about a game
pgnq info game.pgn

# Get info in JSON format for scripting
pgnq info game.pgn --json

# Show just the headers (skip statistics)
pgnq info game.pgn --headers-only

# Info about the 5th game in a tournament file
pgnq info tournament.pgn --game 5

# Read from stdin
cat game.pgn | pgnq info

# Extract specific fields with jq
pgnq info game.pgn --json | jq '.statistics.nodes'
```

**Output:**
```
File: game.pgn

Headers:
  Event: World Championship
  Site: Dubai
  Date: 2021.12.03
  Round: 6
  White: Carlsen, Magnus
  Black: Nepomniachtchi, Ian
  Result: 1-0
  ECO: C42

Statistics:
  Nodes: 89
  Lines: 12
  Comments: 34
  Max depth: 45
  Main line length: 41
```

---

### `pgnq tree`

Visualize the game tree structure using Unicode box-drawing characters (or ASCII). Shows moves in a hierarchical tree format, making it easy to understand the branching structure of variations. Useful for exploring complex opening repertoires or annotated games.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `-d, --depth N` | Maximum depth to display (default: 10). Limits how deep into the tree to render. |
| `-p, --from-path PATH` | Start rendering from a specific node path instead of the root. |
| `--show-comments` | Display comments (truncated) alongside moves. |
| `--show-nags` | Display NAG annotations (!, ?, !!, etc.) with moves. |
| `--ascii` | Use ASCII characters only (no Unicode box drawing). Useful for terminals without Unicode support. |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Examples:**

```bash
# Display tree with default depth (10)
pgnq tree game.pgn

# Limit tree depth to 5 moves
pgnq tree game.pgn --depth 5

# Show tree starting from a specific position
pgnq tree game.pgn --from-path "e4/e5/Nf3"

# Display with comments and annotations
pgnq tree game.pgn --show-comments --show-nags

# ASCII output for compatibility
pgnq tree game.pgn --ascii

# Combine with head to see just the beginning
pgnq tree repertoire.pgn --depth 20 | head -50

# View the Sicilian variations only
pgnq tree openings.pgn --from-path "e4/c5" --depth 8
```

**Output:**
```
1. e4
├── 1... e5
│   ├── 2. Nf3
│   │   ├── 2... Nc6
│   │   │   ├── 3. Bb5 {Ruy Lopez}
│   │   │   └── 3. Bc4 {Italian Game}
│   │   └── 2... Nf6 {Petrov Defense}
│   └── 2. Bc4
│       └── 2... Bc5
└── 1... c5 {Sicilian Defense}
    └── 2. Nf3
        ├── 2... d6
        │   └── 3. d4
        └── 2... Nc6
            └── 3. Bb5
```

---

### `pgnq stats`

Display detailed statistics about a PGN file's tree structure. Provides comprehensive metrics including total nodes, leaf nodes (complete lines), commented positions, maximum depth, main line length, variation count, and NAG distribution. Useful for analyzing the complexity and annotation density of a study or repertoire.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format for programmatic use. |
| `--move-stats` | Include move frequency statistics (most common moves at each depth). |
| `--comment-stats` | Include comment analysis (average length, positions with comments). |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Examples:**

```bash
# Basic statistics
pgnq stats game.pgn

# JSON output for scripting
pgnq stats game.pgn --json

# Compare repertoire sizes
for f in *.pgn; do
  echo "$f: $(pgnq stats "$f" --json | jq '.total_nodes') nodes"
done

# Get specific stats with jq
pgnq stats study.pgn --json | jq '{
  lines: .leaf_nodes,
  depth: .max_depth,
  annotated: .commented_nodes
}'

# Find the most annotated repertoire
for f in repertoires/*.pgn; do
  echo "$(pgnq stats "$f" --json | jq '.commented_nodes') $f"
done | sort -rn | head -5
```

**Output (text):**
```
Statistics:
  Total nodes: 1247
  Leaf nodes (lines): 89
  Commented nodes: 342
  Max depth: 45
  Main line length: 28
  Variations: 156

NAG distribution:
  !: 45
  ?: 23
  !!: 12
  ?!: 8
  !?: 5
```

**Output (JSON):**
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
    "!!": 12,
    "?!": 8,
    "!?": 5
  }
}
```

---

### `pgnq convert`

Transform PGN files between different formats and apply optional modifications. Can convert between standard PGN, Lichess study format, and minimal notation. Supports stripping comments, variations, NAGs, headers, clock times, and evaluations. Useful for cleaning up files, preparing exports for different platforms, or creating simplified versions.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output FILE` | Write to file instead of stdout. |
| `-F, --format FORMAT` | Output format: `standard` (default), `lichess`, or `minimal`. |
| `--no-headers` | Omit PGN tag pairs (Event, White, etc.). |
| `--no-comments` | Strip all comments from the output. |
| `--no-variations` | Keep main line only, remove all variations. |
| `--no-nags` | Remove NAG annotations (!, ?, !!, etc.). |
| `--strip-clocks` | Remove `[%clk ...]` commands from comments. |
| `--strip-evals` | Remove `[%eval ...]` commands from comments. |
| `--line-width N` | Wrap output lines at N characters (default: 80, use 0 for no wrapping). |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Formats:**

| Format | Description |
|--------|-------------|
| `standard` | Traditional PGN with inline comments in braces. |
| `lichess` | Lichess study format with comments on separate lines. |
| `minimal` | Compact format with no formatting or line breaks. |

**Examples:**

```bash
# Convert Lichess export to standard PGN
pgnq convert lichess_study.pgn -F standard -o standard.pgn

# Convert to Lichess format for import
cat game.pgn | pgnq convert -F lichess > lichess_import.pgn

# Remove engine analysis but keep human comments
pgnq convert annotated.pgn --strip-clocks --strip-evals -o clean.pgn

# Create minimal PGN for sharing
pgnq convert game.pgn -F minimal --no-variations --no-comments

# Strip everything for a clean main line
pgnq convert game.pgn --no-comments --no-variations --no-nags --no-headers

# Extract a single game from a database
pgnq convert tournament.pgn --game 42 -o game42.pgn

# Process a file in a pipeline
cat raw.pgn | pgnq convert -F standard | pgnq filter --main-line > mainline.pgn

# Remove clock times but keep comments and evals
pgnq convert online_game.pgn --strip-clocks -o study.pgn

# Wide output for readability
pgnq convert game.pgn --line-width 120

# No line wrapping (single long line)
pgnq convert game.pgn --line-width 0
```

---

### `pgnq extract`

Extract a subtree starting from a specific node path. Creates a new PGN containing only the moves and variations from the specified position onward. Useful for pulling out specific openings, endgames, or critical positions from larger files.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `-p, --path PATH` | Node path to extract from (required). See Path Syntax above. |
| `-o, --output FILE` | Write to file instead of stdout. |
| `-F, --format FORMAT` | Output format: `standard`, `lichess`, or `minimal`. |
| `--with-prefix` | Include the moves from root to the extraction point in the output. |
| `--with-headers` | Copy headers from the original file to the extracted output. |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Path not found |

**Examples:**

```bash
# Extract the Sicilian Najdorf
pgnq extract repertoire.pgn -p "e4/c5/Nf3/d6/d4/cxd4/Nxd4/Nf6/Nc3/a6" -o najdorf.pgn

# Extract with the moves leading to the position
pgnq extract game.pgn -p "e4/e5/Nf3/Nc6/Bb5" --with-prefix

# Extract and preserve original headers
pgnq extract game.pgn -p "e4/c5" --with-headers -o sicilian.pgn

# Extract a specific variation (variation index 1 at c5)
pgnq extract game.pgn -p "e4/c5:1" -o dragon.pgn

# Extract and convert to Lichess format
pgnq extract study.pgn -p "d4/Nf6/c4/e6" -F lichess

# Chain with other commands
pgnq extract repertoire.pgn -p "e4/e5" | pgnq tree --depth 5

# Check if a path exists (use exit code)
if pgnq extract game.pgn -p "e4/c5/Nf3/d6" > /dev/null 2>&1; then
  echo "Najdorf found"
fi
```

---

### `pgnq split`

Split a PGN file into multiple smaller files at specified node paths. Each split specification maps a path to an output filename. Creates separate PGN files in the output directory, each containing the subtree from its specified path. Useful for breaking apart large repertoires into chapter files.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `-s, --split PATH:NAME` | Split specification in the format `path:filename` (without .pgn extension). Can be repeated multiple times. |
| `-o, --output-dir DIR` | Output directory for split files (default: `output/`). Created if it doesn't exist. |
| `-F, --format FORMAT` | Output format for split files: `standard`, `lichess`, or `minimal`. |
| `--include-prefix` | Include the path from root to each split point in the output files. |
| `--dry-run` | Show what would be created without actually writing files. |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Examples:**

```bash
# Split a repertoire into opening chapters
pgnq split repertoire.pgn \
  -s "e4/e5/Nf3/Nc6/Bb5:ruy_lopez" \
  -s "e4/e5/Nf3/Nc6/Bc4:italian" \
  -s "e4/c5/Nf3/d6:sicilian_najdorf" \
  -s "e4/c5/Nf3/Nc6:sicilian_classical" \
  -o chapters/

# Preview what would be created
pgnq split study.pgn \
  -s "e4:e4_lines" \
  -s "d4:d4_lines" \
  --dry-run

# Include the moves leading to each position
pgnq split game.pgn \
  -s "e4/e5/Nf3/Nc6/Bb5/a6/Ba4/Nf6/O-O:ruy_mainline" \
  --include-prefix \
  -o positions/

# Split into Lichess format
pgnq split study.pgn \
  -s "e4/c5:sicilian" \
  -F lichess \
  -o lichess_chapters/

# Split all first-level responses to e4
pgnq split openings.pgn \
  -s "e4/e5:open_games" \
  -s "e4/c5:sicilian" \
  -s "e4/e6:french" \
  -s "e4/c6:caro_kann" \
  -s "e4/d5:scandinavian" \
  -o by_opening/
```

**Output:**
```
Splitting repertoire.pgn...
  ruy_lopez.pgn         (245 nodes) OK
  italian.pgn           (189 nodes) OK
  sicilian_najdorf.pgn  (312 nodes) OK
  sicilian_classical.pgn (156 nodes) OK

Created 4 files in chapters/
```

---

### `pgnq filter`

Filter a PGN tree to include only nodes matching specific criteria. Can filter by path patterns, presence of comments or NAGs, tree depth, or restrict to main line only. Supports inverting filters to exclude matching nodes. Outputs a new PGN containing only the matching portions of the tree.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Input PGN file. Use `-` or omit to read from stdin. |

**Options:**

| Flag | Description |
|------|-------------|
| `-p, --path PATH` | Filter to nodes matching this path pattern. Supports glob patterns (`*`, `**`). |
| `--has-comment` | Include only nodes that have comments. |
| `--has-nag NAG` | Include only nodes with a specific NAG (e.g., `!`, `?`, `!!`, `$1`). |
| `--min-depth N` | Include only nodes at depth N or greater. |
| `--max-depth N` | Include only nodes at depth N or less. |
| `--main-line` | Output main line only (strip all variations). |
| `--invert` | Invert the filter (exclude matching nodes instead of including). |
| `-F, --format FORMAT` | Output format: `standard`, `lichess`, or `minimal`. |
| `--game N` | Select a specific game from a multi-game file (1-indexed). |

**Examples:**

```bash
# Extract just the main line
pgnq filter game.pgn --main-line

# Find all positions with comments
pgnq filter study.pgn --has-comment

# Find all brilliant moves
pgnq filter game.pgn --has-nag "!!"

# Find all mistakes and blunders
pgnq filter annotated.pgn --has-nag "?" | pgnq tree

# Limit to opening moves (first 15 moves)
pgnq filter game.pgn --max-depth 15

# Get the middlegame and endgame only
pgnq filter game.pgn --min-depth 20

# Get a specific depth range
pgnq filter study.pgn --min-depth 10 --max-depth 25

# Filter by path pattern (all Sicilian lines)
pgnq filter repertoire.pgn -p "e4/c5/**"

# Get immediate responses to e4
pgnq filter openings.pgn -p "e4/*"

# Find all UN-annotated positions
pgnq filter study.pgn --has-comment --invert

# Chain filters
pgnq filter study.pgn --has-comment | pgnq filter --has-nag "!" | pgnq tree

# Output filtered results in Lichess format
pgnq filter study.pgn --has-nag "!" -F lichess -o critical_moves.pgn

# Find deeply nested variations
pgnq filter repertoire.pgn --min-depth 20 | pgnq stats --json | jq '.leaf_nodes'
```

---

### `pgnq merge`

Merge multiple PGN files into a single unified tree structure. Combines games by matching identical move sequences and creating variations where they diverge. Useful for building opening repertoires from multiple sources, combining analyses, or consolidating game collections into a single study.

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE...` | One or more input PGN files. Use `-` for stdin (only valid as first file). Supports glob patterns. |

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output FILE` | Write to file instead of stdout. |
| `-F, --format FORMAT` | Output format: `standard`, `lichess`, or `minimal`. |
| `--no-comments` | Strip all comments from the merged output. |
| `--no-nags` | Remove NAG annotations from the merged output. |
| `--concat-comments` | When duplicate moves have different comments, concatenate them (default: keep first). |
| `--line-width N` | Wrap output lines at N characters (default: 80). |

**Merge Behavior:**

- **Level-by-level merging**: Uses a BFS algorithm to merge trees level by level.
- **Move matching**: Moves are matched by normalized SAN (move numbers stripped, whitespace normalized).
- **First-seen priority**: The first file's move order becomes the main line; subsequent files' diverging moves become variations.
- **Headers**: Taken from the first input file.
- **Result**: Always set to `*` (ongoing) since merged trees have multiple endpoints.
- **Comments**: First non-empty comment wins (or concatenate with `--concat-comments`).
- **NAGs**: First non-empty NAG list wins.

**Examples:**

```bash
# Merge multiple opening files
pgnq merge sicilian.pgn french.pgn caro_kann.pgn -o black_repertoire.pgn

# Merge all PGN files in a directory
pgnq merge repertoire/*.pgn -o complete.pgn

# Merge and convert to Lichess format
pgnq merge *.pgn -F lichess -o lichess_study.pgn

# Merge with concatenated comments (useful for combining annotations)
pgnq merge analysis1.pgn analysis2.pgn --concat-comments -o combined_analysis.pgn

# Merge and strip comments for a clean repertoire
pgnq merge *.pgn --no-comments --no-nags -o clean_repertoire.pgn

# Merge and view as tree
pgnq merge file1.pgn file2.pgn | pgnq tree --depth 10

# Merge from stdin and file
cat extra_lines.pgn | pgnq merge - main_repertoire.pgn -o combined.pgn

# Build repertoire from multiple sources and check stats
pgnq merge sources/*.pgn -o repertoire.pgn
pgnq stats repertoire.pgn --json | jq '{lines: .leaf_nodes, depth: .max_depth}'
```

**Example Merge:**

Given two input files:
```
# white_e4.pgn
1. e4 e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez}

# white_d4.pgn
1. d4 d5 2. c4 {Queen's Gambit}
```

Running `pgnq merge white_e4.pgn white_d4.pgn` produces:
```
1. e4 (1. d4 d5 2. c4 {Queen's Gambit}) 1... e5 2. Nf3 Nc6 3. Bb5 {Ruy Lopez}
```

As a tree:
```
├── 1. e4
│   └── 1... e5
│       └── 2. Nf3
│           └── 2... Nc6
│               └── 3. Bb5 {Ruy Lopez}
└── 1. d4
    └── 1... d5
        └── 2. c4 {Queen's Gambit}
```

## PGN Format Reference

`pgnq` uses a liberal parser designed to handle virtually any reasonable PGN. If it looks like chess notation, it will probably work—no strict format enforcement.

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

## More Examples

### Complex Pipeline Operations

```bash
# Extract all annotated lines from multiple studies
for f in studies/*.pgn; do
  pgnq filter "$f" --has-comment | pgnq convert -F minimal
done > annotated_lines.pgn

# Find the deepest variations across a repertoire
pgnq merge repertoire/*.pgn | pgnq stats --json | jq '.max_depth'

# Create a clean export stripping all engine analysis
pgnq convert study.pgn --strip-clocks --strip-evals --no-nags \
  | pgnq filter --has-comment \
  > human_annotations_only.pgn

# Split a large database and get stats on each part
pgnq split database.pgn -s "e4:e4_games" -s "d4:d4_games" -o parts/
for f in parts/*.pgn; do
  echo "=== $f ==="
  pgnq stats "$f" --json | jq '{nodes: .total_nodes, lines: .leaf_nodes}'
done

# Merge overlapping repertoires, preferring comments from the first
pgnq merge primary_repertoire.pgn secondary_repertoire.pgn -o combined.pgn

# Convert an entire directory to Lichess format
for f in *.pgn; do
  pgnq convert "$f" -F lichess -o "lichess_${f}"
done
```

## License

MIT
