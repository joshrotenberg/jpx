# Phonetic Functions

Phonetic encoding functions for sound-based string matching.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`caverphone`](#caverphone) | `string -> string` | Caverphone code |
| [`caverphone2`](#caverphone2) | `string -> string` | Caverphone 2 code |
| [`double_metaphone`](#double-metaphone) | `string -> object` | Double Metaphone codes |
| [`match_rating_codex`](#match-rating-codex) | `string -> string` | Match Rating codex |
| [`metaphone`](#metaphone) | `string -> string` | Metaphone phonetic code |
| [`nysiis`](#nysiis) | `string -> string` | NYSIIS phonetic code |
| [`phonetic_match`](#phonetic-match) | `string, string, string -> boolean` | Check phonetic match with algorithm |
| [`soundex`](#soundex) | `string -> string` | Soundex phonetic code |
| [`sounds_like`](#sounds-like) | `string, string -> boolean` | Check if strings sound similar |

## Functions

### caverphone

Caverphone code

**Signature:** `string -> string`

**Examples:**

```text
# Common name
caverphone('Smith') -> \"SMT1111111\"
# Another name
caverphone('Thompson') -> \"TMPSN11111\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'caverphone(`"Smith"`)'
```

### caverphone2

Caverphone 2 code

**Signature:** `string -> string`

**Examples:**

```text
# Common name
caverphone2('Smith') -> \"SMT1111111\"
# Another name
caverphone2('Thompson') -> \"TMPSN11111\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'caverphone2(`"Smith"`)'
```

### double_metaphone

Double Metaphone codes

**Signature:** `string -> object`

**Examples:**

```text
# Common name
double_metaphone('Smith') -> {primary: 'SM0', secondary: 'XMT'}
# German variant
double_metaphone('Schmidt') -> {primary: 'XMT', secondary: 'SMT'}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'double_metaphone(`"Smith"`)'
```

### match_rating_codex

Match Rating codex

**Signature:** `string -> string`

**Examples:**

```text
# Common name
match_rating_codex('Smith') -> \"SMTH\"
# Another name
match_rating_codex('Johnson') -> \"JHNSN\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'match_rating_codex(`"Smith"`)'
```

### metaphone

Metaphone phonetic code

**Signature:** `string -> string`

**Examples:**

```text
# Common name
metaphone('Smith') -> \"SM0\"
# Ph sound
metaphone('phone') -> \"FN\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'metaphone(`"Smith"`)'
```

### nysiis

NYSIIS phonetic code

**Signature:** `string -> string`

**Examples:**

```text
# Common name
nysiis('Smith') -> \"SNAT\"
# Another name
nysiis('Johnson') -> \"JANSAN\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'nysiis(`"Smith"`)'
```

### phonetic_match

Check phonetic match with algorithm

**Signature:** `string, string, string -> boolean`

**Examples:**

```text
# Soundex match
phonetic_match('Smith', 'Smyth', 'soundex') -> true
# Metaphone match
phonetic_match('Robert', 'Rupert', 'metaphone') -> true
# No match
phonetic_match('John', 'Jane', 'soundex') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'phonetic_match(`"Smith"`, `"Smyth"`, `"soundex"`)'
```

### soundex

Soundex phonetic code

**Signature:** `string -> string`

**Examples:**

```text
# Common name
soundex('Robert') -> \"R163\"
# Same code as Robert
soundex('Rupert') -> \"R163\"
# Another name
soundex('Smith') -> \"S530\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'soundex(`"Robert"`)'
```

### sounds_like

Check if strings sound similar

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Similar sounding
sounds_like('Robert', 'Rupert') -> true
# Spelling variants
sounds_like('Smith', 'Smyth') -> true
# Different names
sounds_like('John', 'Mary') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sounds_like(`"Robert"`, `"Rupert"`)'
```

