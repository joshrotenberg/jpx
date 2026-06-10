//! Object/map manipulation functions.

use std::collections::HashSet;

use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTrainCase,
    ToUpperCamelCase,
};
use serde_json::{Map, Number, Value};

use crate::functions::{Function, custom_error, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register object functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "items", enabled, Box::new(EntriesFn::new()));
    register_if_enabled(
        runtime,
        "from_items",
        enabled,
        Box::new(FromEntriesFn::new()),
    );
    register_if_enabled(
        runtime,
        "from_entries",
        enabled,
        Box::new(FromEntriesFn::new()),
    );
    register_if_enabled(
        runtime,
        "with_entries",
        enabled,
        Box::new(WithEntriesFn::new()),
    );
    register_if_enabled(runtime, "pick", enabled, Box::new(PickFn::new()));
    register_if_enabled(runtime, "omit", enabled, Box::new(OmitFn::new()));
    register_if_enabled(runtime, "invert", enabled, Box::new(InvertFn::new()));
    register_if_enabled(
        runtime,
        "rename_keys",
        enabled,
        Box::new(RenameKeysFn::new()),
    );
    register_if_enabled(
        runtime,
        "flatten_keys",
        enabled,
        Box::new(FlattenKeysFn::new()),
    );
    register_if_enabled(runtime, "flatten", enabled, Box::new(FlattenKeysFn::new()));
    register_if_enabled(
        runtime,
        "unflatten_keys",
        enabled,
        Box::new(UnflattenKeysFn::new()),
    );
    register_if_enabled(
        runtime,
        "unflatten",
        enabled,
        Box::new(UnflattenKeysFn::new()),
    );
    register_if_enabled(
        runtime,
        "flatten_array",
        enabled,
        Box::new(FlattenArrayFn::new()),
    );
    register_if_enabled(runtime, "deep_merge", enabled, Box::new(DeepMergeFn::new()));
    register_if_enabled(
        runtime,
        "deep_equals",
        enabled,
        Box::new(DeepEqualsFn::new()),
    );
    register_if_enabled(runtime, "deep_diff", enabled, Box::new(DeepDiffFn::new()));
    register_if_enabled(runtime, "get", enabled, Box::new(GetFn::new()));
    register_if_enabled(runtime, "get_path", enabled, Box::new(GetFn::new()));
    register_if_enabled(runtime, "has", enabled, Box::new(HasFn::new()));
    register_if_enabled(runtime, "has_path", enabled, Box::new(HasFn::new()));
    register_if_enabled(runtime, "defaults", enabled, Box::new(DefaultsFn::new()));
    register_if_enabled(
        runtime,
        "defaults_deep",
        enabled,
        Box::new(DefaultsDeepFn::new()),
    );
    register_if_enabled(runtime, "set_path", enabled, Box::new(SetPathFn::new()));
    register_if_enabled(
        runtime,
        "delete_path",
        enabled,
        Box::new(DeletePathFn::new()),
    );
    register_if_enabled(runtime, "paths", enabled, Box::new(PathsFn::new()));
    register_if_enabled(runtime, "leaves", enabled, Box::new(LeavesFn::new()));
    register_if_enabled(
        runtime,
        "leaves_with_paths",
        enabled,
        Box::new(LeavesWithPathsFn::new()),
    );
    register_if_enabled(
        runtime,
        "remove_nulls",
        enabled,
        Box::new(RemoveNullsFn::new()),
    );
    register_if_enabled(
        runtime,
        "remove_empty",
        enabled,
        Box::new(RemoveEmptyFn::new()),
    );
    register_if_enabled(
        runtime,
        "remove_empty_strings",
        enabled,
        Box::new(RemoveEmptyStringsFn::new()),
    );
    register_if_enabled(
        runtime,
        "compact_deep",
        enabled,
        Box::new(CompactDeepFn::new()),
    );
    register_if_enabled(
        runtime,
        "completeness",
        enabled,
        Box::new(CompletenessFn::new()),
    );
    register_if_enabled(
        runtime,
        "type_consistency",
        enabled,
        Box::new(TypeConsistencyFn::new()),
    );
    register_if_enabled(
        runtime,
        "data_quality_score",
        enabled,
        Box::new(DataQualityScoreFn::new()),
    );
    register_if_enabled(runtime, "redact", enabled, Box::new(RedactFn::new()));
    register_if_enabled(
        runtime,
        "redact_keys",
        enabled,
        Box::new(RedactKeysFn::new()),
    );
    register_if_enabled(runtime, "mask", enabled, Box::new(MaskFn::new()));
    register_if_enabled(runtime, "pluck_deep", enabled, Box::new(PluckDeepFn::new()));
    register_if_enabled(runtime, "paths_to", enabled, Box::new(PathsToFn::new()));
    register_if_enabled(runtime, "snake_keys", enabled, Box::new(SnakeKeysFn::new()));
    register_if_enabled(runtime, "camel_keys", enabled, Box::new(CamelKeysFn::new()));
    register_if_enabled(runtime, "kebab_keys", enabled, Box::new(KebabKeysFn::new()));
    register_if_enabled(
        runtime,
        "pascal_keys",
        enabled,
        Box::new(PascalKeysFn::new()),
    );
    register_if_enabled(
        runtime,
        "shouty_snake_keys",
        enabled,
        Box::new(ShoutySnakeKeysFn::new()),
    );
    register_if_enabled(
        runtime,
        "shouty_kebab_keys",
        enabled,
        Box::new(ShoutyKebabKeysFn::new()),
    );
    register_if_enabled(runtime, "train_keys", enabled, Box::new(TrainKeysFn::new()));
    register_if_enabled(
        runtime,
        "structural_diff",
        enabled,
        Box::new(StructuralDiffFn::new()),
    );
    register_if_enabled(
        runtime,
        "has_same_shape",
        enabled,
        Box::new(HasSameShapeFn::new()),
    );
    register_if_enabled(
        runtime,
        "infer_schema",
        enabled,
        Box::new(InferSchemaFn::new()),
    );
    register_if_enabled(
        runtime,
        "chunk_by_size",
        enabled,
        Box::new(ChunkBySizeFn::new()),
    );
    register_if_enabled(runtime, "paginate", enabled, Box::new(PaginateFn::new()));
    register_if_enabled(
        runtime,
        "estimate_size",
        enabled,
        Box::new(EstimateSizeFn::new()),
    );
    register_if_enabled(
        runtime,
        "truncate_to_size",
        enabled,
        Box::new(TruncateToSizeFn::new()),
    );
    register_if_enabled(runtime, "template", enabled, Box::new(TemplateFn::new()));
    register_if_enabled(
        runtime,
        "template_strict",
        enabled,
        Box::new(TemplateStrictFn::new()),
    );
}

// =============================================================================
// items(object) -> array of [key, value] pairs (JEP-013)
// =============================================================================

defn!(EntriesFn, vec![arg!(object)], None);

impl Function for EntriesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let entries: Vec<Value> = obj
            .iter()
            .map(|(k, v)| {
                let pair = vec![Value::String(k.clone()), v.clone()];
                Value::Array(pair)
            })
            .collect();

        Ok(Value::Array(entries))
    }
}

// =============================================================================
// from_items(array) -> object from array of [key, value] pairs (JEP-013)
// =============================================================================

defn!(FromEntriesFn, vec![arg!(array)], None);

impl Function for FromEntriesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut result = Map::new();

        for item in arr {
            if let Some(pair) = item.as_array()
                && pair.len() >= 2
                && let Some(key_str) = pair[0].as_str()
            {
                result.insert(key_str.to_string(), pair[1].clone());
            }
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// with_entries(object, expr) -> object (transform entries, jq parity)
// =============================================================================

defn!(WithEntriesFn, vec![arg!(object), arg!(string)], None);

impl Function for WithEntriesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let expr_str = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected expression string"))?;

        let compiled = ctx
            .runtime
            .compile(expr_str)
            .map_err(|_| custom_error(ctx, "Invalid expression in with_entries"))?;

        let mut result = Map::new();

        for (key, value) in obj.iter() {
            let entry = Value::Array(vec![Value::String(key.clone()), value.clone()]);

            let transformed = compiled
                .search(&entry)
                .map_err(|_| custom_error(ctx, "Expression error in with_entries"))?;

            if transformed.is_null() {
                continue;
            }

            if let Some(pair) = transformed.as_array()
                && pair.len() >= 2
                && let Some(new_key) = pair[0].as_str()
            {
                result.insert(new_key.to_string(), pair[1].clone());
            }
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// pick(object, keys) -> object (select specific keys)
// =============================================================================

defn!(PickFn, vec![arg!(object), arg!(array)], None);

impl Function for PickFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let keys_arr = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array of keys"))?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_str().map(|s| s.to_string()))
            .collect();

        let result: Map<String, Value> = obj
            .iter()
            .filter(|(k, _)| keys.contains(k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Value::Object(result))
    }
}

// =============================================================================
// omit(object, keys) -> object (exclude specific keys)
// =============================================================================

defn!(OmitFn, vec![arg!(object), arg!(array)], None);

impl Function for OmitFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let keys_arr = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array of keys"))?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_str().map(|s| s.to_string()))
            .collect();

        let result: Map<String, Value> = obj
            .iter()
            .filter(|(k, _)| !keys.contains(k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Value::Object(result))
    }
}

// =============================================================================
// invert(object) -> object (swap keys and values)
// =============================================================================

defn!(InvertFn, vec![arg!(object)], None);

impl Function for InvertFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let mut result = Map::new();

        for (k, v) in obj.iter() {
            let new_key = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => continue,
            };
            result.insert(new_key, Value::String(k.clone()));
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// rename_keys(object, mapping) -> object
// =============================================================================

defn!(RenameKeysFn, vec![arg!(object), arg!(object)], None);

impl Function for RenameKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let mapping = args[1]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected mapping object"))?;

        let rename_map: std::collections::HashMap<String, String> = mapping
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect();

        let result: Map<String, Value> = obj
            .iter()
            .map(|(k, v)| {
                let new_key = rename_map.get(k).cloned().unwrap_or_else(|| k.clone());
                (new_key, v.clone())
            })
            .collect();

        Ok(Value::Object(result))
    }
}

// =============================================================================
// flatten_keys(object, separator?) -> object
// =============================================================================

defn!(FlattenKeysFn, vec![arg!(object)], Some(arg!(string)));

