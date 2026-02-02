# Strict Mode

Strict mode disables all extension functions, limiting jpx to only the 26 standard JMESPath functions. This is useful for writing portable queries that work with any JMESPath implementation.

## Enabling Strict Mode

Use the `--strict` flag or set the `JPX_STRICT=1` environment variable:

```bash
# Via flag
echo '{"name": "alice"}' | jpx --strict 'upper(name)'
# Error: Unknown function: upper

# Via environment variable
export JPX_STRICT=1
echo '{"items": [1, 2, 3]}' | jpx 'length(items)'
# 3
```

## Standard JMESPath Functions

In strict mode, only these functions are available:

| Function | Description |
|----------|-------------|
| `abs(n)` | Absolute value |
| `avg(array)` | Average of numbers |
| `ceil(n)` | Round up |
| `contains(subject, search)` | Check if contains value |
| `ends_with(str, suffix)` | Check string suffix |
| `floor(n)` | Round down |
| `join(glue, array)` | Join strings |
| `keys(obj)` | Object keys |
| `length(subject)` | Length of array/string/object |
| `map(&expr, array)` | Transform each element |
| `max(array)` | Maximum value |
| `max_by(array, &expr)` | Maximum by expression |
| `merge(obj1, obj2, ...)` | Merge objects |
| `min(array)` | Minimum value |
| `min_by(array, &expr)` | Minimum by expression |
| `not_null(arg1, arg2, ...)` | First non-null value |
| `reverse(array)` | Reverse array |
| `sort(array)` | Sort array |
| `sort_by(array, &expr)` | Sort by expression |
| `starts_with(str, prefix)` | Check string prefix |
| `sum(array)` | Sum of numbers |
| `to_array(arg)` | Convert to array |
| `to_number(arg)` | Convert to number |
| `to_string(arg)` | Convert to string |
| `type(arg)` | Get type name |
| `values(obj)` | Object values |

## Use Cases

### Portable Queries

If you're writing queries that need to work across different JMESPath implementations (AWS CLI, Python jmespath, etc.), use strict mode to ensure compatibility:

```bash
# Test your query works with standard JMESPath
jpx --strict '[?status == `active`].name | sort(@)'
```

### CI/CD Validation

Ensure queries in your codebase don't accidentally use extensions:

```bash
# In your CI pipeline
jpx --strict -Q queries.jpx --check
```

### Gradual Migration

When migrating from another JMESPath tool, start in strict mode and gradually adopt extensions as needed.

## Checking if an Expression Uses Extensions

Use `--explain` to see what functions an expression uses:

```bash
jpx --explain 'users[*].name | unique(@) | sort(@)'
```

The `unique` function is an extension, while `sort` is standard.

## MCP Server Strict Mode

The MCP server also supports strict mode. When configured with `--strict`, only standard JMESPath functions are available:

```json
{
  "mcpServers": {
    "jpx-strict": {
      "command": "jpx-server",
      "args": ["--strict"]
    }
  }
}
```
