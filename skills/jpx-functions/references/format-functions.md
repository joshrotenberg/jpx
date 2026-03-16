# Format Functions (6)

## `from_csv`

**Signature:** `string -> array`

Parse CSV string into array of arrays (jq parity)

```
from_csv("a,b,c\\n1,2,3") -> [["a", "b", "c"], ["1", "2", "3"]]
```
_Basic CSV parsing_

```
from_csv("\"hello, world\",test") -> [["hello, world", "test"]]
```
_Quoted fields with commas_

---

## `from_tsv`

**Signature:** `string -> array`

Parse TSV string into array of arrays (jq parity)

```
from_tsv("a\\tb\\tc\\n1\\t2\\t3") -> [["a", "b", "c"], ["1", "2", "3"]]
```
_Basic TSV parsing_

```
from_tsv("hello world\\ttest") -> [["hello world", "test"]]
```
_Spaces preserved in fields_

---

## `to_csv`

**Signature:** `array -> string`

Convert array to CSV row string (RFC 4180 compliant)

```
to_csv(['a', 'b', 'c']) -> "a,b,c"
```
_Simple strings_

```
to_csv(['hello', `42`, `true`, `null`]) -> "hello,42,true,"
```
_Mixed types_

---

## `to_csv_rows`

**Signature:** `array -> string`

Convert array of arrays to multi-line CSV string

```
to_csv_rows([[`1`, `2`, `3`], [`4`, `5`, `6`]]) -> "1,2,3\n4,5,6"
```
_Numeric rows_

```
to_csv_rows([['a', 'b'], ['c', 'd']]) -> "a,b\nc,d"
```
_String rows_

---

## `to_csv_table`

**Signature:** `array, array? -> string`

Convert array of objects to CSV with header row

```
to_csv_table([{name: 'alice', age: `30`}]) -> "age,name\n30,alice"
```
_Keys sorted alphabetically_

```
to_csv_table([{name: 'alice', age: `30`}], ['name', 'age']) -> "name,age\nalice,30"
```
_Explicit column order_

---

## `to_tsv`

**Signature:** `array -> string`

Convert array to TSV row string (tab-separated)

```
to_tsv(['a', 'b', 'c']) -> "a\tb\tc"
```
_Simple strings_

```
to_tsv(['hello', `42`, `true`]) -> "hello\t42\ttrue"
```
_Mixed types_

---

