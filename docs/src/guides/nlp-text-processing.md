# NLP Text Processing

jpx includes a comprehensive NLP toolkit for text analysis and processing. This guide demonstrates how to build text processing pipelines combining tokenization, stemming, stopword removal, and more.

## NLP Pipeline Overview

A typical NLP preprocessing pipeline:

```
Raw Text → Tokenize → Remove Stopwords → Stem → Analyze
```

jpx makes this composable with pipes:

```bash
echo '"The quick brown foxes are running quickly"' | \
  jpx 'tokens(@) | remove_stopwords(@) | stems(@)'
# ["quick", "brown", "fox", "run", "quick"]
```

---

## Tokenization

### Basic Tokenization

The `tokens` function provides simple, normalized tokenization:

```bash
echo '"Hello, World! This is a TEST."' | jpx 'tokens(@)'
# ["hello", "world", "this", "is", "a", "test"]
```

### Configurable Tokenization

Use `tokenize` for more control:

```bash
# Preserve case
echo '"Hello World"' | jpx 'tokenize(@, `{"case": "preserve"}`)'
# ["Hello", "World"]

# Keep punctuation
echo '"Hello, World!"' | jpx 'tokenize(@, `{"punctuation": "keep"}`)'
# ["hello,", "world!"]

# Both options
echo '"Hello, World!"' | jpx 'tokenize(@, `{"case": "preserve", "punctuation": "keep"}`)'
# ["Hello,", "World!"]
```

---

## Stop Word Removal

Stop words are common words (the, is, at, which) that often don't add meaning for analysis.

### Check Stopwords

```bash
# Get English stopwords
echo '{}' | jpx 'stopwords() | length(@)'
# 179

# Check specific words
echo '{}' | jpx 'is_stopword(`"the"`)'
# true

echo '{}' | jpx 'is_stopword(`"algorithm"`)'
# false
```

### Filter Stopwords from Tokens

```bash
echo '["the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]' | \
  jpx 'remove_stopwords(@)'
# ["quick", "brown", "fox", "jumps", "lazy", "dog"]
```

### Multilingual Support

Stopwords are available in 30+ languages:

```bash
# Spanish
echo '{}' | jpx 'stopwords(`"es"`) | slice(@, `0`, `5`)'
# ["de", "la", "que", "el", "en"]

# German
echo '{}' | jpx 'stopwords(`"de"`) | slice(@, `0`, `5`)'
# ["aber", "alle", "allem", "allen", "aller"]

# French
echo '["le", "chat", "noir", "est", "rapide"]' | jpx 'remove_stopwords(@, `"fr"`)'
# ["chat", "noir", "rapide"]
```

---

## Stemming

Stemming reduces words to their root form, helping match variations like "running", "runs", "ran" to "run".

### Single Word Stemming

```bash
echo '{}' | jpx 'stem(`"running"`)'
# "run"

echo '{}' | jpx 'stem(`"connections"`)'
# "connect"

echo '{}' | jpx 'stem(`"happiness"`)'
# "happi"
```

### Stem Token Arrays

```bash
echo '["running", "jumps", "walked", "swimming"]' | jpx 'stems(@)'
# ["run", "jump", "walk", "swim"]
```

### Multilingual Stemming

Supports 17 languages via Snowball stemmers:

```bash
# Spanish
echo '{}' | jpx 'stem(`"corriendo"`, `"es"`)'
# "corr"

# German
echo '{}' | jpx 'stem(`"laufend"`, `"de"`)'
# "lauf"

# French
echo '["courant", "marchant", "parlant"]' | jpx 'stems(@, `"fr"`)'
# ["cour", "march", "parl"]
```

**Supported languages:** ar (Arabic), da (Danish), de (German), el (Greek), en (English), es (Spanish), fi (Finnish), fr (French), hu (Hungarian), it (Italian), nl (Dutch), no (Norwegian), pt (Portuguese), ro (Romanian), ru (Russian), sv (Swedish), tr (Turkish).

---

## Text Normalization

### Unicode Normalization

Normalize text to consistent Unicode forms:

```bash
# NFC (composed) - default
echo '"café"' | jpx 'normalize_unicode(@)'
# "café"

# NFKC - compatibility composed (expands ligatures)
echo '"ﬁle"' | jpx 'normalize_unicode(@, `"nfkc"`)'
# "file"
```

### Remove Accents

Strip diacritical marks for accent-insensitive matching:

```bash
echo '"café résumé naïve"' | jpx 'remove_accents(@)'
# "cafe resume naive"

echo '"señor über"' | jpx 'remove_accents(@)'
# "senor uber"
```

### Collapse Whitespace

Normalize irregular whitespace:

```bash
echo '"hello    world\t\ntest"' | jpx 'collapse_whitespace(@)'
# "hello world test"
```

---

## Complete NLP Pipelines

### Basic Text Preprocessing

```bash
TEXT='"The quick brown foxes are running and jumping quickly over the lazy dogs."'

echo "$TEXT" | jpx '
  tokens(@) 
  | remove_stopwords(@) 
  | stems(@)
'
# ["quick", "brown", "fox", "run", "jump", "quick", "lazi", "dog"]
```

### Word Frequency Analysis

```bash
TEXT='"The cat sat on the mat. The cat was happy. The mat was soft."'

# Get word frequencies after preprocessing
echo "$TEXT" | jpx '
  tokens(@) 
  | remove_stopwords(@) 
  | frequencies(@)
'
# {"cat": 2, "sat": 1, "mat": 2, "happi": 1, "soft": 1}

# With stemming first
echo "$TEXT" | jpx '
  tokens(@) 
  | remove_stopwords(@) 
  | stems(@)
  | frequencies(@)
'
```

