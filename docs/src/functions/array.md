# Array Functions

Functions for working with arrays: chunking, filtering, transforming, and combining array data.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`bsearch`](#bsearch) | `array, any -> number` | Binary search in a sorted array, returns index or negative insertion point (jq parity) |
| [`cartesian`](#cartesian) | `array, array? -> array` | Compute cartesian product of arrays (jq parity for N-way product) |
| [`chunk`](#chunk) | `array, number -> array` | Split array into chunks of size n |
| [`compact`](#compact) | `array -> array` | Remove null values from array |
| [`difference`](#difference) | `array, array -> array` | Elements in first array not in second |
| [`drop`](#drop) | `array, number -> array` | Drop first n elements |
| [`find_index`](#find-index) | `array, any -> number \| null` | Find index of value in array |
| [`first`](#first) | `array -> any` | Get first element of array |
| [`flatten`](#flatten) | `array -> array` | Flatten array one level deep |
| [`flatten_deep`](#flatten-deep) | `array -> array` | Recursively flatten nested arrays |
| [`frequencies`](#frequencies) | `array -> object` | Count occurrences of each value |
| [`group_by`](#group-by) | `array, string -> object` | Group array elements by key |
| [`includes`](#includes) | `array, any -> boolean` | Check if array contains value |
| [`index_at`](#index-at) | `array, number -> any` | Get element at index (supports negative) |
| [`index_by`](#index-by) | `array, string -> object` | Create lookup map from array using key field (last value wins for duplicates) |
| [`indices_array`](#indices-array) | `array, any -> array` | Find all indices where a value appears in an array (jq parity) |
| [`inside_array`](#inside-array) | `array, array -> boolean` | Check if all elements of first array are contained in second array (inverse of contains, jq parity) |
| [`intersection`](#intersection) | `array, array -> array` | Elements common to both arrays |
| [`last`](#last) | `array -> any` | Get last element of array |
| [`pairwise`](#pairwise) | `array -> array` | Return adjacent pairs from array |
| [`range`](#range) | `number, number -> array` | Generate array of numbers |
| [`sliding_window`](#sliding-window) | `array, number -> array` | Create overlapping windows of size n (alias for window) |
| [`take`](#take) | `array, number -> array` | Take first n elements |
| [`transpose`](#transpose) | `array -> array` | Transpose a 2D array (swap rows and columns) |
| [`union`](#union) | `array, array -> array` | Unique elements from both arrays |
| [`unique`](#unique) | `array -> array` | Remove duplicate values |
| [`zip`](#zip) | `array, array -> array` | Zip two arrays together |

## Functions

### bsearch

Binary search in a sorted array, returns index or negative insertion point (jq parity)

**Signature:** `array, any -> number`

**Examples:**

```text
# Found at index 2
bsearch([1, 3, 5, 7, 9], `5`) -> 2
# Not found, would insert at index 2
bsearch([1, 3, 5, 7, 9], `4`) -> -3
# Not found, would insert at index 0
bsearch([1, 3, 5, 7, 9], `0`) -> -1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bsearch([1, 3, 5, 7, 9], `5`)'
```

### cartesian

Compute cartesian product of arrays (jq parity for N-way product)

**Signature:** `array, array? -> array`

**Examples:**

```text
# Two-array product
cartesian([1, 2], [3, 4]) -> [[1, 3], [1, 4], [2, 3], [2, 4]]
# N-way product (jq style)
cartesian([[1, 2], [3, 4]]) -> [[1, 3], [1, 4], [2, 3], [2, 4]]
# Three-way product
cartesian([['a', 'b'], [1, 2], ['x', 'y']]) -> [['a', 1, 'x'], ['a', 1, 'y'], ...]
# Single array
cartesian([[1, 2]]) -> [[1], [2]]
# Empty array produces empty result
cartesian([[], [1, 2]]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cartesian([1, 2], [3, 4])'
```

### chunk

Split array into chunks of size n

**Signature:** `array, number -> array`

**Examples:**

```text
# Basic chunking
chunk([1, 2, 3, 4], `2`) -> [[1, 2], [3, 4]]
# Uneven chunks
chunk([1, 2, 3, 4, 5], `2`) -> [[1, 2], [3, 4], [5]]
# Empty array
chunk([], `2`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'chunk([1, 2, 3, 4], `2`)'
```

### compact

Remove null values from array

**Signature:** `array -> array`

**Examples:**

```text
# Remove nulls
compact([1, null, 2, null]) -> [1, 2]
# All nulls
compact([null, null]) -> []
# No nulls
compact([1, 2, 3]) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'compact([1, null, 2, null])'
```

### difference

Elements in first array not in second

**Signature:** `array, array -> array`

**Examples:**

```text
# Remove matching elements
difference([1, 2, 3], [2]) -> [1, 3]
# No overlap
difference([1, 2, 3], [4, 5]) -> [1, 2, 3]
# All removed
difference([1, 2], [1, 2]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'difference([1, 2, 3], [2])'
```

### drop

Drop first n elements

**Signature:** `array, number -> array`

**Examples:**

```text
# Drop first 2
drop([1, 2, 3, 4], `2`) -> [3, 4]
# Drop none
drop([1, 2, 3], `0`) -> [1, 2, 3]
# Drop more than length
drop([1, 2], `5`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'drop([1, 2, 3, 4], `2`)'
```

### find_index

Find index of value in array

**Signature:** `array, any -> number | null`

**Examples:**

```text
# Find existing value
find_index([1, 2, 3], `2`) -> 1
# Find string
find_index(['a', 'b', 'c'], 'b') -> 1
# Value not found
find_index([1, 2, 3], `99`) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'find_index([1, 2, 3], `2`)'
```

### first

Get first element of array

**Signature:** `array -> any`

**Examples:**

```text
# Get first number
first([1, 2, 3]) -> 1
# Get first string
first(['a', 'b']) -> 'a'
# Empty array returns null
first([]) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'first([1, 2, 3])'
```

### flatten

Flatten array one level deep

**Signature:** `array -> array`

**Examples:**

```text
# Flatten nested arrays
flatten([[1, 2], [3]]) -> [1, 2, 3]
# Only one level deep
flatten([[1, [2]], [3]]) -> [1, [2], 3]
# Already flat
flatten([1, 2, 3]) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flatten([[1, 2], [3]])'
```

### flatten_deep

Recursively flatten nested arrays

**Signature:** `array -> array`

**Examples:**

```text
# Deeply nested
flatten_deep([[1, [2]], [3]]) -> [1, 2, 3]
# Multiple levels
flatten_deep([[[1]], [[2]], [[3]]]) -> [1, 2, 3]
# Already flat
flatten_deep([1, 2, 3]) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flatten_deep([[1, [2]], [3]])'
```

### frequencies

Count occurrences of each value

**Signature:** `array -> object`

**Examples:**

```text
# Count strings
frequencies(['a', 'b', 'a']) -> {a: 2, b: 1}
# Count numbers
frequencies([1, 2, 1, 1]) -> {1: 3, 2: 1}
# Empty array
frequencies([]) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'frequencies(['a', `"b"`, 'a'])'
```

### group_by

Group array elements by key

**Signature:** `array, string -> object`

**Examples:**

```text
# Group by field
group_by([{t: 'a'}, {t: 'b'}, {t: 'a'}], 't') -> {a: [{t:'a'}, {t:'a'}], b: [{t:'b'}]}
# Group users by role
group_by(users, 'role') -> {admin: [...], user: [...]}
# Empty array returns empty object
group_by([], 'key') -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'group_by([{t: 'a'}, {t: 'b'}, {t: 'a'}], `"t"`)'
```

### includes

Check if array contains value

**Signature:** `array, any -> boolean`

**Examples:**

```text
# Value found
includes([1, 2, 3], `2`) -> true
# Value not found
includes([1, 2, 3], `99`) -> false
# String value
includes(['a', 'b'], 'a') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'includes([1, 2, 3], `2`)'
```

### index_at

Get element at index (supports negative)

**Signature:** `array, number -> any`

**Examples:**

```text
# First element
index_at([1, 2, 3], `0`) -> 1
# Last element (negative index)
index_at([1, 2, 3], `-1`) -> 3
# Out of bounds
index_at([1, 2, 3], `99`) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'index_at([1, 2, 3], `0`)'
```

### index_by

Create lookup map from array using key field (last value wins for duplicates)

**Signature:** `array, string -> object`

**Examples:**

```text
# Index by id field
index_by([{id: 1, name: "alice"}, {id: 2, name: "bob"}], "id") -> {"1": {id: 1, name: "alice"}, "2": {id: 2, name: "bob"}}
# Index by string key
index_by([{code: "US", name: "USA"}], "code") -> {"US": {code: "US", name: "USA"}}
# Last value wins for duplicates
index_by([{t: "a", v: 1}, {t: "a", v: 2}], "t") -> {"a": {t: "a", v: 2}}
# Empty array returns empty object
index_by([], "id") -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'index_by([{id: 1, name: "alice"}, {id: 2, name: "bob"}], "id")'
```

### indices_array

Find all indices where a value appears in an array (jq parity)

**Signature:** `array, any -> array`

**Examples:**

```text
# Find all occurrences
indices_array([1, 2, 3, 2, 4, 2], `2`) -> [1, 3, 5]
# String values
indices_array(['a', 'b', 'a', 'c'], `'a'`) -> [0, 2]
# Not found returns empty
indices_array([1, 2, 3], `5`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'indices_array([1, 2, 3, 2, 4, 2], `2`)'
```

### inside_array

Check if all elements of first array are contained in second array (inverse of contains, jq parity)

**Signature:** `array, array -> boolean`

**Examples:**

```text
# Subset check
inside_array([1, 2], [1, 2, 3, 4]) -> true
# Not a subset
inside_array([1, 5], [1, 2, 3, 4]) -> false
# Empty is subset of any
inside_array([], [1, 2, 3]) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'inside_array([1, 2], [1, 2, 3, 4])'
```

### intersection

Elements common to both arrays

**Signature:** `array, array -> array`

**Examples:**

```text
# Common elements
intersection([1, 2], [2, 3]) -> [2]
# No overlap
intersection([1, 2], [3, 4]) -> []
# Identical arrays
intersection([1, 2, 3], [1, 2, 3]) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'intersection([1, 2], [2, 3])'
```

### last

Get last element of array

**Signature:** `array -> any`

**Examples:**

```text
# Get last number
last([1, 2, 3]) -> 3
# Get last string
last(['a', 'b']) -> 'b'
# Empty array returns null
last([]) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'last([1, 2, 3])'
```

### pairwise

Return adjacent pairs from array

**Signature:** `array -> array`

**Examples:**

```text
# Adjacent pairs
pairwise([1, 2, 3]) -> [[1, 2], [2, 3]]
# Single pair
pairwise([1, 2]) -> [[1, 2]]
# Too few elements
pairwise([1]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pairwise([1, 2, 3])'
```

### range

Generate array of numbers

**Signature:** `number, number -> array`

**Examples:**

```text
# Range 1 to 4
range(`1`, `5`) -> [1, 2, 3, 4]
# Range from zero
range(`0`, `3`) -> [0, 1, 2]
# Empty range
range(`5`, `5`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'range(`1`, `5`)'
```

### sliding_window

Create overlapping windows of size n (alias for window)

**Signature:** `array, number -> array`

**Aliases:** `window`

**Examples:**

```text
# Size 2 windows
sliding_window([1, 2, 3, 4], `2`) -> [[1, 2], [2, 3], [3, 4]]
# Size 3 windows
sliding_window([1, 2, 3, 4], `3`) -> [[1, 2, 3], [2, 3, 4]]
# Window larger than array
sliding_window([1, 2], `3`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sliding_window([1, 2, 3, 4], `2`)'
```

### take

Take first n elements

**Signature:** `array, number -> array`

**Examples:**

```text
# Take first 2
take([1, 2, 3, 4], `2`) -> [1, 2]
# Take none
take([1, 2, 3], `0`) -> []
# Take more than length
take([1, 2], `5`) -> [1, 2]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'take([1, 2, 3, 4], `2`)'
```

### transpose

Transpose a 2D array (swap rows and columns)

**Signature:** `array -> array`

**Examples:**

```text
# Swap rows/columns
transpose([[1, 2], [3, 4]]) -> [[1, 3], [2, 4]]
# 2x3 to 3x2
transpose([[1, 2, 3], [4, 5, 6]]) -> [[1, 4], [2, 5], [3, 6]]
# Empty array
transpose([]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'transpose([[1, 2], [3, 4]])'
```

### union

Unique elements from both arrays

**Signature:** `array, array -> array`

**Examples:**

```text
# Combine with dedup
union([1, 2], [2, 3]) -> [1, 2, 3]
# No overlap
union([1, 2], [3, 4]) -> [1, 2, 3, 4]
# Empty first array
union([], [1, 2]) -> [1, 2]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'union([1, 2], [2, 3])'
```

### unique

Remove duplicate values

**Signature:** `array -> array`

**Examples:**

```text
# Basic deduplication
unique([1, 2, 1, 3]) -> [1, 2, 3]
# String values
unique(['a', 'b', 'a']) -> ['a', 'b']
# Empty array
unique([]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'unique([1, 2, 1, 3])'
```

### zip

Zip two arrays together

**Signature:** `array, array -> array`

**JEP:** JEP-013

**Examples:**

```text
# Basic zip
zip([1, 2], ['a', 'b']) -> [[1, 'a'], [2, 'b']]
# Unequal lengths (truncates)
zip([1, 2, 3], ['a', 'b']) -> [[1, 'a'], [2, 'b']]
# Empty arrays
zip([], []) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'zip([1, 2], ['a', 'b'])'
```

