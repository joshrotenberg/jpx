# Discovery Functions

Functions for searching and matching within collections, useful for tool discovery and fuzzy filtering.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`fuzzy_search`](#fuzzy_search) | `array, string\|object, string -> array` | Search array of objects by multiple fields with relevance scoring |
| [`fuzzy_match`](#fuzzy_match) | `string, string -> object` | Check if a string matches a query, with match details |
| [`fuzzy_score`](#fuzzy_score) | `string, string -> number` | Get numeric match score between value and query |

## Functions

### fuzzy_search

Search an array of objects by multiple fields, returning matches sorted by relevance score.

**Signature:** `array, string|object, string -> array`

The second argument specifies which fields to search:
- **String**: Comma-separated field names with equal weight: `"name,description"`
- **Object**: Field names with custom weights: `` `{"name": 10, "description": 5}` ``

**Examples:**

```jmespath
# Search tools by name and description fields
fuzzy_search(tools, `"name,description"`, `"user"`)
# -> [{item: {...}, score: 100, match_type: "exact", matched_field: "name"}, ...]

# Search with weighted fields (name matches worth more)
fuzzy_search(tools, `{"name": 10, "description": 5}`, `"cache"`)

# Get the name of the top match
fuzzy_search(tools, `"name,tags"`, `"redis"`)[0].item.name
```

**CLI Usage:**

```bash
# Search a list of tools for "backup"
cat tools.json | jpx 'fuzzy_search(@, `"name,description"`, `"backup"`)'
```

**Return format:**

Each result contains:
- `item`: The original object that matched
- `score`: Relevance score (higher = better match)
- `match_type`: How it matched (`"exact"`, `"prefix"`, `"contains"`, `"fuzzy"`)
- `matched_field`: Which field produced the match

---

### fuzzy_match

Check if a single string value matches a query, returning detailed match information.

**Signature:** `string, string -> object`

Match types (in priority order):
- **exact** (score: 1000): Exact match after normalization
- **prefix** (score: 800): Query is a prefix of the value
- **contains** (score: 600): Query appears within the value
- **fuzzy** (score: varies): Similarity-based match
- **none** (score: 0): No meaningful match

**Examples:**

```jmespath
# Exact match
fuzzy_match(`"get_user"`, `"get_user"`)
# -> {matches: true, score: 1000, match_type: "exact"}

# Prefix match
fuzzy_match(`"get_user_info"`, `"get"`)
# -> {matches: true, score: 800, match_type: "prefix"}

# Contains match
fuzzy_match(`"hello_world"`, `"world"`)
# -> {matches: true, score: 600, match_type: "contains"}

# No match (returns similarity score for near-misses)
fuzzy_match(`"hello"`, `"xyz"`)
# -> {matches: false, score: 0, match_type: "none", similarity: 0.0}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'fuzzy_match(`"database_backup"`, `"backup"`)'
```

---

### fuzzy_score

Get the numeric match score between a value and query. This is a simpler alternative to `fuzzy_match` when you only need the score.

**Signature:** `string, string -> number`

Score ranges:
- **1000**: Exact match
- **800**: Prefix match
- **600**: Contains match
- **1-599**: Fuzzy similarity match
- **0**: No match

**Examples:**

```jmespath
# Exact match - highest score
fuzzy_score(`"hello"`, `"hello"`) -> 1000

# Prefix match
fuzzy_score(`"hello_world"`, `"hello"`) -> 800

# Contains match
fuzzy_score(`"say_hello"`, `"hello"`) -> 600

# No match
fuzzy_score(`"hello"`, `"goodbye"`) -> 0
```

**CLI Usage:**

```bash
# Score a potential match
echo '{}' | jpx 'fuzzy_score(`"enterprise_database_create"`, `"database"`)'
```

**Use with filtering:**

```jmespath
# Filter to items with score > 500
tools[?fuzzy_score(name, `"user"`) > `500`]

# Sort by match quality
sort_by(tools, &fuzzy_score(name, `"cache"`)) | reverse(@)
```

## Use Cases

### Tool Discovery

Find relevant tools from a large list:

```jmespath
fuzzy_search(mcp_tools, `"name,description"`, `"backup database"`)
  | [?score > `100`]
  | [:5]
  | [*].{name: item.name, score: score, why: match_type}
```

### Autocomplete

Filter options as user types:

```jmespath
commands[?fuzzy_score(name, query) > `0`]
  | sort_by(@, &fuzzy_score(name, query))
  | reverse(@)
  | [:10]
```

### Deduplication

Find similar entries:

```jmespath
items[?fuzzy_match(title, other_title).match_type == `"fuzzy"` 
      && fuzzy_match(title, other_title).score > `400`]
```
