# jpx MCP Tool Reference

Complete parameter reference for all 34 tools.

## Evaluation Tools

### evaluate
Evaluate a JMESPath expression against JSON input.
- `expression` (string, required): JMESPath expression
- `input` (string, required): JSON input as string

### evaluate_file
Evaluate a JMESPath expression against a JSON file on disk.
- `expression` (string, required): JMESPath expression
- `file_path` (string, required): Absolute path to a JSON file readable by the server process (max 50 MiB)

### batch_evaluate
Evaluate multiple expressions against the same JSON input.
- `expressions` (array of strings, required): List of JMESPath expressions
- `input` (string, required): JSON input as string

Returns an array of results, one per expression.

### evaluate_json
Evaluate a JMESPath expression against native JSON input (no JSON text encoding). The legacy `evaluate` tool keeps its JSON-text contract.
- `input` (any JSON value, required): Native JSON document. Explicit `null` is a valid input; omitting the field is an error. Strings are literal, not encoded JSON.
- `expression` (string, required): JMESPath expression

Unknown fields (for example `input_text` or `file_path`) are rejected. Compact serialized UTF-8 input is limited to 50 MiB (quotes and escapes count).

Returns `{schema_version: 1, result: <value>}` as `structuredContent`, with equivalent JSON text in the text content. The result may be any JSON value, including `null`. There is no output-size bound. Evaluation failures are tool errors.

### batch_evaluate_json
Evaluate multiple expressions against the same native JSON input.
- `input` (any JSON value, required): Native JSON document. Explicit `null` is valid; omission is an error. Strings are literal.
- `expressions` (array of strings, required): At most 1000 expressions

Unknown fields are rejected. The same 50 MiB compact input limit as `evaluate_json` applies.

Returns `{schema_version: 1, results: [...]}` as `structuredContent`, with equivalent JSON text. Entries are in request order. Each has `expression` and exactly one of `result` or `error`; one failing expression does not fail the batch. There is no output-size bound.

### validate
Check if a JMESPath expression is syntactically valid.
- `expression` (string, required): JMESPath expression to validate

Returns `{valid: true}` or `{valid: false, error: "..."}`.

### explain
Break down a JMESPath expression into steps.
- `expression` (string, required): JMESPath expression to explain

Returns structured breakdown: node types, descriptions, functions used, depth, complexity rating. Works on invalid expressions (returns parse error).

## Function Discovery Tools

### functions
List available JMESPath functions.
- `category` (string, optional): Filter by category name (e.g., "String", "Math", "Array")

Returns array of `{name, signature, description}`.

### describe
Get detailed information about a specific function.
- `name` (string, required): Function name or alias

Returns `{name, signature, description, category, examples, aliases}` or error if unknown.

### batch_describe
Get detailed information about several functions in one call.
- `names` (array of strings, required): Function names to describe

Returns one `{name, detail}` entry per requested name, in order. Unknown names have a `null` detail.

### categories
List all function categories. No parameters.

Returns array of category name strings.

### search
Search for functions using fuzzy matching.
- `query` (string, required): Search query
- `limit` (number, optional, default 20): Max results

Searches across names, descriptions, categories, signatures, and aliases. Returns ranked results with match type and score.

### similar
Find functions similar to a specified function.
- `function` (string, required): Function name

Returns functions in the same category, with similar signatures, or related by description keywords.

### suggest_function
Suggest functions for a natural-language task description.
- `task` (string, required): Plain-English description (e.g., "remove duplicates")
- `limit` (number, optional, default 5): Max suggestions

Strips filler words and searches. Returns ranked suggestions with relevance explanations.

## JSON Utility Tools

### format
Pretty-print or compact JSON.
- `input` (string, required): JSON string
- `indent` (number, optional, default 2): Spaces for indentation (0 for compact)

### diff
Generate RFC 6902 JSON Patch between two documents.
- `source` (string, required): Source JSON document
- `target` (string, required): Target JSON document

Returns array of patch operations (add, remove, replace, move, copy, test).

### patch
Apply RFC 6902 JSON Patch to a document.
- `input` (string, required): JSON document
- `patch` (string, required): JSON Patch operations array

