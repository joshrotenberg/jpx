# Available Tools

The jpx MCP server provides **34 tools** organized into seven categories.

## Server Info

### engine_info

Get information about the jpx engine including version, mode, function count,
effective filesystem policy, and ephemeral process state.

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
  "version": "0.5.0",
  "strict_mode": false,
  "function_count": 498,
  "category_count": 33,
  "filesystem": {
    "evaluate_file": {
      "mode": "allowed_roots",
      "allowed_roots": ["/srv/json-data"]
    }
  }
}
```

Set `include_schema` to get the discovery registration JSON schema, or `include_index_stats` for discovery index statistics.

## Function Discovery

Tools for exploring the 470+ JMESPath extension functions.

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

### suggest_function

Describe a task in natural language and get ranked function suggestions with relevance notes.

```json
{
  "task": "remove duplicate values from an array",
  "limit": 5
}
```

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

### evaluate_json

Evaluate a native JSON value directly. Only the required `input` and `expression`
fields are accepted. `input` can be an object, array, string, number, boolean or
explicit null; omitting it is an error. Strings are always literal, including
strings containing JSON text. `input_text`, `file_path` and other unknown fields
are errors; use the legacy text/file tools for those input modes.

```json
{
  "input": {"users": [{"name": "Alice", "age": 35}, {"name": "Bob", "age": 25}]},
  "expression": "users[?age > `30`].name"
}
```

Returns `{"schema_version": 1, "result": ["Alice"]}` as MCP `structuredContent`
and equivalent JSON text. The advertised `outputSchema` requires the version-1
object envelope and allows **any JSON value** in `result`, including scalar or
null results. Invalid requests, limit violations and single evaluation failures
return `isError: true` with corrective text, without a success envelope.

### batch_evaluate_json

Accepts only required native JSON `input` and an `expressions` array of strings.
Expressions run independently in input order using the same engine semantics
as `batch_evaluate`.

```json
{"input": {"name": "Alice"}, "expressions": ["name", "missing", "[", "name"]}
```

The version-1 envelope contains `results`. Each entry has `expression` and exactly
one of `result` (including explicit null) or a nonempty `error` string. For this
example, the ordered outcomes are `"Alice"`, successful null, a syntax error,
and `"Alice"`. The schema prohibits entries with both or neither outcome field.
A completed mixed batch is a successful tool call; `expressions: []` returns
`{"schema_version": 1, "results": []}`. Request/limit failures set `isError: true`.
The entire envelope appears in both `structuredContent` and equivalent JSON text.

#### Typed limits and precision

Both typed tools accept at most **52,428,800 bytes (50 MiB)** of compact serialized
UTF-8 `input`; the exact boundary is accepted. Count JSON punctuation, quotes and
escapes, and the UTF-8 bytes of Unicode characters, not character count or the
size of the full request. Batches allow at most **1000 expressions**, inclusive.
Transport has already allocated the received value. The size check serializes
into a byte counter without building an extra input text buffer; it is not
zero-copy or serialization-free. No result-size or evaluation-allocation bound
is introduced. The complete serialized `CallToolResult` includes both escaped
text and structured data, plus JSON-RPC framing at the transport layer.

Supported signed/unsigned 64-bit integers, including `2^53+1`, `i64::MIN/MAX`
and `u64::MAX`, round-trip through typed Rust/JSON transport. Floats use binary64,
not arbitrary decimal precision. A JavaScript `Number` may round a large integer
before transport; jpx cannot repair that. Retain original JSON text and use
`evaluate`/`batch_evaluate`, or `evaluate_file`, for exact input when the client
cannot preserve integers in native values.

These are the only typed entry points in this slice. Existing evaluation,
stored-query and utility tools keep their existing text-only results. Typed
stored-query/utility parity and output budgets remain deferred from #261;
bounded structural inspection belongs to #262.

### evaluate_file

Query a JSON file directly from disk. More efficient than passing large JSON
content through the protocol. The path must be absolute, permitted by the
effective filesystem policy, valid UTF-8, and at most 50 MiB. HTTP disables
file access by default; pass one or more `--allow-root <DIRECTORY>` options to
enable it for specific canonicalized directory trees. Stdio remains
unrestricted when no roots are supplied for backward compatibility. Use
`engine_info` to inspect the active mode and allowed roots.

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

### inspect

Bounded structural preview of a JSON document. Returns RFC 6901 JSON Pointer
paths with their JSON type and depth, never leaf values. Use it on large or
unfamiliar documents before `paths`, which is exhaustive.

```json
{
  "input": "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}",
  "max_depth": 3,
  "max_paths": 100,
  "array_sample": 1,
  "max_bytes": 16384
}
```

| Limit | Default | Range | Meaning |
|-------|---------|-------|---------|
| `max_depth` | 3 | 0-32 | Deepest level returned (root is depth 0; `0` previews only the root) |
| `max_paths` | 100 | 1-1000 | Records returned, counting the root |
| `array_sample` | 3 | 0-100 | Leading elements visited per array (`0` omits all elements) |
| `max_bytes` | 16384 | 512-65536 | Size of the serialized tool result, including JSON text escaping and the content wrapper (JSON-RPC framing excluded) |

Omitted limits use their defaults. Explicit `null`, out-of-range, negative or
non-integer limits are rejected with the valid range and default, as is input
that is not valid JSON text.

The result is compact JSON text:

```json
{"complete":false,"returned_paths":4,
 "omitted":{"depth":false,"paths":false,"array_sample":true,"bytes":false},
 "limits":{"max_depth":3,"max_paths":100,"array_sample":1,"max_bytes":16384},
 "paths":[{"pointer":"","type":"object","depth":0,"length":1},
          {"pointer":"/users","type":"array","depth":1,"length":2,"truncated":"array_sample"},
          {"pointer":"/users/0","type":"object","depth":2,"length":1},
          {"pointer":"/users/0/name","type":"string","depth":3}]}
```

- Records are in preorder. Object members follow the parsed map's iteration
  order, which for this server is document (insertion) order; only the members
  returned are read. Arrays are sampled from the front. The same input always
  gives the same preview. `~` and `/` in keys are escaped as `~0` and `~1`; the
  root is `""` and an empty key is `"/"`.
- `complete` is true only when every node is in `paths`. `omitted` names each
  limit that actually dropped nodes; reaching a limit exactly at the last node
  is not an omission. `truncated` marks the node whose children were skipped by
  depth or sampling. `returned_paths` is exact; no totals are estimated.
- The walk stops as soon as a limit is hit. The input itself is still parsed in
  full (up to the 50 MiB inline cap), so parsing time and memory are not bounded
  by these limits.

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

Named queries for iterative development. They live in memory for the server process, are shared by every connected client, and disappear when the process exits.

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

List all named queries stored in this server process.

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