fn flatten_object(
    obj: &Map<String, Value>,
    prefix: &str,
    separator: &str,
    result: &mut Map<String, Value>,
) {
    for (k, v) in obj.iter() {
        let new_key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{}{}{}", prefix, separator, k)
        };

        if let Some(nested) = v.as_object() {
            flatten_object(nested, &new_key, separator, result);
        } else {
            result.insert(new_key, v.clone());
        }
    }
}

impl Function for FlattenKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_str().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result = Map::new();
        flatten_object(obj, "", &separator, &mut result);

        Ok(Value::Object(result))
    }
}

// =============================================================================
// unflatten_keys(object, separator?) -> object
// =============================================================================

defn!(UnflattenKeysFn, vec![arg!(object)], Some(arg!(string)));

fn insert_nested(obj: &mut Map<String, Value>, parts: &[&str], value: Value) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        obj.insert(parts[0].to_string(), value);
        return;
    }

    let key = parts[0].to_string();
    let rest = &parts[1..];

    let nested = obj
        .entry(key.clone())
        .or_insert_with(|| Value::Object(Map::new()));

    if let Some(nested_obj) = nested.as_object() {
        let mut new_obj = nested_obj.clone();
        insert_nested(&mut new_obj, rest, value);
        *obj.get_mut(&key).unwrap() = Value::Object(new_obj);
    }
}

impl Function for UnflattenKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_str().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result = Map::new();

        for (key, value) in obj.iter() {
            let parts: Vec<&str> = key.split(&separator).collect();
            insert_nested(&mut result, &parts, value.clone());
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// flatten_array(any, separator?) -> object
// Flattens nested objects AND arrays with numeric indices
// =============================================================================

defn!(FlattenArrayFn, vec![arg!(any)], Some(arg!(string)));

fn flatten_value(value: &Value, prefix: &str, separator: &str, result: &mut Map<String, Value>) {
    match value {
        Value::Object(obj) => {
            if obj.is_empty() {
                if !prefix.is_empty() {
                    result.insert(prefix.to_string(), Value::Object(obj.clone()));
                }
            } else {
                for (k, v) in obj.iter() {
                    let new_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}{}{}", prefix, separator, k)
                    };
                    flatten_value(v, &new_key, separator, result);
                }
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                if !prefix.is_empty() {
                    result.insert(prefix.to_string(), Value::Array(arr.clone()));
                }
            } else {
                for (idx, v) in arr.iter().enumerate() {
                    let new_key = if prefix.is_empty() {
                        idx.to_string()
                    } else {
                        format!("{}{}{}", prefix, separator, idx)
                    };
                    flatten_value(v, &new_key, separator, result);
                }
            }
        }
        _ => {
            if !prefix.is_empty() {
                result.insert(prefix.to_string(), value.clone());
            } else {
                result.insert(String::new(), value.clone());
            }
        }
    }
}

impl Function for FlattenArrayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_str().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result = Map::new();
        flatten_value(&args[0], "", &separator, &mut result);

        Ok(Value::Object(result))
    }
}

// =============================================================================
// deep_merge(obj1, obj2) -> object
// =============================================================================

defn!(DeepMergeFn, vec![arg!(object), arg!(object)], None);

fn deep_merge_objects(
    base: &Map<String, Value>,
    overlay: &Map<String, Value>,
) -> Map<String, Value> {
    let mut result = base.clone();

    for (key, overlay_value) in overlay {
        if let Some(base_value) = result.get(key) {
            if let (Some(base_obj), Some(overlay_obj)) =
                (base_value.as_object(), overlay_value.as_object())
            {
                let merged = deep_merge_objects(base_obj, overlay_obj);
                result.insert(key.clone(), Value::Object(merged));
            } else {
                result.insert(key.clone(), overlay_value.clone());
            }
        } else {
            result.insert(key.clone(), overlay_value.clone());
        }
    }

    result
}

impl Function for DeepMergeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj1 = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let obj2 = args[1]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let merged = deep_merge_objects(obj1, obj2);
        Ok(Value::Object(merged))
    }
}

// =============================================================================
// deep_equals(a, b) -> boolean
// =============================================================================

defn!(DeepEqualsFn, vec![arg!(any), arg!(any)], None);

impl Function for DeepEqualsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a_json = serde_json::to_string(&args[0]).unwrap_or_default();
        let b_json = serde_json::to_string(&args[1]).unwrap_or_default();

        Ok(Value::Bool(a_json == b_json))
    }
}

// =============================================================================
// deep_diff(a, b) -> object with added, removed, changed
// =============================================================================

defn!(DeepDiffFn, vec![arg!(object), arg!(object)], None);

fn compute_deep_diff(a: &Map<String, Value>, b: &Map<String, Value>) -> Map<String, Value> {
    let mut added = Map::new();
    let mut removed = Map::new();
    let mut changed = Map::new();

    for (key, a_value) in a.iter() {
        match b.get(key) {
            None => {
                removed.insert(key.clone(), a_value.clone());
            }
            Some(b_value) => {
                let a_json = serde_json::to_string(a_value).unwrap_or_default();
                let b_json = serde_json::to_string(b_value).unwrap_or_default();

                if a_json != b_json {
                    if let (Some(a_obj), Some(b_obj)) = (a_value.as_object(), b_value.as_object()) {
                        let nested_diff = compute_deep_diff(a_obj, b_obj);
                        changed.insert(key.clone(), Value::Object(nested_diff));
                    } else {
                        let mut change_obj = Map::new();
                        change_obj.insert("from".to_string(), a_value.clone());
                        change_obj.insert("to".to_string(), b_value.clone());
                        changed.insert(key.clone(), Value::Object(change_obj));
                    }
                }
            }
        }
    }

    for (key, b_value) in b.iter() {
        if !a.contains_key(key) {
            added.insert(key.clone(), b_value.clone());
        }
    }

    let mut result = Map::new();
    result.insert("added".to_string(), Value::Object(added));
    result.insert("removed".to_string(), Value::Object(removed));
    result.insert("changed".to_string(), Value::Object(changed));

    result
}

impl Function for DeepDiffFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj_a = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let obj_b = args[1]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let diff = compute_deep_diff(obj_a, obj_b);
        Ok(Value::Object(diff))
    }
}

// =============================================================================
// get(object, path, default?) -> value at path or default
// =============================================================================

defn!(GetFn, vec![arg!(any), arg!(string)], Some(arg!(any)));

fn get_at_path(value: &Value, path: &str) -> Option<Value> {
    if path.is_empty() {
        return Some(value.clone());
    }

    let mut current = value.clone();

    let parts = parse_path_parts(path);

    for part in parts {
        if let Some(idx) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            if let Ok(index) = idx.parse::<usize>() {
                if let Some(arr) = current.as_array() {
                    if index < arr.len() {
                        current = arr[index].clone();
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else if let Ok(index) = part.parse::<usize>() {
            if let Some(arr) = current.as_array() {
                if index < arr.len() {
                    current = arr[index].clone();
                } else {
                    return None;
                }
            } else if let Some(obj) = current.as_object() {
                if let Some(val) = obj.get(&part) {
                    current = val.clone();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else if let Some(obj) = current.as_object() {
            if let Some(val) = obj.get(&part) {
                current = val.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(current)
}

fn parse_path_parts(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                let mut bracket = String::from("[");
                while let Some(&next) = chars.peek() {
                    bracket.push(chars.next().unwrap());
                    if next == ']' {
                        break;
                    }
                }
                parts.push(bracket);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

impl Function for GetFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let path = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string path argument"))?;

        let default_val = if args.len() > 2 {
            args[2].clone()
        } else {
            Value::Null
        };

        match get_at_path(&args[0], path) {
            Some(val) => Ok(val),
            None => Ok(default_val),
        }
    }
}

// =============================================================================
// has(object, path) -> boolean
// =============================================================================

defn!(HasFn, vec![arg!(any), arg!(string)], None);

impl Function for HasFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let path = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string path argument"))?;

        let exists = get_at_path(&args[0], path).is_some();
        Ok(Value::Bool(exists))
    }
}

// =============================================================================
// defaults(object, defaults) -> object with defaults applied
// =============================================================================

defn!(DefaultsFn, vec![arg!(object), arg!(object)], None);

impl Function for DefaultsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let defaults = args[1]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let mut result = obj.clone();

        for (key, value) in defaults.iter() {
            if !result.contains_key(key) {
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(Value::Object(result))
    }
}

// =============================================================================
// defaults_deep(object, defaults) -> object with deep defaults applied
// =============================================================================

defn!(DefaultsDeepFn, vec![arg!(object), arg!(object)], None);

fn apply_defaults_deep(
    obj: &Map<String, Value>,
    defaults: &Map<String, Value>,
) -> Map<String, Value> {
    let mut result = obj.clone();

    for (key, default_value) in defaults.iter() {
        if let Some(existing) = result.get(key) {
            if let (Some(existing_obj), Some(default_obj)) =
                (existing.as_object(), default_value.as_object())
            {
                let merged = apply_defaults_deep(existing_obj, default_obj);
                result.insert(key.clone(), Value::Object(merged));
            }
        } else {
            result.insert(key.clone(), default_value.clone());
        }
    }

    result
}

impl Function for DefaultsDeepFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let defaults = args[1]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let result = apply_defaults_deep(obj, defaults);
        Ok(Value::Object(result))
    }
}

// =============================================================================
// set_path(object, path, value) -> new object with value set at path
// =============================================================================

defn!(SetPathFn, vec![arg!(any), arg!(string), arg!(any)], None);

impl Function for SetPathFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let path = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string path argument"))?;

        let value = args[2].clone();

        let parts = parse_path_for_mutation(path);
        if parts.is_empty() {
            return Ok(value);
        }

        let result = set_at_path(&args[0], &parts, value);
        Ok(result)
    }
}

fn parse_path_for_mutation(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![];
    }

    if path.starts_with('/') {
        parse_json_pointer(path)
    } else {
        parse_path_parts_for_mutation(path)
    }
}

fn parse_path_parts_for_mutation(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                let mut index = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    index.push(chars.next().unwrap());
                }
                parts.push(index);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn parse_json_pointer(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![];
    }

    let path = path.strip_prefix('/').unwrap_or(path);

    if path.is_empty() {
        return vec![];
    }

    path.split('/')
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn set_at_path(value: &Value, parts: &[String], new_value: Value) -> Value {
    if parts.is_empty() {
        return new_value;
    }

    let key = &parts[0];
    let remaining = &parts[1..];

    match value {
        Value::Object(obj) => {
            let mut new_obj = obj.clone();
            if remaining.is_empty() {
                new_obj.insert(key.clone(), new_value);
            } else {
                let existing = obj.get(key).cloned().unwrap_or(Value::Null);
                new_obj.insert(key.clone(), set_at_path(&existing, remaining, new_value));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                let mut new_arr = arr.clone();
                while new_arr.len() <= idx {
                    new_arr.push(Value::Null);
                }
                if remaining.is_empty() {
                    new_arr[idx] = new_value;
                } else {
                    new_arr[idx] = set_at_path(
                        &arr.get(idx).cloned().unwrap_or(Value::Null),
                        remaining,
                        new_value,
                    );
                }
                Value::Array(new_arr)
            } else {
                value.clone()
            }
        }
        _ => {
            if remaining.is_empty() {
                let mut new_obj = Map::new();
                new_obj.insert(key.clone(), new_value);
                Value::Object(new_obj)
            } else {
                let mut new_obj = Map::new();
                new_obj.insert(key.clone(), set_at_path(&Value::Null, remaining, new_value));
                Value::Object(new_obj)
            }
        }
    }
}

// =============================================================================
// delete_path(object, path) -> new object with value removed at path
// =============================================================================

defn!(DeletePathFn, vec![arg!(any), arg!(string)], None);

impl Function for DeletePathFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let path = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string path argument"))?;

        let parts = parse_path_for_mutation(path);
        if parts.is_empty() {
            return Ok(Value::Null);
        }

        let result = delete_at_path(&args[0], &parts);
        Ok(result)
    }
}

