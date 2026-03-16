# Fuzzy Functions (9)

## `damerau_levenshtein`

**Signature:** `string, string -> number`

Damerau-Levenshtein distance

```
damerau_levenshtein('ab', 'ba') -> 1
```
_Single transposition_

```
damerau_levenshtein('hello', 'hello') -> 0
```
_Identical strings_

---

## `hamming`

**Signature:** `string, string -> number|null`

Hamming distance (number of differing positions). Returns null if strings have different lengths

```
hamming('karolin', 'kathrin') -> 3
```
_Three differing positions_

```
hamming('hello', 'hello') -> 0
```
_Identical strings_

---

## `jaro`

**Signature:** `string, string -> number`

Jaro similarity (0-1)

```
jaro('hello', 'hallo') -> 0.866...
```
_Similar words_

```
jaro('hello', 'hello') -> 1.0
```
_Identical strings_

---

## `jaro_winkler`

**Signature:** `string, string -> number`

Jaro-Winkler similarity (0-1)

```
jaro_winkler('hello', 'hallo') -> 0.88
```
_Similar words_

```
jaro_winkler('hello', 'hello') -> 1.0
```
_Identical strings_

---

## `levenshtein`

**Signature:** `string, string -> number`

Levenshtein edit distance

```
levenshtein('kitten', 'sitting') -> 3
```
_Classic example_

```
levenshtein('hello', 'hello') -> 0
```
_Identical strings_

---

## `normalized_damerau_levenshtein`

**Signature:** `string, string -> number`

Normalized Damerau-Levenshtein similarity (0-1)

```
normalized_damerau_levenshtein('hello', 'hello') -> 1.0
```
_Identical strings_

```
normalized_damerau_levenshtein('ab', 'ba') -> 0.5
```
_Transposition_

---

## `normalized_levenshtein`

**Signature:** `string, string -> number`

Normalized Levenshtein (0-1)

```
normalized_levenshtein('ab', 'abc') -> 0.666...
```
_One edit_

```
normalized_levenshtein('hello', 'hello') -> 0.0
```
_Identical_

---

## `osa_distance`

**Signature:** `string, string -> number`

Optimal String Alignment distance (like Levenshtein but allows adjacent transpositions)

```
osa_distance('ab', 'ba') -> 1
```
_Single transposition_

```
osa_distance('hello', 'hello') -> 0
```
_Identical strings_

---

## `sorensen_dice`

**Signature:** `string, string -> number`

Sorensen-Dice coefficient (0-1)

```
sorensen_dice('night', 'nacht') -> 0.25
```
_Similar words_

```
sorensen_dice('hello', 'hello') -> 1.0
```
_Identical strings_

---

