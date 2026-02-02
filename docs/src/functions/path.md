# Path Functions

File path manipulation functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`path_basename`](#path-basename) | `string -> string` | Get filename from path |
| [`path_dirname`](#path-dirname) | `string -> string` | Get directory from path |
| [`path_ext`](#path-ext) | `string -> string` | Get file extension |
| [`path_join`](#path-join) | `string... -> string` | Join path segments |

## Functions

### path_basename

Get filename from path

**Signature:** `string -> string`

**Examples:**

```text
# Unix path
path_basename('/foo/bar.txt') -> \"bar.txt\"
# Directory path
path_basename('/foo/bar/') -> \"bar\"
# Just filename
path_basename('file.txt') -> \"file.txt\"
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
path_dirname('/foo/bar.txt') -> \"/foo\"
# Nested path
path_dirname('/foo/bar/baz') -> \"/foo/bar\"
# No directory
path_dirname('file.txt') -> \"\"
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
path_ext('/foo/bar.txt') -> \"txt\"
# Image file
path_ext('image.png') -> \"png\"
# No extension
path_ext('noext') -> \"\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_ext(`"/foo/bar.txt"`)'
```

### path_join

Join path segments

**Signature:** `string... -> string`

**Examples:**

```text
# Multiple segments
path_join('/foo', 'bar', 'baz') -> \"/foo/bar/baz\"
# Full path
path_join('/home', 'user', 'file.txt') -> \"/home/user/file.txt\"
# Relative path
path_join('a', 'b') -> \"a/b\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'path_join(`"/foo"`, `"bar"`, `"baz"`)'
```

