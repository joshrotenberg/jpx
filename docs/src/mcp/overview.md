# MCP Server Overview

jpx can run as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server, allowing AI assistants like Claude to use JMESPath for JSON querying and transformation.

## What is MCP?

MCP is an open protocol that enables AI assistants to interact with external tools and data sources. By running jpx as an MCP server, you give Claude (and other MCP-compatible assistants) the ability to:

- Query JSON data using JMESPath expressions
- Transform and manipulate JSON structures
- Use all 470+ extension functions
- Explore available functions and their documentation

## Why Use jpx with Claude?

When working with JSON data in Claude, jpx provides:

- **Precise queries**: Extract exactly the data you need
- **Complex transformations**: Reshape data structures on the fly
- **Powerful functions**: String manipulation, math, dates, hashing, and more
- **Consistent results**: Deterministic query execution

## Available Tools

The MCP server exposes **31 tools** organized by purpose:

### Query Execution

| Tool | Description |
|------|-------------|
| `evaluate` | Run a JMESPath expression against JSON input |
| `evaluate_file` | Query JSON files directly from disk |
| `batch_evaluate` | Run multiple expressions against the same input |
| `validate` | Check expression syntax without executing |
| `explain` | Break down an expression into steps with complexity rating |

### Function Discovery

| Tool | Description |
|------|-------------|
| `search` | Fuzzy search for functions by name, description, category, or signature |
| `similar` | Find functions related to a specified function |
| `suggest_function` | Describe a task in natural language and get ranked function suggestions |
| `functions` | List available functions (optionally filter by category) |
| `describe` | Get detailed info for a specific function |
| `batch_describe` | Get details for several functions in one call |
| `categories` | List all function categories |

### JSON Utilities

Tools for JSON manipulation (RFC 6902/7396):

| Tool | Description |
|------|-------------|
| `format` | Pretty-print JSON with configurable indentation |
| `diff` | Generate RFC 6902 JSON Patch between documents |
| `patch` | Apply RFC 6902 JSON Patch operations |
| `merge` | Apply RFC 7396 JSON Merge Patch |
| `keys` | Extract object keys (optionally recursive with dot notation) |
| `stats` | Analyze JSON structure (type, size, depth, field analysis) |
| `paths` | Extract all paths in dot notation (e.g., `users[0].name`) |

### Query Store

Ephemeral, process-scoped named queries for iterative development:

| Tool | Description |
|------|-------------|
| `define_query` | Store a named query for reuse |
| `get_query` | Retrieve a stored query by name |
| `delete_query` | Delete a stored query |
| `list_queries` | List all stored queries |
| `run_query` | Execute a stored query against JSON input |

### Multi-Server Discovery

Tools for semantic search across multiple MCP servers (see [Discovery](./discovery.md)):

| Tool | Description |
|------|-------------|
| `register_tools` | Register an MCP server's tools for BM25 indexing |
| `query_tools` | Semantic search across registered tools |
| `similar_tools` | Find tools related to a specific tool |
| `unregister_discovery` | Remove a server from the registry |
| `list_discovery_servers` | List registered servers |
| `list_discovery_categories` | List tool categories across servers |

### Server Info

| Tool | Description |
|------|-------------|
| `engine_info` | Get engine version, function count, categories, and optional discovery schema |

## Typical Workflow for AI Agents

1. **Analyze data**: Use `stats` and `paths` to understand the JSON structure
2. **Discover functions**: Use `search` to find relevant functions, `similar` to explore alternatives
3. **Build query**: Use `validate` to check syntax, `explain` to understand complex expressions
4. **Execute**: Use `evaluate` or `batch_evaluate` to run queries
5. **Iterate**: Use `define_query` and `run_query` to save and refine queries
6. **Transform**: Use `diff`, `patch`, or `merge` for modifications

## Getting Started

See [Setup](./setup.md) to configure jpx as an MCP server for Claude Desktop.
