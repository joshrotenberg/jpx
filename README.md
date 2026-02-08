# jpx

[![CI](https://github.com/joshrotenberg/jpx/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jpx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/jpx.svg)](https://crates.io/crates/jpx)
[![License](https://img.shields.io/crates/l/jpx.svg)](https://github.com/joshrotenberg/jpx#license)

JMESPath CLI and tools with 400+ extended functions - a powerful jq alternative.

This repository contains the jpx ecosystem:

| Package | Description |
|---------|-------------|
| **[jpx](crates/jpx/)** | CLI tool with REPL, multiple output formats |
| **[jpx-mcp](crates/jpx-mcp/)** | MCP server for AI assistants |
| **[jpx-engine](crates/jpx-engine/)** | Query engine with introspection and discovery |

## Quick Start

```bash
# Install
brew install joshrotenberg/brew/jpx
# or: cargo install jpx

# Use it
echo '{"name": "world"}' | jpx 'upper(name)'
# "WORLD"

curl -s https://api.github.com/users/octocat | jpx '{
  login: login,
  created: format_date(parse_date(created_at), `%B %Y`)
}'
# {"login": "octocat", "created": "January 2011"}
```

## Docker

```bash
# CLI
echo '{"name": "world"}' | docker run -i ghcr.io/joshrotenberg/jpx 'upper(name)'

# MCP Server
docker run -i --rm ghcr.io/joshrotenberg/jpx-mcp
```

## MCP Server

Give Claude (or any MCP client) the ability to query and transform JSON:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx-mcp"
    }
  }
}
```

**Tools:** `evaluate`, `batch_evaluate`, `validate`, `functions`, `describe`, `search`, `similar`, `format`, `diff`, `patch`, `merge`, `stats`, `paths`, `keys`

## Function Categories

The library provides 400+ functions across these categories:

| Category | Examples |
|----------|----------|
| **String** | `upper`, `lower`, `split`, `replace`, `camel_case`, `pad_left` |
| **Array** | `first`, `last`, `unique`, `chunk`, `zip`, `flatten`, `group_by` |
| **Math** | `round`, `sqrt`, `median`, `stddev`, `percentile` |
| **Date/Time** | `now`, `parse_date`, `format_date`, `date_add`, `date_diff` |
| **Hash** | `md5`, `sha256`, `hmac_sha256`, `crc32` |
| **Encoding** | `base64_encode`, `base64_decode`, `hex_encode`, `url_encode` |
| **Regex** | `regex_match`, `regex_extract`, `regex_replace` |
| **Geo** | `haversine`, `geo_distance_km`, `geo_bearing` |
| **Network** | `cidr_contains`, `is_private_ip`, `ip_to_int` |
| **JSON Patch** | `json_patch`, `json_merge_patch`, `json_diff` |
| **Fuzzy** | `levenshtein`, `jaro_winkler`, `soundex`, `metaphone` |
| **Expression** | `map_expr`, `filter_expr`, `sort_by_expr`, `group_by_expr` |

See the [documentation](https://joshrotenberg.github.io/jpx/) for the full function reference.

## Acknowledgments

- **[JMESPath](https://jmespath.org/)** - The query language specification
- **[jmespath.rs](https://crates.io/crates/jmespath)** - Rust implementation by [@mtdowling](https://github.com/mtdowling)
- **[jpx-core](crates/jpx-core/)** - JMESPath implementation with 400+ extension functions

## License

MIT or Apache-2.0

