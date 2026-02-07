//! Array manipulation functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, custom_error};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register only the array functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "unique", enabled, Box::new(UniqueFn::new()));
    register_if_enabled(runtime, "zip", enabled, Box::new(ZipFn::new()));
    register_if_enabled(runtime, "chunk", enabled, Box::new(ChunkFn::new()));
    register_if_enabled(runtime, "take", enabled, Box::new(TakeFn::new()));
    register_if_enabled(runtime, "drop", enabled, Box::new(DropFn::new()));
    register_if_enabled(
        runtime,
        "flatten_deep",
        enabled,
        Box::new(FlattenDeepFn::new()),
    );
    register_if_enabled(runtime, "flatten", enabled, Box::new(FlattenFn::new()));
    register_if_enabled(runtime, "compact", enabled, Box::new(CompactFn::new()));
    register_if_enabled(runtime, "range", enabled, Box::new(RangeFn::new()));
    register_if_enabled(runtime, "index_at", enabled, Box::new(IndexAtFn::new()));
    register_if_enabled(runtime, "includes", enabled, Box::new(IncludesFn::new()));
    register_if_enabled(runtime, "find_index", enabled, Box::new(FindIndexFn::new()));
    register_if_enabled(runtime, "first", enabled, Box::new(FirstFn::new()));
    register_if_enabled(runtime, "last", enabled, Box::new(LastFn::new()));
    register_if_enabled(runtime, "group_by", enabled, Box::new(GroupByFn::new()));
    register_if_enabled(runtime, "index_by", enabled, Box::new(IndexByFn::new()));
    register_if_enabled(runtime, "nth", enabled, Box::new(NthFn::new()));
    register_if_enabled(
        runtime,
        "interleave",
        enabled,
        Box::new(InterleaveFn::new()),
    );
    register_if_enabled(runtime, "rotate", enabled, Box::new(RotateFn::new()));
    register_if_enabled(runtime, "partition", enabled, Box::new(PartitionFn::new()));
    register_if_enabled(
        runtime,
        "difference",
        enabled,
        Box::new(DifferenceFn::new()),
    );
    register_if_enabled(
        runtime,
        "intersection",
        enabled,
        Box::new(IntersectionFn::new()),
    );
    register_if_enabled(runtime, "union", enabled, Box::new(UnionFn::new()));
    register_if_enabled(
        runtime,
        "frequencies",
        enabled,
        Box::new(FrequenciesFn::new()),
    );
    register_if_enabled(runtime, "mode", enabled, Box::new(ModeFn::new()));
    register_if_enabled(runtime, "cartesian", enabled, Box::new(CartesianFn::new()));
    register_if_enabled(runtime, "initial", enabled, Box::new(InitialFn::new()));
    // Alias for initial (Clojure-style name)
    register_if_enabled(runtime, "butlast", enabled, Box::new(InitialFn::new()));
    // Clojure-inspired functions
    register_if_enabled(runtime, "interpose", enabled, Box::new(InterposeFn::new()));
    register_if_enabled(runtime, "zipmap", enabled, Box::new(ZipmapFn::new()));
    register_if_enabled(
        runtime,
        "partition_by",
        enabled,
        Box::new(PartitionByFn::new()),
    );
    register_if_enabled(runtime, "dedupe", enabled, Box::new(DedupeFn::new()));
    register_if_enabled(runtime, "tail", enabled, Box::new(TailFn::new()));
    register_if_enabled(runtime, "without", enabled, Box::new(WithoutFn::new()));
    register_if_enabled(runtime, "xor", enabled, Box::new(XorFn::new()));
    register_if_enabled(runtime, "fill", enabled, Box::new(FillFn::new()));
    register_if_enabled(runtime, "pull_at", enabled, Box::new(PullAtFn::new()));
    register_if_enabled(runtime, "window", enabled, Box::new(WindowFn::new()));
    register_if_enabled(
        runtime,
        "combinations",
        enabled,
        Box::new(CombinationsFn::new()),
    );
    register_if_enabled(runtime, "transpose", enabled, Box::new(TransposeFn::new()));
    register_if_enabled(runtime, "pairwise", enabled, Box::new(PairwiseFn::new()));
    // Alias for window (sliding_window is a common name)
    register_if_enabled(
        runtime,
        "sliding_window",
        enabled,
        Box::new(WindowFn::new()),
    );
    // jq-parity functions
    register_if_enabled(
        runtime,
        "indices_array",
        enabled,
        Box::new(IndicesArrayFn::new()),
    );
    register_if_enabled(
        runtime,
        "inside_array",
        enabled,
        Box::new(InsideArrayFn::new()),
    );
    register_if_enabled(runtime, "bsearch", enabled, Box::new(BsearchFn::new()));
    // Clojure-inspired functions (Phase 3)
    register_if_enabled(
        runtime,
        "repeat_array",
        enabled,
        Box::new(RepeatArrayFn::new()),
    );
    register_if_enabled(runtime, "cycle", enabled, Box::new(CycleFn::new()));
}