fn delete_at_path(value: &Value, parts: &[String]) -> Value {
    if parts.is_empty() {
        return Value::Null;
    }

    let key = &parts[0];
    let remaining = &parts[1..];

    match value {
        Value::Object(obj) => {
            let mut new_obj = obj.clone();
            if remaining.is_empty() {
                new_obj.remove(key);
            } else if let Some(existing) = obj.get(key) {
                new_obj.insert(key.clone(), delete_at_path(existing, remaining));
            }
            Value::Object(new_obj)
        }
        Value::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                if idx < arr.len() {
                    let mut new_arr = arr.clone();
                    if remaining.is_empty() {
                        new_arr.remove(idx);
                    } else {
                        new_arr[idx] = delete_at_path(&arr[idx], remaining);
                    }
                    Value::Array(new_arr)
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

// =============================================================================
// paths(value) -> array of all JSON pointer paths in the value
// =============================================================================

defn!(PathsFn, vec![arg!(any)], None);

impl Function for PathsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut paths = Vec::new();
        collect_paths(&args[0], String::new(), &mut paths);

        let result: Vec<Value> = paths.into_iter().map(Value::String).collect();

        Ok(Value::Array(result))
    }
}

fn collect_paths(value: &Value, current_path: String, paths: &mut Vec<String>) {
    match value {
        Value::Object(obj) => {
            if !current_path.is_empty() {
                paths.push(current_path.clone());
            }
            for (key, val) in obj.iter() {
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                let new_path = format!("{}/{}", current_path, escaped_key);
                collect_paths(val, new_path, paths);
            }
        }
        Value::Array(arr) => {
            if !current_path.is_empty() {
                paths.push(current_path.clone());
            }
            for (idx, val) in arr.iter().enumerate() {
                let new_path = format!("{}/{}", current_path, idx);
                collect_paths(val, new_path, paths);
            }
        }
        _ => {
            if !current_path.is_empty() {
                paths.push(current_path);
            }
        }
    }
}

// =============================================================================
// leaves(value) -> array of all leaf values (non-object, non-array)
// =============================================================================

defn!(LeavesFn, vec![arg!(any)], None);

impl Function for LeavesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut leaves = Vec::new();
        collect_leaves(&args[0], &mut leaves);

        Ok(Value::Array(leaves))
    }
}

fn collect_leaves(value: &Value, leaves: &mut Vec<Value>) {
    match value {
        Value::Object(obj) => {
            for (_, val) in obj.iter() {
                collect_leaves(val, leaves);
            }
        }
        Value::Array(arr) => {
            for val in arr.iter() {
                collect_leaves(val, leaves);
            }
        }
        _ => {
            leaves.push(value.clone());
        }
    }
}

// =============================================================================
// leaves_with_paths(value) -> array of {path, value} objects for all leaves
// =============================================================================

defn!(LeavesWithPathsFn, vec![arg!(any)], None);

impl Function for LeavesWithPathsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut leaves = Vec::new();
        collect_leaves_with_paths(&args[0], String::new(), &mut leaves);

        let result: Vec<Value> = leaves
            .into_iter()
            .map(|(path, value)| {
                let mut obj = Map::new();
                obj.insert("path".to_string(), Value::String(path));
                obj.insert("value".to_string(), value);
                Value::Object(obj)
            })
            .collect();

        Ok(Value::Array(result))
    }
}

fn collect_leaves_with_paths(
    value: &Value,
    current_path: String,
    leaves: &mut Vec<(String, Value)>,
) {
    match value {
        Value::Object(obj) => {
            if obj.is_empty() && !current_path.is_empty() {
                leaves.push((current_path, value.clone()));
            } else {
                for (key, val) in obj.iter() {
                    let escaped_key = key.replace('~', "~0").replace('/', "~1");
                    let new_path = format!("{}/{}", current_path, escaped_key);
                    collect_leaves_with_paths(val, new_path, leaves);
                }
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() && !current_path.is_empty() {
                leaves.push((current_path, value.clone()));
            } else {
                for (idx, val) in arr.iter().enumerate() {
                    let new_path = format!("{}/{}", current_path, idx);
                    collect_leaves_with_paths(val, new_path, leaves);
                }
            }
        }
        _ => {
            let path = if current_path.is_empty() {
                "/".to_string()
            } else {
                current_path
            };
            leaves.push((path, value.clone()));
        }
    }
}

// =============================================================================
// remove_nulls(any) -> any (recursively remove null values)
// =============================================================================

defn!(RemoveNullsFn, vec![arg!(any)], None);

impl Function for RemoveNullsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(remove_nulls_recursive(&args[0]))
    }
}

fn is_null_value(value: &Value) -> bool {
    value.is_null()
}

fn remove_nulls_recursive(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let cleaned: Map<String, Value> = obj
                .iter()
                .filter(|(_, v)| !is_null_value(v))
                .map(|(k, v)| (k.clone(), remove_nulls_recursive(v)))
                .collect();
            Value::Object(cleaned)
        }
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr
                .iter()
                .filter(|v| !is_null_value(v))
                .map(remove_nulls_recursive)
                .collect();
            Value::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// remove_empty(any) -> any
// =============================================================================

defn!(RemoveEmptyFn, vec![arg!(any)], None);

impl Function for RemoveEmptyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(remove_empty_recursive(&args[0]))
    }
}

fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(arr) => arr.is_empty(),
        Value::Object(obj) => obj.is_empty(),
        _ => false,
    }
}

fn remove_empty_recursive(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let cleaned: Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), remove_empty_recursive(v)))
                .filter(|(_, v)| !is_empty_value(v))
                .collect();
            Value::Object(cleaned)
        }
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr
                .iter()
                .map(remove_empty_recursive)
                .filter(|v| !is_empty_value(v))
                .collect();
            Value::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// remove_empty_strings(any) -> any
// =============================================================================

defn!(RemoveEmptyStringsFn, vec![arg!(any)], None);

impl Function for RemoveEmptyStringsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(remove_empty_strings_recursive(&args[0]))
    }
}

fn is_empty_string(value: &Value) -> bool {
    matches!(value, Value::String(s) if s.is_empty())
}

fn remove_empty_strings_recursive(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            let cleaned: Map<String, Value> = obj
                .iter()
                .filter(|(_, v)| !is_empty_string(v))
                .map(|(k, v)| (k.clone(), remove_empty_strings_recursive(v)))
                .collect();
            Value::Object(cleaned)
        }
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr
                .iter()
                .filter(|v| !is_empty_string(v))
                .map(remove_empty_strings_recursive)
                .collect();
            Value::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// compact_deep(array) -> array
// =============================================================================

defn!(CompactDeepFn, vec![arg!(array)], None);

impl Function for CompactDeepFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(compact_deep_recursive(&args[0]))
    }
}

fn compact_deep_recursive(value: &Value) -> Value {
    match value {
        Value::Array(arr) => {
            let cleaned: Vec<Value> = arr
                .iter()
                .filter(|v| !is_null_value(v))
                .map(compact_deep_recursive)
                .collect();
            Value::Array(cleaned)
        }
        Value::Object(obj) => {
            let cleaned: Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), compact_deep_recursive(v)))
                .collect();
            Value::Object(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// completeness(object) -> number (0-100)
// =============================================================================

defn!(CompletenessFn, vec![arg!(object)], None);

impl Function for CompletenessFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "completeness: expected object"))?;

        if obj.is_empty() {
            return Ok(number_value(100.0));
        }

        let mut total_fields = 0;
        let mut non_null_fields = 0;

        count_completeness(&args[0], &mut total_fields, &mut non_null_fields);

        let score = if total_fields > 0 {
            (non_null_fields as f64 / total_fields as f64) * 100.0
        } else {
            100.0
        };

        Ok(number_value(score))
    }
}

fn count_completeness(value: &Value, total: &mut usize, non_null: &mut usize) {
    match value {
        Value::Object(obj) => {
            for (_, v) in obj.iter() {
                *total += 1;
                if !v.is_null() {
                    *non_null += 1;
                }
                count_completeness(v, total, non_null);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter() {
                count_completeness(item, total, non_null);
            }
        }
        _ => {}
    }
}

// =============================================================================
// type_consistency(array) -> object
// =============================================================================

defn!(TypeConsistencyFn, vec![arg!(array)], None);

impl Function for TypeConsistencyFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "type_consistency: expected array"))?;

        if arr.is_empty() {
            let mut result = Map::new();
            result.insert("consistent".to_string(), Value::Bool(true));
            result.insert("types".to_string(), Value::Array(vec![]));
            result.insert("inconsistencies".to_string(), Value::Array(vec![]));
            return Ok(Value::Object(result));
        }

        let first_element = &arr[0];
        if let Some(first_obj) = first_element.as_object() {
            return check_object_array_consistency(arr, first_obj);
        }

        let mut type_counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for item in arr.iter() {
            let type_name = get_type_name(item);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        let types: Vec<Value> = type_counts
            .keys()
            .map(|t| Value::String(t.clone()))
            .collect();

        let consistent = type_counts.len() == 1;

        let mut result = Map::new();
        result.insert("consistent".to_string(), Value::Bool(consistent));
        result.insert("types".to_string(), Value::Array(types));
        result.insert("inconsistencies".to_string(), Value::Array(vec![]));

        Ok(Value::Object(result))
    }
}

