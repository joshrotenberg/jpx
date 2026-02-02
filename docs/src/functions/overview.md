# Function Overview

jpx provides 400+ functions organized into 31 categories.

## Discovering Functions

### List All Functions

```bash
jpx --list-functions
```

### List by Category

```bash
jpx --list-category string
jpx --list-category math
jpx --list-category datetime
```

### Get Function Details

```bash
jpx --describe upper
```

## Categories

| Category | Description | Count |
|----------|-------------|-------|
| [Array](./array.md) | Functions for working with arrays: chunking, filtering, tran... | 27 |
| [Color](./color.md) | Color manipulation and conversion functions. | 8 |
| [Computing](./computing.md) | Computing-related utility functions. | 9 |
| [Date/Time](./datetime.md) | Functions for working with dates and times: parsing, formatt... | 26 |
| [Duration](./duration.md) | Functions for working with time durations. | 5 |
| [Encoding](./encoding.md) | Encoding and decoding functions: Base64, hex, URL encoding, ... | 8 |
| [Expression](./expression.md) | Higher-order functions that work with JMESPath expressions a... | 33 |
| [Format](./format.md) | Data formatting functions for numbers, currencies, and other... | 6 |
| [Fuzzy](./fuzzy.md) | Fuzzy matching and string similarity functions. | 9 |
| [Geolocation](./geo.md) | Geolocation functions: distance calculation, coordinate pars... | 4 |
| [Hash](./hash.md) | Cryptographic hash functions: MD5, SHA family, and other has... | 9 |
| [ID Generation](./ids.md) | Functions for generating various types of unique identifiers... | 3 |
| [JSON Patch](./jsonpatch.md) | JSON Patch (RFC 6902) functions: applying patches, generatin... | 3 |
| [Language](./language.md) | Natural language processing functions. | 5 |
| [Math](./math.md) | Mathematical and statistical functions: arithmetic, rounding... | 35 |
| [Multi-Match](./multimatch.md) | Functions for matching multiple patterns or expressions in a... | 10 |
| [Network](./network.md) | Network-related functions: IP addresses, CIDR notation, and ... | 7 |
| [Object](./object.md) | Functions for working with JSON objects: merging, filtering ... | 48 |
| [Path](./path.md) | File path manipulation functions. | 4 |
| [Phonetic](./phonetic.md) | Phonetic encoding functions for sound-based string matching. | 9 |
| [Random](./rand.md) | Functions for generating random values: numbers, strings, an... | 3 |
| [Regular Expression](./regex.md) | Regular expression functions: matching, replacing, splitting... | 3 |
| [Semantic Versioning](./semver.md) | Semantic versioning functions: parsing, comparing, and manip... | 7 |
| [Standard JMESPath](./standard.md) | These are the standard JMESPath functions as defined in the ... | 26 |
| [String](./string.md) | Functions for string manipulation: case conversion, splittin... | 36 |
| [Text](./text.md) | Text analysis and processing functions. | 11 |
| [Type](./type.md) | Type conversion and checking functions. | 13 |
| [URL](./url.md) | Functions for parsing and manipulating URLs and their compon... | 3 |
| [Utility](./utility.md) | General utility functions that don't fit other categories. | 11 |
| [UUID](./uuid.md) | Functions for generating and working with UUIDs. | 1 |
| [Validation](./validation.md) | Functions for validating data: email, URL, UUID, and format ... | 13 |

## Function Syntax

Functions are called with parentheses:

```bash
function_name(arg1, arg2, ...)
```

### Examples

```bash
# No arguments
echo '{}' | jpx 'now()'

# One argument
echo '{"name": "hello"}' | jpx 'upper(name)'

# Multiple arguments
echo '{"text": "hello world"}' | jpx 'split(text, ` `)'

# Literal arguments (use backticks)
echo '{}' | jpx 'range(`1`, `10`)'
```

## Standard vs Extension Functions

### Standard Functions (26)

These are part of the JMESPath specification and work in all implementations:

`abs`, `avg`, `ceil`, `contains`, `ends_with`, `floor`, `join`, `keys`, `length`, `map`, `max`, `max_by`, `merge`, `min`, `min_by`, `not_null`, `reverse`, `sort`, `sort_by`, `starts_with`, `sum`, `to_array`, `to_number`, `to_string`, `type`, `values`

### Extension Functions (369)

These are jpx-specific and won't work in other JMESPath implementations.

### Strict Mode

Use `--strict` to disable extension functions:

```bash
# This works
jpx --strict 'length(items)' -f data.json

# This fails (upper is an extension)
jpx --strict 'upper(name)' -f data.json
```
