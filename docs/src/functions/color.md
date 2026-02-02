# Color Functions

Color manipulation and conversion functions.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`color_complement`](#color-complement) | `string -> string` | Get complementary color |
| [`color_grayscale`](#color-grayscale) | `string -> string` | Convert to grayscale |
| [`color_invert`](#color-invert) | `string -> string` | Invert a color |
| [`color_mix`](#color-mix) | `string, string, number -> string` | Mix two colors |
| [`darken`](#darken) | `string, number -> string` | Darken a color by percentage |
| [`hex_to_rgb`](#hex-to-rgb) | `string -> object` | Convert hex color to RGB |
| [`lighten`](#lighten) | `string, number -> string` | Lighten a color by percentage |
| [`rgb_to_hex`](#rgb-to-hex) | `number, number, number -> string` | Convert RGB to hex color |

## Functions

### color_complement

Get complementary color

**Signature:** `string -> string`

**Examples:**

```text
# Red to cyan
color_complement('#ff0000') -> \"#00ffff\"
# Green to magenta
color_complement('#00ff00') -> \"#ff00ff\"
# Blue to yellow
color_complement('#0000ff') -> \"#ffff00\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'color_complement(`"#ff0000"`)'
```

### color_grayscale

Convert to grayscale

**Signature:** `string -> string`

**Examples:**

```text
# Red to gray
color_grayscale('#ff0000') -> \"#4c4c4c\"
# Green to gray
color_grayscale('#00ff00') -> \"#969696\"
# White unchanged
color_grayscale('#ffffff') -> \"#ffffff\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'color_grayscale(`"#ff0000"`)'
```

### color_invert

Invert a color

**Signature:** `string -> string`

**Examples:**

```text
# Invert red
color_invert('#ff0000') -> \"#00ffff\"
# Black to white
color_invert('#000000') -> \"#ffffff\"
# White to black
color_invert('#ffffff') -> \"#000000\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'color_invert(`"#ff0000"`)'
```

### color_mix

Mix two colors

**Signature:** `string, string, number -> string`

**Examples:**

```text
# Red and blue to purple
color_mix('#ff0000', '#0000ff', `50`) -> \"#800080\"
# Red and green
color_mix('#ff0000', '#00ff00', `50`) -> \"#808000\"
# Black and white to gray
color_mix('#000000', '#ffffff', `50`) -> \"#808080\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'color_mix(`"#ff0000"`, `"#0000ff"`, `50`)'
```

### darken

Darken a color by percentage

**Signature:** `string, number -> string`

**Examples:**

```text
# Darken blue by 20%
darken('#3366cc', `20`) -> \"#2952a3\"
# Darken red by 50%
darken('#ff0000', `50`) -> \"#800000\"
# White to black
darken('#ffffff', `100`) -> \"#000000\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'darken(`"#3366cc"`, `20`)'
```

### hex_to_rgb

Convert hex color to RGB

**Signature:** `string -> object`

**Examples:**

```text
# Orange
hex_to_rgb('#ff5500') -> {b: 0, g: 85, r: 255}
# Black
hex_to_rgb('#000000') -> {b: 0, g: 0, r: 0}
# White
hex_to_rgb('#ffffff') -> {b: 255, g: 255, r: 255}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'hex_to_rgb(`"#ff5500"`)'
```

### lighten

Lighten a color by percentage

**Signature:** `string, number -> string`

**Examples:**

```text
# Lighten blue by 20%
lighten('#3366cc', `20`) -> \"#5c85d6\"
# Lighten dark red
lighten('#800000', `50`) -> \"#ff0000\"
# Black to white
lighten('#000000', `100`) -> \"#ffffff\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'lighten(`"#3366cc"`, `20`)'
```

### rgb_to_hex

Convert RGB to hex color

**Signature:** `number, number, number -> string`

**Examples:**

```text
# Orange
rgb_to_hex(`255`, `85`, `0`) -> \"#ff5500\"
# Black
rgb_to_hex(`0`, `0`, `0`) -> \"#000000\"
# White
rgb_to_hex(`255`, `255`, `255`) -> \"#ffffff\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'rgb_to_hex(`255`, `85`, `0`)'
```

