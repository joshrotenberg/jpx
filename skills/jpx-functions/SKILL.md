---
name: jpx-functions
description: "Look up and call the 460+ extension functions in jpx beyond standard JMESPath. Covers array, string, math, datetime, hash, encoding, regex, object, expression (higher-order), geo, text, validation, and 20+ more categories. Use when a task needs functions beyond the 26 standard JMESPath built-ins."
license: MIT OR Apache-2.0
metadata:
  author: joshrotenberg
  version: "1.0"
compatibility: Requires jpx, jpx-core, jpx-engine, or jpx-python
---

# jpx Extension Functions

jpx extends JMESPath with 460+ functions across 33 categories. This skill covers function discovery, the most commonly used functions, and key conventions.

## Discovering Functions

### CLI
```bash
jpx --list-functions                    # all functions by category
jpx --list-category String             # functions in a category
jpx --describe split                   # detailed function info
jpx --search "group array"            # fuzzy search
jpx --similar group_by                 # find related functions
```

### MCP tools
Use `search`, `describe`, `functions`, `categories`, `similar`, or `suggest_function`.

## Key Conventions

### Expref arguments

Functions that operate "by key" use expression references (`&expr`), matching the JMESPath spec:

```
sort_by(users, &age)              -- standard
group_by(users, &department)      -- jpx extension (also accepts string for backward compat)
index_by(users, &id)              -- jpx extension
```

### Expression functions

Higher-order functions in the Expression category take `&expr` as their first argument:

```
map_expr(&upper(name), users)     -- transform each element
filter_expr(&(age > `18`), users) -- filter by expression
sort_by_expr(&score, users)       -- sort by expression
group_by_expr(&type, items)       -- group by expression
reduce_expr(&(@ + @@), [1,2,3], `0`)  -- reduce with accumulator
```

In expression function callbacks, `@` is the current element and `@@` is the accumulator (for reduce).

## Categories

| Category | Count | Key Functions |
|----------|-------|---------------|
| [Array](references/array-functions.md) | 36 | `unique`, `chunk`, `group_by`, `index_by`, `zip`, `flatten_deep`, ... |
| [String](references/string-functions.md) | 36 | `split`, `upper`, `lower`, `trim`, `replace`, `pad_left`, `template`, ... |
| [Math](references/math-functions.md) | 42 | `sum`, `avg`, `median`, `round`, `clamp`, `range`, `std_dev`, ... |
| [Object](references/object-functions.md) | 51 | `omit`, `pick`, `deep_merge`, `defaults`, `rename_keys`, `flatten_keys`, ... |
| [Expression](references/expression-functions.md) | 41 | `map_expr`, `filter_expr`, `reduce_expr`, `sort_by_expr`, `group_by_expr`, ... |
| [Datetime](references/datetime-functions.md) | 28 | `now`, `format_date`, `parse_date`, `date_add`, `date_diff`, ... |
| [Text](references/text-functions.md) | 21 | `word_count`, `sentences`, `slug`, `truncate_words`, `ngrams`, ... |
| [Type](references/type-functions.md) | 13 | `to_number`, `to_string`, `type_of`, `is_number`, `auto_parse`, ... |
| [Validation](references/validation-functions.md) | 13 | `is_email`, `is_url`, `is_uuid`, `is_ipv4`, `is_base64`, ... |
| [Utility](references/utility-functions.md) | 11 | `coalesce`, `default`, `env`, `identity`, `if`, ... |
| [Encoding](references/encoding-functions.md) | 10 | `base64_encode`, `base64_decode`, `url_encode`, `url_decode`, `hex`, ... |
| [Geo](references/geo-functions.md) | 10 | `geo_distance`, `geo_bearing`, `geo_midpoint`, `geohash_encode`, ... |
| [Multimatch](references/multimatch-functions.md) | 10 | `match_all`, `extract_all`, `extract_between`, `scan`, ... |
| [Hash](references/hash-functions.md) | 9 | `md5`, `sha256`, `sha512`, `hmac_sha256`, `crc32`, ... |
| [Computing](references/computing-functions.md) | 9 | `bit_and`, `bit_or`, `bit_xor`, `bit_shift_left`, ... |
| [Fuzzy](references/fuzzy-functions.md) | 9 | `levenshtein`, `jaro_winkler`, `soundex`, `hamming`, ... |
| [Phonetic](references/phonetic-functions.md) | 9 | `metaphone`, `double_metaphone`, `soundex`, `nysiis`, ... |
| [Duration](references/duration-functions.md) | 8 | `parse_duration`, `format_duration`, `duration_add`, ... |
| [Color](references/color-functions.md) | 8 | `hex_to_rgb`, `rgb_to_hex`, `color_lighten`, `color_darken`, ... |
| [Path](references/path-functions.md) | 7 | `path_basename`, `path_dirname`, `path_ext`, `path_join`, ... |
| [Network](references/network-functions.md) | 7 | `cidr_contains`, `ip_to_int`, `int_to_ip`, `cidr_network`, ... |
| [Semver](references/semver-functions.md) | 7 | `semver_compare`, `semver_major`, `semver_minor`, `semver_is_valid`, ... |
| [Format](references/format-functions.md) | 6 | `to_csv`, `from_csv`, `to_tsv`, `from_tsv`, `to_json_string`, ... |
| [URL](references/url-functions.md) | 6 | `url_parse`, `url_build`, `query_string_parse`, `query_string_build`, ... |
| [Regex](references/regex-functions.md) | 5 | `regex_match`, `regex_replace`, `regex_extract`, `regex_split`, ... |
| [Rand](references/rand-functions.md) | 5 | `random`, `random_int`, `random_choice`, `random_sample`, ... |
| [Language](references/language-functions.md) | 5 | `detect_language`, `stems`, `remove_stopwords`, ... |
| [Units](references/units-functions.md) | 4 | `convert_temperature`, `convert_length`, `convert_mass`, ... |
| [IDs](references/ids-functions.md) | 3 | `nanoid`, `ulid`, `ulid_timestamp` |
| [JSON Patch](references/jsonpatch-functions.md) | 3 | `json_diff`, `json_patch`, `json_merge_patch` |
| [Discovery](references/discovery-functions.md) | 3 | `fuzzy_match`, `fuzzy_score`, `fuzzy_search` |
| [UUID](references/uuid-functions.md) | 1 | `uuid` |

