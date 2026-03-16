# Validation Functions (13)

## `is_base64`

**Signature:** `string -> boolean`

Check if valid Base64 encoding

```
is_base64('SGVsbG8=') -> true
```
_Valid base64_

```
is_base64('not valid!') -> false
```
_Invalid chars_

---

## `is_credit_card`

**Signature:** `string -> boolean`

Validate credit card number (Luhn check + length)

```
is_credit_card('4111111111111111') -> true
```
_Valid Visa test number_

```
is_credit_card('1234567890123456') -> false
```
_Invalid number_

---

## `is_email`

**Signature:** `string -> boolean`

Validate email address format

```
is_email('user@example.com') -> true
```
_Valid email_

```
is_email('invalid-email') -> false
```
_Missing @_

---

## `is_hex`

**Signature:** `string -> boolean`

Check if valid hexadecimal string

```
is_hex('deadbeef') -> true
```
_Valid hex_

```
is_hex('ABCDEF') -> true
```
_Uppercase hex_

---

## `is_ipv4`

**Signature:** `string -> boolean`

Validate IPv4 address format

```
is_ipv4('192.168.1.1') -> true
```
_Valid IPv4_

```
is_ipv4('256.1.1.1') -> false
```
_Out of range_

---

## `is_ipv6`

**Signature:** `string -> boolean`

Validate IPv6 address format

```
is_ipv6('::1') -> true
```
_Loopback_

```
is_ipv6('2001:db8::1') -> true
```
_Full IPv6_

---

## `is_iso_date`

**Signature:** `string -> boolean`

Validate ISO 8601 date format

```
is_iso_date('2023-12-13T15:30:00Z') -> true
```
_Full ISO format_

```
is_iso_date('2023-12-13') -> true
```
_Date only_

---

## `is_json`

**Signature:** `string -> boolean`

Check if string is valid JSON

```
is_json('{\"a\": 1}') -> true
```
_Valid JSON object_

```
is_json('[1, 2, 3]') -> true
```
_Valid JSON array_

---

## `is_jwt`

**Signature:** `string -> boolean`

Check if valid JWT structure (3 base64url parts)

```
is_jwt('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U') -> true
```
_Valid JWT_

```
is_jwt('eyJhbGciOiJIUzI1NiJ9.eyJuYW1lIjoiSm9obiJ9.signature') -> true
```
_Three parts with dots_

---

## `is_phone`

**Signature:** `string -> boolean`

Validate phone number format

```
is_phone('+1-555-123-4567') -> true
```
_US format with country code_

```
is_phone('555-123-4567') -> true
```
_US format without country code_

---

## `is_url`

**Signature:** `string -> boolean`

Validate URL format

```
is_url('https://example.com') -> true
```
_Simple HTTPS URL_

```
is_url('http://localhost:8080/path') -> true
```
_URL with port and path_

---

## `is_uuid`

**Signature:** `string -> boolean`

Validate UUID format

```
is_uuid('550e8400-e29b-41d4-a716-446655440000') -> true
```
_Valid UUID v4_

```
is_uuid('6ba7b810-9dad-11d1-80b4-00c04fd430c8') -> true
```
_Valid UUID v1_

---

## `luhn_check`

**Signature:** `string -> boolean`

Generic Luhn algorithm check

```
luhn_check('79927398713') -> true
```
_Valid Luhn number_

```
luhn_check('4532015112830366') -> true
```
_Valid credit card number_

---

