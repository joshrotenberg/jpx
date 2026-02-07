//! Mathematical functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register only the math functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "round", enabled, Box::new(RoundFn::new()));
    register_if_enabled(runtime, "floor_fn", enabled, Box::new(FloorFnExt::new()));
    register_if_enabled(runtime, "ceil_fn", enabled, Box::new(CeilFnExt::new()));
    register_if_enabled(runtime, "abs_fn", enabled, Box::new(AbsFnExt::new()));
    register_if_enabled(runtime, "mod_fn", enabled, Box::new(ModFn::new()));
    register_if_enabled(runtime, "pow", enabled, Box::new(PowFn::new()));
    register_if_enabled(runtime, "sqrt", enabled, Box::new(SqrtFn::new()));
    register_if_enabled(runtime, "log", enabled, Box::new(LogFn::new()));
    register_if_enabled(runtime, "clamp", enabled, Box::new(ClampFn::new()));
    register_if_enabled(runtime, "median", enabled, Box::new(MedianFn::new()));
    register_if_enabled(
        runtime,
        "percentile",
        enabled,
        Box::new(PercentileFn::new()),
    );
    register_if_enabled(runtime, "variance", enabled, Box::new(VarianceFn::new()));
    register_if_enabled(runtime, "stddev", enabled, Box::new(StddevFn::new()));
    register_if_enabled(runtime, "sin", enabled, Box::new(SinFn::new()));
    register_if_enabled(runtime, "cos", enabled, Box::new(CosFn::new()));
    register_if_enabled(runtime, "tan", enabled, Box::new(TanFn::new()));
    register_if_enabled(runtime, "asin", enabled, Box::new(AsinFn::new()));
    register_if_enabled(runtime, "acos", enabled, Box::new(AcosFn::new()));
    register_if_enabled(runtime, "atan", enabled, Box::new(AtanFn::new()));
    register_if_enabled(runtime, "atan2", enabled, Box::new(Atan2Fn::new()));
    register_if_enabled(runtime, "deg_to_rad", enabled, Box::new(DegToRadFn::new()));
    register_if_enabled(runtime, "rad_to_deg", enabled, Box::new(RadToDegFn::new()));
    register_if_enabled(runtime, "sign", enabled, Box::new(SignFn::new()));
    register_if_enabled(runtime, "add", enabled, Box::new(AddFn::new()));
    register_if_enabled(runtime, "subtract", enabled, Box::new(SubtractFn::new()));
    register_if_enabled(runtime, "multiply", enabled, Box::new(MultiplyFn::new()));
    register_if_enabled(runtime, "divide", enabled, Box::new(DivideFn::new()));
    register_if_enabled(runtime, "mode", enabled, Box::new(ModeFn::new()));
    register_if_enabled(runtime, "to_fixed", enabled, Box::new(ToFixedFn::new()));
    register_if_enabled(
        runtime,
        "format_number",
        enabled,
        Box::new(FormatNumberFn::new()),
    );
    register_if_enabled(runtime, "histogram", enabled, Box::new(HistogramFn::new()));
    register_if_enabled(runtime, "normalize", enabled, Box::new(NormalizeFn::new()));
    register_if_enabled(runtime, "z_score", enabled, Box::new(ZScoreFn::new()));
    register_if_enabled(
        runtime,
        "correlation",
        enabled,
        Box::new(CorrelationFn::new()),
    );
    register_if_enabled(runtime, "quantile", enabled, Box::new(QuantileFn::new()));
    register_if_enabled(runtime, "moving_avg", enabled, Box::new(MovingAvgFn::new()));
    register_if_enabled(runtime, "ewma", enabled, Box::new(EwmaFn::new()));
    register_if_enabled(
        runtime,
        "covariance",
        enabled,
        Box::new(CovarianceFn::new()),
    );
    register_if_enabled(
        runtime,
        "standardize",
        enabled,
        Box::new(StandardizeFn::new()),
    );
    register_if_enabled(runtime, "quartiles", enabled, Box::new(QuartilesFn::new()));
    register_if_enabled(
        runtime,
        "outliers_iqr",
        enabled,
        Box::new(OutliersIqrFn::new()),
    );
    register_if_enabled(
        runtime,
        "outliers_zscore",
        enabled,
        Box::new(OutliersZscoreFn::new()),
    );
    // Time series functions
    register_if_enabled(runtime, "trend", enabled, Box::new(TrendFn::new()));
    register_if_enabled(
        runtime,
        "trend_slope",
        enabled,
        Box::new(TrendSlopeFn::new()),
    );
    register_if_enabled(
        runtime,
        "rate_of_change",
        enabled,
        Box::new(RateOfChangeFn::new()),
    );
    register_if_enabled(
        runtime,
        "cumulative_sum",
        enabled,
        Box::new(CumulativeSumFn::new()),
    );
}

