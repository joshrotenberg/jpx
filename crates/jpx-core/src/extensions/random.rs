//! Random value generation functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::{Function, custom_error, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, defn};

/// Register random functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "random", enabled, Box::new(RandomFn::new()));
    register_if_enabled(
        runtime,
        "random_choice",
        enabled,
        Box::new(RandomChoiceFn::new()),
    );
    register_if_enabled(runtime, "random_int", enabled, Box::new(RandomIntFn::new()));
    register_if_enabled(runtime, "sample", enabled, Box::new(SampleFn::new()));
    register_if_enabled(runtime, "shuffle", enabled, Box::new(ShuffleFn::new()));
    register_if_enabled(runtime, "uuid", enabled, Box::new(UuidFn::new()));
}

// =============================================================================
// random() -> number (0.0 to 1.0)
// random(min, max) -> number in range [min, max)
// =============================================================================

pub struct RandomFn;

impl Default for RandomFn {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomFn {
    pub fn new() -> RandomFn {
        RandomFn
    }
}

impl Function for RandomFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rand::RngExt;

        // Manual validation: accept 0 or 2 arguments
        if !args.is_empty() && args.len() != 2 {
            return Err(custom_error(ctx, "random() takes 0 or 2 arguments"));
        }

        let mut rng = rand::rng();

        let value: f64 = if args.is_empty() {
            // random() - return 0.0 to 1.0
            rng.random_range(0.0..1.0)
        } else {
            // random(min, max) - return min to max
            let min = args[0]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for min"))?;
            let max = args[1]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for max"))?;
            if min >= max {
                return Err(custom_error(
                    ctx,
                    "random() requires min < max. Use a minimum strictly less than the maximum.",
                ));
            }
            if !min.is_finite() || !max.is_finite() || !(max - min).is_finite() {
                return Err(custom_error(
                    ctx,
                    "random() requires finite bounds with a finite difference. Use a narrower range.",
                ));
            }
            // Floating-point sampling may round samples up to max. Reject
            // those values to preserve this function's half-open range.
            loop {
                let sample = rng.random_range(min..max);
                if sample < max {
                    break sample;
                }
            }
        };

        Ok(number_value(value))
    }
}

// =============================================================================
// random_choice(array) -> any (random element from array)
// =============================================================================

pub struct RandomChoiceFn;

impl Default for RandomChoiceFn {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomChoiceFn {
    pub fn new() -> RandomChoiceFn {
        RandomChoiceFn
    }
}

impl Function for RandomChoiceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rand::seq::IndexedRandom;

        if args.len() != 1 {
            return Err(custom_error(ctx, "random_choice() takes 1 argument"));
        }

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let chosen = arr.choose(&mut rand::rng()).cloned().unwrap_or(Value::Null);

        Ok(chosen)
    }
}

// =============================================================================
// random_int(min, max) -> number (random integer in [min, max])
// =============================================================================

pub struct RandomIntFn;

impl Default for RandomIntFn {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomIntFn {
    pub fn new() -> RandomIntFn {
        RandomIntFn
    }
}

fn random_int_bound(value: &Value) -> Option<i64> {
    let number = value.as_number()?;
    if let Some(integer) = number.as_i64() {
        return Some(integer);
    }
    // Do not round an unsupported u64 through f64 or saturate it to i64::MAX.
    if !number.is_f64() {
        return None;
    }
    let float = number.as_f64()?;
    // i64::MAX rounds up to 2^63 in f64, so the upper bound must be exclusive.
    let limit = -(i64::MIN as f64);
    (float.fract() == 0.0 && (-limit..limit).contains(&float)).then_some(float as i64)
}

impl Function for RandomIntFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rand::RngExt;

        if args.len() != 2 {
            return Err(custom_error(ctx, "random_int() takes 2 arguments"));
        }

        let parse_bound = |value, name| {
            random_int_bound(value).ok_or_else(|| {
                custom_error(
                    ctx,
                    &format!(
                        "random_int() {name} must be an integer in [-9223372036854775808, 9223372036854775807]; use an integral bound within this range"
                    ),
                )
            })
        };
        let min = parse_bound(&args[0], "min")?;
        let max = parse_bound(&args[1], "max")?;

        if min > max {
            return Err(custom_error(ctx, "min must be less than or equal to max"));
        }

        let value = rand::rng().random_range(min..=max);
        Ok(Value::Number(serde_json::Number::from(value)))
    }
}

// =============================================================================
// shuffle(array) -> array (randomly shuffled)
// shuffle(array, seed) -> array (deterministically shuffled)
// =============================================================================

pub struct ShuffleFn;

impl Default for ShuffleFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ShuffleFn {
    pub fn new() -> ShuffleFn {
        ShuffleFn
    }
}