// =============================================================================
// unique(array) -> array
// =============================================================================

defn!(UniqueFn, vec![arg!(array)], None);

impl Function for UniqueFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in arr {
            let key = serde_json::to_string(item).unwrap_or_default();
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// zip(array1, array2) -> array of pairs
// =============================================================================

defn!(ZipFn, vec![arg!(array), arg!(array)], None);

impl Function for ZipFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let result: Vec<Value> = arr1
            .iter()
            .zip(arr2.iter())
            .map(|(a, b)| Value::Array(vec![a.clone(), b.clone()]))
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// chunk(array, size) -> array of arrays
// =============================================================================

defn!(ChunkFn, vec![arg!(array), arg!(number)], None);

impl Function for ChunkFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let size = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number for size"))?;

        if size == 0 {
            return Ok(Value::Array(vec![]));
        }

        let chunks: Vec<Value> = arr
            .chunks(size)
            .map(|chunk| Value::Array(chunk.to_vec()))
            .collect();

        Ok(Value::Array(chunks))
    }
}

// =============================================================================
// take(array, n) -> array (first n elements)
// =============================================================================

defn!(TakeFn, vec![arg!(array), arg!(number)], None);

impl Function for TakeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number"))?;

        let result: Vec<Value> = arr.iter().take(n).cloned().collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// drop(array, n) -> array (skip first n elements)
// =============================================================================

defn!(DropFn, vec![arg!(array), arg!(number)], None);

impl Function for DropFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number"))?;

        let result: Vec<Value> = arr.iter().skip(n).cloned().collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// flatten_deep(array) -> array (recursively flatten)
// =============================================================================

defn!(FlattenDeepFn, vec![arg!(array)], None);

fn flatten_recursive(arr: &[Value]) -> Vec<Value> {
    let mut result = Vec::new();
    for item in arr {
        if let Some(inner) = item.as_array() {
            result.extend(flatten_recursive(inner));
        } else {
            result.push(item.clone());
        }
    }
    result
}

impl Function for FlattenDeepFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        Ok(Value::Array(flatten_recursive(arr)))
    }
}

// =============================================================================
// flatten(array) -> array (single-level flatten)
// =============================================================================

defn!(FlattenFn, vec![arg!(array)], None);

