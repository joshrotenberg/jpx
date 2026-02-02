# Multi-Server Tool Discovery

jpx provides BM25-based semantic search for discovering tools across multiple MCP servers. This enables natural language queries like "how do I backup a database?" instead of scanning flat tool lists.

## The Problem

When an agent connects to multiple MCP servers, it has access to dozens or hundreds of tools. Finding the right tool for a task means:

- Scanning tool names hoping for a match
- Reading descriptions one by one
- Missing tools with different terminology ("backup" vs "snapshot" vs "export")

## The Solution: Registration + Search

jpx provides a discovery registry that indexes tools from any MCP server using BM25 (the algorithm behind search engines). The workflow:

1. **Register**: Agent registers each server's tools with jpx
2. **Search**: Agent queries across all registered tools semantically
3. **Discover**: BM25 finds relevant tools even with different wording

## Architecture

MCP servers don't communicate with each other - they only respond to the agent. The agent orchestrates registration:

```
┌─────────┐     ┌─────────────┐     ┌─────────────┐
│  Agent  │────▶│     jpx     │     │  redisctl   │
│         │────▶│  (registry) │     │  (65 tools) │
│         │────▶│             │     └─────────────┘
│         │     └─────────────┘     ┌─────────────┐
│         │────────────────────────▶│ other-mcp   │
│         │                         │  (N tools)  │
└─────────┘                         └─────────────┘
     │
     │  1. Connect to all servers
     │  2. Get tool metadata from each
     │  3. Register with jpx
     │  4. Query jpx for tool discovery
     ▼
```

## Discovery Tools

### Registration

| Tool | Description |
|------|-------------|
| `register_discovery` | Register an MCP server's tools |
| `unregister_discovery` | Remove a server from the registry |
| `get_discovery_schema` | Get the registration schema |

### Search

| Tool | Description |
|------|-------------|
| `query_tools` | BM25 semantic search across all registered tools |
| `similar_tools` | Find tools related to a specific tool |

### Browse

| Tool | Description |
|------|-------------|
| `list_discovery_servers` | List all registered servers |
| `list_discovery_categories` | List tool categories across servers |
| `inspect_discovery_index` | Debug index statistics |

## Registration Schema

Register a server's tools with `register_discovery`:

```json
{
  "spec": {
    "server": {
      "name": "redisctl",
      "version": "0.1.0",
      "description": "Redis management tools"
    },
    "tools": [
      {
        "name": "enterprise_database_create",
        "description": "Create a new Redis Enterprise database with specified configuration",
        "category": "enterprise",
        "subcategory": "database",
        "tags": ["database", "create", "write"],
        "examples": ["Create a 500MB database named 'cache'"]
      },
      {
        "name": "enterprise_database_backup",
        "description": "Trigger a backup snapshot of a Redis Enterprise database",
        "category": "enterprise",
        "subcategory": "database",
        "tags": ["database", "backup", "snapshot", "read"]
      }
    ],
    "categories": {
      "enterprise": {
        "description": "Redis Enterprise on-premises tools",
        "tool_count": 30
      }
    }
  },
  "replace": false
}
```

### Minimal Schema

Only `server.name` and `tools[].name` are required:

```json
{
  "spec": {
    "server": { "name": "myserver" },
    "tools": [
      { "name": "tool_one" },
      { "name": "tool_two" }
    ]
  }
}
```

### Full Schema

For best search results, include rich metadata:

- `description`: Natural language description (indexed for search)
- `category` / `subcategory`: For browsing and filtering
- `tags`: Keywords that improve search relevance
- `examples`: Usage examples (indexed for search)
- `inputSchema`: Parameter documentation

## Searching

### Semantic Search

```json
// query_tools
{
  "query": "backup database",
  "top_k": 5
}
```

Returns ranked results with scores:

```json
[
  {
    "tool_id": "redisctl:enterprise_database_backup",
    "server": "redisctl",
    "name": "enterprise_database_backup",
    "description": "Trigger a backup snapshot of a Redis Enterprise database",
    "score": 8.42,
    "category": "enterprise"
  },
  {
    "tool_id": "redisctl:enterprise_database_export",
    "server": "redisctl",
    "name": "enterprise_database_export",
    "description": "Export database data to RDB file",
    "score": 5.17,
    "category": "enterprise"
  }
]
```

### Similar Tools

Find tools related to one you know:

```json
// similar_tools
{
  "tool_id": "redisctl:enterprise_database_delete",
  "top_k": 5
}
```

Returns tools with similar descriptions/tags - useful for finding alternatives or related operations.

## Agent Workflow

### Manual Registration

At session start, register each server:

```
1. Get tool list from redisctl (agent already has this via MCP)
2. Call jpx.register_discovery with redisctl's tools
3. Repeat for other MCP servers
4. Use query_tools for semantic search
```

### Automatic Registration (Recommended)

Add to your agent's system prompt or CLAUDE.md:

```markdown
## MCP Tool Discovery

At session start, if jpx and other MCP servers are available:

1. For each non-jpx MCP server, register its tools with jpx:
   - Extract tool name, description, and input schema
   - Call `mcp__jpx__register_discovery` with the server name and tool list

2. When searching for tools, use `mcp__jpx__query_tools` for semantic search
   instead of scanning the tool list manually.

3. Use `mcp__jpx__similar_tools` to find related tools when exploring options.
```

## Example: Multi-Server Setup

With jpx + redisctl + github-mcp:

```
// Register redisctl (65 tools)
register_discovery({
  spec: {
    server: { name: "redisctl", version: "0.1.0" },
    tools: [
      { name: "enterprise_database_create", description: "...", category: "enterprise" },
      { name: "enterprise_database_backup", description: "...", category: "enterprise" },
      // ... 63 more tools
    ]
  }
})

// Register github-mcp (20 tools)
register_discovery({
  spec: {
    server: { name: "github", version: "1.0.0" },
    tools: [
      { name: "create_issue", description: "Create a GitHub issue", category: "issues" },
      { name: "list_pull_requests", description: "List PRs for a repo", category: "pulls" },
      // ... 18 more tools
    ]
  }
})

// Now search across both!
query_tools({ query: "create new resource", top_k: 10 })
// Returns: enterprise_database_create, create_issue, enterprise_crdb_create, ...
```

## BM25 Scoring

The search uses BM25, a proven ranking algorithm that:

- **Weights rare terms higher**: "kubernetes" is more distinctive than "create"
- **Handles term frequency saturation**: Repeating a word doesn't linearly increase relevance
- **Normalizes by document length**: Short and long descriptions scored fairly

This means "backup database" finds tools mentioning "snapshot", "export", "persist" if those terms appear in similar contexts across the corpus.

## Session Scope

The registry is **session-scoped** (in-memory):

- Fresh index each agent session
- No persistence across sessions
- Cheap to rebuild from source

This is intentional - tool metadata changes, and re-registration ensures the index matches current reality.

## Inspecting the Index

Debug with `inspect_discovery_index`:

```json
{
  "server_count": 2,
  "tool_count": 85,
  "term_count": 342,
  "avg_doc_length": 12.4,
  "servers": ["redisctl", "github"]
}
```

List categories with `list_discovery_categories`:

```json
{
  "enterprise": { "tool_count": 30, "servers": ["redisctl"] },
  "cloud": { "tool_count": 25, "servers": ["redisctl"] },
  "issues": { "tool_count": 8, "servers": ["github"] },
  "pulls": { "tool_count": 6, "servers": ["github"] }
}
```
