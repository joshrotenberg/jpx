# Query Files

Query files let you store JMESPath expressions externally, making them reusable, version-controlled, and easier to maintain. jpx supports two formats:

1. **Simple query files** - A single expression in a plain text file
2. **Query libraries (.jpx)** - Multiple named queries in one file

## Simple Query Files

The simplest approach: put your expression in a file and reference it with `-Q`:

```bash
# Create a query file
echo 'users[?active].{name: name, email: email}' > active-users.txt

# Use it
jpx -Q active-users.txt data.json
```

This is useful for:
- Long, complex expressions you don't want to retype
- Sharing queries across scripts
- Version-controlling important queries

## Query Libraries (.jpx)

For projects with multiple related queries, use a `.jpx` query library file. This format lets you define named queries with optional descriptions:

```
-- :name active-users
-- :desc Get all active users with their contact info
users[?active].{name: name, email: email}

-- :name admin-emails
-- :desc Extract just the admin email addresses
users[?role == `admin`].email

-- :name user-stats
-- :desc Summary statistics about users
{
  total: length(users),
  active: length(users[?active]),
  admins: length(users[?role == `admin`])
}
```

### Using Query Libraries

There are two ways to run a query from a library:

```bash
# Colon syntax (concise)
jpx -Q queries.jpx:active-users data.json

# Separate flag (explicit)
jpx -Q queries.jpx --query active-users data.json
```

Both are equivalent. Use whichever feels more natural.

### Listing Available Queries

See what queries are in a library:

```bash
jpx -Q queries.jpx --list-queries
```

Output:
```
Queries in queries.jpx:

  NAME          DESCRIPTION
  -----------   ----------------------------------------
  active-users  Get all active users with their contact info
  admin-emails  Extract just the admin email addresses
  user-stats    Summary statistics about users

Use: jpx -Q queries.jpx:<query-name> <input>
```

### Validating Queries

Check that all queries in a library are syntactically valid:

```bash
jpx -Q queries.jpx --check
```

Output:
```
Validating queries.jpx...

  ✓ active-users
  ✓ admin-emails
  ✓ user-stats

All queries valid.
```

This is useful in CI pipelines to catch syntax errors before deployment.

## .jpx Format Specification

