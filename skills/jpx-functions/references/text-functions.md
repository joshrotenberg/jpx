# Text Functions (21)

## `bigrams`

**Signature:** `string -> array`

Generate word bigrams (2-grams)

```
bigrams('a b c') -> \[\['a', 'b'\], \['b', 'c'\]\]
```
_Basic bigrams_

```
bigrams('the quick brown fox') -> \[\['the', 'quick'\], \['quick', 'brown'\], \['brown', 'fox'\]\]
```
_Sentence bigrams_

---

## `char_count`

**Signature:** `string -> number`

Count characters in text

```
char_count('hello') -> 5
```
_Simple word_

```
char_count('hello world') -> 11
```
_With space_

---

## `char_frequencies`

**Signature:** `string -> object`

Count character frequencies

```
char_frequencies('aab') -> {a: 2, b: 1}
```
_Count repeated chars_

```
char_frequencies('hello') -> {e: 1, h: 1, l: 2, o: 1}
```
_Word frequencies_

---

## `collapse_whitespace`

**Signature:** `string -> string`

Normalize whitespace (multiple spaces to single, trim)

```
collapse_whitespace('  hello   world  ') -> 'hello world'
```
_Collapse spaces_

```
collapse_whitespace('hello\t\nworld') -> 'hello world'
```
_Collapse tabs and newlines_

---

## `is_stopword`

**Signature:** `string, string? -> boolean`

Check if word is a stopword

```
is_stopword('the') -> true
```
_Common stopword_

```
is_stopword('elephant') -> false
```
_Not a stopword_

---

## `ngrams`

**Signature:** `string, number, string? -> array`

Generate n-grams from text (word or character)

```
ngrams('hello', `3`, 'char') -> \['hel', 'ell', 'llo'\]
```
_Character trigrams_

```
ngrams('a b c d', `2`, 'word') -> \[\['a', 'b'\], \['b', 'c'\], \['c', 'd'\]\]
```
_Word bigrams_

---

## `normalize_unicode`

**Signature:** `string, string? -> string`

Unicode normalization (NFC, NFD, NFKC, NFKD)

```
normalize_unicode('café') -> 'café'
```
_NFC normalization (default)_

```
normalize_unicode('ﬁ', 'NFKC') -> 'fi'
```
_Compatibility decomposition_

---

## `paragraph_count`

**Signature:** `string -> number`

Count paragraphs in text

```
paragraph_count('A\\n\\nB') -> 2
```
_Two paragraphs_

```
paragraph_count('Single paragraph') -> 1
```
_Single paragraph_

---

## `reading_time`

**Signature:** `string -> string`

Estimate reading time

```
reading_time('The quick brown fox') -> \"1 min read\"
```
_Short text_

```
reading_time('') -> \"1 min read\"
```
_Empty text minimum_

---

## `reading_time_seconds`

**Signature:** `string -> number`

Estimate reading time in seconds

```
reading_time_seconds('The quick brown fox jumps over the lazy dog') -> 2
```
_Short sentence_

```
reading_time_seconds('') -> 0
```
_Empty text_

---

## `remove_accents`

**Signature:** `string -> string`

Strip diacritics/accents from text

```
remove_accents('café') -> 'cafe'
```
_Remove accent_

```
remove_accents('naïve résumé') -> 'naive resume'
```
_Multiple accents_

---

## `remove_stopwords`

**Signature:** `array, string? -> array`

Remove stopwords from token array

```
remove_stopwords(\['the', 'quick', 'fox'\]) -> \['quick', 'fox'\]
```
_Remove English stopwords_

```
tokens('The quick brown fox') | remove_stopwords(@) -> \['quick', 'brown', 'fox'\]
```
_Pipeline_

---

## `sentence_count`

**Signature:** `string -> number`

Count sentences in text

```
sentence_count('Hello. World!') -> 2
```
_Two sentences_

```
sentence_count('One sentence') -> 1
```
_Single sentence_

---

## `stem`

**Signature:** `string, string? -> string`

Stem a word using Snowball stemmer (Porter algorithm)

```
stem('running') -> 'run'
```
_Basic stemming_

```
stem('cats') -> 'cat'
```
_Plural stemming_

---

## `stems`

**Signature:** `array, string? -> array`

Stem an array of tokens

```
stems(\['running', 'cats'\]) -> \['run', 'cat'\]
```
_Stem multiple words_

```
tokens('The cats are running') | stems(@) -> \['the', 'cat', 'are', 'run'\]
```
_Pipeline with tokens_

---

## `stopwords`

**Signature:** `string? -> array`

Get stopwords list for a language (default: English)

```
stopwords() | length(@) > `100` -> true
```
_English has many stopwords_

```
stopwords('es') | contains(@, 'el') -> true
```
_Spanish stopwords_

---

## `tokenize`

**Signature:** `string, object? -> array`

Configurable tokenization with options for case and punctuation handling

```
tokenize('Hello, World!') -> \['hello', 'world'\]
```
_Default (lowercase, strip punctuation)_

```
tokenize('Hello, World!', `{"case": "preserve"}`) -> \['Hello', 'World'\]
```
_Preserve case_

---

## `tokens`

**Signature:** `string -> array`

Simple word tokenization with normalization (lowercase, strip punctuation)

```
tokens('Hello, World!') -> \['hello', 'world'\]
```
_Basic tokenization_

```
tokens('The quick, brown fox!') -> \['the', 'quick', 'brown', 'fox'\]
```
_Strip punctuation_

---

## `trigrams`

**Signature:** `string -> array`

Generate word trigrams (3-grams)

```
trigrams('a b c d') -> \[\['a', 'b', 'c'\], \['b', 'c', 'd'\]\]
```
_Basic trigrams_

```
trigrams('the quick brown fox jumps') -> \[\['the', 'quick', 'brown'\], \['quick', 'brown', 'fox'\], \['brown', 'fox', 'jumps'\]\]
```
_Sentence trigrams_

---

## `word_count`

**Signature:** `string -> number`

Count words in text

```
word_count('hello world') -> 2
```
_Two words_

```
word_count('one') -> 1
```
_Single word_

---

## `word_frequencies`

**Signature:** `string -> object`

Count word frequencies

```
word_frequencies('a a b') -> {a: 2, b: 1}
```
_Count repeated words_

```
word_frequencies('the quick brown fox') -> {brown: 1, fox: 1, quick: 1, the: 1}
```
_Unique words_

---

