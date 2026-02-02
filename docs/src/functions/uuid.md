# UUID Functions

Functions for generating and working with UUIDs.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`uuid`](#uuid) | `-> string` | Generate a UUID v4 |

## Functions

### uuid

Generate a UUID v4

**Signature:** `-> string`

**Examples:**

```text
# Generate UUID
uuid() -> \"550e8400-e29b-41d4-a716-446655440000\"
# Each call is unique
uuid() -> random unique ID
# Add ID to object
{id: uuid(), name: 'item'} -> with UUID
```

**CLI Usage:**

```bash
echo '{}' | jpx 'uuid()'
```