impl Function for FlattenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut result = Vec::new();
        for item in arr {
            if let Some(inner) = item.as_array() {
                result.extend(inner.iter().cloned());
            } else {
                result.push(item.clone());
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// compact(array) -> array (remove null/false values)
// =============================================================================

defn!(CompactFn, vec![arg!(array)], None);

impl Function for CompactFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let result: Vec<Value> = arr
            .iter()
            .filter(|v| !v.is_null() && !matches!(v, Value::Bool(false)))
            .cloned()
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// range(start, end, step?) -> array
// =============================================================================

defn!(
    RangeFn,
    vec![arg!(number), arg!(number)],
    Some(arg!(number))
);

impl Function for RangeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let start = args[0]
            .as_f64()
            .map(|n| n as i64)
            .ok_or_else(|| custom_error(ctx, "Expected start number"))?;

        let end = args[1]
            .as_f64()
            .map(|n| n as i64)
            .ok_or_else(|| custom_error(ctx, "Expected end number"))?;

        let step = if args.len() > 2 {
            args[2]
                .as_f64()
                .map(|n| n as i64)
                .ok_or_else(|| custom_error(ctx, "Expected step number"))?
        } else {
            1
        };

        if step == 0 {
            return Err(custom_error(ctx, "Step cannot be zero"));
        }

        let mut result = Vec::new();
        let mut current = start;

        const MAX_RANGE: usize = 10000;

        if step > 0 {
            while current < end && result.len() < MAX_RANGE {
                result.push(Value::Number(Number::from(current)));
                current += step;
            }
        } else {
            while current > end && result.len() < MAX_RANGE {
                result.push(Value::Number(Number::from(current)));
                current += step;
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// index_at(array, index) -> element (supports negative index)
// =============================================================================

defn!(IndexAtFn, vec![arg!(array), arg!(number)], None);

impl Function for IndexAtFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let index = args[1]
            .as_f64()
            .map(|n| n as i64)
            .ok_or_else(|| custom_error(ctx, "Expected number for index"))?;

        let len = arr.len() as i64;
        let actual_index = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        if actual_index < arr.len() {
            Ok(arr[actual_index].clone())
        } else {
            Ok(Value::Null)
        }
    }
}

// =============================================================================
// includes(array, value) -> boolean
// =============================================================================

defn!(IncludesFn, vec![arg!(array), arg!(any)], None);

impl Function for IncludesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let search_key = serde_json::to_string(&args[1]).unwrap_or_default();

        let found = arr.iter().any(|item| {
            let item_key = serde_json::to_string(item).unwrap_or_default();
            item_key == search_key
        });

        Ok(Value::Bool(found))
    }
}

// =============================================================================
// find_index(array, value) -> number (-1 if not found)
// =============================================================================

defn!(FindIndexFn, vec![arg!(array), arg!(any)], None);

impl Function for FindIndexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let search_key = serde_json::to_string(&args[1]).unwrap_or_default();

        let index = arr
            .iter()
            .position(|item| {
                let item_key = serde_json::to_string(item).unwrap_or_default();
                item_key == search_key
            })
            .map(|i| i as i64)
            .unwrap_or(-1);

        Ok(Value::Number(Number::from(index)))
    }
}

// =============================================================================
// first(array) -> any (first element or null)
// =============================================================================

defn!(FirstFn, vec![arg!(array)], None);

impl Function for FirstFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        Ok(arr.first().cloned().unwrap_or(Value::Null))
    }
}

// =============================================================================
// last(array) -> any (last element or null)
// =============================================================================

defn!(LastFn, vec![arg!(array)], None);

impl Function for LastFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        Ok(arr.last().cloned().unwrap_or(Value::Null))
    }
}

// =============================================================================
// group_by(array, field_name) -> object
// =============================================================================

defn!(GroupByFn, vec![arg!(array), arg!(string)], None);

impl Function for GroupByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let field_name = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected field name string"))?;

        let mut groups: std::collections::BTreeMap<String, Vec<Value>> =
            std::collections::BTreeMap::new();

        for item in arr {
            let key = if let Some(obj) = item.as_object() {
                if let Some(field_value) = obj.get(field_name) {
                    match field_value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => continue,
                    }
                } else {
                    "null".to_string()
                }
            } else {
                continue;
            };
            groups.entry(key).or_default().push(item.clone());
        }

        let mut result = serde_json::Map::new();
        for (k, v) in groups {
            result.insert(k, Value::Array(v));
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// index_by(array, field_name) -> object (last value wins for duplicates)
// =============================================================================

defn!(IndexByFn, vec![arg!(array), arg!(string)], None);

impl Function for IndexByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let field_name = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected field name string"))?;

        let mut result = serde_json::Map::new();

        for item in arr {
            let key = if let Some(obj) = item.as_object() {
                if let Some(field_value) = obj.get(field_name) {
                    match field_value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => continue,
                    }
                } else {
                    // Skip items without the key field
                    continue;
                }
            } else {
                continue;
            };
            // Last value wins for duplicate keys
            result.insert(key, item.clone());
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// nth(array, n) -> array (every nth element)
// =============================================================================

defn!(NthFn, vec![arg!(array), arg!(number)], None);

impl Function for NthFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number argument"))? as usize;

        if n == 0 {
            return Ok(Value::Null);
        }

        let result: Vec<Value> = arr.iter().step_by(n).cloned().collect();
        Ok(Value::Array(result))
    }
}

// =============================================================================
// interleave(array1, array2) -> array (alternate elements)
// =============================================================================

