# Why jpx?

"I could just write Python or JavaScript for this - why use jpx?"

This is a fair question. Here's why jpx is worth learning for JSON wrangling.

## Almost No Syntax

JMESPath's entire syntax fits on an index card:

| Syntax | Example | Meaning |
|--------|---------|---------|
| Dot access | `foo.bar` | Get nested field |
| Brackets | `[0]`, `[*]`, `[?filter]` | Index, project, filter |
| Pipe | `a \| b` | Chain expressions |
| Functions | `name(args)` | Call a function |
| Literals | `` `"string"` ``, `` `123` ``, `` `true` `` | Literal values |

That's it. No `for`, `if`, `def`, `import`, `class`, `let`, `const`, `=>`, `async`, `await`...

You can learn the entire language in 10 minutes. The power comes from composing simple pieces, not memorizing syntax - the same insight that makes Lisp and shell pipelines compelling.

Compare to Python where "I want to extract some fields" requires knowing:

- Variable assignment
- Import statements
- List comprehensions (or for loops)
- Dictionary access
- String formatting
- File I/O

## Zero Boilerplate

No imports. No script files. No virtual environments. Just pipe and go:

```bash
curl -s https://api.example.com/users | jpx 'users[*].name'
```

Compare to the Python equivalent:

```python
import json
import sys

data = json.load(sys.stdin)
names = [user["name"] for user in data["users"]]
print(json.dumps(names))
```

That's 5 lines (plus saving to a file, making it executable, etc.) vs. a single shell command.

## Composable One-Liners

Shell pipelines are underrated. With jpx, you can do complex transformations in a single expression:

```bash
# Fetch, filter, transform, and sort in one line
curl -s https://api.github.com/users/octocat/repos \
  | jpx '[?stargazers_count > `100`] | sort_by(@, &stargazers_count) | reverse(@) | [*].{name: name, stars: stargazers_count}'
```

Or break it into multiple jpx calls when debugging:

```bash
curl -s https://api.github.com/users/octocat/repos \
  | jpx '[?stargazers_count > `100`]' \
  | jpx 'sort_by(@, &stargazers_count) | reverse(@) | [*].{name: name, stars: stargazers_count}'
```

Each step is independently testable. Add or remove transformations without rewriting a script.

## Declarative vs Imperative

You describe *what* you want, not *how* to loop through it:

**Imperative (Python):**
```python
result = []
for item in data["items"]:
    if item["active"]:
        result.append({"n": item["name"], "v": item["value"]})
```

**Declarative (jpx):**
```bash
jpx 'items[?active].{n: name, v: value}'
```

The jpx version is often more readable because it expresses intent directly.

## The Learning Curve Comparison

| Task | Python | jpx |
|------|--------|-----|
| Get nested field | `data["users"][0]["name"]` | `users[0].name` |
| Filter array | `[x for x in items if x["active"]]` | `items[?active]` |
| Transform all | `[{"n": x["name"]} for x in items]` | `items[*].{n: name}` |
| Chain operations | Multiple lines or nested calls | `a \| b \| c` |
| Sort by field | `sorted(items, key=lambda x: x["age"])` | `sort_by(items, &age)` |

## Consistent Interface

Same syntax across:
- CLI (`jpx 'query'`)
- MCP server (for AI assistants)
- Python bindings (`jpx.search('query', data)`)
- Rust library

Learn once, use everywhere.

## No Runtime Dependencies

jpx is a single ~15MB binary. It works on any machine without Python, Node, or anything else installed.

- Great for CI/CD pipelines
- Works in minimal containers
- No "works on my machine" issues
- No dependency conflicts

## Built for Exploration

When you're exploring an unfamiliar API response, jpx helps you discover what's there:

```bash
# Interactive REPL
jpx --repl

# See all paths in the data
jpx --paths -f data.json

# Search for functions
jpx --search "date"

# Get help on any function
jpx --describe format_date

# Find similar functions
jpx --similar levenshtein
```

Python requires knowing what to import before you start. jpx lets you explore first.

## Safe for Automation

JMESPath expressions are pure functions with no side effects:

- No "oops I overwrote my file" moments
- No network calls from within queries
- No shell injection vulnerabilities
- Predictable, deterministic behavior

This makes jpx safe to use in scripts and automation.

## When to Use What

jpx is for **ad-hoc JSON exploration and transformation**:

- "What's in this API response?"
- "Extract these fields from 50 log files"
- "Quick data munging in a shell script"
- "Prototype a transformation before implementing it"

