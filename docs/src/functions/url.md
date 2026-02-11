# URL Functions

Functions for parsing and manipulating URLs and their components.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`query_string_build`](#query-string-build) | `object -> string` | Build a URL query string from an object |
| [`query_string_parse`](#query-string-parse) | `string -> object` | Parse a URL query string into an object |
| [`url_build`](#url-build) | `object -> string` | Build a URL from component parts |
| [`url_decode`](#url-decode) | `string -> string` | URL decode a string |
| [`url_encode`](#url-encode) | `string -> string` | URL encode a string |
| [`url_parse`](#url-parse) | `string -> object` | Parse URL into components |

## Functions

### query_string_build

Build a URL query string from an object

**Signature:** `object -> string`

**Examples:**

```text
# Basic query string
query_string_build({foo: 'bar', baz: 'qux'}) -> "foo=bar&baz=qux"
# Special characters are encoded
query_string_build({q: 'hello world'}) -> "q=hello+world"
# Empty object
query_string_build({}) -> ""
```

**CLI Usage:**

```bash
echo '{"q": "hello world", "page": "1"}' | jpx 'query_string_build(@)'
```

### query_string_parse

Parse a URL query string into an object

**Signature:** `string -> object`

**Examples:**

```text
# Basic parsing
query_string_parse('foo=bar&baz=qux') -> {"foo": "bar", "baz": "qux"}
# Encoded values are decoded
query_string_parse('greeting=hello%20world') -> {"greeting": "hello world"}
# Empty string
query_string_parse('') -> {}
```

**CLI Usage:**

```bash
echo '"foo=bar&baz=qux"' | jpx 'query_string_parse(@)'
```

### url_build

Build a URL from component parts (inverse of `url_parse`)

**Signature:** `object -> string`

The input object supports these fields:

- `scheme` (required): URL scheme (e.g., `"https"`)
- `host` (required): Hostname (e.g., `"example.com"`)
- `port` (optional): Port number
- `path` (optional): URL path
- `query` (optional): Query string
- `fragment` (optional): Fragment identifier
- `username` (optional): Username for authentication
- `password` (optional): Password for authentication

**Examples:**

```text
# Minimal URL
url_build({scheme: 'https', host: 'example.com'}) -> "https://example.com/"
# With port and path
url_build({scheme: 'https', host: 'example.com', port: 8080, path: '/api/v1'}) -> "https://example.com:8080/api/v1"
# Roundtrip with url_parse
url_build(url_parse('https://example.com/path')) -> "https://example.com/path"
```

**CLI Usage:**

```bash
echo '{"scheme": "https", "host": "example.com", "path": "/api"}' | jpx 'url_build(@)'
```

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

