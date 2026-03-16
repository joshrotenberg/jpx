# AGENTS.md

Instructions for AI coding agents working on the jpx project.

## Agent Skills

This project includes [Agent Skills](https://agentskills.io/) in the `skills/` directory for in-depth guidance on specific topics:

| Skill | When to use |
|-------|-------------|
| [`jmespath-query`](skills/jmespath-query/SKILL.md) | Writing JMESPath expressions (syntax, patterns, built-in functions) |
| [`jpx-functions`](skills/jpx-functions/SKILL.md) | Using the 400+ extension functions (signatures, categories, examples) |
| [`jpx-cli`](skills/jpx-cli/SKILL.md) | Using the jpx CLI (output formats, streaming, pipelines, REPL) |
| [`jpx-mcp`](skills/jpx-mcp/SKILL.md) | Using the jpx MCP server (32 tools, discovery, query store) |

Each skill has a SKILL.md overview and a `references/` directory with detailed docs loaded on demand.

## Project overview

jpx is a JMESPath CLI and toolchain with 400+ extension functions — a `jq` alternative.
Written in Rust (edition 2024, rust-version 1.90), dual-licensed MIT/Apache-2.0.

### Workspace crates

| Crate | Path | Description |
|---|---|---|
| `jpx` | `crates/jpx/` | CLI with REPL, streaming, multiple output formats (JSON, YAML, CSV, TSV, table) |
| `jpx-core` | `crates/jpx-core/` | From-scratch JMESPath parser and interpreter, 425 functions across 32 categories |
| `jpx-engine` | `crates/jpx-engine/` | Query engine, introspection, BM25 discovery index, config |
| `jpx-mcp` | `crates/jpx-mcp/` | MCP server (32 tools) built on `tower-mcp` |
| `python` | `python/` | Python bindings via PyO3 (`jmespath-extensions` on PyPI) |

## Build and test

```bash
# Full test suite (2500+ tests)
cargo test --workspace

# With all features
cargo test --workspace --all-features

# Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Build CLI with optional features
cargo build -p jpx --features let-expr
cargo build -p jpx --features parquet
```

### Feature flags

- `let-expr` — JEP-18 let expressions (`let $var = expr in body`)
- `extensions` — all 400+ extension functions (default on for jpx-core)
- `parquet` — Parquet file input support (CLI only)
- `arrow` — Arrow array support (jpx-engine)
- `schema` — JSON Schema generation via schemars (jpx-engine, for MCP)

### Validate query files

```bash
jpx -Q examples/earthquakes.jpx --check
```

## JMESPath syntax guide

This section covers the most common syntax mistakes AI agents make when writing JMESPath expressions for jpx.

### Literal values use backtick-quoted JSON

Literals are backtick-delimited and must contain valid JSON inside:

```
`42`          — number
`"hello"`     — string (NOT `hello` — that's invalid JSON)
`true`        — boolean
`null`        — null
`[1, 2, 3]`  — array
```

The most common mistake is writing `` `hello` `` instead of `` `"hello"` `` for string literals.

### Single-quoted strings are raw strings

Single quotes produce raw strings, mainly used as function arguments:

```
upper('hello')                     — raw string arg
split(place, ', ')                 — delimiter arg
format_date(ts, '%Y-%m-%d')       — format string arg
```

### Filter comparisons use backtick literals

```
items[?price > `10`]               — number comparison
items[?status == `"active"`]       — string comparison (backtick + JSON string)
items[?enabled == `true`]          — boolean comparison
```

### Expref (&) arguments

Functions that key on a field use expression references, matching the JMESPath spec:

```
sort_by(arr, &name)       — expref (standard JMESPath)
group_by(arr, &name)      — expref (preferred)
group_by(arr, 'name')     — string key (legacy, still works)
```

Check function signatures with `jpx --describe <function>` or the MCP `describe` tool.

### Let expressions (JEP-18)

```
let $x = expr1, $y = expr2 in body
```

When projecting over a let-bound variable, use flatten `[]` not wildcard `[*]`:

```
let $items = data[*].name in sort($items[])     — correct
let $items = data[*].name in sort($items[*])    — fails
```

### Epoch timestamp conversion

USGS and many APIs use millisecond epoch timestamps. jpx datetime functions expect seconds:

```
format_date(divide(properties.time, `1000`), '%b %d %H:%M')
from_epoch(divide(timestamp, `1000`))
```

`format_date(epoch_seconds, format_string)` formats directly — no need to call `from_epoch` first.

### group_by returns an object

`group_by(array, 'key')` returns `{key1: [...], key2: [...]}`. Use `items()` to iterate:

```
group_by(arr, 'region') | items(@) | [*].{region: [0], count: length([1])}
```

## Function discovery

jpx has 400+ functions across 32 categories. Never guess function names — discover them:

```bash
# Search by keyword (BM25 full-text search)
jpx --search "distance"
jpx --search "hash"

# Detailed docs for a specific function
jpx --describe geo_distance_km
jpx --describe format_date

# List all functions in a category
jpx --list-functions --category datetime

# List all categories
jpx --list-categories
```

### MCP equivalents

When using the jpx MCP server, the same discovery is available via tools:

- `search` — find functions by keyword
- `describe` — get function signature, description, and example
- `functions` — list functions, optionally filtered by category
- `categories` — list all 32 categories
- `similar` — find functions related to a given function

Always use `describe` to check a function's exact signature before using it.

## MCP server tools

The MCP server exposes 32 tools. Key patterns:

- **`evaluate`** / **`evaluate_file`** — run a JMESPath expression against JSON input or a file
- **`batch_evaluate`** — run multiple expressions against the same input
- **`validate`** — check expression syntax without executing
- **`explain`** — break down an expression into steps
- **`stats`** / **`paths`** / **`keys`** — analyze JSON structure before querying
- **`define_query`** / **`run_query`** / **`list_queries`** — session-scoped named query store
- **`diff`** / **`patch`** / **`merge`** — JSON Patch (RFC 6902) and Merge Patch (RFC 7396)

### Workflow for exploring unfamiliar JSON

1. Use `stats` to understand the shape (type, size, depth, field frequency)
2. Use `paths` or `keys` to see available fields
3. Use `evaluate` to run exploratory queries
4. Use `define_query` to save useful expressions for reuse

## Query files (.jpx)

Named query libraries are plain text files with this format:

```
-- :name my-query
-- :desc Description of what this query does
expression_here

-- :name another-query
-- :desc Another description
another_expression
```

Run with: `jpx -Q file.jpx:query-name -f data.json`

Validate all queries: `jpx -Q file.jpx --check`

List queries: `jpx -Q file.jpx --list-queries`

## Code conventions

- Commit messages: concise, imperative, focused on "why" not "what"
- PRs: short title (under 70 chars), bulleted summary + test plan in body
- Tests: add tests for new functions and bug fixes; run full suite before PRing
- Docs: MkDocs site in `docs/src/`; function reference is auto-generated from code metadata
- CI skips Rust/Python jobs for docs-only changes (paths filter on `.md`, `docs/**`)

## CLI output formats

The CLI supports multiple output formats — useful for demos and data export:

```bash
jpx 'expr' -f data.json          # Pretty-printed JSON (default)
jpx 'expr' -f data.json -c       # Compact JSON
jpx 'expr' -f data.json -t       # Table (arrays of objects)
jpx 'expr' -f data.json --csv    # CSV
jpx 'expr' -f data.json --tsv    # TSV
jpx 'expr' -f data.json -y       # YAML
jpx 'expr' -f data.json -r       # Raw string (no quotes)
jpx 'expr' -f data.json -l       # One JSON value per line (NDJSON)
```
