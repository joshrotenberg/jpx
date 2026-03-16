# Object Functions (51)

## `camel_keys`

**Signature:** `any -> any`

Recursively convert all keys to camelCase

```
camel_keys({user_name: "alice"}) -> {userName: "alice"}
```
_Snake to camel_

```
camel_keys({"user-name": "bob"}) -> {userName: "bob"}
```
_Kebab to camel_

---

## `chunk_by_size`

**Signature:** `array, number -> array`

Split an array into chunks of approximately the specified byte size

```
chunk_by_size([{a:1}, {b:2}, {c:3}], `100`) -> [[{a:1}, {b:2}, {c:3}]]
```
_All fit in one chunk_

```
chunk_by_size([{a:1}, {b:2}], `10`) -> [[{a:1}], [{b:2}]]
```
_Split into chunks_

---

## `compact_deep`

**Signature:** `array -> array`

Recursively compact arrays, removing nulls at all levels

```
compact_deep([[1, null], [null, 2]]) -> [[1], [2]]
```
_Remove nulls from nested arrays_

```
compact_deep([[1, null], [null, [2, null]]]) -> [[1], [[2]]]
```
_Deep nesting_

---

## `completeness`

**Signature:** `object -> number`

Calculate percentage of non-null fields (0-100)

```
completeness({a: 1, b: 2, c: 3}) -> 100
```
_All fields filled_

```
completeness({a: 1, b: null, c: null}) -> 33.33
```
_One of three filled_

---

## `data_quality_score`

**Signature:** `any -> object`

Analyze data quality and return score with detailed issues

```
data_quality_score({a: 1, b: 'hello'}).score -> 100
```
_Perfect data_

```
data_quality_score({a: null, b: ''}).issues -> [{path: 'a', issue: 'null'}, ...]
```
_Issues detected_

---

## `deep_diff`

**Signature:** `object, object -> object`

Structural diff between two objects

```
deep_diff({a: 1}, {a: 2}) -> {added: {}, removed: {}, changed: {a: {from: 1, to: 2}}}
```
_Changed value_

```
deep_diff({a: 1}, {b: 2}) -> {added: {b: 2}, removed: {a: 1}, changed: {}}
```
_Added and removed_

---

## `deep_equals`

**Signature:** `any, any -> boolean`

Deep equality check for any two values

```
deep_equals({a: {b: 1}}, {a: {b: 1}}) -> true
```
_Equal nested objects_

```
deep_equals({a: 1}, {a: 2}) -> false
```
_Different values_

---

## `deep_merge`

**Signature:** `object, object -> object`

Recursively merge objects

```
deep_merge({a: {b: 1}}, {a: {c: 2}}) -> {a: {b: 1, c: 2}}
```
_Merge nested objects_

```
deep_merge({a: 1}, {b: 2}) -> {a: 1, b: 2}
```
_Merge flat objects_

---

## `defaults`

**Signature:** `object, object -> object`

Assign default values for missing keys (shallow)

```
defaults({a: 1}, {a: 2, b: 3}) -> {a: 1, b: 3}
```
_Keep existing, add missing_

```
defaults({}, {a: 1, b: 2}) -> {a: 1, b: 2}
```
_All defaults applied_

---

## `defaults_deep`

**Signature:** `object, object -> object`

Recursively assign default values for missing keys

```
defaults_deep({a: {b: 1}}, {a: {c: 2}}) -> {a: {b: 1, c: 2}}
```
_Merge nested defaults_

```
defaults_deep({a: {b: 1}}, {a: {b: 2}}) -> {a: {b: 1}}
```
_Keep existing nested_

---

## `delete_path`

**Signature:** `any, string -> any`

Delete value at JSON pointer path (immutable)

```
delete_path({a: 1, b: 2}, '/b') -> {a: 1}
```
_Delete top-level key_

```
delete_path({a: {b: 1, c: 2}}, '/a/b') -> {a: {c: 2}}
```
_Delete nested key_

---

## `estimate_size`

**Signature:** `any -> number`

Estimate the JSON serialization size in bytes

```
estimate_size(`"hello"`) -> 7
```
_String with quotes_

```
estimate_size({a: 1}) -> 7
```
_Simple object_

---

## `flatten`

**Signature:** `object -> object`

Alias for flatten_keys - flatten nested object with dot notation keys

```
flatten({a: {b: 1}}) -> {\"a.b\": 1}
```
_Simple nested_

