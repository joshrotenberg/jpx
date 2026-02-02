# URL Functions

Functions for parsing and manipulating URLs and their components.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`url_decode`](#url-decode) | `string -> string` | URL decode a string |
| [`url_encode`](#url-encode) | `string -> string` | URL encode a string |
| [`url_parse`](#url-parse) | `string -> object` | Parse URL into components |

## Functions

### url_decode

URL decode a string

**Signature:** `string -> string`

**Examples:**

```text
# Decode space
url_decode('hello%20world') -> \"hello world\"
# Decode plus sign
url_decode('a%2Bb') -> \"a+b\"
# Decode percent
url_decode('100%25') -> \"100%\"
# No encoding
url_decode('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'url_decode(`"hello%20world"`)'
```

### url_encode

URL encode a string

**Signature:** `string -> string`

**Examples:**

```text
# Encode space
url_encode('hello world') -> \"hello%20world\"
# Encode plus
url_encode('a+b') -> \"a%2Bb\"
# Encode percent
url_encode('100%') -> \"100%25\"
# No special chars
url_encode('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'url_encode(`"hello world"`)'
```

### url_parse

Parse URL into components

**Signature:** `string -> object`

**Examples:**

```text
# Parse full URL
url_parse('https://example.com/path') -> {scheme: 'https', ...}
# With auth and port
url_parse('http://user:pass@host:8080') -> components
# With query string
url_parse('https://api.example.com/v1?key=val') -> with query
# Relative URL
url_parse('/path/to/file') -> relative path
```

**CLI Usage:**

```bash
echo '{}' | jpx 'url_parse(`"https://example.com/path"`)'
```

