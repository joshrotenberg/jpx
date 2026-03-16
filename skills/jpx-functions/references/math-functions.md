# Math Functions (42)

## `abs_fn`

**Signature:** `number -> number`

Absolute value

```
abs_fn(`-5`) -> 5
```
_Negative to positive_

```
abs_fn(`5`) -> 5
```
_Already positive_

---

## `add`

**Signature:** `number, number -> number`

Add two numbers

```
add(`2`, `3`) -> 5
```
_Add integers_

```
add(`1.5`, `2.5`) -> 4.0
```
_Add floats_

---

## `ceil_fn`

**Signature:** `number -> number`

Round up to nearest integer

```
ceil_fn(`3.2`) -> 4
```
_Round up fraction_

```
ceil_fn(`3.9`) -> 4
```
_Round up high fraction_

---

## `clamp`

**Signature:** `number, number, number -> number`

Clamp value to range

```
clamp(`15`, `0`, `10`) -> 10
```
_Above max_

```
clamp(`-5`, `0`, `10`) -> 0
```
_Below min_

---

## `correlation`

**Signature:** `array, array -> number`

Compute Pearson correlation coefficient between two arrays

```
correlation([1, 2, 3], [1, 2, 3]) -> 1.0
```
_Perfect positive_

```
correlation([1, 2, 3], [3, 2, 1]) -> -1.0
```
_Perfect negative_

---

## `cos`

**Signature:** `number -> number`

Cosine function

```
cos(`0`) -> 1
```
_Cos of 0_

```
cos(`3.14159`) -> -1
```
_Cos of pi_

---

## `covariance`

**Signature:** `array, array -> number`

Covariance between two arrays

```
covariance([1, 2, 3], [1, 2, 3]) -> 0.666...
```
_Perfect positive_

```
covariance([1, 2, 3], [3, 2, 1]) -> -0.666...
```
_Perfect negative_

---

## `cumulative_sum`

**Signature:** `array -> array`

Calculate running cumulative sum of a numeric array

```
cumulative_sum([1, 2, 3, 4]) -> [1, 3, 6, 10]
```
_Running total_

```
cumulative_sum([10, -5, 3]) -> [10, 5, 8]
```
_With negatives_

---

## `divide`

**Signature:** `number, number -> number`

Divide first number by second

```
divide(`10`, `2`) -> 5
```
_Integer division_

```
divide(`7`, `2`) -> 3.5
```
_Fractional result_

---

## `ewma`

**Signature:** `array, number -> array`

Exponential weighted moving average

```
ewma([1, 2, 3, 4, 5], `0.5`) -> [1, 1.5, 2.25, ...]
```
_Alpha 0.5_

```
ewma(prices, `0.3`) -> smoothed prices
```
_Smooth stock prices_

---

## `floor_fn`

**Signature:** `number -> number`

Round down to nearest integer

```
floor_fn(`3.7`) -> 3
```
_Round down high fraction_

```
floor_fn(`3.2`) -> 3
```
_Round down low fraction_

---

## `format_number`

**Signature:** `number, number?, string? -> string`

Format number with separators and optional suffix

```
format_number(`1234567`, `0`) -> \"1,234,567\"
```
_With commas_

```
format_number(`1234.56`, `2`) -> \"1,234.56\"
```
_With decimals_

---

## `histogram`

**Signature:** `array, number -> array[object]`

Compute histogram bins for an array of numbers

```
histogram([1, 2, 3, 4, 5], `3`) -> [{min: 1.0, max: 2.33, count: 2}, ...]
```
_3 bins_

```
histogram([10, 20, 30], `2`) -> [{min: 10.0, max: 20.0, count: 2}, {min: 20.0, max: 30.0, count: 1}]
```
_2 bins_

---

## `kurtosis`

**Signature:** `array -> number`

Excess kurtosis (measure of tailedness) of a numeric array (normal distribution = 0)

```
kurtosis([1, 2, 3, 4, 5]) -> -1.3
```
_Platykurtic (light tails)_

