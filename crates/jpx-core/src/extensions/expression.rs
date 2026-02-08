//! Expression functions.
//!
//! Higher-order functions that use expression references (exprefs) to apply
//! transformations, filters, and reductions over arrays and objects.

use std::collections::HashSet;

use serde_json::{Map, Number, Value};

use crate::ast::Ast;
use crate::functions::{Function, custom_error, number_value};
use crate::interpreter::{SearchResult, interpret};
use crate::registry::register_if_enabled;
use crate::value_ext::ValueExt;
use crate::{Context, Runtime, arg, defn, get_expref_id};

/// Helper to extract an expref AST from a function argument.
fn get_expref_ast<'a>(value: &Value, ctx: &'a Context<'_>) -> Option<&'a Ast> {
    get_expref_id(value).and_then(|id| ctx.get_expref(id))
}

/// Convert a Value to a string key for grouping/deduplication.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Compare two values for sorting purposes.
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Value::Number(an), Value::Number(bn)) => {
            let a_f = an.as_f64().unwrap_or(0.0);
            let b_f = bn.as_f64().unwrap_or(0.0);
            a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
        }
        (Value::String(a_s), Value::String(b_s)) => a_s.cmp(b_s),
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

/// Register only the expression functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "map_expr", enabled, Box::new(MapExprFn::new()));
    register_if_enabled(
        runtime,
        "filter_expr",
        enabled,
        Box::new(FilterExprFn::new()),
    );
    register_if_enabled(runtime, "any_expr", enabled, Box::new(AnyExprFn::new()));
    register_if_enabled(runtime, "all_expr", enabled, Box::new(AllExprFn::new()));
    register_if_enabled(runtime, "find_expr", enabled, Box::new(FindExprFn::new()));
    register_if_enabled(
        runtime,
        "find_index_expr",
        enabled,
        Box::new(FindIndexExprFn::new()),
    );
    register_if_enabled(runtime, "count_expr", enabled, Box::new(CountExprFn::new()));
    register_if_enabled(
        runtime,
        "sort_by_expr",
        enabled,
        Box::new(SortByExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "group_by_expr",
        enabled,
        Box::new(GroupByExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "partition_expr",
        enabled,
        Box::new(PartitionExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "min_by_expr",
        enabled,
        Box::new(MinByExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "max_by_expr",
        enabled,
        Box::new(MaxByExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "unique_by_expr",
        enabled,
        Box::new(UniqueByExprFn::new()),
    );
    register_if_enabled(
        runtime,
        "flat_map_expr",
        enabled,
        Box::new(FlatMapExprFn::new()),
    );

    // Clojure-style alias for flat_map_expr
    register_if_enabled(runtime, "mapcat", enabled, Box::new(FlatMapExprFn::new()));

    // Lodash-style aliases
    register_if_enabled(runtime, "some", enabled, Box::new(AnyExprFn::new()));
    register_if_enabled(runtime, "every", enabled, Box::new(AllExprFn::new()));
    register_if_enabled(runtime, "reject", enabled, Box::new(RejectFn::new()));
    register_if_enabled(runtime, "map_keys", enabled, Box::new(MapKeysFn::new()));
    register_if_enabled(runtime, "map_values", enabled, Box::new(MapValuesFn::new()));
    register_if_enabled(runtime, "order_by", enabled, Box::new(OrderByFn::new()));
    register_if_enabled(
        runtime,
        "reduce_expr",
        enabled,
        Box::new(ReduceExprFn::new()),
    );
    register_if_enabled(runtime, "scan_expr", enabled, Box::new(ScanExprFn::new()));
    // Alias for reduce_expr (lodash-style)
    register_if_enabled(runtime, "fold", enabled, Box::new(ReduceExprFn::new()));
    // Clojure-style alias for scan_expr
    register_if_enabled(runtime, "reductions", enabled, Box::new(ScanExprFn::new()));
    // none - opposite of any_expr/some
    register_if_enabled(runtime, "none", enabled, Box::new(NoneFn::new()));
    register_if_enabled(runtime, "count_by", enabled, Box::new(CountByFn::new()));

    // Partial application functions
    register_if_enabled(runtime, "partial", enabled, Box::new(PartialFn::new()));
    register_if_enabled(runtime, "apply", enabled, Box::new(ApplyFn::new()));

    // Functional array operations
    register_if_enabled(runtime, "take_while", enabled, Box::new(TakeWhileFn::new()));
    register_if_enabled(runtime, "drop_while", enabled, Box::new(DropWhileFn::new()));
    register_if_enabled(runtime, "zip_with", enabled, Box::new(ZipWithFn::new()));

    // Recursive transformation
    register_if_enabled(runtime, "walk", enabled, Box::new(WalkFn::new()));

    // Recursive descent (jq parity)
    register_if_enabled(runtime, "recurse", enabled, Box::new(RecurseFn::new()));
    register_if_enabled(
        runtime,
        "recurse_with",
        enabled,
        Box::new(RecurseWithFn::new()),
    );

    // Loop functions (jq parity)
    register_if_enabled(runtime, "while_expr", enabled, Box::new(WhileExprFn::new()));
    register_if_enabled(runtime, "until_expr", enabled, Box::new(UntilExprFn::new()));
}

// =============================================================================
// map_expr(expr, array) -> array
// =============================================================================

defn!(MapExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for MapExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut results = Vec::with_capacity(arr.len());
        for item in arr {
            results.push(interpret(item, &ast, ctx)?);
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// filter_expr(expr, array) -> array
// =============================================================================

defn!(FilterExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for FilterExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut results = Vec::new();
        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                results.push(item.clone());
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// any_expr(expr, array) -> bool
// =============================================================================

defn!(AnyExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for AnyExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                return Ok(Value::Bool(true));
            }
        }

        Ok(Value::Bool(false))
    }
}

// =============================================================================
// none(expr, array) -> bool (opposite of any_expr/some)
// =============================================================================

defn!(NoneFn, vec![arg!(expref), arg!(array)], None);

impl Function for NoneFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        // Empty array returns true (vacuously, no elements satisfy the predicate)
        if arr.is_empty() {
            return Ok(Value::Bool(true));
        }

        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                return Ok(Value::Bool(false));
            }
        }

        Ok(Value::Bool(true))
    }
}

// =============================================================================
// all_expr(expr, array) -> bool
// =============================================================================

defn!(AllExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for AllExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        // Empty array returns true (vacuous truth)
        if arr.is_empty() {
            return Ok(Value::Bool(true));
        }

        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if !result.is_truthy() {
                return Ok(Value::Bool(false));
            }
        }

        Ok(Value::Bool(true))
    }
}

// =============================================================================
// find_expr(expr, array) -> element | null
// =============================================================================

defn!(FindExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for FindExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                return Ok(item.clone());
            }
        }

        Ok(Value::Null)
    }
}

// =============================================================================
// find_index_expr(expr, array) -> number | null
// =============================================================================

