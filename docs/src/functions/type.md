# Type Functions

Type conversion and checking functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`auto_parse`](#auto-parse) | `any -> any` | Intelligently parse strings to numbers, booleans, and nulls |
| [`is_array`](#is-array) | `any -> boolean` | Check if value is an array |
| [`is_boolean`](#is-boolean) | `any -> boolean` | Check if value is a boolean |
| [`is_empty`](#is-empty) | `any -> boolean` | Check if value is empty |
| [`is_null`](#is-null) | `any -> boolean` | Check if value is null |
| [`is_number`](#is-number) | `any -> boolean` | Check if value is a number |
| [`is_object`](#is-object) | `any -> boolean` | Check if value is an object |
| [`is_string`](#is-string) | `any -> boolean` | Check if value is a string |
| [`parse_booleans`](#parse-booleans) | `any -> any` | Recursively convert boolean strings to booleans |
| [`parse_nulls`](#parse-nulls) | `any -> any` | Recursively convert null-like strings to null |
| [`parse_numbers`](#parse-numbers) | `any -> any` | Recursively convert numeric strings to numbers |
| [`to_boolean`](#to-boolean) | `any -> boolean` | Convert value to boolean |
| [`type_of`](#type-of) | `any -> string` | Get the type of a value |

## Functions

### auto_parse

Intelligently parse strings to numbers, booleans, and nulls

**Signature:** `any -> any`

**Examples:**

```text
# Parse mixed types
auto_parse({num: \"42\", bool: \"true\", nil: \"null\"}) -> {num: 42, bool: true, nil: null}
# Parse array
auto_parse([\"42\", \"true\", \"hello\"]) -> [42, true, \"hello\"]
# Nested parsing
auto_parse({a: {b: \"123\"}}) -> {a: {b: 123}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'auto_parse({num: \"42\", bool: \"true\", nil: \"null\"})'
```

### is_array

Check if value is an array

**Signature:** `any -> boolean`

**Examples:**

```text
# Array returns true
is_array([1, 2]) -> true
# String returns false
is_array('hello') -> false
# Object returns false
is_array({}) -> false
# Empty array is array
is_array([]) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_array([1, 2])'
```

### is_boolean

Check if value is a boolean

**Signature:** `any -> boolean`

**Examples:**

```text
# True is boolean
is_boolean(`true`) -> true
# False is boolean
is_boolean(`false`) -> true
# String is not boolean
is_boolean('true') -> false
# Number is not boolean
is_boolean(`1`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_boolean(`true`)'
```

### is_empty

Check if value is empty

**Signature:** `any -> boolean`

**Examples:**

```text
# Empty array
is_empty([]) -> true
# Empty string
is_empty('') -> true
# Empty object
is_empty({}) -> true
# Non-empty array
is_empty([1, 2]) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_empty([])'
```

### is_null

Check if value is null

**Signature:** `any -> boolean`

**Examples:**

```text
# Null returns true
is_null(`null`) -> true
# Empty string is not null
is_null('') -> false
# Zero is not null
is_null(`0`) -> false
# Missing field is null
is_null(missing_field) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_null(`null`)'
```

### is_number

Check if value is a number

**Signature:** `any -> boolean`

**Examples:**

```text
# Integer is number
is_number(`42`) -> true
# Float is number
is_number(`3.14`) -> true
# String is not number
is_number('42') -> false
# Null is not number
is_number(`null`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_number(`42`)'
```

### is_object

Check if value is an object

**Signature:** `any -> boolean`

**Examples:**

```text
# Object returns true
is_object({a: 1}) -> true
# Empty object is object
is_object({}) -> true
# Array is not object
is_object([]) -> false
# String is not object
is_object('hello') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_object({a: 1})'
```

### is_string

Check if value is a string

**Signature:** `any -> boolean`

**Examples:**

```text
# String returns true
is_string('hello') -> true
# Empty string is string
is_string('') -> true
# Number is not string
is_string(`42`) -> false
# Array is not string
is_string([]) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_string(`"hello"`)'
```

### parse_booleans

Recursively convert boolean strings to booleans

**Signature:** `any -> any`

**Examples:**

```text
# Parse true
parse_booleans({active: "true"}) -> {active: true}
# Parse yes
parse_booleans({flag: "YES"}) -> {flag: true}
# Parse 1
parse_booleans({enabled: "1"}) -> {enabled: true}
# Multiple booleans
parse_booleans({off: "false", on: "true"}) -> {off: false, on: true}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_booleans({active: "true"})'
```

### parse_nulls

Recursively convert null-like strings to null

**Signature:** `any -> any`

**Examples:**

```text
# Parse null
parse_nulls({a: "null"}) -> {a: null}
# Parse None
parse_nulls({a: "None"}) -> {a: null}
# Parse nil
parse_nulls({a: "nil"}) -> {a: null}
# Parse undefined
parse_nulls({a: "undefined"}) -> {a: null}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_nulls({a: "null"})'
```

### parse_numbers

Recursively convert numeric strings to numbers

**Signature:** `any -> any`

**Examples:**

```text
# Parse integer
parse_numbers({count: "42"}) -> {count: 42}
# Parse float
parse_numbers({price: "19.99"}) -> {price: 19.99}
# Nested parsing
parse_numbers({a: {b: "123"}}) -> {a: {b: 123}}
# Non-numeric stays string
parse_numbers({val: "42abc"}) -> {val: "42abc"}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_numbers({count: "42"})'
```

### to_boolean

Convert value to boolean

**Signature:** `any -> boolean`

**Examples:**

```text
# String 'true'
to_boolean('true') -> true
# String 'false'
to_boolean('false') -> false
# Non-zero is true
to_boolean(`1`) -> true
# Zero is false
to_boolean(`0`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_boolean(`"true"`)'
```

### type_of

Get the type of a value

**Signature:** `any -> string`

**Examples:**

```text
# Number type
type_of(`42`) -> \"number\"
# String type
type_of('hello') -> \"string\"
# Array type
type_of([1, 2]) -> \"array\"
# Object type
type_of({a: 1}) -> \"object\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'type_of(`42`)'
```

