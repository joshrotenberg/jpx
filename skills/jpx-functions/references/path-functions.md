# Path Functions (7)

## `path_basename`

**Signature:** `string -> string`

Get filename from path

```
path_basename('/foo/bar.txt') -> \"bar.txt\"
```
_Unix path_

```
path_basename('/foo/bar/') -> \"bar\"
```
_Directory path_

---

## `path_dirname`

**Signature:** `string -> string`

Get directory from path

```
path_dirname('/foo/bar.txt') -> \"/foo\"
```
_Unix path_

```
path_dirname('/foo/bar/baz') -> \"/foo/bar\"
```
_Nested path_

---

## `path_ext`

**Signature:** `string -> string`

Get file extension

```
path_ext('/foo/bar.txt') -> \"txt\"
```
_Text file_

```
path_ext('image.png') -> \"png\"
```
_Image file_

---

## `path_is_absolute`

**Signature:** `string -> boolean`

Check if path is absolute

```
path_is_absolute('/foo/bar') -> true
```
_Absolute path_

```
path_is_absolute('foo/bar') -> false
```
_Relative path_

---

## `path_is_relative`

**Signature:** `string -> boolean`

Check if path is relative

```
path_is_relative('foo/bar') -> true
```
_Relative path_

```
path_is_relative('file.txt') -> true
```
_Filename only_

---

## `path_join`

**Signature:** `string... -> string`

Join path segments

```
path_join('/foo', 'bar', 'baz') -> \"/foo/bar/baz\"
```
_Multiple segments_

```
path_join('/home', 'user', 'file.txt') -> \"/home/user/file.txt\"
```
_Full path_

---

## `path_stem`

**Signature:** `string -> string`

Get filename without extension

```
path_stem('file.txt') -> \"file\"
```
_Simple file_

```
path_stem('/foo/bar.tar.gz') -> \"bar.tar\"
```
_Double extension_

---

