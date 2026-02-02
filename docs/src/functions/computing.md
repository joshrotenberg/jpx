# Computing Functions

Computing-related utility functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`bit_and`](#bit-and) | `number, number -> number` | Bitwise AND |
| [`bit_not`](#bit-not) | `number -> number` | Bitwise NOT |
| [`bit_or`](#bit-or) | `number, number -> number` | Bitwise OR |
| [`bit_shift_left`](#bit-shift-left) | `number, number -> number` | Bitwise left shift |
| [`bit_shift_right`](#bit-shift-right) | `number, number -> number` | Bitwise right shift |
| [`bit_xor`](#bit-xor) | `number, number -> number` | Bitwise XOR |
| [`format_bytes`](#format-bytes) | `number -> string` | Format bytes (decimal) |
| [`format_bytes_binary`](#format-bytes-binary) | `number -> string` | Format bytes (binary) |
| [`parse_bytes`](#parse-bytes) | `string -> number` | Parse byte size string |

## Functions

### bit_and

Bitwise AND

**Signature:** `number, number -> number`

**Examples:**

```text
# 1100 AND 1010 = 1000
bit_and(`12`, `10`) -> 8
# Mask lower 4 bits
bit_and(`255`, `15`) -> 15
# 0111 AND 0011 = 0011
bit_and(`7`, `3`) -> 3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_and(`12`, `10`)'
```

### bit_not

Bitwise NOT

**Signature:** `number -> number`

**Examples:**

```text
# Invert zero
bit_not(`0`) -> -1
# Invert all ones
bit_not(`-1`) -> 0
# Invert one
bit_not(`1`) -> -2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_not(`0`)'
```

### bit_or

Bitwise OR

**Signature:** `number, number -> number`

**Examples:**

```text
# 1100 OR 1010 = 1110
bit_or(`12`, `10`) -> 14
# 0001 OR 0010 = 0011
bit_or(`1`, `2`) -> 3
# Zero OR any = any
bit_or(`0`, `255`) -> 255
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_or(`12`, `10`)'
```

### bit_shift_left

Bitwise left shift

**Signature:** `number, number -> number`

**Examples:**

```text
# Shift 1 left by 4
bit_shift_left(`1`, `4`) -> 16
# Shift by 0 unchanged
bit_shift_left(`1`, `0`) -> 1
# Multiply by 4
bit_shift_left(`5`, `2`) -> 20
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_shift_left(`1`, `4`)'
```

### bit_shift_right

Bitwise right shift

**Signature:** `number, number -> number`

**Examples:**

```text
# Divide by 4
bit_shift_right(`16`, `2`) -> 4
# Shift right by 4
bit_shift_right(`255`, `4`) -> 15
# Shift by 0 unchanged
bit_shift_right(`8`, `0`) -> 8
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_shift_right(`16`, `2`)'
```

### bit_xor

Bitwise XOR

**Signature:** `number, number -> number`

**Examples:**

```text
# 1100 XOR 1010 = 0110
bit_xor(`12`, `10`) -> 6
# Same values = 0
bit_xor(`255`, `255`) -> 0
# 0101 XOR 0011 = 0110
bit_xor(`5`, `3`) -> 6
```

**CLI Usage:**

```bash
echo '{}' | jpx 'bit_xor(`12`, `10`)'
```

### format_bytes

Format bytes (decimal)

**Signature:** `number -> string`

**Examples:**

```text
# Gigabytes
format_bytes(`1500000000`) -> \"1.50 GB\"
# Kilobytes
format_bytes(`1000`) -> \"1.00 KB\"
# Bytes
format_bytes(`500`) -> \"500 B\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'format_bytes(`1500000000`)'
```

### format_bytes_binary

Format bytes (binary)

**Signature:** `number -> string`

**Examples:**

```text
# Gibibytes
format_bytes_binary(`1073741824`) -> \"1.00 GiB\"
# Kibibytes
format_bytes_binary(`1024`) -> \"1.00 KiB\"
# Mebibytes
format_bytes_binary(`1048576`) -> \"1.00 MiB\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'format_bytes_binary(`1073741824`)'
```

### parse_bytes

Parse byte size string

**Signature:** `string -> number`

**Examples:**

```text
# Gigabytes
parse_bytes('1.5 GB') -> 1500000000
# Kilobytes
parse_bytes('1 KB') -> 1000
# Gibibytes
parse_bytes('1 GiB') -> 1073741824
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_bytes(`"1.5 GB"`)'
```

