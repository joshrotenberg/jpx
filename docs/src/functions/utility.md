# Utility Functions

General utility functions that don't fit other categories.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`coalesce`](#coalesce) | `any... -> any` | Return first non-null value |
| [`default`](#default) | `any, any -> any` | Return default value if null |
| [`env`](#env) | `-> object` | Get all environment variables as an object |
| [`get_env`](#get-env) | `string -> string \| null` | Get a single environment variable by name |
| [`if`](#if) | `boolean, any, any -> any` | Conditional expression |
| [`json_decode`](#json-decode) | `string -> any` | Parse JSON string |
| [`json_encode`](#json-encode) | `any -> string` | Serialize value to JSON string |
| [`json_pointer`](#json-pointer) | `any, string -> any` | Access value using JSON Pointer (RFC 6901) |
| [`now`](#now) | `-> number` | Current Unix timestamp in seconds |
| [`now_ms`](#now-ms) | `-> number` | Current Unix timestamp in milliseconds |
| [`pretty`](#pretty) | `any, number? -> string` | Pretty-print value as formatted JSON string |

## Functions

### coalesce

Return first non-null value

**Signature:** `any... -> any`

**Examples:**

```text
# Skip nulls
coalesce(`null`, `null`, 'value') -> \"value\"
# Field fallback
coalesce(field1, field2, 'default') -> first non-null
# First wins
coalesce('first', 'second') -> 'first'
# All null
coalesce(`null`, `null`) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'coalesce(`null`, `null`, `"value"`)'
```

### default

Return default value if null

**Signature:** `any, any -> any`

**Examples:**

```text
# Null uses default
default(`null`, 'fallback') -> \"fallback\"
# Non-null keeps value
default('value', 'fallback') -> 'value'
# Missing field default
default(missing_field, `0`) -> 0
# Empty string not null
default('', 'fallback') -> ''
```

**CLI Usage:**

```bash
echo '{}' | jpx 'default(`null`, `"fallback"`)'
```

### env

Get all environment variables as an object

**Signature:** `-> object`

**Examples:**

```text
# All env vars
env() -> {HOME: '/Users/...', PATH: '...', ...}
# Access specific var
env().HOME -> home directory
# List all var names
keys(env()) -> list of var names
```

**CLI Usage:**

```bash
echo '{}' | jpx 'env()'
```

### get_env

Get a single environment variable by name

**Signature:** `string -> string | null`

**Examples:**

```text
# Get home directory
get_env('HOME') -> \"/Users/josh\"
# Get PATH
get_env('PATH') -> system PATH
# Returns null if not set
get_env('MISSING') -> null
# With default value
default(get_env('PORT'), '8080') -> with fallback
```

**CLI Usage:**

```bash
echo '{}' | jpx 'get_env(`"HOME"`)'
```

### if

Conditional expression

**Signature:** `boolean, any, any -> any`

**Examples:**

```text
# True branch
if(`true`, 'yes', 'no') -> \"yes\"
# False branch
if(`false`, 'yes', 'no') -> \"no\"
# Conditional logic
if(age >= `18`, 'adult', 'minor') -> depends
# Safe access
if(is_empty(arr), 'none', first(arr)) -> value or none
```

**CLI Usage:**

```bash
echo '{}' | jpx 'if(`true`, `"yes"`, `"no"`)'
```

### json_decode

Parse JSON string

**Signature:** `string -> any`

**Examples:**

```text
# Parse object
json_decode('{\"a\": 1}') -> {a: 1}
# Parse array
json_decode('[1, 2, 3]') -> [1, 2, 3]
# Parse string
json_decode('\"hello\"') -> \"hello\"
# Parse field
json_decode(json_field) -> parsed
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_decode(`"{\"a\": 1}"`)'
```

### json_encode

Serialize value to JSON string

**Signature:** `any -> string`

**Examples:**

```text
# Encode object
json_encode({a: 1}) -> \"{\\\"a\\\":1}\"
# Encode array
json_encode([1, 2, 3]) -> \"[1,2,3]\"
# Encode string
json_encode('hello') -> \"\\\"hello\\\"\"
# Encode any value
json_encode(data) -> JSON string
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_encode({a: 1})'
```

### json_pointer

Access value using JSON Pointer (RFC 6901)

**Signature:** `any, string -> any`

**Examples:**

```text
# Nested access
json_pointer({foo: {bar: 1}}, '/foo/bar') -> 1
# Array access
json_pointer(data, '/0/name') -> first item name
# Top-level access
json_pointer({a: 1}, '/a') -> 1
# Missing path
json_pointer(data, '/missing') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_pointer({foo: {bar: 1}}, `"/foo/bar"`)'
```

### now

Current Unix timestamp in seconds

**Signature:** `-> number`

**Examples:**

```text
# Current timestamp
now() -> 1699900000
# Subtract seconds
now() - 3600 -> one hour ago
# Add one day
now() + 86400 -> tomorrow
# Convert to string
from_epoch(now()) -> current ISO
```

**CLI Usage:**

```bash
echo '{}' | jpx 'now()'
```

### now_ms

Current Unix timestamp in milliseconds

**Signature:** `-> number`

**Examples:**

```text
# Current time in ms
now_ms() -> 1699900000000
# Calculate duration
now_ms() - start_ms -> elapsed ms
# Convert to seconds
now_ms() / `1000` -> seconds
# Convert to string
from_epoch_ms(now_ms()) -> current ISO
```

**CLI Usage:**

```bash
echo '{}' | jpx 'now_ms()'
```

### pretty

Pretty-print value as formatted JSON string

**Signature:** `any, number? -> string`

**Examples:**

```text
# Default 2-space indent
pretty({a: 1}) -> \"{\n  \\\"a\\\": 1\n}\"
# Custom indent
pretty({a: 1}, `4`) -> 4-space indent
# Format for display
pretty(config) -> readable JSON
# Embed in output
sprintf('Config:\n%s', pretty(@)) -> embedded
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pretty({a: 1})'
```

