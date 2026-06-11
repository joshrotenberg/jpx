# jpx-core

[![Crates.io](https://img.shields.io/crates/v/jpx-core.svg)](https://crates.io/crates/jpx-core)
[![docs.rs](https://img.shields.io/docsrs/jpx-core)](https://docs.rs/jpx-core)

A complete JMESPath implementation for Rust, built directly on `serde_json::Value`, with 490+ extension functions beyond the standard specification.

This is a from-scratch implementation with no dependency on the unmaintained `jmespath` crate. It provides the parser, interpreter, runtime, and function registry used across the [jpx](https://github.com/joshrotenberg/jpx) ecosystem.

## Install

```toml
[dependencies]
jpx-core = "0.2"
serde_json = "1"
```

## Quick start

```rust
use jpx_core::compile;
use serde_json::json;

let expr = compile("foo.bar").unwrap();
let data = json!({ "foo": { "bar": true } });
assert_eq!(expr.search(&data).unwrap(), json!(true));
```

## Highlights

- Works natively with `serde_json::Value` -- no `Variable` type or conversion overhead
- 26 standard JMESPath built-ins plus 490+ extensions across 32 categories
- `functions.toml` is the single source of truth for function metadata; the registry, documentation, and compliance tests are generated from it at build time
- JEP-18 `let` expressions (`let $x = expr in body`)

## Features

- `extensions` (default) -- the 490+ extension functions
- `let-expr` (default) -- JEP-18 `let` expression support

Build with `default-features = false` for a lean, spec-only JMESPath engine.

## License

Licensed under either of MIT or Apache-2.0 at your option.
