# Utility Functions (11)

## `coalesce`

**Signature:** `any... -> any`

Return first non-null value

```
coalesce(`null`, `null`, 'value') -> \"value\"
```
_Skip nulls_

```
coalesce(field1, field2, 'default') -> first non-null
```
_Field fallback_

---

## `default`

**Signature:** `any, any -> any`

Return default value if null

```
default(`null`, 'fallback') -> \"fallback\"
```
_Null uses default_

```
default('value', 'fallback') -> 'value'
```
_Non-null keeps value_

---

## `env`

**Signature:** `-> object`

Get all environment variables as an object

```
env() -> {HOME: '/Users/...', PATH: '...', ...}
```
_All env vars_

```
env().HOME -> home directory
```
_Access specific var_

---

## `get_env`

**Signature:** `string -> string | null`

Get a single environment variable by name

```
get_env('HOME') -> \"/Users/josh\"
```
_Get home directory_

```
get_env('PATH') -> system PATH
```
_Get PATH_

---

## `if`

**Signature:** `boolean, any, any -> any`

Conditional expression

```
if(`true`, 'yes', 'no') -> \"yes\"
```
_True branch_

```
if(`false`, 'yes', 'no') -> \"no\"
```
_False branch_

---

## `json_decode`

**Signature:** `string -> any`

Parse JSON string

```
json_decode('{\"a\": 1}') -> {a: 1}
```
_Parse object_

```
json_decode('[1, 2, 3]') -> [1, 2, 3]
```
_Parse array_

---

## `json_encode`

**Signature:** `any -> string`

Serialize value to JSON string

```
json_encode({a: 1}) -> \"{\\\"a\\\":1}\"
```
_Encode object_

```
json_encode([1, 2, 3]) -> \"[1,2,3]\"
```
_Encode array_

---

## `json_pointer`

**Signature:** `any, string -> any`

Access value using JSON Pointer (RFC 6901)

```
json_pointer({foo: {bar: 1}}, '/foo/bar') -> 1
```
_Nested access_

```
json_pointer(data, '/0/name') -> first item name
```
_Array access_

---

## `now`

**Signature:** `-> number`

Current Unix timestamp in seconds

```
now() -> 1699900000
```
_Current timestamp_

```
now() - 3600 -> one hour ago
```
_Subtract seconds_

---

## `now_ms`

**Signature:** `-> number`

Current Unix timestamp in milliseconds

```
now_ms() -> 1699900000000
```
_Current time in ms_

```
now_ms() - start_ms -> elapsed ms
```
_Calculate duration_

---

## `pretty`

**Signature:** `any, number? -> string`

Pretty-print value as formatted JSON string

```
pretty({a: 1}) -> \"{\n  \\\"a\\\": 1\n}\"
```
_Default 2-space indent_

```
pretty({a: 1}, `4`) -> 4-space indent
```
_Custom indent_

---

