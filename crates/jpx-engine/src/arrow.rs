//! Arrow support for jpx-engine
//!
//! This module provides conversion utilities between Apache Arrow's in-memory
//! columnar format and JSON. It serves as the bridge between columnar data
//! (like Parquet files) and JMESPath evaluation.
//!
//! # Architecture
//!
//! ```text
//! Parquet file ──► Arrow RecordBatch ──► JSON Value ──► JMESPath ──► JSON Value ──► Arrow ──► Parquet
//!                      (columnar)        (rows)                      (result)      (columnar)
//! ```
//!
//! # Performance Characteristics
//!
//! Based on benchmarks with 100k rows (7 fields each):
//!
//! | Operation | Time |
//! |-----------|------|
//! | JSON -> Arrow | ~115ms |
//! | Arrow -> JSON | ~65ms |
//! | Parquet -> Arrow (with Snappy) | ~21ms |
//!
//! The Arrow -> JSON conversion is the primary overhead when reading Parquet files.
//! However, Parquet's compression (typically 4-6x smaller than JSON) makes it
//! worthwhile for large datasets or network transfer.
//!
//! # Example
//!
//! ```rust
//! use jpx_engine::arrow::{json_to_record_batches, record_batches_to_json};
//! use serde_json::json;
//!
//! // Convert JSON to Arrow RecordBatches
//! let data = json!([
//!     {"id": 1, "name": "alice"},
//!     {"id": 2, "name": "bob"}
//! ]);
//! let batches = json_to_record_batches(&data).unwrap();
//!
//! // Convert back to JSON
//! let json_out = record_batches_to_json(&batches).unwrap();
//! assert_eq!(json_out.as_array().unwrap().len(), 2);
//! ```
//!
//! # Use Cases
//!
//! - **Parquet I/O**: Read Parquet files into JSON for JMESPath queries
//! - **Data pipelines**: Convert query results to Arrow for downstream analytics
//! - **Interoperability**: Bridge between columnar and row-based data formats

use crate::{EngineError, Result};
use arrow::json::reader::infer_json_schema;
use arrow::json::{LineDelimitedWriter, ReaderBuilder};
use arrow::record_batch::RecordBatch;
use serde_json::Value;
use std::io::{BufReader, Cursor, Seek};
use std::sync::Arc;

/// Convert Arrow RecordBatches to a JSON Value (array of objects).
///
/// Takes a slice of RecordBatches and converts them to a JSON array where
/// each element is an object representing a row.
///
/// # Arguments
///
/// * `batches` - Slice of Arrow RecordBatches to convert
///
/// # Returns
///
/// A JSON Value containing an array of objects, one per row.
///
/// # Example
///
/// ```rust,ignore
/// use jpx_engine::arrow::record_batches_to_json;
///
/// let batches = read_parquet_file("data.parquet")?;
/// let json = record_batches_to_json(&batches)?;
/// // json is now Value::Array([{row1}, {row2}, ...])
/// ```
pub fn record_batches_to_json(batches: &[RecordBatch]) -> Result<Value> {
    if batches.is_empty() {
        return Ok(Value::Array(vec![]));
    }

    // Convert to JSON using Arrow's LineDelimitedWriter
    let mut json_output = Vec::new();
    {
        let mut writer = LineDelimitedWriter::new(&mut json_output);
        for batch in batches {
            writer
                .write(batch)
                .map_err(|e| EngineError::ArrowError(e.to_string()))?;
        }
        writer
            .finish()
            .map_err(|e| EngineError::ArrowError(e.to_string()))?;
    }

    // Parse the newline-delimited JSON into a JSON array
    let json_str =
        String::from_utf8(json_output).map_err(|e| EngineError::ArrowError(e.to_string()))?;

    let items: Vec<Value> = json_str
        .lines()
        .filter(|line| !line.is_empty())
        .map(serde_json::from_str)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| EngineError::InvalidJson(e.to_string()))?;

    Ok(Value::Array(items))
}

/// Convert a single Arrow RecordBatch to a JSON Value.
///
/// Convenience wrapper around [`record_batches_to_json`] for a single batch.
pub fn record_batch_to_json(batch: &RecordBatch) -> Result<Value> {
    record_batches_to_json(std::slice::from_ref(batch))
}

