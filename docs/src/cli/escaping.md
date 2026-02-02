# String Literals and Escaping

This guide covers how to properly write string literals, regex patterns, and special characters in JMESPath expressions. Understanding escaping is crucial for working with `split()`, `replace()`, regex functions, and any operation involving literal strings.

## The Golden Rule

In JMESPath:
- **Single quotes** `'...'` = **Identifiers** (field names)
- **Backticks with JSON** `` `"..."` `` = **String literals**

```bash
# WRONG - 'hello' is an identifier (looks for a field named "hello")
echo '{"hello": "world"}' | jpx "'hello'"
# null (unless there's a field whose value equals the string "hello")

# RIGHT - `"hello"` is a string literal
echo '{"data": "hello"}' | jpx 'data == `"hello"`'
# true
```

## String Literals Quick Reference

| What you want | JMESPath syntax | Shell example |
|--------------|-----------------|---------------|
| String `hello` | `` `"hello"` `` | `jpx 'length(`"hello"`)' --null-input` |
| String with spaces | `` `"hello world"` `` | `jpx 'length(`"hello world"`)' --null-input` |
| Empty string | `` `""` `` | `jpx '@ == `""`' --null-input` |
| Number `42` | `` `42` `` | `jpx '`42` + `8`' --null-input` |
| Boolean | `` `true` `` or `` `false` `` | `jpx '`true`' --null-input` |
| Null | `` `null` `` | `jpx '`null`' --null-input` |

## Special Characters

### Newlines, Tabs, and Escapes

Inside backtick literals, use standard JSON escape sequences:

| Character | JMESPath syntax | Example |
|-----------|-----------------|---------|
| Newline | `` `"\n"` `` | `split(@, `"\n"`)` |
| Tab | `` `"\t"` `` | `split(@, `"\t"`)` |
| Carriage return | `` `"\r"` `` | `replace(@, `"\r"`, `""`)` |
| Backslash | `` `"\\"` `` | `split(@, `"\\"`)` |
| Double quote | `` `"\""` `` | `contains(@, `"\""`)` |

### Example: Splitting by Newlines

```bash
# Split multi-line text into array
echo '"line1\nline2\nline3"' | jpx 'split(@, `"\n"`)'
# ["line1", "line2", "line3"]
```

### Example: Working with Tabs

```bash
# Split TSV (tab-separated values)
echo '"name\tage\tcity"' | jpx 'split(@, `"\t"`)'
# ["name", "age", "city"]
```

## Regex Patterns

Regex functions (`regex_match`, `regex_extract`, `regex_replace`, etc.) need their patterns as string literals.

### Regex Quick Reference

| Regex pattern | JMESPath syntax | What it matches |
|--------------|-----------------|-----------------|
| `\d+` (digits) | `` `"\\d+"` `` | One or more digits |
| `\w+` (word chars) | `` `"\\w+"` `` | Word characters |
| `\s+` (whitespace) | `` `"\\s+"` `` | Whitespace |
| `[a-z]+` | `` `"[a-z]+"` `` | Lowercase letters |
| `^start` | `` `"^start"` `` | Starts with "start" |
| `end$` | `` `"end$"` `` | Ends with "end" |

**Key insight**: Backslash escapes in regex patterns need to be escaped for JSON, so `\d` becomes `\\d` inside the backtick literal.

### Example: Extract Numbers

```bash
echo '"Order #12345"' | jpx 'regex_extract(@, `"\\d+"`)'
# "12345"
```

### Example: Match Email Pattern

```bash
echo '"contact@example.com"' | jpx 'regex_match(@, `"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"`)'
# true
```

### Example: Replace Whitespace

```bash
echo '"hello   world"' | jpx 'regex_replace(@, `"\\s+"`, `" "`)'
# "hello world"
```

## Shell Escaping Layers

When running jpx from a shell, you have multiple escaping layers:

1. **Shell** interprets your command line
2. **JMESPath** parses the expression
3. **JSON** (inside backticks) parses the literal
4. **Regex engine** (for regex functions) interprets the pattern

### Shell Quoting Strategies

**Single quotes (recommended)** - Shell doesn't interpret anything:

```bash
# Best: Use single quotes around the entire expression
echo '{"text": "hello"}' | jpx 'regex_match(text, `"\\w+"`)'
```

**Double quotes** - Need to escape backticks and backslashes:

```bash
# Harder: Must escape backticks and double backslashes
echo '{"text": "hello"}' | jpx "regex_match(text, \`\"\\\\w+\"\`)"
```

### Recommendation

Always use single quotes around your JMESPath expression when possible:

```bash
jpx 'your_expression_here'
```

This lets you write backticks and backslashes without shell interference.

## Common Mistakes

### Mistake 1: Using Single Quotes for Strings

```bash
# WRONG - 'hello' is an identifier
echo '{"x": "hello"}' | jpx "x == 'hello'"

# RIGHT - use backtick literal
echo '{"x": "hello"}' | jpx 'x == `"hello"`'
```

### Mistake 2: Forgetting Quotes Inside Backticks

```bash
# WRONG - `hello` is not valid JSON
jpx '`hello`' --null-input

# RIGHT - strings need quotes inside backticks
jpx '`"hello"`' --null-input
```

### Mistake 3: Over-escaping Regex Patterns

```bash
# WRONG - too many backslashes
echo '"abc123"' | jpx 'regex_extract(@, `"\\\\d+"`)'

# RIGHT - just escape once for JSON
echo '"abc123"' | jpx 'regex_extract(@, `"\\d+"`)'
```

### Mistake 4: Confusing Identifier vs String

```bash
# This looks for field named "name" - CORRECT
echo '{"name": "Alice"}' | jpx 'name'
# "Alice"

# This compares against literal string - CORRECT
echo '{"name": "Alice"}' | jpx 'name == `"Alice"`'
# true

# This looks for field named by VALUE of 'name' field - WRONG (usually)
echo '{"name": "Alice"}' | jpx "'name'"
# null (looks for field "name" on string "Alice")
```

## MCP Server Context

When using jpx through the MCP server (e.g., with Claude), there's an additional JSON encoding layer. The MCP server handles this automatically, but be aware:

- Expressions are passed as JSON strings
- The server unescapes before evaluating
- When writing expressions in MCP tool calls, you typically don't need extra escaping

### Example MCP Tool Call

```json
{
  "name": "jpx_query",
  "arguments": {
    "expression": "split(@, `\"\\n\"`)",
    "data": "line1\nline2\nline3"
  }
}
```

The MCP server handles the JSON string escaping, so `\"` becomes `"` and `\\n` becomes `\n` in the actual expression.

## Debugging Tips

1. **Use `--null-input`** to test expressions without needing input data:
   ```bash
   jpx '`"test\\nstring"`' --null-input
   ```

2. **Start simple** - verify your literal works before using it in a function:
   ```bash
   # First, check the literal
   jpx '`"\\d+"`' --null-input
   # "\d+"
   
   # Then use it
   echo '"abc123"' | jpx 'regex_extract(@, `"\\d+"`)'
   ```

3. **Check the AST** for complex expressions:
   ```bash
   jpx --ast 'split(@, `"\n"`)'
   ```

## Summary

| Context | Syntax | Example |
|---------|--------|---------|
| Field access | `fieldname` | `user.name` |
| String literal | `` `"string"` `` | `` `"hello"` `` |
| Newline in string | `` `"\n"` `` | `split(@, `"\n"`)` |
| Regex digit | `` `"\\d+"` `` | `regex_match(@, `"\\d+"`)` |
| Comparison | `field == `"value"`` | `status == `"active"`` |
