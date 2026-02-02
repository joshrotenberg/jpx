# Expression Syntax

JMESPath expression syntax reference.

## Identifiers

Access object properties by name:

```bash
echo '{"name": "Alice", "age": 30}' | jpx 'name'
# "Alice"
```

## Index Expressions

Access array elements by index (0-based):

```bash
echo '{"items": ["a", "b", "c"]}' | jpx 'items[0]'
# "a"

echo '{"items": ["a", "b", "c"]}' | jpx 'items[-1]'
# "c"
```

## Slicing

Extract array slices with `[start:stop:step]`:

```bash
echo '[0, 1, 2, 3, 4, 5]' | jpx '[0:3]'
# [0, 1, 2]

echo '[0, 1, 2, 3, 4, 5]' | jpx '[::2]'
# [0, 2, 4]

echo '[0, 1, 2, 3, 4, 5]' | jpx '[::-1]'
# [5, 4, 3, 2, 1, 0]
```

## Wildcard

Project all elements with `*`:

```bash
echo '{"users": [{"name": "Alice"}, {"name": "Bob"}]}' | jpx 'users[*].name'
# ["Alice", "Bob"]
```

## Flatten

Flatten nested arrays with `[]`:

```bash
echo '[[1, 2], [3, 4]]' | jpx '[]'
# [1, 2, 3, 4]
```

## Filter Expressions

Filter arrays with `[?expression]`:

```bash
echo '[{"age": 25}, {"age": 17}, {"age": 30}]' | jpx '[?age > `18`]'
# [{"age": 25}, {"age": 30}]

echo '[{"name": "Alice", "active": true}, {"name": "Bob", "active": false}]' | jpx '[?active].name'
# ["Alice"]
```

### Comparison Operators

| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less than or equal |
| `>` | Greater than |
| `>=` | Greater than or equal |

### Logical Operators

| Operator | Description |
|----------|-------------|
| `&&` | Logical AND |
| `\|\|` | Logical OR |
| `!` | Logical NOT |

## Pipe Expressions

Chain expressions with `|`:

```bash
echo '{"items": [3, 1, 2]}' | jpx 'items | sort(@) | reverse(@)'
# [3, 2, 1]
```

## Multi-Select List

Create arrays from multiple expressions:

```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx '[a, b]'
# [1, 2]
```

## Multi-Select Hash

Create objects from multiple expressions:

```bash
echo '{"first": "Alice", "last": "Smith", "age": 30}' | jpx '{name: first, surname: last}'
# {"name": "Alice", "surname": "Smith"}
```

## Literal Values

Use backticks for literal values:

```bash
# Literal number
echo '{"items": [1, 2, 3]}' | jpx '[?@ > `1`]'

# Literal string
echo '{"items": ["a", "b"]}' | jpx 'contains(items, `"a"`)'

# Literal array
echo '{}' | jpx '`[1, 2, 3]`'

# Literal object
echo '{}' | jpx '`{"key": "value"}`'
```

## Current Node

Reference the current node with `@`:

```bash
echo '[1, 2, 3, 4, 5]' | jpx '[?@ > `2`]'
# [3, 4, 5]

echo '"hello"' | jpx 'upper(@)'
# "HELLO"
```

## Function Calls

Call functions with `function_name(arg1, arg2, ...)`:

```bash
echo '{"items": [1, 2, 3]}' | jpx 'length(items)'
# 3

echo '{"name": "hello"}' | jpx 'upper(name)'
# "HELLO"

echo '{"a": [1, 2], "b": [3, 4]}' | jpx 'merge(a, b)'
# [1, 2, 3, 4]
```

Functions can take literal JSON values as arguments using backticks:

```bash
# Apply default values to an object
echo '{"name": "Alice"}' | jpx 'defaults(@, `{"role": "user", "active": true}`)'
# {"name": "Alice", "role": "user", "active": true}

# Build an object from key-value pairs
echo 'null' | jpx 'from_items(`[["a", 1], ["b", 2]]`)'
# {"a": 1, "b": 2}
```

## Expression References

Use `&` for expression references (used with higher-order functions):

```bash
echo '[{"age": 30}, {"age": 20}]' | jpx 'sort_by(@, &age)'
# [{"age": 20}, {"age": 30}]

echo '[1, 2, 3]' | jpx 'map(&@ * `2`, @)'
# [2, 4, 6]
```