```
flatten({a: {b: {c: 1}}}) -> {\"a.b.c\": 1}
```
_Deep nested_

---

## `flatten_array`

**Signature:** `any, string? -> object`

Flatten nested objects and arrays with dot notation keys (arrays use numeric indices)

```
flatten_array({a: [1, 2]}) -> {\"a.0\": 1, \"a.1\": 2}
```
_Array with indices_

```
flatten_array({a: {b: [1, 2]}}) -> {\"a.b.0\": 1, \"a.b.1\": 2}
```
_Nested object with array_

---

## `flatten_keys`

**Signature:** `object -> object`

Flatten nested object with dot notation keys

```
flatten_keys({a: {b: 1}}) -> {\"a.b\": 1}
```
_Simple nested_

```
flatten_keys({a: {b: {c: 1}}}) -> {\"a.b.c\": 1}
```
_Deep nested_

---

## `from_entries`

**Signature:** `array -> object`

Alias for from_items. Convert array of [key, value] pairs to object. Familiar to jq/lodash users.

```
from_entries([['a', 1], ['b', 2]]) -> {a: 1, b: 2}
```
_Multiple pairs_

```
items({a: 1, b: 2}) | from_entries(@) -> {a: 1, b: 2}
```
_Round-trip with items_

---

## `from_items`

**Signature:** `array -> object`

Convert array of [key, value] pairs to object

```
from_items([['a', 1]]) -> {a: 1}
```
_Single pair_

```
from_items([['a', 1], ['b', 2]]) -> {a: 1, b: 2}
```
_Multiple pairs_

---

## `get`

**Signature:** `any, string, any? -> any`

Get value at dot-separated path with optional default

```
get({a: {b: 1}}, 'a.b') -> 1
```
_Nested path_

```
get({a: 1}, 'a') -> 1
```
_Top-level key_

---

## `get_path`

**Signature:** `any, string, any? -> any`

Get value at dot-separated path with optional default

```
get_path({a: {b: 1}}, 'a.b') -> 1
```
_Nested path_

---

## `has`

**Signature:** `any, string -> boolean`

Check if dot-separated path exists

```
has({a: {b: 1}}, 'a.b') -> true
```
_Nested path exists_

```
has({a: 1}, 'a') -> true
```
_Top-level exists_

---

## `has_path`

**Signature:** `any, string -> boolean`

Check if dot-separated path exists

```
has_path({a: {b: 1}}, 'a.b') -> true
```
_Nested path exists_

---

## `has_same_shape`

**Signature:** `any, any -> boolean`

Check if two values have the same structure (ignoring actual values)

```
has_same_shape({a: 1, b: 2}, {a: 99, b: 100}) -> true
```
_Same keys, different values_

```
has_same_shape({a: 1}, {a: 1, b: 2}) -> false
```
_Different keys_

---

## `infer_schema`

**Signature:** `any -> object`

Infer a JSON Schema-like type description from a value

```
infer_schema(`42`) -> {type: "number"}
```
_Number type_

```
infer_schema({name: "alice", age: 30}) -> {type: "object", properties: {name: {type: "string"}, age: {type: "number"}}}
```
_Object schema_

---

## `invert`

**Signature:** `object -> object`

Swap keys and values

```
invert({a: 'x'}) -> {x: 'a'}
```
_Swap key and value_

```
invert({a: 'b', c: 'd'}) -> {b: 'a', d: 'c'}
```
_Multiple pairs_

---

## `items`

**Signature:** `object -> array`

Convert object to array of [key, value] pairs

```
items({a: 1}) -> [[\"a\", 1]]
```
_Single key_

```
items({a: 1, b: 2}) -> [[\"a\", 1], [\"b\", 2]]
```
_Multiple keys_

---

## `kebab_keys`

**Signature:** `any -> any`

Recursively convert all keys to kebab-case

```
kebab_keys({userName: "alice"}) -> {"user-name": "alice"}
```
_Camel to kebab_

```
kebab_keys({user_name: "bob"}) -> {"user-name": "bob"}
```
_Snake to kebab_

---

## `leaves`

**Signature:** `any -> array`

Get all leaf values (non-object, non-array)

```
leaves({a: 1, b: [2, 3]}) -> [1, 2, 3]
```
_Mixed structure_

```
leaves({a: {b: 1}}) -> [1]
```
_Nested object_

