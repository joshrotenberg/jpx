# jpx-engine

[![Crates.io](https://img.shields.io/crates/v/jpx-engine.svg)](https://crates.io/crates/jpx-engine)
[![docs.rs](https://img.shields.io/docsrs/jpx-engine)](https://docs.rs/jpx-engine)

A protocol-agnostic JMESPath query engine with 490+ functions, the higher-level "brain" of the [jpx](https://github.com/joshrotenberg/jpx) ecosystem.

It wraps [jpx-core](https://crates.io/crates/jpx-core) with everything beyond basic compile-and-evaluate, so the CLI (`jpx`), the MCP server (`jpx-mcp`), and any future REST/gRPC adapters can be thin wrappers over a single engine.

## Install

```toml
[dependencies]
jpx-engine = "0.3"
serde_json = "1"
```

## Quick start

```rust
use jpx_engine::JpxEngine;
use serde_json::json;

let engine = JpxEngine::new();
let result = engine.evaluate("users[*].name", &json!({
    "users": [{ "name": "alice" }, { "name": "bob" }]
})).unwrap();
assert_eq!(result, json!(["alice", "bob"]));
```

## Capabilities

- **Evaluation** -- single, batch, and string-based evaluation with validation
- **Introspection** -- list, search (BM25), describe, and find-similar over the function registry
- **Discovery** -- cross-server tool discovery with a searchable index
- **Query store** -- named queries scoped to an engine instance
- **Configuration** -- layered `jpx.toml` discovery (cwd, home, XDG) with merge
- **JSON utilities** -- format, diff (RFC 6902), patch, merge (RFC 7396), stats, paths, keys

## Features

All optional (default is none):

- `arrow` -- Apache Arrow conversion for columnar data (used by the CLI for Parquet I/O)
- `let-expr` -- JEP-18 `let` expressions, forwarded to jpx-core
- `schema` -- derive `JsonSchema` on discovery types (used by the MCP server for tool schemas)

## License

Licensed under either of MIT or Apache-2.0 at your option.