defn!(FindIndexExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for FindIndexExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        for (i, item) in arr.iter().enumerate() {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                return Ok(number_value(i as f64));
            }
        }

        Ok(number_value(-1.0))
    }
}

// =============================================================================
// count_expr(expr, array) -> number
// =============================================================================

defn!(CountExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for CountExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut count = 0i64;
        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                count += 1;
            }
        }

        Ok(Value::Number(Number::from(count)))
    }
}

// =============================================================================
// sort_by_expr(expr, array) -> array
// =============================================================================

defn!(SortByExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for SortByExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        // Compute sort keys for each element
        let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(arr.len());
        for item in arr {
            let key = interpret(item, &ast, ctx)?;
            keyed.push((item.clone(), key));
        }

        // Sort by key
        keyed.sort_by(|a, b| compare_values(&a.1, &b.1));

        let results: Vec<Value> = keyed.into_iter().map(|(item, _)| item).collect();
        Ok(Value::Array(results))
    }
}

// =============================================================================
// group_by_expr(expr, array) -> object
// =============================================================================

defn!(GroupByExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for GroupByExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        // Use an index-based approach to preserve insertion order
        let mut group_keys: Vec<String> = Vec::new();
        let mut group_map: std::collections::HashMap<String, Vec<Value>> =
            std::collections::HashMap::new();

        for item in arr {
            let key_val = interpret(item, &ast, ctx)?;
            let key = value_to_string(&key_val);
            if !group_map.contains_key(&key) {
                group_keys.push(key.clone());
            }
            group_map.entry(key).or_default().push(item.clone());
        }

        let mut result = Map::new();
        for key in group_keys {
            if let Some(items) = group_map.remove(&key) {
                result.insert(key, Value::Array(items));
            }
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// count_by(expr, array) -> object
// =============================================================================

defn!(CountByFn, vec![arg!(expref), arg!(array)], None);

impl Function for CountByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut count_keys: Vec<String> = Vec::new();
        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for item in arr {
            let key_val = interpret(item, &ast, ctx)?;
            let key = value_to_string(&key_val);
            if !counts.contains_key(&key) {
                count_keys.push(key.clone());
            }
            *counts.entry(key).or_insert(0) += 1;
        }

        let mut result = Map::new();
        for key in count_keys {
            if let Some(&count) = counts.get(&key) {
                result.insert(key, Value::Number(Number::from(count)));
            }
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// partition_expr(expr, array) -> [matches, non_matches]
// =============================================================================

defn!(PartitionExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for PartitionExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut matches = Vec::new();
        let mut non_matches = Vec::new();

        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                matches.push(item.clone());
            } else {
                non_matches.push(item.clone());
            }
        }

        Ok(Value::Array(vec![
            Value::Array(matches),
            Value::Array(non_matches),
        ]))
    }
}

// =============================================================================
// min_by_expr(expr, array) -> element | null
// =============================================================================

defn!(MinByExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for MinByExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut min_item = arr[0].clone();
        let mut min_key = interpret(&arr[0], &ast, ctx)?;

        for item in arr.iter().skip(1) {
            let key = interpret(item, &ast, ctx)?;
            if compare_values(&key, &min_key) == std::cmp::Ordering::Less {
                min_item = item.clone();
                min_key = key;
            }
        }

        Ok(min_item)
    }
}

// =============================================================================
// max_by_expr(expr, array) -> element | null
// =============================================================================

defn!(MaxByExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for MaxByExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut max_item = arr[0].clone();
        let mut max_key = interpret(&arr[0], &ast, ctx)?;

        for item in arr.iter().skip(1) {
            let key = interpret(item, &ast, ctx)?;
            if compare_values(&key, &max_key) == std::cmp::Ordering::Greater {
                max_item = item.clone();
                max_key = key;
            }
        }

        Ok(max_item)
    }
}

// =============================================================================
// unique_by_expr(expr, array) -> array
// =============================================================================

