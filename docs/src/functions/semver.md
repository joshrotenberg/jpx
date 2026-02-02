# Semantic Versioning Functions

Semantic versioning functions: parsing, comparing, and manipulating version strings.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`semver_compare`](#semver-compare) | `string, string -> number` | Compare versions (-1, 0, 1) |
| [`semver_is_valid`](#semver-is-valid) | `string -> boolean` | Check if string is valid semver |
| [`semver_major`](#semver-major) | `string -> number` | Get major version |
| [`semver_minor`](#semver-minor) | `string -> number` | Get minor version |
| [`semver_parse`](#semver-parse) | `string -> object` | Parse semantic version |
| [`semver_patch`](#semver-patch) | `string -> number` | Get patch version |
| [`semver_satisfies`](#semver-satisfies) | `string, string -> boolean` | Check if version matches constraint |

## Functions

### semver_compare

Compare versions (-1, 0, 1)

**Signature:** `string, string -> number`

**Examples:**

```text
# Less than
semver_compare('1.0.0', '2.0.0') -> -1
# Greater than
semver_compare('2.0.0', '1.0.0') -> 1
# Equal
semver_compare('1.0.0', '1.0.0') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_compare(`"1.0.0"`, `"2.0.0"`)'
```

### semver_is_valid

Check if string is valid semver

**Signature:** `string -> boolean`

**Examples:**

```text
# Valid semver
semver_is_valid('1.2.3') -> true
# With prerelease
semver_is_valid('1.2.3-alpha') -> true
# Invalid format
semver_is_valid('invalid') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_is_valid(`"1.2.3"`)'
```

### semver_major

Get major version

**Signature:** `string -> number`

**Examples:**

```text
# Major version
semver_major('1.2.3') -> 1
# Double digit
semver_major('10.0.0') -> 10
# Zero major
semver_major('0.1.0') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_major(`"1.2.3"`)'
```

### semver_minor

Get minor version

**Signature:** `string -> number`

**Examples:**

```text
# Minor version
semver_minor('1.2.3') -> 2
# Double digit
semver_minor('1.10.0') -> 10
# Zero minor
semver_minor('1.0.0') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_minor(`"1.2.3"`)'
```

### semver_parse

Parse semantic version

**Signature:** `string -> object`

**Examples:**

```text
# Basic version
semver_parse('1.2.3') -> {major: 1, minor: 2, patch: 3}
# With prerelease
semver_parse('1.2.3-alpha').pre -> 'alpha'
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_parse(`"1.2.3"`)'
```

### semver_patch

Get patch version

**Signature:** `string -> number`

**Examples:**

```text
# Patch version
semver_patch('1.2.3') -> 3
# Double digit
semver_patch('1.2.10') -> 10
# Zero patch
semver_patch('1.2.0') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_patch(`"1.2.3"`)'
```

### semver_satisfies

Check if version matches constraint

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Caret range
semver_satisfies('1.2.3', '^1.0.0') -> true
# Outside range
semver_satisfies('2.0.0', '^1.0.0') -> false
# Comparison
semver_satisfies('1.5.0', '>=1.0.0') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'semver_satisfies(`"1.2.3"`, `"^1.0.0"`)'
```

