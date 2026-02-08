//! JSON utility functions: format, diff, patch, merge, keys, paths, stats.
//!
//! This module provides JSON manipulation and analysis tools that complement
//! the core JMESPath evaluation. These operations work directly on JSON data
//! without requiring JMESPath expressions.

use crate::JpxEngine;
use crate::error::{EngineError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Statistics about JSON data structure.
///
/// Provides insights into JSON data including type, size, depth, and
/// detailed field analysis for arrays of objects.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let stats = engine.stats(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#).unwrap();
///
/// println!("Type: {}", stats.root_type);      // "object"
/// println!("Size: {}", stats.size_human);     // "52 bytes"
/// println!("Depth: {}", stats.depth);         // 3
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    /// JSON type of the root value ("object", "array", "string", etc.)
    pub root_type: String,
    /// Size of the JSON string in bytes
    pub size_bytes: usize,
    /// Human-readable size (e.g., "1.5 KB", "2.3 MB")
    pub size_human: String,
    /// Maximum nesting depth (0 for primitives)
    pub depth: usize,
    /// Number of items (arrays only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// Number of keys (objects only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_count: Option<usize>,
    /// Field analysis (arrays of objects only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldAnalysis>>,
    /// Count of each JSON type in array (arrays only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_distribution: Option<HashMap<String, usize>>,
}

/// Analysis of a field across an array of objects.
///
/// Used by [`StatsResult`] to provide insights into consistent fields
/// in arrays of objects, including type information and null counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAnalysis {
    /// Field name (key)
    pub name: String,
    /// Most common type for this field
    pub field_type: String,
    /// Number of objects where this field is null
    pub null_count: usize,
    /// Number of distinct values (omitted for high-cardinality fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_count: Option<usize>,
}

/// Information about a path in a JSON structure.
///
/// Used by [`JpxEngine::paths`] to enumerate all paths in a JSON document.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let paths = engine.paths(r#"{"user": {"name": "alice"}}"#, true, false).unwrap();
///
/// for path in paths {
///     println!("{}: {:?}", path.path, path.path_type);
/// }
/// // Output:
/// // @: Some("object")
/// // user: Some("object")
/// // user.name: Some("string")
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// Path in dot notation (e.g., "user.name", "items.0.id")
    pub path: String,
    /// JSON type at this path (if `include_types` was true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
    /// Value at this path (if `include_values` was true, leaf nodes only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

// =============================================================================
// JpxEngine JSON utility methods
// =============================================================================

