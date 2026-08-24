# jpx-mcp

[![Crates.io](https://img.shields.io/crates/v/jpx-mcp.svg)](https://crates.io/crates/jpx-mcp)

A [Model Context Protocol](https://modelcontextprotocol.io) server that gives AI assistants 31 tools for querying and transforming JSON with JMESPath, powered by [jpx-engine](https://crates.io/crates/jpx-engine) and its 490+ functions.

## Run it

```bash
cargo install jpx-mcp
# or run the prebuilt image:
docker run -i --rm ghcr.io/joshrotenberg/jpx-mcp
```

Register it with an MCP client (for example, Claude):

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx-mcp"
    }
  }
}
```

## Tools

31 tools spanning evaluation, introspection, JSON utilities, an ephemeral process-scoped query store, and tool discovery, including `evaluate`, `batch_evaluate`, `validate`, `explain`, `functions`, `describe`, `batch_describe`, `search`, `similar`, `format`, `diff`, `patch`, `merge`, `stats`, and `paths`. See the [MCP documentation](https://joshrotenberg.github.io/jpx/mcp/overview/) for the full list.

## Library use

The crate also exposes the router so you can embed it in your own transport:

```rust
use jpx_mcp::build_router;

// strict = false keeps the extension functions enabled
let router = build_router(false).expect("failed to build router");
```

`build_router` preserves unrestricted file access for backward compatibility
with trusted local/stdio clients. Embedders serving remote clients should choose
an explicit policy:

```rust
use jpx_mcp::{FileAccessPolicy, build_router_with_file_access};

let policy = FileAccessPolicy::restricted(["/srv/json-data"])
    .expect("allowed root must be an accessible directory");
let router = build_router_with_file_access(false, policy)
    .expect("failed to build router");
```

The `jpx-mcp` binary disables `evaluate_file` over HTTP unless at least one
repeatable `--allow-root <DIRECTORY>` option is supplied. Stdio retains the
historical unrestricted default when no roots are configured. The effective
policy is returned by `engine_info`. Allowed roots should be read-only or
otherwise protected from concurrent mutation by untrusted users.

## License

Licensed under either of MIT or Apache-2.0 at your option.
