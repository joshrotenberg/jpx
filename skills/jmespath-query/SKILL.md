---
name: jmespath-query
description: "Write JMESPath expressions to query and transform JSON data. Use when the user needs to extract, filter, project, or reshape JSON -- or when JMESPath is mentioned. Covers the full JMESPath specification including identifiers, projections, filters, multi-select, pipes, functions, and let expressions. Applicable to any JMESPath implementation (AWS CLI, Azure CLI, Boto3, jpx, etc.)."
license: MIT OR Apache-2.0
metadata:
  author: joshrotenberg
  version: "1.0"
---

# Writing JMESPath Expressions

JMESPath is a query language for JSON. Given a JSON document and an expression, it produces a new JSON value. This skill covers the full specification.

## Quick Reference

| Syntax | What it does | Example |
|--------|-------------|---------|
| `name` | Access field | `{"name":"jo"}` -> `"jo"` |
| `a.b.c` | Nested access | `{"a":{"b":{"c":1}}}` -> `1` |
| `[0]` | Array index | `[10,20,30]` -> `10` |
| `[-1]` | Last element | `[10,20,30]` -> `30` |
| `[0:3]` | Slice | `[0,1,2,3,4]` -> `[0,1,2]` |
| `[*].name` | List projection | `[{"name":"a"},{"name":"b"}]` -> `["a","b"]` |
| `*.size` | Object projection | `{"a":{"size":1},"b":{"size":2}}` -> `[1,2]` |
| `[]` | Flatten | `[[1,2],[3,4]]` -> `[1,2,3,4]` |
| `[?age > \`30\`]` | Filter | Keep elements matching condition |
| `{a: x, b: y}` | Multi-select hash | Build new object |
| `[x, y]` | Multi-select list | Build new array |
| `expr \| func(@)` | Pipe | Chain expressions |
| `length(@)` | Function call | Built-in functions |

## Identifiers and Sub-expressions

Access object fields by name. Chain with `.` for nested access.

```
Input:  {"user": {"name": "alice", "age": 30}}

user          -> {"name": "alice", "age": 30}
user.name     -> "alice"
user.age      -> 30
missing       -> null
user.missing  -> null
```

Quoted identifiers handle special characters: `"my-field"`, `"with spaces"`, `"123starts-with-number"`.

## Index and Slice Expressions

```
Input:  [0, 1, 2, 3, 4, 5]

[0]      -> 0
[2]      -> 2
[-1]     -> 5        (last element)
[-2]     -> 4        (second to last)
[1:4]    -> [1,2,3]  (start:stop, exclusive end)
[:3]     -> [0,1,2]  (first 3)
[-2:]    -> [4,5]    (last 2)
[::2]    -> [0,2,4]  (every other)
[::-1]   -> [5,4,3,2,1,0]  (reverse)
```

## Projections

Projections apply an expression to each element, collecting results. Null results are omitted.

### List projections (`[*]`)

```
Input:  [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]

[*].name       -> ["alice", "bob"]
[*].age        -> [30, 25]
[*].missing    -> []            (nulls are omitted)
```

### Object projections (`*`)

```
Input:  {"web": {"port": 80}, "db": {"port": 5432}}

*.port         -> [80, 5432]
```

### Flatten projections (`[]`)

```
Input:  [[1, 2], [3, 4], [5]]

[]             -> [1, 2, 3, 4, 5]

Input:  [{"tags": ["a","b"]}, {"tags": ["c"]}]

[*].tags[]     -> ["a", "b", "c"]    (project then flatten)
```

## Filter Expressions

Filter arrays with `[?condition]`. The current element is `@`.

```
Input:  [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}, {"name": "carol", "age": 35}]

[?age > `30`]                -> [{"name": "carol", "age": 35}]
[?age >= `30`]               -> [alice, carol objects]
[?name == 'alice']           -> [{"name": "alice", "age": 30}]
[?name != 'bob']             -> [alice, carol objects]
[?age > `25` && age < `35`]  -> [{"name": "alice", "age": 30}]
[?age < `26` || age > `34`]  -> [bob, carol objects]
[?!active]                   -> elements where active is falsy
[?contains(name, 'a')]       -> elements where name contains "a"
```

**Important:** Literal numbers use backticks: `` `30` ``, not `30`. Strings use single quotes: `'alice'`. This is the most common mistake.

## Multi-Select

### Multi-select hash (build objects)

```
Input:  {"first": "alice", "last": "smith", "age": 30}

{name: first, surname: last}  -> {"name": "alice", "surname": "smith"}
{full: join(' ', [first, last]), age: age}  -> {"full": "alice smith", "age": 30}
```

### Multi-select list (build arrays)

```
Input:  {"a": 1, "b": 2, "c": 3}

[a, b]        -> [1, 2]
[a, b, c]     -> [1, 2, 3]
```

### Combined with projections

```
Input:  [{"name": "alice", "score": 95}, {"name": "bob", "score": 80}]

[*].{student: name, grade: score}  -> [{"student":"alice","grade":95}, {"student":"bob","grade":80}]
```

## Pipe Expressions

Use `|` to chain expressions. The right side receives the output of the left side. Pipes stop projections -- the right side gets the full result, not each element.

