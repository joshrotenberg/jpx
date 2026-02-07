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
