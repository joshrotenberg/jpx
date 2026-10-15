---
name: jpx-mcp
description: "Invoke jpx MCP server tools for JMESPath evaluation, function discovery, JSON utilities, and cross-server tool discovery. Use when evaluating expressions via MCP, exploring functions, analyzing JSON structure, or managing named queries."
license: MIT OR Apache-2.0
metadata:
  author: joshrotenberg
  version: "1.0"
compatibility: Requires jpx-mcp server (stdio or HTTP transport)
---

# jpx MCP Server Guide

The jpx MCP server exposes 34 tools for JMESPath evaluation, function discovery, JSON utilities, and more. It supports 470+ extension functions beyond the JMESPath specification.

## Tool Groups

### Evaluation (7 tools)

| Tool | Use when... |
|------|-------------|
| `evaluate` | Running a JMESPath expression against inline JSON text |
| `evaluate_json` | Running an expression against a native JSON value, with a typed result |
| `evaluate_file` | Querying a JSON file on disk (avoids passing large data through protocol) |
| `batch_evaluate` | Running multiple expressions against the same JSON text (parses once) |
| `batch_evaluate_json` | Running multiple expressions against a native JSON value, with typed results |
| `validate` | Checking if an expression is syntactically valid before running it |
| `explain` | Understanding what an expression does step by step |

**`evaluate` example:**
```json
{"expression": "users[?age > `30`].name", "input": "{\"users\":[{\"name\":\"alice\",\"age\":35}]}"}
```

**`batch_evaluate`** is more efficient than calling `evaluate` multiple times when you need several values from the same data -- it parses the input JSON once.

**`evaluate_json`** (`input`, `expression`) and **`batch_evaluate_json`** (`input`, `expressions`) take a native JSON value instead of encoded text:
```json
{"input": {"users": [{"name": "alice", "age": 35}]}, "expression": "users[?age > `30`].name"}
```
- Arguments must be a JSON object with only these named fields; positional arrays and extra fields (`input_text`, `file_path`) are errors.
- `input` is required and may be any JSON value. Explicit `null` is valid; omitting `input` is an error. Strings stay literal (`"[1,2]"` is a string), so use `evaluate` for JSON text.
- Success returns `{"schema_version": 1, "result": ...}` (or `"results": [...]`) as `structuredContent` plus equivalent JSON text.
- Batch entries keep expression order and have `expression` plus exactly one of `result` or `error`; a mixed batch is still a successful call.
- Limits: 50 MiB of compact serialized UTF-8 `input`, at most 1000 expressions. Output size and evaluation allocations are unbounded.
- Request errors, limit violations, and single-expression failures return `isError: true` with corrective text and no `structuredContent`.

**`explain`** returns a structured breakdown with node types, functions used, and complexity rating. Works on invalid expressions too (returns parse error details).

### Function Discovery (7 tools)

| Tool | Use when... |
|------|-------------|
| `functions` | Listing available functions, optionally filtered by category |
| `describe` | Getting detailed info on a specific function (signature, examples) |
| `batch_describe` | Getting details for several functions in one call |
| `categories` | Listing all function categories |
| `search` | Finding functions by keyword (fuzzy matching across names, descriptions) |
| `similar` | Finding functions related to a known function |
| `suggest_function` | Describing what you want in natural language |

**Discovery workflow:**
1. `search` with a keyword to find relevant functions
2. `describe` to get the full signature and examples
3. `evaluate` to try it out

**`suggest_function`** accepts plain English: "remove duplicates from an array" returns ranked suggestions with relevance explanations.

### JSON Utilities (8 tools)

| Tool | Use when... |
|------|-------------|
| `format` | Pretty-printing or compacting JSON (with configurable indent) |
| `diff` | Generating RFC 6902 JSON Patch between two documents |
| `patch` | Applying RFC 6902 JSON Patch operations |
| `merge` | Applying RFC 7396 JSON Merge Patch |
| `keys` | Extracting object keys (optionally recursive with dot notation) |
| `stats` | Analyzing JSON structure (type, size, depth, field analysis) |
| `paths` | Listing all paths in dot notation (optionally with types/values) |
| `inspect` | Previewing large or unfamiliar JSON with bounded traversal and output |

**Data exploration workflow:**
1. `stats` to understand overall structure
2. `inspect` for a bounded preview, or `paths` to see all available fields
3. `keys` with `recursive: true` for nested structure
4. `evaluate` with expressions informed by the structure

### Query Store (5 tools)

| Tool | Use when... |
|------|-------------|
| `define_query` | Saving a named query for reuse while this server process runs |
| `get_query` | Retrieving a stored query's expression |
| `delete_query` | Removing a stored query |
| `list_queries` | Showing all stored queries |
| `run_query` | Executing a stored query against new input |

**Iterative query building:**
1. `define_query` with an initial expression
2. `run_query` to test against data
3. Refine and `define_query` again (updates existing)
4. `run_query` with different inputs

Queries are validated on definition -- invalid expressions are rejected.

### Cross-Server Discovery (6 tools)

| Tool | Use when... |
|------|-------------|
| `register_tools` | Registering another MCP server's tools for searchable discovery |
| `query_tools` | Searching across all registered tools (BM25 full-text search) |
| `similar_tools` | Finding tools similar to a given tool |
| `unregister_discovery` | Removing a server from the discovery index |
| `list_discovery_servers` | Listing registered servers |
| `list_discovery_categories` | Listing tool categories across servers |

**Registration** supports two formats:
- **Full spec:** Detailed `DiscoverySpec` with params, examples, categories, tags
- **Simplified:** Just `server_name` + array of `{name, description, tags}`

**Search** uses BM25 indexing across tool names, descriptions, tags, categories, and parameters.

### Engine Info (1 tool)

| Tool | Use when... |
|------|-------------|
| `engine_info` | Checking server version, mode, function count, and process state |

Optionally include the discovery JSON schema (`include_schema: true`) or index statistics (`include_index_stats: true`).

## Common Workflows

### Explore unfamiliar JSON
```
stats(input) -> paths(input) -> evaluate(expression, input)
```

### Find the right function
```
search("group elements") -> describe("group_by") -> evaluate(...)
```

### Build complex queries iteratively
```
define_query("v1", "users[*].name") -> run_query("v1", data) ->
define_query("v1", "users[?active].name | sort(@)") -> run_query("v1", data)
```

### Compare JSON documents
```
diff(source, target) -> review patch ops -> patch(document, ops)
```

## Configuration

### Transport modes
- **stdio** (default): For local MCP client integration
- **HTTP**: For remote access (`--transport http --host 0.0.0.0 --port 3000`)

### Filesystem access
`evaluate_file` is unrestricted on stdio by default for backward compatibility
and disabled on HTTP by default. Pass `--allow-root <DIRECTORY>` repeatedly to
restrict either transport to canonicalized directory trees. Symlinks cannot
escape those roots during canonical-path validation. Treat allowed roots as a
trust boundary: use read-only mounts or directories that untrusted users cannot
rewrite concurrently. Use `engine_info` to inspect the effective policy.

### Strict mode
`--strict` disables all extension functions, limiting to the 26 standard JMESPath functions.

### Config file
The server discovers `jpx.toml` configuration (cwd -> home -> XDG) for function filtering, query libraries, and other engine settings.

## Related Skills

- [jmespath-query](../jmespath-query/SKILL.md) -- JMESPath expression syntax
- [jpx-functions](../jpx-functions/SKILL.md) -- 470+ extension functions
- [jpx-cli](../jpx-cli/SKILL.md) -- Command-line usage
