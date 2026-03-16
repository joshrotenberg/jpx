# Hash Functions (9)

## `crc32`

**Signature:** `string -> number`

Calculate CRC32 checksum

```
crc32('hello') -> 907060870
```
_Simple string_

```
crc32('') -> 0
```
_Empty string_

---

## `hmac_md5`

**Signature:** `string, string -> string`

Calculate HMAC-MD5 signature

```
hmac_md5('hello', 'secret') -> \"e17e4e4a205c55782dce5b6ff41e6e19\"
```
_With secret key_

```
hmac_md5('message', 'key') -> \"a24c903c3a7e7b741ea77bd467b98bca\"
```
_Different message_

---

## `hmac_sha1`

**Signature:** `string, string -> string`

Calculate HMAC-SHA1 signature

```
hmac_sha1('hello', 'secret') -> \"5112055c36b16a6693045d75a054332e4555b52f\"
```
_With secret key_

```
hmac_sha1('data', 'key') -> \"104152c5bfdca07bc633eebd46199f0255c9f49d\"
```
_Different data_

---

## `hmac_sha256`

**Signature:** `string, string -> string`

Calculate HMAC-SHA256 signature

```
hmac_sha256('hello', 'secret') -> \"88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b\"
```
_With secret key_

```
hmac_sha256('data', 'key') -> \"5031fe3d989c6d1537a013fa6e739da23463fdaec3b70137d828e36ace221bd0\"
```
_Different data_

---

## `hmac_sha512`

**Signature:** `string, string -> string`

Calculate HMAC-SHA512 signature

```
hmac_sha512('hello', 'secret') -> \"d05888a20ae...\"
```
_With secret key_

```
hmac_sha512('data', 'key') -> \"3c5953a18...\"
```
_Different data_

---

## `md5`

**Signature:** `string -> string`

Calculate MD5 hash

```
md5('hello') -> \"5d41402abc4b2a76b9719d911017c592\"
```
_Simple string_

```
md5('') -> \"d41d8cd98f00b204e9800998ecf8427e\"
```
_Empty string_

---

## `sha1`

**Signature:** `string -> string`

Calculate SHA-1 hash

```
sha1('hello') -> \"aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\"
```
_Simple string_

```
sha1('') -> \"da39a3ee5e6b4b0d3255bfef95601890afd80709\"
```
_Empty string_

---

## `sha256`

**Signature:** `string -> string`

Calculate SHA-256 hash

```
sha256('hello') -> \"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\"
```
_Simple string_

```
sha256('') -> \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"
```
_Empty string_

---

## `sha512`

**Signature:** `string -> string`

Calculate SHA-512 hash

```
sha512('hello') -> \"9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043\"
```
_Simple string_

```
sha512('test') -> \"ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff\"
```
_Another string_

---

