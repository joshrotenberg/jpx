# Regex Functions (5)

## `regex_count`

**Signature:** `string, string -> number`

Count the number of regex matches. Use backtick-JSON syntax: regex_count(text, `"\\d+"`)

```
regex_count(`"a1b2c3"`, `"\\\\d+"`) -> 3
```
_Count numbers_

```
regex_count(`"hello world"`, `"[aeiou]"`) -> 3
```
_Count vowels_

---

## `regex_extract`

**Signature:** `string, string -> array`

Extract regex matches. Use backtick-JSON syntax: regex_extract(text, `"\\w+"`)

```
regex_extract(`"a1b2"`, `"\\\\d+"`) -> [\"1\", \"2\"]
```
_Extract numbers_

```
regex_extract(`"hello world"`, `"\\\\w+"`) -> [\"hello\", \"world\"]
```
_Extract words_

---

## `regex_match`

**Signature:** `string, string -> boolean`

Test if string matches regex. Use backtick-JSON syntax: regex_match(text, `"^\\d+$"`)

```
regex_match(`"hello"`, `"^h.*o$"`) -> true
```
_Full match_

```
regex_match(`"test123"`, `"\\\\d+"`) -> true
```
_Contains digits_

---

## `regex_replace`

**Signature:** `string, string, string -> string`

Replace regex matches. Use backtick-JSON syntax: regex_replace(text, `"\\d+"`, `"X"`)

```
regex_replace(`"a1b2"`, `"\\\\d+"`, `"X"`) -> \"aXbX\"
```
_Replace digits_

```
regex_replace(`"hello world"`, `"\\\\s+"`, `"-"`) -> \"hello-world\"
```
_Replace spaces_

---

## `regex_split`

**Signature:** `string, string -> array`

Split a string by a regex pattern. Use backtick-JSON syntax: regex_split(text, `"\\s+"`)

```
regex_split(`"a,b,c"`, `","`) -> [\"a\", \"b\", \"c\"]
```
_Split by comma_

```
regex_split(`"hello   world"`, `"\\\\s+"`) -> [\"hello\", \"world\"]
```
_Split by whitespace_

---

