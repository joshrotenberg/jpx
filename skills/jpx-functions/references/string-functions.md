# String Functions (36)

## `abbreviate`

**Signature:** `string, number, string? -> string`

Truncate string with ellipsis suffix

```
abbreviate('hello world', `8`) -> \"hello...\"
```
_Default ellipsis_

```
abbreviate('hello', `10`) -> \"hello\"
```
_No truncation needed_

---

## `camel_case`

**Signature:** `string -> string`

Convert to camelCase

```
camel_case('hello_world') -> \"helloWorld\"
```
_From snake_case_

```
camel_case('hello-world') -> \"helloWorld\"
```
_From kebab-case_

---

## `capitalize`

**Signature:** `string -> string`

Capitalize the first character

```
capitalize('hello') -> \"Hello\"
```
_Basic capitalize_

```
capitalize('HELLO') -> \"HELLO\"
```
_Already uppercase_

---

## `center`

**Signature:** `string, number, string? -> string`

Center-pad string to given width

```
center('hi', `6`) -> \"  hi  \"
```
_Center with spaces_

```
center('hi', `6`, '-') -> \"--hi--\"
```
_Center with dashes_

---

## `concat`

**Signature:** `string... -> string`

Concatenate strings

```
concat('hello', ' ', 'world') -> \"hello world\"
```
_Multiple strings_

```
concat('a', 'b') -> \"ab\"
```
_Two strings_

---

## `explode`

**Signature:** `string -> array`

Convert a string to an array of Unicode codepoints

```
explode('abc') -> [97, 98, 99]
```
_ASCII characters_

```
explode('A☺') -> [65, 9786]
```
_Unicode characters_

---

## `find_first`

**Signature:** `string, string -> number | null`

Find first occurrence of substring

```
find_first('hello', 'l') -> 2
```
_Find character_

```
find_first('hello world', 'world') -> 6
```
_Find substring_

---

## `find_last`

**Signature:** `string, string -> number | null`

Find last occurrence of substring

```
find_last('hello', 'l') -> 3
```
_Find last character_

```
find_last('foo bar foo', 'foo') -> 8
```
_Find last substring_

---

## `implode`

**Signature:** `array -> string`

Convert an array of Unicode codepoints to a string

```
implode([97, 98, 99]) -> \"abc\"
```
_ASCII codepoints_

```
implode([65, 9786]) -> \"A☺\"
```
_Unicode codepoints_

---

## `indices`

**Signature:** `string, string -> array`

Find all indices of substring occurrences

```
indices('hello', 'l') -> [2, 3]
```
_Multiple occurrences_

```
indices('ababa', 'aba') -> [0, 2]
```
_Overlapping matches_

---

## `inside`

**Signature:** `string, string -> boolean`

Check if search string is contained in string

```
inside('world', 'hello world') -> true
```
_Found_

```
inside('foo', 'hello world') -> false
```
_Not found_

---

## `is_blank`

**Signature:** `string -> boolean`

Check if string is empty or whitespace-only

```
is_blank('   ') -> true
```
_Whitespace only_

```
is_blank('') -> true
```
_Empty string_

---

## `kebab_case`

**Signature:** `string -> string`

Convert to kebab-case

```
kebab_case('helloWorld') -> \"hello-world\"
```
_From camelCase_

```
kebab_case('hello_world') -> \"hello-world\"
```
_From snake_case_

---

## `lower`

**Signature:** `string -> string`

Convert string to lowercase

```
lower('HELLO') -> \"hello\"
```
_All uppercase_

```
lower('Hello World') -> \"hello world\"
```
_Mixed case_

---

## `ltrimstr`

**Signature:** `string, string -> string`

Remove prefix from string if present

```
ltrimstr('foobar', 'foo') -> \"bar\"
```
_Remove prefix_

```
ltrimstr('foobar', 'bar') -> \"foobar\"
```
_Prefix not found_

---

## `mask`

**Signature:** `string, number?, string? -> string`

Mask string, keeping last N characters visible

```
mask('4111111111111111', `4`) -> \"************1111\"
```
_Credit card_

```
mask('secret', `0`) -> \"******\"
```
_Mask all_

---

## `normalize_whitespace`

**Signature:** `string -> string`

Collapse multiple whitespace to single space

```
normalize_whitespace('a  b  c') -> \"a b c\"
```
_Multiple spaces_

```
normalize_whitespace('a\\n\\nb') -> \"a b\"
```
_Newlines to space_

---

## `pad_left`

**Signature:** `string, number, string -> string`

Pad string on the left to reach target length

```
pad_left('5', `3`, '0') -> \"005\"
```
_Zero-pad number_

```
pad_left('hi', `5`, ' ') -> \"   hi\"
```
_Right-align text_

---

## `pad_right`

**Signature:** `string, number, string -> string`

Pad string on the right to reach target length

```
pad_right('5', `3`, '0') -> \"500\"
```
_Pad with zeros_

