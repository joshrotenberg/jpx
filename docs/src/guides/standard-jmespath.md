# Standard JMESPath Only

This guide demonstrates jpx using **only standard JMESPath functions** - no extensions. This is useful when:

- You need queries portable to other JMESPath implementations
- You're learning JMESPath fundamentals
- You want to understand what's possible without extensions

You can also use `jpx --strict` to enforce standard-only mode, which will error if you accidentally use an extension function.

## Standard Functions Reference

JMESPath defines 26 built-in functions:

| Function | Description |
|----------|-------------|
| `abs(n)` | Absolute value |
| `avg(arr)` | Average of numbers |
| `ceil(n)` | Round up |
| `contains(subject, search)` | Check if contains value |
| `ends_with(str, suffix)` | Check string suffix |
| `floor(n)` | Round down |
| `join(sep, arr)` | Join array to string |
| `keys(obj)` | Get object keys |
| `length(val)` | Length of array/string/object |
| `map(&expr, arr)` | Apply expression to each element |
| `max(arr)` | Maximum value |
| `max_by(arr, &expr)` | Element with max expression value |
| `merge(obj1, obj2, ...)` | Merge objects |
| `min(arr)` | Minimum value |
| `min_by(arr, &expr)` | Element with min expression value |
| `not_null(val1, val2, ...)` | First non-null value |
| `reverse(arr)` | Reverse array or string |
| `sort(arr)` | Sort array |
| `sort_by(arr, &expr)` | Sort by expression |
| `starts_with(str, prefix)` | Check string prefix |
| `sum(arr)` | Sum of numbers |
| `to_array(val)` | Convert to array |
| `to_number(val)` | Convert to number |
| `to_string(val)` | Convert to string |
| `type(val)` | Get value type |
| `values(obj)` | Get object values |

---

## Sample Data

We'll use this sample dataset throughout the guide:

```bash
cat > books.json << 'EOF'
{
  "library": {
    "name": "City Library",
    "books": [
      {"id": 1, "title": "The Great Gatsby", "author": "F. Scott Fitzgerald", "year": 1925, "genres": ["fiction", "classic"], "rating": 4.2, "available": true},
      {"id": 2, "title": "To Kill a Mockingbird", "author": "Harper Lee", "year": 1960, "genres": ["fiction", "classic"], "rating": 4.5, "available": false},
      {"id": 3, "title": "1984", "author": "George Orwell", "year": 1949, "genres": ["fiction", "dystopian"], "rating": 4.3, "available": true},
      {"id": 4, "title": "Pride and Prejudice", "author": "Jane Austen", "year": 1813, "genres": ["fiction", "romance"], "rating": 4.4, "available": true},
      {"id": 5, "title": "The Catcher in the Rye", "author": "J.D. Salinger", "year": 1951, "genres": ["fiction", "classic"], "rating": 3.9, "available": false},
      {"id": 6, "title": "Brave New World", "author": "Aldous Huxley", "year": 1932, "genres": ["fiction", "dystopian"], "rating": 4.1, "available": true},
      {"id": 7, "title": "The Hobbit", "author": "J.R.R. Tolkien", "year": 1937, "genres": ["fiction", "fantasy"], "rating": 4.6, "available": true},
      {"id": 8, "title": "Animal Farm", "author": "George Orwell", "year": 1945, "genres": ["fiction", "satire"], "rating": 4.0, "available": false}
    ]
  }
}
EOF
```

---

## Basic Selection

### Access Nested Data

```bash
jpx --strict 'library.name' books.json
```
Output: `"City Library"`

### Get All Books

```bash
jpx --strict 'library.books' books.json
```

### Select Specific Fields

```bash
jpx --strict 'library.books[*].title' books.json
```
Output:
```json
["The Great Gatsby", "To Kill a Mockingbird", "1984", "Pride and Prejudice", "The Catcher in the Rye", "Brave New World", "The Hobbit", "Animal Farm"]
```

### Project Multiple Fields

```bash
jpx --strict 'library.books[*].{title: title, author: author}' books.json
```
Output:
```json
[
  {"title": "The Great Gatsby", "author": "F. Scott Fitzgerald"},
  {"title": "To Kill a Mockingbird", "author": "Harper Lee"},
  ...
]
```

---

## Filtering

### Filter by Condition

```bash
jpx --strict 'library.books[?available == `true`].title' books.json
```
Output:
```json
["The Great Gatsby", "1984", "Pride and Prejudice", "Brave New World", "The Hobbit"]
```

### Filter by Numeric Comparison

