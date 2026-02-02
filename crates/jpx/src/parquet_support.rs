//! Parquet input/output support for jpx CLI
//!
//! This module provides functions to read Parquet files into JSON
//! and write JSON results back to Parquet format.
//!
//! The conversion uses jpx-engine's Arrow support for JSON/Arrow conversion,
//! ensuring consistent behavior across the jpx ecosystem.

use anyhow::{Context, Result};
use jpx_engine::arrow::{json_to_record_batches, record_batches_to_json};
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use serde_json::Value;
use std::fs::File;
use std::path::Path;

/// Read a Parquet file and convert to JSON Value (array of objects).
///
/// Uses Arrow as the intermediate format:
/// Parquet -> Arrow RecordBatch -> JSON Value
pub fn read_parquet_to_json(path: &Path) -> Result<Value> {
    let file = File::open(path).context("Failed to open parquet file")?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .context("Failed to create parquet reader")?;

    let reader = builder.build().context("Failed to build parquet reader")?;

    // Collect all record batches
    let batches: Vec<_> = reader
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to read parquet batches")?;

    if batches.is_empty() {
        return Ok(Value::Array(vec![]));
    }

    // Use engine's Arrow support for conversion
    record_batches_to_json(&batches).context("Failed to convert Arrow to JSON")
}

/// Write JSON Value to a Parquet file.
///
/// Uses Arrow as the intermediate format:
/// JSON Value -> Arrow RecordBatch -> Parquet
///
/// The input should be an array of objects with consistent structure.
pub fn write_json_to_parquet(value: &Value, path: &Path) -> Result<()> {
    // Use engine's Arrow support for conversion
    let batches = json_to_record_batches(value).context("Failed to convert JSON to Arrow")?;

    if batches.is_empty() {
        anyhow::bail!("No data to write to parquet");
    }

    // Write to parquet
    let file = File::create(path).context("Failed to create parquet file")?;
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, batches[0].schema(), Some(props))
        .context("Failed to create parquet writer")?;

    for batch in &batches {
        writer.write(batch).context("Failed to write batch")?;
    }

    writer.close().context("Failed to close parquet writer")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn test_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        let input = json!([
            {"id": 1, "name": "alice", "score": 95.5},
            {"id": 2, "name": "bob", "score": 87.0},
            {"id": 3, "name": "carol", "score": 92.3}
        ]);

        // Write to parquet
        write_json_to_parquet(&input, &path).unwrap();

        // Read back
        let output = read_parquet_to_json(&path).unwrap();

        // Verify structure (values might have slightly different types due to arrow inference)
        let output_arr = output.as_array().unwrap();
        assert_eq!(output_arr.len(), 3);
        assert_eq!(output_arr[0]["name"], "alice");
        assert_eq!(output_arr[1]["name"], "bob");
        assert_eq!(output_arr[2]["name"], "carol");
    }

    #[test]
    fn test_empty_array() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.parquet");

        let result = write_json_to_parquet(&json!([]), &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_dataset() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("large.parquet");

        // Create a larger dataset
        let items: Vec<Value> = (0..1000)
            .map(|i| {
                json!({
                    "id": i,
                    "name": format!("user_{}", i),
                    "email": format!("user{}@example.com", i),
                    "active": i % 2 == 0
                })
            })
            .collect();
        let input = Value::Array(items);

        write_json_to_parquet(&input, &path).unwrap();

        let output = read_parquet_to_json(&path).unwrap();
        let output_arr = output.as_array().unwrap();
        assert_eq!(output_arr.len(), 1000);

        // Verify first and last records
        assert_eq!(output_arr[0]["name"], "user_0");
        assert_eq!(output_arr[999]["name"], "user_999");
    }

    #[test]
    fn test_various_types() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("types.parquet");

        let input = json!([
            {
                "int_val": 42,
                "float_val": 99.5,
                "string_val": "hello world",
                "bool_val": true,
                "null_val": null
            },
            {
                "int_val": -100,
                "float_val": 88.3,
                "string_val": "goodbye",
                "bool_val": false,
                "null_val": null
            }
        ]);

        write_json_to_parquet(&input, &path).unwrap();

        let output = read_parquet_to_json(&path).unwrap();
        let arr = output.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["string_val"], "hello world");
        assert_eq!(arr[1]["bool_val"], false);
    }

    #[test]
    fn test_file_not_found() {
        let result = read_parquet_to_json(Path::new("/nonexistent/path/file.parquet"));
        assert!(result.is_err());
    }

    #[test]
    fn test_non_array_input() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.parquet");

        // Object instead of array
        let result = write_json_to_parquet(&json!({"not": "array"}), &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_overwrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("overwrite.parquet");

        // Write first dataset
        let input1 = json!([{"id": 1, "value": "first"}]);
        write_json_to_parquet(&input1, &path).unwrap();

        // Overwrite with second dataset
        let input2 = json!([{"id": 2, "value": "second"}, {"id": 3, "value": "third"}]);
        write_json_to_parquet(&input2, &path).unwrap();

        // Read back - should have second dataset
        let output = read_parquet_to_json(&path).unwrap();
        let arr = output.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["value"], "second");
    }

    #[test]
    fn test_parquet_file_size_compression() {
        let dir = tempdir().unwrap();
        let parquet_path = dir.path().join("compressed.parquet");
        let json_path = dir.path().join("data.json");

        let categories = ["engineering", "sales", "marketing"];

        // Create dataset with repetitive data (good for compression)
        let items: Vec<Value> = (0..500)
            .map(|i| {
                json!({
                    "id": i,
                    "category": categories[i % 3],
                    "status": "active",
                    "description": "This is a sample description that repeats"
                })
            })
            .collect();
        let input = Value::Array(items);

        // Write JSON
        std::fs::write(&json_path, serde_json::to_string(&input).unwrap()).unwrap();

        // Write Parquet
        write_json_to_parquet(&input, &parquet_path).unwrap();

        // Parquet should be smaller due to compression
        let json_size = std::fs::metadata(&json_path).unwrap().len();
        let parquet_size = std::fs::metadata(&parquet_path).unwrap().len();

        // Parquet with Snappy compression should be significantly smaller
        assert!(
            parquet_size < json_size,
            "Parquet ({} bytes) should be smaller than JSON ({} bytes)",
            parquet_size,
            json_size
        );
    }
}
