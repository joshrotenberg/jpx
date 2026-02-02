# Output Formats

jpx supports multiple output formats beyond JSON, making it easy to integrate with other tools and workflows.

## Available Formats

| Flag | Short | Description |
|------|-------|-------------|
| `--yaml` | `-y` | YAML format |
| `--toml` | | TOML format |
| `--csv` | | Comma-separated values |
| `--tsv` | | Tab-separated values |
| `--lines` | `-l` | One JSON value per line (JSONL) |
| `--table` | `-t` | Formatted table (for arrays of objects) |

## YAML Output

YAML is great for human-readable configuration files and nested data structures.

```bash
# Simple object
echo '{"name": "alice", "age": 30}' | jpx '@' --yaml
```
```yaml
age: 30
name: alice
```

```bash
# Nested structures
echo '{"server": {"host": "localhost", "port": 8080}}' | jpx '@' -y
```
```yaml
server:
  host: localhost
  port: 8080
```

```bash
# Arrays
echo '[{"name": "alice"}, {"name": "bob"}]' | jpx '@' --yaml
```
```yaml
- name: alice
- name: bob
```

### Use Cases

- Converting JSON configs to YAML for Kubernetes, Docker Compose, etc.
- Human-readable output for debugging
- Generating configuration files

```bash
# Extract and convert to YAML
jpx '[*].{name: name, role: role}' -f users.json --yaml > users.yaml
```

## TOML Output

TOML is ideal for configuration files, especially in the Rust ecosystem.

```bash
# Simple object
echo '{"name": "myapp", "version": "1.0.0"}' | jpx '@' --toml
```
```toml
name = "myapp"
version = "1.0.0"
```

```bash
# Nested tables
echo '{"database": {"host": "localhost", "port": 5432}}' | jpx '@' --toml
```
```toml
[database]
host = "localhost"
port = 5432
```

### Use Cases

- Generating Cargo.toml snippets
- Creating configuration files for Rust applications
- Converting JSON API responses to TOML configs

## CSV Output

CSV format is perfect for tabular data that needs to go into spreadsheets or databases.

```bash
# Array of objects becomes a table
echo '[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]' | jpx '@' --csv
```
```csv
age,name
30,alice
25,bob
```

### Nested Object Flattening

Nested objects are automatically flattened using dot notation:

```bash
echo '[
  {"name": "alice", "address": {"city": "NYC", "zip": "10001"}},
  {"name": "bob", "address": {"city": "LA", "zip": "90001"}}
]' | jpx '@' --csv
```
```csv
address.city,address.zip,name
NYC,10001,alice
LA,90001,bob
```

### Arrays in Cells

Arrays are JSON-encoded within cells:

```bash
echo '[{"name": "alice", "tags": ["admin", "dev"]}]' | jpx '@' --csv
```
```csv
name,tags
alice,"[""admin"",""dev""]"
```

### Single Objects

A single object is output as a single-row table:

```bash
echo '{"name": "alice", "role": "admin"}' | jpx '@' --csv
```
```csv
name,role
alice,admin
```

### Array of Primitives

Arrays of primitives become single-column output (no header):

```bash
echo '[1, 2, 3, 4, 5]' | jpx '@' --csv
```
```csv
1
2
3
4
5
```

### Use Cases

- Exporting data for Excel or Google Sheets
- Loading into databases
- Generating reports

```bash
# Export user emails for a mailing list
jpx '[*].{email: email, name: name}' -f users.json --csv > contacts.csv
```

## TSV Output

TSV (tab-separated values) works the same as CSV but uses tabs instead of commas. This is useful for:

- Unix command-line tools that expect tab-separated input
- Pasting directly into spreadsheets
- Avoiding issues with commas in data

```bash
echo '[{"name": "alice", "city": "New York"}, {"name": "bob", "city": "Los Angeles"}]' | jpx '@' --tsv
```
```
city	name
New York	alice
Los Angeles	bob
```

### Use Cases

