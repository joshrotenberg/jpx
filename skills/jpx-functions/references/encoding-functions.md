# Encoding Functions (10)

## `base64_decode`

**Signature:** `string -> string`

Decode base64 string

```
base64_decode('aGVsbG8=') -> \"hello\"
```
_Decode hello_

```
base64_decode('dGVzdA==') -> \"test\"
```
_Decode test_

---

## `base64_encode`

**Signature:** `string -> string`

Encode string to base64

```
base64_encode('hello') -> \"aGVsbG8=\"
```
_Encode hello_

```
base64_encode('test') -> \"dGVzdA==\"
```
_Encode test_

---

## `base64url_decode`

**Signature:** `string -> string`

Decode base64url (RFC 4648 §5) string

```
base64url_decode('aGVsbG8') -> \"hello\"
```
_Decode hello_

```
base64url_decode('dGVzdA') -> \"test\"
```
_Decode test_

---

## `base64url_encode`

**Signature:** `string -> string`

Encode string to base64url (RFC 4648 §5, no padding)

```
base64url_encode('hello') -> \"aGVsbG8\"
```
_Encode hello_

```
base64url_encode('test') -> \"dGVzdA\"
```
_Encode test_

---

## `hex_decode`

**Signature:** `string -> string`

Decode hex string

```
hex_decode('68656c6c6f') -> \"hello\"
```
_Decode hello_

```
hex_decode('74657374') -> \"test\"
```
_Decode test_

---

## `hex_encode`

**Signature:** `string -> string`

Encode string to hex

```
hex_encode('hello') -> \"68656c6c6f\"
```
_Encode hello_

```
hex_encode('test') -> \"74657374\"
```
_Encode test_

---

## `html_escape`

**Signature:** `string -> string`

Escape HTML special characters

```
html_escape('<div>') -> \"&lt;div&gt;\"
```
_Escape tags_

```
html_escape('a & b') -> \"a &amp; b\"
```
_Escape ampersand_

---

## `html_unescape`

**Signature:** `string -> string`

Unescape HTML entities

```
html_unescape('&lt;div&gt;') -> \"<div>\"
```
_Unescape tags_

```
html_unescape('a &amp; b') -> \"a & b\"
```
_Unescape ampersand_

---

## `jwt_decode`

**Signature:** `string -> object`

Decode JWT payload (claims) without verification

```
jwt_decode('eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyXzEyMyJ9.sig').sub -> \"user_123\"
```
_Extract subject claim_

```
jwt_decode('eyJhbGciOiJIUzI1NiJ9.eyJuYW1lIjoiSm9obiJ9.sig').name -> 'John'
```
_Extract name claim_

---

## `jwt_header`

**Signature:** `string -> object`

Decode JWT header without verification

```
jwt_header('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig').alg -> \"HS256\"
```
_Extract algorithm_

```
jwt_header('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.sig').typ -> \"JWT\"
```
_Extract type_

---