```
kurtosis([1, 1, 5, 5]) -> -2.0
```
_Uniform-like distribution_

---

## `log`

**Signature:** `number -> number`

Natural logarithm

```
log(`2.718`) -> ~1
```
_Log of e_

```
log(`1`) -> 0
```
_Log of 1_

---

## `mad`

**Signature:** `array -> number`

Median absolute deviation (robust measure of variability)

```
mad([1, 2, 3, 4, 5]) -> 1.0
```
_MAD of simple array_

```
mad([1, 1, 1, 100]) -> 0.0
```
_MAD with outlier_

---

## `median`

**Signature:** `array -> number`

Calculate median of array

```
median([1, 2, 3, 4, 5]) -> 3
```
_Odd count_

```
median([1, 2, 3, 4]) -> 2.5
```
_Even count_

---

## `mod_fn`

**Signature:** `number, number -> number`

Modulo operation

```
mod_fn(`10`, `3`) -> 1
```
_Remainder of 10/3_

```
mod_fn(`15`, `5`) -> 0
```
_Evenly divisible_

---

## `mode`

**Signature:** `array -> any`

Find the most common value in an array

```
mode([1, 2, 2, 3]) -> 2
```
_Most frequent_

```
mode(['a', 'b', 'a', 'a']) -> 'a'
```
_String mode_

---

## `moving_avg`

**Signature:** `array, number -> array`

Simple moving average with window size

```
moving_avg([1, 2, 3, 4, 5], `3`) -> [null, null, 2, 3, 4]
```
_Window of 3_

```
moving_avg([10, 20, 30, 40], `2`) -> [null, 15, 25, 35]
```
_Window of 2_

---

## `multiply`

**Signature:** `number, number -> number`

Multiply two numbers

```
multiply(`4`, `3`) -> 12
```
_Basic multiplication_

```
multiply(`2.5`, `4`) -> 10
```
_With decimals_

---

## `normalize`

**Signature:** `array -> array`

Min-max normalize an array of numbers to [0, 1] range

```
normalize([1, 3, 5]) -> [0.0, 0.5, 1.0]
```
_Basic normalization_

```
normalize([10, 10, 10]) -> [0, 0, 0]
```
_All same values_

---

## `outliers_iqr`

**Signature:** `array, number? -> array`

Find outliers using IQR method (values outside Q1-1.5*IQR to Q3+1.5*IQR)

```
outliers_iqr([1, 2, 3, 4, 100]) -> [100]
```
_Detect outlier_

```
outliers_iqr([1, 2, 3, 4, 5]) -> []
```
_No outliers_

---

## `outliers_zscore`

**Signature:** `array, number? -> array`

Find outliers using z-score method (values with |z-score| > threshold)

```
outliers_zscore([1, 2, 3, 4, 100]) -> [100]
```
_Detect outlier_

```
outliers_zscore([1, 2, 3, 4, 5]) -> []
```
_No outliers_

---

## `percentile`

**Signature:** `array, number -> number`

Calculate percentile of array

```
percentile([1, 2, 3, 4, 5], `50`) -> 3
```
_50th percentile (median)_

```
percentile([1, 2, 3, 4, 5], `25`) -> 2
```
_25th percentile_

---

## `pow`

**Signature:** `number, number -> number`

Raise to power

```
pow(`2`, `3`) -> 8
```
_2 cubed_

```
pow(`10`, `2`) -> 100
```
_10 squared_

---

## `quantile`

**Signature:** `array, number -> number`

Nth quantile (generalized percentile, q in [0,1])

```
quantile([1, 2, 3, 4, 5], `0.5`) -> 3
```
_Median (0.5)_

```
quantile([1, 2, 3, 4, 5], `0.25`) -> 2
```
_First quartile_

---

## `quartiles`

**Signature:** `array -> object`

Calculate quartiles (Q1, Q2, Q3) and IQR of array

