# jpx-engine

Protocol-agnostic engine for JMESPath evaluation, function discovery, and JSON utilities. This is the core logic used by the MCP server and can be embedded in other applications.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
jpx-engine = "0.1"
```

## Quick Start

```rust
use jpx_engine::JpxEngine;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = JpxEngine::new();
    
    // Evaluate expressions with extended functions
    let data = json!({"items": [1, 2, 3, 4, 5]});
    let result = engine.evaluate("sum(items)", &data)?;
    println!("{}", result); // 15
    
    // Discover available functions
    let functions = engine.functions(Some("String"))?;
    println!("String functions: {}", functions.len());
    
    // Search functions by keyword
    let results = engine.search_functions("date")?;
    for r in results {
        println!("{}: {}", r.name, r.description);
    }
    
    Ok(())
}
```

## Features

### Evaluation
- `evaluate()` - Evaluate JMESPath expression against JSON
- `evaluate_str()` - Evaluate with JSON string input
- `batch_evaluate()` - Multiple expressions against same input
- `validate()` - Check expression syntax without executing

### Function Discovery
- `functions()` - List all functions, optionally by category
- `categories()` - List all function categories
- `describe_function()` - Get detailed function info
- `search_functions()` - Fuzzy search by keyword
- `similar_functions()` - Find related functions

### JSON Utilities
- `format()` - Pretty-print JSON
- `diff()` - Generate JSON Patch (RFC 6902)
- `patch()` - Apply JSON Patch
- `merge()` - Apply JSON Merge Patch (RFC 7396)
- `stats()` - Analyze JSON structure
- `paths()` - Extract all paths in dot notation
- `keys()` - Extract object keys

### Query Store
- `define_query()` - Store named queries
- `run_query()` - Execute stored queries
- `list_queries()` - List all stored queries

### Strict Mode

```rust
let engine = JpxEngine::strict();
// Only standard JMESPath functions available
```

## API Documentation

Full API documentation is available on docs.rs:

**[jpx-engine on docs.rs](https://docs.rs/jpx-engine)**

## Thread Safety

`JpxEngine` is `Send + Sync` and can be safely shared across threads. The query store uses interior mutability with `RwLock` for concurrent access.
