# jpx-mcp

[![Crates.io](https://img.shields.io/crates/v/jpx-mcp.svg)](https://crates.io/crates/jpx-mcp)

A [Model Context Protocol](https://modelcontextprotocol.io) server that gives AI assistants 34 tools for querying and transforming JSON with JMESPath, powered by [jpx-engine](https://crates.io/crates/jpx-engine) and its 490+ functions.

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

34 tools spanning evaluation, introspection, JSON utilities, an ephemeral process-scoped query store, and tool discovery, including `evaluate`, `batch_evaluate`, `evaluate_json`, `batch_evaluate_json`, `validate`, `explain`, `functions`, `describe`, `batch_describe`, `search`, `similar`, `format`, `diff`, `patch`, `merge`, `stats`, `paths`, and `inspect`. See the [MCP documentation](https://joshrotenberg.github.io/jpx/mcp/overview/) for the full list.

### Typed composition

Pass another tool's structured value directly to `evaluate_json` as
`{"input": <native JSON value>, "expression": "..."}`. Read the returned
`structuredContent.result` from the envelope `{"schema_version": 1, "result": ...}`.
`batch_evaluate_json` takes `expressions` instead and returns an ordered `results`
array; each entry has `expression` and exactly one of `result` (possibly null) or
`error`. Empty batches succeed; mixed batches preserve individual failures.
Both tools advertise `outputSchema` and provide equivalent JSON text for
text-only clients. Invalid requests and single evaluation failures set `isError`.

`input` is required and accepts objects, arrays, strings, numbers, booleans and
explicit null. Strings remain literal even if they look like JSON. Unknown
fields, including competing input modes, are rejected. The limits are 50 MiB
of compact serialized UTF-8 input (including quotes and escapes) and 1000 batch
expressions, inclusive. Transport has already allocated the input value; size
checking serializes into a byte counter without an extra input text buffer.
Evaluation can allocate more, and output size is not bounded. Responses contain
both text and structured data, so budget for both plus JSON-RPC framing.

Typed Rust/JSON transport preserves supported signed/unsigned 64-bit integers,
including `2^53+1`, and binary64 floats, not arbitrary decimal precision.
JavaScript `Number` rounding before transport cannot be repaired. Use original
JSON text with `evaluate`/`batch_evaluate`, or `evaluate_file`, when the client
cannot preserve those integers. Legacy tools retain their existing text-only
responses. Typed stored-query/utility parity and output budgets remain deferred;
use `inspect` for bounded structural previews.

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
