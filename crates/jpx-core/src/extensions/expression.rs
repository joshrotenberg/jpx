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