```bash
jpx --strict 'library.books[?rating >= `4.3`].{title: title, rating: rating}' books.json
```
Output:
```json
[
  {"title": "To Kill a Mockingbird", "rating": 4.5},
  {"title": "1984", "rating": 4.3},
  {"title": "Pride and Prejudice", "rating": 4.4},
  {"title": "The Hobbit", "rating": 4.6}
]
```

### Filter by Year Range

```bash
jpx --strict 'library.books[?year >= `1940` && year < `1960`].title' books.json
```
Output:
```json
["1984", "The Catcher in the Rye", "Animal Farm"]
```

### Filter with String Functions

```bash
# Books by authors starting with "J"
jpx --strict 'library.books[?starts_with(author, `J`)].{title: title, author: author}' books.json
```
Output:
```json
[
  {"title": "Pride and Prejudice", "author": "Jane Austen"},
  {"title": "The Catcher in the Rye", "author": "J.D. Salinger"},
  {"title": "The Hobbit", "author": "J.R.R. Tolkien"}
]
```

### Filter with Contains

```bash
# Books with "The" in the title
jpx --strict 'library.books[?contains(title, `The`)].title' books.json
```
Output:
```json
["The Great Gatsby", "The Catcher in the Rye", "The Hobbit"]
```

### Check Array Membership

```bash
# Books in the "dystopian" genre
jpx --strict 'library.books[?contains(genres, `dystopian`)].title' books.json
```
Output:
```json
["1984", "Brave New World"]
```

---

## Sorting

### Sort by Field

```bash
jpx --strict 'sort_by(library.books, &year)[*].{title: title, year: year}' books.json
```
Output (oldest first):
```json
[
  {"title": "Pride and Prejudice", "year": 1813},
  {"title": "The Great Gatsby", "year": 1925},
  {"title": "Brave New World", "year": 1932},
  ...
]
```

### Sort Descending (Reverse)

```bash
jpx --strict 'reverse(sort_by(library.books, &rating))[*].{title: title, rating: rating}' books.json
```
Output (highest rated first):
```json
[
  {"title": "The Hobbit", "rating": 4.6},
  {"title": "To Kill a Mockingbird", "rating": 4.5},
  {"title": "Pride and Prejudice", "rating": 4.4},
  ...
]
```

### Sort Simple Arrays

```bash
jpx --strict 'sort(library.books[*].year)' books.json
```
Output:
```json
[1813, 1925, 1932, 1937, 1945, 1949, 1951, 1960]
```

---

## Aggregation

### Count Items

```bash
jpx --strict 'length(library.books)' books.json
```
Output: `8`

### Count Filtered Items

```bash
jpx --strict 'length(library.books[?available == `true`])' books.json
```
Output: `5`

### Sum Values

```bash
jpx --strict 'sum(library.books[*].year)' books.json
```
Output: `15512`

### Average

```bash
jpx --strict 'avg(library.books[*].rating)' books.json
```
Output: `4.25`

### Min and Max

```bash
jpx --strict '{
  oldest_year: min(library.books[*].year),
  newest_year: max(library.books[*].year),
  lowest_rating: min(library.books[*].rating),
  highest_rating: max(library.books[*].rating)
}' books.json
```
Output:
```json
{
  "oldest_year": 1813,
  "newest_year": 1960,
  "lowest_rating": 3.9,
  "highest_rating": 4.6
}
```

### Find Element with Min/Max

```bash
# Highest rated book
jpx --strict 'max_by(library.books, &rating).{title: title, rating: rating}' books.json
```
Output:
```json
{"title": "The Hobbit", "rating": 4.6}
```

```bash
# Oldest book
jpx --strict 'min_by(library.books, &year).{title: title, year: year}' books.json
```
Output:
```json
{"title": "Pride and Prejudice", "year": 1813}
```

---

## Object Operations

### Get Keys

```bash
jpx --strict 'keys(library.books[0])' books.json
```
Output:
```json
["id", "title", "author", "year", "genres", "rating", "available"]
```

### Get Values

```bash
jpx --strict 'values(library.books[0])' books.json
```
Output:
```json
[1, "The Great Gatsby", "F. Scott Fitzgerald", 1925, ["fiction", "classic"], 4.2, true]
```

### Merge Objects

```bash
jpx --strict 'merge(library.books[0], {checked_out: `true`, due_date: `2025-02-01`})' books.json
```
Output:
```json
{
  "id": 1,
  "title": "The Great Gatsby",
  "author": "F. Scott Fitzgerald",
  "year": 1925,
  "genres": ["fiction", "classic"],
  "rating": 4.2,
  "available": true,
  "checked_out": true,
  "due_date": "2025-02-01"
}
```

---

## Type Conversions

### Check Type

```bash
jpx --strict 'type(library.books)' books.json
```
Output: `"array"`

