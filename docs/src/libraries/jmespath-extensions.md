# jmespath-extensions

The core Rust library providing 400+ extended JMESPath functions.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
jmespath-extensions = "0.8"
```

## Quick Start

```rust
use jmespath_extensions::search;
use serde_json::json;

fn main() {
    let data = json!({"name": "hello world"});
    let result = search("upper(name)", &data).unwrap();
    assert_eq!(result, json!("HELLO WORLD"));
}
```

## Features

The library uses feature flags to control which function categories are included:

- **`full`** - All functions (default)
- **`core`** - String, array, object, math, type, utility (no external deps)
- Individual categories: `string`, `array`, `hash`, `datetime`, `regex`, etc.

```toml
# Minimal build with just core functions
jmespath-extensions = { version = "0.8", default-features = false, features = ["core"] }

# Just what you need
jmespath-extensions = { version = "0.8", default-features = false, features = ["string", "datetime"] }
```

## API Documentation

Full API documentation is available on docs.rs:

**[jmespath-extensions on docs.rs](https://docs.rs/jmespath-extensions)**

The rustdoc includes:
- All public functions with examples
- Function signatures and parameter types
- Category organization
- Feature flag requirements for each function