**Use jpx when:**
- Quick field extraction from JSON
- Ad-hoc API exploration
- Shell script data processing
- CI/CD pipeline JSON manipulation
- You need to explain the query to someone else

**Use Python/JavaScript when:**
- Complex business logic with many conditionals
- Stateful transformations
- Database operations
- Production ETL pipelines
- You need to integrate with other services

The sweet spot: **use jpx for the JSON transformation step** inside larger Python/JS applications:

```python
import jpx

# Complex Python logic here...
data = fetch_from_api()

# Let jpx handle the transformation
result = jpx.search('items[?active].{id: id, name: upper(name)}', data)

# More Python logic...
save_to_database(result)
```

## jpx vs jq

Both tools query and transform JSON from the command line. Here's how they compare.

### What's the Same

- **Pipe-based composition**: Both chain operations with `|`
- **Zero dependencies**: Single binaries, no runtime needed
- **Streaming capable**: Both can process large files
- **Shell-friendly**: Designed for pipelines and scripting

### Syntax Comparison

| Task | jq | jpx |
|------|----|----|
| Get field | `.name` | `name` |
| Nested field | `.user.name` | `user.name` |
| Array element | `.[0]` | `[0]` |
| All elements | `.[]` | `[*]` |
| Filter | `select(.age > 30)` | `[?age > \`30\`]` |
| Project fields | `{name, age}` | `{name: name, age: age}` |
| Length | `length` | `length(@)` |
| Sort | `sort` | `sort(@)` |
| Sort by field | `sort_by(.age)` | `sort_by(@, &age)` |
| Map | `map(.name)` | `[*].name` or `map(&name, @)` |
| Unique | `unique` | `unique(@)` |
| Literals | `30` | `` \`30\` `` (backticks) |

### What jq Does Better

- **Recursive descent**: `..` traverses all levels
- **Variable bindings**: `as $x` for reusing values
- **String interpolation**: `"Hello \(.name)"`
- **Conditionals**: `if-then-else` expressions
- **Define functions**: `def name(args): body;`
- **Reduce**: `reduce .[] as $x (init; update)`

### What jpx Does Better

- **More built-in functions**: 490+ vs ~50 in jq
- **Domain-specific functions**: NLP, geo, fuzzy matching, phonetic, etc.
- **Function discovery**: `--search`, `--describe`, `--similar`
- **Readable filters**: `[?age > \`30\`]` reads like SQL WHERE
- **MCP server**: Use with AI assistants
- **Python bindings**: Same queries in Python code

### Examples Side-by-Side

**Filter and transform:**
```bash
# jq
jq '[.[] | select(.active) | {name, score}]' data.json

# jpx
jpx '[?active].{name: name, score: score}' data.json
```

**Sort by field descending:**
```bash
# jq
jq 'sort_by(.score) | reverse' data.json

# jpx
jpx 'sort_by(@, &score) | reverse(@)' data.json
```

**String operations:**
```bash
# jq
jq '.name | ascii_upcase' data.json

# jpx
jpx 'upper(name)' data.json
```

**Aggregate:**
```bash
# jq
jq '[.[].score] | add / length' data.json

# jpx
jpx 'avg([*].score)' data.json
```

### Different Tools for Different Jobs

jq and jpx aren't really competing - they have different sweet spots.

**jq excels at:**
- **Recursive transformations**: Walking arbitrary nested structures with `..`
- **Algorithmic work**: When you need variables, conditionals, custom functions
- **Stream processing**: Native support for JSON streams
- **When you know jq**: If you're fluent in jq, its expressive power is hard to beat

**jpx excels at:**
- **Domain-specific analysis**: NLP (tokenize, stem, stopwords), fuzzy matching (levenshtein, soundex), geo (distance calculations), dates, validation
- **Explorability**: Built-in function search, describe, and similar - great when you're not sure what you need
- **Cross-platform queries**: Same expressions work in CLI, Python, Rust, and MCP
- **Quick data extraction**: Filter/project syntax reads like SQL, less cryptic than jq for simple cases

**Use both together:**
```bash
# jq for complex reshaping, jpx for domain functions
curl api.example.com | jq '.data | recurse | select(.type == "user")' | jpx '[*].{name: name, similarity: jaro_winkler(name, `"target"`)}'
```

The choice often comes down to: "Do I need jq's algorithmic power, or jpx's 490+ functions?"

## Summary

jpx trades generality for ergonomics. It can't do everything Python can, but for its intended purpose - querying and transforming JSON - it's faster to write, easier to read, and simpler to maintain.

The 10 minutes you spend learning JMESPath will save you hours of writing boilerplate data munging code.
