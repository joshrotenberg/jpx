# JMESPath Syntax Reference

Complete reference for every JMESPath node type.

## Expression Types

### Identifier
```
field_name
```
Accesses a field on the current object. Returns `null` if missing.

### Sub-expression
```
a.b.c
```
Chained field access. Equivalent to evaluating `b.c` against the result of `a`.

### Quoted Identifier
```
"field-with-dashes"
"123numeric"
"has spaces"
```
Access fields with names that aren't valid unquoted identifiers.

### Index Expression
```
[N]      -- Nth element (0-based)
[-N]     -- Nth from end
```
Negative indices count from the end: `[-1]` is last, `[-2]` is second-to-last.

### Slice Expression
```
[start:stop]       -- elements from start to stop-1
[start:stop:step]  -- with step
[:stop]            -- from beginning
[start:]           -- to end
[::step]           -- every Nth element
[::-1]             -- reverse
```
All parameters are optional. Default start=0, stop=end, step=1.

### List Projection
```
[*].expr
```
Evaluate `expr` against each element of an array. Null results are omitted.

### Object Projection
```
*.expr
```
Evaluate `expr` against each value of an object. Null results are omitted.

### Flatten Projection
```
[].expr
```
Flatten one level of nesting, then project. `[[1],[2,3]]` -> `[1,2,3]`.

### Filter Expression
```
[?condition]
```
Keep array elements where `condition` is truthy. Inside the filter, `@` refers to each element.

#### Comparators
```
==    equal
!=    not equal
<     less than
<=    less than or equal
>     greater than
>=    greater than or equal
```

#### Logical Operators
```
&&    and
||    or
!     not (prefix)
```

### Multi-Select Hash
```
{key1: expr1, key2: expr2}
```
Evaluate each expression and build a new object with the given keys.

### Multi-Select List
```
[expr1, expr2, expr3]
```
Evaluate each expression and build a new array.

### Pipe Expression
```
left | right
```
Evaluate `left`, then evaluate `right` against its result. Stops projections.

### Literal
```
`42`           number
`"string"`     string
`true`         boolean
`false`        boolean
`null`         null
`[1, 2]`      array
`{"a": 1}`    object
'string'       shorthand for `"string"`
```

### Current Node
```
@
```
Refers to the current value being evaluated. Useful in filters and function arguments.

### Function Call
```
function_name(arg1, arg2, ...)
```
Call a built-in function. Arguments are expressions evaluated against the current node.

### Expression Reference
```
&expression
```
Creates a reference to an expression (not evaluated immediately). Used by functions like `sort_by`, `max_by`, `map`.

### Let Expression (JEP-18)
```
let $var = expr in body
let $a = expr1, $b = expr2 in body
```
Bind variables for use in the body expression. Variables are prefixed with `$`.

## Operator Precedence (highest to lowest)

1. `.` (sub-expression)
2. `[]` (index/slice/filter/flatten)
3. `*` (object projection)
4. Function calls
5. `|` (pipe)
6. Comparisons (`==`, `!=`, `<`, `>`, `<=`, `>=`)
7. `&&` (and)
8. `||` (or)
9. `!` (not)
