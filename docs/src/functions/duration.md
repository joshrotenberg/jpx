# Duration Functions

Functions for working with time durations.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`duration_hours`](#duration-hours) | `number -> number` | Convert seconds to hours |
| [`duration_minutes`](#duration-minutes) | `number -> number` | Convert seconds to minutes |
| [`duration_seconds`](#duration-seconds) | `number -> number` | Get seconds component |
| [`format_duration`](#format-duration) | `number -> string` | Format seconds as duration string |
| [`parse_duration`](#parse-duration) | `string -> number` | Parse duration string to seconds |

## Functions

### duration_hours

Convert seconds to hours

**Signature:** `number -> number`

**Examples:**

```text
# 2 hours
duration_hours(`7200`) -> 2
# 1 hour
duration_hours(`3600`) -> 1
# 1.5 hours
duration_hours(`5400`) -> 1.5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'duration_hours(`7200`)'
```

### duration_minutes

Convert seconds to minutes

**Signature:** `number -> number`

**Examples:**

```text
# 2 minutes
duration_minutes(`120`) -> 2
# 1 minute
duration_minutes(`60`) -> 1
# 1.5 minutes
duration_minutes(`90`) -> 1.5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'duration_minutes(`120`)'
```

### duration_seconds

Get seconds component

**Signature:** `number -> number`

**Examples:**

```text
# 65 seconds mod 60
duration_seconds(`65`) -> 5
# Exact minutes
duration_seconds(`120`) -> 0
# 1 hour 1 min 1 sec
duration_seconds(`3661`) -> 1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'duration_seconds(`65`)'
```

### format_duration

Format seconds as duration string

**Signature:** `number -> string`

**Examples:**

```text
# 1.5 hours
format_duration(`5400`) -> \"1h30m\"
# 1 hour 1 min 1 sec
format_duration(`3661`) -> \"1h1m1s\"
# 1 minute
format_duration(`60`) -> \"1m\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'format_duration(`5400`)'
```

### parse_duration

Parse duration string to seconds

**Signature:** `string -> number`

**Examples:**

```text
# 1.5 hours
parse_duration('1h30m') -> 5400
# 2 hours
parse_duration('2h') -> 7200
# 30 seconds
parse_duration('30s') -> 30
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_duration(`"1h30m"`)'
```

