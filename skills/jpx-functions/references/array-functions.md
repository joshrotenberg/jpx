# Array Functions (36)

## `bsearch`

**Signature:** `array, any -> number`

Binary search in a sorted array, returns index or negative insertion point (jq parity)

```
bsearch([1, 3, 5, 7, 9], `5`) -> 2
```
_Found at index 2_

```
bsearch([1, 3, 5, 7, 9], `4`) -> -3
```
_Not found, would insert at index 2_

---

## `butlast`

**Signature:** `array -> array`

Return all elements except the last (alias for initial)

```
butlast([1, 2, 3, 4]) -> [1, 2, 3]
```
_Remove last element_

```
butlast(['a', 'b', 'c']) -> ['a', 'b']
```
_String values_

---

## `cartesian`

**Signature:** `array, array? -> array`

Compute cartesian product of arrays (jq parity for N-way product)

```
cartesian([1, 2], [3, 4]) -> [[1, 3], [1, 4], [2, 3], [2, 4]]
```
_Two-array product_

```
cartesian([[1, 2], [3, 4]]) -> [[1, 3], [1, 4], [2, 3], [2, 4]]
```
_N-way product (jq style)_

---

## `chunk`

**Signature:** `array, number -> array`

Split array into chunks of size n

```
chunk([1, 2, 3, 4], `2`) -> [[1, 2], [3, 4]]
```
_Basic chunking_

```
chunk([1, 2, 3, 4, 5], `2`) -> [[1, 2], [3, 4], [5]]
```
_Uneven chunks_

---

## `compact`

**Signature:** `array -> array`

Remove null values from array

```
compact([1, null, 2, null]) -> [1, 2]
```
_Remove nulls_

```
compact([null, null]) -> []
```
_All nulls_

---

## `cycle`

**Signature:** `array, number -> array`

Cycle through array elements n times

```
cycle([1, 2, 3], `2`) -> [1, 2, 3, 1, 2, 3]
```
_Cycle twice_

```
cycle(["a", "b"], `3`) -> ["a", "b", "a", "b", "a", "b"]
```
_Cycle strings_

---

## `dedupe`

**Signature:** `array -> array`

Remove consecutive duplicate values (unlike unique, allows non-adjacent duplicates)

```
dedupe([1, 1, 2, 2, 1, 1]) -> [1, 2, 1]
```
_Remove adjacent duplicates_

```
dedupe(['a', 'a', 'b', 'a']) -> ['a', 'b', 'a']
```
_String values_

---

## `difference`

**Signature:** `array, array -> array`

Elements in first array not in second

```
difference([1, 2, 3], [2]) -> [1, 3]
```
_Remove matching elements_

```
difference([1, 2, 3], [4, 5]) -> [1, 2, 3]
```
_No overlap_

---

## `drop`

**Signature:** `array, number -> array`

Drop first n elements

```
drop([1, 2, 3, 4], `2`) -> [3, 4]
```
_Drop first 2_

```
drop([1, 2, 3], `0`) -> [1, 2, 3]
```
_Drop none_

---

## `find_index`

**Signature:** `array, any -> number | null`

Find index of value in array

```
find_index([1, 2, 3], `2`) -> 1
```
_Find existing value_

```
find_index(['a', 'b', 'c'], 'b') -> 1
```
_Find string_

---

## `first`

**Signature:** `array -> any`

Get first element of array

```
first([1, 2, 3]) -> 1
```
_Get first number_

```
first(['a', 'b']) -> 'a'
```
_Get first string_

---

## `flatten`

**Signature:** `array -> array`

Flatten array one level deep

```
flatten([[1, 2], [3]]) -> [1, 2, 3]
```
_Flatten nested arrays_

```
flatten([[1, [2]], [3]]) -> [1, [2], 3]
```
_Only one level deep_

---

## `flatten_deep`

**Signature:** `array -> array`

Recursively flatten nested arrays

```
flatten_deep([[1, [2]], [3]]) -> [1, 2, 3]
```
_Deeply nested_

