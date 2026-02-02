# Math Functions

Mathematical and statistical functions: arithmetic, rounding, statistics, and number formatting.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`abs_fn`](#abs-fn) | `number -> number` | Absolute value |
| [`add`](#add) | `number, number -> number` | Add two numbers |
| [`ceil_fn`](#ceil-fn) | `number -> number` | Round up to nearest integer |
| [`clamp`](#clamp) | `number, number, number -> number` | Clamp value to range |
| [`cos`](#cos) | `number -> number` | Cosine function |
| [`covariance`](#covariance) | `array, array -> number` | Covariance between two arrays |
| [`cumulative_sum`](#cumulative-sum) | `array -> array` | Calculate running cumulative sum of a numeric array |
| [`divide`](#divide) | `number, number -> number` | Divide first number by second |
| [`ewma`](#ewma) | `array, number -> array` | Exponential weighted moving average |
| [`floor_fn`](#floor-fn) | `number -> number` | Round down to nearest integer |
| [`format_number`](#format-number) | `number, number?, string? -> string` | Format number with separators and optional suffix |
| [`log`](#log) | `number -> number` | Natural logarithm |
| [`median`](#median) | `array -> number` | Calculate median of array |
| [`mod_fn`](#mod-fn) | `number, number -> number` | Modulo operation |
| [`mode`](#mode) | `array -> any` | Find the most common value in an array |
| [`moving_avg`](#moving-avg) | `array, number -> array` | Simple moving average with window size |
| [`multiply`](#multiply) | `number, number -> number` | Multiply two numbers |
| [`outliers_iqr`](#outliers-iqr) | `array, number? -> array` | Find outliers using IQR method (values outside Q1-1.5*IQR to Q3+1.5*IQR) |
| [`outliers_zscore`](#outliers-zscore) | `array, number? -> array` | Find outliers using z-score method (values with \|z-score\| > threshold) |
| [`percentile`](#percentile) | `array, number -> number` | Calculate percentile of array |
| [`pow`](#pow) | `number, number -> number` | Raise to power |
| [`quantile`](#quantile) | `array, number -> number` | Nth quantile (generalized percentile, q in [0,1]) |
| [`quartiles`](#quartiles) | `array -> object` | Calculate quartiles (Q1, Q2, Q3) and IQR of array |
| [`rate_of_change`](#rate-of-change) | `array -> array` | Calculate percentage change between consecutive values |
| [`round`](#round) | `number, number -> number` | Round to specified decimal places |
| [`sin`](#sin) | `number -> number` | Sine function |
| [`sqrt`](#sqrt) | `number -> number` | Square root |
| [`standardize`](#standardize) | `array -> array` | Standardize array to mean=0, std=1 (z-score normalization) |
| [`stddev`](#stddev) | `array -> number` | Calculate standard deviation of array |
| [`subtract`](#subtract) | `number, number -> number` | Subtract second number from first |
| [`tan`](#tan) | `number -> number` | Tangent function |
| [`to_fixed`](#to-fixed) | `number, number -> string` | Format number with exact decimal places |
| [`trend`](#trend) | `array -> string` | Detect trend direction in a numeric array (increasing, decreasing, or stable) |
| [`trend_slope`](#trend-slope) | `array -> number` | Calculate the linear regression slope of a numeric array |
| [`variance`](#variance) | `array -> number` | Calculate variance of array |

## Functions

### abs_fn

Absolute value

**Signature:** `number -> number`

**Examples:**

```text
# Negative to positive
abs_fn(`-5`) -> 5
# Already positive
abs_fn(`5`) -> 5
# Zero stays zero
abs_fn(`0`) -> 0
# Negative float
abs_fn(`-3.14`) -> 3.14
```

**CLI Usage:**

```bash
echo '{}' | jpx 'abs_fn(`-5`)'
```

### add

Add two numbers

**Signature:** `number, number -> number`

**Examples:**

```text
# Add integers
add(`2`, `3`) -> 5
# Add floats
add(`1.5`, `2.5`) -> 4.0
# With negative
add(`-5`, `3`) -> -2
# Add zero
add(`0`, `10`) -> 10
```

**CLI Usage:**

```bash
echo '{}' | jpx 'add(`2`, `3`)'
```

### ceil_fn

Round up to nearest integer

**Signature:** `number -> number`

**Examples:**

```text
# Round up fraction
ceil_fn(`3.2`) -> 4
# Round up high fraction
ceil_fn(`3.9`) -> 4
# Already integer
ceil_fn(`3.0`) -> 3
# Negative rounds toward zero
ceil_fn(`-3.2`) -> -3
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ceil_fn(`3.2`)'
```

### clamp

Clamp value to range

**Signature:** `number, number, number -> number`

**Examples:**

```text
# Above max
clamp(`15`, `0`, `10`) -> 10
# Below min
clamp(`-5`, `0`, `10`) -> 0
# Within range
clamp(`5`, `0`, `10`) -> 5
# Custom range
clamp(`100`, `50`, `75`) -> 75
```

**CLI Usage:**

```bash
echo '{}' | jpx 'clamp(`15`, `0`, `10`)'
```

### cos

Cosine function

**Signature:** `number -> number`

**Examples:**

```text
# Cos of 0
cos(`0`) -> 1
# Cos of pi
cos(`3.14159`) -> -1
# Cos of pi/2
cos(`1.5708`) -> ~0
# Cos of 2*pi
cos(`6.28318`) -> 1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cos(`0`)'
```

### covariance

Covariance between two arrays

**Signature:** `array, array -> number`

**Examples:**

```text
# Perfect positive
covariance([1, 2, 3], [1, 2, 3]) -> 0.666...
# Perfect negative
covariance([1, 2, 3], [3, 2, 1]) -> -0.666...
# No correlation
covariance([1, 2, 3], [1, 1, 1]) -> 0
# From arrays
covariance(x_values, y_values) -> number
```

**CLI Usage:**

```bash
echo '{}' | jpx 'covariance([1, 2, 3], [1, 2, 3])'
```

### cumulative_sum

Calculate running cumulative sum of a numeric array

**Signature:** `array -> array`

**Examples:**

```text
# Running total
cumulative_sum([1, 2, 3, 4]) -> [1, 3, 6, 10]
# With negatives
cumulative_sum([10, -5, 3]) -> [10, 5, 8]
# Single element
cumulative_sum([100]) -> [100]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'cumulative_sum([1, 2, 3, 4])'
```

### divide

Divide first number by second

**Signature:** `number, number -> number`

**Examples:**

```text
# Integer division
divide(`10`, `2`) -> 5
# Fractional result
divide(`7`, `2`) -> 3.5
# Repeating decimal
divide(`1`, `3`) -> 0.333...
# Negative dividend
divide(`-10`, `2`) -> -5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'divide(`10`, `2`)'
```

### ewma

Exponential weighted moving average

**Signature:** `array, number -> array`

**Examples:**

```text
# Alpha 0.5
ewma([1, 2, 3, 4, 5], `0.5`) -> [1, 1.5, 2.25, ...]
# Smooth stock prices
ewma(prices, `0.3`) -> smoothed prices
# High alpha
ewma([10, 20, 30], `0.9`) -> fast response
# Low alpha
ewma([10, 20, 30], `0.1`) -> slow response
```

**CLI Usage:**

```bash
echo '{}' | jpx 'ewma([1, 2, 3, 4, 5], `0.5`)'
```

### floor_fn

Round down to nearest integer

**Signature:** `number -> number`

**Examples:**

```text
# Round down high fraction
floor_fn(`3.7`) -> 3
# Round down low fraction
floor_fn(`3.2`) -> 3
# Already integer
floor_fn(`3.0`) -> 3
# Negative rounds away from zero
floor_fn(`-3.2`) -> -4
```

**CLI Usage:**

```bash
echo '{}' | jpx 'floor_fn(`3.7`)'
```

### format_number

Format number with separators and optional suffix

**Signature:** `number, number?, string? -> string`

**Examples:**

```text
# With commas
format_number(`1234567`, `0`) -> \"1,234,567\"
# With decimals
format_number(`1234.56`, `2`) -> \"1,234.56\"
# Million
format_number(`1000000`, `0`) -> \"1,000,000\"
# Small number
format_number(`99.9`, `1`) -> \"99.9\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'format_number(`1234567`, `0`)'
```

### log

Natural logarithm

**Signature:** `number -> number`

**Examples:**

```text
# Log of e
log(`2.718`) -> ~1
# Log of 1
log(`1`) -> 0
# Log of 10
log(`10`) -> 2.302...
# Log of 100
log(`100`) -> 4.605...
```

**CLI Usage:**

```bash
echo '{}' | jpx 'log(`2.718`)'
```

### median

Calculate median of array

**Signature:** `array -> number`

**Examples:**

```text
# Odd count
median([1, 2, 3, 4, 5]) -> 3
# Even count
median([1, 2, 3, 4]) -> 2.5
# Middle value
median([10, 20, 30]) -> 20
# Single element
median([5]) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'median([1, 2, 3, 4, 5])'
```

### mod_fn

Modulo operation

**Signature:** `number, number -> number`

**Examples:**

```text
# Remainder of 10/3
mod_fn(`10`, `3`) -> 1
# Evenly divisible
mod_fn(`15`, `5`) -> 0
# Odd check
mod_fn(`7`, `2`) -> 1
# Larger numbers
mod_fn(`100`, `7`) -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'mod_fn(`10`, `3`)'
```

### mode

Find the most common value in an array

**Signature:** `array -> any`

**Examples:**

```text
# Most frequent
mode([1, 2, 2, 3]) -> 2
# String mode
mode(['a', 'b', 'a', 'a']) -> 'a'
# Tie returns first
mode([1, 1, 2, 2, 3]) -> 1 or 2
# Single element
mode([5]) -> 5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'mode([1, 2, 2, 3])'
```

### moving_avg

Simple moving average with window size

**Signature:** `array, number -> array`

**Examples:**

```text
# Window of 3
moving_avg([1, 2, 3, 4, 5], `3`) -> [null, null, 2, 3, 4]
# Window of 2
moving_avg([10, 20, 30, 40], `2`) -> [null, 15, 25, 35]
# 7-day moving avg
moving_avg(prices, `7`) -> weekly average
# Window of 1
moving_avg([1, 2, 3], `1`) -> [1, 2, 3]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'moving_avg([1, 2, 3, 4, 5], `3`)'
```

### multiply

Multiply two numbers

**Signature:** `number, number -> number`

**Examples:**

```text
# Basic multiplication
multiply(`4`, `3`) -> 12
# With decimals
multiply(`2.5`, `4`) -> 10
# With negative
multiply(`-3`, `4`) -> -12
# Multiply by zero
multiply(`0`, `100`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'multiply(`4`, `3`)'
```

### outliers_iqr

Find outliers using IQR method (values outside Q1-1.5*IQR to Q3+1.5*IQR)

**Signature:** `array, number? -> array`

**Examples:**

```text
# Detect outlier
outliers_iqr([1, 2, 3, 4, 100]) -> [100]
# No outliers
outliers_iqr([1, 2, 3, 4, 5]) -> []
# Custom multiplier
outliers_iqr(values, `3.0`) -> extreme outliers only
# Find anomalies
outliers_iqr(measurements) -> anomalies
```

**CLI Usage:**

```bash
echo '{}' | jpx 'outliers_iqr([1, 2, 3, 4, 100])'
```

### outliers_zscore

Find outliers using z-score method (values with |z-score| > threshold)

**Signature:** `array, number? -> array`

**Examples:**

```text
# Detect outlier
outliers_zscore([1, 2, 3, 4, 100]) -> [100]
# No outliers
outliers_zscore([1, 2, 3, 4, 5]) -> []
# Custom threshold
outliers_zscore(values, `3.0`) -> extreme outliers only
# Lower threshold
outliers_zscore(measurements, `1.5`) -> more sensitive detection
```

**CLI Usage:**

```bash
echo '{}' | jpx 'outliers_zscore([1, 2, 3, 4, 100])'
```

### percentile

Calculate percentile of array

**Signature:** `array, number -> number`

**Examples:**

```text
# 50th percentile (median)
percentile([1, 2, 3, 4, 5], `50`) -> 3
# 25th percentile
percentile([1, 2, 3, 4, 5], `25`) -> 2
# 75th percentile
percentile([1, 2, 3, 4, 5], `75`) -> 4
# 90th percentile
percentile(scores, `90`) -> top 10% threshold
```

**CLI Usage:**

```bash
echo '{}' | jpx 'percentile([1, 2, 3, 4, 5], `50`)'
```

### pow

Raise to power

**Signature:** `number, number -> number`

**Examples:**

```text
# 2 cubed
pow(`2`, `3`) -> 8
# 10 squared
pow(`10`, `2`) -> 100
# 2^10
pow(`2`, `10`) -> 1024
# Square root via power
pow(`4`, `0.5`) -> 2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'pow(`2`, `3`)'
```

### quantile

Nth quantile (generalized percentile, q in [0,1])

**Signature:** `array, number -> number`

**Examples:**

```text
# Median (0.5)
quantile([1, 2, 3, 4, 5], `0.5`) -> 3
# First quartile
quantile([1, 2, 3, 4, 5], `0.25`) -> 2
# Third quartile
quantile([1, 2, 3, 4, 5], `0.75`) -> 4
# 90th quantile
quantile(values, `0.9`) -> 90th quantile
```

**CLI Usage:**

```bash
echo '{}' | jpx 'quantile([1, 2, 3, 4, 5], `0.5`)'
```

### quartiles

Calculate quartiles (Q1, Q2, Q3) and IQR of array

**Signature:** `array -> object`

**Examples:**

```text
# Basic quartiles
quartiles([1, 2, 3, 4, 5]) -> {min: 1, q1: 2, q2: 3, q3: 4, max: 5, iqr: 2}
# Get IQR
quartiles([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).iqr -> 4.5
# Q2 equals median
quartiles(values).q2 == median(values) -> true
# Distribution analysis
quartiles(scores) -> full distribution stats
```

**CLI Usage:**

```bash
echo '{}' | jpx 'quartiles([1, 2, 3, 4, 5])'
```

### rate_of_change

Calculate percentage change between consecutive values

**Signature:** `array -> array`

**Examples:**

```text
# 10% growth
rate_of_change([100, 110, 121]) -> [10.0, 10.0]
# 50% decline
rate_of_change([100, 50, 25]) -> [-50.0, -50.0]
# Varying rate
rate_of_change([1, 2, 3]) -> [100.0, 50.0]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'rate_of_change([100, 110, 121])'
```

### round

Round to specified decimal places

**Signature:** `number, number -> number`

**Examples:**

```text
# Two decimals
round(`3.14159`, `2`) -> 3.14
# Round to integer
round(`3.5`, `0`) -> 4
# Round up
round(`2.555`, `2`) -> 2.56
# One decimal
round(`123.456`, `1`) -> 123.5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'round(`3.14159`, `2`)'
```

### sin

Sine function

**Signature:** `number -> number`

**Examples:**

```text
# Sin of 0
sin(`0`) -> 0
# Sin of pi/2
sin(`1.5708`) -> 1
# Sin of pi
sin(`3.14159`) -> ~0
# Sin of 2*pi
sin(`6.28318`) -> ~0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sin(`0`)'
```

### sqrt

Square root

**Signature:** `number -> number`

**Examples:**

```text
# Perfect square
sqrt(`16`) -> 4
# Irrational result
sqrt(`2`) -> 1.414...
# Larger number
sqrt(`100`) -> 10
# Zero
sqrt(`0`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'sqrt(`16`)'
```

### standardize

Standardize array to mean=0, std=1 (z-score normalization)

**Signature:** `array -> array`

**Examples:**

```text
# Basic z-scores
standardize([10, 20, 30]) -> [-1.22, 0, 1.22]
# Normalize values
standardize([1, 2, 3, 4, 5]) -> normalized
# Standardize scores
standardize(scores) -> z-scores
# Identical values
standardize([5, 5, 5]) -> [0, 0, 0]
```

**CLI Usage:**

```bash
echo '{}' | jpx 'standardize([10, 20, 30])'
```

### stddev

Calculate standard deviation of array

**Signature:** `array -> number`

**Examples:**

```text
# Basic stddev
stddev([1, 2, 3, 4, 5]) -> 1.414...
# No variation
stddev([10, 10, 10]) -> 0
# High variation
stddev([1, 100]) -> ~49.5
# Measure spread
stddev(values) -> spread measure
```

**CLI Usage:**

```bash
echo '{}' | jpx 'stddev([1, 2, 3, 4, 5])'
```

### subtract

Subtract second number from first

**Signature:** `number, number -> number`

**Examples:**

```text
# Basic subtraction
subtract(`5`, `3`) -> 2
# Negative result
subtract(`10`, `15`) -> -5
# With decimals
subtract(`3.5`, `1.5`) -> 2
# From zero
subtract(`0`, `5`) -> -5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'subtract(`5`, `3`)'
```

### tan

Tangent function

**Signature:** `number -> number`

**Examples:**

```text
# Tan of 0
tan(`0`) -> 0
# Tan of pi/4
tan(`0.7854`) -> ~1
# Tan of pi
tan(`3.14159`) -> ~0
# Calculate tangent
tan(angle) -> ratio
```

**CLI Usage:**

```bash
echo '{}' | jpx 'tan(`0`)'
```

### to_fixed

Format number with exact decimal places

**Signature:** `number, number -> string`

**Examples:**

```text
# Two decimals
to_fixed(`3.14159`, `2`) -> \"3.14\"
# Pad with zeros
to_fixed(`5`, `2`) -> \"5.00\"
# Round to integer
to_fixed(`99.9`, `0`) -> \"100\"
# Extra precision
to_fixed(`1.5`, `3`) -> \"1.500\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_fixed(`3.14159`, `2`)'
```

### trend

Detect trend direction in a numeric array (increasing, decreasing, or stable)

**Signature:** `array -> string`

**Examples:**

```text
# Upward trend
trend([1, 2, 3, 4, 5]) -> "increasing"
# Downward trend
trend([5, 4, 3, 2, 1]) -> "decreasing"
# No change
trend([3, 3, 3, 3]) -> "stable"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trend([1, 2, 3, 4, 5])'
```

### trend_slope

Calculate the linear regression slope of a numeric array

**Signature:** `array -> number`

**Examples:**

```text
# Perfect linear increase
trend_slope([1, 2, 3, 4, 5]) -> 1.0
# Perfect linear decrease
trend_slope([5, 4, 3, 2, 1]) -> -1.0
# Noisy upward
trend_slope([1, 3, 2, 4]) -> ~0.7
```

**CLI Usage:**

```bash
echo '{}' | jpx 'trend_slope([1, 2, 3, 4, 5])'
```

### variance

Calculate variance of array

**Signature:** `array -> number`

**Examples:**

```text
# Basic variance
variance([1, 2, 3, 4, 5]) -> 2
# No variation
variance([10, 10, 10]) -> 0
# High variation
variance([1, 100]) -> 2450.25
# Squared spread
variance(values) -> spread^2
```

**CLI Usage:**

```bash
echo '{}' | jpx 'variance([1, 2, 3, 4, 5])'
```