```bash
# Pipe to awk for further processing
jpx '[*].{name: name, score: score}' -f data.json --tsv | awk -F'\t' '$2 > 90'

# Pipe to column for pretty printing
jpx '@' -f users.json --tsv | column -t -s $'\t'
```

## Lines Output (JSONL)

The `--lines` flag outputs one JSON value per line, also known as JSON Lines (JSONL) or newline-delimited JSON (NDJSON).

```bash
echo '[1, 2, 3, "hello", {"key": "value"}]' | jpx '@' --lines
```
```
1
2
3
"hello"
{"key":"value"}
```

```bash
# Array of objects
echo '[{"id": 1}, {"id": 2}, {"id": 3}]' | jpx '@' -l
```
```json
{"id":1}
{"id":2}
{"id":3}
```

### Use Cases

- Streaming processing with tools like `jq`, `grep`, or `awk`
- Log file format
- Parallel processing (each line is independent)
- Piping to other JSON tools

```bash
# Process each line with another tool
jpx '[*]' -f events.json --lines | while read -r line; do
  echo "$line" | jpx '.timestamp'
done

# Filter with grep
jpx '@' -f logs.json --lines | grep '"level":"error"'

# Count items
jpx '[*]' -f data.json --lines | wc -l
```

## Table Output

The `--table` flag renders arrays of objects as formatted tables, perfect for terminal display.

```bash
echo '[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]' | jpx '@' --table
```
```
┌───────┬─────┐
│ name  │ age │
├───────┼─────┤
│ alice │ 30  │
│ bob   │ 25  │
└───────┴─────┘
```

### Table Styles

Use `--table-style` to choose different table formats:

| Style | Description |
|-------|-------------|
| `unicode` | Default, uses box-drawing characters |
| `ascii` | ASCII characters only (`+`, `-`, `|`) |
| `markdown` | GitHub-flavored markdown tables |
| `plain` | No borders, space-separated |

```bash
# ASCII style (for older terminals)
echo '[{"name": "alice", "age": 30}]' | jpx '@' -t --table-style ascii
```
```
+-------+-----+
| name  | age |
+-------+-----+
| alice | 30  |
+-------+-----+
```

```bash
# Markdown style (for documentation)
echo '[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]' | jpx '@' -t --table-style markdown
```
```
| name  | age |
|-------|-----|
| alice | 30  |
| bob   | 25  |
```

### Use Cases

- Quick data inspection in the terminal
- Generating markdown tables for documentation
- Human-readable output for reports

```bash
# Display users as a table
jpx '[*].{name: name, email: email, role: role}' -f users.json --table

# Generate markdown documentation
jpx '[*].{Function: name, Description: description}' -f functions.json -t --table-style markdown > docs/functions.md
```

## Combining with Expressions

Output formats work with any JMESPath expression:

```bash
# Extract specific fields and output as YAML
echo '[{"name": "alice", "age": 30, "email": "alice@example.com"}]' | \
  jpx '[*].{name: name, email: email}' --yaml
```
```yaml
- email: alice@example.com
  name: alice
```

```bash
# Filter and export as CSV
jpx '[?status == `active`].{id: id, name: name}' -f users.json --csv
```

```bash
# Aggregate and output as TOML
jpx '{total: length(@), active: length([?active])}' -f users.json --toml
```
```toml
active = 42
total = 100
```

## Output to Files

All output formats work with the `-o` flag:

```bash
# Save as YAML
jpx '@' -f config.json --yaml -o config.yaml

# Save as CSV
jpx '[*].{name: name, email: email}' -f users.json --csv -o contacts.csv

# Save as TOML
jpx '@' -f settings.json --toml -o settings.toml
```

## Format Selection Tips

| Use Case | Recommended Format |
|----------|-------------------|
| Human-readable config | `--yaml` |
| Rust/Python config files | `--toml` |
| Spreadsheet export | `--csv` |
| Unix pipeline processing | `--tsv` or `--lines` |
| Streaming/logging | `--lines` |
| Database import | `--csv` |
| API debugging | `--yaml` |
| Terminal display | `--table` |
| Markdown documentation | `--table --table-style markdown` |
