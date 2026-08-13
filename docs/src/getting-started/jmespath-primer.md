# JMESPath Primer

[JMESPath](https://jmespath.org/) is a query language for JSON. This primer covers the essentials you need to start using jpx effectively.

## Basic Syntax

### Accessing Fields

Use dot notation to access object fields:

```bash
echo '{"name": "Alice", "age": 30}' | jpx 'name'
# "Alice"

echo '{"user": {"profile": {"email": "a@b.com"}}}' | jpx 'user.profile.email'
# "a@b.com"
```

### Array Access

Use brackets for array elements:

```bash
echo '["a", "b", "c"]' | jpx '[0]'
# "a"

echo '["a", "b", "c"]' | jpx '[-1]'
# "c" (last element)

echo '["a", "b", "c", "d", "e"]' | jpx '[1:3]'
# ["b", "c"] (slice)
```

### Wildcard Projection

Use `[*]` to project all array elements:

```bash
echo '[{"name": "Alice"}, {"name": "Bob"}]' | jpx '[*].name'
# ["Alice", "Bob"]
```

## Filtering

Filter arrays with `[?expression]`:

```bash
# Filter by field value
echo '[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]' | \
  jpx '[?age > `28`]'
# [{"name": "Alice", "age": 30}]

# Multiple conditions
echo '[{"name": "Alice", "active": true}, {"name": "Bob", "active": false}]' | \
  jpx '[?active == `true`].name'
# ["Alice"]
```

**Note:** Literal values use backticks: `` `28` ``, `` `true` ``, `` `"string"` ``

## Functions

JMESPath has built-in functions. jpx extends these with 470+ more.

### Standard Functions (26)

These work in all JMESPath implementations:

```bash
echo '[3, 1, 4, 1, 5]' | jpx 'length(@)'
# 5

echo '[3, 1, 4, 1, 5]' | jpx 'sort(@)'
# [1, 1, 3, 4, 5]

echo '{"a": 1, "b": 2}' | jpx 'keys(@)'
# ["a", "b"]
```

The `@` symbol refers to the current element.

### Extension Functions (370+)

jpx adds powerful functions for real-world tasks:

```bash
echo '{"name": "hello world"}' | jpx 'upper(name)'
# "HELLO WORLD"

echo '[1, 2, 2, 3, 3, 3]' | jpx 'unique(@)'
# [1, 2, 3]

echo '[10, 20, 30, 40, 50]' | jpx 'median(@)'
# 30
```

Use `--strict` mode to limit to standard functions only.

## Pipes

Chain operations with `|`:

```bash
echo '[{"n": "Alice"}, {"n": "Bob"}, {"n": "Alice"}]' | \
  jpx '[*].n | unique(@) | sort(@)'
# ["Alice", "Bob"]
```

## Multi-Select

### Lists

Create arrays with `[expr1, expr2]`:

```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx '[a, c]'
# [1, 3]
```

### Objects

Create objects with `{key: expr}`:

```bash
echo '{"first": "Alice", "last": "Smith", "age": 30}' | \
  jpx '{name: first, years: age}'
# {"name": "Alice", "years": 30}
```

## Common Patterns

### Transform Array Elements

```bash
echo '{"users": [{"name": "alice"}, {"name": "bob"}]}' | \
  jpx 'users[*].{username: name, active: `true`}'
# [{"username": "alice", "active": true}, {"username": "bob", "active": true}]
```

### Filter and Project

```bash
echo '{"items": [{"price": 10}, {"price": 50}, {"price": 25}]}' | \
  jpx 'items[?price > `20`].price'
# [50, 25]
```

### Aggregate

```bash
echo '{"sales": [100, 200, 150]}' | jpx 'sum(sales)'
# 450

echo '{"scores": [85, 92, 78, 95]}' | jpx 'avg(scores)'
# 87.5
```

### Nested Access with Defaults

```bash
echo '{"user": {}}' | jpx 'user.name || `"unknown"`'
# "unknown"
```

## Finding Functions

Don't memorize 490+ functions. Search for what you need:

```bash
# Search by keyword
jpx --search "date"

# Get function details
jpx --describe format_date

# List by category
jpx --list-category string
```

## Next Steps

- [Basic Usage](../cli/basic-usage.md) - CLI options and features
- [Function Overview](../functions/overview.md) - All 490+ functions
- [Examples](../cli/examples.md) - Common recipes
- [JMESPath Specification](https://jmespath.org/specification.html) - Official language spec
- [JMESPath Tutorial](https://jmespath.org/tutorial.html) - Interactive tutorial
