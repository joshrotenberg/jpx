# JMESPath Let Expressions (JEP-18)

Let expressions allow binding intermediate results to named variables.

**Support:** jpx, jpx-core. Not all JMESPath implementations support let expressions.

## Basic Syntax

```
let $variable = expression in body
```

The `$variable` is available within `body`. Variable names are prefixed with `$`.

## Examples

### Name an intermediate result
```
Input:  [{"name": "alice", "score": 95}, {"name": "bob", "score": 80}]

let $top = sort_by(@, &score) | reverse(@) | [0]
in $top.name

-> "alice"
```

### Multiple bindings
```
let $active = [?status == 'active'],
    $count = length($active)
in {items: $active, total: $count}
```

### Avoid repeating sub-expressions
Without let (repeated work):
```
{
  high: length([?score > `90`]),
  low: length([?score <= `90`]),
  total: length(@)
}
```

With let (cleaner):
```
let $high = [?score > `90`],
    $low = [?score <= `90`]
in {
  high: length($high),
  low: length($low),
  total: length(@)
}
```

### Nested let expressions
```
let $users = [?type == 'user']
in let $admins = $users[?role == 'admin']
   in {all_users: length($users), admins: length($admins)}
```

## Variable Scoping

- Variables are scoped to the `in` body
- Inner let expressions can shadow outer variables
- Variables cannot be referenced before they are defined
- The current node `@` still refers to the original input within the body

## When to Use Let vs Pipe

**Use pipe** for simple linear chains:
```
[*].name | sort(@) | [0]
```

**Use let** when you need the same intermediate result more than once:
```
let $names = [*].name | sort(@)
in {first: $names[0], last: $names[-1], count: length($names)}
```

**Use let** when building complex results from multiple derived values:
```
let $errors = [?level == 'error'],
    $warnings = [?level == 'warn'],
    $total = length(@)
in {
  error_count: length($errors),
  warning_count: length($warnings),
  error_rate: length($errors) / $total,
  latest_error: $errors | sort_by(@, &timestamp) | [-1]
}
```
