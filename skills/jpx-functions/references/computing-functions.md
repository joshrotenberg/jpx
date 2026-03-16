# Computing Functions (9)

## `bit_and`

**Signature:** `number, number -> number`

Bitwise AND

```
bit_and(`12`, `10`) -> 8
```
_1100 AND 1010 = 1000_

```
bit_and(`255`, `15`) -> 15
```
_Mask lower 4 bits_

---

## `bit_not`

**Signature:** `number -> number`

Bitwise NOT

```
bit_not(`0`) -> -1
```
_Invert zero_

```
bit_not(`-1`) -> 0
```
_Invert all ones_

---

## `bit_or`

**Signature:** `number, number -> number`

Bitwise OR

```
bit_or(`12`, `10`) -> 14
```
_1100 OR 1010 = 1110_

```
bit_or(`1`, `2`) -> 3
```
_0001 OR 0010 = 0011_

---

## `bit_shift_left`

**Signature:** `number, number -> number`

Bitwise left shift

```
bit_shift_left(`1`, `4`) -> 16
```
_Shift 1 left by 4_

```
bit_shift_left(`1`, `0`) -> 1
```
_Shift by 0 unchanged_

---

## `bit_shift_right`

**Signature:** `number, number -> number`

Bitwise right shift

```
bit_shift_right(`16`, `2`) -> 4
```
_Divide by 4_

```
bit_shift_right(`255`, `4`) -> 15
```
_Shift right by 4_

---

## `bit_xor`

**Signature:** `number, number -> number`

Bitwise XOR

```
bit_xor(`12`, `10`) -> 6
```
_1100 XOR 1010 = 0110_

```
bit_xor(`255`, `255`) -> 0
```
_Same values = 0_

---

## `format_bytes`

**Signature:** `number -> string`

Format bytes (decimal)

```
format_bytes(`1500000000`) -> \"1.50 GB\"
```
_Gigabytes_

```
format_bytes(`1000`) -> \"1.00 KB\"
```
_Kilobytes_

---

## `format_bytes_binary`

**Signature:** `number -> string`

Format bytes (binary)

```
format_bytes_binary(`1073741824`) -> \"1.00 GiB\"
```
_Gibibytes_

```
format_bytes_binary(`1024`) -> \"1.00 KiB\"
```
_Kibibytes_

---

## `parse_bytes`

**Signature:** `string -> number`

Parse byte size string

```
parse_bytes('1.5 GB') -> 1500000000
```
_Gigabytes_

```
parse_bytes('1 KB') -> 1000
```
_Kilobytes_

---

