# jpx Query Files (.jpx)

Query files let you save and reuse named JMESPath expressions.

## File Format

A `.jpx` file contains one or more named queries. Each query starts with a
`-- :name` directive and may include a `-- :desc` directive. Expressions may
span multiple lines and end at the next `-- :name` directive:

```
-- :name active-users
-- :desc Return active users
[?status == 'active']

-- :name user-names
[*].name | sort(@)

-- :name top-scorers
sort_by(@, &score)
| reverse(@)
| [:10]
```

Lines beginning with `-- ` that are not directives are comments. They are
ignored, as are blank lines between query definitions.

## Usage

```bash
# Run a named query
jpx -Q queries.jpx:active-users data.json

# Alternative: separate --query flag
jpx -Q queries.jpx --query active-users data.json

# List available queries in a file
jpx -Q queries.jpx --list-queries

# Validate all queries without executing
jpx -Q queries.jpx --check
```

## Validation

The `--check` flag parses all queries and reports errors without executing:

```bash
jpx -Q queries.jpx --check
# Exit 0 if all valid, exit 1 if any errors
```

Useful in CI/CD to catch syntax errors in query libraries.

## Tips

- Keep related queries in the same file (e.g., `user-queries.jpx`, `metrics.jpx`)
- Use descriptive names -- they appear in `--list-queries` output
- Use `-- :desc` to document what each query expects as input
- Query files work with all output formats: `jpx -Q q.jpx:name --csv data.json`
- Combine with variable binding: `jpx -Q q.jpx:name --arg status active data.json`
