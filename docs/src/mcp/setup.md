# Setup

Configure jpx-mcp as an MCP server for Claude Desktop or as a standalone HTTP server.

## Installation

Install jpx-mcp:

```bash
cargo install jpx-mcp
```

Or use Docker (no installation required).

## Transport Modes

jpx-mcp supports two transport modes:

| Transport | Use Case |
|-----------|----------|
| `stdio` (default) | Claude Desktop and other MCP clients |
| `http` | Web applications, remote access |

## Claude Desktop Configuration

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

## HTTP Transport

Run jpx-mcp as an HTTP server for web applications or remote access:

```bash
# Start HTTP server on default port 3000
jpx-mcp --transport http

# Custom host and port
jpx-mcp --transport http --host 0.0.0.0 --port 8080

# With request timeout
jpx-mcp --transport http --request-timeout-secs 60
```

!!! warning
    HTTP has no built-in authentication or TLS. The default loopback bind is intended
    for local use. Put the server behind an authenticating TLS reverse proxy before
    exposing it to another machine. Origin validation remains enabled.

### CLI Options

```
jpx-mcp [OPTIONS]

Options:
  -t, --transport <TRANSPORT>       Transport mode [default: stdio] [possible values: stdio, http]
      --strict                      Use standard JMESPath only (no extensions)
  -l, --log-level <LOG_LEVEL>       Log level [default: info]
      --host <HOST>                 HTTP host [default: 127.0.0.1]
  -p, --port <PORT>                 HTTP port [default: 3000]
      --request-timeout-secs <SECS> Request timeout in seconds [default: 30]
  -h, --help                        Print help
  -V, --version                     Print version
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

`evaluate_file` accepts an absolute path up to 50 MiB and can read any file the
server process can read. jpx-mcp does not provide a filesystem sandbox. Run it
as a least-privileged user and only expose it to trusted clients. For Docker,
mount only the required data read-only:

```bash
docker run -i --rm \
  -v /absolute/host/data:/data:ro \
  ghcr.io/joshrotenberg/jpx-mcp
```
