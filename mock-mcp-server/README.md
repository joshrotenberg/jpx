# Mock MCP Server

A configurable mock MCP server for testing jpx's discovery protocol. This server can be instantiated with different configurations to simulate multiple MCP servers registering with jpx.

## Usage

### Preset Configurations

Run with built-in presets:

```bash
# Redis-like management server (5 tools)
mock-mcp-server --preset redis

# PostgreSQL-like database server (4 tools)
mock-mcp-server --preset postgres

# GitHub-like API server (5 tools)
mock-mcp-server --preset github

# Minimal server with just echo/info tools
mock-mcp-server --preset minimal
```

### Custom Configuration

Run with a custom server name:

```bash
mock-mcp-server --name my-custom-server
```

Generate random tools for stress testing:

```bash
mock-mcp-server --name stress-test --random-tools 100
```

Load from a JSON config file:

```bash
mock-mcp-server --config server-config.json
```

Or provide inline JSON:

```bash
mock-mcp-server --json '{"name": "inline-server", "tools": [{"name": "my_tool", "description": "Does something"}]}'
```

## Server Config JSON Schema

```json
{
  "name": "my-server",
  "version": "1.0.0",
  "description": "My mock server",
  "tools": [
    {
      "name": "tool_name",
      "description": "What the tool does",
      "summary": "Short summary",
      "category": "category_name",
      "subcategory": "subcategory_name",
      "tags": ["read", "safe"],
      "params": [
        {
          "name": "param_name",
          "type": "string",
          "required": true,
          "description": "Parameter description"
        }
      ],
      "related": ["other_tool"],
      "aliases": ["alt_name"]
    }
  ],
  "categories": [
    {
      "name": "category_name",
      "description": "Category description",
      "subcategories": ["sub1", "sub2"]
    }
  ]
}
```

## Built-in Tools

Every mock server instance exposes these tools:

- `get_discovery_spec` - Returns the discovery spec for registering with jpx
- `echo` - Echo back input (for testing connectivity)
- `server_info` - Get server metadata

## Testing Discovery Protocol

1. Start jpx MCP server:
   ```bash
   jpx --mcp
   ```

2. Start one or more mock servers (in separate terminals):
   ```bash
   mock-mcp-server --preset redis
   mock-mcp-server --preset postgres
   ```

3. In your MCP client, call `get_discovery_spec` on each mock server to get the registration payload

4. Register each with jpx using `register_discovery`

5. Use jpx's `query_tools` to search across all registered servers

## Programmatic Usage

```rust
use mock_mcp_server::{MockMcpServer, MockToolConfig, ServerConfig, presets};

// Use a preset
let config = presets::redis_server();

// Or build custom
let config = ServerConfig::new("my-server")
    .with_version("1.0.0")
    .with_description("My custom server")
    .with_tool(
        MockToolConfig::full("create_thing", "Create a new thing")
            .with_category("things")
            .with_tags(vec!["write"])
    );

// Run the server
MockMcpServer::run(config).await?;
```

## Stress Testing

Generate many servers for load testing:

```rust
use mock_mcp_server::presets;

// Generate 10 servers with 20 tools each
let servers = presets::random_servers(10, 20);
```
