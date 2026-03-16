# jpx Output Formats

## JSON (default)

Pretty-printed with syntax coloring when output is a terminal.

```bash
jpx 'expression' data.json              # pretty, colored (if terminal)
jpx -c 'expression' data.json           # compact (single line)
jpx --indent 4 'expression' data.json   # custom indent (0-8 spaces)
jpx --tab 'expression' data.json        # tab indentation
jpx -S 'expression' data.json           # sort object keys alphabetically
jpx --color always 'expression' data.json  # force color even in pipes
jpx --color never 'expression' data.json   # disable color
```

## Raw Strings (`-r`, `-j`)

Output string values without JSON quotes. Non-string values are still JSON-encoded.

```bash
echo '{"msg":"hello"}' | jpx -r 'msg'    # hello (no quotes)
echo '{"n":42}' | jpx -r 'n'             # 42 (numbers unchanged)

# Join output: raw without trailing newline
echo '{"id":"abc"}' | jpx -j 'id'        # abc (no newline)
```

## CSV (`--csv`)

Best for arrays of objects. Keys become column headers. Nested objects are flattened with dot notation.

```bash
echo '[{"name":"alice","age":30},{"name":"bob","age":25}]' | jpx --csv '@'
# age,name
# 30,alice
# 25,bob
```

**Nesting:** `{"user":{"name":"alice"}}` produces column `user.name`.

**Primitives:** Arrays of non-objects produce a single headerless column.

**Single object:** Treated as a one-row table.

**Output to file:**
```bash
jpx --csv -o data.csv 'users[*]' data.json
```

## TSV (`--tsv`)

Same as CSV but tab-delimited:
```bash
jpx --tsv 'users[*]' data.json
```

## Table (`-t`)

Formatted table output. Best for arrays of objects.

```bash
jpx -t 'users[*].{name: name, age: age}' data.json
```

### Table styles (`--table-style`)

| Style | Description |
|-------|-------------|
| `unicode` | Box-drawing characters (default) |
| `rounded` | Rounded corners |
| `ascii` | ASCII only |
| `markdown` | GitHub-flavored markdown table |
| `plain` | No borders |
| `sharp` | Sharp corners |
| `modern` | Modern style |

```bash
jpx -t --table-style markdown 'users[*]' data.json
jpx -t --table-style ascii 'users[*]' data.json
```

**Display notes:**
- Boolean values are colored (green/red) when color is enabled
- Null values appear dimmed
- Long strings are truncated at 40 characters
- Arrays show `[N items]`, objects show `{N keys}`

## YAML (`-y`)

```bash
jpx -y 'config' data.json
```

Works with any JSON value. Objects and arrays use standard YAML formatting.

## TOML (`--toml`)

```bash
jpx --toml 'config' data.json
```

Requires an object or array at the root level. Arrays are wrapped in an `items` key.

## Lines (`-l`)

One JSON value per line (NDJSON format). Useful for piping array elements:

```bash
jpx -l 'users[*]' data.json | while read line; do echo "$line"; done
```

Non-arrays output a single line.

## Parquet (`--parquet`)

Requires the `parquet` feature and an output file:
```bash
jpx --parquet -o data.parquet 'records[*]' data.json
```

Best for arrays of objects. Uses Snappy compression.

## Format Compatibility

| Format | Arrays | Objects | Primitives | Streaming |
|--------|--------|---------|------------|-----------|
| JSON | Yes | Yes | Yes | Yes (default) |
| Raw | Yes | Yes | Yes | Yes |
| CSV | Yes | Yes | Partial | Yes |
| TSV | Yes | Yes | Partial | Yes |
| Table | Yes | Yes | No | No |
| YAML | Yes | Yes | Yes | No |
| TOML | Yes | Yes | No | No |
| Lines | Yes | Yes | Yes | No |
| Parquet | Yes | No | No | No |
