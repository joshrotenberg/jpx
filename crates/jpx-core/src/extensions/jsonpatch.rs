//! JSON Patch (RFC 6902) functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::{Function, custom_error};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// json_patch(obj, patch) -> object (RFC 6902)
// Apply a JSON Patch (RFC 6902) to an object.
// =============================================================================

defn!(JsonPatchFn, vec![arg!(any), arg!(array)], None);

impl Function for JsonPatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        // json-patch works with serde_json::Value directly -- no conversion needed
        let mut result = args[0].clone();

        let patch: json_patch::Patch = serde_json::from_value(args[1].clone())
            .map_err(|e| custom_error(ctx, &format!("Invalid JSON Patch format: {}", e)))?;

        json_patch::patch(&mut result, &patch)
            .map_err(|e| custom_error(ctx, &format!("Failed to apply patch: {}", e)))?;

        Ok(result)
    }
}

// =============================================================================
// json_merge_patch(obj, patch) -> object (RFC 7396)
// Apply a JSON Merge Patch (RFC 7396) to an object.
// =============================================================================

defn!(JsonMergePatchFn, vec![arg!(any), arg!(any)], None);

impl Function for JsonMergePatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let mut result = args[0].clone();
        json_patch::merge(&mut result, &args[1]);

        Ok(result)
    }
}

// =============================================================================
// json_diff(a, b) -> array (RFC 6902 JSON Patch)
// Generate a JSON Patch (RFC 6902) that transforms the first object into the second.
// =============================================================================

defn!(JsonDiffFn, vec![arg!(any), arg!(any)], None);

impl Function for JsonDiffFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let patch = json_patch::diff(&args[0], &args[1]);

        let patch_json = serde_json::to_value(&patch)
            .map_err(|e| custom_error(ctx, &format!("Failed to serialize patch: {}", e)))?;

        Ok(patch_json)
    }
}

/// Register JSON patch functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "json_patch", enabled, Box::new(JsonPatchFn::new()));
    register_if_enabled(
        runtime,
        "json_merge_patch",
        enabled,
        Box::new(JsonMergePatchFn::new()),
    );
    register_if_enabled(runtime, "json_diff", enabled, Box::new(JsonDiffFn::new()));
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
    fn test_json_patch_add() {
        let runtime = setup_runtime();
        let data = json!({"doc": {"a": 1}, "patch": [{"op": "add", "path": "/b", "value": 2}]});
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_patch_remove() {
        let runtime = setup_runtime();
        let data = json!({"doc": {"a": 1, "b": 2}, "patch": [{"op": "remove", "path": "/b"}]});
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert!(obj.get("b").is_none());
    }

    #[test]
    fn test_json_patch_replace() {
        let runtime = setup_runtime();
        let data =
            json!({"doc": {"a": 1}, "patch": [{"op": "replace", "path": "/a", "value": 99}]});
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 99);
    }

    #[test]
    fn test_json_patch_multiple_ops() {
        let runtime = setup_runtime();
        let data = json!({
            "doc": {"a": 1},
            "patch": [
                {"op": "add", "path": "/b", "value": 2},
                {"op": "replace", "path": "/a", "value": 10}
            ]
        });
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 10);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_merge_patch_simple() {
        let runtime = setup_runtime();
        let data = json!({"doc": {"a": 1, "b": 2}, "patch": {"b": 3, "c": 4}});
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_f64().unwrap() as i64, 3);
        assert_eq!(obj.get("c").unwrap().as_f64().unwrap() as i64, 4);
    }

    #[test]
    fn test_json_merge_patch_remove_with_null() {
        let runtime = setup_runtime();
        let data = json!({"doc": {"a": 1, "b": 2}, "patch": {"b": null}});
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_f64().unwrap() as i64, 1);
        assert!(obj.get("b").is_none());
    }

    #[test]
    fn test_json_merge_patch_nested() {
        let runtime = setup_runtime();
        let data = json!({"doc": {"a": {"x": 1}}, "patch": {"a": {"y": 2}}});
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("x").unwrap().as_f64().unwrap() as i64, 1);
        assert_eq!(a.get("y").unwrap().as_f64().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_diff_add() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 1, "y": 2}});
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_str().unwrap(), "add");
        assert_eq!(op.get("path").unwrap().as_str().unwrap(), "/y");
    }

    #[test]
    fn test_json_diff_remove() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1, "y": 2}, "b": {"x": 1}});
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_str().unwrap(), "remove");
        assert_eq!(op.get("path").unwrap().as_str().unwrap(), "/y");
    }

    #[test]
    fn test_json_diff_replace() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 2}});
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_str().unwrap(), "replace");
        assert_eq!(op.get("path").unwrap().as_str().unwrap(), "/x");
    }

    #[test]
    fn test_json_diff_no_changes() {
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 1}});
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_json_diff_roundtrip() {
        // Generate a diff and apply it - should get the same result
        let runtime = setup_runtime();
        let data = json!({"a": {"x": 1}, "b": {"x": 2, "y": 3}});

        // First get the diff
        let diff_expr = runtime.compile("json_diff(a, b)").unwrap();
        let diff_result = diff_expr.search(&data).unwrap();

        // Now apply the diff to a
        let data_with_patch = json!({
            "doc": {"x": 1},
            "patch": diff_result
        });
        let patch_expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let patched = patch_expr.search(&data_with_patch).unwrap();

        // Should equal b
        let obj = patched.as_object().unwrap();
        assert_eq!(obj.get("x").unwrap().as_f64().unwrap() as i64, 2);
        assert_eq!(obj.get("y").unwrap().as_f64().unwrap() as i64, 3);
    }
}