```
flatten_deep([[[1]], [[2]], [[3]]]) -> [1, 2, 3]
```
_Multiple levels_

---

## `frequencies`

**Signature:** `array -> object`

Count occurrences of each value

```
frequencies(['a', 'b', 'a']) -> {a: 2, b: 1}
```
_Count strings_

```
frequencies([1, 2, 1, 1]) -> {1: 3, 2: 1}
```
_Count numbers_

---

## `group_by`

**Signature:** `array, expression|string -> object`

Group array elements by expression or field name

```
group_by([{t: 'a'}, {t: 'b'}, {t: 'a'}], &t) -> {a: [{t:'a'}, {t:'a'}], b: [{t:'b'}]}
```
_Group by field (expref)_

```
group_by([{t: 'a'}, {t: 'b'}, {t: 'a'}], 't') -> {a: [{t:'a'}, {t:'a'}], b: [{t:'b'}]}
```
_Group by field (string, legacy)_

---

## `includes`

**Signature:** `array, any -> boolean`

Check if array contains value

```
includes([1, 2, 3], `2`) -> true
```
_Value found_

```
includes([1, 2, 3], `99`) -> false
```
_Value not found_

---

## `index_at`

**Signature:** `array, number -> any`

Get element at index (supports negative)

```
index_at([1, 2, 3], `0`) -> 1
```
_First element_

```
index_at([1, 2, 3], `-1`) -> 3
```
_Last element (negative index)_

---

## `index_by`

**Signature:** `array, expression|string -> object`

Create lookup map from array using expression or key field (last value wins for duplicates)

```
index_by([{id: 1, name: "alice"}, {id: 2, name: "bob"}], &id) -> {"1": {id: 1, name: "alice"}, "2": {id: 2, name: "bob"}}
```
_Index by id (expref)_

```
index_by([{id: 1, name: "alice"}, {id: 2, name: "bob"}], "id") -> {"1": {id: 1, name: "alice"}, "2": {id: 2, name: "bob"}}
```
_Index by id (string, legacy)_

---

## `indices_array`

**Signature:** `array, any -> array`

Find all indices where a value appears in an array (jq parity)

```
indices_array([1, 2, 3, 2, 4, 2], `2`) -> [1, 3, 5]
```
_Find all occurrences_

```
indices_array(['a', 'b', 'a', 'c'], `'a'`) -> [0, 2]
```
_String values_

---

## `inside_array`

**Signature:** `array, array -> boolean`

Check if all elements of first array are contained in second array (inverse of contains, jq parity)

```
inside_array([1, 2], [1, 2, 3, 4]) -> true
```
_Subset check_

```
inside_array([1, 5], [1, 2, 3, 4]) -> false
```
_Not a subset_

---

## `interpose`

**Signature:** `array, any -> array`

Insert separator value between each element of array

```
interpose([1, 2, 3], `0`) -> [1, 0, 2, 0, 3]
```
_Insert zeros between numbers_

```
interpose(['a', 'b', 'c'], `"-"`) -> ['a', '-', 'b', '-', 'c']
```
_Insert separator strings_

---

## `intersection`

**Signature:** `array, array -> array`

Elements common to both arrays

```
intersection([1, 2], [2, 3]) -> [2]
```
_Common elements_

```
intersection([1, 2], [3, 4]) -> []
```
_No overlap_

---

## `lag`

**Signature:** `array, number -> array`

Shift array by n positions forward, prepending nulls

```
lag([1, 2, 3], `1`) -> [null, 1, 2]
```
_Lag by 1_

```
lag([1, 2, 3], `2`) -> [null, null, 1]
```
_Lag by 2_

---

## `last`

**Signature:** `array -> any`

Get last element of array

```
last([1, 2, 3]) -> 3
```
_Get last number_

```
last(['a', 'b']) -> 'b'
```
_Get last string_

---

## `lead`

**Signature:** `array, number -> array`

Shift array by n positions backward, appending nulls

