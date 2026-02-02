# Text Functions

Text analysis and processing functions for NLP pipelines.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`bigrams`](#bigrams) | `string -> array` | Generate word bigrams (2-grams) |
| [`char_count`](#char-count) | `string -> number` | Count characters in text |
| [`char_frequencies`](#char-frequencies) | `string -> object` | Count character frequencies |
| [`collapse_whitespace`](#collapse-whitespace) | `string -> string` | Normalize whitespace |
| [`is_stopword`](#is-stopword) | `string, string? -> boolean` | Check if word is a stopword |
| [`ngrams`](#ngrams) | `string, number, string? -> array` | Generate n-grams from text (word or character) |
| [`normalize_unicode`](#normalize-unicode) | `string, string? -> string` | Unicode normalization (NFC/NFD/NFKC/NFKD) |
| [`paragraph_count`](#paragraph-count) | `string -> number` | Count paragraphs in text |
| [`reading_time`](#reading-time) | `string -> string` | Estimate reading time |
| [`reading_time_seconds`](#reading-time-seconds) | `string -> number` | Estimate reading time in seconds |
| [`remove_accents`](#remove-accents) | `string -> string` | Strip diacritics from text |
| [`remove_stopwords`](#remove-stopwords) | `array, string? -> array` | Filter stopwords from token array |
| [`sentence_count`](#sentence-count) | `string -> number` | Count sentences in text |
| [`stem`](#stem) | `string, string? -> string` | Stem a word (17 languages) |
| [`stems`](#stems) | `array, string? -> array` | Stem array of tokens |
| [`stopwords`](#stopwords) | `string? -> array` | Get stopword list for language |
| [`tokenize`](#tokenize) | `string, object? -> array` | Configurable tokenization |
| [`tokens`](#tokens) | `string -> array` | Simple word tokenization |
| [`trigrams`](#trigrams) | `string -> array` | Generate word trigrams (3-grams) |
| [`word_count`](#word-count) | `string -> number` | Count words in text |
| [`word_frequencies`](#word-frequencies) | `string -> object` | Count word frequencies |

## Functions

### bigrams

Generate word bigrams (2-grams)

**Signature:** `string -> array`

**Examples:**

```text
# Basic bigrams
bigrams('a b c') -> [['a', 'b'], ['b', 'c']]
# Sentence bigrams
bigrams('the quick brown fox') -> [['the', 'quick'], ['quick', 'brown'], ['brown', 'fox']]
# Single word
bigrams('single') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx "bigrams('a b c')"
```

### char_count

Count characters in text

**Signature:** `string -> number`

**Examples:**

```text
# Simple word
char_count('hello') -> 5
# With space
char_count('hello world') -> 11
# Empty string
char_count('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx "char_count('hello')"
```

### char_frequencies

Count character frequencies

**Signature:** `string -> object`

**Examples:**

```text
# Count repeated chars
char_frequencies('aab') -> {a: 2, b: 1}
# Word frequencies
char_frequencies('hello') -> {e: 1, h: 1, l: 2, o: 1}
# Empty string
char_frequencies('') -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx "char_frequencies('aab')"
```

### collapse_whitespace

Normalize whitespace by collapsing multiple spaces, tabs, and newlines into single spaces.

**Signature:** `string -> string`

**Examples:**

```text
# Multiple spaces
collapse_whitespace('hello    world') -> 'hello world'
# Tabs and newlines
collapse_whitespace('a\t\nb') -> 'a b'
# Leading/trailing whitespace
collapse_whitespace('  hello  ') -> 'hello'
```

**CLI Usage:**

```bash
echo '"hello    world"' | jpx 'collapse_whitespace(@)'
```

### is_stopword

Check if a word is a stopword in the specified language.

**Signature:** `string, string? -> boolean`

**Parameters:**
- `word` - The word to check
- `lang` - Language code (default: "en"). Supports 30+ languages.

**Examples:**

```text
# English stopword
is_stopword('the') -> true
# Not a stopword
is_stopword('algorithm') -> false
# Spanish
is_stopword('el', 'es') -> true
# German
is_stopword('und', 'de') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx "is_stopword('the')"
```

### ngrams

Generate n-grams from text (word or character)

**Signature:** `string, number, string? -> array`

**Examples:**

```text
# Character trigrams
ngrams('hello', `3`, 'char') -> ['hel', 'ell', 'llo']
# Word bigrams
ngrams('a b c d', `2`, 'word') -> [['a', 'b'], ['b', 'c'], ['c', 'd']]
# Text shorter than n
ngrams('ab', `3`, 'char') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx "ngrams('hello', \`3\`, 'char')"
```

### normalize_unicode

Apply Unicode normalization to text.

**Signature:** `string, string? -> string`

**Parameters:**
- `text` - The text to normalize
- `form` - Normalization form: "nfc" (default), "nfd", "nfkc", or "nfkd"

**Examples:**

```text
# NFC (composed, default)
normalize_unicode('café') -> 'café'
# NFD (decomposed)
normalize_unicode('é', 'nfd') -> 'é'  # e + combining acute
# NFKC (compatibility composed)
normalize_unicode('ﬁ', 'nfkc') -> 'fi'
```

**CLI Usage:**

```bash
echo '"café"' | jpx "normalize_unicode(@)"
```

### paragraph_count

Count paragraphs in text

**Signature:** `string -> number`

**Examples:**

```text
# Two paragraphs
paragraph_count('A\n\nB') -> 2
# Single paragraph
paragraph_count('Single paragraph') -> 1
# Three paragraphs
paragraph_count('A\n\nB\n\nC') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx "paragraph_count('A\n\nB')"
```

### reading_time

Estimate reading time

**Signature:** `string -> string`

**Examples:**

```text
# Short text
reading_time('The quick brown fox') -> "1 min read"
# Empty text minimum
reading_time('') -> "1 min read"
```

**CLI Usage:**

```bash
echo '{}' | jpx "reading_time('The quick brown fox')"
```

### reading_time_seconds

Estimate reading time in seconds

**Signature:** `string -> number`

**Examples:**

```text
# Short sentence
reading_time_seconds('The quick brown fox jumps over the lazy dog') -> 2
# Empty text
reading_time_seconds('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx "reading_time_seconds('The quick brown fox jumps over the lazy dog')"
```

### remove_accents

Strip diacritical marks (accents) from text.

**Signature:** `string -> string`

**Examples:**

```text
# French
remove_accents('café') -> 'cafe'
# Spanish
remove_accents('señor') -> 'senor'
# German
remove_accents('über') -> 'uber'
# Mixed
remove_accents('naïve résumé') -> 'naive resume'
```

**CLI Usage:**

```bash
echo '"café"' | jpx 'remove_accents(@)'
```

### remove_stopwords

Filter stopwords from an array of tokens.

**Signature:** `array, string? -> array`

**Parameters:**
- `tokens` - Array of word tokens
- `lang` - Language code (default: "en")

**Examples:**

```text
# English
remove_stopwords(['the', 'quick', 'brown', 'fox']) -> ['quick', 'brown', 'fox']
# Complete sentence
remove_stopwords(['i', 'am', 'learning', 'rust']) -> ['learning', 'rust']
# Spanish
remove_stopwords(['el', 'gato', 'negro'], 'es') -> ['gato', 'negro']
```

**CLI Usage:**

```bash
echo '["the", "quick", "brown", "fox"]' | jpx "remove_stopwords(@)"
```

### sentence_count

Count sentences in text

**Signature:** `string -> number`

**Examples:**

```text
# Two sentences
sentence_count('Hello. World!') -> 2
# Single sentence
sentence_count('One sentence') -> 1
# Different terminators
sentence_count('What? Yes! No.') -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx "sentence_count('Hello. World!')"
```

### stem

Stem a single word using the Snowball stemmer.

**Signature:** `string, string? -> string`

**Parameters:**
- `word` - The word to stem
- `lang` - Language code (default: "en"). Supports 17 languages: ar, da, de, el, en, es, fi, fr, hu, it, nl, no, pt, ro, ru, sv, tr.

**Examples:**

```text
# English
stem('running') -> 'run'
stem('connections') -> 'connect'
stem('happiness') -> 'happi'
# Spanish
stem('corriendo', 'es') -> 'corr'
# German
stem('laufend', 'de') -> 'lauf'
```

**CLI Usage:**

```bash
echo '{}' | jpx "stem('running')"
```

### stems

Stem an array of tokens.

**Signature:** `array, string? -> array`

**Parameters:**
- `tokens` - Array of word tokens
- `lang` - Language code (default: "en")

**Examples:**

```text
# Stem multiple words
stems(['running', 'jumps', 'walked']) -> ['run', 'jump', 'walk']
# Spanish
stems(['corriendo', 'saltando'], 'es') -> ['corr', 'salt']
```

**CLI Usage:**

```bash
echo '["running", "jumps", "walked"]' | jpx "stems(@)"
```

### stopwords

Get the list of stopwords for a language.

**Signature:** `string? -> array`

**Parameters:**
- `lang` - Language code (default: "en"). Supports 30+ languages.

**Examples:**

```text
# English stopwords (first few)
stopwords() | slice(@, `0`, `5`) -> ['i', 'me', 'my', 'myself', 'we']
# Check count
stopwords() | length(@) -> 179
# Spanish
stopwords('es') | length(@) -> 313
# German
stopwords('de') | slice(@, `0`, `3`) -> ['aber', 'alle', 'allem']
```

**CLI Usage:**

```bash
echo '{}' | jpx "stopwords() | length(@)"
```

### tokenize

Tokenize text with configurable options.

**Signature:** `string, object? -> array`

**Parameters:**
- `text` - The text to tokenize
- `options` - Optional configuration object:
  - `case`: "lower" (default) or "preserve"
  - `punctuation`: "strip" (default) or "keep"

**Examples:**

```text
# Default (lowercase, strip punctuation)
tokenize('Hello, World!') -> ['hello', 'world']
# Preserve case
tokenize('Hello World', `{"case": "preserve"}`) -> ['Hello', 'World']
# Keep punctuation
tokenize('Hello, World!', `{"punctuation": "keep"}`) -> ['hello,', 'world!']
# Both options
tokenize('Hello!', `{"case": "preserve", "punctuation": "keep"}`) -> ['Hello!']
```

**CLI Usage:**

```bash
echo '"Hello, World!"' | jpx 'tokenize(@)'
```

### tokens

Simple word tokenization (normalized, lowercase, punctuation stripped).

**Signature:** `string -> array`

**Examples:**

```text
# Basic tokenization
tokens('Hello, World!') -> ['hello', 'world']
# Multiple words
tokens('The quick brown fox') -> ['the', 'quick', 'brown', 'fox']
# Numbers included
tokens('Test 123') -> ['test', '123']
```

**CLI Usage:**

```bash
echo '"Hello, World!"' | jpx 'tokens(@)'
```

### trigrams

Generate word trigrams (3-grams)

**Signature:** `string -> array`

**Examples:**

```text
# Basic trigrams
trigrams('a b c d') -> [['a', 'b', 'c'], ['b', 'c', 'd']]
# Sentence trigrams
trigrams('the quick brown fox jumps') -> [['the', 'quick', 'brown'], ['quick', 'brown', 'fox'], ['brown', 'fox', 'jumps']]
# Too few words
trigrams('a b') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx "trigrams('a b c d')"
```

### word_count

Count words in text

**Signature:** `string -> number`

**Examples:**

```text
# Two words
word_count('hello world') -> 2
# Single word
word_count('one') -> 1
# Empty string
word_count('') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx "word_count('hello world')"
```

### word_frequencies

Count word frequencies

**Signature:** `string -> object`

**Examples:**

```text
# Count repeated words
word_frequencies('a a b') -> {a: 2, b: 1}
# Unique words
word_frequencies('the quick brown fox') -> {brown: 1, fox: 1, quick: 1, the: 1}
# Empty string
word_frequencies('') -> {}
```

**CLI Usage:**

```bash
echo '{}' | jpx "word_frequencies('a a b')"
```
