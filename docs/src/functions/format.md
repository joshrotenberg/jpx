# Format Functions

Data formatting functions for numbers, currencies, and other values.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`from_csv`](#from-csv) | `string -> array` | Parse CSV string into array of arrays (jq parity) |
| [`from_tsv`](#from-tsv) | `string -> array` | Parse TSV string into array of arrays (jq parity) |
| [`to_csv`](#to-csv) | `array -> string` | Convert array to CSV row string (RFC 4180 compliant) |
| [`to_csv_rows`](#to-csv-rows) | `array -> string` | Convert array of arrays to multi-line CSV string |
| [`to_csv_table`](#to-csv-table) | `array, array? -> string` | Convert array of objects to CSV with header row |
| [`to_tsv`](#to-tsv) | `array -> string` | Convert array to TSV row string (tab-separated) |

## Functions

### from_csv

Parse CSV string into array of arrays (jq parity)

**Signature:** `string -> array`

**Examples:**

```text
# Basic CSV parsing
from_csv("a,b,c\\n1,2,3") -> [["a", "b", "c"], ["1", "2", "3"]]
# Quoted fields with commas
from_csv("\"hello, world\",test") -> [["hello, world", "test"]]
# Empty string returns empty array
from_csv("") -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'from_csv("a,b,c\\n1,2,3")'
```

### from_tsv

Parse TSV string into array of arrays (jq parity)

**Signature:** `string -> array`

**Examples:**

```text
# Basic TSV parsing
from_tsv("a\\tb\\tc\\n1\\t2\\t3") -> [["a", "b", "c"], ["1", "2", "3"]]
# Spaces preserved in fields
from_tsv("hello world\\ttest") -> [["hello world", "test"]]
# Empty string returns empty array
from_tsv("") -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'from_tsv("a\\tb\\tc\\n1\\t2\\t3")'
```

### to_csv

Convert array to CSV row string (RFC 4180 compliant)

**Signature:** `array -> string`

**Examples:**

```text
# Simple strings
to_csv(['a', 'b', 'c']) -> "a,b,c"
# Mixed types
to_csv(['hello', `42`, `true`, `null`]) -> "hello,42,true,"
# Value with comma is quoted
to_csv(['hello, world', 'test']) -> "\"hello, world\",test"
# Quotes are doubled
to_csv(['say "hello"', 'test']) -> "\"say \"\"hello\"\"\",test"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_csv(['a', `"b"`, 'c'])'
```

### to_csv_rows

Convert array of arrays to multi-line CSV string

**Signature:** `array -> string`

**Examples:**

```text
# Numeric rows
to_csv_rows([[`1`, `2`, `3`], [`4`, `5`, `6`]]) -> "1,2,3\n4,5,6"
# String rows
to_csv_rows([['a', 'b'], ['c', 'd']]) -> "a,b\nc,d"
# Empty array
to_csv_rows([]) -> ""
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_csv_rows([[`1`, `2`, `3`], [`4`, `5`, `6`]])'
```

### to_csv_table

Convert array of objects to CSV with header row

**Signature:** `array, array? -> string`

**Examples:**

```text
# Keys sorted alphabetically
to_csv_table([{name: 'alice', age: `30`}]) -> "age,name\n30,alice"
# Explicit column order
to_csv_table([{name: 'alice', age: `30`}], ['name', 'age']) -> "name,age\nalice,30"
# Missing fields become empty
to_csv_table([{a: `1`}, {a: `2`, b: `3`}], ['a', 'b']) -> "a,b\n1,\n2,3"
# Empty array
to_csv_table([]) -> ""
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_csv_table([{name: 'alice', age: `30`}])'
```

### to_tsv

Convert array to TSV row string (tab-separated)

**Signature:** `array -> string`

**Examples:**

```text
# Simple strings
to_tsv(['a', 'b', 'c']) -> "a\tb\tc"
# Mixed types
to_tsv(['hello', `42`, `true`]) -> "hello\t42\ttrue"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_tsv(['a', `"b"`, 'c'])'
```

