# jpx

Python bindings for [jpx](https://github.com/joshrotenberg/jpx), a JMESPath query
engine with 470+ extension functions on top of the standard JMESPath specification.

The bindings are built in Rust with [PyO3](https://pyo3.rs) and ship as
abi3 wheels, so a single wheel per platform works on CPython 3.9 and newer.

## Install

```bash
pip install jpx
```

## Quick start

```python
import jpx

data = {"people": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}

# Run a query
jpx.search("people[?age > `28`].name", data)
# ['Alice']

# Compile once, reuse many times
expr = jpx.compile("people[*].name")
expr.search(data)
# ['Alice', 'Bob']

# Validate an expression without running it
jpx.validate("people[*].name")
# {'valid': True, ...}
```

## Discovering functions

```python
jpx.list_categories()           # extension categories
jpx.list_functions("string")    # functions in a category
jpx.describe("group_by")        # metadata for a single function
```

## Full engine

`JpxEngine` exposes the complete engine: batch evaluation, function
introspection, JSON utilities (diff, patch, merge, stats, paths), and a named
query store scoped to that `JpxEngine` instance.

```python
from jpx import JpxEngine

engine = JpxEngine()
engine.evaluate("a.b", {"a": {"b": 1}})
engine.diff({"a": 1}, {"a": 2})        # RFC 6902 patch
engine.stats({"a": [1, 2, 3]})          # structural stats
```

## Links

- Documentation: https://joshrotenberg.github.io/jpx/
- Source and issues: https://github.com/joshrotenberg/jpx

## License

MIT OR Apache-2.0