impl JpxEngine {
    /// Format JSON with configurable indentation.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON string to format
    /// * `indent` - Number of spaces per indentation level (0 = compact/minified)
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Pretty-print with 2-space indent
    /// let pretty = engine.format_json(r#"{"a":1,"b":2}"#, 2).unwrap();
    /// assert!(pretty.contains('\n'));
    ///
    /// // Compact/minified
    /// let compact = engine.format_json(r#"{"a":  1, "b": 2}"#, 0).unwrap();
    /// assert!(!compact.contains('\n'));
    /// ```
    pub fn format_json(&self, input: &str, indent: usize) -> Result<String> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        if indent == 0 {
            serde_json::to_string(&value).map_err(|e| EngineError::Internal(e.to_string()))
        } else {
            let indent_bytes = vec![b' '; indent];
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
            let mut buf = Vec::new();
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            value
                .serialize(&mut ser)
                .map_err(|e| EngineError::Internal(e.to_string()))?;
            String::from_utf8(buf).map_err(|e| EngineError::Internal(e.to_string()))
        }
    }

    /// Generate a JSON Patch (RFC 6902) from source to target.
    ///
    /// # Arguments
    ///
    /// * `source` - Original JSON string
    /// * `target` - Modified JSON string
    ///
    /// # Returns
    ///
    /// A JSON array of patch operations that transform `source` into `target`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let patch = engine.diff(r#"{"a": 1}"#, r#"{"a": 2}"#).unwrap();
    /// assert!(!patch.as_array().unwrap().is_empty());
    /// ```
    pub fn diff(&self, source: &str, target: &str) -> Result<Value> {
        let source_val: Value =
            serde_json::from_str(source).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let target_val: Value =
            serde_json::from_str(target).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let patch = json_patch::diff(&source_val, &target_val);
        serde_json::to_value(&patch).map_err(|e| EngineError::Internal(e.to_string()))
    }

    /// Apply a JSON Patch (RFC 6902) to a document.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON document to patch
    /// * `patch` - JSON array of patch operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let result = engine.patch(
    ///     r#"{"a": 1}"#,
    ///     r#"[{"op": "replace", "path": "/a", "value": 2}]"#,
    /// ).unwrap();
    /// assert_eq!(result, json!({"a": 2}));
    /// ```
    pub fn patch(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch: json_patch::Patch =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::patch(&mut doc, &patch)
            .map_err(|e| EngineError::evaluation_failed(e.to_string()))?;

        Ok(doc)
    }

    /// Apply a JSON Merge Patch (RFC 7396) to a document.
    ///
    /// Unlike JSON Patch (RFC 6902), merge patch uses a simple object overlay:
    /// - Present keys with non-null values are set
    /// - Present keys with null values are removed
    /// - Absent keys are unchanged
    ///
    /// # Arguments
    ///
    /// * `input` - JSON document to merge into
    /// * `patch` - JSON merge patch object
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let result = engine.merge(
    ///     r#"{"a": 1, "b": 2}"#,
    ///     r#"{"b": 3, "c": 4}"#,
    /// ).unwrap();
    /// assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    /// ```
    pub fn merge(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch_val: Value =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::merge(&mut doc, &patch_val);
        Ok(doc)
    }

    /// Extract keys from a JSON object.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON string (must be an object for non-recursive mode)
    /// * `recursive` - If `true`, extracts all nested paths in dot notation
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Top-level keys (sorted)
    /// let keys = engine.keys(r#"{"b": 1, "a": {"c": 2}}"#, false).unwrap();
    /// assert_eq!(keys, vec!["a", "b"]);
    ///
    /// // Recursive (dot-notation paths)
    /// let keys = engine.keys(r#"{"a": {"c": 2}, "b": 1}"#, true).unwrap();
    /// assert!(keys.contains(&"a.c".to_string()));
    /// ```
    pub fn keys(&self, input: &str, recursive: bool) -> Result<Vec<String>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut keys = Vec::new();
        if recursive {
            extract_keys_recursive(&value, "", &mut keys);
        } else if let Value::Object(map) = &value {
            keys = map.keys().cloned().collect();
            keys.sort();
        }
        Ok(keys)
    }

    /// Extract all paths from a JSON document.
    ///
    /// Enumerates every path in the JSON structure, optionally including
    /// type information and leaf values.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON string to analyze
    /// * `include_types` - If `true`, each path includes its JSON type
    /// * `include_values` - If `true`, leaf paths include their values
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let paths = engine.paths(r#"{"user": {"name": "alice"}}"#, true, false).unwrap();
    /// assert!(paths.iter().any(|p| p.path == "user.name"));
    /// ```
    pub fn paths(
        &self,
        input: &str,
        include_types: bool,
        include_values: bool,
    ) -> Result<Vec<PathInfo>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut paths = Vec::new();
        extract_paths(&value, "", include_types, include_values, &mut paths);
        Ok(paths)
    }

    /// Analyze JSON data and return structural statistics.
    ///
    /// Returns type information, size, depth, and for arrays of objects,
    /// per-field analysis including type distribution and null counts.
    ///
    /// # Arguments
    ///
    /// * `input` - JSON string to analyze
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let stats = engine.stats(r#"[{"name": "alice"}, {"name": "bob"}]"#).unwrap();
    /// assert_eq!(stats.root_type, "array");
    /// assert_eq!(stats.length, Some(2));
    /// assert!(stats.fields.is_some());
    /// ```
    pub fn stats(&self, input: &str) -> Result<StatsResult> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let size_bytes = input.len();
        let depth = calculate_depth(&value);

        let (length, key_count, fields, type_distribution) = match &value {
            Value::Array(arr) => {
                let type_dist = calculate_type_distribution(arr);
                let field_analysis = if arr.iter().all(|v| v.is_object()) {
                    Some(analyze_array_fields(arr))
                } else {
                    None
                };
                (Some(arr.len()), None, field_analysis, Some(type_dist))
            }
            Value::Object(map) => (None, Some(map.len()), None, None),
            _ => (None, None, None, None),
        };

        Ok(StatsResult {
            root_type: json_type_name(&value).to_string(),
            size_bytes,
            size_human: format_size(size_bytes),
            depth,
            length,
            key_count,
            fields,
            type_distribution,
        })
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Extract keys recursively from a JSON value
fn extract_keys_recursive(value: &Value, prefix: &str, keys: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                keys.push(path.clone());
                extract_keys_recursive(v, &path, keys);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let path = format!("{}.{}", prefix, i);
                extract_keys_recursive(v, &path, keys);
            }
        }
        _ => {}
    }
}