fn check_object_array_consistency(arr: &[Value], first_obj: &Map<String, Value>) -> SearchResult {
    let mut expected_types: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for (key, val) in first_obj.iter() {
        expected_types.insert(key.clone(), get_type_name(val));
    }

    let mut inconsistencies: Vec<Value> = Vec::new();

    for (idx, item) in arr.iter().enumerate().skip(1) {
        if let Some(obj) = item.as_object() {
            for (key, val) in obj.iter() {
                let actual_type = get_type_name(val);
                if let Some(expected) = expected_types.get(key)
                    && &actual_type != expected
                    && actual_type != "null"
                    && expected != "null"
                {
                    let mut issue = Map::new();
                    issue.insert("index".to_string(), Value::Number(Number::from(idx as i64)));
                    issue.insert("field".to_string(), Value::String(key.clone()));
                    issue.insert("expected".to_string(), Value::String(expected.clone()));
                    issue.insert("got".to_string(), Value::String(actual_type));
                    inconsistencies.push(Value::Object(issue));
                }
            }
        }
    }

    let types: Vec<Value> = expected_types
        .iter()
        .map(|(k, v)| {
            let mut obj = Map::new();
            obj.insert("field".to_string(), Value::String(k.clone()));
            obj.insert("type".to_string(), Value::String(v.clone()));
            Value::Object(obj)
        })
        .collect();

    let mut result = Map::new();
    result.insert(
        "consistent".to_string(),
        Value::Bool(inconsistencies.is_empty()),
    );
    result.insert("types".to_string(), Value::Array(types));
    result.insert("inconsistencies".to_string(), Value::Array(inconsistencies));

    Ok(Value::Object(result))
}

fn get_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

// =============================================================================
// data_quality_score(any) -> object
// =============================================================================

defn!(DataQualityScoreFn, vec![arg!(any)], None);

impl Function for DataQualityScoreFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let value = &args[0];

        let mut stats = QualityStats::default();
        analyze_quality(value, String::new(), &mut stats);

        let total_issues = stats.null_count + stats.empty_string_count + stats.type_issues.len();
        let score = if stats.total_fields == 0 {
            100.0
        } else {
            let issue_ratio = total_issues as f64 / stats.total_fields as f64;
            (100.0 * (1.0 - issue_ratio)).max(0.0)
        };

        let mut issues: Vec<Value> = Vec::new();

        for path in &stats.null_paths {
            let mut issue = Map::new();
            issue.insert("path".to_string(), Value::String(path.clone()));
            issue.insert("issue".to_string(), Value::String("null".to_string()));
            issues.push(Value::Object(issue));
        }

        for path in &stats.empty_string_paths {
            let mut issue = Map::new();
            issue.insert("path".to_string(), Value::String(path.clone()));
            issue.insert(
                "issue".to_string(),
                Value::String("empty_string".to_string()),
            );
            issues.push(Value::Object(issue));
        }

        for ti in &stats.type_issues {
            let mut issue = Map::new();
            issue.insert("path".to_string(), Value::String(ti.path.clone()));
            issue.insert(
                "issue".to_string(),
                Value::String("type_mismatch".to_string()),
            );
            issue.insert("expected".to_string(), Value::String(ti.expected.clone()));
            issue.insert("got".to_string(), Value::String(ti.got.clone()));
            issues.push(Value::Object(issue));
        }

        let mut result = Map::new();
        result.insert("score".to_string(), number_value(score));
        result.insert(
            "total_fields".to_string(),
            Value::Number(Number::from(stats.total_fields as i64)),
        );
        result.insert(
            "null_count".to_string(),
            Value::Number(Number::from(stats.null_count as i64)),
        );
        result.insert(
            "empty_string_count".to_string(),
            Value::Number(Number::from(stats.empty_string_count as i64)),
        );
        result.insert(
            "type_inconsistencies".to_string(),
            Value::Number(Number::from(stats.type_issues.len() as i64)),
        );
        result.insert("issues".to_string(), Value::Array(issues));

        Ok(Value::Object(result))
    }
}

#[derive(Default)]
struct QualityStats {
    total_fields: usize,
    null_count: usize,
    empty_string_count: usize,
    null_paths: Vec<String>,
    empty_string_paths: Vec<String>,
    type_issues: Vec<TypeIssue>,
}

struct TypeIssue {
    path: String,
    expected: String,
    got: String,
}

fn analyze_quality(value: &Value, path: String, stats: &mut QualityStats) {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj.iter() {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                stats.total_fields += 1;

                match val {
                    Value::Null => {
                        stats.null_count += 1;
                        stats.null_paths.push(field_path.clone());
                    }
                    Value::String(s) if s.is_empty() => {
                        stats.empty_string_count += 1;
                        stats.empty_string_paths.push(field_path.clone());
                    }
                    _ => {}
                }

                analyze_quality(val, field_path, stats);
            }
        }
        Value::Array(arr) => {
            if arr.len() > 1
                && let Some(Value::Object(first_obj)) = arr.first()
            {
                let expected_types: std::collections::BTreeMap<String, String> = first_obj
                    .iter()
                    .map(|(k, v)| (k.clone(), get_type_name(v)))
                    .collect();

                for (idx, item) in arr.iter().enumerate().skip(1) {
                    if let Value::Object(obj) = item {
                        for (key, val) in obj.iter() {
                            let actual_type = get_type_name(val);
                            if let Some(expected) = expected_types.get(key)
                                && &actual_type != expected
                                && actual_type != "null"
                                && expected != "null"
                            {
                                stats.type_issues.push(TypeIssue {
                                    path: format!("{}[{}].{}", path, idx, key),
                                    expected: expected.clone(),
                                    got: actual_type,
                                });
                            }
                        }
                    }
                }
            }

            for (idx, item) in arr.iter().enumerate() {
                let item_path = format!("{}[{}]", path, idx);
                analyze_quality(item, item_path, stats);
            }
        }
        _ => {}
    }
}

// =============================================================================
// redact(any, keys) -> any (replace values at keys with [REDACTED])
// =============================================================================

defn!(RedactFn, vec![arg!(any), arg!(array)], None);

impl Function for RedactFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let keys_arr = args[1]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array of keys"))?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_str().map(|s| s.to_string()))
            .collect();

        Ok(redact_recursive(&args[0], &keys))
    }
}

fn redact_recursive(value: &Value, keys: &HashSet<String>) -> Value {
    match value {
        Value::Object(obj) => {
            let redacted: Map<String, Value> = obj
                .iter()
                .map(|(k, v)| {
                    if keys.contains(k) {
                        (k.clone(), Value::String("[REDACTED]".to_string()))
                    } else {
                        (k.clone(), redact_recursive(v, keys))
                    }
                })
                .collect();
            Value::Object(redacted)
        }
        Value::Array(arr) => {
            let redacted: Vec<Value> = arr.iter().map(|v| redact_recursive(v, keys)).collect();
            Value::Array(redacted)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// redact_keys(any, pattern) -> any (redact keys matching regex pattern)
// =============================================================================

defn!(RedactKeysFn, vec![arg!(any), arg!(string)], None);

impl Function for RedactKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let pattern = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected pattern string"))?;

        let regex = regex::Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {}", e)))?;

        Ok(redact_keys_recursive(&args[0], &regex))
    }
}

fn redact_keys_recursive(value: &Value, pattern: &regex::Regex) -> Value {
    match value {
        Value::Object(obj) => {
            let redacted: Map<String, Value> = obj
                .iter()
                .map(|(k, v)| {
                    if pattern.is_match(k) {
                        (k.clone(), Value::String("[REDACTED]".to_string()))
                    } else {
                        (k.clone(), redact_keys_recursive(v, pattern))
                    }
                })
                .collect();
            Value::Object(redacted)
        }
        Value::Array(arr) => {
            let redacted: Vec<Value> = arr
                .iter()
                .map(|v| redact_keys_recursive(v, pattern))
                .collect();
            Value::Array(redacted)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// mask(string, show_last?) -> string (mask all but last N chars)
// =============================================================================

defn!(MaskFn, vec![arg!(string)], Some(arg!(number)));

impl Function for MaskFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let show_last = if args.len() > 1 {
            args[1].as_f64().unwrap_or(4.0) as usize
        } else {
            4
        };

        // Count and slice by characters so "show the last N characters" is
        // correct for multibyte input and never slices mid-code-point.
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let masked = if len <= show_last {
            "*".repeat(len)
        } else {
            let mask_count = len - show_last;
            let visible: String = chars[mask_count..].iter().collect();
            format!("{}{}", "*".repeat(mask_count), visible)
        };

        Ok(Value::String(masked))
    }
}

// =============================================================================
// pluck_deep(any, key) -> array
// =============================================================================

defn!(PluckDeepFn, vec![arg!(any), arg!(string)], None);

impl Function for PluckDeepFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let key = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected key string"))?;

        let mut results: Vec<Value> = Vec::new();
        pluck_deep_recursive(&args[0], key, &mut results);
        Ok(Value::Array(results))
    }
}

fn pluck_deep_recursive(value: &Value, key: &str, results: &mut Vec<Value>) {
    match value {
        Value::Object(obj) => {
            if let Some(v) = obj.get(key) {
                results.push(v.clone());
            }
            for (_, v) in obj.iter() {
                pluck_deep_recursive(v, key, results);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                pluck_deep_recursive(v, key, results);
            }
        }
        _ => {}
    }
}

// =============================================================================
// paths_to(any, key) -> array
// =============================================================================

defn!(PathsToFn, vec![arg!(any), arg!(string)], None);

impl Function for PathsToFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let key = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected key string"))?;

        let mut paths: Vec<String> = Vec::new();
        paths_to_recursive(&args[0], key, String::new(), &mut paths);

        let result: Vec<Value> = paths.into_iter().map(Value::String).collect();
        Ok(Value::Array(result))
    }
}

