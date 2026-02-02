# Multi-Match Functions

Functions for matching multiple patterns or expressions in a single operation.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`extract_all`](#extract-all) | `string, array[string] -> array[object]` | Extract all pattern matches with positions (Aho-Corasick) |
| [`extract_between`](#extract-between) | `string, string, string -> string\|null` | Extract text between two delimiters |
| [`match_all`](#match-all) | `string, array[string] -> boolean` | Check if string contains all of the patterns (Aho-Corasick) |
| [`match_any`](#match-any) | `string, array[string] -> boolean` | Check if string contains any of the patterns (Aho-Corasick) |
| [`match_count`](#match-count) | `string, array[string] -> number` | Count total pattern matches in string (Aho-Corasick) |
| [`match_positions`](#match-positions) | `string, array[string] -> array[object]` | Get start/end positions of all pattern matches (Aho-Corasick) |
| [`match_which`](#match-which) | `string, array[string] -> array[string]` | Return array of patterns that match the string (Aho-Corasick) |
| [`replace_many`](#replace-many) | `string, object -> string` | Replace multiple patterns simultaneously (Aho-Corasick) |
| [`split_keep`](#split-keep) | `string, string -> array[string]` | Split string keeping delimiters in result |
| [`tokenize`](#tokenize) | `string, object? -> array[string]` | Smart word tokenization with optional lowercase and min_length |

## Functions

### extract_all

Extract all pattern matches with positions (Aho-Corasick)

**Signature:** `string, array[string] -> array[object]`

**Examples:**

```text
# Multiple patterns
extract_all('error warning', ['error', 'warning']) -> [{pattern: 'error', match: 'error', start: 0, end: 5}, ...]
# Overlapping matches
extract_all('abab', ['a', 'b']) -> [{pattern: 'a', match: 'a', start: 0, end: 1}, ...]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'extract_all(`"error warning"`, ['error', 'warning'])'
```

### extract_between

Extract text between two delimiters

**Signature:** `string, string, string -> string|null`

**Examples:**

```text
# HTML tag content
extract_between('<title>Page</title>', '<title>', '</title>') -> \"Page\"
# Bracketed content
extract_between('Hello [world]!', '[', ']') -> \"world\"
# No delimiters found
extract_between('no match', '[', ']') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'extract_between(`"<title>Page</title>"`, `"<title>"`, `"</title>"`)'
```

### match_all

Check if string contains all of the patterns (Aho-Corasick)

**Signature:** `string, array[string] -> boolean`

**Examples:**

```text
# All patterns found
match_all('hello world', ['hello', 'world']) -> true
# Missing pattern
match_all('hello world', ['hello', 'foo']) -> false
# Empty patterns
match_all('abc', []) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_all(`"hello world"`, ['hello', 'world'])'
```

### match_any

Check if string contains any of the patterns (Aho-Corasick)

**Signature:** `string, array[string] -> boolean`

**Examples:**

```text
# One pattern found
match_any('hello world', ['world', 'foo']) -> true
# No patterns found
match_any('hello world', ['foo', 'bar']) -> false
# Empty patterns
match_any('abc', []) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_any(`"hello world"`, ['world', 'foo'])'
```

### match_count

Count total pattern matches in string (Aho-Corasick)

**Signature:** `string, array[string] -> number`

**Examples:**

```text
# Count all matches
match_count('abcabc', ['a', 'b']) -> 4
# Repeated pattern
match_count('hello', ['l']) -> 2
# No matches
match_count('abc', ['x']) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_count(`"abcabc"`, ['a', 'b'])'
```

### match_positions

Get start/end positions of all pattern matches (Aho-Corasick)

**Signature:** `string, array[string] -> array[object]`

**Examples:**

```text
# Find positions
match_positions('The quick fox', ['quick', 'fox']) -> [{pattern: 'quick', start: 4, end: 9}, ...]
# Multiple occurrences
match_positions('abab', ['ab']) -> [{pattern: 'ab', start: 0, end: 2}, {pattern: 'ab', start: 2, end: 4}]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_positions(`"The quick fox"`, ['quick', 'fox'])'
```

### match_which

Return array of patterns that match the string (Aho-Corasick)

**Signature:** `string, array[string] -> array[string]`

**Examples:**

```text
# Find matching patterns
match_which('hello world', ['hello', 'foo', 'world']) -> [\"hello\", \"world\"]
# Partial matches
match_which('abc', ['a', 'b', 'x']) -> [\"a\", \"b\"]
# No matches
match_which('abc', ['x', 'y']) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_which(`"hello world"`, ['hello', `"foo"`, 'world'])'
```

### replace_many

Replace multiple patterns simultaneously (Aho-Corasick)

**Signature:** `string, object -> string`

**Examples:**

```text
# Multiple replacements
replace_many('hello world', {hello: 'hi', world: 'earth'}) -> \"hi earth\"
# Repeated pattern
replace_many('aaa', {a: 'b'}) -> \"bbb\"
# No matches
replace_many('abc', {x: 'y'}) -> \"abc\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'replace_many(`"hello world"`, {hello: 'hi', world: 'earth'})'
```

### split_keep

Split string keeping delimiters in result

**Signature:** `string, string -> array[string]`

**Examples:**

```text
# Keep dashes
split_keep('a-b-c', '-') -> [\"a\", \"-\", \"b\", \"-\", \"c\"]
# Keep spaces
split_keep('hello world', ' ') -> [\"hello\", \" \", \"world\"]
# No delimiter
split_keep('abc', '-') -> [\"abc\"]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'split_keep(`"a-b-c"`, `"-"`)'
```

### tokenize

Smart word tokenization with optional lowercase and min_length

**Signature:** `string, object? -> array[string]`

**Examples:**

```text
# Basic tokenization
tokenize('Hello, World!') -> [\"Hello\", \"World\"]
# Lowercase option
tokenize('Hello, World!', {lowercase: `true`}) -> [\"hello\", \"world\"]
# Minimum length
tokenize('a bb ccc', {min_length: `2`}) -> [\"bb\", \"ccc\"]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'tokenize(`"Hello, World!"`)'
```

