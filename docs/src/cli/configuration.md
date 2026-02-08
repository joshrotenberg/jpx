# Configuration

jpx supports layered configuration through `jpx.toml` files, allowing you to customize engine behavior, filter functions, and define reusable queries.

## Discovery Order

Configuration is loaded from multiple sources, with later sources overriding earlier ones:

1. **Defaults** -- all functions enabled, strict mode off
2. **Global** -- `~/.config/jpx/jpx.toml`
3. **Project-local** -- `jpx.toml` found by walking up from the current directory
4. **Environment** -- file path specified by `$JPX_CONFIG`
5. **CLI flags** -- command-line arguments (highest priority)

## Full Example

```toml
[engine]
strict = false

[functions]
disabled_categories = ["geo", "phonetic"]
disabled_functions = ["env"]

[queries]
libraries = ["~/.config/jpx/common.jpx"]

[queries.inline]
active-users = { expression = "users[?active].name", description = "Get active user names" }
```

## Sections

### `[engine]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `strict` | bool | `false` | When true, only standard JMESPath functions are available |

### `[functions]`

Controls which extension functions are available. Supports two mutually exclusive approaches:

**Blocklist (default):** Everything enabled, opt out selectively.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `disabled_categories` | array | `[]` | Categories to disable entirely |
| `disabled_functions` | array | `[]` | Individual functions to disable |

**Allowlist:** Only specified categories enabled.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled_categories` | array | none | If set, only these categories are enabled |

!!! note
    `enabled_categories` and `disabled_categories` are mutually exclusive. Setting `enabled_categories` clears the disabled list.

### `[queries]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `libraries` | array | `[]` | Paths to `.jpx` query library files to load |

### `[queries.inline]`

Define named queries directly in the config file:

```toml
[queries.inline]
active-users = { expression = "users[?active].name", description = "Get active user names" }
item-count = { expression = "length(items)", description = "Count items" }
```

Each inline query has:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `expression` | string | yes | The JMESPath expression |
| `description` | string | no | Optional description |

## Merge Semantics

When multiple config files are found, they are merged with these rules:

| Field | Merge behavior |
|-------|---------------|
| `engine.strict` | Later wins (if `true`, stays `true`) |
| `functions.disabled_categories` | Union (combined from all sources) |
| `functions.disabled_functions` | Union (combined from all sources) |
| `functions.enabled_categories` | Later replaces (switches to allowlist) |
| `queries.libraries` | Concatenated |
| `queries.inline` | Later keys override same-name entries |

## Examples

### Disable specific categories

```toml
[functions]
disabled_categories = ["geo", "phonetic", "color"]
```

### Allowlist mode

Only enable string and math functions:

```toml
[functions]
enabled_categories = ["string", "math"]
```

### Shared query library

```toml
[queries]
libraries = ["~/.config/jpx/common.jpx", "./project-queries.jpx"]

[queries.inline]
health-check = { expression = "status == `ok`", description = "Check service health" }
```

## CLI-Level Configuration

The jpx CLI also reads output preferences from `~/.config/jpx/config.toml` or `~/.jpxrc`. These settings control default output behavior (color mode, format preferences) and are separate from the engine configuration above.

## Environment Variables

Set `JPX_CONFIG` to point to a specific config file:

```bash
export JPX_CONFIG=/path/to/custom/jpx.toml
jpx 'expression' -f data.json
```

This takes precedence over global and project-local config files, but CLI flags still win.