The `.jpx` format is inspired by [SQLDelight](https://cashapp.github.io/sqldelight/) and [HugSQL](https://www.hugsql.org/) patterns: store multiple named queries in a single file with comment-based metadata directives.

### Directives

| Directive | Required | Description |
|-----------|----------|-------------|
| `-- :name <name>` | Yes | Starts a new query. The name is used for lookup via `-Q file.jpx:name`. |
| `-- :desc <text>` | No | Adds a description to the current query. Shown by `--list-queries`. |
| `-- <anything>` | - | Plain comment. Ignored by the parser. |

A query definition begins at its `-- :name` directive and ends at the next `-- :name` directive or end of file.

### Grammar

```
library     = { query }+
query       = name_line [ desc_line ] { comment | blank | expr_line }+
name_line   = "-- :name " NAME
desc_line   = "-- :desc " TEXT
comment     = "-- " TEXT | "--"
blank       = empty or whitespace-only line
expr_line   = any non-blank line not starting with "-- "
```

- **NAME**: Non-empty text after `-- :name `, trimmed of leading/trailing whitespace
- **TEXT**: Arbitrary text for the rest of the line
- Expression lines are joined with newlines in order, then the result is trimmed

### Multi-Line Expressions

Expression lines are concatenated with newline separators, preserving indentation. The final expression is trimmed of leading and trailing whitespace. This lets you write readable multi-line constructions:

```
-- :name user-report
{
  total: length(users),
  active: length(users[?active]),
  admins: length(users[?role == `admin`])
}
```

The resulting expression is:

```
{
  total: length(users),
  active: length(users[?active]),
  admins: length(users[?role == `admin`])
}
```

### Blank Lines and Comments Within Queries

Blank lines between `-- :name` directives are ignored -- they don't become part of the expression. You can use them freely for visual separation:

```
-- :name query-a
length(@)

-- This blank line above and this comment are ignored

-- :name query-b
keys(@)
```

Comments (`-- ` lines) between directives are also ignored.

### Error Reporting

The parser reports errors with line numbers for easy debugging:

| Error | Example | Message |
|-------|---------|---------|
| Empty name | `-- :name ` | `Empty query name. Give the query a name, as in '-- :name my-query'. at line 3` |
| No expression | `-- :name foo` followed by `-- :name bar` | `Query 'foo' has no expression. Add the expression on the line after '-- :name foo', or remove the entry. at line 1` |
| Duplicate name | Two `-- :name stats` | `Duplicate query name 'stats'. Query names must be unique within a file; rename one of them. at line 7` |
| No queries | File with only comments | `No queries found. Use '-- :name <query-name>' to define queries.` |

Each message names the corrective action, not just the diagnosis. That is a
deliberate contract, described in [AGENTS.md](https://github.com/joshrotenberg/jpx/blob/main/AGENTS.md)
and enforced by `crates/jpx/tests/error_contract.rs`.

### Detection Logic

jpx decides how to treat a query file based on these rules:

1. **File extension**: Files ending in `.jpx` are always treated as query libraries
2. **Content sniffing**: If the first non-empty line starts with `-- :name `, the file is treated as a library regardless of extension
3. **Fallback**: Everything else is a simple single-expression file

### Colon Syntax and Windows Paths

The colon syntax `-Q file.jpx:query-name` is shorthand for `-Q file.jpx --query query-name`.

On Windows, paths containing drive letters (e.g., `C:\queries\file.jpx`) are handled correctly -- the parser recognizes drive-letter prefixes and only splits on the final colon that separates the file path from the query name:

```bash
# Windows
jpx -Q C:\queries\file.jpx:my-query data.json

# Unix
jpx -Q /opt/queries/file.jpx:my-query data.json
```

If your query name contains a colon, use the explicit `--query` flag instead.

## Real-World Examples

### NLP Analysis Library

Create reusable text processing pipelines:

```
-- :name clean-html
-- :desc Strip HTML tags and normalize whitespace
regex_replace(@, `<[^>]+>`, ` `) | collapse_whitespace(@)

-- :name extract-keywords
-- :desc Get top keywords from text (stemmed, no stopwords)
tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)

-- :name title-keywords
-- :desc Extract keywords from article titles
hits[*].title | join(` `, @) | tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)

-- :name reading-stats
-- :desc Get reading time and word count
{
  word_count: word_count(@),
  reading_time: reading_time(@),
  sentence_count: sentence_count(@)
}
```

Use it:
```bash
# Clean HTML from a field
jpx 'story_text' data.json | jpx -Q nlp.jpx:clean-html

# Analyze Hacker News titles
jpx -Q nlp.jpx:title-keywords hn_front.json
```

### API Response Processing

Standardize how you extract data from APIs:

```
-- :name github-repos
-- :desc Extract repo summary from GitHub API response
[*].{
  name: name,
  stars: stargazers_count,
  language: language,
  description: description | default(@, `"No description"`)
}

-- :name github-issues
-- :desc Format GitHub issues for display
[*].{
  number: number,
  title: title,
  state: state,
  author: user.login,
  labels: labels[*].name | join(`, `, @)
}

-- :name paginated-total
-- :desc Get total from paginated API response
{
  count: length(items),
  total: total_count,
  has_more: length(items) < total_count
}
```

### Data Transformation Library

Common transformations for ETL pipelines:

```
-- :name flatten-nested
-- :desc Flatten nested user records for CSV export
[*].{
  id: id,
  name: profile.name,
  email: profile.email,
  city: profile.address.city,
  country: profile.address.country,
  created: metadata.created_at
}

-- :name aggregate-by-status
-- :desc Group and count records by status
group_by(@, &status) | map(&{ status: [0].status, count: length(@) }, @)

-- :name enrich-timestamps
-- :desc Add formatted date fields
[*] | map(&merge(@, {
  created_date: format_datetime(created_at, '%Y-%m-%d'),
  created_time: format_datetime(created_at, '%H:%M:%S')
}), @)
```

### Log Analysis Library

Queries for processing structured logs:

```
-- :name errors-only
-- :desc Filter to just error-level logs
[?level == `error` || level == `ERROR`]

-- :name errors-by-service
-- :desc Count errors grouped by service name
[?level == `error`] | group_by(@, &service) | map(&{ service: [0].service, count: length(@) }, @)

-- :name recent-errors
-- :desc Errors from the last hour with context
[?level == `error`] | sort_by(@, &timestamp) | reverse(@) | [:20].{
  time: timestamp,
  service: service,
  message: message,
  trace_id: trace_id
}

-- :name slow-requests
-- :desc Requests taking longer than 1 second
[?duration_ms > `1000`] | sort_by(@, &duration_ms) | reverse(@)
```

## Bundled Examples

jpx ships with several example `.jpx` files you can reference or copy:

| File | Description |
|------|-------------|
| `examples/nlp.jpx` | Text processing and NLP pipelines |
| `examples/hacker-news.jpx` | Hacker News API analysis queries |
| `examples/github.jpx` | GitHub API response processing |
| `examples/logs.jpx` | Structured log analysis |
| `examples/data-transforms.jpx` | Common ETL transformations |

Additional query libraries used in tests:

| File | Description |
|------|-------------|
| `crates/jpx/queries/user-summary.jpx` | User data aggregation |
| `crates/jpx/queries/server-stats.jpx` | Server metrics queries |
| `crates/jpx/queries/order-report.jpx` | Order data reporting |

## Best Practices

### Naming Conventions

Use clear, descriptive names:
- `active-users` not `au` or `query1`
- `errors-by-service` not `err-svc`
- Use kebab-case for consistency

### Add Descriptions

Always add `-- :desc` lines. They show up in `--list-queries` and help others (and future you) understand what each query does.

### Organize by Domain

Group related queries into domain-specific libraries:
- `nlp.jpx` - Text processing pipelines
- `api.jpx` - API response transformations
- `logs.jpx` - Log analysis queries
- `etl.jpx` - Data transformation queries

### Version Control

Query libraries are plain text files - perfect for git:
- Track changes to important queries
- Review query changes in PRs
- Share queries across your team

### Validate in CI

Add query validation to your CI pipeline:

```yaml
# .github/workflows/validate.yml
- name: Validate query libraries
  run: |
    for f in queries/*.jpx; do
      jpx -Q "$f" --check || exit 1
    done
```

## CLI Reference

| Option | Description |
|--------|-------------|
| `-Q, --query-file <FILE>` | Load expression from file or query library |
| `--query <NAME>` | Select a named query from a .jpx library |
| `--list-queries` | List all queries in a .jpx file |
| `--check` | Validate all queries without running |

## Migration from Simple Files

If you have many simple query files, consolidate them:

```bash
# Before: multiple files
queries/
  active-users.txt
  admin-emails.txt
  user-stats.txt

# After: one library
queries/users.jpx
```

Just add `-- :name` headers to combine them:

```
-- :name active-users
users[?active].{name: name, email: email}

-- :name admin-emails
users[?role == `admin`].email

-- :name user-stats
{total: length(users), active: length(users[?active])}
```