### N-gram Analysis After Preprocessing

```bash
TEXT='"Machine learning models require training data"'

# Word bigrams
echo "$TEXT" | jpx '
  tokens(@) 
  | remove_stopwords(@) 
  | join(` `, @) 
  | bigrams(@)
'
# [["machine", "learning"], ["learning", "models"], ["models", "require"], ["require", "training"], ["training", "data"]]
```

### Multilingual Pipeline

```bash
# French text processing
FRENCH='"Les chats noirs sont très rapides et intelligents"'

echo "$FRENCH" | jpx '
  tokens(@) 
  | remove_stopwords(@, `"fr"`) 
  | stems(@, `"fr"`)
'
# ["chat", "noir", "rapid", "intelligent"]
```

---

## Working with JSON Data

### Process Text Fields in Objects

```bash
cat <<'EOF' | jpx '
  [*].{
    id: id,
    original: text,
    tokens: tokens(text) | remove_stopwords(@) | stems(@),
    word_count: word_count(text)
  }
'
[
  {"id": 1, "text": "The quick brown fox jumps over the lazy dog"},
  {"id": 2, "text": "Machine learning algorithms are transforming industries"},
  {"id": 3, "text": "Natural language processing enables text analysis"}
]
EOF
```

### Extract and Analyze Descriptions

```bash
# Analyze product descriptions
cat <<'EOF' | jpx '
  products[*].{
    name: name,
    keywords: tokens(description) | remove_stopwords(@) | stems(@) | unique(@)
  }
'
{
  "products": [
    {"name": "Widget A", "description": "A fantastic widget for organizing your daily tasks"},
    {"name": "Widget B", "description": "An amazing widget for managing your weekly schedule"}
  ]
}
EOF
```

### Text Similarity Preparation

Prepare text for similarity comparison:

```bash
cat <<'EOF' | jpx '
  [*].{
    id: id,
    normalized: text 
      | lower(@) 
      | remove_accents(collapse_whitespace(@))
      | tokens(@) 
      | remove_stopwords(@) 
      | stems(@) 
      | sort(@) 
      | join(` `, @)
  }
'
[
  {"id": 1, "text": "The café serves excellent coffee"},
  {"id": 2, "text": "Cafés serving good coffees"}
]
EOF
# Both normalize to similar representations for comparison
```

---

## Combining with Other jpx Functions

### With Language Detection

```bash
cat <<'EOF' | jpx '
  [*].{
    text: text,
    language: detect_language_iso(text),
    tokens: tokens(text)
  }
'
[
  {"text": "Hello world, this is English"},
  {"text": "Bonjour le monde, ceci est français"},
  {"text": "Hola mundo, esto es español"}
]
EOF
```

### With Fuzzy Matching

Find similar terms after stemming:

```bash
echo '{}' | jpx '{
  terms: [`"running"`, `"runner"`, `"ran"`, `"run"`] | stems(@) | unique(@),
  similar: jaro_winkler(`"run"`, `"ran"`)
}'
# {"terms": ["run"], "similar": 0.933...}
```

### With Phonetic Matching

Combine text normalization with phonetic codes:

```bash
echo '["café", "cafe", "caffè"]' | jpx '[*].{
  original: @,
  normalized: remove_accents(@),
  soundex: soundex(remove_accents(@))
}'
```

---

## Performance Tips

1. **Chain operations efficiently**: The pipe operator streams data without intermediate allocations

2. **Filter early**: Remove stopwords before expensive operations like stemming

3. **Use `tokens` for simple cases**: It's optimized for the common case (lowercase, no punctuation)

4. **Batch process**: Process arrays of text rather than calling functions repeatedly

```bash
# Good: Process array at once
echo '["text1", "text2", "text3"]' | jpx '[*] | stems(tokens(@[0]))'

# Better: Use map expressions
echo '{"texts": ["running fast", "jumping high"]}' | jpx 'texts[*] | [*].tokens(@)'
```

---

## Using Query Libraries

Save your NLP pipelines in a `.jpx` query library for reuse. See [examples/nlp.jpx](https://github.com/joshrotenberg/jmespath-extensions/blob/main/examples/nlp.jpx) for ready-to-use text processing queries:

```bash
# List available NLP queries
jpx -Q examples/nlp.jpx --list-queries

# Clean HTML from text
echo '"<p>Hello <b>World</b>!</p>"' | jpx -Q examples/nlp.jpx:clean-html

# Extract keywords
echo '"The quick brown foxes are running quickly"' | jpx -Q examples/nlp.jpx:extract-keywords

# Get reading statistics
cat article.txt | jpx -Q examples/nlp.jpx:reading-stats
```

Create your own domain-specific library:

```
-- :name preprocess
-- :desc Standard preprocessing pipeline
tokens(@) | remove_stopwords(@) | stems(@)

-- :name keyword-extract
-- :desc Top 10 keywords from text
tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@) | items(@) | sort_by(@, &[1]) | reverse(@) | [:10][*][0]
```

See [Query Files](../../guide/query-files.md) for more on creating and using query libraries.

---

## Related Functions

- [Language Detection](../functions/language.md) - `detect_language`, `detect_script`
- [Fuzzy Matching](../functions/fuzzy.md) - `levenshtein`, `jaro_winkler`
- [Phonetic](../functions/phonetic.md) - `soundex`, `metaphone`, `sounds_like`
- [String Functions](../functions/string.md) - `lower`, `upper`, `trim`, `split`
