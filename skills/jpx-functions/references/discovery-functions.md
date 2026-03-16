# Discovery Functions (3)

## `fuzzy_match`

**Signature:** `string, string -> object`

Check if a single string value matches a query, returning match details

```
fuzzy_match('get_user', 'get_user') -> {matches: true, score: 1000, match_type: 'exact'}
```
_Exact match_

```
fuzzy_match('get_user_info', 'get') -> {matches: true, score: 800, match_type: 'prefix'}
```
_Prefix match_

---

## `fuzzy_score`

**Signature:** `string, string -> number`

Get the numeric match score between a value and query (higher is better match)

```
fuzzy_score('hello', 'hello') -> 1000
```
_Exact match score (highest)_

```
fuzzy_score('hello_world', 'hello') -> 800
```
_Prefix match score_

---

## `fuzzy_search`

**Signature:** `array, string|object, string -> array`

Search an array of objects by multiple fields, returning matches sorted by relevance score

```
fuzzy_search(tools, 'name,description', 'user') -> [{item: {...}, score: 100, match_type: 'exact', matched_field: 'name'}, ...]
```
_Search with comma-separated fields_

```
fuzzy_search(tools, `{"name": 10, "description": 5}`, 'cache') -> weighted search
```
_Search with weighted fields_

---

