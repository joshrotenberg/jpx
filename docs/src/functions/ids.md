# ID Generation Functions

Functions for generating various types of unique identifiers.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`nanoid`](#nanoid) | `number? -> string` | Generate nanoid |
| [`ulid`](#ulid) | `-> string` | Generate ULID |
| [`ulid_timestamp`](#ulid-timestamp) | `string -> number` | Extract timestamp from ULID |

## Functions

### nanoid

Generate nanoid

**Signature:** `number? -> string`

**Examples:**

```text
# Default 21 chars
nanoid() -> \"V1StGXR8_Z5jdHi6B-myT\"
# Custom length
nanoid(`10`) -> \"IRFa-VaY2b\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'nanoid()'
```

### ulid

Generate ULID

**Signature:** `-> string`

**Examples:**

```text
# Generate ULID
ulid() -> \"01ARZ3NDEKTSV4RRFFQ69G5FAV\"
# Always 26 chars
length(ulid()) -> 26
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ulid()'
```

### ulid_timestamp

Extract timestamp from ULID

**Signature:** `string -> number`

**Examples:**

```text
# Extract timestamp
ulid_timestamp('01ARZ3NDEKTSV4RRFFQ69G5FAV') -> 1469918176385
# Another ULID
ulid_timestamp('01BX5ZZKBKACTAV9WEVGEMMVRY') -> 1484581420610
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ulid_timestamp(`"01ARZ3NDEKTSV4RRFFQ69G5FAV"`)'
```

