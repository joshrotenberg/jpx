---
name: jpx-cli
description: "Run jpx to query and transform JSON from the command line. Covers output formats (CSV, TSV, table, YAML), streaming NDJSON, input modes (slurp, raw-input, null-input), query files, variable binding, and shell pipelines. Use when building shell pipelines or exporting JSON data."
license: MIT OR Apache-2.0
metadata:
  author: joshrotenberg
  version: "1.0"
allowed-tools: Bash(jpx:*)
---

# jpx CLI Usage Guide

jpx is a JMESPath CLI with 400+ extended functions. It reads JSON, applies expressions, and outputs results in multiple formats.

## Basic Usage

```bash
# Expression as positional argument
echo '{"name":"alice"}' | jpx 'name'
# -> "alice"

# File input
jpx 'users[*].name' data.json

# Explicit file flag
jpx -f data.json 'users[*].name'

# Null input (no stdin needed)
jpx -n 'now()'

# Multiple expressions (chained pipeline)
jpx '[*].name' 'sort(@)' '[0]' data.json

# Same with -e flags
jpx -e '[*].name' -e 'sort(@)' -e '[0]' -f data.json
```

## Output Formats

### JSON (default)
```bash
jpx 'expression' data.json              # pretty-printed, colored
jpx -c 'expression' data.json           # compact (one line)
jpx --indent 4 'expression' data.json   # 4-space indent
jpx --tab 'expression' data.json        # tab indent
jpx -S 'expression' data.json           # sort object keys
jpx --color never 'expression' data.json  # no color
```

### Raw strings
```bash
jpx -r 'name' data.json                 # "alice" -> alice (no quotes)
jpx -j 'name' data.json                 # raw, no trailing newline
```

### CSV / TSV
```bash
jpx --csv 'users[*]' data.json          # CSV with headers from object keys
jpx --tsv 'users[*]' data.json          # tab-separated
# Nested objects are flattened: {"addr":{"city":"NYC"}} -> addr.city column
```

### Table
```bash
jpx -t 'users[*]' data.json             # formatted table
jpx -t --table-style ascii 'users[*]' data.json
# Styles: unicode (default), ascii, markdown, plain, rounded, sharp, modern
```

### YAML / TOML
```bash
jpx -y 'config' data.json               # YAML output
jpx --toml 'config' data.json           # TOML output
```

### Lines (NDJSON)
```bash
jpx -l 'users[*]' data.json             # one JSON value per line
```

### Output to file
```bash
jpx -o result.json 'expression' data.json
jpx --csv -o data.csv 'users[*]' data.json
```

## Input Modes

### Slurp (`-s`)
Combine multiple JSON values from stdin into a single array:
```bash
echo -e '{"id":1}\n{"id":2}\n{"id":3}' | jpx -s 'length(@)'
# -> 3
```

### Raw input (`-R`)
Read each line as a JSON string (not parsed as JSON):
```bash
echo -e "hello\nworld" | jpx -R -s '[*] | sort(@)'
# -> ["hello", "world"]
```

### Null input (`-n`)
Use `null` as input, don't read stdin:
```bash
jpx -n 'now()'                    # current timestamp
jpx -n 'range(`1`, `11`)'         # [1,2,...,10]
jpx -n 'uuid()'                   # random UUID
```

## Streaming Mode

Process NDJSON line by line with constant memory:
```bash
# Each line processed independently
cat events.ndjson | jpx --stream 'user.name'

# Null results are automatically skipped
cat events.ndjson | jpx --stream '[?level == `error`] | [0]'

# Streaming with CSV output (headers from first record)
cat events.ndjson | jpx --stream --csv '{user: user.name, action: event}'

# Streaming with TSV
cat events.ndjson | jpx --stream --tsv '@'

# Unbuffered output (flush after each record)
tail -f events.ndjson | jpx --stream --unbuffered 'message'
```

**Note:** `--stream` is incompatible with `--table`, `--yaml`, `--toml`, and `--lines` (these need the full dataset). Use `--slurp` instead if you need those formats.

See [references/streaming.md](references/streaming.md) for more streaming patterns.

## Variable Binding

Bind variables for use in expressions:
```bash
# String variable
jpx --arg name Alice '[?name == $name]' data.json

# JSON variable (parsed)
jpx --argjson threshold 42 '[?score > $threshold]' data.json
jpx --argjson ids '[1,2,3]' '[?contains($ids, id)]' data.json
```

## Query Files

Save and reuse expressions with `.jpx` query files:
```bash
# Run a named query
jpx -Q queries.jpx:active-users data.json

# Alternative syntax
jpx -Q queries.jpx --query active-users data.json

# List available queries
jpx -Q queries.jpx --list-queries

# Validate queries without running
jpx -Q queries.jpx --check
```

See [references/query-files.md](references/query-files.md) for the `.jpx` file format.

## Function Discovery

```bash
jpx --list-functions                     # all 400+ functions
jpx --list-category String              # functions in a category
jpx --describe format_date              # detailed function info
jpx --search "date format"             # fuzzy search
jpx --similar group_by                  # find related functions
```

## Data Inspection

```bash
jpx --stats -f data.json                # type, size, depth analysis
jpx --paths -f data.json                # all paths in dot notation
jpx --paths --types -f data.json        # paths with types
jpx --paths --values -f data.json       # paths with values
```

## JSON Operations

```bash
# Diff two files (RFC 6902 JSON Patch)
jpx --diff before.json after.json

# Apply a patch
jpx --patch changes.json -f original.json

# Merge (RFC 7396)
jpx --merge overlay.json -f base.json
```

## Other Features

```bash
# Explain expression (AST breakdown)
jpx --explain 'sort_by([*], &name) | [0]'

# Benchmark
jpx --bench 1000 'complex.expression' data.json

# Interactive REPL
jpx --repl -f data.json
jpx --repl --demo users                 # with demo dataset

# Exit status (for shell conditionals)
jpx -x '[?critical]' data.json && echo "found critical items"
```

## Shell Pipeline Recipes

```bash
# Fetch API, extract, and format as table
curl -s https://api.example.com/users | jpx -t '[*].{name: name, email: email}'

# Process log file, count errors by type
cat app.log | jpx --stream -s '[*]' | jpx 'group_by(@, &error_type)'

# Convert JSON to CSV for spreadsheet
jpx --csv 'records[*]' -o export.csv data.json

# Find largest files in package.json dependencies
jpx 'dependencies | keys(@) | sort(@)' package.json
```

## Related Skills

- [jmespath-query](../jmespath-query/SKILL.md) -- JMESPath expression syntax
- [jpx-functions](../jpx-functions/SKILL.md) -- 400+ extension functions
- [jpx-mcp](../jpx-mcp/SKILL.md) -- MCP server for AI agent integration
