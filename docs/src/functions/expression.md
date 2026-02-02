# Expression Functions

Higher-order functions that work with JMESPath expressions as arguments.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`all_expr`](#all-expr) | `array, expression -> boolean` | Return true if every element satisfies the expression (short-circuits on false) |
| [`any_expr`](#any-expr) | `array, expression -> boolean` | Return true if any element satisfies the expression (short-circuits) |
| [`apply`](#apply) | `object\|string, ...any -> any` | Apply a partial function or invoke a function by name with arguments |
| [`count_by`](#count-by) | `string, array -> object` | Count occurrences grouped by expression result |
| [`count_expr`](#count-expr) | `array, expression -> number` | Count how many elements satisfy the expression |
| [`drop_while`](#drop-while) | `string, array -> array` | Drop elements from array while expression is truthy |
| [`every`](#every) | `string, array -> boolean` | Check if all elements match (alias for all_expr) |
| [`filter_expr`](#filter-expr) | `array, expression -> array` | Keep elements where JMESPath expression evaluates to truthy value |
| [`find_expr`](#find-expr) | `array, expression -> any` | Return first element where expression is truthy, or null if none match |
| [`find_index_expr`](#find-index-expr) | `array, expression -> number \| null` | Return zero-based index of first matching element, or -1 if none match |
| [`flat_map_expr`](#flat-map-expr) | `array, expression -> array` | Apply expression to each element and flatten all results into one array |
| [`group_by_expr`](#group-by-expr) | `array, expression -> object` | Group elements into object keyed by expression result |
| [`map_expr`](#map-expr) | `array, expression -> array` | Apply a JMESPath expression to each element, returning transformed array |
| [`map_keys`](#map-keys) | `string, object -> object` | Transform object keys using expression |
| [`map_values`](#map-values) | `string, object -> object` | Transform object values using expression |
| [`max_by_expr`](#max-by-expr) | `array, expression -> any` | Return element with largest expression result, or null for empty array |
| [`min_by_expr`](#min-by-expr) | `array, expression -> any` | Return element with smallest expression result, or null for empty array |
| [`order_by`](#order-by) | `array, array[[string, string]] -> array` | Sort array by multiple fields with ascending/descending control |
| [`partial`](#partial) | `string, ...any -> object` | Create a partial function with some arguments pre-filled |
| [`partition_expr`](#partition-expr) | `array, expression -> array` | Split array into [matches, non-matches] based on expression |
| [`recurse`](#recurse) | `any -> array` | Collect all nested values recursively (jq parity) |
| [`recurse_with`](#recurse-with) | `any, expression -> array` | Recursive descent with expression filter (jq parity) |
| [`reduce_expr`](#reduce-expr) | `string, array, any -> any` | Reduce array to single value using accumulator expression |
| [`reject`](#reject) | `string, array -> array` | Keep elements where expression is falsy (inverse of filter_expr) |
| [`scan_expr`](#scan-expr) | `string, array, any -> array` | Like reduce but returns array of intermediate accumulator values |
| [`some`](#some) | `string, array -> boolean` | Check if any element matches (alias for any_expr) |
| [`sort_by_expr`](#sort-by-expr) | `array, expression -> array` | Sort array by expression result in ascending order |
| [`take_while`](#take-while) | `string, array -> array` | Take elements from array while expression is truthy |
| [`unique_by_expr`](#unique-by-expr) | `array, expression -> array` | Remove duplicates by expression result, keeping first occurrence |
| [`until_expr`](#until-expr) | `any, expression, expression -> array` | Loop until condition becomes true, collecting intermediate values (jq parity) |
| [`walk`](#walk) | `string, any -> any` | Recursively apply expression to all components of a value (bottom-up) |
| [`while_expr`](#while-expr) | `any, expression, expression -> array` | Loop while condition is true, collecting intermediate values (jq parity) |
| [`zip_with`](#zip-with) | `string, array, array -> array` | Zip two arrays with a custom combiner expression |

## Functions

### all_expr

Return true if every element satisfies the expression (short-circuits on false)

**Signature:** `array, expression -> boolean`

**Aliases:** `every`

**Examples:**

```text
# All positive
all_expr([1, 2, 3], &@ > `0`) -> true
# Not all > 2
all_expr([1, 2, 3], &@ > `2`) -> false
# Empty array is true
all_expr([], &@ > `0`) -> true
# Check all adults
all_expr(users, &age >= `18`) -> true/false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'all_expr([1, 2, 3], &@ > `0`)'
```

### any_expr

Return true if any element satisfies the expression (short-circuits)

**Signature:** `array, expression -> boolean`

**Aliases:** `some`

**Examples:**

```text
# At least one > 2
any_expr([1, 2, 3], &@ > `2`) -> true
# None > 5
any_expr([1, 2, 3], &@ > `5`) -> false
# Empty array is false
any_expr([], &@ > `0`) -> false
# Any active user
any_expr(users, &status == 'active') -> true/false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'any_expr([1, 2, 3], &@ > `2`)'
```

### apply

Apply a partial function or invoke a function by name with arguments

**Signature:** `object|string, ...any -> any`

**Examples:**

```text
# Apply partial function
apply(partial('join', `\"-\"`), `[\"a\", \"b\"]`) -> 'a-b' 
# Call function by name
apply('length', 'hello') -> 5
# Call with multiple args
apply('add', `1`, `2`) -> 3
# Partial with contains
apply(partial('contains', 'hello'), 'ell') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'apply(partial(`"join"`, `\"-\"`), `[\"a\", \"b\"]`)'
```

### count_by

Count occurrences grouped by expression result

**Signature:** `string, array -> object`

**Examples:**

```text
# Count by field
count_by('type', [{type: 'a'}, {type: 'b'}, {type: 'a'}]) -> {a: 2, b: 1}
# Count orders by status
count_by('status', orders) -> {pending: 5, shipped: 3}
# Count by condition
count_by('@ > `50`', [30, 60, 70, 40]) -> {true: 2, false: 2}
# Empty array
count_by('category', []) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'count_by(`"type"`, [{type: 'a'}, {type: 'b'}, {type: 'a'}])'
```

### count_expr

Count how many elements satisfy the expression

**Signature:** `array, expression -> number`

**Examples:**

```text
# Count > 1
count_expr([1, 2, 3], &@ > `1`) -> 2
# None match
count_expr([1, 2, 3], &@ > `5`) -> 0
# Count active users
count_expr(users, &active == `true`) -> 5
# Empty array
count_expr([], &@ > `0`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'count_expr([1, 2, 3], &@ > `1`)'
```

### drop_while

Drop elements from array while expression is truthy

**Signature:** `string, array -> array`

**Examples:**

```text
# Drop while < 4
drop_while('@ < `4`', [1, 2, 3, 5, 1]) -> [5, 1]
# None dropped
drop_while('@ < `0`', [1, 2, 3]) -> [1, 2, 3]
# All dropped
drop_while('@ < `10`', [1, 2, 3]) -> []
# Drop short strings
drop_while('length(@) < `3`', ['a', 'ab', 'abc', 'x']) -> ['abc', 'x']
```

**CLI Usage:**

```bash
echo '{}' | jpx 'drop_while(`"@ < `4`"`, [1, 2, 3, 5, 1])'
```

### every

Check if all elements match (alias for all_expr)

**Signature:** `string, array -> boolean`

**Examples:**

```text
# All positive
every('@ > `0`', [1, 2, 3]) -> true
# Not all > 2
every('@ > `2`', [1, 2, 3]) -> false
# All non-empty
every('length(@) > `0`', ['a', 'b']) -> true
# Empty is vacuously true
every('@ > `0`', []) -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'every(`"@ > `0`"`, [1, 2, 3])'
```

### filter_expr

Keep elements where JMESPath expression evaluates to truthy value

**Signature:** `array, expression -> array`

**Aliases:** `filter`

**Examples:**

```text
# Filter numbers
filter_expr([1, 2, 3], &@ > `1`) -> [2, 3]
# Filter objects by field
filter_expr(users, &age >= `18`) -> [adult users]
# Filter empty strings
filter_expr(['a', '', 'b'], &length(@) > `0`) -> ['a', 'b']
# Empty array returns empty
filter_expr([], &@ > `0`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'filter_expr([1, 2, 3], &@ > `1`)'
```

### find_expr

Return first element where expression is truthy, or null if none match

**Signature:** `array, expression -> any`

**Examples:**

```text
# First > 1
find_expr([1, 2, 3], &@ > `1`) -> 2
# None found
find_expr([1, 2, 3], &@ > `5`) -> null
# Find user
find_expr(users, &name == 'alice') -> {name: 'alice', ...}
# Empty array
find_expr([], &@ > `0`) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'find_expr([1, 2, 3], &@ > `1`)'
```

### find_index_expr

Return zero-based index of first matching element, or -1 if none match

**Signature:** `array, expression -> number | null`

**Examples:**

```text
# Index of first > 1
find_index_expr([1, 2, 3], &@ > `1`) -> 1
# Not found
find_index_expr([1, 2, 3], &@ > `5`) -> -1
# Exact match
find_index_expr([5, 10, 15], &@ == `10`) -> 1
# Empty array
find_index_expr([], &@ > `0`) -> -1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'find_index_expr([1, 2, 3], &@ > `1`)'
```

### flat_map_expr

Apply expression to each element and flatten all results into one array

**Signature:** `array, expression -> array`

**Examples:**

```text
# Flatten nested arrays
flat_map_expr([[1], [2]], &@) -> [1, 2]
# Duplicate and transform
flat_map_expr([1, 2], &[@, @ * `2`]) -> [1, 2, 2, 4]
# Flatten object arrays
flat_map_expr(users, &tags) -> all tags flattened
# Empty array
flat_map_expr([], &@) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'flat_map_expr([[1], [2]], &@)'
```

### group_by_expr

Group elements into object keyed by expression result

**Signature:** `array, expression -> object`

**Examples:**

```text
# Group by field
group_by_expr([{t: 'a'}, {t: 'b'}], &t) -> {a: [...], b: [...]}
# Group by condition
group_by_expr([1, 2, 3, 4], &@ > `2`) -> {true: [3, 4], false: [1, 2]}
# Group users
group_by_expr(users, &department) -> {eng: [...], sales: [...]}
# Empty array
group_by_expr([], &@) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'group_by_expr([{t: 'a'}, {t: 'b'}], &t)'
```

### map_expr

Apply a JMESPath expression to each element, returning transformed array

**Signature:** `array, expression -> array`

**Examples:**

```text
# Double each number
map_expr([1, 2], &@ * `2`) -> [2, 4]
# Extract field from objects
map_expr(users, &name) -> ['alice', 'bob']
# Transform strings
map_expr(['hello', 'world'], &upper(@)) -> ['HELLO', 'WORLD']
# Empty array returns empty
map_expr([], &@ * `2`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'map_expr([1, 2], &@ * `2`)'
```

### map_keys

Transform object keys using expression

**Signature:** `string, object -> object`

**Examples:**

```text
# Uppercase keys
map_keys('upper(@)', {a: 1}) -> {A: 1}
# Lowercase keys
map_keys('lower(@)', {A: 1, B: 2}) -> {a: 1, b: 2}
# Add suffix
map_keys('concat(@, `"_suffix"`)', {x: 1}) -> {x_suffix: 1}
# Empty object
map_keys('upper(@)', {}) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'map_keys(`"upper(@)"`, {a: 1})'
```

### map_values

Transform object values using expression

**Signature:** `string, object -> object`

**Examples:**

```text
# Double values
map_values('@ * `2`', {a: 1, b: 2}) -> {a: 2, b: 4}
# Uppercase strings
map_values('upper(@)', {a: 'x', b: 'y'}) -> {a: 'X', b: 'Y'}
# Add to values
map_values('@ + `10`', {x: 5}) -> {x: 15}
# Empty object
map_values('@ * `2`', {}) -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'map_values(`"@ * `2`"`, {a: 1, b: 2})'
```

### max_by_expr

Return element with largest expression result, or null for empty array

**Signature:** `array, expression -> any`

**Examples:**

```text
# Max by field
max_by_expr([{a: 2}, {a: 1}], &a) -> {a: 2}
# Longest string
max_by_expr(['a', 'abc', 'ab'], &length(@)) -> 'abc'
# Max price product
max_by_expr(products, &price) -> most expensive
# Empty array
max_by_expr([], &a) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'max_by_expr([{a: 2}, {a: 1}], &a)'
```

### min_by_expr

Return element with smallest expression result, or null for empty array

**Signature:** `array, expression -> any`

**Examples:**

```text
# Min by field
min_by_expr([{a: 2}, {a: 1}], &a) -> {a: 1}
# Shortest string
min_by_expr(['a', 'abc', 'ab'], &length(@)) -> 'a'
# Min price product
min_by_expr(products, &price) -> cheapest
# Empty array
min_by_expr([], &a) -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'min_by_expr([{a: 2}, {a: 1}], &a)'
```

### order_by

Sort array by multiple fields with ascending/descending control

**Signature:** `array, array[[string, string]] -> array`

**Examples:**

```text
# Multi-field sort
order_by(items, [['name', 'asc'], ['price', 'desc']]) -> sorted
# Single field ascending
order_by(users, [['age', 'asc']]) -> youngest first
# Descending sort
order_by(products, [['price', 'desc']]) -> most expensive first
# Empty array
order_by([], [['a', 'asc']]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'order_by(items, [['name', 'asc'], ['price', 'desc']])'
```

### partial

Create a partial function with some arguments pre-filled

**Signature:** `string, ...any -> object`

**Examples:**

```text
# Partial contains
partial('contains', `\"hello\"`) -> {__partial__: true, ...}
# Partial addition
partial('add', `10`) -> partial add 10
# Partial join
partial('join', '-') -> partial join with dash
# Partial multiply
partial('multiply', `2`) -> partial multiply by 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'partial(`"contains"`, `\"hello\"`)'
```

### partition_expr

Split array into [matches, non-matches] based on expression

**Signature:** `array, expression -> array`

**Examples:**

```text
# Split by condition
partition_expr([1, 2, 3], &@ > `1`) -> [[2, 3], [1]]
# Even vs odd
partition_expr([1, 2, 3, 4], &@ % `2` == `0`) -> [[2, 4], [1, 3]]
# Partition users
partition_expr(users, &active) -> [active, inactive]
# Empty array
partition_expr([], &@ > `0`) -> [[], []]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'partition_expr([1, 2, 3], &@ > `1`)'
```

### recurse

Collect all nested values recursively (jq parity)

**Signature:** `any -> array`

**Examples:**

```text
# Nested object
recurse({a: {b: 1}}) -> [{a: {b: 1}}, {b: 1}, 1]
# Nested array
recurse([1, [2, 3]]) -> [[1, [2, 3]], 1, [2, 3], 2, 3]
# Scalar value
recurse(`5`) -> [5]
# All descendants
recurse({a: 1, b: {c: 2}}) -> all nested values
```

**CLI Usage:**

```bash
echo '{}' | jpx 'recurse({a: {b: 1}})'
```

### recurse_with

Recursive descent with expression filter (jq parity)

**Signature:** `any, expression -> array`

**Examples:**

```text
# Follow 'a' key recursively
recurse_with({a: {a: 1}}, &a) -> [{a: 1}, 1]
# Follow index recursively
recurse_with([1, [2, [3]]], &[1]) -> [[2, [3]], [3]]
# Tree traversal
recurse_with({children: [{children: []}]}, &children) -> nested children
```

**CLI Usage:**

```bash
echo '{}' | jpx 'recurse_with({a: {a: 1}}, &a)'
```

### reduce_expr

Reduce array to single value using accumulator expression

**Signature:** `string, array, any -> any`

**Aliases:** `fold`

**Examples:**

```text
# Sum numbers
reduce_expr('add(accumulator, current)', [1, 2, 3], `0`) -> 6
# Product
reduce_expr('multiply(accumulator, current)', [2, 3, 4], `1`) -> 24
# Find max
reduce_expr('max(accumulator, current)', [3, 1, 4], `0`) -> 4
# Concat strings
reduce_expr('concat(accumulator, current)', ['a', 'b'], '') -> 'ab'
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reduce_expr(`"add(accumulator, current)"`, [1, 2, 3], `0`)'
```

### reject

Keep elements where expression is falsy (inverse of filter_expr)

**Signature:** `string, array -> array`

**Examples:**

```text
# Reject > 2
reject('@ > `2`', [1, 2, 3, 4]) -> [1, 2]
# Reject negatives
reject('@ < `0`', [1, -1, 2, -2]) -> [1, 2]
# Reject active users
reject('active', users) -> inactive users
# Empty array
reject('@ > `0`', []) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reject(`"@ > `2`"`, [1, 2, 3, 4])'
```

### scan_expr

Like reduce but returns array of intermediate accumulator values

**Signature:** `string, array, any -> array`

**Examples:**

```text
# Running sum
scan_expr('add(accumulator, current)', [1, 2, 3], `0`) -> [1, 3, 6]
# Running product
scan_expr('multiply(accumulator, current)', [2, 3, 4], `1`) -> [2, 6, 24]
# Running max
scan_expr('max(accumulator, current)', [3, 1, 4], `0`) -> [3, 3, 4]
# Empty array
scan_expr('add(accumulator, current)', [], `0`) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'scan_expr(`"add(accumulator, current)"`, [1, 2, 3], `0`)'
```

### some

Check if any element matches (alias for any_expr)

**Signature:** `string, array -> boolean`

**Examples:**

```text
# Some > 2
some('@ > `2`', [1, 2, 3]) -> true
# None > 5
some('@ > `5`', [1, 2, 3]) -> false
# Any active user
some('active', users) -> true/false
# Empty is false
some('@ > `0`', []) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'some(`"@ > `2`"`, [1, 2, 3])'
```

### sort_by_expr

Sort array by expression result in ascending order

**Signature:** `array, expression -> array`

**Examples:**

```text
# Sort by field
sort_by_expr([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]
# Sort by length
sort_by_expr(['bb', 'a', 'ccc'], &length(@)) -> ['a', 'bb', 'ccc']
# Sort users by name
sort_by_expr(users, &name) -> alphabetical
# Empty array
sort_by_expr([], &a) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sort_by_expr([{a: 2}, {a: 1}], &a)'
```

### take_while

Take elements from array while expression is truthy

**Signature:** `string, array -> array`

**Examples:**

```text
# Take while < 4
take_while('@ < `4`', [1, 2, 3, 5, 1]) -> [1, 2, 3]
# Take while positive
take_while('@ > `0`', [3, 2, 1, 0, 5]) -> [3, 2, 1]
# Take short strings
take_while('length(@) < `3`', ['a', 'ab', 'abc']) -> ['a', 'ab']
# None taken
take_while('@ < `0`', [1, 2, 3]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'take_while(`"@ < `4`"`, [1, 2, 3, 5, 1])'
```

### unique_by_expr

Remove duplicates by expression result, keeping first occurrence

**Signature:** `array, expression -> array`

**Examples:**

```text
# Unique by field
unique_by_expr([{a: 1}, {a: 1}], &a) -> [{a: 1}]
# First wins
unique_by_expr([{id: 1, v: 'a'}, {id: 1, v: 'b'}], &id) -> [{id: 1, v: 'a'}]
# Unique by length
unique_by_expr(['aa', 'b', 'cc'], &length(@)) -> ['aa', 'b']
# Empty array
unique_by_expr([], &a) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'unique_by_expr([{a: 1}, {a: 1}], &a)'
```

### until_expr

Loop until condition becomes true, collecting intermediate values (jq parity)

**Signature:** `any, expression, expression -> array`

**Examples:**

```text
# Count until >= 5
until_expr(`1`, &@ >= `5`, &add(@, `1`)) -> [1, 2, 3, 4, 5]
# Double until >= 100
until_expr(`2`, &@ >= `100`, &multiply(@, `2`)) -> [2, 4, 8, 16, 32, 64, 128]
# Halve until <= 1
until_expr(`100`, &@ <= `1`, &divide(@, `2`)) -> [100, 50, 25, 12, 6, 3, 1]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'until_expr(`1`, &@ >= `5`, &add(@, `1`))'
```

### walk

Recursively apply expression to all components of a value (bottom-up)

**Signature:** `string, any -> any`

**Examples:**

```text
# Identity transform
walk('@', data) -> data unchanged
# Get type of result
walk('type(@)', {a: 1}) -> 'object'
# Lengths of nested arrays
walk('length(@)', [[], []]) -> 2
# Keys at each level
walk('keys(@)', {a: {b: 1}}) -> ['a']
```

**CLI Usage:**

```bash
echo '{}' | jpx 'walk(`"@"`, data)'
```

### while_expr

Loop while condition is true, collecting intermediate values (jq parity)

**Signature:** `any, expression, expression -> array`

**Examples:**

```text
# Count from 1 while < 5
while_expr(`1`, &@ < `5`, &add(@, `1`)) -> [1, 2, 3, 4]
# Double while < 100
while_expr(`2`, &@ < `100`, &multiply(@, `2`)) -> [2, 4, 8, 16, 32, 64]
# Halve while > 1
while_expr(`10`, &@ > `1`, &divide(@, `2`)) -> [10, 5, 2]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'while_expr(`1`, &@ < `5`, &add(@, `1`))'
```

### zip_with

Zip two arrays with a custom combiner expression

**Signature:** `string, array, array -> array`

**Examples:**

```text
# Add pairs
zip_with('add([0], [1])', [1, 2], [10, 20]) -> [11, 22]
# Multiply pairs
zip_with('multiply([0], [1])', [2, 3], [4, 5]) -> [8, 15]
# Concat pairs
zip_with('concat([0], [1])', ['a', 'b'], ['1', '2']) -> ['a1', 'b2']
# Empty arrays
zip_with('add([0], [1])', [], []) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'zip_with(`"add([0], [1])"`, [1, 2], [10, 20])'
```

