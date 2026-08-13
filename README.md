# jpx

[![CI](https://github.com/joshrotenberg/jpx/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jpx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/jpx.svg)](https://crates.io/crates/jpx)
[![License](https://img.shields.io/crates/l/jpx.svg)](https://github.com/joshrotenberg/jpx#license)

JMESPath CLI and tools with 490+ functions, including 470+ extensions.

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
  created: format_date(parse_date(created_at), '%B %Y')
}'
# {"login": "octocat", "created": "January 2011"}
```

## Where jpx fits

Use jpx for shell pipelines, JSONL/NDJSON streams, agent-driven JSON work, and
cases where the extraction itself should be a small, reviewable artifact. A
`.jpx` query library can hold named, multi-line queries and be validated in CI.

Use jq when you already know it and the query is straightforward. Use a general
programming language such as Python for control flow, joins across sources,
retries, subprocess orchestration, or anything else that makes a query language
fight the problem.

## Query Libraries

Store reusable queries in a `.jpx` file:

```jmespath
-- :name active-users
-- :desc Return active user names
users[?active].name | sort(@)

-- :name summary
{
  total: length(users),
  active: length(users[?active])
}
```

```bash
jpx -Q queries.jpx:active-users data.json
jpx -Q queries.jpx --list-queries
jpx -Q queries.jpx --check
```

See [Query Files](https://joshrotenberg.github.io/jpx/cli/query-files/) for the
complete format and usage guide.

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

**Tools** (31 total): `evaluate`, `batch_evaluate`, `validate`, `explain`, `functions`, `describe`, `batch_describe`, `search`, `similar`, `format`, `diff`, `patch`, `merge`, `stats`, `paths`, `keys`, plus an ephemeral process-scoped query store. See the [MCP docs](https://joshrotenberg.github.io/jpx/mcp/overview/) for the full list.

## Function Categories

The library provides 490+ functions across these categories:

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
- **[jmespath.rs](https://crates.io/crates/jmespath)** - the original Rust implementation and a compatibility/benchmark reference
- **[jpx-core](crates/jpx-core/)** - jpx's independent JMESPath implementation with 470+ extension functions

## License

MIT or Apache-2.0