// =============================================================================
// round(number, precision?) -> number
// =============================================================================

defn!(RoundFn, vec![arg!(number)], Some(arg!(number)));

impl Function for RoundFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_f64().unwrap();

        let precision = if args.len() > 1 {
            args[1].as_f64().map(|p| p as i32).unwrap_or(0)
        } else {
            0
        };

        let result = if precision == 0 {
            n.round()
        } else {
            let multiplier = 10_f64.powi(precision);
            (n * multiplier).round() / multiplier
        };

        Ok(number_value(result))
    }
}

// =============================================================================
// floor_fn(number) -> number
// =============================================================================

defn!(FloorFnExt, vec![arg!(number)], None);

impl Function for FloorFnExt {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(Value::Number(Number::from(n.floor() as i64)))
    }
}

// =============================================================================
// ceil_fn(number) -> number
// =============================================================================

defn!(CeilFnExt, vec![arg!(number)], None);

impl Function for CeilFnExt {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(Value::Number(Number::from(n.ceil() as i64)))
    }
}

// =============================================================================
// abs_fn(number) -> number
// =============================================================================

defn!(AbsFnExt, vec![arg!(number)], None);

impl Function for AbsFnExt {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.abs()))
    }
}

// =============================================================================
// mod_fn(number, divisor) -> number
// =============================================================================

defn!(ModFn, vec![arg!(number), arg!(number)], None);

impl Function for ModFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_f64().unwrap();
        let divisor = args[1].as_f64().unwrap();

        if divisor == 0.0 {
            return Err(crate::functions::custom_error(ctx, "Division by zero"));
        }

        Ok(number_value(n % divisor))
    }
}

// =============================================================================
// pow(base, exponent) -> number
// =============================================================================

defn!(PowFn, vec![arg!(number), arg!(number)], None);

impl Function for PowFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let base = args[0].as_f64().unwrap();
        let exp = args[1].as_f64().unwrap();
        Ok(number_value(base.powf(exp)))
    }
}

// =============================================================================
// sqrt(number) -> number
// =============================================================================

defn!(SqrtFn, vec![arg!(number)], None);

impl Function for SqrtFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();

        if n < 0.0 {
            return Err(crate::functions::custom_error(
                ctx,
                "Cannot take square root of negative number",
            ));
        }

        Ok(number_value(n.sqrt()))
    }
}

// =============================================================================
// log(number, base?) -> number (default base e)
// =============================================================================

defn!(LogFn, vec![arg!(number)], Some(arg!(number)));

impl Function for LogFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_f64().unwrap();

        if n <= 0.0 {
            return Err(crate::functions::custom_error(
                ctx,
                "Logarithm requires positive number",
            ));
        }

        let result = if args.len() > 1 {
            let base = args[1].as_f64().unwrap();
            n.log(base)
        } else {
            n.ln()
        };

        Ok(number_value(result))
    }
}

// =============================================================================
// clamp(number, min, max) -> number
// =============================================================================

defn!(
    ClampFn,
    vec![arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for ClampFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_f64().unwrap();
        let min = args[1].as_f64().unwrap();
        let max = args[2].as_f64().unwrap();

        let result = n.max(min).min(max);

        Ok(number_value(result))
    }
}

// =============================================================================
// median(array) -> number
// =============================================================================

defn!(MedianFn, vec![arg!(array)], None);

impl Function for MedianFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let mut numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if numbers.is_empty() {
            return Ok(Value::Null);
        }

        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = numbers.len();
        let median = if len.is_multiple_of(2) {
            (numbers[len / 2 - 1] + numbers[len / 2]) / 2.0
        } else {
            numbers[len / 2]
        };

        Ok(number_value(median))
    }
}

