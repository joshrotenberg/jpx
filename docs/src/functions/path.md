# Path Functions

File path manipulation functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`path_basename`](#path-basename) | `string -> string` | Get filename from path |
| [`path_dirname`](#path-dirname) | `string -> string` | Get directory from path |
| [`path_ext`](#path-ext) | `string -> string` | Get file extension |
| [`path_is_absolute`](#path-is-absolute) | `string -> boolean` | Check if path is absolute |
| [`path_is_relative`](#path-is-relative) | `string -> boolean` | Check if path is relative |
| [`path_join`](#path-join) | `string... -> string` | Join path segments |
| [`path_stem`](#path-stem) | `string -> string` | Get filename without extension |

## Functions

### path_basename

Get filename from path

**Signature:** `string -> string`

**Examples:**

```text
# Unix path
path_basename('/foo/bar.txt') -> "bar.txt"
# Directory path
path_basename('/foo/bar/') -> "bar"
# Just filename
path_basename('file.txt') -> "file.txt"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_basename(`"/foo/bar.txt"`)'
```

### path_dirname

Get directory from path

**Signature:** `string -> string`

**Examples:**

```text
# Unix path
path_dirname('/foo/bar.txt') -> "/foo"
# Nested path
path_dirname('/foo/bar/baz') -> "/foo/bar"
# No directory
path_dirname('file.txt') -> ""
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_dirname(`"/foo/bar.txt"`)'
```

### path_ext

Get file extension

**Signature:** `string -> string`

**Examples:**

```text
# Text file
path_ext('/foo/bar.txt') -> "txt"
# Image file
path_ext('image.png') -> "png"
# No extension
path_ext('noext') -> ""
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_ext(`"/foo/bar.txt"`)'
```

### path_is_absolute

Check if path is absolute

**Signature:** `string -> boolean`

**Examples:**

```text
# Absolute path
path_is_absolute('/foo/bar') -> true
# Relative path
path_is_absolute('foo/bar') -> false
# Filename only
path_is_absolute('file.txt') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_is_absolute(`"/foo/bar"`)'
```

### path_is_relative

Check if path is relative

**Signature:** `string -> boolean`

**Examples:**

```text
# Relative path
path_is_relative('foo/bar') -> true
# Filename only
path_is_relative('file.txt') -> true
# Absolute path
path_is_relative('/foo/bar') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_is_relative(`"foo/bar"`)'
```

### path_join

Join path segments

**Signature:** `string... -> string`

**Examples:**

```text
# Multiple segments
path_join('/foo', 'bar', 'baz') -> "/foo/bar/baz"
# Full path
path_join('/home', 'user', 'file.txt') -> "/home/user/file.txt"
# Relative path
path_join('a', 'b') -> "a/b"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_join(`"/foo"`, `"bar"`, `"baz"`)'
```

### path_stem

Get filename without extension

**Signature:** `string -> string`

**Examples:**

```text
# Simple file
path_stem('file.txt') -> "file"
# Double extension
path_stem('/foo/bar.tar.gz') -> "bar.tar"
# No extension
path_stem('noext') -> "noext"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_stem(`"/foo/bar.txt"`)'
```
