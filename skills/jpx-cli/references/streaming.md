# jpx Streaming Mode

Process NDJSON (newline-delimited JSON) input line by line with constant memory usage.

## How It Works

- Each line is parsed as an independent JSON value
- Expressions are evaluated per-line
- Null results are automatically skipped
- Invalid JSON lines produce a warning on stderr (suppressed with `-q`)
- Empty lines are skipped

## Basic Streaming

```bash
# Input: one JSON object per line
echo '{"name":"alice","age":30}
{"name":"bob","age":25}
{"name":"carol","age":35}' | jpx --stream 'name'

# Output:
# "alice"
# "bob"
# "carol"
```

## Streaming with CSV/TSV

Headers are derived from the first result object. Subsequent records use the same column order.

```bash
# CSV output
cat users.ndjson | jpx --stream --csv '@'
# age,name
# 30,alice
# 25,bob

# TSV output
cat users.ndjson | jpx --stream --tsv '@'

# CSV with expression (reshape before output)
cat events.ndjson | jpx --stream --csv '{user: user.name, action: event_type}'
```

**Behavior notes:**
- Extra fields in later records (not in the first record) are silently dropped
- Missing fields in later records produce empty cells
- Non-object results get a single "value" column
- Nested objects are flattened with dot notation (e.g., `addr.city`)

## Filtering in Streaming Mode

Since each line is independent, you can't use array filters directly. Instead, return null for lines you want to skip:

```bash
# Using if() to filter (null results are skipped)
cat events.ndjson | jpx --stream 'if(level == `error`, @, null)'

# Expression that naturally returns null skips the line
cat events.ndjson | jpx --stream '[?level == `error`] | [0]'
```

## Multiple Expressions

Expressions are chained -- each receives the output of the previous:

```bash
cat data.ndjson | jpx --stream 'items[*]' 'sort(@)' '[0]'
```

## Unbuffered Output

For real-time processing (e.g., tailing a log):

```bash
tail -f events.ndjson | jpx --stream --unbuffered '{ts: timestamp, msg: message}'
```

## Incompatible Flags

These require buffering the full dataset and cannot be used with `--stream`:
- `--table` / `-t`
- `--yaml` / `-y`
- `--toml`
- `--lines` / `-l`
- `--slurp` / `-s`
- `--null-input` / `-n`

Use `--slurp` to collect streaming input into an array first if you need these formats.

## Performance

Streaming mode processes input with O(1) memory regardless of file size. Useful for:
- Large NDJSON files (gigabytes)
- Log file processing
- Continuous data streams (with `--unbuffered`)
- Pipelines where you don't need the full dataset in memory