// =============================================================================
// percentile(array, p) -> number (pth percentile, p in 0-100)
// =============================================================================

defn!(PercentileFn, vec![arg!(array), arg!(number)], None);

impl Function for PercentileFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();
        let p = args[1].as_f64().unwrap();

        if !(0.0..=100.0).contains(&p) {
            return Err(crate::functions::custom_error(
                ctx,
                "Percentile must be between 0 and 100",
            ));
        }

        let mut numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if numbers.is_empty() {
            return Ok(Value::Null);
        }

        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = numbers.len();
        if len == 1 {
            return Ok(number_value(numbers[0]));
        }

        let rank = (p / 100.0) * (len - 1) as f64;
        let lower_idx = rank.floor() as usize;
        let upper_idx = rank.ceil() as usize;
        let fraction = rank - lower_idx as f64;

        let result = if lower_idx == upper_idx {
            numbers[lower_idx]
        } else {
            numbers[lower_idx] * (1.0 - fraction) + numbers[upper_idx] * fraction
        };

        Ok(number_value(result))
    }
}

// =============================================================================
// variance(array) -> number (population variance)
// =============================================================================

defn!(VarianceFn, vec![arg!(array)], None);

impl Function for VarianceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if numbers.is_empty() {
            return Ok(Value::Null);
        }

        let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
        let variance =
            numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers.len() as f64;

        Ok(number_value(variance))
    }
}

// =============================================================================
// stddev(array) -> number (population standard deviation)
// =============================================================================

defn!(StddevFn, vec![arg!(array)], None);

impl Function for StddevFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if numbers.is_empty() {
            return Ok(Value::Null);
        }

        let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
        let variance =
            numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers.len() as f64;
        let stddev = variance.sqrt();

        Ok(number_value(stddev))
    }
}

// =============================================================================
// Trigonometric functions
// =============================================================================

defn!(SinFn, vec![arg!(number)], None);

impl Function for SinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.sin()))
    }
}

defn!(CosFn, vec![arg!(number)], None);

impl Function for CosFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.cos()))
    }
}

defn!(TanFn, vec![arg!(number)], None);

impl Function for TanFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.tan()))
    }
}

defn!(AsinFn, vec![arg!(number)], None);

impl Function for AsinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        let result = n.asin();
        // Return null for out-of-domain values (|n| > 1 produces NaN)
        if result.is_nan() {
            Ok(Value::Null)
        } else {
            Ok(number_value(result))
        }
    }
}

defn!(AcosFn, vec![arg!(number)], None);

impl Function for AcosFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        let result = n.acos();
        // Return null for out-of-domain values (|n| > 1 produces NaN)
        if result.is_nan() {
            Ok(Value::Null)
        } else {
            Ok(number_value(result))
        }
    }
}

defn!(AtanFn, vec![arg!(number)], None);

impl Function for AtanFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.atan()))
    }
}

defn!(Atan2Fn, vec![arg!(number), arg!(number)], None);

impl Function for Atan2Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let y = args[0].as_f64().unwrap();
        let x = args[1].as_f64().unwrap();
        Ok(number_value(y.atan2(x)))
    }
}

defn!(DegToRadFn, vec![arg!(number)], None);

impl Function for DegToRadFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.to_radians()))
    }
}

defn!(RadToDegFn, vec![arg!(number)], None);

impl Function for RadToDegFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        Ok(number_value(n.to_degrees()))
    }
}

defn!(SignFn, vec![arg!(number)], None);

impl Function for SignFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_f64().unwrap();
        let sign = if n > 0.0 {
            1
        } else if n < 0.0 {
            -1
        } else {
            0
        };
        Ok(Value::Number(Number::from(sign)))
    }
}

// =============================================================================
// add(a, b) -> number
// =============================================================================

defn!(AddFn, vec![arg!(number), arg!(number)], None);

impl Function for AddFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_f64().unwrap();
        let b = args[1].as_f64().unwrap();
        Ok(number_value(a + b))
    }
}

// =============================================================================
// subtract(a, b) -> number
// =============================================================================

defn!(SubtractFn, vec![arg!(number), arg!(number)], None);