fn paths_to_recursive(value: &Value, key: &str, current_path: String, paths: &mut Vec<String>) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj.iter() {
                let new_path = if current_path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", current_path, k)
                };
                if k == key {
                    paths.push(new_path.clone());
                }
                paths_to_recursive(v, key, new_path, paths);
            }
        }
        Value::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                let new_path = if current_path.is_empty() {
                    idx.to_string()
                } else {
                    format!("{}.{}", current_path, idx)
                };
                paths_to_recursive(v, key, new_path, paths);
            }
        }
        _ => {}
    }
}

// =============================================================================
// snake_keys(any) -> any
// =============================================================================

defn!(SnakeKeysFn, vec![arg!(any)], None);

impl Function for SnakeKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| s.to_snake_case()))
    }
}

// =============================================================================
// camel_keys(any) -> any
// =============================================================================

defn!(CamelKeysFn, vec![arg!(any)], None);

impl Function for CamelKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| {
            s.to_lower_camel_case()
        }))
    }
}

// =============================================================================
// kebab_keys(any) -> any
// =============================================================================

defn!(KebabKeysFn, vec![arg!(any)], None);

impl Function for KebabKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| s.to_kebab_case()))
    }
}

// =============================================================================
// pascal_keys(any) -> any
// =============================================================================

defn!(PascalKeysFn, vec![arg!(any)], None);

impl Function for PascalKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| {
            s.to_upper_camel_case()
        }))
    }
}

// =============================================================================
// shouty_snake_keys(any) -> any
// =============================================================================

defn!(ShoutySnakeKeysFn, vec![arg!(any)], None);

impl Function for ShoutySnakeKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| {
            s.to_shouty_snake_case()
        }))
    }
}

// =============================================================================
// shouty_kebab_keys(any) -> any
// =============================================================================

defn!(ShoutyKebabKeysFn, vec![arg!(any)], None);

impl Function for ShoutyKebabKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| {
            s.to_shouty_kebab_case()
        }))
    }
}

// =============================================================================
// train_keys(any) -> any
// =============================================================================

defn!(TrainKeysFn, vec![arg!(any)], None);

impl Function for TrainKeysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(transform_keys_recursive(&args[0], |s| s.to_train_case()))
    }
}

fn transform_keys_recursive<F>(value: &Value, transform: F) -> Value
where
    F: Fn(&str) -> String + Copy,
{
    match value {
        Value::Object(obj) => {
            let transformed: Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (transform(k), transform_keys_recursive(v, transform)))
                .collect();
            Value::Object(transformed)
        }
        Value::Array(arr) => {
            let transformed: Vec<Value> = arr
                .iter()
                .map(|v| transform_keys_recursive(v, transform))
                .collect();
            Value::Array(transformed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// structural_diff(obj1, obj2) -> object
// =============================================================================

defn!(StructuralDiffFn, vec![arg!(any), arg!(any)], None);

impl Function for StructuralDiffFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut added: Vec<String> = Vec::new();
        let mut removed: Vec<String> = Vec::new();
        let mut type_changed: Vec<Map<String, Value>> = Vec::new();
        let mut unchanged: Vec<String> = Vec::new();

        compare_structure(
            &args[0],
            &args[1],
            String::new(),
            &mut added,
            &mut removed,
            &mut type_changed,
            &mut unchanged,
        );

        let mut result = Map::new();
        result.insert(
            "added".to_string(),
            Value::Array(added.into_iter().map(Value::String).collect()),
        );
        result.insert(
            "removed".to_string(),
            Value::Array(removed.into_iter().map(Value::String).collect()),
        );
        result.insert(
            "type_changed".to_string(),
            Value::Array(type_changed.into_iter().map(Value::Object).collect()),
        );
        result.insert(
            "unchanged".to_string(),
            Value::Array(unchanged.into_iter().map(Value::String).collect()),
        );

        Ok(Value::Object(result))
    }
}

fn get_structural_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn compare_structure(
    a: &Value,
    b: &Value,
    path: String,
    added: &mut Vec<String>,
    removed: &mut Vec<String>,
    type_changed: &mut Vec<Map<String, Value>>,
    unchanged: &mut Vec<String>,
) {
    let type_a = get_structural_type(a);
    let type_b = get_structural_type(b);

    if type_a != type_b {
        let mut change = Map::new();
        change.insert(
            "path".to_string(),
            Value::String(if path.is_empty() {
                "$".to_string()
            } else {
                path
            }),
        );
        change.insert("from".to_string(), Value::String(type_a.to_string()));
        change.insert("to".to_string(), Value::String(type_b.to_string()));
        type_changed.push(change);
        return;
    }

    match (a, b) {
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            for key in obj_a.keys() {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                if let Some(val_b) = obj_b.get(key) {
                    compare_structure(
                        obj_a.get(key).unwrap(),
                        val_b,
                        new_path,
                        added,
                        removed,
                        type_changed,
                        unchanged,
                    );
                } else {
                    removed.push(new_path);
                }
            }
            for key in obj_b.keys() {
                if !obj_a.contains_key(key) {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    added.push(new_path);
                }
            }
        }
        _ => {
            if !path.is_empty() {
                unchanged.push(path);
            }
        }
    }
}

// =============================================================================
// has_same_shape(obj1, obj2) -> boolean
// =============================================================================

defn!(HasSameShapeFn, vec![arg!(any), arg!(any)], None);

impl Function for HasSameShapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let same = check_same_shape(&args[0], &args[1]);
        Ok(Value::Bool(same))
    }
}

fn check_same_shape(a: &Value, b: &Value) -> bool {
    let type_a = get_structural_type(a);
    let type_b = get_structural_type(b);

    if type_a != type_b {
        return false;
    }

    match (a, b) {
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            if obj_a.keys().collect::<HashSet<_>>() != obj_b.keys().collect::<HashSet<_>>() {
                return false;
            }
            for key in obj_a.keys() {
                if !check_same_shape(obj_a.get(key).unwrap(), obj_b.get(key).unwrap()) {
                    return false;
                }
            }
            true
        }
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            if arr_a.is_empty() && arr_b.is_empty() {
                return true;
            }
            if arr_a.is_empty() || arr_b.is_empty() {
                return true;
            }
            check_same_shape(&arr_a[0], &arr_b[0])
        }
        _ => true,
    }
}

// =============================================================================
// infer_schema(any) -> object
// =============================================================================

defn!(InferSchemaFn, vec![arg!(any)], None);

impl Function for InferSchemaFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        Ok(infer_schema_recursive(&args[0]))
    }
}

fn infer_schema_recursive(value: &Value) -> Value {
    match value {
        Value::Null => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("null".to_string()));
            Value::Object(schema)
        }
        Value::Bool(_) => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("boolean".to_string()));
            Value::Object(schema)
        }
        Value::Number(_) => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("number".to_string()));
            Value::Object(schema)
        }
        Value::String(_) => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("string".to_string()));
            Value::Object(schema)
        }
        Value::Array(arr) => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("array".to_string()));
            if !arr.is_empty() {
                let items_schema = infer_schema_recursive(&arr[0]);
                schema.insert("items".to_string(), items_schema);
            }
            Value::Object(schema)
        }
        Value::Object(obj) => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("object".to_string()));

            let mut properties = Map::new();
            for (key, val) in obj.iter() {
                let prop_schema = infer_schema_recursive(val);
                properties.insert(key.clone(), prop_schema);
            }
            schema.insert("properties".to_string(), Value::Object(properties));
            Value::Object(schema)
        }
    }
}

// =============================================================================
// chunk_by_size(array, max_bytes) -> array of arrays
// =============================================================================

defn!(ChunkBySizeFn, vec![arg!(array), arg!(number)], None);

impl Function for ChunkBySizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array"))?;

        let max_bytes = args[1].as_f64().unwrap_or(4000.0) as usize;

        let mut chunks: Vec<Value> = Vec::new();
        let mut current_chunk: Vec<Value> = Vec::new();
        let mut current_size: usize = 2;

        for item in arr {
            let item_size = serde_json::to_string(item).map(|s| s.len()).unwrap_or(0);

            if current_size + item_size + 1 > max_bytes && !current_chunk.is_empty() {
                chunks.push(Value::Array(current_chunk));
                current_chunk = Vec::new();
                current_size = 2;
            }

            current_chunk.push(item.clone());
            current_size += item_size + 1;
        }

        if !current_chunk.is_empty() {
            chunks.push(Value::Array(current_chunk));
        }

        Ok(Value::Array(chunks))
    }
}

// =============================================================================
// paginate(array, page, per_page) -> object with pagination metadata
// =============================================================================

defn!(
    PaginateFn,
    vec![arg!(array), arg!(number), arg!(number)],
    None
);

impl Function for PaginateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array"))?;

        let page = args[1].as_f64().unwrap_or(1.0).max(1.0) as usize;
        let per_page = args[2].as_f64().unwrap_or(10.0).max(1.0) as usize;

        let total = arr.len();
        let total_pages = total.div_ceil(per_page);
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(total);

        let data: Vec<Value> = if start < total {
            arr[start..end].to_vec()
        } else {
            vec![]
        };

        let mut result = Map::new();
        result.insert("data".to_string(), Value::Array(data));
        result.insert("page".to_string(), Value::Number(Number::from(page as i64)));
        result.insert(
            "per_page".to_string(),
            Value::Number(Number::from(per_page as i64)),
        );
        result.insert(
            "total".to_string(),
            Value::Number(Number::from(total as i64)),
        );
        result.insert(
            "total_pages".to_string(),
            Value::Number(Number::from(total_pages as i64)),
        );
        result.insert("has_next".to_string(), Value::Bool(page < total_pages));
        result.insert("has_prev".to_string(), Value::Bool(page > 1));

        Ok(Value::Object(result))
    }
}

// =============================================================================
// estimate_size(any) -> number
// =============================================================================

defn!(EstimateSizeFn, vec![arg!(any)], None);

impl Function for EstimateSizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let size = serde_json::to_string(&args[0])
            .map(|s| s.len())
            .unwrap_or(0);
        Ok(Value::Number(Number::from(size as i64)))
    }
}

// =============================================================================
// truncate_to_size(any, max_bytes) -> any
// =============================================================================

defn!(TruncateToSizeFn, vec![arg!(any), arg!(number)], None);

