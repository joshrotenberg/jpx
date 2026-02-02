# Working with Files

jpx provides several ways to read input and write output, making it flexible for different workflows.

## Reading Input

### From stdin (default)

Pipe JSON directly to jpx:

```bash
echo '{"name": "alice"}' | jpx 'name'
# "alice"

cat data.json | jpx 'users[*].name'

curl -s https://api.example.com/data | jpx 'items[0]'
```

### From a File

Use `-f` or `--file` to read from a file:

```bash
jpx -f data.json 'users[*].name'

jpx --file config.json 'settings.theme'
```

### Null Input

Use `-n` or `--null-input` for expressions that don't need input data:

```bash
jpx -n 'now()'
# "2024-01-15T10:30:00Z"

jpx -n 'uuid()'
# "550e8400-e29b-41d4-a716-446655440000"

jpx -n 'range(1, 5)'
# [1, 2, 3, 4]
```

## Writing Output

### To stdout (default)

Output goes to stdout with pretty-printing by default:

```bash
jpx -f data.json 'users[0]'
# {
#   "name": "alice",
#   "age": 30
# }
```

### To a File

Use `-o` or `--output` to write to a file:

```bash
jpx -f data.json 'users[?active]' -o active-users.json

jpx -f input.json 'transform(@)' --output result.json
```

### Compact Output

Use `-c` or `--compact` for single-line JSON (no pretty-printing):

```bash
jpx -c -f data.json 'users[0]'
# {"name":"alice","age":30}
```

## Processing Multiple Files

### Sequential Processing

Process files one at a time in a loop:

```bash
for f in *.json; do
  jpx -f "$f" 'metadata.title'
done
```

### Combining Files with Slurp

Use `-s` or `--slurp` to combine multiple JSON values into an array:

```bash
cat file1.json file2.json file3.json | jpx -s 'length(@)'
# 3

cat *.json | jpx -s '[*].name'
```

### Streaming JSON Lines (NDJSON)

Use `--stream` (or `--each`) for newline-delimited JSON:

```bash
# Process each line independently
cat events.ndjson | jpx --stream 'event_type'

# Filter and transform streaming data
cat logs.ndjson | jpx --stream '[?level == `error`].message'
```

Streaming mode uses constant memory regardless of file size.

## Query Files

Store queries in files for reuse:

### Plain Text Query Files

```bash
# query.txt contains: users[?active].{name: name, email: email}
jpx -Q query.txt -f data.json
```

### Query Libraries (.jpx)

Create reusable query collections:

```sql
-- queries.jpx
-- :name active-users
-- :description Get all active users
users[?active]

-- :name user-emails
-- :description Extract just the email addresses
users[*].email
```

Run named queries:

```bash
jpx -Q queries.jpx:active-users -f data.json

# Or with --query flag
jpx -Q queries.jpx --query user-emails -f data.json

# List available queries
jpx -Q queries.jpx --list-queries
```

See [Query Files](./query-files.md) for more details.

## JSON Patch Operations

### Generate Patches

Compare two files and generate a JSON Patch (RFC 6902):

```bash
jpx --diff original.json modified.json > changes.patch
```

### Apply Patches

Apply a patch to a document:

```bash
jpx --patch changes.patch -f document.json -o updated.json
```

### Merge Patches

Apply a JSON Merge Patch (RFC 7396):

```bash
jpx --merge updates.json -f document.json
```

## Tips

### Checking File Validity

Use `--stats` to quickly validate and analyze a JSON file:

```bash
jpx --stats -f data.json
```

### Exploring File Structure

Use `--paths` to see all paths in a JSON document:

```bash
jpx --paths -f data.json

# With types
jpx --paths --types -f data.json

# With values
jpx --paths --values -f data.json
```

### Benchmarking File Operations

Measure query performance on a file:

```bash
jpx --bench 1000 -f large-data.json 'complex[*].transformation'
```