impl Function for SubtractFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_f64().unwrap();
        let b = args[1].as_f64().unwrap();
        Ok(number_value(a - b))
    }
}

// =============================================================================
// multiply(a, b) -> number
// =============================================================================

defn!(MultiplyFn, vec![arg!(number), arg!(number)], None);

impl Function for MultiplyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_f64().unwrap();
        let b = args[1].as_f64().unwrap();
        Ok(number_value(a * b))
    }
}

// =============================================================================
// divide(a, b) -> number
// =============================================================================

defn!(DivideFn, vec![arg!(number), arg!(number)], None);

impl Function for DivideFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_f64().unwrap();
        let b = args[1].as_f64().unwrap();
        if b == 0.0 {
            return Err(crate::functions::custom_error(ctx, "Division by zero"));
        }
        Ok(number_value(a / b))
    }
}

// =============================================================================
// mode(array) -> any (most common value)
// =============================================================================

defn!(ModeFn, vec![arg!(array)], None);

impl Function for ModeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        // Count occurrences - use JSON string representation as key
        let mut counts: std::collections::HashMap<String, (usize, Value)> =
            std::collections::HashMap::new();

        for item in arr.iter() {
            let key = serde_json::to_string(item).unwrap_or_default();
            counts
                .entry(key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, item.clone()));
        }

        // Find the value with highest count
        let (_, (_, mode_value)) = counts
            .into_iter()
            .max_by_key(|(_, (count, _))| *count)
            .unwrap();

        Ok(mode_value)
    }
}

// =============================================================================
// to_fixed(number, precision) -> string
// =============================================================================

defn!(ToFixedFn, vec![arg!(number), arg!(number)], None);

impl Function for ToFixedFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_f64().unwrap();
        let precision = args[1].as_f64().unwrap() as usize;

        let result = format!("{:.prec$}", num, prec = precision);
        Ok(Value::String(result))
    }
}

// =============================================================================
// format_number(number, precision?, suffix?) -> string
// =============================================================================

defn!(FormatNumberFn, vec![arg!(number)], Some(arg!(any)));

impl Function for FormatNumberFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_f64().unwrap();

        let precision = args
            .get(1)
            .and_then(|v| v.as_f64())
            .map(|n| n as usize)
            .unwrap_or(0);

        let suffix = args.get(2).and_then(|v| v.as_str()).map(|s| s.to_string());

        // Handle suffix scaling (k, M, B, T)
        let (scaled_num, auto_suffix) = if let Some(ref s) = suffix {
            match s.as_str() {
                "k" | "K" => (num / 1_000.0, "k"),
                "M" => (num / 1_000_000.0, "M"),
                "B" => (num / 1_000_000_000.0, "B"),
                "T" => (num / 1_000_000_000_000.0, "T"),
                "auto" => {
                    let abs_num = num.abs();
                    if abs_num >= 1_000_000_000_000.0 {
                        (num / 1_000_000_000_000.0, "T")
                    } else if abs_num >= 1_000_000_000.0 {
                        (num / 1_000_000_000.0, "B")
                    } else if abs_num >= 1_000_000.0 {
                        (num / 1_000_000.0, "M")
                    } else if abs_num >= 1_000.0 {
                        (num / 1_000.0, "k")
                    } else {
                        (num, "")
                    }
                }
                _ => (num, s.as_str()),
            }
        } else {
            (num, "")
        };

        // Format with precision
        let formatted = format!("{:.prec$}", scaled_num, prec = precision);

        // Add thousand separators to the integer part
        let result = if suffix.is_none() || suffix.as_deref() == Some("") {
            add_thousand_separators(&formatted)
        } else {
            format!("{}{}", formatted, auto_suffix)
        };

        Ok(Value::String(result))
    }
}

/// Add thousand separators (commas) to a number string.
fn add_thousand_separators(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1);

    // Handle negative sign
    let (sign, digits) = if let Some(stripped) = int_part.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", int_part)
    };

    // Add commas every 3 digits from the right
    let digit_chars: Vec<char> = digits.chars().collect();
    let len = digit_chars.len();
    let with_commas: String = digit_chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let pos_from_right = len - 1 - i;
            if pos_from_right > 0 && pos_from_right.is_multiple_of(3) {
                format!("{},", c)
            } else {
                c.to_string()
            }
        })
        .collect();

    match dec_part {
        Some(dec) => format!("{}{}.{}", sign, with_commas, dec),
        None => format!("{}{}", sign, with_commas),
    }
}

