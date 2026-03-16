# Semver Functions (7)

## `semver_compare`

**Signature:** `string, string -> number`

Compare versions (-1, 0, 1)

```
semver_compare('1.0.0', '2.0.0') -> -1
```
_Less than_

```
semver_compare('2.0.0', '1.0.0') -> 1
```
_Greater than_

---

## `semver_is_valid`

**Signature:** `string -> boolean`

Check if string is valid semver

```
semver_is_valid('1.2.3') -> true
```
_Valid semver_

```
semver_is_valid('1.2.3-alpha') -> true
```
_With prerelease_

---

## `semver_major`

**Signature:** `string -> number`

Get major version

```
semver_major('1.2.3') -> 1
```
_Major version_

```
semver_major('10.0.0') -> 10
```
_Double digit_

---

## `semver_minor`

**Signature:** `string -> number`

Get minor version

```
semver_minor('1.2.3') -> 2
```
_Minor version_

```
semver_minor('1.10.0') -> 10
```
_Double digit_

---

## `semver_parse`

**Signature:** `string -> object`

Parse semantic version

```
semver_parse('1.2.3') -> {major: 1, minor: 2, patch: 3}
```
_Basic version_

```
semver_parse('1.2.3-alpha').pre -> 'alpha'
```
_With prerelease_

---

## `semver_patch`

**Signature:** `string -> number`

Get patch version

```
semver_patch('1.2.3') -> 3
```
_Patch version_

```
semver_patch('1.2.10') -> 10
```
_Double digit_

---

## `semver_satisfies`

**Signature:** `string, string -> boolean`

Check if version matches constraint

```
semver_satisfies('1.2.3', '^1.0.0') -> true
```
_Caret range_

```
semver_satisfies('2.0.0', '^1.0.0') -> false
```
_Outside range_

---

