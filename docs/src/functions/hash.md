# Hash Functions

Cryptographic hash functions: MD5, SHA family, and other hash algorithms.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`crc32`](#crc32) | `string -> number` | Calculate CRC32 checksum |
| [`hmac_md5`](#hmac-md5) | `string, string -> string` | Calculate HMAC-MD5 signature |
| [`hmac_sha1`](#hmac-sha1) | `string, string -> string` | Calculate HMAC-SHA1 signature |
| [`hmac_sha256`](#hmac-sha256) | `string, string -> string` | Calculate HMAC-SHA256 signature |
| [`hmac_sha512`](#hmac-sha512) | `string, string -> string` | Calculate HMAC-SHA512 signature |
| [`md5`](#md5) | `string -> string` | Calculate MD5 hash |
| [`sha1`](#sha1) | `string -> string` | Calculate SHA-1 hash |
| [`sha256`](#sha256) | `string -> string` | Calculate SHA-256 hash |
| [`sha512`](#sha512) | `string -> string` | Calculate SHA-512 hash |

## Functions

### crc32

Calculate CRC32 checksum

**Signature:** `string -> number`

**Examples:**

```text
# Simple string
crc32('hello') -> 907060870
# Empty string
crc32('') -> 0
# Data with space
crc32('test data') -> 2706273274
```

**CLI Usage:**

```bash
echo '{}' | jpx 'crc32(`"hello"`)'
```

### hmac_md5

Calculate HMAC-MD5 signature

**Signature:** `string, string -> string`

**Examples:**

```text
# With secret key
hmac_md5('hello', 'secret') -> \"e17e4e4a205c55782dce5b6ff41e6e19\"
# Different message
hmac_md5('message', 'key') -> \"a24c903c3a7e7b741ea77bd467b98bca\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hmac_md5(`"hello"`, `"secret"`)'
```

### hmac_sha1

Calculate HMAC-SHA1 signature

**Signature:** `string, string -> string`

**Examples:**

```text
# With secret key
hmac_sha1('hello', 'secret') -> \"5112055c36b16a6693045d75a054332e4555b52f\"
# Different data
hmac_sha1('data', 'key') -> \"104152c5bfdca07bc633eebd46199f0255c9f49d\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hmac_sha1(`"hello"`, `"secret"`)'
```

### hmac_sha256

Calculate HMAC-SHA256 signature

**Signature:** `string, string -> string`

**Examples:**

```text
# With secret key
hmac_sha256('hello', 'secret') -> \"88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b\"
# Different data
hmac_sha256('data', 'key') -> \"5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hmac_sha256(`"hello"`, `"secret"`)'
```

### hmac_sha512

Calculate HMAC-SHA512 signature

**Signature:** `string, string -> string`

**Examples:**

```text
# With secret key
hmac_sha512('hello', 'secret') -> \"d05888a20ae...\"
# Different data
hmac_sha512('data', 'key') -> \"3c5953a18...\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hmac_sha512(`"hello"`, `"secret"`)'
```

### md5

Calculate MD5 hash

**Signature:** `string -> string`

**Examples:**

```text
# Simple string
md5('hello') -> \"5d41402abc4b2a76b9719d911017c592\"
# Empty string
md5('') -> \"d41d8cd98f00b204e9800998ecf8427e\"
# Another string
md5('test') -> \"098f6bcd4621d373cade4e832627b4f6\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'md5(`"hello"`)'
```

### sha1

Calculate SHA-1 hash

**Signature:** `string -> string`

**Examples:**

```text
# Simple string
sha1('hello') -> \"aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\"
# Empty string
sha1('') -> \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"
# Another string
sha1('test') -> \"a94a8fe5ccb19ba61c4c0873d391e987982fbbd3\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sha1(`"hello"`)'
```

### sha256

Calculate SHA-256 hash

**Signature:** `string -> string`

**Examples:**

```text
# Simple string
sha256('hello') -> \"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\"
# Empty string
sha256('') -> \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"
# Another string
sha256('test') -> \"9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sha256(`"hello"`)'
```

### sha512

Calculate SHA-512 hash

**Signature:** `string -> string`

**Examples:**

```text
# Simple string
sha512('hello') -> \"9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043\"
# Another string
sha512('test') -> \"ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sha512(`"hello"`)'
```

