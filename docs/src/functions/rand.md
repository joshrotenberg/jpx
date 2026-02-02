# Random Functions

Functions for generating random values: numbers, strings, and selections.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`random`](#random) | `-> number` | Generate random number between 0 and 1 |
| [`sample`](#sample) | `array, number -> array` | Random sample from array |
| [`shuffle`](#shuffle) | `array -> array` | Randomly shuffle array |

## Functions

### random

Generate random number between 0 and 1

**Signature:** `-> number`

**Examples:**

```text
# Random float
random() -> 0.123456...
# Random 0-99
floor(multiply(random(), `100`)) -> 42
```

**CLI Usage:**

```bash
echo '{}' | jpx 'random()'
```

### sample

Random sample from array

**Signature:** `array, number -> array`

**Examples:**

```text
# Sample 2 items
sample([1, 2, 3, 4], `2`) -> [3, 1]
# Sample 1 item
sample(['a', 'b', 'c'], `1`) -> ['b']
# Always returns n items
length(sample([1, 2, 3], `2`)) -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sample([1, 2, 3, 4], `2`)'
```

### shuffle

Randomly shuffle array

**Signature:** `array -> array`

**Examples:**

```text
# Shuffle numbers
shuffle([1, 2, 3]) -> [2, 3, 1]
# Preserves length
length(shuffle([1, 2, 3])) -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shuffle([1, 2, 3])'
```

