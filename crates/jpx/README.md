# jpx - JMESPath CLI with Extended Functions

[![Crates.io](https://img.shields.io/crates/v/jpx.svg)](https://crates.io/crates/jpx)
[![Downloads](https://img.shields.io/crates/d/jpx.svg)](https://crates.io/crates/jpx)
[![License](https://img.shields.io/crates/l/jpx.svg)](https://github.com/joshrotenberg/jpx#license)

A command-line tool for querying JSON data using JMESPath expressions with 470+ additional functions beyond the standard JMESPath specification.

## Acknowledgments

jpx builds on the excellent work of the JMESPath community:

- **[JMESPath](https://jmespath.org/)** - The query language specification created by [James Saryerwinnie](https://github.com/jamesls)
- **[jmespath.rs](https://crates.io/crates/jmespath)** - The Rust implementation by [@mtdowling](https://github.com/mtdowling) that provides our parsing, evaluation, and standard functions
- **[jp](https://github.com/jmespath/jp)** - The official JMESPath CLI (Go) - a minimal, focused tool that inspired jpx's design

If you only need standard JMESPath without extensions, consider using [jp](https://github.com/jmespath/jp) or the [`jmespath`](https://crates.io/crates/jmespath) crate directly.

## jpx vs jq

Coming from [jq](https://jqlang.org/)? Here's a quick comparison:

| | jq | jpx |
|---|-----|-----|
| **Language** | Custom DSL | JMESPath (standardized) |
| **Functions** | ~70 built-in | 470+ extensions |
| **Ecosystem** | Standalone | Works with AWS CLI, Ansible |
| **Streaming** | ✅ | ❌ |

```bash
# jq                                    # jpx
jq '[.[] | select(.age > 30)]'          jpx '[?age > `30`]'
jq '.[].name'                           jpx '[*].name'
jq '.name | ascii_upcase'               jpx 'upper(name)'
```

**Choose jpx** for: extended functions (geo, hash, fuzzy, semver), multiple output formats, AI/MCP integration, JMESPath ecosystem compatibility.

**Choose jq** for: streaming large files, custom function definitions, complex recursive transformations.

**[Full comparison →](https://joshrotenberg.github.io/jpx/cli/why-jpx/)**

## Quick Start (Docker)

Try jpx instantly without installing anything:

```bash
# Fetch a Hacker News story and extract fields
curl -s 'https://hacker-news.firebaseio.com/v0/item/1.json' | \
  docker run -i ghcr.io/joshrotenberg/jpx '{title: title, by: by, score: score}'
# {"by": "pg", "score": 57, "title": "Y Combinator"}

# Basic query
echo '{"name": "Alice", "scores": [85, 92, 78]}' | docker run -i ghcr.io/joshrotenberg/jpx 'name'
# "Alice"

# Calculate average
echo '{"scores": [85, 92, 78]}' | docker run -i ghcr.io/joshrotenberg/jpx 'avg(scores)'
# 85.0

# String manipulation
echo '{"name": "hello world"}' | docker run -i ghcr.io/joshrotenberg/jpx 'upper(name)'
# "HELLO WORLD"

# Filter and transform
echo '[{"name":"alice","age":30},{"name":"bob","age":25}]' | docker run -i ghcr.io/joshrotenberg/jpx '[?age > `28`].name'
# ["alice"]
```

## Installation

```bash
# Docker (no install needed)
docker pull ghcr.io/joshrotenberg/jpx

# Homebrew (macOS/Linux)
brew tap joshrotenberg/brew
brew install jpx

# Pre-built binaries (macOS, Linux, Windows)
# Download from https://github.com/joshrotenberg/jpx/releases

# From crates.io
cargo install jpx

# From source
git clone https://github.com/joshrotenberg/jpx
cd jpx/crates/jpx
cargo install --path .

# Without MCP server support (smaller binary)
cargo install --path . --no-default-features
```

## MCP Server (AI Assistant Integration)

For MCP (Model Context Protocol) support, use the dedicated [`jpx-mcp`](https://crates.io/crates/jpx-mcp) package:

```bash
# Install
cargo install jpx-mcp

# Or use Docker
docker run -i --rm ghcr.io/joshrotenberg/jpx-mcp
```

See the [MCP documentation](https://joshrotenberg.github.io/jpx/mcp/overview/) for setup instructions with Claude Desktop.

## Usage

```bash
jpx [OPTIONS] [EXPRESSIONS]...

Arguments:
  [EXPRESSIONS]...  JMESPath expression(s) to evaluate (multiple are chained as a pipeline)

Options:
  -e, --expression <EXPR>     Expression(s) to evaluate (can be chained)
  -Q, --query-file <FILE>     Read JMESPath expression from file
  -f, --file <FILE>           Input file (reads from stdin if not provided)
  -o, --output <FILE>         Output file (writes to stdout if not provided)
  -n, --null-input            Don't read input, use null as input value
  -s, --slurp                 Read all inputs into an array
      --stream, --each        Process input line by line (NDJSON/JSON Lines)

Output Formats:
  -r, --raw                   Output raw strings without quotes
  -c, --compact               Compact output (no pretty printing)
  -y, --yaml                  Output as YAML
      --toml                  Output as TOML
      --csv                   Output as CSV (for arrays of objects)
      --tsv                   Output as TSV (for arrays of objects)
  -l, --lines                 Output one JSON value per line (for arrays)
  -t, --table                 Output as a formatted table (for arrays of objects)
      --table-style <STYLE>   Table style: unicode, ascii, markdown, plain, rounded, sharp, modern
      --color <MODE>          Colorize output (auto, always, never)

JSON Patch Operations:
      --diff <SRC> <TGT>      Generate JSON Patch (RFC 6902) from two files
      --patch <FILE>          Apply JSON Patch (RFC 6902) to input
      --merge <FILE>          Apply JSON Merge Patch (RFC 7396) to input

Data Analysis:
      --stats                 Show statistics about the input data
      --paths                 List all paths in the input JSON
      --types                 Show types alongside paths (use with --paths)
      --values                Show values alongside paths (use with --paths)

Function Discovery:
      --list-functions        List all available extension functions
      --list-category <NAME>  List functions in a specific category
      --describe <FUNCTION>   Show detailed info for a specific function
      --search <QUERY>        Search functions by name, description, or category
      --similar <FUNCTION>    Find functions similar to the specified function

Debugging:
      --explain               Show how an expression is parsed (AST)
      --debug                 Show diagnostic information
      --bench [N]             Benchmark expression performance
      --warmup <N>            Warmup iterations before benchmarking

Modes:
  -q, --quiet                 Suppress errors and warnings
  -v, --verbose               Show expression details and timing
      --strict                Strict mode - only standard JMESPath (no extensions)
      --repl                  Start interactive REPL mode
      --demo <NAME>           Load a demo dataset (use with --repl)

Other:
      --completions <SHELL>   Generate shell completions (bash, zsh, fish, powershell, elvish)
  -h, --help                  Print help
  -V, --version               Print version
```

## Environment Variables

Configure jpx defaults via environment variables (CLI flags take precedence):

| Variable | Description |
|----------|-------------|
| `JPX_VERBOSE=1` | Enable verbose mode |
| `JPX_QUIET=1` | Enable quiet mode |
| `JPX_STRICT=1` | Enable strict mode (standard JMESPath only) |
| `JPX_RAW=1` | Output raw strings without quotes |
| `JPX_COMPACT=1` | Compact output (no pretty printing) |

```bash
# Set defaults in your shell profile
export JPX_RAW=1        # Always output raw strings

# Temporarily use strict mode
JPX_STRICT=1 jpx 'length(@)' data.json

# Unset to use extensions again
unset JPX_STRICT
jpx 'upper(name)' data.json  # Extension functions work
```

## Function Discovery

```bash
# List all available functions grouped by category
# Shows 26 standard JMESPath functions and 470+ extension functions
jpx --list-functions

# List functions in a specific category
jpx --list-category string
jpx --list-category math
jpx --list-category geo
jpx --list-category standard  # List all 26 standard JMESPath functions

# Get detailed info about a specific function
# Shows type (standard JMESPath or extension), category, signature, and example
jpx --describe upper
jpx --describe haversine_km
jpx --describe abs  # Standard JMESPath function
```

## Examples

### Basic Queries

```bash
# Simple field access
echo '{"name": "Alice", "age": 30}' | jpx 'name'
# "Alice"

# Array operations
echo '[1, 2, 3, 4, 5]' | jpx 'sum(@)'
# 15.0

# Nested access
echo '{"user": {"profile": {"email": "alice@example.com"}}}' | jpx 'user.profile.email'
# "alice@example.com"
```

### Streaming (NDJSON/JSON Lines)

Process newline-delimited JSON one line at a time with constant memory usage. Perfect for large log files, event streams, and data pipelines.

```bash
# Basic streaming - each line is processed independently
cat events.jsonl | jpx --stream 'user.name'

# With raw output for piping to other tools
cat logs.jsonl | jpx --stream -r 'message' | grep "error"

# From a file
jpx --stream 'id' -f huge_dataset.jsonl

# Using the --each alias
cat data.jsonl | jpx --each 'timestamp'

# With expression functions
cat users.jsonl | jpx --stream -r 'upper(name)'
```

**Performance**: ~1.25 million lines/second (processes 100k lines in ~80ms)

**How it works**:
- Each line is parsed as a complete JSON object
- Expression is compiled once, reused for all lines
- Results output immediately (no accumulation)
- Empty lines and null results are skipped
- Errors on individual lines don't stop processing (use `-q` to suppress)

**Comparison with `--slurp`**:
- `--slurp` loads all lines into memory as an array, then queries
- `--stream` processes each line independently with constant memory
- Use `--slurp` when you need to aggregate across lines (e.g., `sum([*].value)`)
- Use `--stream` for large files or when lines are independent

### String Functions

```bash
# Case conversion
echo '{"name": "hello world"}' | jpx 'upper(name)'
# "HELLO WORLD"

echo '{"name": "Hello World"}' | jpx 'snake_case(name)'
# "hello_world"

# String manipulation
echo '{"text": "  trim me  "}' | jpx -r 'trim(text)'
# trim me

echo '{"words": ["hello", "world"]}' | jpx -r 'join(`", "`, words)'
# hello, world
```

### Array Functions

```bash
# Get unique values
echo '{"nums": [1, 2, 2, 3, 3, 3]}' | jpx 'unique(nums)'
# [1, 2, 3]

# Chunk arrays
echo '{"items": [1, 2, 3, 4, 5, 6]}' | jpx 'chunk(items, `2`)'
# [[1, 2], [3, 4], [5, 6]]

# Array statistics
echo '[10, 20, 30, 40, 50]' | jpx 'median(@)'
# 30.0

echo '[10, 20, 30, 40, 50]' | jpx 'stddev(@)'
# 14.142135623730951
```

### Date/Time Functions

```bash
# Current Unix timestamp
echo '{}' | jpx 'now()'
# 1705312200.0

# Format a Unix timestamp
echo '{"ts": 1705276800}' | jpx -r "format_date(ts, '%Y-%m-%d')"
# 2024-01-15

# Date arithmetic (add 7 days to timestamp)
echo '{"ts": 1705276800}' | jpx 'date_add(ts, `7`, `"days"`)'
# 1705881600.0
```

### Duration Functions

```bash
# Parse human-readable durations
echo '{"duration": "1h30m"}' | jpx 'parse_duration(duration)'
# 5400.0

echo '{"duration": "2d12h"}' | jpx 'duration_hours(parse_duration(duration))'
# 60.0

# Format seconds as duration
echo '{"seconds": 3725}' | jpx -r 'format_duration(seconds)'
# 1h2m5s
```

### Color Functions

```bash
# Convert colors
echo '{"color": "#ff5500"}' | jpx 'hex_to_rgb(color)'
# {"r": 255, "g": 85, "b": 0}

echo '{"r": 255, "g": 128, "b": 0}' | jpx -r 'rgb_to_hex(r, g, b)'
# #ff8000

# Color manipulation
echo '{"color": "#3366cc"}' | jpx -r 'lighten(color, `20`)'
# #5c85d6

echo '{"color": "#ff0000"}' | jpx -r 'color_complement(color)'
# #00ffff
```

### Computing Functions

```bash
# Parse byte sizes
echo '{"size": "1.5 GB"}' | jpx 'parse_bytes(size)'
# 1500000000.0

# Format bytes
echo '{"bytes": 1073741824}' | jpx -r 'format_bytes_binary(bytes)'
# 1.00 GiB

# Bitwise operations
echo '{"a": 12, "b": 10}' | jpx 'bit_and(a, b)'
# 8

echo '{"n": 1}' | jpx 'bit_shift_left(n, `4`)'
# 16
```

### Hash and Encoding Functions

```bash
# Hash functions
echo '{"text": "hello"}' | jpx -r 'md5(text)'
# 5d41402abc4b2a76b9719d911017c592

echo '{"text": "hello"}' | jpx -r 'sha256(text)'
# 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

# Base64 encoding
echo '{"text": "hello world"}' | jpx -r 'base64_encode(text)'
# aGVsbG8gd29ybGQ=

# URL encoding
echo '{"query": "hello world & more"}' | jpx -r 'url_encode(query)'
# hello%20world%20%26%20more
```

### Geo Functions

```bash
# Calculate distance between coordinates (km)
echo '{"lat1": 40.7128, "lon1": -74.0060, "lat2": 34.0522, "lon2": -118.2437}' | jpx 'geo_distance_km(lat1, lon1, lat2, lon2)'
# 3935.746...

# Calculate bearing
echo '{"lat1": 40.7128, "lon1": -74.0060, "lat2": 34.0522, "lon2": -118.2437}' | jpx 'geo_bearing(lat1, lon1, lat2, lon2)'
# 273.687...
```

### Network Functions

```bash
# IP address operations
echo '{"ip": "192.168.1.1"}' | jpx 'ip_to_int(ip)'
# 3232235777

echo '{"cidr": "10.0.0.0/8", "ip": "10.1.2.3"}' | jpx 'cidr_contains(cidr, ip)'
# true

echo '{"ip": "192.168.1.1"}' | jpx 'is_private_ip(ip)'
# true
```

### Semver Functions

```bash
# Parse semantic versions
echo '{"version": "1.2.3-beta.1"}' | jpx 'semver_parse(version)'
# {"major": 1, "minor": 2, "patch": 3, "prerelease": "beta.1", "build": null}

# Compare versions
echo '{"v1": "1.2.3", "v2": "1.3.0"}' | jpx 'semver_compare(v1, v2)'
# -1

# Check version constraints
echo '{"version": "1.5.0", "constraint": "^1.0.0"}' | jpx 'semver_satisfies(version, constraint)'
# true
```

### Text Analysis Functions

```bash
# Word and character counts
echo '{"text": "Hello world, this is a test."}' | jpx 'word_count(text)'
# 6

echo '{"text": "Hello world!"}' | jpx 'char_count(text)'
# 12

# Reading time estimation
echo '{"article": "This is a longer article with many words..."}' | jpx 'reading_time(article)'
# "1 min read"

# Word frequencies
echo '{"text": "the quick brown fox jumps over the lazy dog the"}' | jpx 'word_frequencies(text)'
# {"the": 3, "quick": 1, "brown": 1, ...}
```

### Phonetic Functions

```bash
# Soundex encoding
echo '{"name": "Robert"}' | jpx -r 'soundex(name)'
# R163

# Check if names sound alike
echo '{"name1": "Robert", "name2": "Rupert"}' | jpx 'sounds_like(name1, name2)'
# true

# Double Metaphone
echo '{"name": "Smith"}' | jpx 'double_metaphone(name)'
# {"primary": "SM0", "secondary": "XMT"}
```

### Fuzzy Matching Functions

```bash
# Levenshtein distance
echo '{"s1": "kitten", "s2": "sitting"}' | jpx 'levenshtein(s1, s2)'
# 3

# Jaro-Winkler similarity
echo '{"s1": "MARTHA", "s2": "MARHTA"}' | jpx 'jaro_winkler(s1, s2)'
# 0.961...

# Sorensen-Dice coefficient
echo '{"s1": "night", "s2": "nacht"}' | jpx 'sorensen_dice(s1, s2)'
# 0.25
```

### ID Generation Functions

```bash
# Generate UUIDs
echo '{}' | jpx -r 'uuid()'
# 550e8400-e29b-41d4-a716-446655440000

# Generate nanoids
echo '{}' | jpx -r 'nanoid()'
# V1StGXR8_Z5jdHi6B-myT

# Generate ULIDs
echo '{}' | jpx -r 'ulid()'
# 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Validation Functions

```bash
# Email validation
echo '{"email": "user@example.com"}' | jpx 'is_email(email)'
# true

# URL validation
echo '{"url": "https://example.com/path"}' | jpx 'is_url(url)'
# true

# UUID validation
echo '{"id": "550e8400-e29b-41d4-a716-446655440000"}' | jpx 'is_uuid(id)'
# true

# IP address validation
echo '{"ip": "192.168.1.1"}' | jpx 'is_ipv4(ip)'
# true
```

### Expression Functions (Higher-Order)

```bash
# Filter with expression reference
echo '[{"age": 25}, {"age": 17}, {"age": 30}]' | jpx 'filter_expr(&age >= `18`, @)'
# [{"age": 25}, {"age": 30}]

# Map with expression reference
echo '[{"name": "Alice"}, {"name": "Bob"}]' | jpx 'map_expr(&name, @)'
# ["Alice", "Bob"]

# Group by expression reference
echo '[{"type": "a", "v": 1}, {"type": "b", "v": 2}, {"type": "a", "v": 3}]' | jpx 'group_by_expr(&type, @)'
# {"a": [{"type": "a", "v": 1}, {"type": "a", "v": 3}], "b": [{"type": "b", "v": 2}]}
```

## Using Test Data Files

The `testdata/` directory contains sample JSON files for experimenting:

```bash
# Users data
jpx -f testdata/users.json '[].name'
jpx -f testdata/users.json 'filter_expr(@, &age > `25`) | [].name'

# Server logs
jpx -f testdata/servers.json 'filter_expr(@, &status == `active`) | length(@)'
jpx -f testdata/servers.json '[].{name: name, uptime: format_duration(uptime_seconds)}'

# E-commerce orders
jpx -f testdata/orders.json 'sum([].total)'
jpx -f testdata/orders.json 'group_by_expr(@, &status)'

# Geo locations
jpx -f testdata/locations.json 'haversine_km([0].lat, [0].lon, [1].lat, [1].lon)'

# Versions
jpx -f testdata/packages.json 'sort_by_expr(@, &semver_parse(version).major)'
```

## Using Query Files

For complex queries, you can store the JMESPath expression in a file and use `-Q` / `--query-file`:

```bash
# Create a query file
cat > transform.jmespath << 'EOF'
{
  users: @[?active].{
    name: name,
    email: contact.email,
    joined: format_date(created_at, '%Y-%m-%d')
  },
  total: length(@[?active]),
  generated: now()
}
EOF

# Run the query
jpx -Q transform.jmespath -f users.json
```

Benefits of query files:
- Easier to write and edit complex expressions
- Can be version controlled
- Reusable across different data files
- No shell escaping issues

See the `queries/` directory for example query files.

## Tips

- Use `-r` (raw) when piping string output to other commands
- Use `-c` (compact) for single-line JSON output
- Use `--list-functions` to see all available functions
- Backticks create literal values: `` `5` `` is number 5, `` `"hello"` `` is string
- Use `&` prefix for expression references in higher-order functions

## License

MIT OR Apache-2.0
