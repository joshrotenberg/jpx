# Cookbook

Common tasks and how to do them in jpx.

## Extracting Data

Most JSON wrangling starts with pulling out the fields you care about.

### Get a nested field
```bash
echo '{"user": {"profile": {"name": "Alice"}}}' | jpx 'user.profile.name'
# "Alice"
```

### Get multiple fields
```bash
echo '{"id": 1, "name": "Alice", "email": "a@example.com", "age": 30}' | jpx '{id: id, name: name}'
# {"id": 1, "name": "Alice"}
```

### Get a field from every object in an array
```bash
echo '[{"name": "Alice"}, {"name": "Bob"}]' | jpx '[*].name'
# ["Alice", "Bob"]
```

### Get with a default value
```bash
echo '{"name": "Alice"}' | jpx 'get(@, `"email"`, `"no email"`)'
# "no email"
```

## Filtering

Filter expressions (`[?condition]`) are one of JMESPath's most powerful features. They let you select array elements that match criteria without writing loops.

### Filter by a condition
```bash
echo '[{"status": "active"}, {"status": "inactive"}]' | jpx '[?status == `"active"`]'
# [{"status": "active"}]
```

### Filter by multiple conditions
```bash
echo '[{"age": 25, "active": true}, {"age": 35, "active": true}, {"age": 30, "active": false}]' \
  | jpx '[?age > `30` && active]'
# [{"age": 35, "active": true}]
```

### Filter by nested field
```bash
echo '[{"user": {"role": "admin"}}, {"user": {"role": "guest"}}]' | jpx '[?user.role == `"admin"`]'
# [{"user": {"role": "admin"}}]
```

### Check if field exists
```bash
echo '[{"name": "Alice", "email": "a@b.com"}, {"name": "Bob"}]' | jpx '[?email]'
# [{"name": "Alice", "email": "a@b.com"}]
```

## Transforming

Reshape data to match the structure you need - rename fields, compute new values, or convert between formats.

### Rename fields
```bash
echo '{"firstName": "Alice", "lastName": "Smith"}' | jpx '{first: firstName, last: lastName}'
# {"first": "Alice", "last": "Smith"}
```

### Add a computed field
```bash
echo '{"price": 100, "qty": 3}' | jpx '{price: price, qty: qty, total: multiply(price, qty)}'
# {"price": 100, "qty": 3, "total": 300}
```

### Convert object keys to snake_case
```bash
echo '{"firstName": "Alice", "lastName": "Smith"}' | jpx 'snake_keys(@)'
# {"first_name": "Alice", "last_name": "Smith"}
```

### Flatten nested structure
```bash
echo '{"user": {"name": "Alice", "address": {"city": "NYC"}}}' | jpx 'flatten(@)'
# {"user.name": "Alice", "user.address.city": "NYC"}
```

## Arrays

Common operations on lists of items - sorting, deduplication, grouping, and slicing.

### Sort
```bash
echo '[3, 1, 4, 1, 5]' | jpx 'sort(@)'
# [1, 1, 3, 4, 5]
```

### Sort objects by field
```bash
echo '[{"name": "Bob", "age": 25}, {"name": "Alice", "age": 30}]' | jpx 'sort_by(@, &name)'
# [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
```

### Remove duplicates
```bash
echo '[1, 2, 2, 3, 3, 3]' | jpx 'unique(@)'
# [1, 2, 3]
```

### First/last N items
```bash
echo '[1, 2, 3, 4, 5]' | jpx '[:3]'   # first 3
# [1, 2, 3]

echo '[1, 2, 3, 4, 5]' | jpx '[-2:]'  # last 2
# [4, 5]
```

### Group by field
```bash
echo '[{"dept": "eng", "name": "Alice"}, {"dept": "eng", "name": "Bob"}, {"dept": "sales", "name": "Carol"}]' \
  | jpx 'group_by(@, `"dept"`)'
# {"eng": [{"dept": "eng", "name": "Alice"}, {"dept": "eng", "name": "Bob"}], "sales": [...]}
```

### Chunk into batches
```bash
echo '[1, 2, 3, 4, 5, 6, 7]' | jpx 'chunk(@, `3`)'
# [[1, 2, 3], [4, 5, 6], [7]]
```

## Aggregation

Reduce arrays to summary values - counts, sums, averages, and grouped statistics.

### Count
```bash
echo '[1, 2, 3, 4, 5]' | jpx 'length(@)'
# 5
```

### Sum, average, min, max
```bash
echo '[10, 20, 30]' | jpx 'sum(@)'
# 60

echo '[10, 20, 30]' | jpx 'avg(@)'
# 20

echo '[10, 20, 30]' | jpx 'min(@)'
# 10
```

### Count by group
```bash
echo '[{"status": "active"}, {"status": "active"}, {"status": "inactive"}]' \
  | jpx 'group_by(@, `"status"`) | map_values(`"length(@)"`, @)'
# {"active": 2, "inactive": 1}
```

## Strings

Text manipulation - case conversion, splitting, joining, and search/replace.

### Upper/lower case
```bash
echo '"hello"' | jpx 'upper(@)'
# "HELLO"
```

### Split and join
```bash
echo '"a,b,c"' | jpx 'split(@, `","`)'
# ["a", "b", "c"]

echo '["a", "b", "c"]' | jpx 'join(`", "`, @)'
# "a, b, c"
```

### Replace
```bash
echo '"hello world"' | jpx 'replace(@, `"world"`, `"jpx"`)'
# "hello jpx"
```

### Trim whitespace
```bash
echo '"  hello  "' | jpx 'trim(@)'
# "hello"
```

## Dates

Working with timestamps - get current time, format for display, or calculate durations.

