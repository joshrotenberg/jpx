# Standard Functions (26)

## `abs`

**Signature:** `number -> number`

Returns the absolute value of a number

```
abs(`-5`) -> 5
```
_Negative number_

```
abs(`5`) -> 5
```
_Positive number_

---

## `avg`

**Signature:** `array[number] -> number`

Returns the average of an array of numbers

```
avg([1, 2, 3]) -> 2
```
_Simple average_

```
avg([10, 20, 30, 40]) -> 25
```
_Four numbers_

---

## `ceil`

**Signature:** `number -> number`

Returns the smallest integer greater than or equal to the number

```
ceil(`1.5`) -> 2
```
_Round up decimal_

```
ceil(`-1.5`) -> -1
```
_Negative rounds toward zero_

---

## `contains`

**Signature:** `array|string, any -> boolean`

Returns true if the subject contains the search value

```
contains([1, 2, 3], `2`) -> true
```
_Array contains number_

```
contains('hello', 'ell') -> true
```
_String contains substring_

---

## `ends_with`

**Signature:** `string, string -> boolean`

Returns true if the subject ends with the suffix

```
ends_with('hello', 'lo') -> true
```
_Ends with suffix_

```
ends_with('hello', 'he') -> false
```
_Does not end with_

---

## `floor`

**Signature:** `number -> number`

Returns the largest integer less than or equal to the number

```
floor(`1.9`) -> 1
```
_Round down decimal_

```
floor(`-1.5`) -> -2
```
_Negative rounds away from zero_

---

## `join`

**Signature:** `string, array[string] -> string`

Returns array elements joined into a string with a separator

```
join(', ', ['a', 'b', 'c']) -> \"a, b, c\"
```
_Join with comma_

```
join('-', ['2024', '01', '15']) -> \"2024-01-15\"
```
_Join date parts_

---

## `keys`

**Signature:** `object -> array[string]`

Returns an array of keys from an object

```
keys({a: 1, b: 2}) -> [\"a\", \"b\"]
```
_Get object keys_

```
keys({name: \"John\", age: 30}) -> [\"age\", \"name\"]
```
_Keys sorted alphabetically_

---

## `length`

**Signature:** `array|object|string -> number`

Returns the length of an array, object, or string

```
length([1, 2, 3]) -> 3
```
_Array length_

```
length('hello') -> 5
```
_String length_

---

## `map`

**Signature:** `expression, array -> array`

Applies an expression to each element of an array

```
map(&a, [{a: 1}, {a: 2}]) -> [1, 2]
```
_Extract field from objects_

```
map(&length(@), ['a', 'bb', 'ccc']) -> [1, 2, 3]
```
_Apply function_

---

## `max`

**Signature:** `array[number]|array[string] -> number|string`

Returns the maximum value in an array

```
max([1, 3, 2]) -> 3
```
_Max of numbers_

```
max(['a', 'c', 'b']) -> 'c'
```
_Max of strings_

---

## `max_by`

**Signature:** `array, expression -> any`

Returns the element with maximum value by expression

```
max_by([{a: 1}, {a: 2}], &a) -> {a: 2}
```
_Max by field_

```
max_by([{name: 'a', age: 30}, {name: 'b', age: 25}], &age) -> {name: 'a', age: 30}
```
_Max by age_

---

## `merge`

**Signature:** `object... -> object`

Merges objects into a single object

```
merge({a: 1}, {b: 2}) -> {a: 1, b: 2}
```
_Merge two objects_

```
merge({a: 1}, {a: 2}) -> {a: 2}
```
_Later values override_

---

## `min`

**Signature:** `array[number]|array[string] -> number|string`

Returns the minimum value in an array

```
min([1, 3, 2]) -> 1
```
_Min of numbers_

```
min(['a', 'c', 'b']) -> 'a'
```
_Min of strings_

---

## `min_by`

**Signature:** `array, expression -> any`

Returns the element with minimum value by expression

```
min_by([{a: 1}, {a: 2}], &a) -> {a: 1}
```
_Min by field_

```
min_by([{name: 'a', age: 30}, {name: 'b', age: 25}], &age) -> {name: 'b', age: 25}
```
_Min by age_

---

## `not_null`

**Signature:** `any... -> any`

Returns the first non-null argument

```
not_null(`null`, 'a', 'b') -> \"a\"
```
_Skip nulls_

```
not_null('first', 'second') -> \"first\"
```
_First non-null_

---

## `reverse`

**Signature:** `array|string -> array|string`

Reverses an array or string

```
reverse([1, 2, 3]) -> [3, 2, 1]
```
_Reverse array_

```
reverse('hello') -> 'olleh'
```
_Reverse string_

---

## `sort`

**Signature:** `array[number]|array[string] -> array`

Sorts an array of numbers or strings

```
sort([3, 1, 2]) -> [1, 2, 3]
```
_Sort numbers_

```
sort(['c', 'a', 'b']) -> ['a', 'b', 'c']
```
_Sort strings_

---

## `sort_by`

**Signature:** `array, expression -> array`

Sorts an array by expression result

```
sort_by([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]
```
_Sort by field_

```
sort_by(['bb', 'a', 'ccc'], &length(@)) -> ['a', 'bb', 'ccc']
```
_Sort by length_

---

## `starts_with`

**Signature:** `string, string -> boolean`

Returns true if the subject starts with the prefix

```
starts_with('hello', 'he') -> true
```
_Starts with prefix_

```
starts_with('hello', 'lo') -> false
```
_Does not start with_

---

## `sum`

**Signature:** `array[number] -> number`

Returns the sum of an array of numbers

```
sum([1, 2, 3]) -> 6
```
_Sum of numbers_

```
sum([10, -5, 3]) -> 8
```
_With negative_

---

## `to_array`

**Signature:** `any -> array`

Converts a value to an array

```
to_array('hello') -> [\"hello\"]
```
_String to array_

```
to_array([1, 2]) -> [1, 2]
```
_Array unchanged_

---

## `to_number`

**Signature:** `any -> number`

Converts a value to a number

```
to_number('42') -> 42
```
_String to number_

```
to_number('3.14') -> 3.14
```
_String to float_

---

## `to_string`

**Signature:** `any -> string`

Converts a value to a string

```
to_string(`42`) -> \"42\"
```
_Number to string_

```
to_string(`true`) -> \"true\"
```
_Boolean to string_

---

## `type`

**Signature:** `any -> string`

Returns the type of a value as a string

```
type('hello') -> \"string\"
```
_String type_

```
type(`42`) -> \"number\"
```
_Number type_

---

## `values`

**Signature:** `object -> array`

Returns an array of values from an object

```
values({a: 1, b: 2}) -> [1, 2]
```
_Get object values_

```
values({x: 'hello', y: 'world'}) -> ['hello', 'world']
```
_String values_

---