impl Function for ShuffleFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        // Manual validation: 1 or 2 arguments
        if args.is_empty() || args.len() > 2 {
            return Err(custom_error(ctx, "shuffle() takes 1 or 2 arguments"));
        }

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let mut result: Vec<Value> = arr.clone();

        if args.len() == 2 {
            // Deterministic shuffle with seed
            let seed = args[1]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for seed"))?
                as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            result.shuffle(&mut rng);
        } else {
            // Random shuffle
            result.shuffle(&mut rand::rng());
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// sample(array, n) -> array (random sample of n elements)
// sample(array, n, seed) -> array (deterministic sample)
// =============================================================================

pub struct SampleFn;

impl Default for SampleFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleFn {
    pub fn new() -> SampleFn {
        SampleFn
    }
}

impl Function for SampleFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        // Manual validation: 2 or 3 arguments
        if args.len() < 2 || args.len() > 3 {
            return Err(custom_error(ctx, "sample() takes 2 or 3 arguments"));
        }

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number argument"))? as usize;

        use rand::SeedableRng;
        use rand::seq::IndexedRandom;

        let sample: Vec<Value> = if args.len() == 3 {
            // Deterministic sample with seed
            let seed = args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for seed"))?
                as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            arr.sample(&mut rng, n.min(arr.len())).cloned().collect()
        } else {
            // Random sample
            arr.sample(&mut rand::rng(), n.min(arr.len()))
                .cloned()
                .collect()
        };

        Ok(Value::Array(sample))
    }
}

// =============================================================================
// uuid() -> string (UUID v4)
// =============================================================================

defn!(UuidFn, vec![], None);

