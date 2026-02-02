# Basic Usage

## Input Sources

jpx can read JSON from multiple sources:

### Standard Input (stdin)

```bash
echo '{"name": "Alice"}' | jpx 'name'
# "Alice"

cat data.json | jpx 'users[*].name'
curl -s https://api.example.com/data | jpx 'results[0]'
```

### File Input

```bash
jpx 'users[*].name' -f data.json
jpx --file users.json 'length(@)'
```

### Null Input

Use `-n` to start with null (useful for functions that don't need input):

```bash
jpx -n 'now()'
# 1705312200

jpx -n 'uuid()'
# "550e8400-e29b-41d4-a716-446655440000"

jpx -n 'range(`1`, `5`)'
# [1, 2, 3, 4]
```

## Output Options

### Pretty Printing (default)

By default, jpx pretty-prints JSON output with colors:

```bash
echo '{"a":1,"b":2}' | jpx '@'
```

Output:
```json
{
  "a": 1,
  "b": 2
}
```

### Compact Output

Use `-c` for single-line output:

```bash
echo '{"a":1,"b":2}' | jpx -c '@'
```

Output:
```json
{"a":1,"b":2}
```

### Raw Strings

Use `-r` to output strings without quotes:

```bash
echo '{"msg": "hello"}' | jpx -r 'msg'
```

Output:
```
hello
```

### Output to File

Write results to a file:

```bash
jpx 'users[*].email' -f data.json -o emails.json
# (writes to emails.json)
```

## Slurp Mode

Read multiple JSON objects into an array:

```bash
echo '{"a":1}
{"b":2}
{"c":3}' | jpx -s 'length(@)'
```

Output:
```
3
```

## Expression as Positional Argument

The expression can be the first positional argument:

```bash
jpx 'users[*].name' -f data.json
```

Or use `-e` / `--expression`:

```bash
jpx -e 'users[*].name' -f data.json
```

## Chaining Expressions

Three ways to chain transformations:

```bash
# 1. Pipes within a single expression (most common)
jpx 'users | [*].name | sort(@)' -f data.json

# 2. Multiple positional arguments
jpx 'users' '[*].name' 'sort(@)' -f data.json

# 3. Multiple -e flags
jpx -e 'users' -e '[*].name' -e 'sort(@)' -f data.json
```

All three are equivalent - each expression receives the output of the previous one.

## Verbose Mode

See expression details and timing:

```bash
echo '{"x": 1}' | jpx -v 'x'
```

Output:
```
Input: object (1 keys)

[1] Expression: x
[1] Result: number (1)
[1] Time: 0.040ms

Total time: 0.214ms
1
```

## Quiet Mode

Suppress errors and warnings:

```bash
jpx -q 'invalid[' -f data.json
# (no error output, exits with non-zero status)
```

## Color Control

Control colorized output:

```bash
jpx --color=always 'name' -f data.json  # Force colors
jpx --color=never 'name' -f data.json   # No colors
jpx --color=auto 'name' -f data.json    # Auto-detect (default)
```
