# CLI Reference

Complete reference for all jpx command-line options.

## Synopsis

```
jpx [OPTIONS] [EXPRESSIONS]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[EXPRESSIONS]...` | JMESPath expression(s) to evaluate (multiple are chained as a pipeline) |

## Options

### Input/Output

| Option | Description |
|--------|-------------|
| `-e, --expression <EXPR>` | Expression(s) to evaluate (can be repeated) |
| `-Q, --query-file <FILE>` | Read JMESPath expression from file (supports `.jpx` libraries with colon syntax: `file.jpx:query-name`) |
| `--query <NAME>` | Select a named query from a `.jpx` library |
| `--list-queries` | List all queries in a `.jpx` library file |
| `--check` | Validate all queries in a `.jpx` library without running |
| `-f, --file <FILE>` | Input JSON file (reads stdin if not provided) |
| `-o, --output <FILE>` | Output file (writes to stdout if not provided) |
| `-n, --null-input` | Don't read input, use null as input value |
| `-s, --slurp` | Read all inputs into an array |
| `--stream` | Stream mode - process input line by line (for NDJSON/JSON Lines) |

### Output Format

| Option | Description |
|--------|-------------|
| `-r, --raw` | Output raw strings without quotes |
| `-c, --compact` | Compact output (no pretty printing) |
| `-y, --yaml` | Output as YAML |
| `--toml` | Output as TOML |
| `--csv` | Output as CSV (comma-separated values) |
| `--tsv` | Output as TSV (tab-separated values) |
| `-l, --lines` | Output one JSON value per line |
| `-t, --table` | Output as a formatted table (for arrays of objects) |
| `--table-style <STYLE>` | Table style: `unicode` (default), `ascii`, `markdown`, `plain` |
| `--color <MODE>` | Colorize output: `auto`, `always`, `never` |

See [Output Formats](./output-formats.md) for detailed examples.

### Modes

| Option | Description |
|--------|-------------|
| `--strict` | Strict mode - only standard JMESPath functions |
| `-v, --verbose` | Show expression details and timing |
| `-q, --quiet` | Suppress errors and warnings |

### JSON Patch Operations

| Option | Description |
|--------|-------------|
| `--diff <SOURCE> <TARGET>` | Generate JSON Patch (RFC 6902) from two files |
| `--patch <PATCH_FILE>` | Apply JSON Patch (RFC 6902) to input |
| `--merge <MERGE_FILE>` | Apply JSON Merge Patch (RFC 7396) to input |

### Data Analysis

| Option | Description |
|--------|-------------|
| `--stats` | Show statistics about the input data |
| `--paths` | List all paths in the input JSON |
| `--types` | Show types alongside paths (use with `--paths`) |
| `--values` | Show values alongside paths (use with `--paths`) |

### Benchmarking

| Option | Description |
|--------|-------------|
| `--bench [N]` | Benchmark expression performance (default: 100 iterations) |
| `--warmup <N>` | Number of warmup iterations before benchmarking (default: 5) |

### Function Discovery

| Option | Description |
|--------|-------------|
| `--list-functions` | List all available functions |
| `--list-category <NAME>` | List functions in a specific category |
| `--describe <FUNCTION>` | Show detailed info for a function |
| `--search <QUERY>` | Search functions by name, description, or category (fuzzy matching) |
| `--similar <FUNCTION>` | Find functions similar to the specified function |

### Debugging

| Option | Description |
|--------|-------------|
| `--explain` | Show how an expression is parsed (AST) |
| `--debug` | Show diagnostic information for troubleshooting |

### Interactive Mode

| Option | Description |
|--------|-------------|
| `--repl` | Start interactive REPL mode |
| `--demo <NAME>` | Load a demo dataset (use with `--repl`) |

### Other

| Option | Description |
|--------|-------------|
| `--completions <SHELL>` | Generate shell completions (`bash`, `zsh`, `fish`, `powershell`, `elvish`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `JPX_VERBOSE=1` | Enable verbose mode |
| `JPX_QUIET=1` | Enable quiet mode |
| `JPX_STRICT=1` | Enable strict mode |
| `JPX_RAW=1` | Output raw strings |
| `JPX_COMPACT=1` | Compact output |

Environment variables are overridden by command-line flags.

## Quick Examples

A few examples showing common flag combinations. For comprehensive examples, see:
- [Basic Usage](./basic-usage.md) - getting started with queries
- [Cookbook](./cookbook.md) - task-oriented recipes
- [Output Formats](./output-formats.md) - tables, CSV, YAML
- [Query Files](./query-files.md) - reusable query libraries

```bash
# Query a file, raw output for scripting
jpx -r 'config.api_key' -f settings.json

# Chain expressions, table output
jpx 'users' '[?active]' -t -f data.json

# Stream NDJSON, filter errors
cat logs.ndjson | jpx --stream '[?level == `"error"`]'

# Analyze structure before querying
jpx --paths --types -f data.json

# Find functions by keyword
jpx --search unique

# Run a named query from a library
jpx -Q queries.jpx:active-users -f data.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid expression, file not found, etc.) |
