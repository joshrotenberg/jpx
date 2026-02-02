# Validation Functions

Functions for validating data: email, URL, UUID, and format validation.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`is_base64`](#is-base64) | `string -> boolean` | Check if valid Base64 encoding |
| [`is_credit_card`](#is-credit-card) | `string -> boolean` | Validate credit card number (Luhn check + length) |
| [`is_email`](#is-email) | `string -> boolean` | Validate email address format |
| [`is_hex`](#is-hex) | `string -> boolean` | Check if valid hexadecimal string |
| [`is_ipv4`](#is-ipv4) | `string -> boolean` | Validate IPv4 address format |
| [`is_ipv6`](#is-ipv6) | `string -> boolean` | Validate IPv6 address format |
| [`is_iso_date`](#is-iso-date) | `string -> boolean` | Validate ISO 8601 date format |
| [`is_json`](#is-json) | `string -> boolean` | Check if string is valid JSON |
| [`is_jwt`](#is-jwt) | `string -> boolean` | Check if valid JWT structure (3 base64url parts) |
| [`is_phone`](#is-phone) | `string -> boolean` | Validate phone number format |
| [`is_url`](#is-url) | `string -> boolean` | Validate URL format |
| [`is_uuid`](#is-uuid) | `string -> boolean` | Validate UUID format |
| [`luhn_check`](#luhn-check) | `string -> boolean` | Generic Luhn algorithm check |

## Functions

### is_base64

Check if valid Base64 encoding

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid base64
is_base64('SGVsbG8=') -> true
# Invalid chars
is_base64('not valid!') -> false
# Empty is valid
is_base64('') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_base64(`"SGVsbG8="`)'
```

### is_credit_card

Validate credit card number (Luhn check + length)

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid Visa test number
is_credit_card('4111111111111111') -> true
# Invalid number
is_credit_card('1234567890123456') -> false
# Valid Mastercard test
is_credit_card('5500000000000004') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_credit_card(`"4111111111111111"`)'
```

### is_email

Validate email address format

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid email
is_email('user@example.com') -> true
# Missing @
is_email('invalid-email') -> false
# Subdomain
is_email('user@sub.domain.com') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_email(`"user@example.com"`)'
```

### is_hex

Check if valid hexadecimal string

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid hex
is_hex('deadbeef') -> true
# Uppercase hex
is_hex('ABCDEF') -> true
# Invalid chars
is_hex('ghijkl') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_hex(`"deadbeef"`)'
```

### is_ipv4

Validate IPv4 address format

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid IPv4
is_ipv4('192.168.1.1') -> true
# Out of range
is_ipv4('256.1.1.1') -> false
# Private IP
is_ipv4('10.0.0.1') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_ipv4(`"192.168.1.1"`)'
```

### is_ipv6

Validate IPv6 address format

**Signature:** `string -> boolean`

**Examples:**

```text
# Loopback
is_ipv6('::1') -> true
# Full IPv6
is_ipv6('2001:db8::1') -> true
# IPv4 is not IPv6
is_ipv6('192.168.1.1') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_ipv6(`"::1"`)'
```

### is_iso_date

Validate ISO 8601 date format

**Signature:** `string -> boolean`

**Examples:**

```text
# Full ISO format
is_iso_date('2023-12-13T15:30:00Z') -> true
# Date only
is_iso_date('2023-12-13') -> true
# US format invalid
is_iso_date('12/13/2023') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_iso_date(`"2023-12-13T15:30:00Z"`)'
```

### is_json

Check if string is valid JSON

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid JSON object
is_json('{\"a\": 1}') -> true
# Valid JSON array
is_json('[1, 2, 3]') -> true
# Invalid JSON
is_json('not json') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_json(`"{\"a\": 1}"`)'
```

### is_jwt

Check if valid JWT structure (3 base64url parts)

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid JWT
is_jwt('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U') -> true
# Three parts with dots
is_jwt('eyJhbGciOiJIUzI1NiJ9.eyJuYW1lIjoiSm9obiJ9.signature') -> true
# Four parts
is_jwt('not.a.valid.jwt') -> false
# No dots
is_jwt('invalid') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_jwt(`"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"`)'
```

### is_phone

Validate phone number format

**Signature:** `string -> boolean`

**Examples:**

```text
# US format with country code
is_phone('+1-555-123-4567') -> true
# US format without country code
is_phone('555-123-4567') -> true
# UK format
is_phone('+44 20 7946 0958') -> true
# Invalid phone
is_phone('invalid') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_phone(`"+1-555-123-4567"`)'
```

### is_url

Validate URL format

**Signature:** `string -> boolean`

**Examples:**

```text
# Simple HTTPS URL
is_url('https://example.com') -> true
# URL with port and path
is_url('http://localhost:8080/path') -> true
# FTP URL
is_url('ftp://files.example.com') -> true
# Invalid URL
is_url('not a url') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_url(`"https://example.com"`)'
```

### is_uuid

Validate UUID format

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid UUID v4
is_uuid('550e8400-e29b-41d4-a716-446655440000') -> true
# Valid UUID v1
is_uuid('6ba7b810-9dad-11d1-80b4-00c04fd430c8') -> true
# Nil UUID
is_uuid('00000000-0000-0000-0000-000000000000') -> true
# Invalid format
is_uuid('not-a-uuid') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_uuid(`"550e8400-e29b-41d4-a716-446655440000"`)'
```

### luhn_check

Generic Luhn algorithm check

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid Luhn number
luhn_check('79927398713') -> true
# Valid credit card number
luhn_check('4532015112830366') -> true
# Invalid Luhn
luhn_check('1234567890') -> false
# Single zero
luhn_check('0') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'luhn_check(`"79927398713"`)'
```

