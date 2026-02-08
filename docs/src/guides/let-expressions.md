# Let Expressions (JEP-18)

Let expressions add lexical scoping to JMESPath, allowing you to bind intermediate results to named variables. This is the single biggest composability improvement for complex queries.

## Syntax

```
let $variable = expression in body
```

Bind a value to `$variable`, then evaluate `body` with that variable in scope.

### Multiple Bindings

```
let $x = expr1, $y = expr2 in body
```

Bind multiple variables in a single let expression. Bindings are evaluated left-to-right in the outer scope (earlier bindings are not visible to later ones in the same let).

### Nested Lets

```
let $a = expr1 in let $b = expr2 in body
```

Variables from outer lets are available in inner lets.

## Examples

### Naming Intermediate Results

Without let expressions, complex queries repeat subexpressions:

```bash
echo '{"people": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}, {"name": "Charlie", "age": 35}]}' \
  | jpx 'people[?age > `28`] | [*].name | sort(@)'
# ["Alice", "Charlie"]
```

With let expressions, name the intermediate steps:

```bash
echo '{"people": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}, {"name": "Charlie", "age": 35}]}' \
  | jpx 'let $adults = people[?age > `28`] in sort($adults[*].name)'
# ["Alice", "Charlie"]
```

### Parameterizing Thresholds

```bash
echo '{"scores": [85, 92, 67, 78, 95, 43]}' \
  | jpx 'let $cutoff = `80` in scores[?@ >= $cutoff]'
# [85, 92, 95]
```

### Multiple Bindings for Readable Transforms

```bash
echo '{"orders": [{"item": "Book", "qty": 2, "price": 15}, {"item": "Pen", "qty": 5, "price": 3}]}' \
  | jpx 'let $items = orders[*].item, $totals = orders[*].{item: item, total: multiply(qty, price)} in $totals'
# [{"item": "Book", "total": 30}, {"item": "Pen", "total": 15}]
```

### Nested Lets with Shadowing

Variables can be shadowed in inner scopes. The inner binding takes precedence within its scope, and the outer value is restored afterward:

```bash
echo '{"x": 10, "y": 20}' \
  | jpx 'let $a = x in let $a = y in $a'
# 20
```

### Using Variables in Filters

```bash
echo '{"threshold": 30, "people": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 35}]}' \
  | jpx 'let $min_age = threshold in people[?age >= $min_age][*].name'
# ["Bob"]
```

### Variables with Functions

```bash
echo '{"data": [3, 1, 4, 1, 5, 9, 2, 6]}' \
  | jpx 'let $sorted = sort(data), $count = length(data) in {sorted: $sorted, count: $count}'
# {"sorted": [1, 1, 2, 3, 4, 5, 6, 9], "count": 8}
```

## Scoping Rules

- **Lexical scoping**: Variables are visible only within the `in` body of the let that defines them.
- **Shadowing**: An inner let can rebind a variable name. The original value is restored when the inner scope ends.
- **Projection stopping**: Let expressions stop projections, similar to pipe expressions. Inside a `let ... in body`, the body evaluates against the current node, not projected elements.
- **Binding evaluation order**: In `let $a = e1, $b = e2 in body`, both `e1` and `e2` are evaluated in the scope before the let (the outer scope). `$a` is not visible when evaluating `e2`.

## Strict Mode

Let expressions are a JEP-18 extension to the JMESPath specification. They are **disabled in strict mode** (`--strict` or `JPX_STRICT=1`), which enforces standard JMESPath compliance only.

```bash
echo '{"x": 1}' | jpx --strict 'let $a = x in $a'
# Error: Let expressions are not available in strict mode
```

## When to Use Let Expressions

- **Complex multi-step queries**: Name intermediate results instead of deeply nesting pipes
- **Repeated subexpressions**: Compute a value once and reference it multiple times
- **Parameterized filters**: Bind thresholds or configuration values for readability
- **Building structured output**: Compute several derived values and assemble them into an object