impl Function for TruncateToSizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let max_bytes = args[1].as_f64().unwrap_or(1000.0) as usize;
        let current_size = serde_json::to_string(&args[0])
            .map(|s| s.len())
            .unwrap_or(0);

        if current_size <= max_bytes {
            return Ok(args[0].clone());
        }

        if let Some(arr) = args[0].as_array() {
            let mut result: Vec<Value> = Vec::new();
            let mut size = 2;

            for item in arr {
                let item_size = serde_json::to_string(item).map(|s| s.len()).unwrap_or(0);
                if size + item_size + 1 > max_bytes {
                    break;
                }
                result.push(item.clone());
                size += item_size + 1;
            }
            return Ok(Value::Array(result));
        }

        if let Some(s) = args[0].as_str() {
            let target_len = max_bytes.saturating_sub(2);
            let truncated: String = s.chars().take(target_len).collect();
            return Ok(Value::String(truncated));
        }

        Ok(args[0].clone())
    }
}

// =============================================================================
// template(object, template_string) -> string
// =============================================================================

defn!(TemplateFn, vec![arg!(any), arg!(string)], None);

impl Function for TemplateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        if args.len() >= 2 && args[1].is_null() {
            return Err(custom_error(
                ctx,
                "template: second argument is null. Template strings must be JMESPath \
                 literals using backticks, e.g., template(@, `\"Hello {{name}}\"`)",
            ));
        }

        self.signature.validate(args, ctx)?;

        let template = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected template string"))?;

        let result =
            expand_template(&args[0], template, false).map_err(|e| custom_error(ctx, &e))?;
        Ok(Value::String(result))
    }
}

// =============================================================================
// template_strict(object, template_string) -> string
// =============================================================================

defn!(TemplateStrictFn, vec![arg!(any), arg!(string)], None);

impl Function for TemplateStrictFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        if args.len() >= 2 && args[1].is_null() {
            return Err(custom_error(
                ctx,
                "template_strict: second argument is null. Template strings must be JMESPath \
                 literals using backticks, e.g., template_strict(@, `\"Hello {{name}}\"`)",
            ));
        }

        self.signature.validate(args, ctx)?;

        let template = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected template string"))?;

        let result =
            expand_template(&args[0], template, true).map_err(|e| custom_error(ctx, &e))?;
        Ok(Value::String(result))
    }
}

