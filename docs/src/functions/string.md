# String Functions

Functions for string manipulation: case conversion, splitting, joining, padding, and text processing.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`abbreviate`](#abbreviate) | `string, number, string? -> string` | Truncate string with ellipsis suffix |
| [`camel_case`](#camel-case) | `string -> string` | Convert to camelCase |
| [`capitalize`](#capitalize) | `string -> string` | Capitalize the first character |
| [`center`](#center) | `string, number, string? -> string` | Center-pad string to given width |
| [`concat`](#concat) | `string... -> string` | Concatenate strings |
| [`explode`](#explode) | `string -> array` | Convert a string to an array of Unicode codepoints |
| [`find_first`](#find-first) | `string, string -> number \| null` | Find first occurrence of substring |
| [`find_last`](#find-last) | `string, string -> number \| null` | Find last occurrence of substring |
| [`implode`](#implode) | `array -> string` | Convert an array of Unicode codepoints to a string |
| [`indices`](#indices) | `string, string -> array` | Find all indices of substring occurrences |
| [`inside`](#inside) | `string, string -> boolean` | Check if search string is contained in string |
| [`is_blank`](#is-blank) | `string -> boolean` | Check if string is empty or whitespace-only |
| [`kebab_case`](#kebab-case) | `string -> string` | Convert to kebab-case |
| [`lower`](#lower) | `string -> string` | Convert string to lowercase |
| [`pascal_case`](#pascal-case) | `string -> string` | Convert to PascalCase |
| [`ltrimstr`](#ltrimstr) | `string, string -> string` | Remove prefix from string if present |
| [`mask`](#mask) | `string, number?, string? -> string` | Mask string, keeping last N characters visible |
| [`normalize_whitespace`](#normalize-whitespace) | `string -> string` | Collapse multiple whitespace to single space |
| [`pad_left`](#pad-left) | `string, number, string -> string` | Pad string on the left to reach target length |
| [`pad_right`](#pad-right) | `string, number, string -> string` | Pad string on the right to reach target length |
| [`redact`](#redact) | `string, string, string? -> string` | Redact regex pattern matches with replacement |
| [`repeat`](#repeat) | `string, number -> string` | Repeat a string n times |
| [`shouty_kebab_case`](#shouty-kebab-case) | `string -> string` | Convert to SHOUTY-KEBAB-CASE |
| [`shouty_snake_case`](#shouty-snake-case) | `string -> string` | Convert to SHOUTY_SNAKE_CASE |
| [`replace`](#replace) | `string, string, string -> string` | Replace occurrences of a substring |
| [`reverse_string`](#reverse-string) | `string -> string` | Reverse a string |
| [`rtrimstr`](#rtrimstr) | `string, string -> string` | Remove suffix from string if present |
| [`shell_escape`](#shell-escape) | `string -> string` | Escape a string for safe use in shell commands (POSIX sh compatible, jq parity) |
| [`slice`](#slice) | `string, number, number -> string` | Extract substring by start and end index |
| [`snake_case`](#snake-case) | `string -> string` | Convert to snake_case |
| [`split`](#split) | `string, string -> array` | Split string by delimiter |
| [`sprintf`](#sprintf) | `string, any... -> string` | Printf-style string formatting |
| [`substr`](#substr) | `string, number, number -> string` | Extract substring by start index and length |
| [`title`](#title) | `string -> string` | Convert to title case |
| [`train_case`](#train-case) | `string -> string` | Convert to Train-Case |
| [`trim`](#trim) | `string -> string` | Remove leading and trailing whitespace |
| [`trim_left`](#trim-left) | `string -> string` | Remove leading whitespace |
| [`trim_right`](#trim-right) | `string -> string` | Remove trailing whitespace |
| [`upper`](#upper) | `string -> string` | Convert string to uppercase |
| [`wrap`](#wrap) | `string, number -> string` | Wrap text to specified width |

## Functions

### abbreviate

Truncate string with ellipsis suffix

**Signature:** `string, number, string? -> string`

**Examples:**

```text
# Default ellipsis
abbreviate('hello world', `8`) -> \"hello...\"
# No truncation needed
abbreviate('hello', `10`) -> \"hello\"
# Custom suffix
abbreviate('hello world', `8`, '>>') -> \"hello >>\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'abbreviate(`"hello world"`, `8`)'
```

### camel_case

Convert to camelCase

**Signature:** `string -> string`

**Examples:**

```text
# From snake_case
camel_case('hello_world') -> \"helloWorld\"
# From kebab-case
camel_case('hello-world') -> \"helloWorld\"
# From title case
camel_case('Hello World') -> \"helloWorld\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'camel_case(`"hello_world"`)'
```

### capitalize

Capitalize the first character

**Signature:** `string -> string`

**Examples:**

```text
# Basic capitalize
capitalize('hello') -> \"Hello\"
# Already uppercase
capitalize('HELLO') -> \"HELLO\"
# Empty string
capitalize('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'capitalize(`"hello"`)'
```

### center

Center-pad string to given width

**Signature:** `string, number, string? -> string`

**Examples:**

```text
# Center with spaces
center('hi', `6`) -> \"  hi  \"
# Center with dashes
center('hi', `6`, '-') -> \"--hi--\"
# Already wider
center('hello', `3`) -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'center(`"hi"`, `6`)'
```

### concat

Concatenate strings

**Signature:** `string... -> string`

**Examples:**

```text
# Multiple strings
concat('hello', ' ', 'world') -> \"hello world\"
# Two strings
concat('a', 'b') -> \"ab\"
# Single string
concat('only') -> \"only\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'concat(`"hello"`, `" "`, `"world"`)'
```

### explode

Convert a string to an array of Unicode codepoints

**Signature:** `string -> array`

**Examples:**

```text
# ASCII characters
explode('abc') -> [97, 98, 99]
# Unicode characters
explode('A☺') -> [65, 9786]
# Empty string
explode('') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'explode(`"abc"`)'
```

### find_first

Find first occurrence of substring

**Signature:** `string, string -> number | null`

**JEP:** JEP-014

**Examples:**

```text
# Find character
find_first('hello', 'l') -> 2
# Find substring
find_first('hello world', 'world') -> 6
# Not found
find_first('hello', 'x') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'find_first(`"hello"`, `"l"`)'
```

### find_last

Find last occurrence of substring

**Signature:** `string, string -> number | null`

**JEP:** JEP-014

**Examples:**

```text
# Find last character
find_last('hello', 'l') -> 3
# Find last substring
find_last('foo bar foo', 'foo') -> 8
# Not found
find_last('hello', 'x') -> null
```

**CLI Usage:**

```bash
echo '{}' | jpx 'find_last(`"hello"`, `"l"`)'
```

### implode

Convert an array of Unicode codepoints to a string

**Signature:** `array -> string`

**Examples:**

```text
# ASCII codepoints
implode([97, 98, 99]) -> \"abc\"
# Unicode codepoints
implode([65, 9786]) -> \"A☺\"
# Empty array
implode([]) -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'implode([97, 98, 99])'
```

### indices

Find all indices of substring occurrences

**Signature:** `string, string -> array`

**Examples:**

```text
# Multiple occurrences
indices('hello', 'l') -> [2, 3]
# Overlapping matches
indices('ababa', 'aba') -> [0, 2]
# No matches
indices('hello', 'x') -> []
```

**CLI Usage:**

```bash
echo '{}' | jpx 'indices(`"hello"`, `"l"`)'
```

### inside

Check if search string is contained in string

**Signature:** `string, string -> boolean`

**Examples:**

```text
# Found
inside('world', 'hello world') -> true
# Not found
inside('foo', 'hello world') -> false
# Empty string always matches
inside('', 'hello') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'inside(`"world"`, `"hello world"`)'
```

### is_blank

Check if string is empty or whitespace-only

**Signature:** `string -> boolean`

**Examples:**

```text
# Whitespace only
is_blank('   ') -> true
# Empty string
is_blank('') -> true
# Has content
is_blank('hello') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_blank(`"   "`)'
```

### kebab_case

Convert to kebab-case

**Signature:** `string -> string`

**Examples:**

```text
# From camelCase
kebab_case('helloWorld') -> \"hello-world\"
# From snake_case
kebab_case('hello_world') -> \"hello-world\"
# From title case
kebab_case('Hello World') -> \"hello-world\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'kebab_case(`"helloWorld"`)'
```

### lower

Convert string to lowercase

**Signature:** `string -> string`

**JEP:** JEP-014

**Examples:**

```text
# All uppercase
lower('HELLO') -> \"hello\"
# Mixed case
lower('Hello World') -> \"hello world\"
# Already lowercase
lower('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'lower(`"HELLO"`)'
```

### pascal_case

Convert to PascalCase (also known as UpperCamelCase)

**Signature:** `string -> string`

**Examples:**

```text
# From snake_case
pascal_case('hello_world') -> "HelloWorld"
# From camelCase
pascal_case('helloWorld') -> "HelloWorld"
# From kebab-case
pascal_case('hello-world') -> "HelloWorld"
# Handles acronyms
pascal_case('xml_parser') -> "XmlParser"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pascal_case(`"hello_world"`)'
```

### ltrimstr

Remove prefix from string if present

**Signature:** `string, string -> string`

**Examples:**

```text
# Remove prefix
ltrimstr('foobar', 'foo') -> \"bar\"
# Prefix not found
ltrimstr('foobar', 'bar') -> \"foobar\"
# Empty prefix
ltrimstr('hello', '') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ltrimstr(`"foobar"`, `"foo"`)'
```

### mask

Mask string, keeping last N characters visible

**Signature:** `string, number?, string? -> string`

**Examples:**

```text
# Credit card
mask('4111111111111111', `4`) -> \"************1111\"
# Mask all
mask('secret', `0`) -> \"******\"
# Custom mask char
mask('password', `4`, '#') -> \"####word\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'mask(`"4111111111111111"`, `4`)'
```

### normalize_whitespace

Collapse multiple whitespace to single space

**Signature:** `string -> string`

**Examples:**

```text
# Multiple spaces
normalize_whitespace('a  b  c') -> \"a b c\"
# Newlines to space
normalize_whitespace('a\\n\\nb') -> \"a b\"
# Already normalized
normalize_whitespace('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'normalize_whitespace(`"a  b  c"`)'
```

### pad_left

Pad string on the left to reach target length

**Signature:** `string, number, string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Zero-pad number
pad_left('5', `3`, '0') -> \"005\"
# Right-align text
pad_left('hi', `5`, ' ') -> \"   hi\"
# Already long enough
pad_left('hello', `3`, '0') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pad_left(`"5"`, `3`, `"0"`)'
```

### pad_right

Pad string on the right to reach target length

**Signature:** `string, number, string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Pad with zeros
pad_right('5', `3`, '0') -> \"500\"
# Left-align text
pad_right('hi', `5`, ' ') -> \"hi   \"
# Already long enough
pad_right('hello', `3`, '0') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pad_right(`"5"`, `3`, `"0"`)'
```

### redact

Redact regex pattern matches with replacement

**Signature:** `string, string, string? -> string`

**Examples:**

```text
# Redact email
redact('email: test@example.com', '\\S+@\\S+', '[EMAIL]') -> \"email: [EMAIL]\"
# Redact phone
redact('call 555-1234', '\\d{3}-\\d{4}', '[PHONE]') -> \"call [PHONE]\"
# No matches
redact('no match', 'xyz', '[X]') -> \"no match\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'redact(`"email: test@example.com"`, `"\\S+@\\S+"`, `"[EMAIL]"`)'
```

### repeat

Repeat a string n times

**Signature:** `string, number -> string`

**Examples:**

```text
# Repeat 3 times
repeat('ab', `3`) -> \"ababab\"
# Create separator
repeat('-', `5`) -> \"-----\"
# Zero times
repeat('x', `0`) -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'repeat(`"ab"`, `3`)'
```

### shouty_kebab_case

Convert to SHOUTY-KEBAB-CASE (also known as COBOL-CASE or SCREAMING-KEBAB-CASE)

**Signature:** `string -> string`

**Examples:**

```text
# From camelCase
shouty_kebab_case('helloWorld') -> "HELLO-WORLD"
# From snake_case
shouty_kebab_case('hello_world') -> "HELLO-WORLD"
# From PascalCase
shouty_kebab_case('HelloWorld') -> "HELLO-WORLD"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shouty_kebab_case(`"helloWorld"`)'
```

### shouty_snake_case

Convert to SHOUTY_SNAKE_CASE (also known as SCREAMING_SNAKE_CASE or CONSTANT_CASE)

**Signature:** `string -> string`

**Examples:**

```text
# From camelCase
shouty_snake_case('helloWorld') -> "HELLO_WORLD"
# From kebab-case
shouty_snake_case('hello-world') -> "HELLO_WORLD"
# From PascalCase
shouty_snake_case('HelloWorld') -> "HELLO_WORLD"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shouty_snake_case(`"helloWorld"`)'
```

### replace

Replace occurrences of a substring

**Signature:** `string, string, string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Replace all occurrences
replace('hello', 'l', 'L') -> \"heLLo\"
# No match
replace('hello', 'x', 'y') -> \"hello\"
# Replace all
replace('aaa', 'a', 'b') -> \"bbb\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'replace(`"hello"`, `"l"`, `"L"`)'
```

### reverse_string

Reverse a string

**Signature:** `string -> string`

**Examples:**

```text
# Reverse word
reverse_string('hello') -> \"olleh\"
# Two chars
reverse_string('ab') -> \"ba\"
# Empty string
reverse_string('') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'reverse_string(`"hello"`)'
```

### rtrimstr

Remove suffix from string if present

**Signature:** `string, string -> string`

**Examples:**

```text
# Remove suffix
rtrimstr('foobar', 'bar') -> \"foo\"
# Remove file extension
rtrimstr('hello.txt', '.txt') -> \"hello\"
# Suffix not present
rtrimstr('hello', 'xyz') -> \"hello\"
# Only removes once
rtrimstr('barbar', 'bar') -> \"bar\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'rtrimstr(`"foobar"`, `"bar"`)'
```

### shell_escape

Escape a string for safe use in shell commands (POSIX sh compatible, jq parity)

**Signature:** `string -> string`

**Examples:**

```text
# Simple string unchanged
shell_escape('hello') -> \"hello\"
# Spaces get quoted
shell_escape('hello world') -> \"'hello world'\"
# Variables are escaped
shell_escape('$PATH') -> \"'$PATH'\"
# Single quotes escaped
shell_escape('it'\\''s') -> \"'it'\\\\''s'\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'shell_escape(`"hello"`)'
```

### slice

Extract substring by start and end index

**Signature:** `string, number, number -> string`

**Examples:**

```text
# Middle slice
slice('hello', `1`, `4`) -> \"ell\"
# From start
slice('hello', `0`, `2`) -> \"he\"
# To end
slice('hello', `3`, `5`) -> \"lo\"
# Two characters
slice('abcdef', `2`, `4`) -> \"cd\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'slice(`"hello"`, `1`, `4`)'
```

### snake_case

Convert to snake_case

**Signature:** `string -> string`

**Examples:**

```text
# From camelCase
snake_case('helloWorld') -> \"hello_world\"
# From PascalCase
snake_case('HelloWorld') -> \"hello_world\"
# From kebab-case
snake_case('hello-world') -> \"hello_world\"
# From UPPER_CASE
snake_case('HELLO_WORLD') -> \"hello_world\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'snake_case(`"helloWorld"`)'
```

### split

Split string by delimiter

**Signature:** `string, string -> array`

**JEP:** JEP-014

**Examples:**

```text
# Basic split
split('a,b,c', ',') -> [\"a\", \"b\", \"c\"]
# Split by space
split('hello world', ' ') -> [\"hello\", \"world\"]
# No delimiter found
split('no-delim', ',') -> [\"no-delim\"]
# Empty string
split('', ',') -> [\"\"]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'split(`"a,b,c"`, `","`)'
```

### sprintf

Printf-style string formatting

**Signature:** `string, any... -> string`

**Examples:**

```text
# Float formatting
sprintf('Pi is %.2f', `3.14159`) -> \"Pi is 3.14\"
# String interpolation
sprintf('Hello %s', 'world') -> \"Hello world\"
# Integer formatting
sprintf('%d items', `42`) -> \"42 items\"
# Multiple args
sprintf('%s: %d', 'count', `5`) -> \"count: 5\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sprintf(`"Pi is %.2f"`, `3.14159`)'
```

### substr

Extract substring by start index and length

**Signature:** `string, number, number -> string`

**Examples:**

```text
# From index 1, length 3
substr('hello', `1`, `3`) -> \"ell\"
# From start
substr('hello', `0`, `2`) -> \"he\"
# Middle portion
substr('abcdef', `2`, `2`) -> \"cd\"
# Single character
substr('hello', `4`, `1`) -> \"o\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'substr(`"hello"`, `1`, `3`)'
```

### title

Convert to title case

**Signature:** `string -> string`

**Examples:**

```text
# Basic title case
title('hello world') -> \"Hello World\"
# From uppercase
title('HELLO WORLD') -> \"Hello World\"
# Single word
title('hello') -> \"Hello\"
# Single letters
title('a b c') -> \"A B C\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'title(`"hello world"`)'
```

### train_case

Convert to Train-Case (also known as HTTP-Header-Case)

**Signature:** `string -> string`

**Examples:**

```text
# From camelCase
train_case('helloWorld') -> "Hello-World"
# From snake_case
train_case('hello_world') -> "Hello-World"
# From lowercase
train_case('content type') -> "Content-Type"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'train_case(`"helloWorld"`)'
```

### trim

Remove leading and trailing whitespace

**Signature:** `string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Remove both sides
trim('  hello  ') -> \"hello\"
# No whitespace
trim('hello') -> \"hello\"
# Only whitespace
trim('   ') -> \"\"
# Tabs and newlines
trim('\t\nhello\t\n') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trim(`"  hello  "`)'
```

### trim_left

Remove leading whitespace

**Signature:** `string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Remove leading spaces
trim_left('  hello') -> \"hello\"
# Trailing preserved
trim_left('hello  ') -> \"hello  \"
# Remove tabs
trim_left('\t\thello') -> \"hello\"
# No change needed
trim_left('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trim_left(`"  hello"`)'
```

### trim_right

Remove trailing whitespace

**Signature:** `string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Remove trailing spaces
trim_right('hello  ') -> \"hello\"
# Leading preserved
trim_right('  hello') -> \"  hello\"
# Remove tabs
trim_right('hello\t\t') -> \"hello\"
# No change needed
trim_right('hello') -> \"hello\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trim_right(`"hello  "`)'
```

### upper

Convert string to uppercase

**Signature:** `string -> string`

**JEP:** JEP-014

**Examples:**

```text
# Basic uppercase
upper('hello') -> \"HELLO\"
# Mixed case
upper('Hello World') -> \"HELLO WORLD\"
# Already uppercase
upper('HELLO') -> \"HELLO\"
# With numbers
upper('abc123') -> \"ABC123\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'upper(`"hello"`)'
```

### wrap

Wrap text to specified width

**Signature:** `string, number -> string`

**Examples:**

```text
# Wrap at word boundary
wrap('hello world', `5`) -> \"hello\\nworld\"
# Multiple wraps
wrap('a b c d e', `3`) -> \"a b\\nc d\\ne\"
# No wrap needed
wrap('short', `10`) -> \"short\"
# Longer text
wrap('one two three', `7`) -> \"one two\\nthree\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'wrap(`"hello world"`, `5`)'
```

