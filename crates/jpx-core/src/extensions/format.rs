//! CSV and TSV formatting functions.

use std::collections::HashSet;

use csv::WriterBuilder;
use serde_json::Value;

use crate::functions::{Function, custom_error};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Convert a serde_json::Value to a string suitable for CSV field.
fn value_to_csv_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Write a single row using the csv crate's writer.
fn write_csv_row(fields: &[String], delimiter: u8) -> Result<String, std::io::Error> {
    let mut wtr = WriterBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_writer(vec![]);

    wtr.write_record(fields)?;
    wtr.flush()?;

    let data = wtr
        .into_inner()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut s = String::from_utf8(data).unwrap_or_default();
    if s.ends_with('\n') {
        s.pop();
    }
    if s.ends_with('\r') {
        s.pop();
    }
    Ok(s)
}

/// Write multiple rows using the csv crate's writer.
fn write_csv_rows(rows: &[Vec<String>], delimiter: u8) -> Result<String, std::io::Error> {
    let mut wtr = WriterBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_writer(vec![]);

    for row in rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;

    let data = wtr
        .into_inner()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut s = String::from_utf8(data).unwrap_or_default();
    if s.ends_with('\n') {
        s.pop();
    }
    if s.ends_with('\r') {
        s.pop();
    }
    Ok(s)
}

// =============================================================================
// to_csv(array) -> string
// =============================================================================

defn!(ToCsvFn, vec![arg!(array)], None);

impl Function for ToCsvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::String(String::new()));
        }

        let fields: Vec<String> = arr.iter().map(value_to_csv_string).collect();

        match write_csv_row(&fields, b',') {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Err(custom_error(ctx, &format!("CSV write error: {}", e))),
        }
    }
}

// =============================================================================
// to_tsv(array) -> string
// =============================================================================

defn!(ToTsvFn, vec![arg!(array)], None);

impl Function for ToTsvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Value::String(String::new()));
        }

        let fields: Vec<String> = arr.iter().map(value_to_csv_string).collect();

        match write_csv_row(&fields, b'\t') {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Err(custom_error(ctx, &format!("TSV write error: {}", e))),
        }
    }
}

// =============================================================================
// to_csv_rows(array_of_arrays) -> string
// =============================================================================

defn!(ToCsvRowsFn, vec![arg!(array)], None);

impl Function for ToCsvRowsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let rows_var = args[0].as_array().unwrap();

        if rows_var.is_empty() {
            return Ok(Value::String(String::new()));
        }

        let rows: Vec<Vec<String>> = rows_var
            .iter()
            .map(|row| {
                if let Some(arr) = row.as_array() {
                    arr.iter().map(value_to_csv_string).collect()
                } else {
                    vec![value_to_csv_string(row)]
                }
            })
            .collect();

        match write_csv_rows(&rows, b',') {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Err(custom_error(ctx, &format!("CSV write error: {}", e))),
        }
    }
}

// =============================================================================
// to_csv_table(array_of_objects, columns?) -> string
// =============================================================================

defn!(ToCsvTableFn, vec![arg!(array)], Some(arg!(array)));

impl Function for ToCsvTableFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let rows = args[0].as_array().unwrap();

        if rows.is_empty() {
            return Ok(Value::String(String::new()));
        }

        // Determine columns: from second argument or from first object's keys
        let columns: Vec<String> = if args.len() > 1 {
            args[1]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else if let Some(obj) = rows[0].as_object() {
            let mut keys: Vec<String> = obj.keys().cloned().collect();
            keys.sort();
            keys
        } else {
            return Ok(Value::String(String::new()));
        };

        if columns.is_empty() {
            return Ok(Value::String(String::new()));
        }

        let mut all_rows: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);

        // Header row
        all_rows.push(columns.clone());

        // Data rows
        for row in rows.iter() {
            if let Some(obj) = row.as_object() {
                let data_row: Vec<String> = columns
                    .iter()
                    .map(|col| obj.get(col).map(value_to_csv_string).unwrap_or_default())
                    .collect();
                all_rows.push(data_row);
            } else {
                all_rows.push(columns.iter().map(|_| String::new()).collect());
            }
        }

        match write_csv_rows(&all_rows, b',') {
            Ok(s) => Ok(Value::String(s)),
            Err(e) => Err(custom_error(ctx, &format!("CSV write error: {}", e))),
        }
    }
}

// =============================================================================
// from_csv(string) -> array of arrays
// =============================================================================

defn!(FromCsvFn, vec![arg!(string)], None);

impl Function for FromCsvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let input = args[0].as_str().unwrap();
        parse_delimited(input, b',', ctx)
    }
}

// =============================================================================
// from_tsv(string) -> array of arrays
// =============================================================================

defn!(FromTsvFn, vec![arg!(string)], None);

impl Function for FromTsvFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let input = args[0].as_str().unwrap();
        parse_delimited(input, b'\t', ctx)
    }
}

/// Parse a delimited string (CSV or TSV) into an array of arrays.
fn parse_delimited(input: &str, delimiter: u8, ctx: &Context<'_>) -> SearchResult {
    use csv::ReaderBuilder;

    if input.trim().is_empty() {
        return Ok(Value::Array(vec![]));
    }

    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(input.as_bytes());

    let mut rows: Vec<Value> = Vec::new();

    for result in reader.records() {
        match result {
            Ok(record) => {
                let row: Vec<Value> = record
                    .iter()
                    .map(|field| Value::String(field.to_string()))
                    .collect();
                rows.push(Value::Array(row));
            }
            Err(e) => {
                return Err(custom_error(ctx, &format!("CSV parse error: {}", e)));
            }
        }
    }

    Ok(Value::Array(rows))
}

/// Register format functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "to_csv", enabled, Box::new(ToCsvFn::new()));
    register_if_enabled(runtime, "to_tsv", enabled, Box::new(ToTsvFn::new()));
    register_if_enabled(
        runtime,
        "to_csv_rows",
        enabled,
        Box::new(ToCsvRowsFn::new()),
    );
    register_if_enabled(
        runtime,
        "to_csv_table",
        enabled,
        Box::new(ToCsvTableFn::new()),
    );
    register_if_enabled(runtime, "from_csv", enabled, Box::new(FromCsvFn::new()));
    register_if_enabled(runtime, "from_tsv", enabled, Box::new(FromTsvFn::new()));
}