## Most Used Functions (Quick Reference)

### Array
```
unique([1,2,2,3])                        -> [1,2,3]
chunk([1,2,3,4,5], `2`)                  -> [[1,2],[3,4],[5]]
group_by(users, &role)                   -> {"admin":[...], "user":[...]}
index_by(users, &id)                     -> {"1":{...}, "2":{...}}
zip([1,2], ['a','b'])                    -> [[1,"a"],[2,"b"]]
flatten_deep([[1,[2]],[3]])              -> [1,2,3]
first([1,2,3])                           -> 1
last([1,2,3])                            -> 3
range(`1`, `6`)                          -> [1,2,3,4,5]
```

### String
```
split('a,b,c', ',')                      -> ["a","b","c"]
upper('hello')                           -> "HELLO"
lower('HELLO')                           -> "hello"
trim('  hi  ')                           -> "hi"
replace('hello', 'l', 'r')              -> "herro"
pad_left('42', `5`, '0')                -> "00042"
template('Hello, {name}!', {"name":"Jo"}) -> "Hello, Jo!"
snake_case('helloWorld')                 -> "hello_world"
```

### Math
```
sum([1,2,3])                             -> 6
avg([1,2,3])                             -> 2.0
median([1,2,3,4])                        -> 2.5
round(`3.14159`, `2`)                    -> 3.14
clamp(`15`, `0`, `10`)                   -> 10
std_dev([2,4,4,4,5,5,7,9])              -> 2.0
```

### Datetime
```
now()                                    -> "2024-01-15T10:30:00Z"
format_date('2024-01-15', '%B %d, %Y')  -> "January 15, 2024"
date_add('2024-01-15', `30`, 'days')     -> "2024-02-14..."
date_diff('2024-03-01', '2024-01-01', 'days') -> 60
```

### Object
```
omit(obj, ['password', 'secret'])        -> obj without those keys
pick(obj, ['name', 'email'])             -> obj with only those keys
deep_merge(base, overlay)               -> recursively merged object
defaults(obj, {"color": "blue"})        -> fills in missing keys
rename_keys(obj, {"old": "new"})        -> renamed keys
```

### Utility
```
if(condition, then_val, else_val)        -> conditional
coalesce(null, null, 'found')            -> "found"
default(value, 'fallback')              -> value or fallback if null
```

## Runtime Setup (Rust)

For embedding jpx-core in Rust applications:

```rust
let mut runtime = jpx_core::Runtime::new();
runtime.register_builtin_functions();  // 26 spec functions
let mut registry = jpx_core::FunctionRegistry::new();
registry.register_all();               // loads extensions from functions.toml
registry.apply(&mut runtime);          // registers enabled functions
```

## Common Errors

- **"Unknown function 'xyz'"** -- the function doesn't exist or is misspelled. Use `jpx --search xyz` or the MCP `search` tool to find the correct name.
- **"Invalid argument type"** -- wrong argument type (e.g., passing a string where a number is expected). Use `jpx --describe function_name` to check the signature.
- **"Expected expref"** -- the function expects `&expr` but got a plain value. Add `&` prefix: `sort_by(arr, &field)` not `sort_by(arr, field)`.
- **Extension function in strict mode** -- strict mode (`--strict` or `JPX_STRICT=1`) disables all extensions. Only the 26 standard functions are available.

## Related Skills

- [jmespath-query](../jmespath-query/SKILL.md) -- JMESPath expression syntax (covers the 26 standard functions)
- [jpx-cli](../jpx-cli/SKILL.md) -- CLI tool usage
- [jpx-mcp](../jpx-mcp/SKILL.md) -- MCP server tools
