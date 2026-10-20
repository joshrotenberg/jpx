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
use arrow::json::reader::infer_json_schema_from_iterator;
use arrow::json::{LineDelimitedWriter, ReaderBuilder, WriterBuilder};
use arrow::record_batch::RecordBatch;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::io::{BufReader, Cursor};
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
/// Every schema field is emitted, including null fields in nested objects.
/// Null objects/lists and null list elements remain null. Arrow cannot distinguish
/// an originally absent JSON field from an explicit null in the same column;
/// both are emitted as a present key with a null value.
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
        let mut writer: LineDelimitedWriter<_> = WriterBuilder::new()
            .with_explicit_nulls(true)
            .build(&mut json_output);
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

// Null list elements provide validity, not an element type. Arrow 60's schema
// inference dispatches on the first element and rejects nulls beside containers.
// Omit them only from the inference representation; the reader receives original
// JSON below, preserving null positions, list lengths and null/empty distinctions.
struct InferenceValue<'a>(&'a Value);

impl Serialize for InferenceValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self.0 {
            Value::Array(values) => serializer.collect_seq(
                values
                    .iter()
                    .filter(|value| !value.is_null())
                    .map(InferenceValue),
            ),
            Value::Object(fields) => serializer.collect_map(
                fields
                    .iter()
                    .map(|(key, value)| (key, InferenceValue(value))),
            ),
            value => value.serialize(serializer),
        }
    }
}

fn infer_nullable_schema<'a>(
    rows: impl Iterator<Item = &'a Value>,
) -> Result<arrow::datatypes::Schema> {
    let records = rows.map(|row| {
        // Process one row at a time and retain the previous serde JSON parsing
        // limits and numeric conversion, without building a second whole document.
        let text = serde_json::to_string(&InferenceValue(row))
            .map_err(|error| arrow::error::ArrowError::JsonError(error.to_string()))?;
        serde_json::from_str::<Value>(&text)
            .map_err(|error| arrow::error::ArrowError::JsonError(error.to_string()))
    });
    infer_json_schema_from_iterator(records)
        .map_err(|error| EngineError::ArrowError(error.to_string()))
}