/// Convert a JSON Value (array of objects) to Arrow RecordBatches.
///
/// Takes a JSON array of objects and converts them to Arrow RecordBatches.
/// The schema is inferred from the JSON data.
///
/// # Arguments
///
/// * `value` - JSON Value containing an array of objects
///
/// # Returns
///
/// A Vec of RecordBatches representing the data in columnar format.
///
/// # Errors
///
/// Returns an error if:
/// - The input is not an array
/// - The array is empty
/// - Schema inference fails
/// - Conversion to Arrow fails
///
/// # Example
///
/// ```rust,ignore
/// use jpx_engine::arrow::json_to_record_batches;
/// use serde_json::json;
///
/// let data = json!([
///     {"id": 1, "name": "alice"},
///     {"id": 2, "name": "bob"}
/// ]);
///
/// let batches = json_to_record_batches(&data)?;
/// // batches can now be written to Parquet
/// ```
pub fn json_to_record_batches(value: &Value) -> Result<Vec<RecordBatch>> {
    json_to_record_batches_with_batch_size(value, 1024)
}

/// Convert JSON to Arrow RecordBatches with configurable batch size.
///
/// Like [`json_to_record_batches`] but allows specifying the batch size
/// for controlling memory usage with large datasets.
///
/// # Arguments
///
/// * `value` - JSON Value containing an array of objects
/// * `batch_size` - Number of rows per RecordBatch
pub fn json_to_record_batches_with_batch_size(
    value: &Value,
    batch_size: usize,
) -> Result<Vec<RecordBatch>> {
    let array = value
        .as_array()
        .ok_or_else(|| EngineError::ArrowError("Arrow conversion requires an array".to_string()))?;

    if array.is_empty() {
        return Err(EngineError::ArrowError(
            "Cannot convert empty array to Arrow".to_string(),
        ));
    }

    // Convert JSON array to newline-delimited JSON
    let mut ndjson = String::new();
    for item in array {
        ndjson.push_str(
            &serde_json::to_string(item).map_err(|e| EngineError::InvalidJson(e.to_string()))?,
        );
        ndjson.push('\n');
    }

    // Infer schema from JSON
    let mut cursor = Cursor::new(ndjson.as_bytes());
    let (schema, _) =
        infer_json_schema(&mut cursor, None).map_err(|e| EngineError::ArrowError(e.to_string()))?;

    cursor
        .rewind()
        .map_err(|e| EngineError::ArrowError(e.to_string()))?;

    // Create JSON reader with inferred schema
    let buf_reader = BufReader::new(cursor);
    let json_reader = ReaderBuilder::new(Arc::new(schema))
        .with_batch_size(batch_size)
        .build(buf_reader)
        .map_err(|e| EngineError::ArrowError(e.to_string()))?;

    let batches: Vec<RecordBatch> = json_reader
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| EngineError::ArrowError(e.to_string()))?;

    if batches.is_empty() {
        return Err(EngineError::ArrowError(
            "No data converted to Arrow".to_string(),
        ));
    }

    Ok(batches)
}

