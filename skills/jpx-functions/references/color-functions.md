# Color Functions (8)

## `color_complement`

**Signature:** `string -> string`

Get complementary color

```
color_complement('#ff0000') -> \"#00ffff\"
```
_Red to cyan_

```
color_complement('#00ff00') -> \"#ff00ff\"
```
_Green to magenta_

---

## `color_grayscale`

**Signature:** `string -> string`

Convert to grayscale

```
color_grayscale('#ff0000') -> \"#4c4c4c\"
```
_Red to gray_

```
color_grayscale('#00ff00') -> \"#969696\"
```
_Green to gray_

---

## `color_invert`

**Signature:** `string -> string`

Invert a color

```
color_invert('#ff0000') -> \"#00ffff\"
```
_Invert red_

```
color_invert('#000000') -> \"#ffffff\"
```
_Black to white_

---

## `color_mix`

**Signature:** `string, string, number -> string`

Mix two colors

```
color_mix('#ff0000', '#0000ff', `50`) -> \"#800080\"
```
_Red and blue to purple_

```
color_mix('#ff0000', '#00ff00', `50`) -> \"#808000\"
```
_Red and green_

---

## `darken`

**Signature:** `string, number -> string`

Darken a color by percentage

```
darken('#3366cc', `20`) -> \"#2952a3\"
```
_Darken blue by 20%_

```
darken('#ff0000', `50`) -> \"#800000\"
```
_Darken red by 50%_

---

## `hex_to_rgb`

**Signature:** `string -> object`

Convert hex color to RGB

```
hex_to_rgb('#ff5500') -> {b: 0, g: 85, r: 255}
```
_Orange_

```
hex_to_rgb('#000000') -> {b: 0, g: 0, r: 0}
```
_Black_

---

## `lighten`

**Signature:** `string, number -> string`

Lighten a color by percentage

```
lighten('#3366cc', `20`) -> \"#5c85d6\"
```
_Lighten blue by 20%_

```
lighten('#800000', `50`) -> \"#ff0000\"
```
_Lighten dark red_

---

## `rgb_to_hex`

**Signature:** `number, number, number -> string`

Convert RGB to hex color

```
rgb_to_hex(`255`, `85`, `0`) -> \"#ff5500\"
```
_Orange_

```
rgb_to_hex(`0`, `0`, `0`) -> \"#000000\"
```
_Black_

---

