# jpx MCP Tool Reference

Complete parameter reference for all 30 tools.

## Evaluation Tools

### evaluate
Evaluate a JMESPath expression against JSON input.
- `expression` (string, required): JMESPath expression
- `input` (string, required): JSON input as string

### evaluate_file
Evaluate a JMESPath expression against a JSON file on disk.
- `expression` (string, required): JMESPath expression
- `file_path` (string, required): Absolute path to JSON file (max 50MB)

### batch_evaluate
Evaluate multiple expressions against the same JSON input.
- `expressions` (array of strings, required): List of JMESPath expressions
- `input` (string, required): JSON input as string

Returns an array of results, one per expression.

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

## Query Store Tools

### define_query
Store a named query for the session.
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
