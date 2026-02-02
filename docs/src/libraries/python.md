# Installation

## From PyPI

```bash
pip install jmespath-extensions
```

## From Source

```bash
# Clone the repository
git clone https://github.com/joshrotenberg/jmespath-extensions
cd jmespath-extensions/crates/jmespath-extensions-py

# Install maturin if needed
pip install maturin

# Build and install
maturin develop --release
```

## Verify Installation

```python
import jmespath_extensions

# Check it works
result = jmespath_extensions.search("length(@)", [1, 2, 3])
print(result)  # 3

# List available functions
functions = jmespath_extensions.list_functions()
print(f"Available functions: {len(functions)}")
```

## Requirements

- Python 3.8+
- No runtime dependencies (Rust extensions are statically linked)
# Usage

## Basic Searching

The primary function is `search()`, which evaluates a JMESPath expression against data:

```python
import jmespath_extensions as jpx

data = {
    "users": [
        {"name": "alice", "age": 30, "active": True},
        {"name": "bob", "age": 25, "active": False},
        {"name": "carol", "age": 35, "active": True}
    ]
}

# Extract active users
active = jpx.search("users[?active].name", data)
print(active)  # ['alice', 'carol']

# Use extended functions
result = jpx.search("users | sort_by(@, &age) | [*].name", data)
print(result)  # ['bob', 'alice', 'carol']
```

## Compiled Expressions

For repeated use, compile the expression once:

```python
import jmespath_extensions as jpx

# Compile once
expr = jpx.compile_expr("users[?age > `30`].name")

# Use many times
for dataset in datasets:
    result = expr.search(dataset)
    print(result)
```

## Exploring Functions

### List All Functions

```python
import jmespath_extensions as jpx

# All functions
all_funcs = jpx.list_functions()
print(f"Total: {len(all_funcs)} functions")

# Filter by category
string_funcs = jpx.list_functions("string")
print(f"String functions: {string_funcs}")
```

### List Categories

```python
categories = jpx.list_categories()
print(categories)
# ['standard', 'string', 'array', 'object', 'math', 'datetime', ...]
```

### Get Function Details

```python
info = jpx.describe("split")
print(info)
# {
#   'name': 'split',
#   'category': 'String',
#   'signature': 'string, string -> array',
#   'description': 'Split string by delimiter...',
#   'example': 'split(`"a,b,c"`, `","`) -> ["a", "b", "c"]'
# }
```

## Query Libraries

Parse and use `.jpx` query library files:

```python
import jmespath_extensions as jpx

# Parse a query library
content = '''
-- :name active-users
-- :desc Get active users
users[?active]

-- :name user-count
length(users)
'''

library = jpx.parse_query_library(content)

# List queries
for query in library.list():
    print(f"{query.name}: {query.description}")

# Get a specific query
query = library.get("active-users")
if query:
    result = jpx.search(query.expression, data)
    
# Get all query names
names = library.names()
```

### Check if Content is a Query Library

```python
# Detect format
if jpx.is_query_library(content):
    library = jpx.parse_query_library(content)
else:
    # Plain expression
    result = jpx.search(content.strip(), data)
```

## Extended Functions Examples

### String Functions

```python
# Split and join
jpx.search('split(`"a,b,c"`, `","`)', {})  # ['a', 'b', 'c']
jpx.search('join(`"-"`, `["a","b","c"]`)', {})  # 'a-b-c'

# Case conversion
jpx.search('upper(`"hello"`)', {})  # 'HELLO'
jpx.search('snake_case(`"helloWorld"`)', {})  # 'hello_world'
```

### Date/Time Functions

```python
# Current time
jpx.search('now()', {})  # 1699900000 (Unix timestamp)
jpx.search('now_iso()', {})  # '2024-01-15T10:30:00Z'

# Parse and format dates
jpx.search('parse_datetime(`"2024-01-15"`, `"%Y-%m-%d"`)', {})
```

### Math Functions

```python
data = {"values": [1, 2, 3, 4, 5]}

jpx.search('mean(values)', data)  # 3.0
jpx.search('std_dev(values)', data)  # 1.414...
jpx.search('percentile(values, `50`)', data)  # 3
```

### Fuzzy Matching

