# Available Tools

The jpx MCP server provides **29 tools** organized into seven categories.

## Server Info

### engine_info

Get information about the jpx engine including version, mode, function count, and session state.

```json
{
  "include_schema": false,
  "include_index_stats": false
}
```

Returns:
```json
{
  "name": "jpx-mcp",
  "version": "0.4.0",
  "strict_mode": false,
  "function_count": 395,
  "category_count": 31,
  "categories": ["Array", "Color", "Computing", ...]
}
```

Set `include_schema` to get the discovery registration JSON schema, or `include_index_stats` for discovery index statistics.

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
  "function": "upper"
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

Get detailed documentation for a specific function including signature, parameters, return type, and examples.

```json
{
  "name": "pad_left"
}
```

### batch_describe

Get details for several functions in one call. Returns one `{name, detail}` entry per requested name, in order; unknown names yield a `null` detail rather than failing the whole batch.

```json
{
  "names": ["pad_left", "split", "join"]
}
```

### categories

List all function categories with function counts.

```json
{}
```

## Query Execution

Tools for running JMESPath expressions.

### evaluate

Run a JMESPath expression against JSON input.

```json
{
  "expression": "users[?age > `30`].name",
  "input": "{\"users\": [{\"name\": \"Alice\", \"age\": 35}, {\"name\": \"Bob\", \"age\": 25}]}"
}
```

Returns: `["Alice"]`

### evaluate_file

Query a JSON file directly from disk. More efficient than passing large JSON content through the protocol.

```json
{
  "expression": "length(items)",
  "file_path": "/path/to/data.json"
}
```

### batch_evaluate

Run multiple expressions against the same input. Parses the input once and runs all expressions.

```json
{
  "expressions": ["length(@)", "keys(@)", "@.name"],
  "input": "{\"name\": \"test\", \"value\": 42}"
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

### explain

Break down a JMESPath expression into steps. Returns node types, descriptions, functions used, and a complexity rating. Also works for invalid expressions (returns the parse error).

```json
{
  "expression": "users[?age > `30`].name | sort(@)"
}
```

## Data Analysis

Tools for understanding JSON structure before writing queries.

### stats

Analyze JSON structure and statistics.

```json
{
  "input": "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}"
}
```

Returns type, size, depth, and field analysis.

### paths

Extract all paths in a JSON document using dot notation.

```json
{
  "input": "{\"user\": {\"profile\": {\"name\": \"Alice\"}}}",
  "include_types": true,
  "include_values": false
}
```

### keys

Extract object keys, optionally recursive with dot notation.

```json
{
  "input": "{\"a\": {\"b\": 1}}",
  "recursive": true
}
```

## JSON Utilities

Tools for JSON manipulation following RFC standards (RFC 6902 for JSON Patch, RFC 7396 for Merge Patch).

### format

Pretty-print JSON with configurable indentation. Use `indent: 0` for compact output.

```json
{
  "input": "{\"compact\":true}",
  "indent": 2
}
```

### diff

Generate RFC 6902 JSON Patch between two documents.

```json
{
  "source": "{\"a\": 1}",
  "target": "{\"a\": 2, \"b\": 3}"
}
```

Returns patch operations: `[{"op": "replace", "path": "/a", "value": 2}, {"op": "add", "path": "/b", "value": 3}]`

### patch

Apply RFC 6902 JSON Patch operations.

```json
{
  "input": "{\"a\": 1}",
  "patch": "[{\"op\": \"add\", \"path\": \"/b\", \"value\": 2}]"
}
```

### merge

Apply RFC 7396 JSON Merge Patch.

```json
{
  "input": "{\"a\": 1, \"b\": 2}",
  "patch": "{\"b\": null, \"c\": 3}"
}
```

Returns: `{"a": 1, "c": 3}` (null removes keys)

## Query Store

Session-scoped named queries for iterative development. Queries persist for the duration of the MCP session.

### define_query

Store a named JMESPath query for reuse. The query is validated before storing.

```json
{
  "name": "active_users",
  "expression": "users[?status == 'active'].name",
  "description": "Get names of active users"
}
```

### get_query

Retrieve a stored query by name.

```json
{
  "name": "active_users"
}
```

### run_query

Execute a stored query by name against JSON input.

```json
{
  "name": "active_users",
  "input": "{\"users\": [{\"name\": \"Alice\", \"status\": \"active\"}, {\"name\": \"Bob\", \"status\": \"inactive\"}]}"
}
```

### list_queries

List all named queries stored in this session.

```json
{}
```

### delete_query

Delete a stored query by name.

```json
{
  "name": "active_users"
}
```

## Multi-Server Discovery

Tools for semantic search across multiple MCP servers. See [Multi-Server Discovery](./discovery.md) for full documentation.

### register_tools

Register an MCP server's tools for BM25 indexing. Accepts either a full discovery spec (via `spec`) or a simplified format (via `server_name` + `tools`).

Full spec:

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

Simplified:

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

Semantic search across all registered tools using BM25.

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

### unregister_discovery

Remove a server from the registry.

```json
{
  "server_name": "myserver"
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