/// Convert a JSON Value (array of objects) to Arrow RecordBatches.
///
/// Takes a JSON array of objects and converts them to Arrow RecordBatches.
/// The schema is inferred from all rows. Null list elements contribute no type
/// evidence; they remain in the data read into Arrow. Lists without any non-null
/// element evidence use Arrow's null element type.
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

    let schema = infer_nullable_schema(array.iter())?;

    // Read the original JSON, including every null element omitted for inference.
    let buf_reader = BufReader::new(Cursor::new(ndjson.as_bytes()));
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
/// The inferred Arrow Schema, sampling at most the first 100 rows. Null list
/// elements contribute no type evidence. Conversion uses all rows instead.
pub fn infer_schema_from_json(value: &Value) -> Result<arrow::datatypes::Schema> {
    let array = value
        .as_array()
        .ok_or_else(|| EngineError::ArrowError("Schema inference requires an array".to_string()))?;

    if array.is_empty() {
        return Err(EngineError::ArrowError(
            "Cannot infer schema from empty array".to_string(),
        ));
    }

    infer_nullable_schema(array.iter().take(100))
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
                "float_val": 3.125,
                "string_val": "hello",
                "bool_val": true,
                "null_val": null
            },
            {
                "int_val": -100,
                "float_val": 2.75,
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

    #[test]
    fn test_single_row_roundtrip() {
        let input = json!([{"a": 1}]);

        let batches = json_to_record_batches(&input).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 1);

        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["a"], 1);
    }

    #[test]
    fn test_all_null_values() {
        let input = json!([
            {"a": null},
            {"a": null}
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        assert!(!batches.is_empty());

        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 2);

        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        for row in arr {
            assert_eq!(row.as_object().unwrap().get("a"), Some(&Value::Null));
        }
        assert_eq!(output, input);
    }

    #[test]
    fn test_mixed_null_values() {
        let input = json!([
            {"a": 1},
            {"a": null},
            {"a": 3}
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();

        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["a"], 1);
        assert_eq!(arr[1].as_object().unwrap().get("a"), Some(&Value::Null));
        assert_eq!(arr[2]["a"], 3);
    }

    #[test]
    fn test_nullable_types_across_batches() {
        let input = json!([
            {"id":1,"text":null,"flag":null,"integer":null,"float":null},
            {"id":2,"text":"café 🦀","flag":false,"integer":-7,"float":1.25},
            {"id":3,"text":null,"flag":true,"integer":0,"float":null}
        ]);
        let batches = json_to_record_batches_with_batch_size(&input, 1).unwrap();
        assert_eq!(batches.len(), 3);
        let output = record_batches_to_json(&batches).unwrap();
        assert_eq!(output, input);
        for field in ["text", "flag", "integer", "float"] {
            assert_eq!(
                output[0].as_object().unwrap().get(field),
                Some(&Value::Null)
            );
        }
    }

    #[test]
    fn test_nullable_nested_structs_and_lists() {
        let input = json!([
            {"id":1,"record":{"name":"a","note":null},
                "items":[{"name":"x","note":null}],"tags":["a",null]},
            {"id":2,"record":null,"items":[],"tags":null},
            {"id":3,"record":{"name":null,"note":"present"},"items":null,"tags":[]}
        ]);
        let batches = json_to_record_batches(&input).unwrap();
        let output = record_batches_to_json(&batches).unwrap();
        assert_eq!(output, input);
        assert_eq!(output[0]["record"].get("note"), Some(&Value::Null));
        assert_eq!(output[0]["items"][0].get("note"), Some(&Value::Null));
        assert_eq!(output[1].get("record"), Some(&Value::Null));
    }

    #[test]
    fn test_arrow_null_struct_list_elements_are_preserved() {
        use arrow::array::{Array, ListBuilder, StringBuilder, StructBuilder};
        use arrow::datatypes::{DataType, Field, Schema};

        // External Arrow/Parquet data can contain null list elements of struct
        // type, even though JSON schema inference currently rejects that shape.
        let fields = vec![
            Field::new("name", DataType::Utf8, true),
            Field::new("note", DataType::Utf8, true),
        ];
        let fields = arrow::datatypes::Fields::from(fields);
        let mut items = ListBuilder::new(StructBuilder::from_fields(fields, 2));
        for (name, valid) in [(Some("x"), true), (None, false)] {
            items
                .values()
                .field_builder::<StringBuilder>(0)
                .unwrap()
                .append_option(name);
            items
                .values()
                .field_builder::<StringBuilder>(1)
                .unwrap()
                .append_null();
            items.values().append(valid);
        }
        items.append(true);
        items.append(false);
        let items = Arc::new(items.finish());
        let schema = Arc::new(Schema::new(vec![Field::new(
            "items",
            items.data_type().clone(),
            true,
        )]));
        let batch = RecordBatch::try_new(schema, vec![items]).unwrap();
        assert_eq!(
            record_batch_to_json(&batch).unwrap(),
            json!([{"items":[{"name":"x","note":null},null]}, {"items":null}])
        );
    }

    #[test]
    fn test_json_nullable_object_lists_in_either_order() {
        for items in [
            json!([null, {"name":"x","optional":null}]),
            json!([{"name":"x","optional":null}, null]),
        ] {
            let input = json!([{"items":items}]);
            let original = input.clone();
            let schema = infer_schema_from_json(&input).unwrap();
            let batches = json_to_record_batches(&input).unwrap();
            assert_eq!(schema, *batches[0].schema());
            assert_eq!(record_batches_to_json(&batches).unwrap(), input);
            assert_eq!(input, original);
        }
    }

    #[test]
    fn test_null_only_lists_before_struct_evidence_across_batches() {
        let input = json!([
            {"items":[null,null]},
            {"items":[]},
            {"items":null},
            {"items":[null,{"name":"x","optional":null}]},
            {"items":[{"name":null,"optional":"present"},null]}
        ]);
        let batches = json_to_record_batches_with_batch_size(&input, 1).unwrap();
        assert_eq!(batches.len(), 5);
        assert_eq!(record_batches_to_json(&batches).unwrap(), input);
        let mut reversed = input.as_array().unwrap().clone();
        reversed.reverse();
        let reversed = Value::Array(reversed);
        let batches = json_to_record_batches_with_batch_size(&reversed, 2).unwrap();
        assert_eq!(record_batches_to_json(&batches).unwrap(), reversed);
    }

    #[test]
    fn test_nested_nullable_lists_preserve_nulls_and_empty_lists() {
        let input = json!([
            {"items":[null,{"children":[null,{"value":"x","optional":null}]}],
                "matrix":[null,[null,1],[]],"only_nulls":[null,null]},
            {"items":[{"children":null}],"matrix":null,"only_nulls":[]},
            {"items":null,"matrix":[[2,null]],"only_nulls":null}
        ]);
        let batches = json_to_record_batches(&input).unwrap();
        assert_eq!(record_batches_to_json(&batches).unwrap(), input);
    }

    #[test]
    fn test_nullable_schema_preview_keeps_sample_limit_but_conversion_uses_all_rows() {
        use arrow::datatypes::DataType;
        let mut rows = vec![json!({"items":[null]}); 100];
        rows.push(json!({"items":[{"name":"last"},null]}));
        let input = Value::Array(rows);
        let schema = infer_schema_from_json(&input).unwrap();
        let DataType::List(item) = schema.field_with_name("items").unwrap().data_type() else {
            panic!("items must remain a list");
        };
        assert_eq!(item.data_type(), &DataType::Null);
        let batches = json_to_record_batches_with_batch_size(&input, 10).unwrap();
        assert_eq!(record_batches_to_json(&batches).unwrap(), input);
    }

    #[test]
    fn test_schema_inference_retains_json_recursion_limit() {
        let mut nested = json!(0);
        for _ in 0..130 {
            nested = json!([nested]);
        }
        let input = json!([{"nested":nested}]);
        for result in [
            infer_schema_from_json(&input).map(|_| ()),
            json_to_record_batches(&input).map(|_| ()),
        ] {
            let error = result.unwrap_err().to_string();
            assert!(error.contains("recursion limit exceeded"), "{error}");
        }
    }

    #[test]
    fn test_nullable_lists_still_reject_mixed_container_and_scalar_types() {
        for input in [
            json!([{"items":[null,{"name":"x"},1]}]),
            json!([{"items":[null,[1],{"name":"x"}]}]),
        ] {
            assert!(infer_schema_from_json(&input).is_err());
            assert!(json_to_record_batches(&input).is_err());
        }
    }

    #[test]
    fn test_schema_fields_missing_in_json_become_explicit_nulls() {
        // Arrow represents an absent JSON field and explicit null identically.
        let input = json!([
            {"id":1,"record":{"name":"a"}},
            {"id":2,"record":{},"optional":"present"},
            {"id":3}
        ]);
        let batches = json_to_record_batches(&input).unwrap();
        assert_eq!(
            record_batches_to_json(&batches).unwrap(),
            json!([
                {"id":1,"record":{"name":"a"},"optional":null},
                {"id":2,"record":{"name":null},"optional":"present"},
                {"id":3,"record":null,"optional":null}
            ])
        );
    }

    #[test]
    fn test_string_only_data() {
        let input = json!([
            {"first": "Alice", "last": "Smith"},
            {"first": "Bob", "last": "Jones"},
            {"first": "Carol", "last": "White"}
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();

        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["first"], "Alice");
        assert_eq!(arr[0]["last"], "Smith");
        assert_eq!(arr[1]["first"], "Bob");
        assert_eq!(arr[1]["last"], "Jones");
        assert_eq!(arr[2]["first"], "Carol");
        assert_eq!(arr[2]["last"], "White");
    }

    #[test]
    fn test_boolean_field_roundtrip() {
        let input = json!([
            {"name": "flag_a", "enabled": true},
            {"name": "flag_b", "enabled": false},
            {"name": "flag_c", "enabled": true}
        ]);

        let batches = json_to_record_batches(&input).unwrap();
        let output = record_batches_to_json(&batches).unwrap();
        let arr = output.as_array().unwrap();

        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["enabled"], true);
        assert_eq!(arr[1]["enabled"], false);
        assert_eq!(arr[2]["enabled"], true);
        assert_eq!(arr[0]["name"], "flag_a");
        assert_eq!(arr[1]["name"], "flag_b");
        assert_eq!(arr[2]["name"], "flag_c");
    }
}