---

## `leaves_with_paths`

**Signature:** `any -> array`

Get all leaf values with their JSON pointer paths

```
leaves_with_paths({a: 1}) -> [{path: \"/a\", value: 1}]
```
_Single leaf_

```
leaves_with_paths({a: {b: 1}}) -> [{path: \"/a/b\", value: 1}]
```
_Nested path_

---

## `mask`

**Signature:** `string, number? -> string`

Mask a string, showing only the last N characters

```
mask("4111111111111111") -> "************1111"
```
_Credit card default_

```
mask("555-123-4567", `3`) -> "*********567"
```
_Phone with 3 visible_

---

## `omit`

**Signature:** `object, array -> object`

Remove specific keys from object

```
omit({a: 1, b: 2}, ['a']) -> {b: 2}
```
_Remove one key_

```
omit({a: 1, b: 2, c: 3}, ['a', 'c']) -> {b: 2}
```
_Remove multiple keys_

---

## `paginate`

**Signature:** `array, number, number -> object`

Get a page of items from an array with metadata

```
paginate([1,2,3,4,5], `2`, `1`) -> {items: [1,2], page: 1, page_size: 2, total_items: 5, total_pages: 3, has_next: true, has_prev: false}
```
_First page_

```
paginate([1,2,3,4,5], `2`, `3`) -> {items: [5], page: 3, page_size: 2, total_items: 5, total_pages: 3, has_next: false, has_prev: true}
```
_Last page_

---

## `paths`

**Signature:** `any -> array`

List all JSON pointer paths in value

```
paths({a: {b: 1}}) -> [\"/a\", \"/a/b\"]
```
_Nested object_

```
paths({a: 1, b: 2}) -> [\"/a\", \"/b\"]
```
_Flat object_

---

## `paths_to`

**Signature:** `any, string -> array`

Find all dot-notation paths to a key anywhere in structure

```
paths_to({a: {id: 1}, b: {id: 2}}, "id") -> ["a.id", "b.id"]
```
_Find paths to id_

```
paths_to({users: [{id: 1}]}, "id") -> ["users.0.id"]
```
_Array paths_

---

## `pick`

**Signature:** `object, array -> object`

Select specific keys from object

```
pick({a: 1, b: 2}, ['a']) -> {a: 1}
```
_Pick one key_

```
pick({a: 1, b: 2, c: 3}, ['a', 'c']) -> {a: 1, c: 3}
```
_Pick multiple keys_

---

## `pluck_deep`

**Signature:** `any, string -> array`

Find all values for a key anywhere in nested structure

```
pluck_deep({users: [{id: 1}, {id: 2}], meta: {id: 99}}, "id") -> [1, 2, 99]
```
_Find all ids_

```
pluck_deep({a: {b: {c: 1}}, d: {c: 2}}, "c") -> [1, 2]
```
_Nested values_

---

## `redact`

**Signature:** `any, array -> any`

Recursively replace values at specified keys with [REDACTED]

```
redact({name: "alice", password: "secret"}, ["password"]) -> {name: "alice", password: "[REDACTED]"}
```
_Redact password_

```
redact({user: {name: "bob", ssn: "123"}}, ["ssn"]) -> {user: {name: "bob", ssn: "[REDACTED]"}}
```
_Nested redact_

---

## `redact_keys`

**Signature:** `any, string -> any`

Recursively redact keys matching a regex pattern

```
redact_keys({password: "x", api_key: "y"}, "password|api_key") -> {password: "[REDACTED]", api_key: "[REDACTED]"}
```
_Multiple patterns_

```
redact_keys({secret_key: "a", secret_token: "b"}, "secret.*") -> {secret_key: "[REDACTED]", secret_token: "[REDACTED]"}
```
_Wildcard pattern_

---

## `remove_empty`

**Signature:** `any -> any`

Recursively remove nulls, empty strings, empty arrays, and empty objects

```
remove_empty({a: \"\", b: [], c: {}, d: null, e: \"hello\"}) -> {e: \"hello\"}
```
_Remove all empty values_

```
remove_empty({a: {b: \"\", c: 1}}) -> {a: {c: 1}}
```
_Nested cleanup_

---

## `remove_empty_strings`

**Signature:** `any -> any`

Recursively remove empty string values

```
remove_empty_strings({name: \"alice\", bio: \"\"}) -> {name: \"alice\"}
```
_Remove empty strings_

