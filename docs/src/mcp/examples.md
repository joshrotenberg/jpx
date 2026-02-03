# MCP Examples

This page demonstrates common workflows using the jpx MCP server with Claude or other AI assistants.

## Data Exploration

### Understanding Unknown JSON

When you receive unfamiliar JSON data, start by exploring its structure:

```
User: I have this API response, can you help me understand it?

Claude: Let me analyze the structure first.

[Uses stats tool]
{
  "input": "{\"users\": [{\"id\": 1, \"profile\": {\"name\": \"Alice\", \"settings\": {\"theme\": \"dark\"}}}]}"
}

Result: Object with 1 key, depth 4, contains array of 1 object

[Uses paths tool]
{
  "input": "{\"users\": [{\"id\": 1, \"profile\": {\"name\": \"Alice\"}}]}",
  "include_types": true
}

Result:
  users: array
  users.0: object
  users.0.id: number
  users.0.profile: object
  users.0.profile.name: string
```

### Finding the Right Function

When you need to transform data but aren't sure which function to use:

```
User: I need to remove duplicate entries from an array

Claude: Let me search for relevant functions.

[Uses search tool]
{
  "query": "remove duplicates array unique"
}

Result: Found unique(), distinct(), dedupe() - all do similar things.

[Uses describe tool]
{
  "name": "unique"
}

Result: unique(array) - Returns array with duplicate elements removed, preserving order.
```

## Query Building

### Iterative Query Development

Build complex queries step by step:

```
User: Help me extract active premium users with their subscription dates

Claude: Let me build this query incrementally.

[Uses evaluate tool - Step 1: Filter active users]
{
  "expression": "users[?status == `active`]",
  "input": "{\"users\": [{\"name\": \"Alice\", \"status\": \"active\", \"plan\": \"premium\", \"since\": \"2024-01-15\"}, {\"name\": \"Bob\", \"status\": \"inactive\", \"plan\": \"free\", \"since\": \"2023-06-01\"}]}"
}

[Uses evaluate tool - Step 2: Add premium filter]
{
  "expression": "users[?status == `active` && plan == `premium`]",
  "input": "..."
}

[Uses evaluate tool - Step 3: Project fields]
{
  "expression": "users[?status == `active` && plan == `premium`].{name: name, member_since: since}",
  "input": "..."
}

Final query: users[?status == `active` && plan == `premium`].{name: name, member_since: since}
```

### Validating Expressions

Check syntax before running on large datasets:

```
[Uses validate tool]
{
  "expression": "users[?active].name | sort(@"
}

Result: Invalid - unclosed parenthesis at position 27

[Uses validate tool]
{
  "expression": "users[?active].name | sort(@)"
}

Result: Valid expression
```

## Batch Operations

### Multiple Extractions

Extract several pieces of information at once:

```
[Uses batch_evaluate tool]
{
  "input": "{\"items\": [1, 2, 3, 4, 5], \"metadata\": {\"count\": 5, \"type\": \"numbers\"}}",
  "expressions": [
    "length(items)",
    "sum(items)",
    "avg(items)",
    "metadata.type"
  ]
}

Result:
  length(items): 5
  sum(items): 15
  avg(items): 3
  metadata.type: "numbers"
```

## File Operations

### Working with Large Files

Query files directly without loading into memory:

```
[Uses evaluate_file tool]
{
  "file_path": "/path/to/large-dataset.json",
  "expression": "records[?timestamp > `2024-01-01`] | length(@)"
}

Result: 15234 records match the filter
```

## JSON Transformations

### Generating Patches

Track changes between document versions:

```
[Uses diff tool]
{
  "source": "{\"version\": 1, \"config\": {\"debug\": true, \"timeout\": 30}}",
  "target": "{\"version\": 2, \"config\": {\"debug\": false, \"timeout\": 60, \"retries\": 3}}"
}

Result:
[
  {"op": "replace", "path": "/version", "value": 2},
  {"op": "replace", "path": "/config/debug", "value": false},
  {"op": "replace", "path": "/config/timeout", "value": 60},
  {"op": "add", "path": "/config/retries", "value": 3}
]
```

### Applying Updates

Merge configuration updates:

```
[Uses merge tool]
{
  "input": "{\"database\": {\"host\": \"localhost\", \"port\": 5432}, \"cache\": {\"enabled\": true}}",
  "patch": "{\"database\": {\"host\": \"db.example.com\"}, \"cache\": null}"
}

Result:
{
  "database": {"host": "db.example.com", "port": 5432}
}
```

Note: Setting a key to `null` in merge patch removes it.

## Function Discovery Workflows

### Learning New Functions

Explore related functionality:

```
[Uses similar tool]
{
  "function": "snake_case"
}

Result:
  Same category: camel_case, kebab_case, pascal_case, title_case
  Similar signature: upper, lower, capitalize
  Related concepts: snake_keys, camel_keys (for object keys)
```

### Category Exploration

See all functions in a category:

```
[Uses functions tool]
{
  "category": "datetime"
}

Result:
  now() - Current timestamp
  format_date(date, format) - Format date string
  parse_date(string, format) - Parse date from string
  date_add(date, amount, unit) - Add time to date
  date_diff(date1, date2, unit) - Difference between dates
  ...
```

## Multi-Server Discovery

When working with multiple MCP servers, use discovery tools to find the right tool across all servers:

```
[Uses query_tools tool]
{
  "query": "send email notification",
  "top_k": 5
}

Result:
  1. notifications:send_email (score: 12.4)
  2. mailer:compose_message (score: 8.2)
  3. alerts:notify_user (score: 6.1)
```

See [Multi-Server Discovery](./discovery.md) for more details on setting up cross-server tool search.
