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

# Enable evaluate_file beneath specific directories (repeatable)
jpx-mcp --transport http \
  --allow-root /srv/json-data \
  --allow-root /srv/reports
```

!!! warning
    HTTP has no built-in authentication or TLS. The default loopback bind is intended
    for local use. Put the server behind an authenticating TLS reverse proxy before
    exposing it to another machine. Origin validation remains enabled. `evaluate_file`
    is disabled for HTTP unless at least one `--allow-root` is provided.

### CLI Options

```
jpx-mcp [OPTIONS]

Options:
  -t, --transport <TRANSPORT>       Transport mode [default: stdio] [possible values: stdio, http]
      --strict                      Use standard JMESPath only (no extensions)
  -l, --log-level <LOG_LEVEL>       Log level [default: info]
      --host <HOST>                 HTTP host [default: 127.0.0.1]
  -p, --port <PORT>                 HTTP port [default: 3000]
      --request-timeout-secs <REQUEST_TIMEOUT_SECS>
                                      Request timeout in seconds [default: 30]
      --allow-root <DIRECTORY>      Allow evaluate_file beneath this directory (repeatable)
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

### Filesystem access policy

`evaluate_file` accepts an absolute path to a UTF-8 JSON file no larger than
50 MiB. Its default access depends on the transport:

| Configuration | Effective policy |
|---|---|
| stdio with no `--allow-root` | Unrestricted, for backward compatibility with trusted local clients |
| HTTP with no `--allow-root` | Disabled |
| either transport with one or more `--allow-root` options | Restricted to files beneath those roots |

Allowed roots are canonicalized at startup. Requested files are canonicalized
before the policy check, so a symlink inside an allowed root cannot escape to a
file outside it. Roots must already exist and must be directories. Call
`engine_info` to inspect the effective mode and canonical roots.

The allowed-root check is a canonical-path policy, not a capability sandbox
against concurrent filesystem mutation. Do not grant an allowed root that an
untrusted user can rewrite while the server is running; prefer read-only mounts
or directories writable only by the server operator.

For Docker, mount only the required data read-only and allow that container
directory explicitly:

```bash
docker run -i --rm \
  -v /absolute/host/data:/data:ro \
  ghcr.io/joshrotenberg/jpx-mcp \
  --allow-root /data
```

If a file is rejected, either choose a file beneath an existing allowed root or
restart `jpx-mcp` with another `--allow-root <DIRECTORY>` option. Filesystem
permissions still apply after the policy check.
