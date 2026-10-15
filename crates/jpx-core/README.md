# jpx-core

[![Crates.io](https://img.shields.io/crates/v/jpx-core.svg)](https://crates.io/crates/jpx-core)
[![docs.rs](https://img.shields.io/docsrs/jpx-core)](https://docs.rs/jpx-core)

A complete JMESPath implementation for Rust, built directly on `serde_json::Value`, with 470+ extension functions beyond the standard specification.

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
- 26 standard JMESPath built-ins plus 470+ extensions across 32 extension categories
- `functions.toml` is the single source of truth for function metadata; the registry, documentation, and compliance tests are generated from it at build time
- JEP-18 `let` expressions (`let $x = expr in body`)

## Features

- `extensions` (default) -- all 470+ extension functions; enables every `ext-*` feature below
- `let-expr` (default) -- JEP-18 `let` expression support

Build with `default-features = false` for a lean, spec-only JMESPath engine.

### Extension categories

Each `ext-*` feature compiles a group of extension categories and the dependencies they need. To take a subset, turn off default features and list the ones you want:

```toml
[dependencies]
jpx-core = { version = "0.3", default-features = false, features = ["let-expr", "ext-core", "ext-strings", "ext-regex"] }
```

| Feature | Categories | Dependencies | Size |
|---|---|---|---|
| `ext-core` | array, object, math, type, utility, path, expression, duration, color, computing, units | heck | 760 KB |
| `ext-strings` | string | heck | 150 KB |
| `ext-text` | text, fuzzy, phonetic, multi-match, discovery | rust-stemmers, stop-words, unicode-normalization, strsim, rphonetic, aho-corasick | 1.3 MB |
| `ext-lang` | language | whatlang | 260 KB |
| `ext-regex` | regex | regex | 1.3 MB |
| `ext-validation` | validation | regex, base64, chrono | 1.4 MB |
| `ext-hash` | hash | md-5, sha1, sha2, hmac, crc32fast, hex | 70 KB |
| `ext-encoding` | encoding, url | base64, hex, urlencoding, url, form_urlencoded | 300 KB |
| `ext-ids` | uuid, rand, ids | uuid, rand, nanoid, ulid | 70 KB |
| `ext-time` | datetime | chrono, chrono-tz, interim, dateparser | 2.7 MB |
| `ext-geo` | geo | geoutils, geohash | 50 KB |
| `ext-semver` | semver | semver | 35 KB |
| `ext-net` | network | ipnetwork | 20 KB |
| `ext-jsonpatch` | jsonpatch | json-patch | 140 KB |
| `ext-format` | format | csv | 85 KB |

Size is the approximate growth of a stripped release binary that registers every function in the feature, measured on aarch64-apple-darwin. Features that share a dependency (for example `ext-regex` and `ext-validation`) cost less together than the sum of their rows; `extensions` adds about 5.5 MB in all.

`redact_keys` (object) and `redact_pattern` (string) take a regex pattern, so they are compiled only when `ext-regex` is also enabled. Without it, the rest of their categories are available and those two functions are absent.

`Category::is_available` reports whether a category is compiled in, and `Category::feature_name` names the feature that provides it. `FunctionRegistry` lists only compiled-in functions, so introspection matches what the runtime can call.

## License

Licensed under either of MIT or Apache-2.0 at your option.
