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
    register_if_enabled(runtime, "shuffle", enabled, Box::new(ShuffleFn::new()));
    register_if_enabled(runtime, "sample", enabled, Box::new(SampleFn::new()));
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