defn!(InterleaveFn, vec![arg!(array), arg!(array)], None);

impl Function for InterleaveFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut result = Vec::with_capacity(arr1.len() + arr2.len());
        let mut iter1 = arr1.iter();
        let mut iter2 = arr2.iter();

        loop {
            match (iter1.next(), iter2.next()) {
                (Some(a), Some(b)) => {
                    result.push(a.clone());
                    result.push(b.clone());
                }
                (Some(a), None) => {
                    result.push(a.clone());
                    result.extend(iter1.cloned());
                    break;
                }
                (None, Some(b)) => {
                    result.push(b.clone());
                    result.extend(iter2.cloned());
                    break;
                }
                (None, None) => break,
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// rotate(array, n) -> array (rotate elements by n positions)
// =============================================================================

defn!(RotateFn, vec![arg!(array), arg!(number)], None);

impl Function for RotateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number argument"))? as i64;

        let len = arr.len() as i64;
        let rotation = ((n % len) + len) % len;
        let rotation = rotation as usize;

        let mut result = Vec::with_capacity(arr.len());
        result.extend(arr[rotation..].iter().cloned());
        result.extend(arr[..rotation].iter().cloned());

        Ok(Value::Array(result))
    }
}

// =============================================================================
// partition(array, n) -> array (split into n equal parts)
// =============================================================================

defn!(PartitionFn, vec![arg!(array), arg!(number)], None);

impl Function for PartitionFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number argument"))? as usize;

        if n == 0 {
            return Ok(Value::Null);
        }

        let len = arr.len();
        let base_size = len / n;
        let remainder = len % n;

        let mut result = Vec::with_capacity(n);
        let mut start = 0;

        for i in 0..n {
            let size = base_size + if i < remainder { 1 } else { 0 };
            if size > 0 {
                result.push(Value::Array(arr[start..start + size].to_vec()));
            } else {
                result.push(Value::Array(vec![]));
            }
            start += size;
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// difference(arr1, arr2) -> array (set difference)
// =============================================================================

defn!(DifferenceFn, vec![arg!(array), arg!(array)], None);

impl Function for DifferenceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect();

        let result: Vec<Value> = arr1
            .iter()
            .filter(|v| {
                let key = serde_json::to_string(*v).unwrap_or_default();
                !set2.contains(&key)
            })
            .cloned()
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// intersection(arr1, arr2) -> array (set intersection)
// =============================================================================

defn!(IntersectionFn, vec![arg!(array), arg!(array)], None);

impl Function for IntersectionFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect();

        let mut seen: HashSet<String> = HashSet::new();
        let result: Vec<Value> = arr1
            .iter()
            .filter(|v| {
                let key = serde_json::to_string(*v).unwrap_or_default();
                set2.contains(&key) && seen.insert(key)
            })
            .cloned()
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// union(arr1, arr2) -> array (set union)
// =============================================================================

defn!(UnionFn, vec![arg!(array), arg!(array)], None);

impl Function for UnionFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut seen: HashSet<String> = HashSet::new();
        let mut result: Vec<Value> = Vec::new();

        for item in arr1.iter().chain(arr2.iter()) {
            let key = serde_json::to_string(item).unwrap_or_default();
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// frequencies(array) -> object (count occurrences)
// =============================================================================

defn!(FrequenciesFn, vec![arg!(array)], None);

impl Function for FrequenciesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for item in arr {
            let key = match item {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => serde_json::to_string(item).unwrap_or_else(|_| "null".to_string()),
            };
            *counts.entry(key).or_insert(0) += 1;
        }

        let mut result = serde_json::Map::new();
        // Use BTreeMap for sorted output
        let sorted: std::collections::BTreeMap<String, i64> = counts.into_iter().collect();
        for (k, v) in sorted {
            result.insert(k, Value::Number(Number::from(v)));
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// mode(array) -> any (most frequent value)
// =============================================================================

defn!(ModeFn, vec![arg!(array)], None);

impl Function for ModeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Null);
        }

        let mut counts: std::collections::HashMap<String, (i64, Value)> =
            std::collections::HashMap::new();

        for item in arr {
            let key = serde_json::to_string(item).unwrap_or_default();
            counts
                .entry(key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, item.clone()));
        }

        let (_, (_, mode_value)) = counts
            .into_iter()
            .max_by_key(|(_, (count, _))| *count)
            .unwrap();

        Ok(mode_value)
    }
}