```
pad_right('hi', `5`, ' ') -> \"hi   \"
```
_Left-align text_

---

## `redact`

**Signature:** `string, string, string? -> string`

Redact regex pattern matches with replacement

```
redact('email: test@example.com', '\\S+@\\S+', '[EMAIL]') -> \"email: [EMAIL]\"
```
_Redact email_

```
redact('call 555-1234', '\\d{3}-\\d{4}', '[PHONE]') -> \"call [PHONE]\"
```
_Redact phone_

---

## `repeat`

**Signature:** `string, number -> string`

Repeat a string n times

```
repeat('ab', `3`) -> \"ababab\"
```
_Repeat 3 times_

```
repeat('-', `5`) -> \"-----\"
```
_Create separator_

---

## `replace`

**Signature:** `string, string, string -> string`

Replace occurrences of a substring. Use backtick-JSON syntax: replace(text, `"\n"`, `" "`)

```
replace(`"hello"`, `"l"`, `"L"`) -> \"heLLo\"
```
_Replace all occurrences_

```
replace(`"line1\\nline2"`, `"\\n"`, `" "`) -> \"line1 line2\"
```
_Replace newlines_

---

## `reverse_string`

**Signature:** `string -> string`

Reverse a string

```
reverse_string('hello') -> \"olleh\"
```
_Reverse word_

```
reverse_string('ab') -> \"ba\"
```
_Two chars_

---

## `rtrimstr`

**Signature:** `string, string -> string`

Remove suffix from string if present

```
rtrimstr('foobar', 'bar') -> \"foo\"
```
_Remove suffix_

```
rtrimstr('hello.txt', '.txt') -> \"hello\"
```
_Remove file extension_

---

## `shell_escape`

**Signature:** `string -> string`

Escape a string for safe use in shell commands (POSIX sh compatible, jq parity)

```
shell_escape('hello') -> \"hello\"
```
_Simple string unchanged_

```
shell_escape('hello world') -> \"'hello world'\"
```
_Spaces get quoted_

---

## `slice`

**Signature:** `string, number, number -> string`

Extract substring by start and end index

```
slice('hello', `1`, `4`) -> \"ell\"
```
_Middle slice_

```
slice('hello', `0`, `2`) -> \"he\"
```
_From start_

---

## `snake_case`

**Signature:** `string -> string`

Convert to snake_case

```
snake_case('helloWorld') -> \"hello_world\"
```
_From camelCase_

```
snake_case('HelloWorld') -> \"hello_world\"
```
_From PascalCase_

---

## `split`

**Signature:** `string, string -> array`

Split string by delimiter. Use backtick-JSON syntax for literals: split(text, `"\n"`) to split on newlines

```
split(`"a,b,c"`, `","`) -> [\"a\", \"b\", \"c\"]
```
_Basic split_

```
split(`"hello world"`, `" "`) -> [\"hello\", \"world\"]
```
_Split by space_

---

## `sprintf`

**Signature:** `string, any... -> string`

Printf-style string formatting

```
sprintf('Pi is %.2f', `3.14159`) -> \"Pi is 3.14\"
```
_Float formatting_

```
sprintf('Hello %s', 'world') -> \"Hello world\"
```
_String interpolation_

---

## `substr`

**Signature:** `string, number, number -> string`

Extract substring by start index and length

```
substr('hello', `1`, `3`) -> \"ell\"
```
_From index 1, length 3_

```
substr('hello', `0`, `2`) -> \"he\"
```
_From start_

---

## `title`

**Signature:** `string -> string`

Convert to title case

```
title('hello world') -> \"Hello World\"
```
_Basic title case_

```
title('HELLO WORLD') -> \"Hello World\"
```
_From uppercase_

---

## `trim`

**Signature:** `string -> string`

Remove leading and trailing whitespace

```
trim('  hello  ') -> \"hello\"
```
_Remove both sides_

```
trim('hello') -> \"hello\"
```
_No whitespace_

---

## `trim_left`

**Signature:** `string -> string`

Remove leading whitespace

```
trim_left('  hello') -> \"hello\"
```
_Remove leading spaces_

```
trim_left('hello  ') -> \"hello  \"
```
_Trailing preserved_

---

## `trim_right`

**Signature:** `string -> string`

Remove trailing whitespace

```
trim_right('hello  ') -> \"hello\"
```
_Remove trailing spaces_

```
trim_right('  hello') -> \"  hello\"
```
_Leading preserved_

---

## `upper`

**Signature:** `string -> string`

Convert string to uppercase

```
upper('hello') -> \"HELLO\"
```
_Basic uppercase_

```
upper('Hello World') -> \"HELLO WORLD\"
```
_Mixed case_

---

## `wrap`

**Signature:** `string, number -> string`

Wrap text to specified width

```
wrap('hello world', `5`) -> \"hello\\nworld\"
```
_Wrap at word boundary_

```
wrap('a b c d e', `3`) -> \"a b\\nc d\\ne\"
```
_Multiple wraps_

---

