# Regular Expression Functions

Regular expression functions: matching, replacing, splitting, and pattern extraction.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`regex_count`](#regex-count) | `string, string -> number` | Count regex matches |
| [`regex_extract`](#regex-extract) | `string, string -> array` | Extract regex matches |
| [`regex_match`](#regex-match) | `string, string -> boolean` | Test if string matches regex |
| [`regex_replace`](#regex-replace) | `string, string, string -> string` | Replace regex matches |
| [`regex_split`](#regex-split) | `string, string -> array` | Split string by regex pattern |

## Functions

### regex_count

Count the number of regex matches

**Signature:** `string, string -> number`

**Examples:**

```text
# Count numbers
regex_count('a1b2c3', '\\\\d+') -> 3
# Count vowels
regex_count('hello world', '[aeiou]') -> 3
# No matches
regex_count('abc', '\\d+') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'regex_count(`"a1b2c3"`, `"\\\\d+"`)'
```

### regex_extract

Extract regex matches

**Signature:** `string, string -> array`

**Examples:**

```text
# Extract numbers
regex_extract('a1b2', '\\\\d+') -> [\"1\", \"2\"]
# Extract words
regex_extract('hello world', '\\\\w+') -> [\"hello\", \"world\"]
# No matches
regex_extract('abc', '\\d+') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'regex_extract(`"a1b2"`, `"\\\\d+"`)'
```

### regex_match

Test if string matches regex

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Full match
regex_match('hello', '^h.*o$') -> true
# Contains digits
regex_match('test123', '\\d+') -> true
# No match
regex_match('abc', '^\\d+$') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'regex_match(`"hello"`, `"^h.*o$"`)'
```

### regex_replace

Replace regex matches

**Signature:** `string, string, string -> string`

**Examples:**

```text
# Replace digits
regex_replace('a1b2', '\\\\d+', 'X') -> \"aXbX\"
# Replace spaces
regex_replace('hello world', '\\\\s+', '-') -> \"hello-world\"
# No match unchanged
regex_replace('abc', 'x', 'y') -> \"abc\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'regex_replace(`"a1b2"`, `"\\\\d+"`, `"X"`)'
```

### regex_split

Split a string by a regex pattern

**Signature:** `string, string -> array`

**Examples:**

```text
# Split by comma
regex_split('a,b,c', ',') -> ["a", "b", "c"]
# Split by whitespace
regex_split('hello   world', '\\\\s+') -> ["hello", "world"]
# No match returns single element
regex_split('abc', ',') -> ["abc"]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'regex_split(`"a,b,c"`, `","`)'
```