// =============================================================================
// cartesian(arr1, arr2) -> array (cartesian product of 2 arrays)
// cartesian(array_of_arrays) -> array (cartesian product of N arrays, jq parity)
// =============================================================================

defn!(CartesianFn, vec![arg!(array)], Some(arg!(array)));

impl Function for CartesianFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let first = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        // Two modes:
        // 1. cartesian(arr1, arr2) - two separate arrays (original behavior)
        // 2. cartesian(array_of_arrays) - single array containing arrays (jq-style)

        if args.len() == 2 {
            // Original two-argument mode
            let arr2 = args[1]
                .as_array()
                .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

            let mut result = Vec::with_capacity(first.len() * arr2.len());
            for a in first {
                for b in arr2 {
                    result.push(Value::Array(vec![a.clone(), b.clone()]));
                }
            }
            Ok(Value::Array(result))
        } else {
            // Single argument mode - check if it's an array of arrays
            // If all elements are arrays, do N-way cartesian product
            let arrays: Vec<&Vec<Value>> =
                first.iter().filter_map(|item| item.as_array()).collect();

            if arrays.len() != first.len() || arrays.is_empty() {
                // Not all elements are arrays, or empty - return empty
                return Ok(Value::Array(vec![]));
            }

            // N-way cartesian product
            let result = cartesian_product_n(&arrays);
            Ok(Value::Array(result))
        }
    }
}

/// Compute N-way cartesian product of arrays.
fn cartesian_product_n(arrays: &[&Vec<Value>]) -> Vec<Value> {
    if arrays.is_empty() {
        return vec![];
    }

    if arrays.len() == 1 {
        // Single array - each element becomes a 1-element tuple
        return arrays[0]
            .iter()
            .map(|item| Value::Array(vec![item.clone()]))
            .collect();
    }

    // Calculate total size for pre-allocation
    let total_size: usize = arrays.iter().map(|a| a.len()).product();
    if total_size == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(total_size);

    // Iterative cartesian product using indices
    let mut indices = vec![0usize; arrays.len()];

    loop {
        // Build current combination
        let combo: Vec<Value> = indices
            .iter()
            .enumerate()
            .map(|(arr_idx, &elem_idx)| arrays[arr_idx][elem_idx].clone())
            .collect();
        result.push(Value::Array(combo));

        // Increment indices (like counting in mixed radix)
        let mut carry = true;
        for i in (0..arrays.len()).rev() {
            if carry {
                indices[i] += 1;
                if indices[i] >= arrays[i].len() {
                    indices[i] = 0;
                } else {
                    carry = false;
                }
            }
        }

        // If we carried all the way through, we're done
        if carry {
            break;
        }
    }

    result
}

// =============================================================================
// initial(array) -> array (all elements except the last)
// =============================================================================

defn!(InitialFn, vec![arg!(array)], None);

impl Function for InitialFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let result: Vec<Value> = arr[..arr.len() - 1].to_vec();
        Ok(Value::Array(result))
    }
}

// =============================================================================
// interpose(array, separator) -> array (insert separator between elements)
// =============================================================================

defn!(InterposeFn, vec![arg!(array), arg!(any)], None);

impl Function for InterposeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let separator = args[1].clone();

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        if arr.len() == 1 {
            return Ok(Value::Array(arr.clone()));
        }

        let mut result = Vec::with_capacity(arr.len() * 2 - 1);
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                result.push(separator.clone());
            }
            result.push(item.clone());
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// zipmap(keys, values) -> object (create object from parallel arrays)
// =============================================================================

defn!(ZipmapFn, vec![arg!(array), arg!(array)], None);