```
lead([1, 2, 3], `1`) -> [2, 3, null]
```
_Lead by 1_

```
lead([1, 2, 3], `2`) -> [3, null, null]
```
_Lead by 2_

---

## `pairwise`

**Signature:** `array -> array`

Return adjacent pairs from array

```
pairwise([1, 2, 3]) -> [[1, 2], [2, 3]]
```
_Adjacent pairs_

```
pairwise([1, 2]) -> [[1, 2]]
```
_Single pair_

---

## `partition_by`

**Signature:** `array, expression|string -> array`

Split array into partitions when expression or field value changes (preserves order unlike group_by)

```
partition_by([{t: "a"}, {t: "a"}, {t: "b"}, {t: "a"}], &t) -> [[{t: "a"}, {t: "a"}], [{t: "b"}], [{t: "a"}]]
```
_Split on field change (expref)_

```
partition_by([{t: "a"}, {t: "a"}, {t: "b"}, {t: "a"}], "t") -> [[{t: "a"}, {t: "a"}], [{t: "b"}], [{t: "a"}]]
```
_Split on field change (string, legacy)_

---

## `range`

**Signature:** `number, number -> array`

Generate array of numbers

```
range(`1`, `5`) -> [1, 2, 3, 4]
```
_Range 1 to 4_

```
range(`0`, `3`) -> [0, 1, 2]
```
_Range from zero_

---

## `repeat_array`

**Signature:** `any, number -> array`

Create array with value repeated n times

```
repeat_array(`1`, `3`) -> [1, 1, 1]
```
_Repeat number_

```
repeat_array(`"x"`, `4`) -> ["x", "x", "x", "x"]
```
_Repeat string_

---

## `sliding_window`

**Signature:** `array, number -> array`

Create overlapping windows of size n (alias for window)

```
sliding_window([1, 2, 3, 4], `2`) -> [[1, 2], [2, 3], [3, 4]]
```
_Size 2 windows_

```
sliding_window([1, 2, 3, 4], `3`) -> [[1, 2, 3], [2, 3, 4]]
```
_Size 3 windows_

---

## `take`

**Signature:** `array, number -> array`

Take first n elements

```
take([1, 2, 3, 4], `2`) -> [1, 2]
```
_Take first 2_

```
take([1, 2, 3], `0`) -> []
```
_Take none_

---

## `transpose`

**Signature:** `array -> array`

Transpose a 2D array (swap rows and columns)

```
transpose([[1, 2], [3, 4]]) -> [[1, 3], [2, 4]]
```
_Swap rows/columns_

```
transpose([[1, 2, 3], [4, 5, 6]]) -> [[1, 4], [2, 5], [3, 6]]
```
_2x3 to 3x2_

---

## `union`

**Signature:** `array, array -> array`

Unique elements from both arrays

```
union([1, 2], [2, 3]) -> [1, 2, 3]
```
_Combine with dedup_

```
union([1, 2], [3, 4]) -> [1, 2, 3, 4]
```
_No overlap_

---

## `unique`

**Signature:** `array -> array`

Remove duplicate values

```
unique([1, 2, 1, 3]) -> [1, 2, 3]
```
_Basic deduplication_

```
unique(['a', 'b', 'a']) -> ['a', 'b']
```
_String values_

---

## `zip`

**Signature:** `array, array -> array`

Zip two arrays together

```
zip([1, 2], ['a', 'b']) -> [[1, 'a'], [2, 'b']]
```
_Basic zip_

```
zip([1, 2, 3], ['a', 'b']) -> [[1, 'a'], [2, 'b']]
```
_Unequal lengths (truncates)_

---

## `zipmap`

**Signature:** `array, array -> object`

Create object from parallel arrays of keys and values

```
zipmap(["a", "b", "c"], [1, 2, 3]) -> {"a": 1, "b": 2, "c": 3}
```
_Basic zipmap_

```
zipmap(["x", "y"], [10, 20, 30]) -> {"x": 10, "y": 20}
```
_Uses shorter length_

---

