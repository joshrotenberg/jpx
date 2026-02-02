# Fuzzy Functions

Fuzzy matching and string similarity functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`damerau_levenshtein`](#damerau-levenshtein) | `string, string -> number` | Damerau-Levenshtein distance |
| [`hamming`](#hamming) | `string, string -> number\|null` | Hamming distance (number of differing positions). Returns null if strings have different lengths |
| [`jaro`](#jaro) | `string, string -> number` | Jaro similarity (0-1) |
| [`jaro_winkler`](#jaro-winkler) | `string, string -> number` | Jaro-Winkler similarity (0-1) |
| [`levenshtein`](#levenshtein) | `string, string -> number` | Levenshtein edit distance |
| [`normalized_damerau_levenshtein`](#normalized-damerau-levenshtein) | `string, string -> number` | Normalized Damerau-Levenshtein similarity (0-1) |
| [`normalized_levenshtein`](#normalized-levenshtein) | `string, string -> number` | Normalized Levenshtein (0-1) |
| [`osa_distance`](#osa-distance) | `string, string -> number` | Optimal String Alignment distance (like Levenshtein but allows adjacent transpositions) |
| [`sorensen_dice`](#sorensen-dice) | `string, string -> number` | Sorensen-Dice coefficient (0-1) |

## Functions

### damerau_levenshtein

Damerau-Levenshtein distance

**Signature:** `string, string -> number`

**Examples:**

```text
# Single transposition
damerau_levenshtein('ab', 'ba') -> 1
# Identical strings
damerau_levenshtein('hello', 'hello') -> 0
# Multiple edits
damerau_levenshtein('ca', 'abc') -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'damerau_levenshtein(`"ab"`, `"ba"`)'
```

### hamming

Hamming distance (number of differing positions). Returns null if strings have different lengths

**Signature:** `string, string -> number|null`

**Examples:**

```text
# Three differing positions
hamming('karolin', 'kathrin') -> 3
# Identical strings
hamming('hello', 'hello') -> 0
# Different lengths
hamming('hello', 'hi') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hamming(`"karolin"`, `"kathrin"`)'
```

### jaro

Jaro similarity (0-1)

**Signature:** `string, string -> number`

**Examples:**

```text
# Similar words
jaro('hello', 'hallo') -> 0.866...
# Identical strings
jaro('hello', 'hello') -> 1.0
# Completely different
jaro('abc', 'xyz') -> 0.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'jaro(`"hello"`, `"hallo"`)'
```

### jaro_winkler

Jaro-Winkler similarity (0-1)

**Signature:** `string, string -> number`

**Examples:**

```text
# Similar words
jaro_winkler('hello', 'hallo') -> 0.88
# Identical strings
jaro_winkler('hello', 'hello') -> 1.0
# Completely different
jaro_winkler('abc', 'xyz') -> 0.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'jaro_winkler(`"hello"`, `"hallo"`)'
```

### levenshtein

Levenshtein edit distance

**Signature:** `string, string -> number`

**Examples:**

```text
# Classic example
levenshtein('kitten', 'sitting') -> 3
# Identical strings
levenshtein('hello', 'hello') -> 0
# All different
levenshtein('abc', 'def') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'levenshtein(`"kitten"`, `"sitting"`)'
```

### normalized_damerau_levenshtein

Normalized Damerau-Levenshtein similarity (0-1)

**Signature:** `string, string -> number`

**Examples:**

```text
# Identical strings
normalized_damerau_levenshtein('hello', 'hello') -> 1.0
# Transposition
normalized_damerau_levenshtein('ab', 'ba') -> 0.5
# Completely different
normalized_damerau_levenshtein('abc', 'xyz') -> 0.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'normalized_damerau_levenshtein(`"hello"`, `"hello"`)'
```

### normalized_levenshtein

Normalized Levenshtein (0-1)

**Signature:** `string, string -> number`

**Examples:**

```text
# One edit
normalized_levenshtein('ab', 'abc') -> 0.666...
# Identical
normalized_levenshtein('hello', 'hello') -> 0.0
# All different
normalized_levenshtein('abc', 'xyz') -> 1.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'normalized_levenshtein(`"ab"`, `"abc"`)'
```

### osa_distance

Optimal String Alignment distance (like Levenshtein but allows adjacent transpositions)

**Signature:** `string, string -> number`

**Examples:**

```text
# Single transposition
osa_distance('ab', 'ba') -> 1
# Identical strings
osa_distance('hello', 'hello') -> 0
# Multiple edits
osa_distance('ca', 'abc') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'osa_distance(`"ab"`, `"ba"`)'
```

### sorensen_dice

Sorensen-Dice coefficient (0-1)

**Signature:** `string, string -> number`

**Examples:**

```text
# Similar words
sorensen_dice('night', 'nacht') -> 0.25
# Identical strings
sorensen_dice('hello', 'hello') -> 1.0
# No common bigrams
sorensen_dice('abc', 'xyz') -> 0.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sorensen_dice(`"night"`, `"nacht"`)'
```

