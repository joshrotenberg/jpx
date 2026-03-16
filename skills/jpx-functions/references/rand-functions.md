# Rand Functions (5)

## `random`

**Signature:** `-> number`

Generate random number between 0 and 1

```
random() -> 0.123456...
```
_Random float_

```
floor(multiply(random(), `100`)) -> 42
```
_Random 0-99_

---

## `random_choice`

**Signature:** `array -> any`

Pick a random element from an array

```
random_choice(['a', 'b', 'c']) -> 'b'
```
_Random element_

```
random_choice([]) -> null
```
_Empty array returns null_

---

## `random_int`

**Signature:** `number, number -> number`

Generate a random integer in an inclusive range [min, max]

```
random_int(`1`, `10`) -> 7
```
_Random integer 1-10_

```
random_int(`0`, `1`) -> 0
```
_Coin flip_

---

## `sample`

**Signature:** `array, number -> array`

Random sample from array

```
sample([1, 2, 3, 4], `2`) -> [3, 1]
```
_Sample 2 items_

```
sample(['a', 'b', 'c'], `1`) -> ['b']
```
_Sample 1 item_

---

## `shuffle`

**Signature:** `array -> array`

Randomly shuffle array

```
shuffle([1, 2, 3]) -> [2, 3, 1]
```
_Shuffle numbers_

```
length(shuffle([1, 2, 3])) -> 3
```
_Preserves length_

---

