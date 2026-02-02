# Introduction

This documentation covers the **JMESPath Extensions** project, which provides 400+ additional functions for [JMESPath](https://jmespath.org/) queries.

## Project Components

This project includes multiple components:

| Component | Description | Documentation |
|-----------|-------------|---------------|
| **[jmespath_extensions](https://crates.io/crates/jmespath_extensions)** | Rust library with 400+ extension functions | [docs.rs](https://docs.rs/jmespath_extensions) |
| **[jpx](https://crates.io/crates/jpx)** | Command-line tool for querying JSON | This site |
| **[jmespath-extensions-py](https://pypi.org/project/jmespath-extensions/)** | Python bindings | [Python section](./python/installation.md) |
| **MCP Server** | AI assistant integration | [MCP section](./mcp/overview.md) |

This documentation primarily focuses on **jpx**, the CLI tool. For using the Rust library directly in your code, see the [rustdoc documentation](https://docs.rs/jmespath_extensions).

## What is JMESPath?

[JMESPath](https://jmespath.org/) is a query language for JSON. It allows you to declaratively specify how to extract and transform elements from a JSON document.

## Why JMESPath Extensions?

While standard JMESPath is powerful, it's intentionally minimal with only 26 built-in functions. This project extends JMESPath with 400+ additional functions for:

- **String manipulation**: `upper`, `lower`, `split`, `replace`, `camel_case`, `snake_case`, and more
- **Array operations**: `unique`, `chunk`, `flatten`, `group_by`, `zip`, and more
- **Object manipulation**: `pick`, `omit`, `deep_merge`, `flatten_keys`, and more
- **Math & statistics**: `round`, `sqrt`, `median`, `stddev`, `percentile`, and more
- **Date/time**: `now`, `format_date`, `parse_date`, `date_add`, `date_diff`
- **Hashing & encoding**: `md5`, `sha256`, `base64_encode`, `hex_encode`
- **Fuzzy matching**: `levenshtein`, `jaro_winkler`, `soundex`
- **And much more**: geo functions, validation, regex, UUIDs, network utilities...

## Features

- **Rust Library**: Use the functions in your Rust applications ([docs.rs](https://docs.rs/jmespath_extensions))
- **Powerful CLI**: Query JSON from files, stdin, or inline with jpx
- **MCP Server**: Use jpx with AI assistants like Claude
- **Python Bindings**: Use the same 400+ functions in Python
- **Function Discovery**: Built-in help for all functions
- **Strict Mode**: Use only standard JMESPath for portable queries

## Quick Example

```bash
# Basic query
echo '{"users": [{"name": "alice"}, {"name": "bob"}]}' | jpx 'users[*].name'
# ["alice", "bob"]

# Using extension functions
echo '{"name": "hello world"}' | jpx 'upper(name)'
# "HELLO WORLD"

echo '{"items": [1, 2, 2, 3, 3, 3]}' | jpx 'unique(items)'
# [1, 2, 3]

echo '{"values": [10, 20, 30, 40, 50]}' | jpx 'median(values)'
# 30
```

## Getting Started

Ready to dive in? Start with [Installation](./getting-started/installation.md) to get jpx set up on your system.