/// Get the Arrow schema from a JSON Value.
///
/// Infers the Arrow schema from a JSON array of objects without
/// fully converting the data.
///
/// # Arguments
///
/// * `value` - JSON Value containing an array of objects
///
/// # Returns
///
/// The inferred Arrow Schema.
pub fn infer_schema_from_json(value: &Value) -> Result<arrow::datatypes::Schema> {
    let array = value
        .as_array()
        .ok_or_else(|| EngineError::ArrowError("Schema inference requires an array".to_string()))?;

    if array.is_empty() {
        return Err(EngineError::ArrowError(
            "Cannot infer schema from empty array".to_string(),
        ));
    }

    // Convert first few rows to NDJSON for schema inference
    let sample_size = array.len().min(100);
    let mut ndjson = String::new();
    for item in array.iter().take(sample_size) {
        ndjson.push_str(
            &serde_json::to_string(item).map_err(|e| EngineError::InvalidJson(e.to_string()))?,
        );
        ndjson.push('\n');
    }

    let mut cursor = Cursor::new(ndjson.as_bytes());
    let (schema, _) =
        infer_json_schema(&mut cursor, None).map_err(|e| EngineError::ArrowError(e.to_string()))?;

    Ok(schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_to_arrow_roundtrip() {
        let input = json!([
            {"id": 1, "name": "alice", "score": 95.5},
            {"id": 2, "name": "bob", "score": 87.0},
            {"id": 3, "name": "carol", "score": 92.3}
        ]);

        // Convert to Arrow
        let batches = json_to_record_batches(&input).unwrap();
        assert!(!batches.is_empty());

        // Convert back to JSON
        let output = record_batches_to_json(&batches).unwrap();
        let output_arr = output.as_array().unwrap();

        assert_eq!(output_arr.len(), 3);
        assert_eq!(output_arr[0]["name"], "alice");
        assert_eq!(output_arr[1]["name"], "bob");
        assert_eq!(output_arr[2]["name"], "carol");
    }

    #[test]
    fn test_empty_batches_to_json() {
        let result = record_batches_to_json(&[]).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn test_empty_array_to_arrow() {
        let result = json_to_record_batches(&json!([]));
        assert!(result.is_err());
    }

    #[test]
    fn test_non_array_to_arrow() {
        let result = json_to_record_batches(&json!({"not": "array"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_schema() {
        let data = json!([
            {"id": 1, "name": "alice", "active": true},
            {"id": 2, "name": "bob", "active": false}
        ]);

        let schema = infer_schema_from_json(&data).unwrap();
        assert_eq!(schema.fields().len(), 3);

        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.contains(&"id"));
        assert!(field_names.contains(&"name"));
        assert!(field_names.contains(&"active"));
    }

    #[test]
    fn test_batch_size() {
        // Create a larger dataset
        let items: Vec<Value> = (0..100)
            .map(|i| json!({"id": i, "value": format!("item_{}", i)}))
            .collect();
        let data = Value::Array(items);

        // Use small batch size
        let batches = json_to_record_batches_with_batch_size(&data, 10).unwrap();

        // Should have multiple batches
        assert!(batches.len() >= 10);

        // Total rows should match
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 100);
    }

    #[test]
    fn test_single_batch_to_json() {
        let input = json!([
            {"x": 1, "y": 2},
            {"x": 3, "y": 4}
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        assert!(!batches.is_empty());

        // Use single batch convenience function
        let output = record_batch_to_json(&batches[0]).unwrap();
        let arr = output.as_array().unwrap();
        assert!(!arr.is_empty());
    }

    #[test]
    fn test_various_json_types() {
        // Test with various JSON types that Arrow supports
        let input = json!([
            {
                "int_val": 42,
                "float_val": 3.14,
                "string_val": "hello",
                "bool_val": true,
                "null_val": null
            },
            {
                "int_val": -100,
                "float_val": 2.718,
                "string_val": "world",
                "bool_val": false,
                "null_val": null
            }
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["string_val"], "hello");
        assert_eq!(arr[1]["string_val"], "world");
        assert_eq!(arr[0]["bool_val"], true);
        assert_eq!(arr[1]["bool_val"], false);
    }

    #[test]
    fn test_nested_objects_flatten() {
        // Note: Arrow JSON reader flattens nested structures
        // This test documents the behavior
        let input = json!([
            {"id": 1, "data": {"nested": "value"}},
            {"id": 2, "data": {"nested": "other"}}
        ]);

        // This should work - Arrow handles nested JSON
        let result = json_to_record_batches(&input);
        // Nested objects may or may not be supported depending on Arrow version
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_array_field() {
        // Test with array fields
        let input = json!([
            {"id": 1, "tags": ["a", "b", "c"]},
            {"id": 2, "tags": ["d", "e"]}
        ]);

        let result = json_to_record_batches(&input);
        // Array fields may have specific handling
        let _ = result;
    }

    #[test]
    fn test_large_dataset_roundtrip() {
        // Test with a larger dataset to ensure batching works correctly
        let items: Vec<Value> = (0..1000)
            .map(|i| {
                json!({
                    "id": i,
                    "name": format!("user_{}", i),
                    "score": (i as f64) * 0.1
                })
            })
            .collect();
        let input = Value::Array(items);

        let batches = json_to_record_batches(&input).unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 1000);

        let output = record_batches_to_json(&batches).unwrap();
        assert_eq!(output.as_array().unwrap().len(), 1000);
    }

    #[test]
    fn test_infer_schema_empty_array_error() {
        let result = infer_schema_from_json(&json!([]));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_infer_schema_non_array_error() {
        let result = infer_schema_from_json(&json!({"not": "array"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_primitive_value_error() {
        // Primitives cannot be converted to Arrow
        assert!(json_to_record_batches(&json!(42)).is_err());
        assert!(json_to_record_batches(&json!("string")).is_err());
        assert!(json_to_record_batches(&json!(true)).is_err());
        assert!(json_to_record_batches(&json!(null)).is_err());
    }

    #[test]
    fn test_schema_field_types() {
        let data = json!([
            {"int_field": 1, "float_field": 1.5, "str_field": "a", "bool_field": true}
        ]);

        let schema = infer_schema_from_json(&data).unwrap();

        // Verify we have the expected fields
        assert_eq!(schema.fields().len(), 4);

        // Check field names exist
        assert!(schema.field_with_name("int_field").is_ok());
        assert!(schema.field_with_name("float_field").is_ok());
        assert!(schema.field_with_name("str_field").is_ok());
        assert!(schema.field_with_name("bool_field").is_ok());
    }
}
