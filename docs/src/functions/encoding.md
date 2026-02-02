# Encoding Functions

Encoding and decoding functions: Base64, hex, URL encoding, and more.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`base64_decode`](#base64-decode) | `string -> string` | Decode base64 string |
| [`base64_encode`](#base64-encode) | `string -> string` | Encode string to base64 |
| [`hex_decode`](#hex-decode) | `string -> string` | Decode hex string |
| [`hex_encode`](#hex-encode) | `string -> string` | Encode string to hex |
| [`html_escape`](#html-escape) | `string -> string` | Escape HTML special characters |
| [`html_unescape`](#html-unescape) | `string -> string` | Unescape HTML entities |
| [`jwt_decode`](#jwt-decode) | `string -> object` | Decode JWT payload (claims) without verification |
| [`jwt_header`](#jwt-header) | `string -> object` | Decode JWT header without verification |

## Functions

### base64_decode

Decode base64 string

**Signature:** `string -> string`

**Examples:**

```text
# Decode hello
base64_decode('aGVsbG8=') -> \"hello\"
# Decode test
base64_decode('dGVzdA==') -> \"test\"
# Empty string
base64_decode('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'base64_decode(`"aGVsbG8="`)'
```

### base64_encode

Encode string to base64

**Signature:** `string -> string`

**Examples:**

```text
# Encode hello
base64_encode('hello') -> \"aGVsbG8=\"
# Encode test
base64_encode('test') -> \"dGVzdA==\"
# Empty string
base64_encode('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'base64_encode(`"hello"`)'
```

### hex_decode

Decode hex string

**Signature:** `string -> string`

**Examples:**

```text
# Decode hello
hex_decode('68656c6c6f') -> \"hello\"
# Decode test
hex_decode('74657374') -> \"test\"
# Empty string
hex_decode('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hex_decode(`"68656c6c6f"`)'
```

### hex_encode

Encode string to hex

**Signature:** `string -> string`

**Examples:**

```text
# Encode hello
hex_encode('hello') -> \"68656c6c6f\"
# Encode test
hex_encode('test') -> \"74657374\"
# Empty string
hex_encode('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hex_encode(`"hello"`)'
```

### html_escape

Escape HTML special characters

**Signature:** `string -> string`

**Examples:**

```text
# Escape tags
html_escape('<div>') -> \"&lt;div&gt;\"
# Escape ampersand
html_escape('a & b') -> \"a &amp; b\"
# Escape quotes
html_escape('\"quoted\"') -> \"&quot;quoted&quot;\"
# Sanitize input
html_escape(user_input) -> safe for HTML
```

**CLI Usage:**

```bash
echo '{}' | jpx 'html_escape(`"<div>"`)'
```

### html_unescape

Unescape HTML entities

**Signature:** `string -> string`

**Examples:**

```text
# Unescape tags
html_unescape('&lt;div&gt;') -> \"<div>\"
# Unescape ampersand
html_unescape('a &amp; b') -> \"a & b\"
# Unescape quotes
html_unescape('&quot;quoted&quot;') -> \"\\\"quoted\\\"\"
# Decode entities
html_unescape(escaped) -> original
```

**CLI Usage:**

```bash
echo '{}' | jpx 'html_unescape(`"&lt;div&gt;"`)'
```

### jwt_decode

Decode JWT payload (claims) without verification

**Signature:** `string -> object`

**Examples:**

```text
# Extract subject claim
jwt_decode('eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEyMyJ9.sig').sub -> \"user_123\"
# Extract name claim
jwt_decode('eyJhbGciOiJIUzI1NiJ9.eyJuYW1lIjoiSm9obiJ9.sig').name -> 'John'
```

**CLI Usage:**

```bash
echo '{}' | jpx 'jwt_decode(`"eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEyMyJ9.sig"`).sub'
```

### jwt_header

Decode JWT header without verification

**Signature:** `string -> object`

**Examples:**

```text
# Extract algorithm
jwt_header('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig').alg -> \"HS256\"
# Extract type
jwt_header('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig').typ -> \"JWT\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'jwt_header(`"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig"`).alg'
```

