# JSON Patch Functions

JSON Patch (RFC 6902) functions: applying patches, generating diffs, and path operations.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`json_diff`](#json-diff) | `object, object -> array` | Generate JSON Patch (RFC 6902) that transforms first object into second |
| [`json_merge_patch`](#json-merge-patch) | `object, object -> object` | Apply JSON Merge Patch (RFC 7396) to an object |
| [`json_patch`](#json-patch) | `object, array -> object` | Apply JSON Patch (RFC 6902) operations to an object |

## Functions

### json_diff

Generate JSON Patch (RFC 6902) that transforms first object into second

**Signature:** `object, object -> array`

**Examples:**

```text
# Replace value
json_diff({a: 1}, {a: 2}) -> [{op: 'replace', path: '/a', value: 2}]
# Add field
json_diff({a: 1}, {a: 1, b: 2}) -> [{op: 'add', path: '/b', value: 2}]
# Remove field
json_diff({a: 1, b: 2}, {a: 1}) -> [{op: 'remove', path: '/b'}]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_diff({a: 1}, {a: 2})'
```

### json_merge_patch

Apply JSON Merge Patch (RFC 7396) to an object

**Signature:** `object, object -> object`

**Examples:**

```text
# Merge objects
json_merge_patch({a: 1, b: 2}, {b: 3, c: 4}) -> {a: 1, b: 3, c: 4}
# Null removes field
json_merge_patch({a: 1}, {a: `null`}) -> {}
# Add to empty
json_merge_patch({}, {a: 1}) -> {a: 1}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_merge_patch({a: 1, b: 2}, {b: 3, c: 4})'
```

### json_patch

Apply JSON Patch (RFC 6902) operations to an object

**Signature:** `object, array -> object`

**Examples:**

```text
# Add operation
json_patch({a: 1}, [{op: 'add', path: '/b', value: 2}]) -> {a: 1, b: 2}
# Replace operation
json_patch({a: 1}, [{op: 'replace', path: '/a', value: 2}]) -> {a: 2}
# Remove operation
json_patch({a: 1, b: 2}, [{op: 'remove', path: '/b'}]) -> {a: 1}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'json_patch({a: 1}, [{op: 'add', path: '/b', value: 2}])'
```

