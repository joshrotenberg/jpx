# Ids Functions (3)

## `nanoid`

**Signature:** `number? -> string`

Generate nanoid

```
nanoid() -> \"V1StGXR8_Z5jdHi6B-myT\"
```
_Default 21 chars_

```
nanoid(`10`) -> \"IRFa-VaY2b\"
```
_Custom length_

---

## `ulid`

**Signature:** `-> string`

Generate ULID

```
ulid() -> \"01ARZ3NDEKTSV4RRFFQ69G5FAV\"
```
_Generate ULID_

```
length(ulid()) -> 26
```
_Always 26 chars_

---

## `ulid_timestamp`

**Signature:** `string -> number`

Extract timestamp from ULID

```
ulid_timestamp('01ARZ3NDEKTSV4RRFFQ69G5FAV') -> 1469918176385
```
_Extract timestamp_

```
ulid_timestamp('01BX5ZZKBKACTAV9WEVGEMMVRY') -> 1484581420610
```
_Another ULID_

---

