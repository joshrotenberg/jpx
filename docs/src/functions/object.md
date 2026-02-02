# Object Functions

Functions for working with JSON objects: merging, filtering keys/values, and transformations.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`camel_keys`](#camel-keys) | `any -> any` | Recursively convert all keys to camelCase |
| [`chunk_by_size`](#chunk-by-size) | `array, number -> array` | Split an array into chunks of approximately the specified byte size |
| [`compact_deep`](#compact-deep) | `array -> array` | Recursively compact arrays, removing nulls at all levels |
| [`completeness`](#completeness) | `object -> number` | Calculate percentage of non-null fields (0-100) |
| [`data_quality_score`](#data-quality-score) | `any -> object` | Analyze data quality and return score with detailed issues |
| [`deep_diff`](#deep-diff) | `object, object -> object` | Structural diff between two objects |
| [`deep_equals`](#deep-equals) | `any, any -> boolean` | Deep equality check for any two values |
| [`deep_merge`](#deep-merge) | `object, object -> object` | Recursively merge objects |
| [`defaults`](#defaults) | `object, object -> object` | Assign default values for missing keys (shallow) |
| [`defaults_deep`](#defaults-deep) | `object, object -> object` | Recursively assign default values for missing keys |
| [`delete_path`](#delete-path) | `any, string -> any` | Delete value at JSON pointer path (immutable) |
| [`estimate_size`](#estimate-size) | `any -> number` | Estimate the JSON serialization size in bytes |
| [`flatten`](#flatten) | `object -> object` | Alias for flatten_keys - flatten nested object with dot notation keys |
| [`flatten_array`](#flatten-array) | `any, string? -> object` | Flatten nested objects and arrays with dot notation keys (arrays use numeric indices) |
| [`flatten_keys`](#flatten-keys) | `object -> object` | Flatten nested object with dot notation keys |
| [`from_items`](#from-items) | `array -> object` | Convert array of [key, value] pairs to object |
| [`get`](#get) | `any, string, any? -> any` | Get value at dot-separated path with optional default |
| [`has`](#has) | `any, string -> boolean` | Check if dot-separated path exists |
| [`has_same_shape`](#has-same-shape) | `any, any -> boolean` | Check if two values have the same structure (ignoring actual values) |
| [`infer_schema`](#infer-schema) | `any -> object` | Infer a JSON Schema-like type description from a value |
| [`invert`](#invert) | `object -> object` | Swap keys and values |
| [`items`](#items) | `object -> array` | Convert object to array of [key, value] pairs |
| [`kebab_keys`](#kebab-keys) | `any -> any` | Recursively convert all keys to kebab-case |
| [`leaves`](#leaves) | `any -> array` | Get all leaf values (non-object, non-array) |
| [`pascal_keys`](#pascal-keys) | `any -> any` | Recursively convert all keys to PascalCase |
| [`leaves_with_paths`](#leaves-with-paths) | `any -> array` | Get all leaf values with their JSON pointer paths |
| [`mask`](#mask) | `string, number? -> string` | Mask a string, showing only the last N characters |
| [`omit`](#omit) | `object, array -> object` | Remove specific keys from object |
| [`paginate`](#paginate) | `array, number, number -> object` | Get a page of items from an array with metadata |
| [`paths`](#paths) | `any -> array` | List all JSON pointer paths in value |
| [`paths_to`](#paths-to) | `any, string -> array` | Find all dot-notation paths to a key anywhere in structure |
| [`pick`](#pick) | `object, array -> object` | Select specific keys from object |
| [`pluck_deep`](#pluck-deep) | `any, string -> array` | Find all values for a key anywhere in nested structure |
| [`redact`](#redact) | `any, array -> any` | Recursively replace values at specified keys with [REDACTED] |
| [`redact_keys`](#redact-keys) | `any, string -> any` | Recursively redact keys matching a regex pattern |
| [`remove_empty`](#remove-empty) | `any -> any` | Recursively remove nulls, empty strings, empty arrays, and empty objects |
| [`remove_empty_strings`](#remove-empty-strings) | `any -> any` | Recursively remove empty string values |
| [`remove_nulls`](#remove-nulls) | `any -> any` | Recursively remove null values |
| [`rename_keys`](#rename-keys) | `object, object -> object` | Rename object keys |
| [`set_path`](#set-path) | `any, string, any -> any` | Set value at JSON pointer path (immutable) |
| [`shouty_kebab_keys`](#shouty-kebab-keys) | `any -> any` | Recursively convert all keys to SHOUTY-KEBAB-CASE |
| [`shouty_snake_keys`](#shouty-snake-keys) | `any -> any` | Recursively convert all keys to SHOUTY_SNAKE_CASE |
| [`snake_keys`](#snake-keys) | `any -> any` | Recursively convert all keys to snake_case |
| [`train_keys`](#train-keys) | `any -> any` | Recursively convert all keys to Train-Case |
| [`structural_diff`](#structural-diff) | `any, any -> object` | Compare two values and return their structural differences |
| [`template`](#template) | `object, string -> string` | Expand a template string with values from an object using {{key}} syntax |
| [`template_strict`](#template-strict) | `object, string -> string \| null` | Expand a template string, returning null if any variable is missing |
| [`truncate_to_size`](#truncate-to-size) | `array, number -> array` | Truncate an array to fit within approximately the specified byte size |
| [`type_consistency`](#type-consistency) | `array -> object` | Check if array elements have consistent types |
| [`unflatten`](#unflatten) | `object -> object` | Alias for unflatten_keys - restore nested object from dot notation keys |
| [`unflatten_keys`](#unflatten-keys) | `object -> object` | Restore nested object from dot notation keys |
| [`with_entries`](#with-entries) | `object, string -> object` | Transform object entries using an expression (jq parity) |

## Functions

### camel_keys

Recursively convert all keys to camelCase

**Signature:** `any -> any`

**Examples:**

```text
# Snake to camel
camel_keys({user_name: "alice"}) -> {userName: "alice"}
# Kebab to camel
camel_keys({"user-name": "bob"}) -> {userName: "bob"}
# Nested
camel_keys({user_info: {first_name: "x"}}) -> {userInfo: {firstName: "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'camel_keys({user_name: "alice"})'
```

### chunk_by_size

Split an array into chunks of approximately the specified byte size

**Signature:** `array, number -> array`

**Examples:**

```text
# All fit in one chunk
chunk_by_size([{a:1}, {b:2}, {c:3}], `100`) -> [[{a:1}, {b:2}, {c:3}]]
# Split into chunks
chunk_by_size([{a:1}, {b:2}], `10`) -> [[{a:1}], [{b:2}]]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'chunk_by_size([{a:1}, {b:2}, {c:3}], `100`)'
```

### compact_deep

Recursively compact arrays, removing nulls at all levels

**Signature:** `array -> array`

**Examples:**

```text
# Remove nulls from nested arrays
compact_deep([[1, null], [null, 2]]) -> [[1], [2]]
# Deep nesting
compact_deep([[1, null], [null, [2, null]]]) -> [[1], [[2]]]
# Simple array
compact_deep([1, null, 3]) -> [1, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'compact_deep([[1, null], [null, 2]])'
```

### completeness

Calculate percentage of non-null fields (0-100)

**Signature:** `object -> number`

**Examples:**

```text
# All fields filled
completeness({a: 1, b: 2, c: 3}) -> 100
# One of three filled
completeness({a: 1, b: null, c: null}) -> 33.33
# Nested null counted
completeness({a: 1, b: {c: null}}) -> 66.67
# Empty object is complete
completeness({}) -> 100
```

**CLI Usage:**

```bash
echo '{}' | jpx 'completeness({a: 1, b: 2, c: 3})'
```

### data_quality_score

Analyze data quality and return score with detailed issues

**Signature:** `any -> object`

**Examples:**

```text
# Perfect data
data_quality_score({a: 1, b: 'hello'}).score -> 100
# Issues detected
data_quality_score({a: null, b: ''}).issues -> [{path: 'a', issue: 'null'}, ...]
# Count null fields
data_quality_score(users).null_count -> 5
# Count type mismatches
data_quality_score(data).type_inconsistencies -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'data_quality_score({a: 1, b: 'hello'}).score'
```

### deep_diff

Structural diff between two objects

**Signature:** `object, object -> object`

**Examples:**

```text
# Changed value
deep_diff({a: 1}, {a: 2}) -> {added: {}, removed: {}, changed: {a: {from: 1, to: 2}}}
# Added and removed
deep_diff({a: 1}, {b: 2}) -> {added: {b: 2}, removed: {a: 1}, changed: {}}
# No differences
deep_diff({a: 1}, {a: 1}) -> {added: {}, removed: {}, changed: {}}
# New key added
deep_diff({}, {a: 1}) -> {added: {a: 1}, removed: {}, changed: {}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'deep_diff({a: 1}, {a: 2})'
```

### deep_equals

Deep equality check for any two values

**Signature:** `any, any -> boolean`

**Examples:**

```text
# Equal nested objects
deep_equals({a: {b: 1}}, {a: {b: 1}}) -> true
# Different values
deep_equals({a: 1}, {a: 2}) -> false
# Equal arrays
deep_equals([1, 2], [1, 2]) -> true
# Order matters
deep_equals([1, 2], [2, 1]) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'deep_equals({a: {b: 1}}, {a: {b: 1}})'
```

### deep_merge

Recursively merge objects

**Signature:** `object, object -> object`

**Examples:**

```text
# Merge nested objects
deep_merge({a: {b: 1}}, {a: {c: 2}}) -> {a: {b: 1, c: 2}}
# Merge flat objects
deep_merge({a: 1}, {b: 2}) -> {a: 1, b: 2}
# Later values override
deep_merge({a: 1}, {a: 2}) -> {a: 2}
# Merge with empty object
deep_merge({}, {a: 1}) -> {a: 1}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'deep_merge({a: {b: 1}}, {a: {c: 2}})'
```

### defaults

Assign default values for missing keys (shallow)

**Signature:** `object, object -> object`

**Examples:**

```text
# Keep existing, add missing
defaults({a: 1}, {a: 2, b: 3}) -> {a: 1, b: 3}
# All defaults applied
defaults({}, {a: 1, b: 2}) -> {a: 1, b: 2}
# No defaults needed
defaults({a: 1, b: 2}, {}) -> {a: 1, b: 2}
# Different keys
defaults({x: 1}, {y: 2}) -> {x: 1, y: 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'defaults({a: 1}, {a: 2, b: 3})'
```

### defaults_deep

Recursively assign default values for missing keys

**Signature:** `object, object -> object`

**Examples:**

```text
# Merge nested defaults
defaults_deep({a: {b: 1}}, {a: {c: 2}}) -> {a: {b: 1, c: 2}}
# Keep existing nested
defaults_deep({a: {b: 1}}, {a: {b: 2}}) -> {a: {b: 1}}
# Apply all nested defaults
defaults_deep({}, {a: {b: 1}}) -> {a: {b: 1}}
# Different structure
defaults_deep({x: 1}, {y: {z: 2}}) -> {x: 1, y: {z: 2}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'defaults_deep({a: {b: 1}}, {a: {c: 2}})'
```

### delete_path

Delete value at JSON pointer path (immutable)

**Signature:** `any, string -> any`

**Examples:**

```text
# Delete top-level key
delete_path({a: 1, b: 2}, '/b') -> {a: 1}
# Delete nested key
delete_path({a: {b: 1, c: 2}}, '/a/b') -> {a: {c: 2}}
# Delete array element
delete_path([1, 2, 3], '/1') -> [1, 3]
# Path not found
delete_path({a: 1}, '/x') -> {a: 1}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'delete_path({a: 1, b: 2}, `"/b"`)'
```

### estimate_size

Estimate the JSON serialization size in bytes

**Signature:** `any -> number`

**Examples:**

```text
# String with quotes
estimate_size(`"hello"`) -> 7
# Simple object
estimate_size({a: 1}) -> 7
# Simple array
estimate_size([1, 2, 3]) -> 7
```

**CLI Usage:**

```bash
echo '{}' | jpx 'estimate_size(`"hello"`)'
```

### flatten

Alias for flatten_keys - flatten nested object with dot notation keys

**Signature:** `object -> object`

**Examples:**

```text
# Simple nested
flatten({a: {b: 1}}) -> {\"a.b\": 1}
# Deep nested
flatten({a: {b: {c: 1}}}) -> {\"a.b.c\": 1}
# Flat object
flatten({a: 1, b: 2}) -> {\"a\": 1, \"b\": 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flatten({a: {b: 1}})'
```

### flatten_array

Flatten nested objects and arrays with dot notation keys (arrays use numeric indices)

**Signature:** `any, string? -> object`

**Examples:**

```text
# Array with indices
flatten_array({a: [1, 2]}) -> {\"a.0\": 1, \"a.1\": 2}
# Nested object with array
flatten_array({a: {b: [1, 2]}}) -> {\"a.b.0\": 1, \"a.b.1\": 2}
# Array of objects
flatten_array([{a: 1}, {b: 2}]) -> {\"0.a\": 1, \"1.b\": 2}
# Custom separator
flatten_array({a: {b: 1}}, '/') -> {\"a/b\": 1}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flatten_array({a: [1, 2]})'
```

### flatten_keys

Flatten nested object with dot notation keys

**Signature:** `object -> object`

**Examples:**

```text
# Simple nested
flatten_keys({a: {b: 1}}) -> {\"a.b\": 1}
# Deep nested
flatten_keys({a: {b: {c: 1}}}) -> {\"a.b.c\": 1}
# Flat object
flatten_keys({a: 1, b: 2}) -> {\"a\": 1, \"b\": 2}
# Mixed nesting
flatten_keys({x: {y: 1}, z: 2}) -> {\"x.y\": 1, \"z\": 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flatten_keys({a: {b: 1}})'
```

### from_items

Convert array of [key, value] pairs to object

**Signature:** `array -> object`

**JEP:** JEP-013

**Examples:**

```text
# Single pair
from_items([['a', 1]]) -> {a: 1}
# Multiple pairs
from_items([['a', 1], ['b', 2]]) -> {a: 1, b: 2}
# Empty array
from_items([]) -> {}
# String value
from_items([['x', 'hello']]) -> {x: 'hello'}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'from_items([['a', 1]])'
```

### get

Get value at dot-separated path with optional default

**Signature:** `any, string, any? -> any`

**Examples:**

```text
# Nested path
get({a: {b: 1}}, 'a.b') -> 1
# Top-level key
get({a: 1}, 'a') -> 1
# Missing with default
get({a: 1}, 'x', 'default') -> 'default'
# Deep path
get({a: {b: {c: 3}}}, 'a.b.c') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'get({a: {b: 1}}, `"a.b"`)'
```

### has

Check if dot-separated path exists

**Signature:** `any, string -> boolean`

**Examples:**

```text
# Nested path exists
has({a: {b: 1}}, 'a.b') -> true
# Top-level exists
has({a: 1}, 'a') -> true
# Key missing
has({a: 1}, 'b') -> false
# Nested missing
has({a: {b: 1}}, 'a.c') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'has({a: {b: 1}}, `"a.b"`)'
```

### has_same_shape

Check if two values have the same structure (ignoring actual values)

**Signature:** `any, any -> boolean`

**Examples:**

```text
# Same keys, different values
has_same_shape({a: 1, b: 2}, {a: 99, b: 100}) -> true
# Different keys
has_same_shape({a: 1}, {a: 1, b: 2}) -> false
# Same length arrays
has_same_shape([1, 2], [3, 4]) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'has_same_shape({a: 1, b: 2}, {a: 99, b: 100})'
```

### infer_schema

Infer a JSON Schema-like type description from a value

**Signature:** `any -> object`

**Examples:**

```text
# Number type
infer_schema(`42`) -> {type: "number"}
# Object schema
infer_schema({name: "alice", age: 30}) -> {type: "object", properties: {name: {type: "string"}, age: {type: "number"}}}
# Array schema
infer_schema([1, 2, 3]) -> {type: "array", items: {type: "number"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'infer_schema(`42`)'
```

### invert

Swap keys and values

**Signature:** `object -> object`

**Examples:**

```text
# Swap key and value
invert({a: 'x'}) -> {x: 'a'}
# Multiple pairs
invert({a: 'b', c: 'd'}) -> {b: 'a', d: 'c'}
# Empty object
invert({}) -> {}
# Real-world example
invert({name: 'id', value: 'data'}) -> {id: 'name', data: 'value'}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'invert({a: 'x'})'
```

### items

Convert object to array of [key, value] pairs

**Signature:** `object -> array`

**JEP:** JEP-013

**Examples:**

```text
# Single key
items({a: 1}) -> [[\"a\", 1]]
# Multiple keys
items({a: 1, b: 2}) -> [[\"a\", 1], [\"b\", 2]]
# Empty object
items({}) -> []
# String value
items({x: 'hello'}) -> [[\"x\", \"hello\"]]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'items({a: 1})'
```

### kebab_keys

Recursively convert all keys to kebab-case

**Signature:** `any -> any`

**Examples:**

```text
# Camel to kebab
kebab_keys({userName: "alice"}) -> {"user-name": "alice"}
# Snake to kebab
kebab_keys({user_name: "bob"}) -> {"user-name": "bob"}
# Nested
kebab_keys({userInfo: {firstName: "x"}}) -> {"user-info": {"first-name": "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'kebab_keys({userName: "alice"})'
```

### leaves

Get all leaf values (non-object, non-array)

**Signature:** `any -> array`

**Examples:**

```text
# Mixed structure
leaves({a: 1, b: [2, 3]}) -> [1, 2, 3]
# Nested object
leaves({a: {b: 1}}) -> [1]
# Flat array
leaves([1, 2, 3]) -> [1, 2, 3]
# Flat object
leaves({a: 1, b: 2}) -> [1, 2]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'leaves({a: 1, b: [2, 3]})'
```

### pascal_keys

Recursively convert all keys to PascalCase

**Signature:** `any -> any`

**Examples:**

```text
# Camel to pascal
pascal_keys({userName: "alice"}) -> {UserName: "alice"}
# Snake to pascal
pascal_keys({user_name: "bob"}) -> {UserName: "bob"}
# Nested
pascal_keys({userInfo: {firstName: "x"}}) -> {UserInfo: {FirstName: "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pascal_keys({user_name: "alice"})'
```

### leaves_with_paths

Get all leaf values with their JSON pointer paths

**Signature:** `any -> array`

**Examples:**

```text
# Single leaf
leaves_with_paths({a: 1}) -> [{path: \"/a\", value: 1}]
# Nested path
leaves_with_paths({a: {b: 1}}) -> [{path: \"/a/b\", value: 1}]
# Array indices
leaves_with_paths([1, 2]) -> [{path: \"/0\", value: 1}, {path: \"/1\", value: 2}]
# Empty object
leaves_with_paths({}) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'leaves_with_paths({a: 1})'
```

### mask

Mask a string, showing only the last N characters

**Signature:** `string, number? -> string`

**Examples:**

```text
# Credit card default
mask("4111111111111111") -> "************1111"
# Phone with 3 visible
mask("555-123-4567", `3`) -> "*********567"
# Short string masked
mask("abc") -> "***"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'mask("4111111111111111")'
```

### omit

Remove specific keys from object

**Signature:** `object, array -> object`

**Examples:**

```text
# Remove one key
omit({a: 1, b: 2}, ['a']) -> {b: 2}
# Remove multiple keys
omit({a: 1, b: 2, c: 3}, ['a', 'c']) -> {b: 2}
# Key not present
omit({a: 1}, ['x']) -> {a: 1}
# Empty removal list
omit({a: 1, b: 2}, []) -> {a: 1, b: 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'omit({a: 1, b: 2}, ['a'])'
```

### paginate

Get a page of items from an array with metadata

**Signature:** `array, number, number -> object`

**Examples:**

```text
# First page
paginate([1,2,3,4,5], `2`, `1`) -> {items: [1,2], page: 1, page_size: 2, total_items: 5, total_pages: 3, has_next: true, has_prev: false}
# Last page
paginate([1,2,3,4,5], `2`, `3`) -> {items: [5], page: 3, page_size: 2, total_items: 5, total_pages: 3, has_next: false, has_prev: true}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'paginate([1,2,3,4,5], `2`, `1`)'
```

### paths

List all JSON pointer paths in value

**Signature:** `any -> array`

**Examples:**

```text
# Nested object
paths({a: {b: 1}}) -> [\"/a\", \"/a/b\"]
# Flat object
paths({a: 1, b: 2}) -> [\"/a\", \"/b\"]
# Array paths
paths([1, 2]) -> [\"/0\", \"/1\"]
# Empty object
paths({}) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'paths({a: {b: 1}})'
```

### paths_to

Find all dot-notation paths to a key anywhere in structure

**Signature:** `any, string -> array`

**Examples:**

```text
# Find paths to id
paths_to({a: {id: 1}, b: {id: 2}}, "id") -> ["a.id", "b.id"]
# Array paths
paths_to({users: [{id: 1}]}, "id") -> ["users.0.id"]
# Key not found
paths_to({a: 1}, "x") -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'paths_to({a: {id: 1}, b: {id: 2}}, "id")'
```

### pick

Select specific keys from object

**Signature:** `object, array -> object`

**Examples:**

```text
# Pick one key
pick({a: 1, b: 2}, ['a']) -> {a: 1}
# Pick multiple keys
pick({a: 1, b: 2, c: 3}, ['a', 'c']) -> {a: 1, c: 3}
# Key not present
pick({a: 1}, ['x']) -> {}
# Empty pick list
pick({a: 1, b: 2}, []) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pick({a: 1, b: 2}, ['a'])'
```

### pluck_deep

Find all values for a key anywhere in nested structure

**Signature:** `any, string -> array`

**Examples:**

```text
# Find all ids
pluck_deep({users: [{id: 1}, {id: 2}], meta: {id: 99}}, "id") -> [1, 2, 99]
# Nested values
pluck_deep({a: {b: {c: 1}}, d: {c: 2}}, "c") -> [1, 2]
# Key not found
pluck_deep({a: 1}, "x") -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pluck_deep({users: [{id: 1}, {id: 2}], meta: {id: 99}}, "id")'
```

### redact

Recursively replace values at specified keys with [REDACTED]

**Signature:** `any, array -> any`

**Examples:**

```text
# Redact password
redact({name: "alice", password: "secret"}, ["password"]) -> {name: "alice", password: "[REDACTED]"}
# Nested redact
redact({user: {name: "bob", ssn: "123"}}, ["ssn"]) -> {user: {name: "bob", ssn: "[REDACTED]"}}
# Array of objects
redact([{token: "abc"}], ["token"]) -> [{token: "[REDACTED]"}]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'redact({name: "alice", password: "secret"}, ["password"])'
```

### redact_keys

Recursively redact keys matching a regex pattern

**Signature:** `any, string -> any`

**Examples:**

```text
# Multiple patterns
redact_keys({password: "x", api_key: "y"}, "password|api_key") -> {password: "[REDACTED]", api_key: "[REDACTED]"}
# Wildcard pattern
redact_keys({secret_key: "a", secret_token: "b"}, "secret.*") -> {secret_key: "[REDACTED]", secret_token: "[REDACTED]"}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'redact_keys({password: "x", api_key: "y"}, "password|api_key")'
```

### remove_empty

Recursively remove nulls, empty strings, empty arrays, and empty objects

**Signature:** `any -> any`

**Examples:**

```text
# Remove all empty values
remove_empty({a: \"\", b: [], c: {}, d: null, e: \"hello\"}) -> {e: \"hello\"}
# Nested cleanup
remove_empty({a: {b: \"\", c: 1}}) -> {a: {c: 1}}
# Array cleanup
remove_empty([\"\", \"hello\", [], null]) -> [\"hello\"]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'remove_empty({a: \"\", b: [], c: {}, d: null, e: \"hello\"})'
```

### remove_empty_strings

Recursively remove empty string values

**Signature:** `any -> any`

**Examples:**

```text
# Remove empty strings
remove_empty_strings({name: \"alice\", bio: \"\"}) -> {name: \"alice\"}
# From arrays
remove_empty_strings([\"hello\", \"\", \"world\"]) -> [\"hello\", \"world\"]
# Nested
remove_empty_strings({a: {b: \"\", c: \"x\"}}) -> {a: {c: \"x\"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'remove_empty_strings({name: \"alice\", bio: \"\"})'
```

### remove_nulls

Recursively remove null values

**Signature:** `any -> any`

**Examples:**

```text
# Remove nulls from object
remove_nulls({a: 1, b: null, c: 2}) -> {a: 1, c: 2}
# Nested nulls
remove_nulls({a: {b: null, c: 1}}) -> {a: {c: 1}}
# Remove from array
remove_nulls([1, null, 2]) -> [1, 2]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'remove_nulls({a: 1, b: null, c: 2})'
```

### rename_keys

Rename object keys

**Signature:** `object, object -> object`

**Examples:**

```text
# Rename one key
rename_keys({a: 1}, {a: 'b'}) -> {b: 1}
# Rename multiple
rename_keys({a: 1, b: 2}, {a: 'x', b: 'y'}) -> {x: 1, y: 2}
# No matching key
rename_keys({a: 1}, {x: 'y'}) -> {a: 1}
# Real-world rename
rename_keys({old: 'value'}, {old: 'new'}) -> {new: 'value'}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'rename_keys({a: 1}, {a: 'b'})'
```

### set_path

Set value at JSON pointer path (immutable)

**Signature:** `any, string, any -> any`

**Examples:**

```text
# Add new key
set_path({a: 1}, '/b', `2`) -> {a: 1, b: 2}
# Update existing
set_path({a: 1}, '/a', `2`) -> {a: 2}
# Set nested path
set_path({a: {}}, '/a/b', `1`) -> {a: {b: 1}}
# Update array element
set_path([1, 2], '/1', `5`) -> [1, 5]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'set_path({a: 1}, `"/b"`, `2`)'
```

### shouty_kebab_keys

Recursively convert all keys to SHOUTY-KEBAB-CASE

**Signature:** `any -> any`

**Examples:**

```text
# Camel to shouty kebab
shouty_kebab_keys({userName: "alice"}) -> {"USER-NAME": "alice"}
# Snake to shouty kebab
shouty_kebab_keys({user_name: "bob"}) -> {"USER-NAME": "bob"}
# Nested
shouty_kebab_keys({userInfo: {firstName: "x"}}) -> {"USER-INFO": {"FIRST-NAME": "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shouty_kebab_keys({user_name: "alice"})'
```

### shouty_snake_keys

Recursively convert all keys to SHOUTY_SNAKE_CASE

**Signature:** `any -> any`

**Examples:**

```text
# Camel to shouty snake
shouty_snake_keys({userName: "alice"}) -> {USER_NAME: "alice"}
# Kebab to shouty snake
shouty_snake_keys({"user-name": "bob"}) -> {USER_NAME: "bob"}
# Nested
shouty_snake_keys({userInfo: {firstName: "x"}}) -> {USER_INFO: {FIRST_NAME: "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shouty_snake_keys({user_name: "alice"})'
```

### snake_keys

Recursively convert all keys to snake_case

**Signature:** `any -> any`

**Examples:**

```text
# Camel to snake
snake_keys({userName: "alice"}) -> {user_name: "alice"}
# Kebab to snake
snake_keys({"user-name": "bob"}) -> {user_name: "bob"}
# Nested
snake_keys({userInfo: {firstName: "x"}}) -> {user_info: {first_name: "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'snake_keys({userName: "alice"})'
```

### train_keys

Recursively convert all keys to Train-Case

**Signature:** `any -> any`

**Examples:**

```text
# Camel to train
train_keys({userName: "alice"}) -> {"User-Name": "alice"}
# Snake to train
train_keys({user_name: "bob"}) -> {"User-Name": "bob"}
# Nested
train_keys({userInfo: {firstName: "x"}}) -> {"User-Info": {"First-Name": "x"}}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'train_keys({user_name: "alice"})'
```

### structural_diff

Compare two values and return their structural differences

**Signature:** `any, any -> object`

**Examples:**

```text
# Changed value
structural_diff({a: 1}, {a: 2}) -> {changed: [{path: "a", from: 1, to: 2}], added: [], removed: []}
# Added key
structural_diff({a: 1}, {a: 1, b: 2}) -> {changed: [], added: [{path: "b", value: 2}], removed: []}
# Removed key
structural_diff({a: 1, b: 2}, {a: 1}) -> {changed: [], added: [], removed: [{path: "b", value: 2}]}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'structural_diff({a: 1}, {a: 2})'
```

### template

Expand a template string with values from an object using {{key}} syntax

**Signature:** `object, string -> string`

**Examples:**

```text
# Simple substitution
template({name: "Alice"}, `"Hello, {{name}}!"`) -> "Hello, Alice!"
# Nested access
template({user: {name: "Bob"}}, `"Hi {{user.name}}"`) -> "Hi Bob"
# Default value
template({}, `"Hello {{name|World}}"`) -> "Hello World"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'template({name: "Alice"}, `"Hello, {{name}}!"`)'
```

### template_strict

Expand a template string, returning null if any variable is missing

**Signature:** `object, string -> string | null`

**Examples:**

```text
# All vars present
template_strict({name: "Alice"}, `"Hello, {{name}}!"`) -> "Hello, Alice!"
# Missing variable
template_strict({}, `"Hello, {{name}}!"`) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'template_strict({name: "Alice"}, `"Hello, {{name}}!"`)'
```

### truncate_to_size

Truncate an array to fit within approximately the specified byte size

**Signature:** `array, number -> array`

**Examples:**

```text
# All fit
truncate_to_size([{a:1}, {b:2}, {c:3}], `100`) -> [{a:1}, {b:2}, {c:3}]
# Truncated
truncate_to_size([{a:1}, {b:2}, {c:3}], `15`) -> [{a:1}, {b:2}]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'truncate_to_size([{a:1}, {b:2}, {c:3}], `100`)'
```

### type_consistency

Check if array elements have consistent types

**Signature:** `array -> object`

**Examples:**

```text
# Consistent numbers
type_consistency([1, 2, 3]).consistent -> true
# Mixed types
type_consistency([1, 'two', 3]).consistent -> false
# Object field mismatch
type_consistency([{a: 1}, {a: 'x'}]).inconsistencies -> [{field: 'a', ...}]
# Empty array is consistent
type_consistency([]).consistent -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'type_consistency([1, 2, 3]).consistent'
```

### unflatten

Alias for unflatten_keys - restore nested object from dot notation keys

**Signature:** `object -> object`

**Examples:**

```text
# Simple nested
unflatten({\"a.b\": 1}) -> {a: {b: 1}}
# Deep nested
unflatten({\"a.b.c\": 1}) -> {a: {b: {c: 1}}}
# Flat keys
unflatten({\"a\": 1, \"b\": 2}) -> {a: 1, b: 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'unflatten({\"a.b\": 1})'
```

### unflatten_keys

Restore nested object from dot notation keys

**Signature:** `object -> object`

**Examples:**

```text
# Simple nested
unflatten_keys({\"a.b\": 1}) -> {a: {b: 1}}
# Deep nested
unflatten_keys({\"a.b.c\": 1}) -> {a: {b: {c: 1}}}
# Flat keys
unflatten_keys({\"a\": 1, \"b\": 2}) -> {a: 1, b: 2}
# Mixed nesting
unflatten_keys({\"x.y\": 1, \"z\": 2}) -> {x: {y: 1}, z: 2}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'unflatten_keys({\"a.b\": 1})'
```

### with_entries

Transform object entries using an expression (jq parity)

**Signature:** `object, string -> object`

**Examples:**

```text
# Transform keys and values
with_entries({a: 1, b: 2}, '[upper(@[0]), multiply(@[1], `2`)]') -> {A: 2, B: 4}
# Transform values only
with_entries({a: 1, b: 2}, '[@[0], add(@[1], `10`)]') -> {a: 11, b: 12}
# Filter entries
with_entries({a: 1, b: 2, c: 3}, 'if(@[1] > `1`, @, `null`)') -> {b: 2, c: 3}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'with_entries({a: 1, b: 2}, `"[upper(@[0]), multiply(@[1], `2`)]"`)'
```

