# Libraries

jpx is built on a stack of Rust crates that can be used independently in your own projects.

## jpx-core

Self-contained JMESPath implementation with 490+ extension functions. Provides the parser, runtime, and function registry used by all other jpx crates. This is a from-scratch implementation with no upstream dependency on the unmaintained `jmespath` crate.

- Uses `serde_json::Value` natively -- no type conversion overhead
- 26 standard JMESPath built-in functions + 490+ extensions across 32 categories
- `functions.toml` is the single source of truth for function metadata
- Optional `let-expr` feature for JMESPath `let` expressions

[crates.io](https://crates.io/crates/jpx-core) | [docs.rs](https://docs.rs/jpx-core)

## jpx-engine

Query engine with introspection, function discovery, configuration, and query store. Wraps jpx-core with higher-level features for building tools and services.

- Single, batch, and streaming evaluation
- Function introspection with BM25 search indexing
- Session-scoped query store for named queries
- JSON utilities: format, diff (RFC 6902), patch, merge (RFC 7396), stats, paths
- Layered configuration via `jpx.toml` with discovery (cwd, home, XDG)

[crates.io](https://crates.io/crates/jpx-engine) | [docs.rs](https://docs.rs/jpx-engine)

## jpx-mcp

Model Context Protocol server exposing 31 JMESPath tools to AI assistants, built on jpx-engine. Run it as a binary or embed the router in your own transport.

- 31 tools across evaluation, introspection, JSON utilities, query store, and discovery
- Install with `cargo install jpx-mcp`, or run the `ghcr.io/joshrotenberg/jpx-mcp` image
- See the [MCP Server](mcp/overview.md) section for setup and the full tool list

[crates.io](https://crates.io/crates/jpx-mcp) | [docs.rs](https://docs.rs/jpx-mcp)

## Python Bindings

Use jpx functions in Python via the `jpx` package on PyPI.

- Module-level functions: `search()`, `compile()`, `validate()`, `list_functions()`
- `CompiledExpression` class for reusing compiled expressions
- `JpxEngine` class wrapping the full engine

[PyPI](https://pypi.org/project/jpx/)