impl Function for ZipmapFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let keys = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument for keys"))?;

        let values = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument for values"))?;

        let len = keys.len().min(values.len());
        let mut result = serde_json::Map::new();

        for i in 0..len {
            let key = keys[i]
                .as_str()
                .ok_or_else(|| custom_error(ctx, "Keys must be strings"))?;
            result.insert(key.to_string(), values[i].clone());
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// partition_by(array, field_name) -> array (split when field value changes)
// =============================================================================

defn!(PartitionByFn, vec![arg!(array), arg!(string)], None);

impl Function for PartitionByFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let field_name = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected field name string"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut result: Vec<Value> = Vec::new();
        let mut current_partition: Vec<Value> = Vec::new();
        let mut last_key: Option<String> = None;

        for item in arr {
            // Extract key from object field, or serialize the item itself for primitives
            let key = if let Some(obj) = item.as_object() {
                if let Some(field_value) = obj.get(field_name) {
                    serde_json::to_string(field_value).unwrap_or_default()
                } else {
                    "null".to_string()
                }
            } else {
                // For non-objects, use the value itself as the key
                serde_json::to_string(item).unwrap_or_default()
            };

            match &last_key {
                Some(prev_key) if *prev_key == key => {
                    current_partition.push(item.clone());
                }
                _ => {
                    if !current_partition.is_empty() {
                        result.push(Value::Array(current_partition));
                    }
                    current_partition = vec![item.clone()];
                    last_key = Some(key);
                }
            }
        }

        // Don't forget the last partition
        if !current_partition.is_empty() {
            result.push(Value::Array(current_partition));
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// dedupe(array) -> array (remove consecutive duplicates)
// =============================================================================

defn!(DedupeFn, vec![arg!(array)], None);

impl Function for DedupeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let mut result: Vec<Value> = Vec::with_capacity(arr.len());
        let mut last_key: Option<String> = None;

        for item in arr {
            let key = serde_json::to_string(item).unwrap_or_default();
            match &last_key {
                Some(prev_key) if *prev_key == key => {
                    // Skip consecutive duplicate
                }
                _ => {
                    result.push(item.clone());
                    last_key = Some(key);
                }
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// tail(array) -> array (all elements except the first)
// =============================================================================

defn!(TailFn, vec![arg!(array)], None);

impl Function for TailFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let result: Vec<Value> = arr[1..].to_vec();
        Ok(Value::Array(result))
    }
}

// =============================================================================
// without(array, values_array) -> array (remove specified values)
// =============================================================================

defn!(WithoutFn, vec![arg!(array), arg!(array)], None);

impl Function for WithoutFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let exclude = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument for values to exclude"))?;

        // Create a set of serialized values to exclude for efficient lookup
        let exclude_set: HashSet<String> = exclude
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect();

        let result: Vec<Value> = arr
            .iter()
            .filter(|item| {
                let key = serde_json::to_string(*item).unwrap_or_default();
                !exclude_set.contains(&key)
            })
            .cloned()
            .collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// xor(array1, array2) -> array (symmetric difference)
// =============================================================================

defn!(XorFn, vec![arg!(array), arg!(array)], None);

impl Function for XorFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let arr2 = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        // Create sets of serialized values
        let set1: HashSet<String> = arr1
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect();

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .collect();

        let mut result = Vec::new();

        // Add elements from arr1 that are not in arr2
        for item in arr1 {
            let key = serde_json::to_string(item).unwrap_or_default();
            if !set2.contains(&key) {
                result.push(item.clone());
            }
        }

        // Add elements from arr2 that are not in arr1
        for item in arr2 {
            let key = serde_json::to_string(item).unwrap_or_default();
            if !set1.contains(&key) {
                result.push(item.clone());
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// window(array, size, step?) -> array (sliding window)
// =============================================================================

defn!(
    WindowFn,
    vec![arg!(array), arg!(number)],
    Some(arg!(number))
);

impl Function for WindowFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let size = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for window size"))?
            as usize;

        if size == 0 {
            return Ok(Value::Array(vec![]));
        }

        // Default step is 1
        let step = if args.len() > 2 {
            args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for step"))? as usize
        } else {
            1
        };

        if step == 0 {
            return Err(custom_error(ctx, "Step cannot be zero"));
        }

        let len = arr.len();
        if len < size {
            return Ok(Value::Array(vec![]));
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i + size <= len {
            let window: Vec<Value> = arr[i..i + size].to_vec();
            result.push(Value::Array(window));
            i += step;
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// combinations(array, k) -> array (k-combinations of array)
// =============================================================================

defn!(CombinationsFn, vec![arg!(array), arg!(number)], None);

fn generate_combinations(arr: &[Value], k: usize) -> Vec<Vec<Value>> {
    if k == 0 {
        return vec![vec![]];
    }
    if arr.len() < k {
        return vec![];
    }

    let mut result = Vec::new();

    // Include first element in combination
    let first = arr[0].clone();
    let rest = &arr[1..];
    for mut combo in generate_combinations(rest, k - 1) {
        let mut new_combo = vec![first.clone()];
        new_combo.append(&mut combo);
        result.push(new_combo);
    }

    // Exclude first element
    result.extend(generate_combinations(rest, k));

    result
}

impl Function for CombinationsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let k = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for k"))? as usize;

        // Limit to prevent excessive computation
        const MAX_COMBINATIONS: usize = 10000;

        // Quick check: if C(n, k) would be too large, return error
        let n = arr.len();
        if n > 20 && k > 3 && k < n - 3 {
            return Err(custom_error(ctx, "Combination size too large"));
        }

        let combinations = generate_combinations(arr, k);

        if combinations.len() > MAX_COMBINATIONS {
            return Err(custom_error(ctx, "Too many combinations generated"));
        }

        let result: Vec<Value> = combinations.into_iter().map(Value::Array).collect();

        Ok(Value::Array(result))
    }
}

// =============================================================================
// fill(array, value, start?, end?) -> array (fill range with value)
// =============================================================================

defn!(FillFn, vec![arg!(array), arg!(any)], Some(arg!(number)));

impl Function for FillFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let fill_value = args[1].clone();

        let len = arr.len();
        if len == 0 {
            return Ok(Value::Array(vec![]));
        }

        // Default start is 0, default end is array length
        let start = if args.len() > 2 {
            let s = args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for start index"))?
                as i64;
            // Handle negative indices
            if s < 0 {
                (len as i64 + s).max(0) as usize
            } else {
                (s as usize).min(len)
            }
        } else {
            0
        };

        let end = if args.len() > 3 {
            let e = args[3]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for end index"))?
                as i64;
            // Handle negative indices
            if e < 0 {
                (len as i64 + e).max(0) as usize
            } else {
                (e as usize).min(len)
            }
        } else {
            len
        };

        let mut result: Vec<Value> = arr.clone();

        for item in result.iter_mut().take(end.min(len)).skip(start) {
            *item = fill_value.clone();
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// pull_at(array, indices_array) -> array (get elements at specified indices)
// =============================================================================

defn!(PullAtFn, vec![arg!(array), arg!(array)], None);

impl Function for PullAtFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let indices = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array of indices"))?;

        let len = arr.len();
        let mut result = Vec::new();

        for idx_var in indices {
            let idx = idx_var
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number in indices array"))?
                as i64;

            // Handle negative indices
            let actual_idx = if idx < 0 {
                (len as i64 + idx).max(0) as usize
            } else {
                idx as usize
            };

            if actual_idx < len {
                result.push(arr[actual_idx].clone());
            }
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// transpose(array) -> array
// =============================================================================

defn!(TransposeFn, vec![arg!(array)], None);

impl Function for TransposeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        // Get all inner arrays and find the minimum length
        let mut inner_arrays: Vec<&Vec<Value>> = Vec::new();
        let mut min_len = usize::MAX;

        for item in arr {
            if let Some(inner) = item.as_array() {
                min_len = min_len.min(inner.len());
                inner_arrays.push(inner);
            } else {
                // If any element is not an array, return empty
                return Ok(Value::Array(vec![]));
            }
        }

        if inner_arrays.is_empty() || min_len == 0 {
            return Ok(Value::Array(vec![]));
        }

        // Transpose: create new arrays where each contains the i-th element from each inner array
        let mut result = Vec::with_capacity(min_len);
        for i in 0..min_len {
            let mut row = Vec::with_capacity(inner_arrays.len());
            for inner in &inner_arrays {
                row.push(inner[i].clone());
            }
            result.push(Value::Array(row));
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// pairwise(array) -> array
// =============================================================================

defn!(PairwiseFn, vec![arg!(array)], None);

impl Function for PairwiseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        if arr.len() < 2 {
            return Ok(Value::Array(vec![]));
        }

        let mut result = Vec::with_capacity(arr.len() - 1);
        for i in 0..arr.len() - 1 {
            let pair = vec![arr[i].clone(), arr[i + 1].clone()];
            result.push(Value::Array(pair));
        }

        Ok(Value::Array(result))
    }
}

// =============================================================================
// indices_array(array, value) -> array of indices
// =============================================================================

defn!(IndicesArrayFn, vec![arg!(array), arg!(any)], None);

impl Function for IndicesArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let search_key = serde_json::to_string(&args[1]).unwrap_or_default();

        let mut indices: Vec<Value> = Vec::new();
        for (i, item) in arr.iter().enumerate() {
            let item_key = serde_json::to_string(item).unwrap_or_default();
            if item_key == search_key {
                indices.push(Value::Number(Number::from(i as i64)));
            }
        }

        Ok(Value::Array(indices))
    }
}

// =============================================================================
// inside_array(needle, haystack) -> boolean
// =============================================================================

defn!(InsideArrayFn, vec![arg!(array), arg!(array)], None);

impl Function for InsideArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let needle = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let haystack = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        // Build a set of serialized haystack values for efficient lookup
        let haystack_set: HashSet<String> = haystack
            .iter()
            .map(|item| serde_json::to_string(item).unwrap_or_default())
            .collect();

        // Check if all needle elements are in the haystack
        let result = needle.iter().all(|item| {
            let item_key = serde_json::to_string(item).unwrap_or_default();
            haystack_set.contains(&item_key)
        });

        Ok(Value::Bool(result))
    }
}

// =============================================================================
// bsearch(sorted_array, value) -> number
// =============================================================================

defn!(BsearchFn, vec![arg!(array), arg!(any)], None);

impl Function for BsearchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let target = &args[1];

        if arr.is_empty() {
            return Ok(Value::Number(Number::from(-1)));
        }

        // Helper to compare two Value values
        // Returns Ordering based on type-aware comparison
        fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
            match (a, b) {
                // Numbers: compare numerically
                (Value::Number(n1), Value::Number(n2)) => {
                    let f1 = n1.as_f64().unwrap_or(0.0);
                    let f2 = n2.as_f64().unwrap_or(0.0);
                    f1.partial_cmp(&f2).unwrap_or(std::cmp::Ordering::Equal)
                }
                // Strings: compare lexicographically
                (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
                // Booleans: false < true
                (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
                // Different types or complex types: fall back to JSON string comparison
                _ => {
                    let s1 = serde_json::to_string(a).unwrap_or_default();
                    let s2 = serde_json::to_string(b).unwrap_or_default();
                    s1.cmp(&s2)
                }
            }
        }

        let mut left = 0i64;
        let mut right = arr.len() as i64 - 1;

        while left <= right {
            let mid = left + (right - left) / 2;

            match compare_values(&arr[mid as usize], target) {
                std::cmp::Ordering::Equal => {
                    return Ok(Value::Number(Number::from(mid)));
                }
                std::cmp::Ordering::Less => {
                    left = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    right = mid - 1;
                }
            }
        }

        // Not found - return -(insertion_point) - 1
        // At this point, left is the insertion point
        let result = -(left) - 1;
        Ok(Value::Number(Number::from(result)))
    }
}

// =============================================================================
// repeat_array(value, n) -> array (create array with value repeated n times)
// =============================================================================

defn!(RepeatArrayFn, vec![arg!(any), arg!(number)], None);

impl Function for RepeatArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = &args[0];
        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for count argument"))?
            as i64;

        if n < 0 {
            return Err(custom_error(ctx, "Count must be non-negative"));
        }

        let result: Vec<Value> = (0..n).map(|_| value.clone()).collect();
        Ok(Value::Array(result))
    }
}

// =============================================================================
// cycle(array, n) -> array (cycle through array n times)
// =============================================================================

defn!(CycleFn, vec![arg!(array), arg!(number)], None);

impl Function for CycleFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for count argument"))?
            as i64;

        if n < 0 {
            return Err(custom_error(ctx, "Count must be non-negative"));
        }

        if arr.is_empty() || n == 0 {
            return Ok(Value::Array(vec![]));
        }

        let mut result: Vec<Value> = Vec::with_capacity(arr.len() * n as usize);
        for _ in 0..n {
            result.extend(arr.iter().cloned());
        }
        Ok(Value::Array(result))
    }
}