```python
tools = [
    {"name": "create_user", "description": "Create a new user"},
    {"name": "delete_user", "description": "Delete an existing user"},
    {"name": "list_users", "description": "List all users"}
]

# Fuzzy search
results = jpx.search(
    'fuzzy_search(@, `"name,description"`, `"user create"`)',
    tools
)
```

## Error Handling

```python
import jmespath_extensions as jpx

try:
    # Invalid expression
    jpx.search("invalid[", {})
except ValueError as e:
    print(f"Parse error: {e}")

try:
    # Runtime error
    jpx.search("length(@)", "not an array")
except Exception as e:
    print(f"Evaluation error: {e}")
```
# API Reference

## Functions

### search

```python
def search(expression: str, data: Any) -> Any
```

Evaluate a JMESPath expression against data.

**Parameters:**
- `expression`: JMESPath expression string
- `data`: Python data (dict, list, str, int, float, bool, None)

**Returns:** The result of the expression evaluation

**Raises:** `ValueError` for parse errors, `Exception` for runtime errors

**Example:**
```python
result = jpx.search("users[0].name", {"users": [{"name": "alice"}]})
# Returns: "alice"
```

---

### compile_expr

```python
def compile_expr(expression: str) -> CompiledExpression
```

Compile a JMESPath expression for repeated use.

**Parameters:**
- `expression`: JMESPath expression string

**Returns:** `CompiledExpression` object

**Raises:** `ValueError` for parse errors

**Example:**
```python
expr = jpx.compile_expr("length(@)")
result = expr.search([1, 2, 3])  # 3
```

---

### list_functions

```python
def list_functions(category: Optional[str] = None) -> List[str]
```

List available function names.

**Parameters:**
- `category`: Optional category filter (e.g., "string", "math", "datetime")

**Returns:** List of function names

**Example:**
```python
all_funcs = jpx.list_functions()
string_funcs = jpx.list_functions("string")
```

---

### list_categories

```python
def list_categories() -> List[str]
```

List all function categories.

**Returns:** List of category names

**Example:**
```python
categories = jpx.list_categories()
# ['standard', 'string', 'array', 'object', 'math', ...]
```

---

### describe

```python
def describe(name: str) -> Optional[Dict]
```

Get detailed information about a function.

**Parameters:**
- `name`: Function name or alias

**Returns:** Dictionary with function info, or `None` if not found

**Return fields:**
- `name`: Function name
- `category`: Category name
- `signature`: Type signature
- `description`: Description text
- `example`: Example usage

**Example:**
```python
info = jpx.describe("split")
print(info["signature"])  # "string, string -> array"
```

---

### parse_query_library

```python
def parse_query_library(content: str) -> QueryLibrary
```

Parse a query library from string content.

**Parameters:**
- `content`: Query library content in `.jpx` format

**Returns:** `QueryLibrary` object

**Raises:** `ValueError` for parse errors

**Example:**
```python
library = jpx.parse_query_library('''
-- :name my-query
-- :desc A sample query
users[?active]
''')
```

---

### is_query_library

```python
def is_query_library(content: str) -> bool
```

Check if content appears to be a query library format.

**Parameters:**
- `content`: String content to check

**Returns:** `True` if content starts with query library directives

**Example:**
```python
if jpx.is_query_library(content):
    library = jpx.parse_query_library(content)
```

---

## Classes

### CompiledExpression

A compiled JMESPath expression for efficient repeated evaluation.

#### Methods

##### search

```python
def search(self, data: Any) -> Any
```

Evaluate this expression against data.

**Parameters:**
- `data`: Python data to evaluate against

**Returns:** Evaluation result

---

### QueryLibrary

A collection of named queries parsed from a `.jpx` file.

#### Methods

##### get

```python
def get(self, name: str) -> Optional[NamedQuery]
```

Get a query by name.

**Parameters:**
- `name`: Query name

**Returns:** `NamedQuery` or `None`

##### list

```python
def list(self) -> List[NamedQuery]
```

Get all queries in the library.

**Returns:** List of `NamedQuery` objects

##### names

```python
def names(self) -> List[str]
```

Get all query names.

**Returns:** List of query names

---

### NamedQuery

A single named query from a query library.

#### Properties

- `name: str` - Query name
- `description: Optional[str]` - Query description
- `expression: str` - JMESPath expression
- `line_number: int` - Line number in source file

**Example:**
```python
query = library.get("my-query")
if query:
    print(f"{query.name}: {query.description}")
    result = jpx.search(query.expression, data)
```