impl Function for UuidFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let id = uuid::Uuid::new_v4();
        Ok(Value::String(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::{Value, json};

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_random() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random()").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let value = result.as_f64().unwrap();
        assert!((0.0..1.0).contains(&value));
    }

    #[test]
    fn test_random_custom_range() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random(`-2.5`, `-0.5`)").unwrap();
        for _ in 0..64 {
            let result = expr.search(&Value::Null).unwrap();
            assert!((-2.5..-0.5).contains(&result.as_f64().unwrap()));
        }
    }

    #[test]
    fn test_random_excludes_adjacent_upper_bound() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random(min, max)").unwrap();
        for min in [1.0_f64, -1.0, 1e300, -1e300] {
            let data = json!({"min": min, "max": min.next_up()});
            // The only representable value in [min, min.next_up()) is min.
            for _ in 0..128 {
                let result = expr.search(&data).unwrap();
                assert_eq!(result.as_f64(), Some(min));
            }
        }
    }

    #[test]
    fn test_random_rejects_empty_or_reversed_range() {
        let runtime = setup_runtime();
        for query in [
            "random(`1`, `1`)",
            "random(`2`, `1`)",
            "random(`-0.0`, `0.0`)",
        ] {
            let error = runtime
                .compile(query)
                .unwrap()
                .search(&Value::Null)
                .unwrap_err();
            assert!(error.to_string().contains("Use a minimum strictly less"));
        }
    }

    #[test]
    fn test_random_rejects_overflowing_range() {
        let runtime = setup_runtime();
        let error = runtime
            .compile("random(`-1e308`, `1e308`)")
            .unwrap()
            .search(&Value::Null)
            .unwrap_err();
        assert!(error.to_string().contains("Use a narrower range"));
    }

    #[test]
    fn test_random_choice() {
        let runtime = setup_runtime();
        let data = json!(["a", "b", "c"]);
        let expr = runtime.compile("random_choice(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_string());
        let s = result.as_str().unwrap();
        assert!(["a", "b", "c"].contains(&s));
    }

    #[test]
    fn test_random_choice_single_element() {
        let runtime = setup_runtime();
        let data = json!([42]);
        let expr = runtime.compile("random_choice(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_random_choice_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("random_choice(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_random_int() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(`1`, `10`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let value = result.as_i64().unwrap();
        assert!((1..=10).contains(&value));
    }

    #[test]
    fn test_random_int_min_equals_max() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(`5`, `5`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result, json!(5));
    }

    #[test]
    fn test_random_int_preserves_exact_singleton_bounds() {
        let runtime = setup_runtime();
        let data_expr = runtime.compile("random_int(min, max)").unwrap();
        for bound in [
            i64::MIN,
            i64::MIN + 1,
            -9_007_199_254_740_993,
            -9_007_199_254_740_991,
            0,
            9_007_199_254_740_991,
            9_007_199_254_740_993,
            i64::MAX - 1,
            i64::MAX,
        ] {
            let data = json!({ "min": bound, "max": bound });
            assert_eq!(data_expr.search(&data).unwrap(), json!(bound));
            let literal_expr = runtime
                .compile(&format!("random_int(`{bound}`, `{bound}`)"))
                .unwrap();
            assert_eq!(literal_expr.search(&Value::Null).unwrap(), json!(bound));
        }
    }

    #[test]
    fn test_random_int_preserves_exact_narrow_ranges() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(min, max)").unwrap();
        for (min, max) in [
            (i64::MIN + 1, i64::MIN + 2),
            (-9_007_199_254_740_994, -9_007_199_254_740_993),
            (9_007_199_254_740_993, 9_007_199_254_740_994),
            (i64::MAX - 2, i64::MAX - 1),
            (i64::MIN, i64::MAX),
        ] {
            let data = json!({ "min": min, "max": max });
            for _ in 0..128 {
                let result = expr.search(&data).unwrap().as_i64().unwrap();
                assert!(
                    (min..=max).contains(&result),
                    "{result} outside [{min}, {max}]"
                );
            }
        }
    }

    #[test]
    fn test_random_int_accepts_integral_floats() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(min, max)").unwrap();
        for (bound, expected) in [
            (-0.0, 0),
            (1.0, 1),
            (-1e3, -1000),
            (9_007_199_254_740_994.0, 9_007_199_254_740_994),
            (i64::MIN as f64, i64::MIN),
            (9_223_372_036_854_774_784.0, i64::MAX - 1023),
        ] {
            let data = json!({ "min": bound, "max": bound });
            assert_eq!(expr.search(&data).unwrap(), json!(expected));
        }
        let literal_expr = runtime.compile("random_int(`1.0`, `1e0`)").unwrap();
        assert_eq!(literal_expr.search(&Value::Null).unwrap(), json!(1));
    }

    #[test]
    fn test_random_int_rejects_unsupported_bounds() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(min, max)").unwrap();
        for invalid in [
            json!(1.5),
            json!(-1.5),
            json!(9_223_372_036_854_775_808_u64),
            json!(u64::MAX),
            json!(9_223_372_036_854_775_808.0),
            json!(-9_223_372_036_854_777_856.0),
            json!(1e30),
            json!(-1e30),
            json!("1"),
            json!(null),
        ] {
            for name in ["min", "max"] {
                let mut data = json!({ "min": 0, "max": 1 });
                data[name] = invalid.clone();
                let error = expr.search(&data).unwrap_err().to_string();
                assert!(
                    error.contains(&format!("random_int() {name} must be an integer")),
                    "{error}"
                );
                assert!(
                    error.contains("[-9223372036854775808, 9223372036854775807]"),
                    "{error}"
                );
            }
        }
    }

    #[test]
    fn test_random_int_min_greater_than_max() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random_int(`10`, `1`)").unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    #[test]
    fn test_shuffle() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("shuffle(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    fn assert_sample_without_replacement(input: &Value, output: &Value, count: usize) {
        let mut remaining = input.as_array().unwrap().clone();
        let output = output.as_array().unwrap();
        assert_eq!(output.len(), count.min(remaining.len()));
        for value in output {
            let index = remaining
                .iter()
                .position(|candidate| candidate == value)
                .expect("every sampled value must consume an element from the input");
            remaining.remove(index);
        }
    }

    #[test]
    fn test_shuffle_preserves_elements() {
        let runtime = setup_runtime();
        for query in ["shuffle(@)", "shuffle(@, `42`)"] {
            let expr = runtime.compile(query).unwrap();
            for data in [
                json!([]),
                json!([null]),
                json!([1, 1, "two", null, {"three": 3}, [4]]),
            ] {
                let result = expr.search(&data).unwrap();
                assert_sample_without_replacement(&data, &result, data.as_array().unwrap().len());
            }
        }
    }

    #[test]
    fn test_sample_count_and_elements() {
        let runtime = setup_runtime();
        for count in [0, 2, 99] {
            for seed in ["", ", `42`"] {
                let expr = runtime
                    .compile(&format!("sample(@, `{count}`{seed})"))
                    .unwrap();
                for data in [
                    json!([]),
                    json!([null]),
                    json!([1, 1, "two", null, {"three": 3}, [4]]),
                ] {
                    let result = expr.search(&data).unwrap();
                    assert_sample_without_replacement(&data, &result, count);
                }
            }
        }
    }

    #[test]
    fn test_seeded_operations_are_repeatable() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5, 6]);
        for query in ["shuffle(@, `42`)", "sample(@, `3`, `42`)"] {
            let expr = runtime.compile(query).unwrap();
            assert_eq!(expr.search(&data).unwrap(), expr.search(&data).unwrap());
        }
    }

    #[test]
    fn test_uuid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("uuid()").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let uuid_str = result.as_str().unwrap();
        // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        assert_eq!(uuid_str.len(), 36);
    }
}
