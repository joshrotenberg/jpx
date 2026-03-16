# Multimatch Functions (10)

## `extract_all`

**Signature:** `string, array[string] -> array[object]`

Extract all pattern matches with positions (Aho-Corasick)

```
extract_all('error warning', ['error', 'warning']) -> [{pattern: 'error', match: 'error', start: 0, end: 5}, ...]
```
_Multiple patterns_

```
extract_all('abab', ['a', 'b']) -> [{pattern: 'a', match: 'a', start: 0, end: 1}, ...]
```
_Overlapping matches_

---

## `extract_between`

**Signature:** `string, string, string -> string|null`

Extract text between two delimiters

```
extract_between('<title>Page</title>', '<title>', '</title>') -> \"Page\"
```
_HTML tag content_

```
extract_between('Hello [world]!', '[', ']') -> \"world\"
```
_Bracketed content_

---

## `match_all`

**Signature:** `string, array[string] -> boolean`

Check if string contains all of the patterns (Aho-Corasick)

```
match_all('hello world', ['hello', 'world']) -> true
```
_All patterns found_

```
match_all('hello world', ['hello', 'foo']) -> false
```
_Missing pattern_

---

## `match_any`

**Signature:** `string, array[string] -> boolean`

Check if string contains any of the patterns (Aho-Corasick)

```
match_any('hello world', ['world', 'foo']) -> true
```
_One pattern found_

```
match_any('hello world', ['foo', 'bar']) -> false
```
_No patterns found_

---

## `match_count`

**Signature:** `string, array[string] -> number`

Count total pattern matches in string (Aho-Corasick)

```
match_count('abcabc', ['a', 'b']) -> 4
```
_Count all matches_

```
match_count('hello', ['l']) -> 2
```
_Repeated pattern_

---

## `match_positions`

**Signature:** `string, array[string] -> array[object]`

Get start/end positions of all pattern matches (Aho-Corasick)

```
match_positions('The quick fox', ['quick', 'fox']) -> [{pattern: 'quick', start: 4, end: 9}, ...]
```
_Find positions_

```
match_positions('abab', ['ab']) -> [{pattern: 'ab', start: 0, end: 2}, {pattern: 'ab', start: 2, end: 4}]
```
_Multiple occurrences_

---

## `match_which`

**Signature:** `string, array[string] -> array[string]`

Return array of patterns that match the string (Aho-Corasick)

```
match_which('hello world', ['hello', 'foo', 'world']) -> [\"hello\", \"world\"]
```
_Find matching patterns_

```
match_which('abc', ['a', 'b', 'x']) -> [\"a\", \"b\"]
```
_Partial matches_

---

## `mm_tokenize`

**Signature:** `string, object? -> array[string]`

Smart word tokenization with optional lowercase and min_length

```
mm_tokenize('Hello, World!') -> [\"Hello\", \"World\"]
```
_Basic tokenization_

```
mm_tokenize('Hello, World!', {lowercase: `true`}) -> [\"hello\", \"world\"]
```
_Lowercase option_

---

## `replace_many`

**Signature:** `string, object -> string`

Replace multiple patterns simultaneously (Aho-Corasick)

```
replace_many('hello world', {hello: 'hi', world: 'earth'}) -> \"hi earth\"
```
_Multiple replacements_

```
replace_many('aaa', {a: 'b'}) -> \"bbb\"
```
_Repeated pattern_

---

## `split_keep`

**Signature:** `string, string -> array[string]`

Split string keeping delimiters in result

```
split_keep('a-b-c', '-') -> [\"a\", \"-\", \"b\", \"-\", \"c\"]
```
_Keep dashes_

```
split_keep('hello world', ' ') -> [\"hello\", \" \", \"world\"]
```
_Keep spaces_

---