```bash
jpx --strict 'type(library.books[0].rating)' books.json
```
Output: `"number"`

### Convert to String

```bash
jpx --strict 'library.books[*].{title: title, year_str: to_string(year)}' books.json
```

### Convert to Number

```bash
echo '{"value": "42"}' | jpx --strict 'to_number(value)'
```
Output: `42`

### Convert to Array

```bash
jpx --strict 'to_array(library.name)' books.json
```
Output: `["City Library"]`

---

## String Operations

### Join Array Elements

```bash
jpx --strict 'join(`, `, library.books[*].title)' books.json
```
Output: `"The Great Gatsby, To Kill a Mockingbird, 1984, Pride and Prejudice, The Catcher in the Rye, Brave New World, The Hobbit, Animal Farm"`

### Check Prefix/Suffix

```bash
# Titles ending with specific words
jpx --strict 'library.books[?ends_with(title, `Rye`)].title' books.json
```
Output: `["The Catcher in the Rye"]`

---

## The Map Function

The `map` function applies an expression to each element:

```bash
# Extract just the year from each book
jpx --strict 'map(&year, library.books)' books.json
```
Output:
```json
[1925, 1960, 1949, 1813, 1951, 1932, 1937, 1945]
```

```bash
# Create formatted strings
jpx --strict 'map(&{book: title, published: year}, library.books)' books.json
```

---

## Handling Nulls

### First Non-Null Value

```bash
echo '{"a": null, "b": null, "c": "found"}' | jpx --strict 'not_null(a, b, c)'
```
Output: `"found"`

### Null-Safe Field Access

```bash
echo '{"items": [{"name": "a", "value": 1}, {"name": "b"}]}' | \
  jpx --strict 'items[*].{name: name, value: not_null(value, `0`)}'
```
Output:
```json
[
  {"name": "a", "value": 1},
  {"name": "b", "value": 0}
]
```

---

## Pipelines

Chain operations with the pipe operator `|`:

```bash
# Filter, sort, and take top 3
jpx --strict 'library.books 
  | [?available == `true`] 
  | sort_by(@, &rating) 
  | reverse(@) 
  | [:3] 
  | [*].{title: title, rating: rating}' books.json
```
Output:
```json
[
  {"title": "The Hobbit", "rating": 4.6},
  {"title": "Pride and Prejudice", "rating": 4.4},
  {"title": "1984", "rating": 4.3}
]
```

---

## Complex Example: Library Report

Combining multiple standard functions:

```bash
jpx --strict '{
  library_name: library.name,
  total_books: length(library.books),
  available_count: length(library.books[?available == `true`]),
  checked_out_count: length(library.books[?available == `false`]),
  average_rating: avg(library.books[*].rating),
  rating_range: {
    min: min(library.books[*].rating),
    max: max(library.books[*].rating)
  },
  year_range: {
    oldest: min(library.books[*].year),
    newest: max(library.books[*].year)
  },
  top_rated: max_by(library.books, &rating).title,
  authors: sort(library.books[*].author),
  available_titles: sort(library.books[?available == `true`][*].title)
}' books.json
```

Output:
```json
{
  "library_name": "City Library",
  "total_books": 8,
  "available_count": 5,
  "checked_out_count": 3,
  "average_rating": 4.25,
  "rating_range": {"min": 3.9, "max": 4.6},
  "year_range": {"oldest": 1813, "newest": 1960},
  "top_rated": "The Hobbit",
  "authors": ["Aldous Huxley", "F. Scott Fitzgerald", "George Orwell", "George Orwell", "Harper Lee", "J.D. Salinger", "J.R.R. Tolkien", "Jane Austen"],
  "available_titles": ["1984", "Brave New World", "Pride and Prejudice", "The Great Gatsby", "The Hobbit"]
}
```

---

## When to Use Extensions

Standard JMESPath is powerful for basic transformations, but you'll want extensions for:

- **String manipulation**: `split`, `trim`, `lower`, `upper`, `replace`
- **Date/time**: `from_epoch`, `format_date`, `date_diff`
- **Advanced math**: `median`, `stddev`, `round`, `percentile`
- **Grouping**: `group_by`, `frequencies`, `unique`
- **Text processing**: `tokens`, `stems`, `word_count`
- **Hashing**: `sha256`, `md5`, `base64_encode`
- **And 350+ more functions**

To see what's available:
```bash
jpx --list-functions
jpx --search "group"
jpx --describe group_by
```

## Related

- [JMESPath Specification](https://jmespath.org/specification.html)
- [jpx Function Reference](../functions/index.md)
- [Real-World Dataset Guides](./index.md)
