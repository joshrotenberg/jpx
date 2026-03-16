# Expression Functions (41)

## `all_expr`

**Signature:** `array, expression -> boolean`

Return true if every element satisfies the expression (short-circuits on false)

```
all_expr([1, 2, 3], &@ > `0`) -> true
```
_All positive_

```
all_expr([1, 2, 3], &@ > `2`) -> false
```
_Not all > 2_

---

## `any_expr`

**Signature:** `array, expression -> boolean`

Return true if any element satisfies the expression (short-circuits)

```
any_expr([1, 2, 3], &@ > `2`) -> true
```
_At least one > 2_

```
any_expr([1, 2, 3], &@ > `5`) -> false
```
_None > 5_

---

## `apply`

**Signature:** `object|string, ...any -> any`

Apply a partial function or invoke a function by name with arguments

```
apply(partial('join', `\"-\"`), `[\"a\", \"b\"]`) -> 'a-b' 
```
_Apply partial function_

```
apply('length', 'hello') -> 5
```
_Call function by name_

---

## `count_by`

**Signature:** `expref, array -> object`

Count occurrences grouped by expression result

```
count_by(&type, [{type: 'a'}, {type: 'b'}, {type: 'a'}]) -> {a: 2, b: 1}
```
_Count by field_

```
count_by(&status, orders) -> {pending: 5, shipped: 3}
```
_Count orders by status_

---

## `count_expr`

**Signature:** `array, expression -> number`

Count how many elements satisfy the expression

```
count_expr([1, 2, 3], &@ > `1`) -> 2
```
_Count > 1_

```
count_expr([1, 2, 3], &@ > `5`) -> 0
```
_None match_

---

## `dense_rank`

**Signature:** `expref, array -> array`

Assign ranks without gaps for ties (1, 2, 2, 3)

```
dense_rank(&score, [{"score":90},{"score":85},{"score":90}]) -> [1, 2, 1]
```
_Dense rank without gaps_

```
dense_rank(&@, [3, 1, 2, 1]) -> [1, 3, 2, 3]
```
_Dense rank numbers_

---

## `drop_while`

**Signature:** `expref, array -> array`

Drop elements from array while expression is truthy

```
drop_while(&@ < `4`, [1, 2, 3, 5, 1]) -> [5, 1]
```
_Drop while < 4_

```
drop_while(&@ < `0`, [1, 2, 3]) -> [1, 2, 3]
```
_None dropped_

---

## `every`

**Signature:** `expref, array -> boolean`

Check if all elements match (alias for all_expr)

```
every(&@ > `0`, [1, 2, 3]) -> true
```
_All positive_

```
every(&@ > `2`, [1, 2, 3]) -> false
```
_Not all > 2_

---

## `filter_expr`

**Signature:** `array, expression -> array`

Keep elements where JMESPath expression evaluates to truthy value

```
filter_expr([1, 2, 3], &@ > `1`) -> [2, 3]
```
_Filter numbers_

```
filter_expr(users, &age >= `18`) -> [adult users]
```
_Filter objects by field_

---

## `find_expr`

**Signature:** `array, expression -> any`

Return first element where expression is truthy, or null if none match

```
find_expr([1, 2, 3], &@ > `1`) -> 2
```
_First > 1_

```
find_expr([1, 2, 3], &@ > `5`) -> null
```
_None found_

---

## `find_index_expr`

**Signature:** `array, expression -> number | null`

Return zero-based index of first matching element, or -1 if none match

```
find_index_expr([1, 2, 3], &@ > `1`) -> 1
```
_Index of first > 1_

```
find_index_expr([1, 2, 3], &@ > `5`) -> -1
```
_Not found_

---

## `flat_map_expr`

**Signature:** `array, expression -> array`

Apply expression to each element and flatten all results into one array

```
flat_map_expr([[1], [2]], &@) -> [1, 2]
```
_Flatten nested arrays_

```
flat_map_expr([1, 2], &[@, @ * `2`]) -> [1, 2, 2, 4]
```
_Duplicate and transform_

---

## `fold`

**Signature:** `expref, array, any -> any`

Alias for reduce_expr - reduce array to single value using accumulator expression

```
fold(&add(accumulator, current), [1, 2, 3], `0`) -> 6
```
_Sum numbers_

---

## `group_by_expr`

**Signature:** `expression, array -> object`

Group elements into object keyed by expression result (legacy alias, prefer group_by(array, &expr))

```
group_by_expr(&t, [{t: 'a'}, {t: 'b'}]) -> {a: [...], b: [...]}
```
_Group by field_

```
group_by_expr(&@ > `2`, [1, 2, 3, 4]) -> {true: [3, 4], false: [1, 2]}
```
_Group by condition_

---

## `map_expr`

**Signature:** `array, expression -> array`

Apply a JMESPath expression to each element, returning transformed array

```
map_expr([1, 2], &@ * `2`) -> [2, 4]
```
_Double each number_

```
map_expr(users, &name) -> ['alice', 'bob']
```
_Extract field from objects_

---

## `map_keys`

**Signature:** `expref, object -> object`

Transform object keys using expression

```
map_keys(&upper(@), {a: 1}) -> {A: 1}
```
_Uppercase keys_

```
map_keys(&lower(@), {A: 1, B: 2}) -> {a: 1, b: 2}
```
_Lowercase keys_

---

## `map_values`

**Signature:** `expref, object -> object`

Transform object values using expression

```
map_values(&@ * `2`, {a: 1, b: 2}) -> {a: 2, b: 4}
```
_Double values_

```
map_values(&upper(@), {a: 'x', b: 'y'}) -> {a: 'X', b: 'Y'}
```
_Uppercase strings_

---

## `mapcat`

**Signature:** `expref, array -> array`

Apply expression to each element and concatenate all results (Clojure-style alias for flat_map_expr)

```
mapcat(&tags, [{tags: ['a', 'b']}, {tags: ['c']}]) -> ['a', 'b', 'c']
```
_Flatten tags from objects_

```
mapcat(&@, [[1, 2], [3, 4]]) -> [1, 2, 3, 4]
```
_Flatten nested arrays_

---

## `max_by_expr`

**Signature:** `array, expression -> any`

Return element with largest expression result, or null for empty array

```
max_by_expr([{a: 2}, {a: 1}], &a) -> {a: 2}
```
_Max by field_

```
max_by_expr(['a', 'abc', 'ab'], &length(@)) -> 'abc'
```
_Longest string_

---

## `min_by_expr`

**Signature:** `array, expression -> any`

Return element with smallest expression result, or null for empty array

```
min_by_expr([{a: 2}, {a: 1}], &a) -> {a: 1}
```
_Min by field_

```
min_by_expr(['a', 'abc', 'ab'], &length(@)) -> 'a'
```
_Shortest string_

---

## `none`

**Signature:** `expref, array -> boolean`

Return true if no elements satisfy the expression (opposite of any_expr/some)

```
none(&@ > `5`, [1, 2, 3]) -> true
```
_No elements > 5_

```
none(&@ > `5`, [1, 2, 10]) -> false
```
_10 is > 5_

---

## `order_by`

**Signature:** `array, array[[string, string]] -> array`

Sort array by multiple fields with ascending/descending control

```
order_by(items, [['name', 'asc'], ['price', 'desc']]) -> sorted
```
_Multi-field sort_

```
order_by(users, [['age', 'asc']]) -> youngest first
```
_Single field ascending_

---

## `partial`

**Signature:** `string, ...any -> object`

Create a partial function with some arguments pre-filled

```
partial('contains', `\"hello\"`) -> {__partial__: true, ...}
```
_Partial contains_

```
partial('add', `10`) -> partial add 10
```
_Partial addition_

---

## `partition_expr`

**Signature:** `array, expression -> array`

Split array into [matches, non-matches] based on expression

```
partition_expr([1, 2, 3], &@ > `1`) -> [[2, 3], [1]]
```
_Split by condition_

```
partition_expr([1, 2, 3, 4], &@ % `2` == `0`) -> [[2, 4], [1, 3]]
```
_Even vs odd_

---

## `pivot`

**Signature:** `array, expression, expression -> object`

Transform array of objects into object using key/value exprefs

```
pivot([{"name":"a","v":1},{"name":"b","v":2}], &name, &v) -> {"a": 1, "b": 2}
```
_Pivot by name_

```
pivot([{"k":"x","v":10},{"k":"y","v":20}], &k, &v) -> {"x": 10, "y": 20}
```
_Key-value pairs to object_

---

## `rank`

**Signature:** `expref, array -> array`

Assign ranks with gaps for ties (1, 2, 2, 4)

```
rank(&score, [{"score":90},{"score":85},{"score":90}]) -> [1, 3, 1]
```
_Rank with gaps_

```
rank(&@, [3, 1, 2, 1]) -> [1, 3, 2, 3]
```
_Rank numbers_

---

## `recurse`

**Signature:** `any -> array`

Collect all nested values recursively (jq parity)

```
recurse({a: {b: 1}}) -> [{a: {b: 1}}, {b: 1}, 1]
```
_Nested object_

```
recurse([1, [2, 3]]) -> [[1, [2, 3]], 1, [2, 3], 2, 3]
```
_Nested array_

---

## `recurse_with`

**Signature:** `any, expression -> array`

Recursive descent with expression filter (jq parity)

```
recurse_with({a: {a: 1}}, &a) -> [{a: 1}, 1]
```
_Follow 'a' key recursively_

```
recurse_with([1, [2, [3]]], &[1]) -> [[2, [3]], [3]]
```
_Follow index recursively_

---

## `reduce_expr`

**Signature:** `expref, array, any -> any`

Reduce array to single value using accumulator expression

```
reduce_expr(&add(accumulator, current), [1, 2, 3], `0`) -> 6
```
_Sum numbers_

```
reduce_expr(&multiply(accumulator, current), [2, 3, 4], `1`) -> 24
```
_Product_

---

## `reductions`

**Signature:** `expref, array, any -> array`

Return array of intermediate values from reduction (Clojure-style alias for scan_expr)

```
reductions(&sum([accumulator, current]), [1, 2, 3, 4], `0`) -> [1, 3, 6, 10]
```
_Running sum_

```
reductions(&max([accumulator, current]), [3, 1, 4, 1, 5], `0`) -> [3, 3, 4, 4, 5]
```
_Running max_

---

## `reject`

**Signature:** `expref, array -> array`

Keep elements where expression is falsy (inverse of filter_expr)

```
reject(&@ > `2`, [1, 2, 3, 4]) -> [1, 2]
```
_Reject > 2_

```
reject(&@ < `0`, [1, -1, 2, -2]) -> [1, 2]
```
_Reject negatives_

---

## `scan_expr`

**Signature:** `expref, array, any -> array`

Like reduce but returns array of intermediate accumulator values

```
scan_expr(&add(accumulator, current), [1, 2, 3], `0`) -> [1, 3, 6]
```
_Running sum_

```
scan_expr(&multiply(accumulator, current), [2, 3, 4], `1`) -> [2, 6, 24]
```
_Running product_

---

## `some`

**Signature:** `expref, array -> boolean`

Check if any element matches (alias for any_expr)

```
some(&@ > `2`, [1, 2, 3]) -> true
```
_Some > 2_

```
some(&@ > `5`, [1, 2, 3]) -> false
```
_None > 5_

---

## `sort_by_expr`

**Signature:** `array, expression -> array`

Sort array by expression result in ascending order

```
sort_by_expr([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]
```
_Sort by field_

```
sort_by_expr(['bb', 'a', 'ccc'], &length(@)) -> ['a', 'bb', 'ccc']
```
_Sort by length_

---

## `take_while`

**Signature:** `expref, array -> array`

Take elements from array while expression is truthy

```
take_while(&@ < `4`, [1, 2, 3, 5, 1]) -> [1, 2, 3]
```
_Take while < 4_

```
take_while(&@ > `0`, [3, 2, 1, 0, 5]) -> [3, 2, 1]
```
_Take while positive_

---

## `unique_by_expr`

**Signature:** `array, expression -> array`

Remove duplicates by expression result, keeping first occurrence

```
unique_by_expr([{a: 1}, {a: 1}], &a) -> [{a: 1}]
```
_Unique by field_

```
unique_by_expr([{id: 1, v: 'a'}, {id: 1, v: 'b'}], &id) -> [{id: 1, v: 'a'}]
```
_First wins_

---

## `unpivot`

**Signature:** `object -> array`

Transform object into array of {key, value} pairs (inverse of pivot)

```
unpivot({"a": 1, "b": 2}) -> [{"key": "a", "value": 1}, {"key": "b", "value": 2}]
```
_Unpivot object_

```
unpivot({"x": "hello"}) -> [{"key": "x", "value": "hello"}]
```
_Single entry_

---

## `until_expr`

**Signature:** `any, expression, expression -> array`

Loop until condition becomes true, collecting intermediate values (jq parity)

```
until_expr(`1`, &@ >= `5`, &add(@, `1`)) -> [1, 2, 3, 4, 5]
```
_Count until >= 5_

```
until_expr(`2`, &@ >= `100`, &multiply(@, `2`)) -> [2, 4, 8, 16, 32, 64, 128]
```
_Double until >= 100_

---

## `walk`

**Signature:** `expref, any -> any`

Recursively apply expression to all components of a value (bottom-up)

```
walk(&@, data) -> data unchanged
```
_Identity transform_

```
walk(&type(@), {a: 1}) -> 'object'
```
_Get type of result_

---

## `while_expr`

**Signature:** `any, expression, expression -> array`

Loop while condition is true, collecting intermediate values (jq parity)

```
while_expr(`1`, &@ < `5`, &add(@, `1`)) -> [1, 2, 3, 4]
```
_Count from 1 while < 5_

```
while_expr(`2`, &@ < `100`, &multiply(@, `2`)) -> [2, 4, 8, 16, 32, 64]
```
_Double while < 100_

---

## `zip_with`

**Signature:** `expref, array, array -> array`

Zip two arrays with a custom combiner expression

```
zip_with(&add([0], [1]), [1, 2], [10, 20]) -> [11, 22]
```
_Add pairs_

```
zip_with(&multiply([0], [1]), [2, 3], [4, 5]) -> [8, 15]
```
_Multiply pairs_

---