### merge
Apply RFC 7396 JSON Merge Patch to a document.
- `input` (string, required): JSON document
- `patch` (string, required): JSON Merge Patch document

### keys
Extract keys from a JSON object.
- `input` (string, required): JSON document
- `recursive` (boolean, optional, default false): If true, returns all nested keys in dot notation

### stats
Analyze JSON structure.
- `input` (string, required): JSON to analyze

Returns type, size, depth, field analysis (for arrays of objects), and type distribution.

### paths
Extract all paths from JSON in dot notation.
- `input` (string, required): JSON to extract paths from
- `include_types` (boolean, optional, default true): Include type for each path
- `include_values` (boolean, optional, default false): Include values for leaf paths

### inspect
Bounded structural preview of JSON text. Use before `paths` on large or unfamiliar documents.
- `input` (string, required): JSON input as text (max 50 MiB)
- `max_depth` (integer, optional, 0-32, default 3): Deepest level returned; the root is depth 0. `0` returns only the root.
- `max_paths` (integer, optional, 1-1000, default 100): Maximum path records, including the root
- `array_sample` (integer, optional, 0-100, default 3): Leading elements visited per array. `0` visits no array elements.
- `max_bytes` (integer, optional, 512-65536, default 16384): Byte budget for the whole serialized tool result, including JSON text escaping and the content wrapper

Limits must be whole numbers in range. Explicit `null`, negative, fractional, non-numeric and out-of-range values are rejected with a message naming the range and default; omit a limit to use its default. Invalid JSON text is rejected, and a `max_bytes` too small for the root record is an error.

The entire input is parsed and allocated before the walk, so the limits bound traversal and output, not parsing. Leaf values are never returned. Records are in deterministic preorder: object members in parsed document order, arrays prefix-sampled.

Returns JSON text (no `structuredContent`):
- `complete` (boolean): true only when every structural node is in `paths`
- `returned_paths` (number): count of records in `paths`
- `omitted` (object of booleans): `depth`, `paths`, `array_sample`, `bytes`; true only for limits that actually dropped nodes
- `limits` (object): the applied `max_depth`, `max_paths`, `array_sample`, `max_bytes`
- `paths` (array): records of `pointer` (RFC 6901 JSON Pointer, `""` is the root), `type` (object, array, string, number, boolean, null), `depth`, and for non-scalar nodes `length`. `truncated` is `depth` or `array_sample` when that node's children were skipped.

## Query Store Tools

### define_query
Store a named query in ephemeral process memory. The store is shared across clients and cleared on restart.
- `name` (string, required): Unique query name
- `expression` (string, required): JMESPath expression
- `description` (string, optional): What the query does

Expression is validated before storing. Overwrites existing query with same name.

### get_query
Retrieve a stored query.
- `name` (string, required): Query name

### delete_query
Delete a stored query.
- `name` (string, required): Query name

### list_queries
List all stored queries. No parameters.

### run_query
Execute a stored query against input.
- `name` (string, required): Query name
- `input` (string, required): JSON input as string

## Discovery Tools

### register_tools
Register an MCP server's tools for cross-server discovery.

**Full spec format:**
- `spec` (object): Complete DiscoverySpec with `server`, `tools`, `categories`
- `replace` (boolean, optional, default true): Replace existing registration

**Simplified format:**
- `server_name` (string): Server name
- `version` (string, optional): Server version
- `tools` (array): List of `{name, description, tags}`
- `replace` (boolean, optional, default true)

### query_tools
Search across all registered tools.
- `query` (string, required): Search query
- `top_k` (number, optional, default 10): Max results

### similar_tools
Find tools similar to a specified tool.
- `tool_id` (string, required): Tool ID in "server:tool_name" format
- `top_k` (number, optional, default 5): Max results

### unregister_discovery
Remove a server from the discovery index.
- `server_name` (string, required): Server name to remove

### list_discovery_servers
List all registered servers. No parameters.

### list_discovery_categories
List tool categories from registered servers. No parameters.

## Engine Info

### engine_info
Get engine information.
- `include_schema` (boolean, optional, default false): Include discovery JSON schema
- `include_index_stats` (boolean, optional, default false): Include BM25 index statistics