```
remove_empty_strings([\"hello\", \"\", \"world\"]) -> [\"hello\", \"world\"]
```
_From arrays_

---

## `remove_nulls`

**Signature:** `any -> any`

Recursively remove null values

```
remove_nulls({a: 1, b: null, c: 2}) -> {a: 1, c: 2}
```
_Remove nulls from object_

```
remove_nulls({a: {b: null, c: 1}}) -> {a: {c: 1}}
```
_Nested nulls_

---

## `rename_keys`

**Signature:** `object, object -> object`

Rename object keys

```
rename_keys({a: 1}, {a: 'b'}) -> {b: 1}
```
_Rename one key_

```
rename_keys({a: 1, b: 2}, {a: 'x', b: 'y'}) -> {x: 1, y: 2}
```
_Rename multiple_

---

## `set_path`

**Signature:** `any, string, any -> any`

Set value at JSON pointer path (immutable)

```
set_path({a: 1}, '/b', `2`) -> {a: 1, b: 2}
```
_Add new key_

```
set_path({a: 1}, '/a', `2`) -> {a: 2}
```
_Update existing_

---

## `snake_keys`

**Signature:** `any -> any`

Recursively convert all keys to snake_case

```
snake_keys({userName: "alice"}) -> {user_name: "alice"}
```
_Camel to snake_

```
snake_keys({"user-name": "bob"}) -> {user_name: "bob"}
```
_Kebab to snake_

---

## `structural_diff`

**Signature:** `any, any -> object`

Compare two values and return their structural differences

```
structural_diff({a: 1}, {a: 2}) -> {changed: [{path: "a", from: 1, to: 2}], added: [], removed: []}
```
_Changed value_

```
structural_diff({a: 1}, {a: 1, b: 2}) -> {changed: [], added: [{path: "b", value: 2}], removed: []}
```
_Added key_

---

## `template`

**Signature:** `object, string -> string`

Expand a template string with values from an object using {{key}} syntax

```
template({name: "Alice"}, `"Hello, {{name}}!"`) -> "Hello, Alice!"
```
_Simple substitution_

```
template({user: {name: "Bob"}}, `"Hi {{user.name}}"`) -> "Hi Bob"
```
_Nested access_

---

## `template_strict`

**Signature:** `object, string -> string | null`

Expand a template string, returning null if any variable is missing

```
template_strict({name: "Alice"}, `"Hello, {{name}}!"`) -> "Hello, Alice!"
```
_All vars present_

```
template_strict({}, `"Hello, {{name}}!"`) -> null
```
_Missing variable_

---

## `truncate_to_size`

**Signature:** `array, number -> array`

Truncate an array to fit within approximately the specified byte size

```
truncate_to_size([{a:1}, {b:2}, {c:3}], `100`) -> [{a:1}, {b:2}, {c:3}]
```
_All fit_

```
truncate_to_size([{a:1}, {b:2}, {c:3}], `15`) -> [{a:1}, {b:2}]
```
_Truncated_

---

## `type_consistency`

**Signature:** `array -> object`

Check if array elements have consistent types

```
type_consistency([1, 2, 3]).consistent -> true
```
_Consistent numbers_

```
type_consistency([1, 'two', 3]).consistent -> false
```
_Mixed types_

---

## `unflatten`

**Signature:** `object -> object`

Alias for unflatten_keys - restore nested object from dot notation keys

```
unflatten({\"a.b\": 1}) -> {a: {b: 1}}
```
_Simple nested_

```
unflatten({\"a.b.c\": 1}) -> {a: {b: {c: 1}}}
```
_Deep nested_

---

## `unflatten_keys`

**Signature:** `object -> object`

Restore nested object from dot notation keys

```
unflatten_keys({\"a.b\": 1}) -> {a: {b: 1}}
```
_Simple nested_

```
unflatten_keys({\"a.b.c\": 1}) -> {a: {b: {c: 1}}}
```
_Deep nested_

---

## `with_entries`

**Signature:** `object, string -> object`

Transform object entries using an expression (jq parity)

```
with_entries({a: 1, b: 2}, '[upper(@[0]), multiply(@[1], `2`)]') -> {A: 2, B: 4}
```
_Transform keys and values_

```
with_entries({a: 1, b: 2}, '[@[0], add(@[1], `10`)]') -> {a: 11, b: 12}
```
_Transform values only_

---

