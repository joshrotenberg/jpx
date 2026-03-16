# Jsonpatch Functions (3)

## `json_diff`

**Signature:** `object, object -> array`

Generate JSON Patch (RFC 6902) that transforms first object into second

```
json_diff({a: 1}, {a: 2}) -> [{op: 'replace', path: '/a', value: 2}]
```
_Replace value_

```
json_diff({a: 1}, {a: 1, b: 2}) -> [{op: 'add', path: '/b', value: 2}]
```
_Add field_

---

## `json_merge_patch`

**Signature:** `object, object -> object`

Apply JSON Merge Patch (RFC 7396) to an object

```
json_merge_patch({a: 1, b: 2}, {b: 3, c: 4}) -> {a: 1, b: 3, c: 4}
```
_Merge objects_

```
json_merge_patch({a: 1}, {a: `null`}) -> {}
```
_Null removes field_

---

## `json_patch`

**Signature:** `object, array -> object`

Apply JSON Patch (RFC 6902) operations to an object

```
json_patch({a: 1}, [{op: 'add', path: '/b', value: 2}]) -> {a: 1, b: 2}
```
_Add operation_

```
json_patch({a: 1}, [{op: 'replace', path: '/a', value: 2}]) -> {a: 2}
```
_Replace operation_

---

