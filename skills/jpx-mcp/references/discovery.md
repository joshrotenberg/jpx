# Cross-Server Tool Discovery

jpx-mcp includes a tool discovery registry that lets you search across tools from multiple MCP servers.

## Registering Tools

### Simplified Registration

For quick registration with minimal metadata:

```json
{
  "server_name": "my-server",
  "version": "1.0",
  "tools": [
    {"name": "search_docs", "description": "Search documentation", "tags": ["search", "docs"]},
    {"name": "create_issue", "description": "Create a GitHub issue", "tags": ["github", "issues"]}
  ]
}
```

### Full Spec Registration

For detailed registration with parameters, examples, and categories:

```json
{
  "spec": {
    "server": {"name": "my-server", "version": "1.0", "description": "My MCP server"},
    "tools": [
      {
        "name": "search_docs",
        "description": "Full-text search across documentation",
        "category": "search",
        "tags": ["search", "docs", "full-text"],
        "params": [
          {"name": "query", "description": "Search query", "required": true},
          {"name": "limit", "description": "Max results", "required": false}
        ],
        "examples": [{"description": "Search for auth docs", "input": {"query": "authentication"}}]
      }
    ],
    "categories": {
      "search": {"description": "Search and discovery tools"}
    }
  }
}
```

### Replacing Registrations

By default, `replace: true` overwrites existing registrations for the same server. Set `replace: false` to error if already registered.

## Searching Tools

### query_tools

BM25 full-text search across all registered tools:

```json
{"query": "search documentation", "top_k": 5}
```

Searches across tool names, descriptions, tags, categories, and parameter descriptions. Returns ranked results with match scores.

### similar_tools

Find tools related to a known tool:

```json
{"tool_id": "my-server:search_docs", "top_k": 5}
```

Uses the tool's indexed content to find related tools across all servers.

## Managing the Registry

- `list_discovery_servers` -- see all registered servers with tool counts
- `list_discovery_categories` -- see categories across all servers
- `unregister_discovery` -- remove a server (e.g., before re-registering with updated tools)

## Use Cases

- **Multi-server environments:** Register tools from several MCP servers, then search across all of them to find the right tool for a task
- **Tool recommendations:** Use `similar_tools` to discover alternative tools you might not know about
- **Inventory:** `list_discovery_servers` gives a quick overview of all available capabilities