```
Input:  [{"name": "alice"}, {"name": "bob"}, {"name": "carol"}]

[*].name                  -> ["alice", "bob", "carol"]
[*].name | sort(@)        -> ["alice", "bob", "carol"]
[*].name | sort(@) | [0]  -> "alice"
[*].name | length(@)      -> 3
[?score > `80`] | [0]     -> first element matching filter
```

**Key insight:** Without the pipe, `[*].name | [0]` takes the first of the projected names. With the pipe, `[0]` operates on the collected array, not on each element.

## Literal Values

Backtick-quoted values are JSON literals within expressions:

```
`42`           -> number 42
`"hello"`      -> string "hello"
`true`         -> boolean true
`null`         -> null
`[1, 2, 3]`   -> array [1, 2, 3]
`{"a": 1}`    -> object {"a": 1}
```

Strings can also use single quotes (simpler for shell usage): `'hello'` is equivalent to `` `"hello"` ``.

## Built-in Functions

JMESPath spec defines 26 functions. The most commonly used:

| Function | Description | Example |
|----------|-------------|---------|
| `length(x)` | Array/string/object size | `length([1,2,3])` -> `3` |
| `sort(arr)` | Sort array | `sort([3,1,2])` -> `[1,2,3]` |
| `sort_by(arr, &key)` | Sort by expression | `sort_by(users, &age)` |
| `reverse(x)` | Reverse array/string | `reverse([1,2,3])` -> `[3,2,1]` |
| `contains(subject, search)` | Check membership | `contains([1,2], `1`)` -> `true` |
| `keys(obj)` | Object keys | `keys({"a":1})` -> `["a"]` |
| `values(obj)` | Object values | `values({"a":1})` -> `[1]` |
| `join(sep, arr)` | Join strings | `join(', ', ["a","b"])` -> `"a, b"` |
| `to_string(x)` | Convert to string | `to_string(`42`)` -> `"42"` |
| `to_number(x)` | Convert to number | `to_number('42')` -> `42` |
| `type(x)` | Type name | `type([])` -> `"array"` |
| `not_null(a, b, ...)` | First non-null | `not_null(null, 'ok')` -> `"ok"` |
| `max(arr)` / `min(arr)` | Max/min value | `max([1,3,2])` -> `3` |
| `max_by(arr, &key)` | Max by expression | `max_by(users, &score)` |
| `min_by(arr, &key)` | Min by expression | `min_by(users, &age)` |
| `starts_with(s, prefix)` | String prefix check | `starts_with('hello', 'he')` -> `true` |
| `ends_with(s, suffix)` | String suffix check | `ends_with('hello', 'lo')` -> `true` |
| `map(&expr, arr)` | Transform each element | `map(&age, users)` |
| `merge(obj1, obj2)` | Merge objects | `merge({"a":1}, {"b":2})` |
| `sum(arr)` | Sum of numbers | `sum([1,2,3])` -> `6` |
| `avg(arr)` | Average of numbers | `avg([1,2,3])` -> `2.0` |
| `floor(n)` / `ceil(n)` | Round down/up | `floor(`3.7`)` -> `3` |
| `abs(n)` | Absolute value | `abs(`-5`)` -> `5` |

### Expression references (exprefs)

Functions like `sort_by`, `max_by`, `min_by`, and `map` take an **expression reference** using `&`:

```
sort_by(users, &age)         -- sort by the "age" field
sort_by(users, &to_number(id))  -- sort by numeric id
max_by(items, &price)        -- item with highest price
map(&name, users)            -- extract all names
```

The `&` prefix creates a reference to the expression that the function evaluates against each element.

## Let Expressions (JEP-18)

Let expressions bind intermediate results to variables. Supported by jpx and some other implementations.

```
let $names = [*].name in sort($names) | [0]

let $adults = [?age >= `18`],
    $count = length($adults)
in {adults: $adults, count: $count}
```

Variables are scoped to the `in` body. Use let expressions to name intermediate results and avoid repeating sub-expressions.

See [references/let-expressions.md](references/let-expressions.md) for advanced patterns.

## Common Patterns

### Extract and reshape
```
[*].{id: id, full_name: join(' ', [first, last])}
```

### Filter, sort, take top N
```
[?status == 'active'] | sort_by(@, &score) | reverse(@) | [:5]
```

### Nested array flattening
```
departments[*].employees[] | [?role == 'engineer']
```

### Conditional value
```
not_null(preferred_name, display_name, email)
```

### Count by filtering
```
length([?type == 'error'])
```

### Check if any/all match
```
length([?status == 'failed']) > `0`     -- any failed?
length([?status != 'ok']) == `0`        -- all ok?
```

## Common Mistakes

1. **Forgetting backticks for numbers:** `[?age > 30]` is wrong -- use `[?age > `30`]`
2. **Using double quotes for strings in expressions:** `[?name == "alice"]` is wrong -- use single quotes `[?name == 'alice']`
3. **Expecting nulls in projections:** `[*].missing` returns `[]`, not `[null, null]` -- projections skip nulls
4. **Pipe vs no pipe:** `[*].name[0]` gets first char of each name; `[*].name | [0]` gets first name
5. **Filter on nested field:** `[?address.city == 'NYC']` works -- you can use sub-expressions in filters

## Extended Functions

jpx extends JMESPath with 400+ additional functions for strings, math, dates, hashing, encoding, regex, and more. See the [jpx-functions skill](../jpx-functions/SKILL.md) for details.
