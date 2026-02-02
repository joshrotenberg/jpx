# Standard JMESPath Functions

These are the standard JMESPath functions as defined in the specification. They work in all JMESPath implementations.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`abs`](#abs) | `number -> number` | Returns the absolute value of a number |
| [`avg`](#avg) | `array[number] -> number` | Returns the average of an array of numbers |
| [`ceil`](#ceil) | `number -> number` | Returns the smallest integer greater than or equal to the number |
| [`contains`](#contains) | `array\|string, any -> boolean` | Returns true if the subject contains the search value |
| [`ends_with`](#ends-with) | `string, string -> boolean` | Returns true if the subject ends with the suffix |
| [`floor`](#floor) | `number -> number` | Returns the largest integer less than or equal to the number |
| [`join`](#join) | `string, array[string] -> string` | Returns array elements joined into a string with a separator |
| [`keys`](#keys) | `object -> array[string]` | Returns an array of keys from an object |
| [`length`](#length) | `array\|object\|string -> number` | Returns the length of an array, object, or string |
| [`map`](#map) | `expression, array -> array` | Applies an expression to each element of an array |
| [`max`](#max) | `array[number]\|array[string] -> number\|string` | Returns the maximum value in an array |
| [`max_by`](#max-by) | `array, expression -> any` | Returns the element with maximum value by expression |
| [`merge`](#merge) | `object... -> object` | Merges objects into a single object |
| [`min`](#min) | `array[number]\|array[string] -> number\|string` | Returns the minimum value in an array |
| [`min_by`](#min-by) | `array, expression -> any` | Returns the element with minimum value by expression |
| [`not_null`](#not-null) | `any... -> any` | Returns the first non-null argument |
| [`reverse`](#reverse) | `array\|string -> array\|string` | Reverses an array or string |
| [`sort`](#sort) | `array[number]\|array[string] -> array` | Sorts an array of numbers or strings |
| [`sort_by`](#sort-by) | `array, expression -> array` | Sorts an array by expression result |
| [`starts_with`](#starts-with) | `string, string -> boolean` | Returns true if the subject starts with the prefix |
| [`sum`](#sum) | `array[number] -> number` | Returns the sum of an array of numbers |
| [`to_array`](#to-array) | `any -> array` | Converts a value to an array |
| [`to_number`](#to-number) | `any -> number` | Converts a value to a number |
| [`to_string`](#to-string) | `any -> string` | Converts a value to a string |
| [`type`](#type) | `any -> string` | Returns the type of a value as a string |
| [`values`](#values) | `object -> array` | Returns an array of values from an object |

## Functions

### abs

Returns the absolute value of a number

**Signature:** `number -> number`

**Examples:**

```text
# Negative number
abs(`-5`) -> 5
# Positive number
abs(`5`) -> 5
# Zero
abs(`0`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'abs(`-5`)'
```

### avg

Returns the average of an array of numbers

**Signature:** `array[number] -> number`

**Examples:**

```text
# Simple average
avg([1, 2, 3]) -> 2
# Four numbers
avg([10, 20, 30, 40]) -> 25
# Single element
avg([5]) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'avg([1, 2, 3])'
```

### ceil

Returns the smallest integer greater than or equal to the number

**Signature:** `number -> number`

**Examples:**

```text
# Round up decimal
ceil(`1.5`) -> 2
# Negative rounds toward zero
ceil(`-1.5`) -> -1
# Integer unchanged
ceil(`5`) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ceil(`1.5`)'
```

### contains

Returns true if the subject contains the search value

**Signature:** `array|string, any -> boolean`

**Examples:**

```text
# Array contains number
contains([1, 2, 3], `2`) -> true
# String contains substring
contains('hello', 'ell') -> true
# Not found
contains([1, 2, 3], `5`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'contains([1, 2, 3], `2`)'
```

### ends_with

Returns true if the subject ends with the suffix

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Ends with suffix
ends_with('hello', 'lo') -> true
# Does not end with
ends_with('hello', 'he') -> false
# File extension
ends_with('test.txt', '.txt') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ends_with(`"hello"`, `"lo"`)'
```

### floor

Returns the largest integer less than or equal to the number

**Signature:** `number -> number`

**Examples:**

```text
# Round down decimal
floor(`1.9`) -> 1
# Negative rounds away from zero
floor(`-1.5`) -> -2
# Integer unchanged
floor(`5`) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'floor(`1.9`)'
```

### join

Returns array elements joined into a string with a separator

**Signature:** `string, array[string] -> string`

**Examples:**

```text
# Join with comma
join(', ', ['a', 'b', 'c']) -> \"a, b, c\"
# Join date parts
join('-', ['2024', '01', '15']) -> \"2024-01-15\"
# Join without separator
join('', ['a', 'b', 'c']) -> \"abc\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'join(`", "`, ['a', `"b"`, 'c'])'
```

### keys

Returns an array of keys from an object

**Signature:** `object -> array[string]`

**Examples:**

```text
# Get object keys
keys({a: 1, b: 2}) -> [\"a\", \"b\"]
# Keys sorted alphabetically
keys({name: \"John\", age: 30}) -> [\"age\", \"name\"]
# Empty object
keys({}) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'keys({a: 1, b: 2})'
```

### length

Returns the length of an array, object, or string

**Signature:** `array|object|string -> number`

**Examples:**

```text
# Array length
length([1, 2, 3]) -> 3
# String length
length('hello') -> 5
# Object key count
length({a: 1, b: 2}) -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'length([1, 2, 3])'
```

### map

Applies an expression to each element of an array

**Signature:** `expression, array -> array`

**Examples:**

```text
# Extract field from objects
map(&a, [{a: 1}, {a: 2}]) -> [1, 2]
# Apply function
map(&length(@), ['a', 'bb', 'ccc']) -> [1, 2, 3]
# Identity mapping
map(&@, [1, 2, 3]) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'map(&a, [{a: 1}, {a: 2}])'
```

### max

Returns the maximum value in an array

**Signature:** `array[number]|array[string] -> number|string`

**Examples:**

```text
# Max of numbers
max([1, 3, 2]) -> 3
# Max of strings
max(['a', 'c', 'b']) -> 'c'
# Negative numbers
max([-5, -1, -10]) -> -1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'max([1, 3, 2])'
```

### max_by

Returns the element with maximum value by expression

**Signature:** `array, expression -> any`

**Examples:**

```text
# Max by field
max_by([{a: 1}, {a: 2}], &a) -> {a: 2}
# Max by age
max_by([{name: 'a', age: 30}, {name: 'b', age: 25}], &age) -> {name: 'a', age: 30}
# Max by length
max_by(['aa', 'b', 'ccc'], &length(@)) -> 'ccc'
```

**CLI Usage:**

```bash
echo '{}' | jpx 'max_by([{a: 1}, {a: 2}], &a)'
```

### merge

Merges objects into a single object

**Signature:** `object... -> object`

**Examples:**

```text
# Merge two objects
merge({a: 1}, {b: 2}) -> {a: 1, b: 2}
# Later values override
merge({a: 1}, {a: 2}) -> {a: 2}
# Multiple objects
merge({a: 1}, {b: 2}, {c: 3}) -> {a: 1, b: 2, c: 3}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'merge({a: 1}, {b: 2})'
```

### min

Returns the minimum value in an array

**Signature:** `array[number]|array[string] -> number|string`

**Examples:**

```text
# Min of numbers
min([1, 3, 2]) -> 1
# Min of strings
min(['a', 'c', 'b']) -> 'a'
# Negative numbers
min([-5, -1, -10]) -> -10
```

**CLI Usage:**

```bash
echo '{}' | jpx 'min([1, 3, 2])'
```

### min_by

Returns the element with minimum value by expression

**Signature:** `array, expression -> any`

**Examples:**

```text
# Min by field
min_by([{a: 1}, {a: 2}], &a) -> {a: 1}
# Min by age
min_by([{name: 'a', age: 30}, {name: 'b', age: 25}], &age) -> {name: 'b', age: 25}
# Min by length
min_by(['aa', 'b', 'ccc'], &length(@)) -> 'b'
```

**CLI Usage:**

```bash
echo '{}' | jpx 'min_by([{a: 1}, {a: 2}], &a)'
```

### not_null

Returns the first non-null argument

**Signature:** `any... -> any`

**Examples:**

```text
# Skip nulls
not_null(`null`, 'a', 'b') -> \"a\"
# First non-null
not_null('first', 'second') -> \"first\"
# Multiple nulls
not_null(`null`, `null`, `1`) -> 1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'not_null(`null`, `"a"`, `"b"`)'
```

### reverse

Reverses an array or string

**Signature:** `array|string -> array|string`

**Examples:**

```text
# Reverse array
reverse([1, 2, 3]) -> [3, 2, 1]
# Reverse string
reverse('hello') -> 'olleh'
# Empty array
reverse([]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reverse([1, 2, 3])'
```

### sort

Sorts an array of numbers or strings

**Signature:** `array[number]|array[string] -> array`

**Examples:**

```text
# Sort numbers
sort([3, 1, 2]) -> [1, 2, 3]
# Sort strings
sort(['c', 'a', 'b']) -> ['a', 'b', 'c']
# Empty array
sort([]) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sort([3, 1, 2])'
```

### sort_by

Sorts an array by expression result

**Signature:** `array, expression -> array`

**Examples:**

```text
# Sort by field
sort_by([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]
# Sort by length
sort_by(['bb', 'a', 'ccc'], &length(@)) -> ['a', 'bb', 'ccc']
# Sort by name
sort_by([{name: 'z'}, {name: 'a'}], &name) -> [{name: 'a'}, {name: 'z'}]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sort_by([{a: 2}, {a: 1}], &a)'
```

### starts_with

Returns true if the subject starts with the prefix

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Starts with prefix
starts_with('hello', 'he') -> true
# Does not start with
starts_with('hello', 'lo') -> false
# URL protocol
starts_with('https://example.com', 'https') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'starts_with(`"hello"`, `"he"`)'
```

### sum

Returns the sum of an array of numbers

**Signature:** `array[number] -> number`

**Examples:**

```text
# Sum of numbers
sum([1, 2, 3]) -> 6
# With negative
sum([10, -5, 3]) -> 8
# Empty array
sum([]) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sum([1, 2, 3])'
```

### to_array

Converts a value to an array

**Signature:** `any -> array`

**Examples:**

```text
# String to array
to_array('hello') -> [\"hello\"]
# Array unchanged
to_array([1, 2]) -> [1, 2]
# Number to array
to_array(`5`) -> [5]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_array(`"hello"`)'
```

### to_number

Converts a value to a number

**Signature:** `any -> number`

**Examples:**

```text
# String to number
to_number('42') -> 42
# String to float
to_number('3.14') -> 3.14
# Number unchanged
to_number(`5`) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_number(`"42"`)'
```

### to_string

Converts a value to a string

**Signature:** `any -> string`

**Examples:**

```text
# Number to string
to_string(`42`) -> \"42\"
# Boolean to string
to_string(`true`) -> \"true\"
# String unchanged
to_string('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_string(`42`)'
```

### type

Returns the type of a value as a string

**Signature:** `any -> string`

**Examples:**

```text
# String type
type('hello') -> \"string\"
# Number type
type(`42`) -> \"number\"
# Array type
type([1, 2]) -> \"array\"
# Object type
type({a: 1}) -> \"object\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'type(`"hello"`)'
```

### values

Returns an array of values from an object

**Signature:** `object -> array`

**Examples:**

```text
# Get object values
values({a: 1, b: 2}) -> [1, 2]
# String values
values({x: 'hello', y: 'world'}) -> ['hello', 'world']
# Empty object
values({}) -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'values({a: 1, b: 2})'
```

