# jpx

JMESPath CLI and tools with 400+ extended functions.

## Project Structure

- `crates/jpx/` - CLI tool with REPL and multiple output formats
- `crates/jpx-engine/` - Query engine with introspection and discovery
- `crates/jpx-server/` - MCP server for AI assistants
- `mock-mcp-server/` - Testing utility for MCP protocol

## Dependencies

This repo depends on:
- `jmespath_extensions` from crates.io (the function library)
- `tower-mcp` from crates.io (MCP server framework)

## Testing Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --all-features
cargo test --test '*' --all-features
```

## Key Files

- `Cargo.toml` - Workspace configuration with cargo-dist metadata
- `release-plz.toml` - Release automation configuration
- `.github/workflows/` - CI/CD pipelines
