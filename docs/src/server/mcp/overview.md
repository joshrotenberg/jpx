# MCP Server Overview

jpx can run as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server, allowing AI assistants like Claude to use JMESPath for JSON querying and transformation.

## What is MCP?

MCP is an open protocol that enables AI assistants to interact with external tools and data sources. By running jpx as an MCP server, you give Claude (and other MCP-compatible assistants) the ability to:

- Query JSON data using JMESPath expressions
- Transform and manipulate JSON structures
- Use all 400+ extension functions
- Explore available functions and their documentation

## Why Use jpx with Claude?

When working with JSON data in Claude, jpx provides:

- **Precise queries**: Extract exactly the data you need
- **Complex transformations**: Reshape data structures on the fly
- **Powerful functions**: String manipulation, math, dates, hashing, and more
- **Consistent results**: Deterministic query execution

## Available Tools

The MCP server exposes 25 tools organized by purpose:

### Function Discovery Tools

Tools for finding and exploring available JMESPath functions:

| Tool | Description |
|------|-------------|
| `search` | Fuzzy search for functions by name, description, category, or signature |
| `similar` | Find functions related to a specified function |
| `functions` | List available functions (optionally filter by category) |
| `describe` | Get detailed info for a specific function |
| `categories` | List all function categories |

### Multi-Server Tool Discovery

Tools for semantic search across multiple MCP servers (see [Discovery](./discovery.md)):

| Tool | Description |
|------|-------------|
| `register_discovery` | Register an MCP server's tools for indexing |
| `query_tools` | BM25 semantic search across registered tools |
| `similar_tools` | Find tools related to a specific tool |
| `list_discovery_servers` | List registered servers |
| `list_discovery_categories` | List tool categories across servers |
| `inspect_discovery_index` | Debug index statistics |
| `unregister_discovery` | Remove a server from the registry |
| `get_discovery_schema` | Get the registration schema |

### Data Analysis Tools

Tools for understanding JSON structure before querying:

| Tool | Description |
|------|-------------|
| `stats` | Analyze JSON structure (type, size, depth, field analysis) |
| `paths` | Extract all paths in dot notation (e.g., `users[0].name`) |
| `keys` | Extract object keys (optionally recursive with dot notation) |

### Query Tools

Tools for evaluating JMESPath expressions:

| Tool | Description |
|------|-------------|
| `evaluate` | Run a JMESPath expression against JSON input |
| `evaluate_file` | Query JSON files directly from disk |
| `batch_evaluate` | Run multiple expressions against the same input |
| `validate` | Check expression syntax without executing |

### JSON Utility Tools

Tools for JSON manipulation (RFC 6902/7396):

| Tool | Description |
|------|-------------|
| `format` | Pretty-print JSON with configurable indentation |
| `diff` | Generate RFC 6902 JSON Patch between documents |
| `patch` | Apply RFC 6902 JSON Patch operations |
| `merge` | Apply RFC 7396 JSON Merge Patch |

## Typical Workflow for AI Agents

1. **Analyze data**: Use `stats` and `paths` to understand the JSON structure
2. **Discover functions**: Use `search` to find relevant functions, `similar` to explore alternatives
3. **Build query**: Use `validate` to check syntax before executing
4. **Execute**: Use `evaluate` or `batch_evaluate` to run queries
5. **Transform**: Use `diff`, `patch`, or `merge` for modifications

## Getting Started

See [Setup](./setup.md) to configure jpx as an MCP server for Claude Desktop.
