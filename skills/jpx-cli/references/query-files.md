# jpx Query Files (.jpx)

Query files let you save and reuse named JMESPath expressions.

## File Format

A `.jpx` file contains named queries, one per line, in `name: expression` format:

```
# Comments start with #
active-users: [?status == 'active']
user-names: [*].name | sort(@)
error-count: length([?level == 'error'])

# Multi-word names use hyphens
top-scorers: sort_by(@, &score) | reverse(@) | [:10]
```

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
- Comments document what each query expects as input
- Query files work with all output formats: `jpx -Q q.jpx:name --csv data.json`
- Combine with variable binding: `jpx -Q q.jpx:name --arg status active data.json`