fn expand_template(data: &Value, template: &str, strict: bool) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();

            let mut var_name = String::new();
            let mut fallback: Option<String> = None;

            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        break;
                    }
                } else if next == '|' {
                    chars.next();
                    let mut fb = String::new();
                    while let Some(&fc) = chars.peek() {
                        if fc == '}' {
                            break;
                        }
                        fb.push(chars.next().unwrap());
                    }
                    fallback = Some(fb);
                } else {
                    var_name.push(chars.next().unwrap());
                }
            }

            let value = get_template_value(data, &var_name);

            match value {
                Some(v) => result.push_str(&value_to_string(&v)),
                None => {
                    if strict {
                        return Err(format!("missing variable '{}'", var_name));
                    }
                    if let Some(fb) = fallback {
                        result.push_str(&fb);
                    }
                }
            }
        } else if c == '\\' && chars.peek() == Some(&'{') {
            result.push(chars.next().unwrap());
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn get_template_value(data: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.trim().split('.').collect();
    let mut current = data.clone();

    for part in parts {
        if let Ok(idx) = part.parse::<usize>() {
            if let Some(arr) = current.as_array()
                && idx < arr.len()
            {
                current = arr[idx].clone();
                continue;
            }
            return None;
        }

        if let Some(obj) = current.as_object() {
            if let Some(val) = obj.get(part) {
                current = val.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    if current.is_null() {
        None
    } else {
        Some(current)
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        _ => serde_json::to_string(value).unwrap_or_default(),
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
    fn test_items() {
        let runtime = setup_runtime();
        let expr = runtime.compile("items(@)").unwrap();
        let data = json!({"a": 1, "b": 2});
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // JEP-013: Each item should be a [key, value] pair
        let first = arr[0].as_array().unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].as_str().unwrap(), "a");
        assert_eq!(first[1].as_f64().unwrap() as i64, 1);
    }

    #[test]
    fn test_items_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("items(@)").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_from_items() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = json!([["a", 1], ["b", 2]]);
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_from_items_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = json!([]);
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_from_items_duplicate_keys() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = json!([["x", 1], ["x", 2]]);
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        // Last value wins
        assert_eq!(obj.get("x").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_items_from_items_roundtrip() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(items(@))").unwrap();
        let data = json!({"a": 1, "b": "hello", "c": true});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_str().unwrap(), "hello");
        assert!(obj.get("c").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_with_entries_identity() {
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[0], @[1]]')").unwrap();
        let data = json!({"a": 1, "b": 2});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_with_entries_transform_keys() {
        // Test that we can transform keys by prepending a prefix
        // Using join to concatenate strings (built-in)
        let runtime = setup_runtime();
        let expr = runtime
            .compile(r#"with_entries(@, '[join(`""`, [`"prefix_"`, @[0]]), @[1]]')"#)
            .unwrap();
        let data = json!({"a": 1, "b": 2});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("prefix_a"));
        assert!(obj.contains_key("prefix_b"));
    }

    #[test]
    fn test_with_entries_swap_key_value() {
        // Test swapping keys and values (for string values)
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[1], @[0]]')").unwrap();
        let data = json!({"a": "x", "b": "y"});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("x").unwrap().as_str().unwrap(), "a");
        assert_eq!(obj.get("y").unwrap().as_str().unwrap(), "b");
    }

    #[test]
    fn test_with_entries_filter_null() {
        // Test that returning null from the expression skips entries
        // This tests the filtering behavior of with_entries
        let runtime = setup_runtime();
        // Return null for all entries - result should be empty object
        let expr = runtime.compile(r#"with_entries(@, '`null`')"#).unwrap();
        let data = json!({"a": 1, "b": 2});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_with_entries_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[0], @[1]]')").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_pick() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pick(@, `[\"a\"]`)").unwrap();
        let data = json!({"a": 1, "b": 2});
        let result = expr.search(&data).unwrap();
        let result_obj = result.as_object().unwrap();
        assert_eq!(result_obj.len(), 1);
        assert!(result_obj.contains_key("a"));
    }

    #[test]
    fn test_deep_equals_objects() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}, "c": {"b": 1}});
        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_deep_equals_objects_different() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}, "c": {"b": 2}});
        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_deep_equals_arrays() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, [2, 3]], "b": [1, [2, 3]]});
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_deep_equals_arrays_order_matters() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, 2], "b": [2, 1]});
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_deep_equals_primitives() {
        let runtime = setup_runtime();
        let data = json!({"a": "hello", "b": "hello", "c": "world"});

        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());

        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_deep_diff_added() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 1, "y": 2}});
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let added = diff.get("added").unwrap().as_object().unwrap();
        assert!(added.contains_key("y"));
        assert!(diff.get("removed").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_deep_diff_removed() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1, "y": 2}, "b": {"x": 1}});
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let removed = diff.get("removed").unwrap().as_object().unwrap();
        assert!(removed.contains_key("y"));
        assert!(diff.get("added").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_deep_diff_changed() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 2}});
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let changed = diff.get("changed").unwrap().as_object().unwrap();
        assert!(changed.contains_key("x"));
        let x_change = changed.get("x").unwrap().as_object().unwrap();
        assert!(x_change.contains_key("from"));
        assert!(x_change.contains_key("to"));
    }

    #[test]
    fn test_deep_diff_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": {"y": 1}}, "b": {"x": {"y": 2}}});
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        // The change should be nested under x
        let changed = diff.get("changed").unwrap().as_object().unwrap();
        assert!(changed.contains_key("x"));
    }

    #[test]
    fn test_deep_diff_no_changes() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 1}});
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        assert!(diff.get("added").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("removed").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_get_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": {"c": 1}}});
        let expr = runtime.compile("get(@, 'a.b.c')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_get_with_default() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime.compile("get(@, 'b.c', 'default')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "default");
    }

    #[test]
    fn test_get_array_index() {
        let runtime = setup_runtime();
        let data = json!({"a": [{"b": 1}, {"b": 2}]});
        let expr = runtime.compile("get(@, 'a[0].b')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_get_missing_returns_null() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime.compile("get(@, 'x.y.z')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_has_exists() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}});
        let expr = runtime.compile("has(@, 'a.b')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_has_not_exists() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime.compile("has(@, 'a.b.c')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_has_array_index() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, 2, 3]});
        let expr = runtime.compile("has(@, 'a[1]')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_has_array_index_out_of_bounds() {
        let runtime = setup_runtime();
        let data = json!({"a": [1, 2]});
        let expr = runtime.compile("has(@, 'a[5]')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_defaults_shallow() {
        let runtime = setup_runtime();
        let data = json!({"obj": {"a": 1}, "defs": {"a": 2, "b": 3}});
        let expr = runtime.compile("defaults(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 1.0); // original kept
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_defaults_empty_object() {
        let runtime = setup_runtime();
        let data = json!({"obj": {}, "defs": {"a": 1, "b": 2}});
        let expr = runtime.compile("defaults(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_defaults_deep_nested() {
        let runtime = setup_runtime();
        let data = json!({"obj": {"a": {"b": 1}}, "defs": {"a": {"b": 2, "c": 3}}});
        let expr = runtime.compile("defaults_deep(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("b").unwrap().as_f64().unwrap(), 1.0); // original kept
        assert_eq!(a.get("c").unwrap().as_f64().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_defaults_deep_new_nested() {
        let runtime = setup_runtime();
        let data = json!({"obj": {"x": 1}, "defs": {"x": 2, "y": {"z": 3}}});
        let expr = runtime.compile("defaults_deep(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("x").unwrap().as_f64().unwrap(), 1.0); // original kept
        let y = obj.get("y").unwrap().as_object().unwrap();
        assert_eq!(y.get("z").unwrap().as_f64().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_set_path_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": 2});
        let expr = runtime.compile("set_path(@, '/c', `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap(), 2.0);
        assert_eq!(obj.get("c").unwrap().as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_set_path_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}});
        let expr = runtime.compile("set_path(@, '/a/c', `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("b").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(a.get("c").unwrap().as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_set_path_create_nested() {
        let runtime = setup_runtime();
        let data = json!({});
        let expr = runtime
            .compile("set_path(@, '/a/b/c', `\"deep\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        let b = a.get("b").unwrap().as_object().unwrap();
        assert_eq!(b.get("c").unwrap().as_str().unwrap(), "deep");
    }

    #[test]
    fn test_set_path_array_index() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2, 3]});
        let expr = runtime.compile("set_path(@, '/items/1', `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[0].as_f64().unwrap(), 1.0);
        assert_eq!(items[1].as_f64().unwrap(), 99.0);
        assert_eq!(items[2].as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_delete_path_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": 2, "c": 3});
        let expr = runtime.compile("delete_path(@, '/b')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        assert!(!obj.contains_key("b"));
    }

    #[test]
    fn test_delete_path_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1, "c": 2}});
        let expr = runtime.compile("delete_path(@, '/a/b')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.len(), 1);
        assert!(a.contains_key("c"));
        assert!(!a.contains_key("b"));
    }

    #[test]
    fn test_delete_path_array() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2, 3]});
        let expr = runtime.compile("delete_path(@, '/items/1')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_f64().unwrap(), 1.0);
        assert_eq!(items[1].as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_paths_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}, "c": 2});
        let expr = runtime.compile("paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let paths = result.as_array().unwrap();
        assert!(paths.len() >= 3); // /a, /a/b, /c
    }

    #[test]
    fn test_paths_with_array() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2]});
        let expr = runtime.compile("paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let paths: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect();
        assert!(paths.contains(&"/items".to_string()));
        assert!(paths.contains(&"/items/0".to_string()));
        assert!(paths.contains(&"/items/1".to_string()));
    }

    #[test]
    fn test_leaves_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": {"c": 2}, "d": [3, 4]});
        let expr = runtime.compile("leaves(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 4); // 1, 2, 3, 4
    }

    #[test]
    fn test_leaves_strings() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "tags": ["a", "b"]});
        let expr = runtime.compile("leaves(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 3); // "alice", "a", "b"
    }

    #[test]
    fn test_leaves_with_paths_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": {"c": 2}});
        let expr = runtime.compile("leaves_with_paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 2);
        // Each leaf should have path and value
        let first = leaves[0].as_object().unwrap();
        assert!(first.contains_key("path"));
        assert!(first.contains_key("value"));
    }

    #[test]
    fn test_set_path_immutable() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime.compile("set_path(@, '/b', `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        // Original should be unchanged (immutable semantics)
        let original = data.as_object().unwrap();
        assert!(!original.contains_key("b"));
        // Result should have the new key
        let new_obj = result.as_object().unwrap();
        assert!(new_obj.contains_key("b"));
    }

    // =========================================================================
    // Dot notation path tests (for set_path, delete_path, get_path, has_path)
    // =========================================================================

    #[test]
    fn test_set_path_dot_notation() {
        let runtime = setup_runtime();
        let data = json!({"a": {"c": 1}});
        let expr = runtime.compile("set_path(@, `\"a.b\"`, `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(nested.get("b").unwrap().as_f64().unwrap() as i64, 99);
        assert_eq!(nested.get("c").unwrap().as_f64().unwrap() as i64, 1);
    }

    #[test]
    fn test_set_path_dot_notation_deep() {
        let runtime = setup_runtime();
        let data = json!({});
        let expr = runtime
            .compile("set_path(@, `\"a.b.c\"`, `\"deep\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        let b = a.get("b").unwrap().as_object().unwrap();
        assert_eq!(b.get("c").unwrap().as_str().unwrap(), "deep");
    }

    #[test]
    fn test_set_path_dot_notation_array_index() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2, 3]});
        let expr = runtime.compile("set_path(@, `\"items.1\"`, `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[1].as_f64().unwrap() as i64, 99);
    }

    #[test]
    fn test_delete_path_dot_notation() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1, "c": 2}});
        let expr = runtime.compile("delete_path(@, `\"a.b\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert!(!nested.contains_key("b"));
        assert!(nested.contains_key("c"));
    }

    #[test]
    fn test_delete_path_dot_notation_array() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2, 3]});
        let expr = runtime.compile("delete_path(@, `\"items.1\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_f64().unwrap() as i64, 1);
        assert_eq!(items[1].as_f64().unwrap() as i64, 3);
    }

    #[test]
    fn test_get_path_alias() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": {"c": 42}}});
        let expr = runtime.compile("get_path(@, `\"a.b.c\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 42);
    }

    #[test]
    fn test_get_path_with_default() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime
            .compile("get_path(@, `\"a.b.c\"`, `\"default\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "default");
    }

    #[test]
    fn test_get_path_array_index() {
        let runtime = setup_runtime();
        let data = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
        let expr = runtime.compile("get_path(@, `\"users.0.name\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "alice");
    }

    #[test]
    fn test_get_path_array_index_out_of_bounds() {
        let runtime = setup_runtime();
        let data = json!({"users": [{"name": "alice"}]});
        let expr = runtime
            .compile("get_path(@, `\"users.5.name\"`, `\"unknown\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "unknown");
    }

    #[test]
    fn test_has_path_alias() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}});
        let expr = runtime.compile("has_path(@, `\"a.b\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_has_path_missing() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": 1}});
        let expr = runtime.compile("has_path(@, `\"a.c\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_has_path_array_index() {
        let runtime = setup_runtime();
        let data = json!({"items": [1, 2, 3]});
        let expr = runtime.compile("has_path(@, `\"items.1\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    // =========================================================================
    // remove_nulls tests
    // =========================================================================

    #[test]
    fn test_remove_nulls_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": null, "c": 2});
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        assert!(!obj.contains_key("b"));
    }

    #[test]
    fn test_remove_nulls_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": {"c": null, "d": 2}});
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("b").unwrap().as_object().unwrap();
        assert_eq!(nested.len(), 1);
        assert!(nested.contains_key("d"));
        assert!(!nested.contains_key("c"));
    }

    #[test]
    fn test_remove_nulls_array() {
        let runtime = setup_runtime();
        let data = json!([1, null, 2, null, 3]);
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    // =========================================================================
    // remove_empty tests
    // =========================================================================

    #[test]
    fn test_remove_empty_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": "", "b": [], "c": {}, "d": null, "e": "hello"});
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("e"));
    }

    #[test]
    fn test_remove_empty_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": "", "c": 1}, "d": []});
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(nested.len(), 1);
        assert!(nested.contains_key("c"));
    }

    #[test]
    fn test_remove_empty_array() {
        let runtime = setup_runtime();
        let data = json!(["", "hello", [], null, "world"]);
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // =========================================================================
    // remove_empty_strings tests
    // =========================================================================

    #[test]
    fn test_remove_empty_strings_basic() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "bio": "", "age": 30});
        let expr = runtime.compile("remove_empty_strings(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("age"));
        assert!(!obj.contains_key("bio"));
    }

    #[test]
    fn test_remove_empty_strings_array() {
        let runtime = setup_runtime();
        let data = json!(["hello", "", "world", ""]);
        let expr = runtime.compile("remove_empty_strings(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // =========================================================================
    // compact_deep tests
    // =========================================================================

    #[test]
    fn test_compact_deep_basic() {
        let runtime = setup_runtime();
        let data = json!([[1, null], [null, 2]]);
        let expr = runtime.compile("compact_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_array().unwrap();
        assert_eq!(first.len(), 1);
        let second = arr[1].as_array().unwrap();
        assert_eq!(second.len(), 1);
    }

    #[test]
    fn test_compact_deep_nested() {
        let runtime = setup_runtime();
        let data = json!([[1, null], [null, [2, null, 3]]]);
        let expr = runtime.compile("compact_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let second = arr[1].as_array().unwrap();
        let inner = second[0].as_array().unwrap();
        assert_eq!(inner.len(), 2); // [2, 3]
    }

    // =========================================================================
    // completeness tests
    // =========================================================================

    #[test]
    fn test_completeness_all_filled() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": "hello", "c": true});
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_f64().unwrap();
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_completeness_with_nulls() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": null, "c": null});
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_f64().unwrap();
        // 1 out of 3 fields is non-null = 33.33%
        assert!((score - 33.33).abs() < 1.0);
    }

    #[test]
    fn test_completeness_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": {"c": null, "d": 2}, "e": null});
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_f64().unwrap();
        // 5 total fields: a(1), b(obj), b.c(null), b.d(2), e(null)
        // 3 non-null: a, b, b.d
        // 3/5 = 60%
        assert!((score - 60.0).abs() < 1.0);
    }

    // =========================================================================
    // type_consistency tests
    // =========================================================================

    #[test]
    fn test_type_consistency_consistent() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("consistent").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_type_consistency_inconsistent() {
        let runtime = setup_runtime();
        let data = json!([1, "two", 3]);
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(!obj.get("consistent").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_type_consistency_object_array() {
        let runtime = setup_runtime();
        let data = json!([{"name": "alice", "age": 30}, {"name": "bob", "age": "unknown"}]);
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(!obj.get("consistent").unwrap().as_bool().unwrap());
        let inconsistencies = obj.get("inconsistencies").unwrap().as_array().unwrap();
        assert_eq!(inconsistencies.len(), 1);
    }

    // =========================================================================
    // data_quality_score tests
    // =========================================================================

    #[test]
    fn test_data_quality_score_perfect() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": "hello"});
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let score = obj.get("score").unwrap().as_f64().unwrap();
        assert_eq!(score, 100.0);
        assert_eq!(obj.get("null_count").unwrap().as_f64().unwrap() as i64, 0);
    }

    #[test]
    fn test_data_quality_score_with_issues() {
        let runtime = setup_runtime();
        let data = json!({"a": 1, "b": null, "c": ""});
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("null_count").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(
            obj.get("empty_string_count").unwrap().as_f64().unwrap() as i64,
            1
        );
        let issues = obj.get("issues").unwrap().as_array().unwrap();
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_data_quality_score_type_mismatch() {
        let runtime = setup_runtime();
        let data = json!({"users": [{"age": 30}, {"age": "thirty"}]});
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("type_inconsistencies").unwrap().as_f64().unwrap() as i64,
            1
        );
    }

    // =========================================================================
    // redact tests
    // =========================================================================

    #[test]
    fn test_redact_basic() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "password": "secret123", "ssn": "123-45-6789"});
        let expr = runtime
            .compile(r#"redact(@, `["password", "ssn"]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "alice");
        assert_eq!(obj.get("password").unwrap().as_str().unwrap(), "[REDACTED]");
        assert_eq!(obj.get("ssn").unwrap().as_str().unwrap(), "[REDACTED]");
    }

    #[test]
    fn test_redact_nested() {
        let runtime = setup_runtime();
        let data = json!({"user": {"name": "bob", "password": "secret"}});
        let expr = runtime.compile(r#"redact(@, `["password"]`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let user = obj.get("user").unwrap().as_object().unwrap();
        assert_eq!(user.get("name").unwrap().as_str().unwrap(), "bob");
        assert_eq!(
            user.get("password").unwrap().as_str().unwrap(),
            "[REDACTED]"
        );
    }

    #[test]
    fn test_redact_array_of_objects() {
        let runtime = setup_runtime();
        let data = json!([
            {"name": "alice", "token": "abc"},
            {"name": "bob", "token": "xyz"}
        ]);
        let expr = runtime.compile(r#"redact(@, `["token"]`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let first = arr[0].as_object().unwrap();
        assert_eq!(first.get("token").unwrap().as_str().unwrap(), "[REDACTED]");
    }

    // =========================================================================
    // mask tests
    // =========================================================================

    #[test]
    fn test_mask_default() {
        let runtime = setup_runtime();
        let data = json!("4111111111111111");
        let expr = runtime.compile("mask(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "************1111");
    }

    #[test]
    fn test_mask_custom_length() {
        let runtime = setup_runtime();
        let data = json!("555-123-4567");
        let expr = runtime.compile("mask(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "*********567");
    }

    #[test]
    fn test_mask_short_string() {
        let runtime = setup_runtime();
        let data = json!("abc");
        let expr = runtime.compile("mask(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // If string is shorter than show_last, mask everything
        assert_eq!(result.as_str().unwrap(), "***");
    }

    // =========================================================================
    // redact_keys tests
    // =========================================================================

    #[test]
    fn test_redact_keys_basic() {
        let runtime = setup_runtime();
        let data = json!({"password": "secret", "api_key": "abc123", "name": "test"});
        let expr = runtime
            .compile(r#"redact_keys(@, `"password|api_key"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("password").unwrap().as_str().unwrap(), "[REDACTED]");
        assert_eq!(obj.get("api_key").unwrap().as_str().unwrap(), "[REDACTED]");
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "test");
    }

    #[test]
    fn test_redact_keys_pattern() {
        let runtime = setup_runtime();
        let data = json!({"secret_key": "a", "secret_token": "b", "name": "test"});
        let expr = runtime.compile(r#"redact_keys(@, `"secret.*"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("secret_key").unwrap().as_str().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(
            obj.get("secret_token").unwrap().as_str().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "test");
    }

    // =========================================================================
    // pluck_deep tests
    // =========================================================================

    #[test]
    fn test_pluck_deep_basic() {
        let runtime = setup_runtime();
        let data = json!({"users": [{"id": 1}, {"id": 2}], "meta": {"id": 99}});
        let expr = runtime.compile(r#"pluck_deep(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_pluck_deep_nested() {
        let runtime = setup_runtime();
        let data = json!({"a": {"b": {"c": 1}}, "d": {"c": 2}});
        let expr = runtime.compile(r#"pluck_deep(@, `"c"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_pluck_deep_not_found() {
        let runtime = setup_runtime();
        let data = json!({"a": 1});
        let expr = runtime.compile(r#"pluck_deep(@, `"x"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // paths_to tests
    // =========================================================================

    #[test]
    fn test_paths_to_basic() {
        let runtime = setup_runtime();
        let data = json!({"a": {"id": 1}, "b": {"id": 2}});
        let expr = runtime.compile(r#"paths_to(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let paths: Vec<String> = arr
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect();
        assert!(paths.contains(&"a.id".to_string()));
        assert!(paths.contains(&"b.id".to_string()));
    }

    #[test]
    fn test_paths_to_array() {
        let runtime = setup_runtime();
        let data = json!({"users": [{"id": 1}]});
        let expr = runtime.compile(r#"paths_to(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str().unwrap(), "users.0.id");
    }

    // =========================================================================
    // snake_keys tests
    // =========================================================================

    #[test]
    fn test_snake_keys_camel() {
        let runtime = setup_runtime();
        let data = json!({"userName": "alice"});
        let expr = runtime.compile("snake_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user_name"));
        assert_eq!(obj.get("user_name").unwrap().as_str().unwrap(), "alice");
    }

    #[test]
    fn test_snake_keys_nested() {
        let runtime = setup_runtime();
        let data = json!({"userInfo": {"firstName": "bob"}});
        let expr = runtime.compile("snake_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user_info"));
        let nested = obj.get("user_info").unwrap().as_object().unwrap();
        assert!(nested.contains_key("first_name"));
    }

    // =========================================================================
    // camel_keys tests
    // =========================================================================

    #[test]
    fn test_camel_keys_snake() {
        let runtime = setup_runtime();
        let data = json!({"user_name": "alice"});
        let expr = runtime.compile("camel_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("userName"));
        assert_eq!(obj.get("userName").unwrap().as_str().unwrap(), "alice");
    }

    #[test]
    fn test_camel_keys_nested() {
        let runtime = setup_runtime();
        let data = json!({"user_info": {"first_name": "bob"}});
        let expr = runtime.compile("camel_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("userInfo"));
        let nested = obj.get("userInfo").unwrap().as_object().unwrap();
        assert!(nested.contains_key("firstName"));
    }

    // =========================================================================
    // kebab_keys tests
    // =========================================================================

    #[test]
    fn test_kebab_keys_camel() {
        let runtime = setup_runtime();
        let data = json!({"userName": "alice"});
        let expr = runtime.compile("kebab_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user-name"));
        assert_eq!(obj.get("user-name").unwrap().as_str().unwrap(), "alice");
    }

    #[test]
    fn test_kebab_keys_snake() {
        let runtime = setup_runtime();
        let data = json!({"user_name": "bob"});
        let expr = runtime.compile("kebab_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user-name"));
        assert_eq!(obj.get("user-name").unwrap().as_str().unwrap(), "bob");
    }

    // =========================================================================
    // structural_diff tests
    // =========================================================================

    #[test]
    fn test_structural_diff_added() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 1, "y": 2}});
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let added = obj.get("added").unwrap().as_array().unwrap();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].as_str().unwrap(), "y");
    }

    #[test]
    fn test_structural_diff_removed() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1, "y": 2}, "b": {"x": 1}});
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let removed = obj.get("removed").unwrap().as_array().unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].as_str().unwrap(), "y");
    }

    #[test]
    fn test_structural_diff_type_changed() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": "string"}});
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let type_changed = obj.get("type_changed").unwrap().as_array().unwrap();
        assert_eq!(type_changed.len(), 1);
    }

    #[test]
    fn test_has_same_shape_true() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 2}});
        let expr = runtime.compile("has_same_shape(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_has_same_shape_false() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"y": 2}});
        let expr = runtime.compile("has_same_shape(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    // =========================================================================
    // infer_schema tests
    // =========================================================================

    #[test]
    fn test_infer_schema_object() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "age": 30});
        let expr = runtime.compile("infer_schema(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let schema = result.as_object().unwrap();
        assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "object");
        let props = schema.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("name"));
        assert!(props.contains_key("age"));
    }

    #[test]
    fn test_infer_schema_array() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3]);
        let expr = runtime.compile("infer_schema(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let schema = result.as_object().unwrap();
        assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "array");
        let items = schema.get("items").unwrap().as_object().unwrap();
        assert_eq!(items.get("type").unwrap().as_str().unwrap(), "number");
    }

    // =========================================================================
    // chunk_by_size tests
    // =========================================================================

    #[test]
    fn test_chunk_by_size() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime.compile("chunk_by_size(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let chunks = result.as_array().unwrap();
        assert!(chunks.len() > 1); // Should be split into multiple chunks
    }

    // =========================================================================
    // paginate tests
    // =========================================================================

    #[test]
    fn test_paginate() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let expr = runtime.compile("paginate(@, `2`, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();

        let page_data = obj.get("data").unwrap().as_array().unwrap();
        assert_eq!(page_data.len(), 3);
        assert_eq!(page_data[0].as_f64().unwrap() as i64, 4); // Page 2 starts at index 3

        assert_eq!(obj.get("page").unwrap().as_f64().unwrap() as i64, 2);
        assert_eq!(obj.get("total").unwrap().as_f64().unwrap() as i64, 10);
        assert_eq!(obj.get("total_pages").unwrap().as_f64().unwrap() as i64, 4);
        assert!(obj.get("has_next").unwrap().as_bool().unwrap());
        assert!(obj.get("has_prev").unwrap().as_bool().unwrap());
    }

    // =========================================================================
    // estimate_size tests
    // =========================================================================

    #[test]
    fn test_estimate_size() {
        let runtime = setup_runtime();
        let data = json!({"hello": "world"});
        let expr = runtime.compile("estimate_size(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let size = result.as_f64().unwrap() as i64;
        assert!(size > 0);
    }

    // =========================================================================
    // truncate_to_size tests
    // =========================================================================

    #[test]
    fn test_truncate_to_size_array() {
        let runtime = setup_runtime();
        let data = json!([1, 2, 3, 4, 5]);
        let expr = runtime.compile("truncate_to_size(@, `5`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.len() < 5); // Should be truncated
    }

    // =========================================================================
    // template tests
    // =========================================================================

    #[test]
    fn test_template_basic() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice", "age": 30});
        let expr = runtime
            .compile(r#"template(@, `"Hello {{name}}, you are {{age}} years old"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            "Hello alice, you are 30 years old"
        );
    }

    #[test]
    fn test_template_nested() {
        let runtime = setup_runtime();
        let data = json!({"user": {"name": "bob"}});
        let expr = runtime
            .compile(r#"template(@, `"Welcome {{user.name}}!"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Welcome bob!");
    }

    #[test]
    fn test_template_missing_default() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice"});
        let expr = runtime
            .compile(r#"template(@, `"{{name}} - {{title}}"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "alice - ");
    }

    #[test]
    fn test_template_fallback() {
        let runtime = setup_runtime();
        let data = json!({});
        let expr = runtime
            .compile(r#"template(@, `"Hello {{name|Guest}}"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello Guest");
    }

    #[test]
    fn test_template_null_template_error() {
        let runtime = setup_runtime();
        let data = json!({"name": "alice"});
        // Simulate what happens when user forgets backticks - the template string
        // evaluates as a field reference which returns null
        let expr = runtime.compile(r#"template(@, missing_field)"#).unwrap();
        let result = expr.search(&data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("second argument is null"),
            "Error should mention null argument: {}",
            err_msg
        );
        assert!(
            err_msg.contains("backticks"),
            "Error should mention backticks: {}",
            err_msg
        );
    }
}