```
quartiles([1, 2, 3, 4, 5]) -> {min: 1, q1: 2, q2: 3, q3: 4, max: 5, iqr: 2}
```
_Basic quartiles_

```
quartiles([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).iqr -> 4.5
```
_Get IQR_

---

## `rate_of_change`

**Signature:** `array -> array`

Calculate percentage change between consecutive values

```
rate_of_change([100, 110, 121]) -> [10.0, 10.0]
```
_10% growth_

```
rate_of_change([100, 50, 25]) -> [-50.0, -50.0]
```
_50% decline_

---

## `round`

**Signature:** `number, number -> number`

Round to specified decimal places

```
round(`3.14159`, `2`) -> 3.14
```
_Two decimals_

```
round(`3.5`, `0`) -> 4
```
_Round to integer_

---

## `sin`

**Signature:** `number -> number`

Sine function

```
sin(`0`) -> 0
```
_Sin of 0_

```
sin(`1.5708`) -> 1
```
_Sin of pi/2_

---

## `skew`

**Signature:** `array -> number`

Skewness (measure of asymmetry) of a numeric array using Fisher-Pearson coefficient

```
skew([1, 2, 3, 4, 5]) -> 0.0
```
_Symmetric distribution_

```
skew([1, 1, 1, 10]) -> 1.15...
```
_Right-skewed_

---

## `sqrt`

**Signature:** `number -> number`

Square root

```
sqrt(`16`) -> 4
```
_Perfect square_

```
sqrt(`2`) -> 1.414...
```
_Irrational result_

---

## `standardize`

**Signature:** `array -> array`

Standardize array to mean=0, std=1 (z-score normalization)

```
standardize([10, 20, 30]) -> [-1.22, 0, 1.22]
```
_Basic z-scores_

```
standardize([1, 2, 3, 4, 5]) -> normalized
```
_Normalize values_

---

## `stddev`

**Signature:** `array -> number`

Calculate standard deviation of array

```
stddev([1, 2, 3, 4, 5]) -> 1.414...
```
_Basic stddev_

```
stddev([10, 10, 10]) -> 0
```
_No variation_

---

## `subtract`

**Signature:** `number, number -> number`

Subtract second number from first

```
subtract(`5`, `3`) -> 2
```
_Basic subtraction_

```
subtract(`10`, `15`) -> -5
```
_Negative result_

---

## `tan`

**Signature:** `number -> number`

Tangent function

```
tan(`0`) -> 0
```
_Tan of 0_

```
tan(`0.7854`) -> ~1
```
_Tan of pi/4_

---

## `to_fixed`

**Signature:** `number, number -> string`

Format number with exact decimal places

```
to_fixed(`3.14159`, `2`) -> \"3.14\"
```
_Two decimals_

```
to_fixed(`5`, `2`) -> \"5.00\"
```
_Pad with zeros_

---

## `trend`

**Signature:** `array -> string`

Detect trend direction in a numeric array (increasing, decreasing, or stable)

```
trend([1, 2, 3, 4, 5]) -> "increasing"
```
_Upward trend_

```
trend([5, 4, 3, 2, 1]) -> "decreasing"
```
_Downward trend_

---

## `trend_slope`

**Signature:** `array -> number`

Calculate the linear regression slope of a numeric array

```
trend_slope([1, 2, 3, 4, 5]) -> 1.0
```
_Perfect linear increase_

```
trend_slope([5, 4, 3, 2, 1]) -> -1.0
```
_Perfect linear decrease_

---

## `variance`

**Signature:** `array -> number`

Calculate variance of array

```
variance([1, 2, 3, 4, 5]) -> 2
```
_Basic variance_

```
variance([10, 10, 10]) -> 0
```
_No variation_

---

## `z_score`

**Signature:** `array -> array`

Compute z-scores (standard scores) for an array of numbers

```
z_score([1, 3, 5]) -> [-1.22, 0.0, 1.22]
```
_Basic z-scores_

```
z_score([10, 20, 30])[1] -> 0.0
```
_Middle value is 0_

---

