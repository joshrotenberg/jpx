# Setup with Claude Desktop

Configure jpx-mcp as an MCP server for Claude Desktop.

## Prerequisites

1. Install jpx-mcp:
   ```bash
   cargo install jpx-mcp
   ```
   Or use Docker (no installation required).

2. Have Claude Desktop installed

## Configuration

### Docker (Recommended)

The simplest way to run jpx as an MCP server using the dedicated server image:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/joshrotenberg/jpx-mcp"]
    }
  }
}
```

Available for `linux/amd64` and `linux/arm64`.

### macOS

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx-mcp"
    }
  }
}
```

If jpx-mcp is not in your PATH, use the full path:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "/Users/yourname/.cargo/bin/jpx-mcp"
    }
  }
}
```

### Windows

Edit `%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "C:\\path\\to\\jpx-mcp.exe"
    }
  }
}
```

## Verify Setup

1. Restart Claude Desktop
2. Look for the jpx tools in Claude's tool list
3. Try a simple query:

```
User: I have this JSON: {"users": [{"name": "alice"}, {"name": "bob"}]}
      Get all the names.

Claude: [Uses jpx.evaluate]
        Result: ["alice", "bob"]
```

## Strict Mode

To use only standard JMESPath functions (no extensions):

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx-mcp",
      "args": ["--strict"]
    }
  }
}
```

Or with Docker:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "ghcr.io/joshrotenberg/jpx-mcp", "--strict"]
    }
  }
}
```

## Troubleshooting

### jpx-mcp not found

Make sure jpx-mcp is in your PATH, or use the full path in the config.

### Tools not appearing

1. Check the config file syntax (must be valid JSON)
2. Restart Claude Desktop completely
3. Check Claude Desktop logs for errors

### Permission errors on file access

The `evaluate_file` tool has security restrictions. It only allows access to files in safe directories (not system paths).