// =============================================================================
// histogram(array, bins) -> array
// =============================================================================

defn!(HistogramFn, vec![arg!(array), arg!(number)], None);

impl Function for HistogramFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let num_bins = args[1].as_f64().unwrap() as usize;

        if num_bins == 0 {
            return Err(crate::functions::custom_error(
                ctx,
                "Number of bins must be greater than 0",
            ));
        }

        // Extract numeric values
        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Handle case where all values are the same
        let bin_width = if (max_val - min_val).abs() < f64::EPSILON {
            1.0
        } else {
            (max_val - min_val) / num_bins as f64
        };

        // Initialize bins
        let mut bins: Vec<(f64, f64, usize)> = (0..num_bins)
            .map(|i| {
                let bin_min = min_val + (i as f64 * bin_width);
                let bin_max = if i == num_bins - 1 {
                    max_val
                } else {
                    min_val + ((i + 1) as f64 * bin_width)
                };
                (bin_min, bin_max, 0)
            })
            .collect();

        // Count values in each bin
        for val in &values {
            let bin_idx = if (max_val - min_val).abs() < f64::EPSILON {
                0
            } else {
                let idx = ((val - min_val) / bin_width) as usize;
                idx.min(num_bins - 1)
            };
            bins[bin_idx].2 += 1;
        }

        // Convert to array of objects
        let result: Vec<Value> = bins
            .into_iter()
            .map(|(bin_min, bin_max, count)| {
                let mut map = serde_json::Map::new();
                map.insert("min".to_string(), number_value(bin_min));
                map.insert("max".to_string(), number_value(bin_max));
                map.insert("count".to_string(), Value::Number(Number::from(count)));
                Value::Object(map)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// normalize(array) -> array
// =============================================================================

defn!(NormalizeFn, vec![arg!(array)], None);

impl Function for NormalizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        let result: Vec<Value> = values
            .iter()
            .map(|v| {
                let normalized = if range.abs() < f64::EPSILON {
                    0.0
                } else {
                    (v - min_val) / range
                };
                number_value(normalized)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// z_score(array) -> array
// =============================================================================

defn!(ZScoreFn, vec![arg!(array)], None);

impl Function for ZScoreFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        let result: Vec<Value> = values
            .iter()
            .map(|v| {
                let z = if stddev.abs() < f64::EPSILON {
                    0.0
                } else {
                    (v - mean) / stddev
                };
                number_value(z)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// correlation(arr1, arr2) -> number
// =============================================================================

defn!(CorrelationFn, vec![arg!(array), arg!(array)], None);

impl Function for CorrelationFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().unwrap();
        let arr2 = args[1].as_array().unwrap();

        let values1: Vec<f64> = arr1.iter().filter_map(|v| v.as_f64()).collect();
        let values2: Vec<f64> = arr2.iter().filter_map(|v| v.as_f64()).collect();

        if values1.is_empty() || values2.is_empty() {
            return Ok(Value::Null);
        }

        // Use the shorter length
        let n = values1.len().min(values2.len());
        if n == 0 {
            return Ok(Value::Null);
        }

        let values1 = &values1[..n];
        let values2 = &values2[..n];

        let mean1 = values1.iter().sum::<f64>() / n as f64;
        let mean2 = values2.iter().sum::<f64>() / n as f64;

        let mut cov = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        for i in 0..n {
            let d1 = values1[i] - mean1;
            let d2 = values2[i] - mean2;
            cov += d1 * d2;
            var1 += d1 * d1;
            var2 += d2 * d2;
        }

        let denom = (var1 * var2).sqrt();
        let correlation = if denom.abs() < f64::EPSILON {
            0.0
        } else {
            cov / denom
        };

        Ok(number_value(correlation))
    }
}

// =============================================================================
// quantile(array, q) -> number (q in [0, 1])
// =============================================================================

defn!(QuantileFn, vec![arg!(array), arg!(number)], None);

impl Function for QuantileFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();
        let q = args[1].as_f64().unwrap();

        if !(0.0..=1.0).contains(&q) {
            return Ok(Value::Null);
        }

        let mut values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Null);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len();
        let pos = q * (n - 1) as f64;
        let lower = pos.floor() as usize;
        let upper = pos.ceil() as usize;
        let frac = pos - lower as f64;

        let result = if lower == upper {
            values[lower]
        } else {
            values[lower] * (1.0 - frac) + values[upper] * frac
        };

        Ok(number_value(result))
    }
}

// =============================================================================
// moving_avg(array, window) -> array
// =============================================================================

defn!(MovingAvgFn, vec![arg!(array), arg!(number)], None);

impl Function for MovingAvgFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let window = args[1].as_f64().unwrap() as usize;

        if window == 0 {
            return Ok(Value::Null);
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() || window > values.len() {
            return Ok(Value::Array(vec![]));
        }

        let mut result: Vec<Value> = Vec::new();

        for i in 0..values.len() {
            if i + 1 < window {
                result.push(Value::Null);
            } else {
                let start = i + 1 - window;
                let sum: f64 = values[start..=i].iter().sum();
                let avg = sum / window as f64;
                result.push(number_value(avg));
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// ewma(array, alpha) -> array
// =============================================================================

defn!(EwmaFn, vec![arg!(array), arg!(number)], None);

impl Function for EwmaFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();
        let alpha = args[1].as_f64().unwrap();

        if !(0.0..=1.0).contains(&alpha) {
            return Ok(Value::Null);
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut result: Vec<Value> = Vec::new();
        let mut ewma = values[0];

        for value in &values {
            ewma = alpha * value + (1.0 - alpha) * ewma;
            result.push(number_value(ewma));
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// covariance(arr1, arr2) -> number
// =============================================================================

defn!(CovarianceFn, vec![arg!(array), arg!(array)], None);

impl Function for CovarianceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().unwrap();
        let arr2 = args[1].as_array().unwrap();

        let values1: Vec<f64> = arr1.iter().filter_map(|v| v.as_f64()).collect();
        let values2: Vec<f64> = arr2.iter().filter_map(|v| v.as_f64()).collect();

        if values1.is_empty() || values1.len() != values2.len() {
            return Ok(Value::Null);
        }

        let n = values1.len() as f64;
        let mean1: f64 = values1.iter().sum::<f64>() / n;
        let mean2: f64 = values2.iter().sum::<f64>() / n;

        let cov: f64 = values1
            .iter()
            .zip(values2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum::<f64>()
            / n;

        Ok(number_value(cov))
    }
}

// =============================================================================
// standardize(array) -> array (z-score normalization)
// =============================================================================

defn!(StandardizeFn, vec![arg!(array)], None);

impl Function for StandardizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let n = values.len() as f64;
        let mean: f64 = values.iter().sum::<f64>() / n;
        let variance: f64 = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let result: Vec<Value> = values
            .iter()
            .map(|x| {
                let standardized = if std_dev.abs() < f64::EPSILON {
                    0.0
                } else {
                    (x - mean) / std_dev
                };
                number_value(standardized)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// quartiles(array) -> object {q1, q2, q3, min, max, iqr}
// =============================================================================

defn!(QuartilesFn, vec![arg!(array)], None);

impl Function for QuartilesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let mut values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Null);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len();
        let min = values[0];
        let max = values[n - 1];

        let q1 = percentile_value(&values, 25.0);
        let q2 = percentile_value(&values, 50.0);
        let q3 = percentile_value(&values, 75.0);
        let iqr = q3 - q1;

        let mut result = serde_json::Map::new();
        result.insert("min".to_string(), number_value(min));
        result.insert("q1".to_string(), number_value(q1));
        result.insert("q2".to_string(), number_value(q2));
        result.insert("q3".to_string(), number_value(q3));
        result.insert("max".to_string(), number_value(max));
        result.insert("iqr".to_string(), number_value(iqr));

        Ok(Value::Object(result))
    }
}

fn percentile_value(sorted_values: &[f64], p: f64) -> f64 {
    let n = sorted_values.len();
    if n == 1 {
        return sorted_values[0];
    }

    let k = (p / 100.0) * (n - 1) as f64;
    let f = k.floor() as usize;
    let c = k.ceil() as usize;

    if f == c {
        sorted_values[f]
    } else {
        let d = k - f as f64;
        sorted_values[f] * (1.0 - d) + sorted_values[c] * d
    }
}

// =============================================================================
// outliers_iqr(array, multiplier?) -> array
// =============================================================================

defn!(OutliersIqrFn, vec![arg!(array)], Some(arg!(number)));

impl Function for OutliersIqrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let multiplier = if args.len() > 1 {
            args[1].as_f64().unwrap_or(1.5)
        } else {
            1.5
        };

        let mut values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let q1 = percentile_value(&values, 25.0);
        let q3 = percentile_value(&values, 75.0);
        let iqr = q3 - q1;

        let lower_bound = q1 - multiplier * iqr;
        let upper_bound = q3 + multiplier * iqr;

        let outliers: Vec<Value> = arr
            .iter()
            .filter(|v| {
                if let Some(n) = v.as_f64() {
                    n < lower_bound || n > upper_bound
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(Value::Array(outliers))
    }
}

// =============================================================================
// outliers_zscore(array, threshold?) -> array
// =============================================================================

defn!(OutliersZscoreFn, vec![arg!(array)], Some(arg!(number)));

impl Function for OutliersZscoreFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let threshold = if args.len() > 1 {
            args[1].as_f64().unwrap_or(2.0)
        } else {
            2.0
        };

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let n = values.len() as f64;
        let mean: f64 = values.iter().sum::<f64>() / n;
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        if stddev.abs() < f64::EPSILON {
            return Ok(Value::Array(vec![]));
        }

        let outliers: Vec<Value> = arr
            .iter()
            .filter(|v| {
                if let Some(n) = v.as_f64() {
                    let z = (n - mean) / stddev;
                    z.abs() > threshold
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(Value::Array(outliers))
    }
}

// =============================================================================
// trend(array) -> string ("increasing", "decreasing", "stable")
// =============================================================================

defn!(TrendFn, vec![arg!(array)], None);

impl Function for TrendFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.len() < 2 {
            return Ok(Value::String("stable".to_string()));
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.len() < 2 {
            return Ok(Value::String("stable".to_string()));
        }

        // Calculate linear regression slope
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        // Determine trend based on slope relative to data magnitude
        let mean: f64 = sum_y / n;
        let threshold = mean.abs() * 0.01;

        let trend = if slope > threshold {
            "increasing"
        } else if slope < -threshold {
            "decreasing"
        } else {
            "stable"
        };

        Ok(Value::String(trend.to_string()))
    }
}

// =============================================================================
// trend_slope(array) -> number (linear regression slope)
// =============================================================================

defn!(TrendSlopeFn, vec![arg!(array)], None);

impl Function for TrendSlopeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.len() < 2 {
            return Ok(number_value(0.0));
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.len() < 2 {
            return Ok(number_value(0.0));
        }

        // Calculate linear regression slope
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        Ok(number_value(slope))
    }
}

// =============================================================================
// rate_of_change(array) -> array (percentage change between consecutive elements)
// =============================================================================

defn!(RateOfChangeFn, vec![arg!(array)], None);

impl Function for RateOfChangeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.len() < 2 {
            return Ok(Value::Array(vec![]));
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        if values.len() < 2 {
            return Ok(Value::Array(vec![]));
        }

        let changes: Vec<Value> = values
            .windows(2)
            .map(|w| {
                let prev = w[0];
                let curr = w[1];
                let pct_change = if prev != 0.0 {
                    ((curr - prev) / prev) * 100.0
                } else {
                    0.0
                };
                number_value(pct_change)
            })
            .collect();

        Ok(Value::Array(changes))
    }
}

// =============================================================================
// cumulative_sum(array) -> array (running total)
// =============================================================================

defn!(CumulativeSumFn, vec![arg!(array)], None);

impl Function for CumulativeSumFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();

        let mut running_sum = 0.0;
        let cumsum: Vec<Value> = values
            .iter()
            .map(|v| {
                running_sum += v;
                number_value(running_sum)
            })
            .collect();

        Ok(Value::Array(cumsum))
    }
}