/// Extract paths from a JSON value
fn extract_paths(
    value: &Value,
    prefix: &str,
    include_types: bool,
    include_values: bool,
    paths: &mut Vec<PathInfo>,
) {
    let current_path = if prefix.is_empty() {
        "@".to_string()
    } else {
        prefix.to_string()
    };

    match value {
        Value::Object(map) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("object".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        Value::Array(arr) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("array".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}.{}", prefix, i);
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        _ => {
            paths.push(PathInfo {
                path: current_path,
                path_type: if include_types {
                    Some(json_type_name(value).to_string())
                } else {
                    None
                },
                value: if include_values {
                    Some(value.clone())
                } else {
                    None
                },
            });
        }
    }
}

/// Calculate the nesting depth of a JSON value
fn calculate_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(calculate_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(calculate_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Get the type name of a JSON value
fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Calculate type distribution in an array
fn calculate_type_distribution(arr: &[Value]) -> HashMap<String, usize> {
    let mut dist = HashMap::new();
    for item in arr {
        *dist.entry(json_type_name(item).to_string()).or_insert(0) += 1;
    }
    dist
}

/// Analyze fields in an array of objects
fn analyze_array_fields(arr: &[Value]) -> Vec<FieldAnalysis> {
    let mut field_types: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut field_null_counts: HashMap<String, usize> = HashMap::new();
    let mut field_values: HashMap<String, Vec<Value>> = HashMap::new();

    for item in arr {
        if let Value::Object(map) = item {
            for (k, v) in map {
                let types = field_types.entry(k.clone()).or_default();
                *types.entry(json_type_name(v).to_string()).or_insert(0) += 1;

                if v.is_null() {
                    *field_null_counts.entry(k.clone()).or_insert(0) += 1;
                }

                // Track unique values for low-cardinality detection
                let values = field_values.entry(k.clone()).or_default();
                if values.len() < 100 && !values.contains(v) {
                    values.push(v.clone());
                }
            }
        }
    }

    let mut fields: Vec<FieldAnalysis> = field_types
        .into_iter()
        .map(|(name, types)| {
            let predominant_type = types
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(t, _)| t)
                .unwrap_or_else(|| "unknown".to_string());

            let null_count = field_null_counts.get(&name).copied().unwrap_or(0);
            let unique_count = field_values.get(&name).map(|v| v.len());

            FieldAnalysis {
                name,
                field_type: predominant_type,
                null_count,
                unique_count,
            }
        })
        .collect();

    fields.sort_by(|a, b| a.name.cmp(&b.name));
    fields
}

/// Format size in human-readable form
fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::JpxEngine;
    use serde_json::json;

    #[test]
    fn test_format_json() {
        let engine = JpxEngine::new();

        let formatted = engine.format_json(r#"{"a":1,"b":2}"#, 2).unwrap();
        assert!(formatted.contains('\n'));

        let compact = engine.format_json(r#"{"a":1,"b":2}"#, 0).unwrap();
        assert!(!compact.contains('\n'));
    }

    #[test]
    fn test_diff() {
        let engine = JpxEngine::new();

        let patch = engine.diff(r#"{"a": 1}"#, r#"{"a": 2}"#).unwrap();

        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
    }

    #[test]
    fn test_patch() {
        let engine = JpxEngine::new();

        let result = engine
            .patch(
                r#"{"a": 1}"#,
                r#"[{"op": "replace", "path": "/a", "value": 2}]"#,
            )
            .unwrap();

        assert_eq!(result, json!({"a": 2}));
    }

    #[test]
    fn test_merge() {
        let engine = JpxEngine::new();

        let result = engine
            .merge(r#"{"a": 1, "b": 2}"#, r#"{"b": 3, "c": 4}"#)
            .unwrap();

        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_keys() {
        let engine = JpxEngine::new();

        let keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, false).unwrap();
        assert_eq!(keys, vec!["a", "b"]);

        let recursive_keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, true).unwrap();
        assert!(recursive_keys.contains(&"b.c".to_string()));
    }

    #[test]
    fn test_paths() {
        let engine = JpxEngine::new();

        let paths = engine.paths(r#"{"a": 1}"#, true, false).unwrap();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_stats() {
        let engine = JpxEngine::new();

        let stats = engine.stats(r#"[1, 2, 3]"#).unwrap();
        assert_eq!(stats.root_type, "array");
        assert_eq!(stats.length, Some(3));
    }

    // =========================================================================
    // format_json additional tests
    // =========================================================================

    #[test]
    fn test_format_json_invalid_json() {
        let engine = JpxEngine::new();
        let result = engine.format_json("not json", 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_json_indent_4() {
        let engine = JpxEngine::new();
        let formatted = engine.format_json(r#"{"a":1}"#, 4).unwrap();
        // The second line should start with 4 spaces
        let lines: Vec<&str> = formatted.lines().collect();
        assert!(lines.len() > 1);
        assert!(lines[1].starts_with("    "));
    }

    #[test]
    fn test_format_json_preserves_data() {
        let engine = JpxEngine::new();
        let input = r#"{"name":"alice","age":30,"active":true,"score":null}"#;
        let formatted = engine.format_json(input, 2).unwrap();
        let original: serde_json::Value = serde_json::from_str(input).unwrap();
        let roundtripped: serde_json::Value = serde_json::from_str(&formatted).unwrap();
        assert_eq!(original, roundtripped);
    }

    // =========================================================================
    // diff additional tests
    // =========================================================================

    #[test]
    fn test_diff_identical() {
        let engine = JpxEngine::new();
        let patch = engine.diff(r#"{"a":1,"b":2}"#, r#"{"a":1,"b":2}"#).unwrap();
        let patch_arr = patch.as_array().unwrap();
        assert!(patch_arr.is_empty());
    }

    #[test]
    fn test_diff_added_key() {
        let engine = JpxEngine::new();
        let patch = engine.diff(r#"{"a":1}"#, r#"{"a":1,"b":2}"#).unwrap();
        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
        let has_add = patch_arr
            .iter()
            .any(|op| op.get("op").and_then(|v| v.as_str()) == Some("add"));
        assert!(has_add, "Expected an 'add' operation in the patch");
    }

    #[test]
    fn test_diff_removed_key() {
        let engine = JpxEngine::new();
        let patch = engine.diff(r#"{"a":1,"b":2}"#, r#"{"a":1}"#).unwrap();
        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
        let has_remove = patch_arr
            .iter()
            .any(|op| op.get("op").and_then(|v| v.as_str()) == Some("remove"));
        assert!(has_remove, "Expected a 'remove' operation in the patch");
    }

    #[test]
    fn test_diff_nested_change() {
        let engine = JpxEngine::new();
        let patch = engine.diff(r#"{"a":{"b":1}}"#, r#"{"a":{"b":2}}"#).unwrap();
        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
        // The operation should target the nested path /a/b
        let targets_nested = patch_arr.iter().any(|op| {
            op.get("path")
                .and_then(|v| v.as_str())
                .map(|p| p.contains("/a/b"))
                .unwrap_or(false)
        });
        assert!(targets_nested, "Expected a patch operation targeting /a/b");
    }

    #[test]
    fn test_diff_invalid_json() {
        let engine = JpxEngine::new();
        let result = engine.diff("not json", r#"{"a":1}"#);
        assert!(result.is_err());
    }

    // =========================================================================
    // patch additional tests
    // =========================================================================

    #[test]
    fn test_patch_add_operation() {
        let engine = JpxEngine::new();
        let result = engine
            .patch(r#"{"a":1}"#, r#"[{"op":"add","path":"/b","value":2}]"#)
            .unwrap();
        assert_eq!(result, json!({"a": 1, "b": 2}));
    }

    #[test]
    fn test_patch_remove_operation() {
        let engine = JpxEngine::new();
        let result = engine
            .patch(r#"{"a":1,"b":2}"#, r#"[{"op":"remove","path":"/b"}]"#)
            .unwrap();
        assert_eq!(result, json!({"a": 1}));
    }

    #[test]
    fn test_patch_invalid_patch() {
        let engine = JpxEngine::new();
        let result = engine.patch(r#"{"a":1}"#, r#"not a patch"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_patch_empty_patch() {
        let engine = JpxEngine::new();
        let result = engine.patch(r#"{"a":1,"b":2}"#, r#"[]"#).unwrap();
        assert_eq!(result, json!({"a": 1, "b": 2}));
    }

    // =========================================================================
    // merge additional tests
    // =========================================================================

    #[test]
    fn test_merge_overlapping_keys() {
        let engine = JpxEngine::new();
        let result = engine
            .merge(r#"{"a":1,"b":2}"#, r#"{"a":10,"b":20}"#)
            .unwrap();
        assert_eq!(result, json!({"a": 10, "b": 20}));
    }

    #[test]
    fn test_merge_null_removes_key() {
        let engine = JpxEngine::new();
        let result = engine.merge(r#"{"a":1,"b":2}"#, r#"{"b":null}"#).unwrap();
        assert_eq!(result, json!({"a": 1}));
    }

    #[test]
    fn test_merge_nested_objects() {
        let engine = JpxEngine::new();
        let result = engine
            .merge(r#"{"a":{"x":1,"y":2},"b":3}"#, r#"{"a":{"y":20,"z":30}}"#)
            .unwrap();
        assert_eq!(result, json!({"a": {"x": 1, "y": 20, "z": 30}, "b": 3}));
    }

    #[test]
    fn test_merge_invalid_json() {
        let engine = JpxEngine::new();
        let result = engine.merge("not json", r#"{"a":1}"#);
        assert!(result.is_err());
    }

    // =========================================================================
    // keys additional tests
    // =========================================================================

    #[test]
    fn test_keys_empty_object() {
        let engine = JpxEngine::new();
        let keys = engine.keys(r#"{}"#, false).unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_keys_recursive_nested() {
        let engine = JpxEngine::new();
        let keys = engine.keys(r#"{"a":{"b":{"c":1}},"d":2}"#, true).unwrap();
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"a.b".to_string()));
        assert!(keys.contains(&"a.b.c".to_string()));
        assert!(keys.contains(&"d".to_string()));
    }

    #[test]
    fn test_keys_non_object_non_recursive() {
        let engine = JpxEngine::new();
        // Arrays do not have string keys, so non-recursive should return empty
        let keys = engine.keys(r#"[1, 2, 3]"#, false).unwrap();
        assert!(keys.is_empty());
    }

    // =========================================================================
    // paths additional tests
    // =========================================================================

    #[test]
    fn test_paths_with_types_and_values() {
        let engine = JpxEngine::new();
        let paths = engine
            .paths(r#"{"name":"alice","age":30}"#, true, true)
            .unwrap();
        // Find the "name" path
        let name_path = paths.iter().find(|p| p.path == "name").unwrap();
        assert_eq!(name_path.path_type, Some("string".to_string()));
        assert_eq!(name_path.value, Some(json!("alice")));
        // Find the "age" path
        let age_path = paths.iter().find(|p| p.path == "age").unwrap();
        assert_eq!(age_path.path_type, Some("number".to_string()));
        assert_eq!(age_path.value, Some(json!(30)));
    }

    #[test]
    fn test_paths_root_is_at_sign() {
        let engine = JpxEngine::new();
        let paths = engine.paths(r#"{"a":1}"#, false, false).unwrap();
        assert!(!paths.is_empty());
        assert_eq!(paths[0].path, "@");
    }

    #[test]
    fn test_paths_array_indices() {
        let engine = JpxEngine::new();
        let paths = engine.paths(r#"[10, 20, 30]"#, true, true).unwrap();
        // Root should be "@" with type "array"
        assert_eq!(paths[0].path, "@");
        assert_eq!(paths[0].path_type, Some("array".to_string()));
        // Elements should be indexed as .0, .1, .2
        let index_paths: Vec<&str> = paths.iter().map(|p| p.path.as_str()).collect();
        assert!(index_paths.contains(&".0"));
        assert!(index_paths.contains(&".1"));
        assert!(index_paths.contains(&".2"));
    }

    // =========================================================================
    // stats additional tests
    // =========================================================================

    #[test]
    fn test_stats_object() {
        let engine = JpxEngine::new();
        let stats = engine
            .stats(r#"{"name":"alice","age":30,"active":true}"#)
            .unwrap();
        assert_eq!(stats.root_type, "object");
        assert_eq!(stats.key_count, Some(3));
        assert!(stats.length.is_none());
    }

    #[test]
    fn test_stats_empty_array() {
        let engine = JpxEngine::new();
        let stats = engine.stats(r#"[]"#).unwrap();
        assert_eq!(stats.root_type, "array");
        assert_eq!(stats.length, Some(0));
        assert_eq!(stats.depth, 1);
    }

    #[test]
    fn test_stats_array_of_objects_fields() {
        let engine = JpxEngine::new();
        let stats = engine
            .stats(r#"[{"name":"alice","age":30},{"name":"bob","age":25}]"#)
            .unwrap();
        assert_eq!(stats.root_type, "array");
        assert_eq!(stats.length, Some(2));
        let fields = stats.fields.unwrap();
        let field_names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"name"));
        assert!(field_names.contains(&"age"));
    }

    #[test]
    fn test_stats_nested_depth() {
        let engine = JpxEngine::new();
        // 4 levels deep: root obj -> a obj -> b obj -> c obj -> value
        let stats = engine.stats(r#"{"a":{"b":{"c":{"d":1}}}}"#).unwrap();
        assert_eq!(stats.depth, 4);
    }

    #[test]
    fn test_stats_invalid_json() {
        let engine = JpxEngine::new();
        let result = engine.stats("not json");
        assert!(result.is_err());
    }

    // =========================================================================
    // Private helper function tests
    // =========================================================================

    #[test]
    fn test_calculate_depth_primitive() {
        use super::calculate_depth;
        assert_eq!(calculate_depth(&json!(42)), 0);
        assert_eq!(calculate_depth(&json!("hello")), 0);
        assert_eq!(calculate_depth(&json!(true)), 0);
        assert_eq!(calculate_depth(&json!(null)), 0);
    }

    #[test]
    fn test_calculate_depth_nested() {
        use super::calculate_depth;
        // 3 levels: obj -> obj -> obj -> primitive
        let value = json!({"a": {"b": {"c": 1}}});
        assert_eq!(calculate_depth(&value), 3);
    }

    #[test]
    fn test_format_size_bytes() {
        use super::format_size;
        assert_eq!(format_size(100), "100 bytes");
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(1023), "1023 bytes");
    }

    #[test]
    fn test_format_size_kb() {
        use super::format_size;
        let result = format_size(1024);
        assert!(
            result.contains("KB"),
            "Expected KB in '{}' for 1024 bytes",
            result
        );
        let result = format_size(2048);
        assert!(
            result.contains("KB"),
            "Expected KB in '{}' for 2048 bytes",
            result
        );
    }

    #[test]
    fn test_format_size_mb() {
        use super::format_size;
        let result = format_size(1024 * 1024);
        assert!(result.contains("MB"), "Expected MB in '{}' for 1MB", result);
        let result = format_size(2 * 1024 * 1024);
        assert!(result.contains("MB"), "Expected MB in '{}' for 2MB", result);
    }
}
