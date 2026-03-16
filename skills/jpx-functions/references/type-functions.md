# Type Functions (13)

## `auto_parse`

**Signature:** `any -> any`

Intelligently parse strings to numbers, booleans, and nulls

```
auto_parse({num: \"42\", bool: \"true\", nil: \"null\"}) -> {num: 42, bool: true, nil: null}
```
_Parse mixed types_

```
auto_parse([\"42\", \"true\", \"hello\"]) -> [42, true, \"hello\"]
```
_Parse array_

---

## `is_array`

**Signature:** `any -> boolean`

Check if value is an array

```
is_array([1, 2]) -> true
```
_Array returns true_

```
is_array('hello') -> false
```
_String returns false_

---

## `is_boolean`

**Signature:** `any -> boolean`

Check if value is a boolean

```
is_boolean(`true`) -> true
```
_True is boolean_

```
is_boolean(`false`) -> true
```
_False is boolean_

---

## `is_empty`

**Signature:** `any -> boolean`

Check if value is empty

```
is_empty([]) -> true
```
_Empty array_

```
is_empty('') -> true
```
_Empty string_

---

## `is_null`

**Signature:** `any -> boolean`

Check if value is null

```
is_null(`null`) -> true
```
_Null returns true_

```
is_null('') -> false
```
_Empty string is not null_

---

## `is_number`

**Signature:** `any -> boolean`

Check if value is a number

```
is_number(`42`) -> true
```
_Integer is number_

```
is_number(`3.14`) -> true
```
_Float is number_

---

## `is_object`

**Signature:** `any -> boolean`

Check if value is an object

```
is_object({a: 1}) -> true
```
_Object returns true_

```
is_object({}) -> true
```
_Empty object is object_

---

## `is_string`

**Signature:** `any -> boolean`

Check if value is a string

```
is_string('hello') -> true
```
_String returns true_

```
is_string('') -> true
```
_Empty string is string_

---

## `parse_booleans`

**Signature:** `any -> any`

Recursively convert boolean strings to booleans

```
parse_booleans({active: "true"}) -> {active: true}
```
_Parse true_

```
parse_booleans({flag: "YES"}) -> {flag: true}
```
_Parse yes_

---

## `parse_nulls`

**Signature:** `any -> any`

Recursively convert null-like strings to null

```
parse_nulls({a: "null"}) -> {a: null}
```
_Parse null_

```
parse_nulls({a: "None"}) -> {a: null}
```
_Parse None_

---

## `parse_numbers`

**Signature:** `any -> any`

Recursively convert numeric strings to numbers

```
parse_numbers({count: "42"}) -> {count: 42}
```
_Parse integer_

```
parse_numbers({price: "19.99"}) -> {price: 19.99}
```
_Parse float_

---

## `to_boolean`

**Signature:** `any -> boolean`

Convert value to boolean

```
to_boolean('true') -> true
```
_String 'true'_

```
to_boolean('false') -> false
```
_String 'false'_

---

## `type_of`

**Signature:** `any -> string`

Get the type of a value

```
type_of(`42`) -> \"number\"
```
_Number type_

```
type_of('hello') -> \"string\"
```
_String type_

---

