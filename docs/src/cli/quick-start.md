# Quick Start

jpx is a command-line JSON processor with 400+ built-in functions. Pipe JSON in, get transformed JSON out.

Five things to know to start using it:

## 1. Get a field

```bash
echo '{"name": "Alice", "city": "NYC"}' | jpx 'name'
# "Alice"
```

Nested fields use dots:

```bash
echo '{"user": {"name": "Alice"}}' | jpx 'user.name'
# "Alice"
```

## 2. Get all items from an array

Use `[*]` to get every element:

```bash
echo '[{"name": "Alice"}, {"name": "Bob"}]' | jpx '[*].name'
# ["Alice", "Bob"]
```

## 3. Filter an array

Use `[?condition]` to filter:

```bash
echo '[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]' | jpx '[?age > `28`]'
# [{"name": "Alice", "age": 30}]
```

Note: literal values use backticks (`` `28` ``).

## 4. Call a function

jpx has 400+ functions. Pass data with `@` (current value):

```bash
echo '[3, 1, 4, 1, 5]' | jpx 'sort(@)'
# [1, 1, 3, 4, 5]

echo '[3, 1, 4, 1, 5]' | jpx 'unique(@)'
# [3, 1, 4, 5]

echo '{"name": "hello"}' | jpx 'upper(name)'
# "HELLO"
```

## 5. Chain with pipes

Combine operations with `|`:

```bash
echo '[{"n": "Alice"}, {"n": "Bob"}, {"n": "Alice"}]' | jpx '[*].n | unique(@) | sort(@)'
# ["Alice", "Bob"]
```

---

That's it. You can do a lot with just these five patterns.

## Finding Functions

Don't memorize 400 functions. Search for what you need:

```bash
jpx --search unique
jpx --describe unique
```

## Next Steps

- [Cookbook](../guide/cookbook.md) - Common tasks and recipes
- [Basic Usage](./basic-usage.md) - More CLI options
- [Function Reference](../functions/overview.md) - All functions by category
- [Why jpx?](../guide/why-jpx.md) - Compare jpx to Python, jq, and other tools
