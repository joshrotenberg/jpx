# Available Tools

The jpx MCP server provides 26 tools organized into five categories.

## Function Discovery

Tools for exploring the 400+ JMESPath extension functions.

### search

Fuzzy search for functions by name, description, category, or signature.

```json
{
  "query": "string manipulation",
  "limit": 10
}
```

### similar

Find functions related to a specified function.

```json
{
  "function": "upper",
  "limit": 5
}
```

Returns functions with similar purpose (e.g., `lower`, `capitalize`, `title_case`).

### functions

List available functions, optionally filtered by category.

```json
{
  "category": "string"
}
```

### describe

Get detailed documentation for a specific function.

```json
{
  "function": "pad_left"
}
```

Returns signature, description, parameters, return type, and examples.

### categories

List all function categories with tool counts.

```json
{}
```

## Data Analysis

Tools for understanding JSON structure before writing queries.

### stats

Analyze JSON structure and statistics.

```json
{
  "json": {"users": [{"name": "Alice"}, {"name": "Bob"}]}
}
```

Returns type, size, depth, and field analysis.

### paths

Extract all paths in a JSON document using dot notation.

```json
{
  "json": {"user": {"profile": {"name": "Alice"}}}
}
```

Returns: `["user", "user.profile", "user.profile.name"]`

### keys

Extract object keys, optionally recursive with dot notation.

```json
{
  "json": {"a": {"b": 1}},
  "recursive": true
}
```

## Query Execution

Tools for running JMESPath expressions.

### evaluate

Run a JMESPath expression against JSON input.

```json
{
  "expression": "users[?age > `30`].name",
  "json": {"users": [{"name": "Alice", "age": 35}, {"name": "Bob", "age": 25}]}
}
```

Returns: `["Alice"]`

### evaluate_file

Query a JSON file directly from disk. Useful when the file is too large to pass inline or when you want to avoid serialization overhead.

```json
{
  "expression": "length(items)",
  "path": "/path/to/data.json"
}
```

### batch_evaluate

Run multiple expressions against the same input.

```json
{
  "expressions": ["length(@)", "keys(@)", "@.name"],
  "json": {"name": "test", "value": 42}
}
```

### validate

Check expression syntax without executing.

```json
{
  "expression": "users[?active].name"
}
```

Returns validation status and any syntax errors.

## JSON Utilities

Tools for JSON manipulation following RFC standards (RFC 6902 for JSON Patch, RFC 7396 for Merge Patch).

### format

Pretty-print JSON with configurable indentation.

```json
{
  "json": {"compact": true},
  "indent": 2
}
```

### diff

Generate RFC 6902 JSON Patch between two documents.

```json
{
  "source": {"a": 1},
  "target": {"a": 2, "b": 3}
}
```

Returns patch operations: `[{"op": "replace", "path": "/a", "value": 2}, {"op": "add", "path": "/b", "value": 3}]`

### patch

Apply RFC 6902 JSON Patch operations.

```json
{
  "json": {"a": 1},
  "operations": [{"op": "add", "path": "/b", "value": 2}]
}
```

### merge

Apply RFC 7396 JSON Merge Patch.

```json
{
  "json": {"a": 1, "b": 2},
  "patch": {"b": null, "c": 3}
}
```

Returns: `{"a": 1, "c": 3}` (null removes keys)

## Multi-Server Discovery

Tools for semantic search across multiple MCP servers. See [Multi-Server Discovery](./discovery.md) for full documentation.

### register_discovery

Register an MCP server's tools for BM25 indexing using the full discovery spec.

```json
{
  "spec": {
    "server": {"name": "myserver", "version": "1.0.0"},
    "tools": [
      {"name": "tool_one", "description": "Does something useful", "category": "utils"}
    ]
  }
}
```

### register_tools_simple

Quick registration without the full schema. Just provide server name and tools array.

```json
{
  "server_name": "myserver",
  "tools": [
    {"name": "backup_db", "description": "Backup the database", "tags": ["database", "backup"]},
    {"name": "restore_db", "description": "Restore from backup", "tags": ["database", "restore"]}
  ]
}
```

### query_tools

Semantic search across all registered tools.

```json
{
  "query": "backup database",
  "top_k": 5
}
```

Returns ranked results with BM25 scores.

### similar_tools

Find tools related to a specific tool.

```json
{
  "tool_id": "myserver:tool_one",
  "top_k": 5
}
```

### list_discovery_servers

List all registered servers with tool counts.

```json
{}
```

### list_discovery_categories

List tool categories across all registered servers.

```json
{}
```

### inspect_discovery_index

Get index statistics for debugging.

```json
{}
```

Returns server count, tool count, term count, and average document length.

### unregister_discovery

Remove a server from the registry.

```json
{
  "server": "myserver"
}
```

### get_discovery_schema

Get the JSON schema for registration.

```json
{}
```