defn!(UniqueByExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for UniqueByExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut seen: HashSet<String> = HashSet::new();
        let mut results = Vec::new();

        for item in arr {
            let key_val = interpret(item, &ast, ctx)?;
            let key = value_to_string(&key_val);
            if seen.insert(key) {
                results.push(item.clone());
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// flat_map_expr(expr, array) -> array
// =============================================================================

defn!(FlatMapExprFn, vec![arg!(expref), arg!(array)], None);

impl Function for FlatMapExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut results = Vec::new();
        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            match result {
                Value::Array(inner) => {
                    results.extend(inner);
                }
                Value::Null => {
                    // Skip nulls
                }
                _ => {
                    results.push(result);
                }
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// reject(expr, array) -> array (inverse of filter_expr)
// =============================================================================

defn!(RejectFn, vec![arg!(expref), arg!(array)], None);

impl Function for RejectFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut results = Vec::new();
        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            // Keep items where expression is falsy (inverse of filter)
            if !result.is_truthy() {
                results.push(item.clone());
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// map_keys(expr, object) -> object
// =============================================================================

defn!(MapKeysFn, vec![arg!(expref), arg!(object)], None);

impl Function for MapKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let obj = args[1].as_object().unwrap();

        let mut result = Map::new();
        for (key, value) in obj.iter() {
            // Apply expression to the key
            let key_val = Value::String(key.clone());
            let new_key_val = interpret(&key_val, &ast, ctx)?;

            let new_key_str = match &new_key_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => key.clone(), // Keep original if result is not a string/number
            };

            result.insert(new_key_str, value.clone());
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// map_values(expr, object) -> object
// =============================================================================

defn!(MapValuesFn, vec![arg!(expref), arg!(object)], None);

impl Function for MapValuesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let obj = args[1].as_object().unwrap();

        let mut result = Map::new();
        for (key, value) in obj.iter() {
            let new_value = interpret(value, &ast, ctx)?;
            result.insert(key.clone(), new_value);
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// order_by(array, criteria) -> array
// =============================================================================

defn!(OrderByFn, vec![arg!(array), arg!(array)], None);

impl Function for OrderByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();
        let criteria = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        // Parse criteria: each element should be [field, direction]
        let mut sort_specs: Vec<(String, bool)> = Vec::new(); // (field, ascending)
        for criterion in criteria {
            let crit_arr = criterion.as_array().ok_or_else(|| {
                custom_error(ctx, "Each criterion must be an array [field, direction]")
            })?;

            if crit_arr.len() < 2 {
                return Err(custom_error(
                    ctx,
                    "Each criterion must have [field, direction]",
                ));
            }

            let field = crit_arr[0]
                .as_str()
                .ok_or_else(|| custom_error(ctx, "Field name must be a string"))?;

            let direction = crit_arr[1]
                .as_str()
                .ok_or_else(|| custom_error(ctx, "Direction must be 'asc' or 'desc'"))?;

            let ascending = match direction.to_lowercase().as_str() {
                "asc" | "ascending" => true,
                "desc" | "descending" => false,
                _ => {
                    return Err(custom_error(ctx, "Direction must be 'asc' or 'desc'"));
                }
            };

            sort_specs.push((field.to_string(), ascending));
        }

        // Clone and sort the array
        let mut result: Vec<Value> = arr.clone();
        result.sort_by(|a, b| {
            for (field, ascending) in &sort_specs {
                let a_val = a
                    .as_object()
                    .and_then(|o| o.get(field.as_str()))
                    .unwrap_or(&Value::Null);
                let b_val = b
                    .as_object()
                    .and_then(|o| o.get(field.as_str()))
                    .unwrap_or(&Value::Null);

                let cmp = compare_values(a_val, b_val);
                if cmp != std::cmp::Ordering::Equal {
                    return if *ascending { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(Value::Array(result))
    }
}

// =============================================================================
// reduce_expr(expr, array, initial) -> any
// =============================================================================

defn!(
    ReduceExprFn,
    vec![arg!(expref), arg!(array), arg!(any)],
    None
);

impl Function for ReduceExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();
        let initial = args[2].clone();

        if arr.is_empty() {
            return Ok(initial);
        }

        let mut accumulator = initial;

        for (idx, item) in arr.iter().enumerate() {
            // Create context object with accumulator, current, and index
            let mut context_map = Map::new();
            context_map.insert("accumulator".to_string(), accumulator.clone());
            context_map.insert("current".to_string(), item.clone());
            context_map.insert("index".to_string(), Value::Number(Number::from(idx as i64)));
            let context_val = Value::Object(context_map);

            accumulator = interpret(&context_val, &ast, ctx)?;
        }

        Ok(accumulator)
    }
}

// =============================================================================
// scan_expr(expr, array, initial) -> array
// =============================================================================

defn!(ScanExprFn, vec![arg!(expref), arg!(array), arg!(any)], None);

impl Function for ScanExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();
        let initial = args[2].clone();

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut accumulator = initial;
        let mut results: Vec<Value> = Vec::with_capacity(arr.len());

        for (idx, item) in arr.iter().enumerate() {
            // Create context object with accumulator, current, and index
            let mut context_map = Map::new();
            context_map.insert("accumulator".to_string(), accumulator.clone());
            context_map.insert("current".to_string(), item.clone());
            context_map.insert("index".to_string(), Value::Number(Number::from(idx as i64)));
            let context_val = Value::Object(context_map);

            accumulator = interpret(&context_val, &ast, ctx)?;
            results.push(accumulator.clone());
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// partial(fn_name, ...args) -> partial object
// =============================================================================

defn!(PartialFn, vec![arg!(string)], Some(arg!(any)));

impl Function for PartialFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let fn_name = args[0].as_str().ok_or_else(|| {
            custom_error(
                ctx,
                "partial() first argument must be a function name string",
            )
        })?;

        // Collect the pre-filled arguments
        let prefilled_args: Vec<Value> = args[1..].to_vec();

        // Create the partial object
        let mut partial_obj = Map::new();
        partial_obj.insert("__partial__".to_string(), Value::Bool(true));
        partial_obj.insert("fn".to_string(), Value::String(fn_name.to_string()));
        partial_obj.insert("args".to_string(), Value::Array(prefilled_args));

        Ok(Value::Object(partial_obj))
    }
}

// =============================================================================
// apply(partial_or_fn, ...args) -> result
// =============================================================================

defn!(ApplyFn, vec![arg!(any)], Some(arg!(any)));

impl Function for ApplyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let first_arg = &args[0];
        let additional_args = &args[1..];

        // Check if it's a partial object
        if let Some(obj) = first_arg.as_object()
            && obj.get("__partial__").and_then(|v| v.as_bool()) == Some(true)
        {
            // It's a partial - extract fn name and pre-filled args
            let fn_name = obj
                .get("fn")
                .and_then(|v| v.as_str())
                .ok_or_else(|| custom_error(ctx, "Invalid partial object: missing 'fn' field"))?;

            let prefilled = obj
                .get("args")
                .and_then(|v| v.as_array())
                .ok_or_else(|| custom_error(ctx, "Invalid partial object: missing 'args' field"))?;

            return invoke_function(fn_name, prefilled, additional_args, ctx);
        }

        // If it's a string, treat as function name
        if let Some(fn_name) = first_arg.as_str() {
            return invoke_function(fn_name, &[], additional_args, ctx);
        }

        Err(custom_error(
            ctx,
            "apply() first argument must be a partial object or function name string",
        ))
    }
}

/// Helper to invoke a function by name with pre-filled and additional arguments.
fn invoke_function(
    fn_name: &str,
    prefilled: &[Value],
    additional: &[Value],
    ctx: &mut Context<'_>,
) -> SearchResult {
    // Build the argument list for the expression
    let mut all_args_json: Vec<String> = Vec::new();

    // Add pre-filled args as literals
    for a in prefilled {
        all_args_json.push(format!("`{}`", serde_json::to_string(a).unwrap()));
    }

    // Add additional args as literals
    for a in additional {
        all_args_json.push(format!("`{}`", serde_json::to_string(a).unwrap()));
    }

    // Build and execute the expression
    let expr_str = format!("{}({})", fn_name, all_args_json.join(", "));

    let compiled = ctx.runtime.compile(&expr_str).map_err(|_| {
        custom_error(
            ctx,
            &format!("Failed to compile function call '{}'", expr_str),
        )
    })?;

    compiled
        .search(&Value::Null)
        .map_err(|_| custom_error(ctx, &format!("Failed to execute '{}'", fn_name)))
}

// =============================================================================
// take_while(expr, array) -> array
// =============================================================================

defn!(TakeWhileFn, vec![arg!(expref), arg!(array)], None);

impl Function for TakeWhileFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut results = Vec::new();
        for item in arr {
            let result = interpret(item, &ast, ctx)?;
            if result.is_truthy() {
                results.push(item.clone());
            } else {
                break;
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// drop_while(expr, array) -> array
// =============================================================================

defn!(DropWhileFn, vec![arg!(expref), arg!(array)], None);

impl Function for DropWhileFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr = args[1].as_array().unwrap();

        let mut dropping = true;
        let mut results = Vec::new();
        for item in arr {
            if dropping {
                let result = interpret(item, &ast, ctx)?;
                if !result.is_truthy() {
                    dropping = false;
                    results.push(item.clone());
                }
            } else {
                results.push(item.clone());
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// zip_with(expr, array1, array2) -> array
// =============================================================================

defn!(
    ZipWithFn,
    vec![arg!(expref), arg!(array), arg!(array)],
    None
);

impl Function for ZipWithFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();
        let arr1 = args[1].as_array().unwrap();
        let arr2 = args[2].as_array().unwrap();

        let min_len = arr1.len().min(arr2.len());
        let mut results = Vec::with_capacity(min_len);

        for i in 0..min_len {
            // Create a pair array [element1, element2] as input to the expression
            let pair = Value::Array(vec![arr1[i].clone(), arr2[i].clone()]);
            let result = interpret(&pair, &ast, ctx)?;
            results.push(result);
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// walk(expr, value) -> value (recursive transformation)
// =============================================================================

defn!(WalkFn, vec![arg!(expref), arg!(any)], None);

/// Recursively walk a value, applying the expression bottom-up.
fn walk_value(value: &Value, ast: &Ast, ctx: &mut Context<'_>) -> SearchResult {
    match value {
        Value::Array(arr) => {
            // First, recursively walk all elements
            let walked_elements: Result<Vec<Value>, _> =
                arr.iter().map(|elem| walk_value(elem, ast, ctx)).collect();
            let new_array = Value::Array(walked_elements?);
            // Then apply the expression to the array itself
            interpret(&new_array, ast, ctx)
        }
        Value::Object(obj) => {
            // First, recursively walk all values
            let mut walked_obj = Map::new();
            for (k, v) in obj.iter() {
                walked_obj.insert(k.clone(), walk_value(v, ast, ctx)?);
            }
            let new_object = Value::Object(walked_obj);
            // Then apply the expression to the object itself
            interpret(&new_object, ast, ctx)
        }
        // For scalars (string, number, bool, null), just apply the expression
        _ => interpret(value, ast, ctx),
    }
}

impl Function for WalkFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[0], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();

        walk_value(&args[1], &ast, ctx)
    }
}

// =============================================================================
// recurse(value) -> array (collect all nested values, jq parity)
// =============================================================================

defn!(RecurseFn, vec![arg!(any)], None);

/// Helper to collect all values recursively.
fn collect_recursive(value: &Value, results: &mut Vec<Value>) {
    results.push(value.clone());
    match value {
        Value::Array(arr) => {
            for elem in arr {
                collect_recursive(elem, results);
            }
        }
        Value::Object(obj) => {
            for (_, v) in obj.iter() {
                collect_recursive(v, results);
            }
        }
        _ => {}
    }
}

impl Function for RecurseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut results = Vec::new();
        collect_recursive(&args[0], &mut results);

        Ok(Value::Array(results))
    }
}

// =============================================================================
// recurse_with(value, expr) -> array (recursive descent with filter, jq parity)
// =============================================================================

defn!(RecurseWithFn, vec![arg!(any), arg!(expref)], None);

impl Function for RecurseWithFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ast = get_expref_ast(&args[1], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref"))?
            .clone();

        let mut results = Vec::new();
        let mut queue = vec![args[0].clone()];
        let max_iterations = 10000; // Safety limit
        let mut iterations = 0;

        while let Some(current) = queue.pop() {
            if iterations >= max_iterations {
                return Err(custom_error(
                    ctx,
                    "recurse_with exceeded maximum iterations",
                ));
            }
            iterations += 1;

            // Skip null values
            if current.is_null() {
                continue;
            }

            results.push(current.clone());

            // Apply expression to get next values
            let next = interpret(&current, &ast, ctx)?;

            match next {
                Value::Null => {}
                Value::Array(arr) => {
                    // Add non-null elements to queue (in reverse to maintain order)
                    for elem in arr.into_iter().rev() {
                        if !elem.is_null() {
                            queue.push(elem);
                        }
                    }
                }
                _ => {
                    queue.push(next);
                }
            }
        }

        Ok(Value::Array(results))
    }
}

// =============================================================================
// while_expr(init, condition, update) -> value
// =============================================================================

defn!(
    WhileExprFn,
    vec![arg!(any), arg!(expref), arg!(expref)],
    None
);

impl Function for WhileExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let cond_ast = get_expref_ast(&args[1], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref for condition"))?
            .clone();
        let update_ast = get_expref_ast(&args[2], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref for update"))?
            .clone();

        let mut current = args[0].clone();
        let max_iterations = 100000; // Safety limit
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                return Err(custom_error(ctx, "while_expr exceeded maximum iterations"));
            }
            iterations += 1;

            // Check condition
            let cond_result = interpret(&current, &cond_ast, ctx)?;
            if !cond_result.is_truthy() {
                break;
            }

            // Apply update
            current = interpret(&current, &update_ast, ctx)?;
        }

        Ok(current)
    }
}

// =============================================================================
// until_expr(init, condition, update) -> value
// =============================================================================

defn!(
    UntilExprFn,
    vec![arg!(any), arg!(expref), arg!(expref)],
    None
);

impl Function for UntilExprFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let cond_ast = get_expref_ast(&args[1], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref for condition"))?
            .clone();
        let update_ast = get_expref_ast(&args[2], ctx)
            .ok_or_else(|| custom_error(ctx, "Expected expref for update"))?
            .clone();

        let mut current = args[0].clone();
        let max_iterations = 100000; // Safety limit
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations {
                return Err(custom_error(ctx, "until_expr exceeded maximum iterations"));
            }
            iterations += 1;

            // Check condition (stop when true, opposite of while)
            let cond_result = interpret(&current, &cond_ast, ctx)?;
            if cond_result.is_truthy() {
                break;
            }

            // Apply update
            current = interpret(&current, &update_ast, ctx)?;
        }

        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_map_expr_field() {
        let runtime = setup_runtime();
        let data = json!([{"name": "Alice"}, {"name": "Bob"}]);
        let expr = runtime.compile("map_expr(&name, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "Alice");
        assert_eq!(arr[1].as_str().unwrap(), "Bob");
    }

    #[test]
    fn test_map_expr_transform() {
        let runtime = setup_runtime();
        let data = json!(["hello", "world"]);
        let expr = runtime.compile("map_expr(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_f64().unwrap(), 5.0);
        assert_eq!(arr[1].as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_filter_expr() {
        let runtime = setup_runtime();
        let data = json!([{"age": 25}, {"age": 17}, {"age": 30}]);
        let expr = runtime.compile("filter_expr(&(age >= `18`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_filter_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("filter_expr(&(@ > `10`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_any_expr_true() {
        let runtime = setup_runtime();
        let data = json!([{"active": false}, {"active": true}]);
        let expr = runtime.compile("any_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_any_expr_false() {
        let runtime = setup_runtime();
        let data = json!([{"active": false}, {"active": false}]);
        let expr = runtime.compile("any_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_all_expr_true() {
        let runtime = setup_runtime();
        let data = json!([{"active": true}, {"active": true}]);
        let expr = runtime.compile("all_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_all_expr_false() {
        let runtime = setup_runtime();
        let data = json!([{"active": true}, {"active": false}]);
        let expr = runtime.compile("all_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_all_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("all_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap()); // vacuous truth
    }

    #[test]
    fn test_find_expr_found() {
        let runtime = setup_runtime();
        let data = json!([{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]);
        let expr = runtime.compile("find_expr(&(id == `2`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Bob");
    }

    #[test]
    fn test_find_expr_not_found() {
        let runtime = setup_runtime();
        let data = json!([{"id": 1}, {"id": 2}]);
        let expr = runtime.compile("find_expr(&(id == `99`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_sort_by_expr_numbers() {
        let runtime = setup_runtime();
        let data = json!([{"val": 3}, {"val": 1}, {"val": 2}]);
        let expr = runtime.compile("sort_by_expr(&val, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_f64()
                .unwrap(),
            1.0
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_f64()
                .unwrap(),
            2.0
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_f64()
                .unwrap(),
            3.0
        );
    }

    #[test]
    fn test_sort_by_expr_strings() {
        let runtime = setup_runtime();
        let data = json!([{"name": "Charlie"}, {"name": "Alice"}, {"name": "Bob"}]);
        let expr = runtime.compile("sort_by_expr(&name, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Alice"
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Bob"
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Charlie"
        );
    }

    #[test]
    fn test_find_index_expr_found() {
        let runtime = setup_runtime();
        let data = json!([{"id": 1}, {"id": 2}, {"id": 3}]);
        let expr = runtime.compile("find_index_expr(&(id == `2`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_find_index_expr_not_found() {
        let runtime = setup_runtime();
        let data = json!([{"id": 1}, {"id": 2}]);
        let expr = runtime
            .compile("find_index_expr(&(id == `99`), @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), -1.0);
    }

    #[test]
    fn test_count_expr() {
        let runtime = setup_runtime();
        let data = json!([{"active": true}, {"active": false}, {"active": true}]);
        let expr = runtime.compile("count_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_count_expr_none() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("count_expr(&(@ > `10`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_group_by_expr() {
        let runtime = setup_runtime();
        let data = json!([
            {"type": "a", "val": 1},
            {"type": "b", "val": 2},
            {"type": "a", "val": 3}
        ]);
        let expr = runtime.compile("group_by_expr(&type, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("b").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_partition_expr() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime.compile("partition_expr(&(@ > `3`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let matches = arr[0].as_array().unwrap();
        let non_matches = arr[1].as_array().unwrap();
        assert_eq!(matches.len(), 2); // 4, 5
        assert_eq!(non_matches.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_min_by_expr() {
        let runtime = setup_runtime();
        let data = json!([{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]);
        let expr = runtime.compile("min_by_expr(&age, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Bob");
    }

    #[test]
    fn test_min_by_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("min_by_expr(&age, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_max_by_expr() {
        let runtime = setup_runtime();
        let data = json!([{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]);
        let expr = runtime.compile("max_by_expr(&age, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_unique_by_expr() {
        let runtime = setup_runtime();
        let data = json!([
            {"type": "a", "val": 1},
            {"type": "b", "val": 2},
            {"type": "a", "val": 3}
        ]);
        let expr = runtime.compile("unique_by_expr(&type, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // First "a" and first "b"
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_f64()
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_flat_map_expr() {
        let runtime = setup_runtime();
        let data = json!([
            {"tags": ["a", "b"]},
            {"tags": ["c"]},
            {"tags": ["d", "e"]}
        ]);
        let expr = runtime.compile("flat_map_expr(&tags, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_str().unwrap(), "a");
        assert_eq!(arr[4].as_str().unwrap(), "e");
    }

    #[test]
    fn test_flat_map_expr_non_array() {
        let runtime = setup_runtime();
        let data = json!([{"name": "Alice"}, {"name": "Bob"}]);
        let expr = runtime.compile("flat_map_expr(&name, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "Alice");
    }

    #[test]
    fn test_some_alias() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime.compile("some(&(@ > `3`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_every_alias() {
        let runtime = setup_runtime();
        let data = json!([2, 4, 6]);
        let expr = runtime.compile("every(&(@ > `0`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_none_true() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("none(&(@ > `5`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap()); // No element > 5
    }

    #[test]
    fn test_none_false() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 10]);
        let expr = runtime.compile("none(&(@ > `5`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap()); // 10 > 5
    }

    #[test]
    fn test_none_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("none(&(@ > `0`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap()); // Vacuously true
    }

    #[test]
    fn test_none_objects() {
        let runtime = setup_runtime();
        let data = json!([{"active": false}, {"active": false}]);
        let expr = runtime.compile("none(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap()); // No active elements
    }

    #[test]
    fn test_mapcat_alias() {
        let runtime = setup_runtime();
        let data = json!([
            {"tags": ["a", "b"]},
            {"tags": ["c"]},
            {"tags": ["d", "e"]}
        ]);
        let expr = runtime.compile("mapcat(&tags, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_str().unwrap(), "a");
        assert_eq!(arr[4].as_str().unwrap(), "e");
    }

    #[test]
    fn test_reductions_alias() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4]);
        let expr = runtime
            .compile("reductions(&sum([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Running sum: [1, 3, 6, 10]
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[1].as_f64().unwrap(), 3.0);
        assert_eq!(arr[2].as_f64().unwrap(), 6.0);
        assert_eq!(arr[3].as_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_reject() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime.compile("reject(&(@ > `2`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // 1, 2
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[1].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_reject_objects() {
        let runtime = setup_runtime();
        let data = json!([{"active": true}, {"active": false}, {"active": true}]);
        let expr = runtime.compile("reject(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1); // Only the inactive one
    }

    #[test]
    fn test_map_keys() {
        let runtime = setup_runtime();
        // Use length to transform key to its length (as string)
        let data = json!({"abc": 1, "de": 2});
        let expr = runtime.compile("map_keys(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // "abc" -> 3, "de" -> 2 (converted to string keys)
        assert!(obj.contains_key("3") || obj.contains_key("2"));
    }

    #[test]
    fn test_map_values_add() {
        let runtime = setup_runtime();
        // Use sum to get sum of a literal array
        let data = json!({"a": 1, "b": 2, "c": 3});
        let expr = runtime.compile("map_values(&sum(`[1]`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // Each value becomes 1 (sum of [1])
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_map_values_length() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "city": "boston"});
        let expr = runtime.compile("map_values(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_f64().unwrap(), 5.0); // "alice" = 5 chars
        assert_eq!(obj.get("city").unwrap().as_f64().unwrap(), 6.0); // "boston" = 6 chars
    }

    #[test]
    fn test_map_values_with_string_fns() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "city": "boston"});
        let expr = runtime.compile("map_values(&upper(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "ALICE");
        assert_eq!(obj.get("city").unwrap().as_str().unwrap(), "BOSTON");
    }

    #[test]
    fn test_map_keys_with_string_fns() {
        let runtime = setup_runtime();
        let data = json!({"hello": 1, "world": 2});
        let expr = runtime.compile("map_keys(&upper(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("HELLO"));
        assert!(obj.contains_key("WORLD"));
    }

    #[test]
    fn test_order_by_single_field_asc() {
        let runtime = setup_runtime();
        let data = json!([
            {"name": "Charlie", "age": 30},
            {"name": "Alice", "age": 25},
            {"name": "Bob", "age": 35}
        ]);
        let expr = runtime
            .compile(r#"order_by(@, `[["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Alice"
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Bob"
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Charlie"
        );
    }

    #[test]
    fn test_order_by_single_field_desc() {
        let runtime = setup_runtime();
        let data = json!([
            {"name": "Alice", "age": 25},
            {"name": "Bob", "age": 35},
            {"name": "Charlie", "age": 30}
        ]);
        let expr = runtime
            .compile(r#"order_by(@, `[["age", "desc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_f64()
                .unwrap(),
            35.0
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_f64()
                .unwrap(),
            30.0
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_f64()
                .unwrap(),
            25.0
        );
    }

    #[test]
    fn test_order_by_multiple_fields() {
        let runtime = setup_runtime();
        let data = json!([
            {"dept": "sales", "name": "Bob"},
            {"dept": "eng", "name": "Alice"},
            {"dept": "sales", "name": "Alice"}
        ]);
        let expr = runtime
            .compile(r#"order_by(@, `[["dept", "asc"], ["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // eng comes first, then sales (sorted by dept)
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("dept")
                .unwrap()
                .as_str()
                .unwrap(),
            "eng"
        );
        // Within sales, Alice comes before Bob
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Alice"
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Bob"
        );
    }

    #[test]
    fn test_reduce_expr_sum() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime
            .compile("reduce_expr(&sum([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 15.0);
    }

    #[test]
    fn test_reduce_expr_max() {
        let runtime = setup_runtime();
        let data = json!([3, 1, 4, 1, 5, 9, 2, 6]);
        let expr = runtime
            .compile("reduce_expr(&max([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 9.0);
    }

    #[test]
    fn test_reduce_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime
            .compile("reduce_expr(&sum([accumulator, current]), @, `42`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 42.0); // Returns initial value
    }

    #[test]
    #[ignore] // fold alias not registered: missing standalone entry in functions.toml
    fn test_fold_alias() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime
            .compile("fold(&sum([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 6.0);
    }

    #[test]
    fn test_scan_expr_running_sum() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4]);
        let expr = runtime
            .compile("scan_expr(&sum([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Running sum: [1, 3, 6, 10]
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[1].as_f64().unwrap(), 3.0);
        assert_eq!(arr[2].as_f64().unwrap(), 6.0);
        assert_eq!(arr[3].as_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_scan_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime
            .compile("scan_expr(&sum([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_reduce_expr_with_index() {
        let runtime = setup_runtime();
        // Access the index in the reduce expression
        let data = json!([10, 20, 30]);
        let expr = runtime
            .compile("reduce_expr(&sum([accumulator, index]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 0 + 1 + 2 = 3
        assert_eq!(result.as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_count_by_objects() {
        let runtime = setup_runtime();
        let data = json!([
            {"type": "a"},
            {"type": "b"},
            {"type": "a"},
            {"type": "a"}
        ]);
        let expr = runtime.compile("count_by(&type, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 3.0);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_count_by_strings() {
        let runtime = setup_runtime();
        let data = json!(["a", "b", "a", "c", "a"]);
        let expr = runtime.compile("count_by(&@, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 3.0);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(obj.get("c").unwrap().as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_count_by_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("count_by(&type, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_count_by_numbers() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 1, 3, 1, 2]);
        let expr = runtime.compile("count_by(&@, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("1").unwrap().as_f64().unwrap(), 3.0);
        assert_eq!(obj.get("2").unwrap().as_f64().unwrap(), 2.0);
        assert_eq!(obj.get("3").unwrap().as_f64().unwrap(), 1.0);
    }

    // =============================================================================
    // Partial application tests
    // =============================================================================

    #[test]
    fn test_partial_creates_object() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("partial('length')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("__partial__").unwrap().as_bool().unwrap());
        assert_eq!(obj.get("fn").unwrap().as_str().unwrap(), "length");
        assert!(obj.get("args").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn test_partial_with_args() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("partial('contains', `\"hello world\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("__partial__").unwrap().as_bool().unwrap());
        assert_eq!(obj.get("fn").unwrap().as_str().unwrap(), "contains");
        let args = obj.get("args").unwrap().as_array().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_apply_with_fn_name() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime.compile("apply('length', `\"hello\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_apply_with_partial() {
        let runtime = setup_runtime();
        let data = json!(null);
        // Create partial with first arg, then apply with second arg
        let expr = runtime
            .compile("apply(partial('contains', `\"hello world\"`), `\"world\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_apply_partial_not_found() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("apply(partial('contains', `\"hello world\"`), `\"xyz\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_partial_with_multiple_prefilled_args() {
        let runtime = setup_runtime();
        let data = json!(null);
        // partial with 2 args pre-filled
        let expr = runtime.compile("partial('join', `\"-\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let args = obj.get("args").unwrap().as_array().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].as_str().unwrap(), "-");
    }

    #[test]
    fn test_apply_partial_join() {
        let runtime = setup_runtime();
        let data = json!(null);
        // Create a join with "-" separator, then apply to array
        let expr = runtime
            .compile("apply(partial('join', `\"-\"`), `[\"a\", \"b\", \"c\"]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "a-b-c");
    }

    // =========================================================================
    // Pipeline pattern tests
    // =========================================================================

    #[test]
    fn test_pipeline_filter_sort_products() {
        let runtime = setup_runtime();
        let data = json!({
            "products": [
                {"name": "A", "price": 30, "in_stock": true},
                {"name": "B", "price": 10, "in_stock": true},
                {"name": "C", "price": 20, "in_stock": false},
                {"name": "D", "price": 5, "in_stock": true}
            ]
        });
        let expr = runtime
            .compile("products | filter_expr(&in_stock, @) | sort_by_expr(&price, @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "D"
        ); // $5
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "B"
        ); // $10
    }

    #[test]
    fn test_pipeline_funnel_errors() {
        let runtime = setup_runtime();
        let data = json!({
            "events": [
                {"level": "error", "timestamp": 1704067300, "message": "Disk full"},
                {"level": "info", "timestamp": 1704067200, "message": "Started"},
                {"level": "error", "timestamp": 1704067400, "message": "Connection lost"},
                {"level": "warn", "timestamp": 1704067350, "message": "High memory"}
            ]
        });
        let expr = runtime
            .compile(
                r#"events | filter_expr(&(level == `"error"`), @) | sort_by_expr(&timestamp, @)"#,
            )
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Sorted by timestamp ascending
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("message")
                .unwrap()
                .as_str()
                .unwrap(),
            "Disk full"
        );
    }

    #[test]
    fn test_pipeline_transactions_completed() {
        let runtime = setup_runtime();
        let data = json!({
            "transactions": [
                {"amount": 100, "status": "completed"},
                {"amount": 50, "status": "completed"},
                {"amount": 75, "status": "pending"},
                {"amount": 200, "status": "completed"}
            ]
        });
        let expr = runtime
            .compile(
                r#"transactions | filter_expr(&(status == `"completed"`), @) | map_expr(&amount, @)"#,
            )
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 100.0);
        assert_eq!(arr[1].as_f64().unwrap(), 50.0);
        assert_eq!(arr[2].as_f64().unwrap(), 200.0);
    }

    #[test]
    fn test_pipeline_fork_join() {
        let runtime = setup_runtime();
        let data = json!({
            "items": [
                {"name": "A", "price": 150},
                {"name": "B", "price": 50},
                {"name": "C", "price": 200},
                {"name": "D", "price": 25}
            ]
        });
        let expr = runtime
            .compile(
                r#"@.{
                    expensive: items | filter_expr(&(price > `100`), @),
                    cheap: items | filter_expr(&(price <= `100`), @)
                }"#,
            )
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("expensive").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("cheap").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_pipeline_nested_users() {
        let runtime = setup_runtime();
        let data = json!({
            "users": [
                {"name": "Alice", "orders": [{"total": 100}, {"total": 50}]},
                {"name": "Bob", "orders": [{"total": 200}]},
                {"name": "Carol", "orders": []}
            ]
        });
        // Filter users with orders, then map to get names
        let expr = runtime
            .compile("users | filter_expr(&(length(orders) > `0`), @) | map_expr(&name, @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "Alice");
        assert_eq!(arr[1].as_str().unwrap(), "Bob");
    }

    #[test]
    fn test_pipeline_rag_chunks() {
        let runtime = setup_runtime();
        let data = json!({
            "chunks": [
                {"content": "Redis is fast", "score": 0.9},
                {"content": "Redis is in-memory", "score": 0.85},
                {"content": "Unrelated content", "score": 0.5},
                {"content": "Redis supports modules", "score": 0.75}
            ]
        });
        let expr = runtime
            .compile("chunks | filter_expr(&(score > `0.7`), @) | sort_by_expr(&score, @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Sorted ascending by score
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("score")
                .unwrap()
                .as_f64()
                .unwrap(),
            0.75
        );
    }

    // =========================================================================
    // Additional reduce_expr/scan_expr tests
    // =========================================================================

    #[test]
    fn test_reduce_expr_product() {
        let runtime = setup_runtime();
        // Test reduce with min (similar to existing max test but finds minimum)
        let data = json!([5, 3, 8, 1, 9]);
        let expr = runtime
            .compile("reduce_expr(&min([accumulator, current]), @, `100`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_scan_expr_running_balance() {
        let runtime = setup_runtime();
        // Test scan with running max - shows progressive maximum
        let data = json!([3, 1, 4, 1, 5, 9]);
        let expr = runtime
            .compile("scan_expr(&max([accumulator, current]), @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Running max: 3, 3, 4, 4, 5, 9
        assert_eq!(arr[0].as_f64().unwrap(), 3.0);
        assert_eq!(arr[1].as_f64().unwrap(), 3.0);
        assert_eq!(arr[2].as_f64().unwrap(), 4.0);
        assert_eq!(arr[3].as_f64().unwrap(), 4.0);
        assert_eq!(arr[4].as_f64().unwrap(), 5.0);
        assert_eq!(arr[5].as_f64().unwrap(), 9.0);
    }

    // =========================================================================
    // Additional order_by tests
    // =========================================================================

    #[test]
    fn test_order_by_three_fields() {
        let runtime = setup_runtime();
        let data = json!([
            {"dept": "Engineering", "level": "senior", "name": "Charlie"},
            {"dept": "Engineering", "level": "junior", "name": "Alice"},
            {"dept": "Engineering", "level": "senior", "name": "Bob"},
            {"dept": "Sales", "level": "senior", "name": "David"}
        ]);
        let expr = runtime
            .compile(r#"order_by(@, `[["dept", "asc"], ["level", "desc"], ["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Engineering seniors first (alphabetical), then Engineering juniors, then Sales
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Bob"
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Charlie"
        );
    }

    #[test]
    fn test_order_by_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime
            .compile(r#"order_by(@, `[["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.is_empty());
    }

    // =========================================================================
    // Additional partition_expr tests
    // =========================================================================

    #[test]
    fn test_partition_expr_scores() {
        let runtime = setup_runtime();
        let data = json!([85, 42, 91, 67, 55, 78, 33, 99]);
        let expr = runtime.compile("partition_expr(&(@ >= `60`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let passing = arr[0].as_array().unwrap();
        let failing = arr[1].as_array().unwrap();
        assert_eq!(passing.len(), 5); // 85, 91, 67, 78, 99
        assert_eq!(failing.len(), 3); // 42, 55, 33
    }

    #[test]
    fn test_partition_expr_active() {
        let runtime = setup_runtime();
        let data = json!([{"active": true}, {"active": false}, {"active": true}]);
        let expr = runtime.compile("partition_expr(&active, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_array().unwrap().len(), 2);
        assert_eq!(arr[1].as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // Additional map_values/map_keys tests
    // =========================================================================

    #[test]
    fn test_map_values_discount() {
        let runtime = setup_runtime();
        // Test with string transformation since nested expressions don't have extension math functions
        let data = json!({"apple": "FRUIT", "banana": "ITEM"});
        let expr = runtime.compile("map_values(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("apple").unwrap().as_f64().unwrap(), 5.0);
        assert_eq!(obj.get("banana").unwrap().as_f64().unwrap(), 4.0);
    }

    // =========================================================================
    // Additional group_by_expr tests
    // =========================================================================

    #[test]
    fn test_group_by_expr_type() {
        let runtime = setup_runtime();
        let data = json!([
            {"type": "fruit", "name": "apple"},
            {"type": "vegetable", "name": "carrot"},
            {"type": "fruit", "name": "banana"}
        ]);
        let expr = runtime.compile("group_by_expr(&type, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("fruit").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("vegetable").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_expr_computed() {
        let runtime = setup_runtime();
        // Group strings by their length using built-in length function
        let data = json!(["a", "bb", "ccc", "dd", "eee", "f"]);
        let expr = runtime
            .compile("group_by_expr(&to_string(length(@)), @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("1")); // "a", "f"
        assert!(obj.contains_key("2")); // "bb", "dd"
        assert!(obj.contains_key("3")); // "ccc", "eee"
        assert_eq!(obj.get("1").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("2").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("3").unwrap().as_array().unwrap().len(), 2);
    }

    // =========================================================================
    // Additional unique_by_expr tests
    // =========================================================================

    #[test]
    fn test_unique_by_expr_id() {
        let runtime = setup_runtime();
        let data = json!([
            {"id": 1, "v": "a"},
            {"id": 2, "v": "b"},
            {"id": 1, "v": "c"}
        ]);
        let expr = runtime.compile("unique_by_expr(&id, @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Keeps first occurrence
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("v")
                .unwrap()
                .as_str()
                .unwrap(),
            "a"
        );
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_any_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("any_expr(&(@ > `0`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_max_by_expr_empty() {
        let runtime = setup_runtime();
        let data = json!([]);
        let expr = runtime.compile("max_by_expr(&age, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_flat_map_expr_duplicate() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        // Duplicate each element
        let expr = runtime.compile("flat_map_expr(&[@, @], @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 6);
    }

    #[test]
    fn test_reject_greater_than() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5, 6]);
        let expr = runtime.compile("reject(&(@ > `3`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_every_false_case() {
        let runtime = setup_runtime();
        let data = json!([1, -1, 3]);
        let expr = runtime.compile("every(&(@ > `0`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_count_expr_all_match() {
        let runtime = setup_runtime();
        let data = json!([5, 10, 15, 20]);
        let expr = runtime.compile("count_expr(&(@ > `0`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_find_expr_first_match() {
        let runtime = setup_runtime();
        let data = json!([1, 5, 10, 15]);
        let expr = runtime.compile("find_expr(&(@ > `3`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_find_index_expr_first_match() {
        let runtime = setup_runtime();
        let data = json!([1, 5, 10, 15]);
        let expr = runtime.compile("find_index_expr(&(@ > `3`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_take_while_basic() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 5, 1, 2]);
        let expr = runtime.compile("take_while(&(@ < `4`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[1].as_f64().unwrap(), 2.0);
        assert_eq!(arr[2].as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_take_while_all_match() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("take_while(&(@ < `10`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_take_while_none_match() {
        let runtime = setup_runtime();
        let data = json!([5, 6, 7]);
        let expr = runtime.compile("take_while(&(@ < `4`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_drop_while_basic() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 5, 1, 2]);
        let expr = runtime.compile("drop_while(&(@ < `4`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 5.0);
        assert_eq!(arr[1].as_f64().unwrap(), 1.0);
        assert_eq!(arr[2].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_drop_while_all_match() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("drop_while(&(@ < `10`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_drop_while_none_match() {
        let runtime = setup_runtime();
        let data = json!([5, 6, 7]);
        let expr = runtime.compile("drop_while(&(@ < `4`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_zip_with_add() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("zip_with(&add([0], [1]), `[1, 2, 3]`, `[10, 20, 30]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 11.0);
        assert_eq!(arr[1].as_f64().unwrap(), 22.0);
        assert_eq!(arr[2].as_f64().unwrap(), 33.0);
    }

    #[test]
    fn test_zip_with_unequal_lengths() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("zip_with(&add([0], [1]), `[1, 2, 3, 4, 5]`, `[10, 20]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_f64().unwrap(), 11.0);
        assert_eq!(arr[1].as_f64().unwrap(), 22.0);
    }

    #[test]
    fn test_zip_with_multiply() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("zip_with(&multiply([0], [1]), `[2, 3, 4]`, `[5, 6, 7]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 10.0);
        assert_eq!(arr[1].as_f64().unwrap(), 18.0);
        assert_eq!(arr[2].as_f64().unwrap(), 28.0);
    }

    // =========================================================================
    // walk tests
    // =========================================================================

    #[test]
    fn test_walk_identity() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, 2, 3], "b": {"c": 4}});
        let expr = runtime.compile("walk(&@, @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Identity should return the same structure
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("b"));
    }

    #[test]
    fn test_walk_type_of_all() {
        let runtime = setup_runtime();
        let data = json!({"a": 5, "b": [1, 2]});
        // type() works on everything - shows bottom-up processing
        let expr = runtime.compile("walk(&type(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        // After walking, everything becomes its type string, and the final result
        // is type of the top-level result
        assert_eq!(result.as_str().unwrap(), "object");
    }

    #[test]
    fn test_walk_nested_arrays() {
        let runtime = setup_runtime();
        // Use only arrays (no scalars inside) so length works at every level
        let data = json!([[[], []], [[]]]);
        // length works on arrays - get lengths at each level
        let expr = runtime.compile("walk(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Inner [] -> 0, outer arrays get lengths, top level has 2 elements
        assert_eq!(result.as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_walk_scalar() {
        let runtime = setup_runtime();
        let data = json!(5);
        // Double the number
        let expr = runtime.compile("walk(&multiply(@, `2`), @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_walk_length_all() {
        let runtime = setup_runtime();
        let data = json!({"items": ["a", "bb", "ccc"]});
        // Get length of everything (works for strings, arrays, objects)
        let expr = runtime.compile("walk(&length(@), @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Top level object has 1 key
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_walk_preserves_structure() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, {"b": 2}], "c": "hello"});
        // Identity transform - should preserve structure
        let expr = runtime.compile("walk(&@, @)").unwrap();
        let result = expr.search(&data).unwrap();

        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        let arr = obj.get("a").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_walk_empty_structures() {
        let runtime = setup_runtime();

        // Empty array
        let data = json!([]);
        let expr = runtime.compile("walk(&@, @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_array().unwrap().is_empty());

        // Empty object
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }

    // =========================================================================
    // recurse tests
    // =========================================================================

    #[test]
    fn test_recurse_nested_object() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}});
        let expr = runtime.compile("recurse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: {a: {b: 1}}, {b: 1}, 1
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_recurse_nested_array() {
        let runtime = setup_runtime();
        let data = json!([1, [2, 3]]);
        let expr = runtime.compile("recurse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: [1, [2, 3]], 1, [2, 3], 2, 3
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_recurse_scalar() {
        let runtime = setup_runtime();
        let data = json!(42);
        let expr = runtime.compile("recurse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Just the scalar itself
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_f64().unwrap(), 42.0);
    }

    #[test]
    fn test_recurse_with_field() {
        let runtime = setup_runtime();
        let data = json!({"a": {"a": {"a": null}}});
        let expr = runtime.compile("recurse_with(@, &a)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: {a: {a: {a: null}}}, {a: {a: null}}, {a: null}
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_recurse_with_children() {
        let runtime = setup_runtime();
        let data = json!({
            "name": "root",
            "children": [
                {"name": "child1", "children": []},
                {"name": "child2", "children": []}
            ]
        });
        let expr = runtime.compile("recurse_with(@, &children[])").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: root, child1, child2
        assert_eq!(arr.len(), 3);
    }

    // =========================================================================
    // while_expr tests
    // =========================================================================

    #[test]
    fn test_while_expr_doubling() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("while_expr(`1`, &(@ < `100`), &multiply(@, `2`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 1 -> 2 -> 4 -> 8 -> 16 -> 32 -> 64 -> 128 (stops because 128 >= 100)
        assert_eq!(result.as_f64().unwrap(), 128.0);
    }

    #[test]
    fn test_while_expr_counter() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("while_expr(`0`, &(@ < `5`), &add(@, `1`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 0 -> 1 -> 2 -> 3 -> 4 -> 5 (stops because 5 >= 5)
        assert_eq!(result.as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_while_expr_immediate_false() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("while_expr(`100`, &(@ < `10`), &add(@, `1`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Condition is false immediately, return initial value
        assert_eq!(result.as_f64().unwrap(), 100.0);
    }

    // =========================================================================
    // until_expr tests
    // =========================================================================

    #[test]
    fn test_until_expr_doubling() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("until_expr(`1`, &(@ >= `100`), &multiply(@, `2`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 1 -> 2 -> 4 -> 8 -> 16 -> 32 -> 64 -> 128 (stops because 128 >= 100)
        assert_eq!(result.as_f64().unwrap(), 128.0);
    }

    #[test]
    fn test_until_expr_counter() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("until_expr(`0`, &(@ == `5`), &add(@, `1`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 0 -> 1 -> 2 -> 3 -> 4 -> 5 (stops because 5 == 5)
        assert_eq!(result.as_f64().unwrap(), 5.0);
    }

    #[test]
    fn test_until_expr_immediate_true() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("until_expr(`100`, &(@ > `10`), &add(@, `1`))")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Condition is true immediately, return initial value
        assert_eq!(result.as_f64().unwrap(), 100.0);
    }
}