### Current time
```bash
jpx -n 'now()'
# 1705312200 (Unix timestamp)

jpx -n 'format_date(now(), `"%Y-%m-%dT%H:%M:%SZ"`)'
# "2024-01-15T10:30:00Z"
```

### Format a timestamp
```bash
jpx -n 'format_date(now(), `"%B %d, %Y"`)'
# "January 15, 2024"

# From a Unix timestamp
echo '1705312200' | jpx 'format_date(@, `"%Y-%m-%d"`)'
# "2024-01-15"
```

### Add time to a timestamp
```bash
jpx -n 'date_add(now(), `7`, `"days"`) | format_date(@, `"%Y-%m-%d"`)'
# "2024-01-22" (7 days from now)
```

## Working with Objects

Inspect and manipulate object structure - list keys, pick/omit fields, merge objects.

### Get all keys
```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx 'keys(@)'
# ["a", "b", "c"]
```

### Get all values
```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx 'values(@)'
# [1, 2, 3]
```

### Pick specific keys
```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx 'pick(@, [`"a"`, `"c"`])'
# {"a": 1, "c": 3}
```

### Omit specific keys
```bash
echo '{"a": 1, "b": 2, "c": 3}' | jpx 'omit(@, [`"b"`])'
# {"a": 1, "c": 3}
```

### Merge objects
```bash
echo '{}' | jpx 'merge({a: `1`}, {b: `2`})'
# {"a": 1, "b": 2}
```

## Cleaning Data

Real-world JSON is messy. These functions help you strip out nulls, empty strings, and other unwanted values.

### Remove nulls
```bash
echo '{"a": 1, "b": null, "c": 3}' | jpx 'remove_nulls(@)'
# {"a": 1, "c": 3}
```

### Remove empty strings
```bash
echo '{"a": "hello", "b": "", "c": "world"}' | jpx 'remove_empty_strings(@)'
# {"a": "hello", "c": "world"}
```

### Compact (remove all empty values)
```bash
echo '{"a": 1, "b": null, "c": "", "d": [], "e": {}}' | jpx 'remove_empty(@)'
# {"a": 1}
```

## Putting It Together

Chain these patterns together for real-world data processing tasks.

### Extract, filter, transform, sort
```bash
echo '[
  {"name": "Alice", "dept": "eng", "salary": 100000},
  {"name": "Bob", "dept": "sales", "salary": 80000},
  {"name": "Carol", "dept": "eng", "salary": 120000}
]' | jpx '[?dept == `"eng"`] | sort_by(@, &salary) | [*].{name: name, salary: salary}'
# [{"name": "Alice", "salary": 100000}, {"name": "Carol", "salary": 120000}]
```

### API response processing
```bash
curl -s 'https://api.github.com/users/octocat/repos' \
  | jpx '[?stargazers_count > `100`] | sort_by(@, &stargazers_count) | reverse(@) | [:5] | [*].{name: name, stars: stargazers_count}'
```

### Log analysis
```bash
cat logs.json | jpx '[?level == `"error"`] | group_by(@, `"service"`) | map_values(`"length(@)"`, @)'
# {"api": 12, "worker": 3, "scheduler": 1}
```

## Workflow Tips

jpx has built-in tools to help you build and reuse queries.

### Explore data structure first

Before writing a query, see what you're working with:

```bash
jpx --paths -f data.json
```
```
users (array)
users.0 (object)
users.0.name (string)
users.0.email (string)
users.0.role (string)
```

```bash
jpx --stats -f data.json
```
```
Type: object
Size: 2 keys
Depth: 3
Array fields:
  users: 150 items (object)
    Fields: name (100%), email (98%), role (100%), last_login (85%)
```

### Find functions you need

Don't memorize 400 functions - search for them:

```bash
jpx --search "date"
```
```
✓ Found 12 functions matching 'date':

  ▸ EXACT:
    date_add [datetime] - Add time to timestamp
    date_diff [datetime] - Difference between timestamps

  ▸ PREFIX:
    format_date [datetime] - Format timestamp as string
    parse_date [datetime] - Parse string to timestamp
    ...
```

```bash
jpx --describe format_date
```
```
format_date
===========
Category: datetime
Signature: number, string -> string
Description: Format a Unix timestamp as a string

Example:
  format_date(`1705312200`, '%Y-%m-%d') -> "2024-01-15"
```

### Save queries to files

For complex or reusable queries, store them in a `.jpx` file:

```sql
-- queries.jpx

-- :name active-users
-- :description Get all active users with their email
users[?status == `"active"`].{name: name, email: email}

-- :name error-summary  
-- :description Count errors by service
[?level == `"error"`] | group_by(@, `"service"`) | map_values(`"length(@)"`, @)
```

Then run by name:

```bash
jpx -Q queries.jpx:active-users -f data.json

# List available queries
jpx -Q queries.jpx --list-queries
```
```
Available queries in queries.jpx:

  active-users
    Get all active users with their email

  error-summary
    Count errors by service
```

### Debug complex expressions

When a query isn't working, break it down:

```bash
jpx --explain 'users[?active].name | sort(@)'
```
```
Expression AST:
  Pipe
  ├─ Projection
  │  ├─ Field: users
  │  └─ Filter: active
  │     └─ Field: name
  └─ Function: sort
     └─ Current (@)
```

### Benchmark performance

For queries you'll run often, check how fast they are:

```bash
jpx --bench 1000 '[*].{id: id, name: upper(name)}' -f users.json
```
```
Benchmarking: [*].{id: id, name: upper(name)}
Iterations: 1000

  Mean:    0.234ms
  Median:  0.215ms
  p95:     0.312ms
  p99:     0.456ms

Throughput: 4,273 ops/sec
```
