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
        use rand::Rng;

        // Manual validation: accept 0 or 2 arguments
        if !args.is_empty() && args.len() != 2 {
            return Err(custom_error(ctx, "random() takes 0 or 2 arguments"));
        }

        let mut rng = rand::thread_rng();

        let value: f64 = if args.is_empty() {
            // random() - return 0.0 to 1.0
            rng.gen_range(0.0..1.0)
        } else {
            // random(min, max) - return min to max
            let min = args[0]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for min"))?;
            let max = args[1]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for max"))?;
            rng.gen_range(min..max)
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
        use rand::seq::SliceRandom;

        if args.len() != 1 {
            return Err(custom_error(ctx, "random_choice() takes 1 argument"));
        }

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let chosen = arr
            .choose(&mut rand::thread_rng())
            .cloned()
            .unwrap_or(Value::Null);

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

impl Function for RandomIntFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        use rand::Rng;

        if args.len() != 2 {
            return Err(custom_error(ctx, "random_int() takes 2 arguments"));
        }

        let min = args[0]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for min"))? as i64;
        let max = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for max"))? as i64;

        if min > max {
            return Err(custom_error(ctx, "min must be less than or equal to max"));
        }

        let value = rand::thread_rng().gen_range(min..=max);
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
            result.shuffle(&mut rand::thread_rng());
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
        use rand::seq::SliceRandom;

        let sample: Vec<Value> = if args.len() == 3 {
            // Deterministic sample with seed
            let seed = args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for seed"))?
                as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            arr.choose_multiple(&mut rng, n.min(arr.len()))
                .cloned()
                .collect()
        } else {
            // Random sample
            arr.choose_multiple(&mut rand::thread_rng(), n.min(arr.len()))
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
