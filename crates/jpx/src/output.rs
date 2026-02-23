use crate::args::ColorMode;
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

/// Recursively sort all object keys alphabetically in a JSON value
pub(crate) fn sort_value_keys(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let sorted: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), sort_value_keys(v)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect();
            Value::Object(sorted)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_value_keys).collect()),
        other => other.clone(),
    }
}

/// Helper to write output to file or stdout
pub(crate) fn write_output(content: &str, output_path: &Option<String>) -> Result<()> {
    if let Some(path) = output_path {
        let mut file = File::create(path)
            .with_context(|| format!("Failed to create output file: {}", path))?;
        writeln!(file, "{}", content)
            .with_context(|| format!("Failed to write to output file: {}", path))?;
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Output JSON with optional coloring and key sorting
pub(crate) fn output_json(
    value: &serde_json::Value,
    compact: bool,
    color_mode: &ColorMode,
    sort_keys: bool,
) -> Result<()> {
    let value = if sort_keys {
        std::borrow::Cow::Owned(sort_value_keys(value))
    } else {
        std::borrow::Cow::Borrowed(value)
    };

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    if compact {
        println!("{}", serde_json::to_string(&*value)?);
    } else if use_color {
        use colored_json::{ColoredFormatter, PrettyFormatter};
        let formatter = ColoredFormatter::new(PrettyFormatter::new());
        println!("{}", formatter.to_colored_json_auto(&*value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&*value)?);
    }
    Ok(())
}

/// Output as YAML
pub(crate) fn output_as_yaml(
    value: &serde_json::Value,
    output_path: &Option<String>,
) -> Result<()> {
    let yaml = serde_yaml::to_string(value).context("Failed to serialize to YAML")?;
    // Remove trailing newline that serde_yaml adds
    let yaml = yaml.trim_end();
    write_output(yaml, output_path)
}

/// Output as TOML
pub(crate) fn output_as_toml(
    value: &serde_json::Value,
    output_path: &Option<String>,
) -> Result<()> {
    // TOML requires a table at the root level
    match value {
        serde_json::Value::Object(_) => {
            let toml_value: toml::Value = serde_json::from_value(value.clone())
                .context("Failed to convert JSON to TOML-compatible structure")?;
            let toml_str =
                toml::to_string_pretty(&toml_value).context("Failed to serialize to TOML")?;
            write_output(toml_str.trim_end(), output_path)
        }
        serde_json::Value::Array(arr) => {
            // Wrap array in a table with "items" key
            let mut wrapper = serde_json::Map::new();
            wrapper.insert("items".to_string(), serde_json::Value::Array(arr.clone()));
            let toml_value: toml::Value =
                serde_json::from_value(serde_json::Value::Object(wrapper))
                    .context("Failed to convert JSON array to TOML-compatible structure")?;
            let toml_str =
                toml::to_string_pretty(&toml_value).context("Failed to serialize to TOML")?;
            write_output(toml_str.trim_end(), output_path)
        }
        _ => Err(anyhow::anyhow!(
            "TOML output requires an object or array at the root level, got {}",
            get_type_name(value)
        )),
    }
}

/// Output one JSON value per line (for arrays)
pub(crate) fn output_as_lines(
    value: &serde_json::Value,
    output_path: &Option<String>,
) -> Result<()> {
    match value {
        serde_json::Value::Array(arr) => {
            let lines: Vec<String> = arr
                .iter()
                .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()))
                .collect();
            write_output(&lines.join("\n"), output_path)
        }
        _ => {
            // For non-arrays, just output the single value
            let line = serde_json::to_string(value)?;
            write_output(&line, output_path)
        }
    }
}

/// Output as CSV
pub(crate) fn output_as_csv(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
    output_as_delimited(value, output_path, b',')
}

/// Output as TSV
pub(crate) fn output_as_tsv(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
    output_as_delimited(value, output_path, b'\t')
}

/// Output as delimited format (CSV or TSV)
fn output_as_delimited(
    value: &serde_json::Value,
    output_path: &Option<String>,
    delimiter: u8,
) -> Result<()> {
    let format_name = if delimiter == b',' { "CSV" } else { "TSV" };

    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return write_output("", output_path);
            }

            // Check if all items are objects
            let all_objects = arr.iter().all(|v| v.is_object());

            if all_objects {
                // Array of objects - flatten and output as table
                output_objects_as_delimited(arr, output_path, delimiter)
            } else {
                // Array of primitives or mixed - output as single column
                let mut wtr = csv::WriterBuilder::new()
                    .delimiter(delimiter)
                    .from_writer(vec![]);

                for item in arr {
                    let cell = value_to_cell(item);
                    wtr.write_record([&cell])?;
                }

                let output = String::from_utf8(wtr.into_inner()?)?;
                write_output(output.trim_end(), output_path)
            }
        }
        serde_json::Value::Object(_) => {
            // Single object - output as two columns (key, value)
            output_objects_as_delimited(std::slice::from_ref(value), output_path, delimiter)
        }
        _ => Err(anyhow::anyhow!(
            "{} output requires an array or object, got {}",
            format_name,
            get_type_name(value)
        )),
    }
}

/// Output array of objects as delimited format with flattening
fn output_objects_as_delimited(
    arr: &[serde_json::Value],
    output_path: &Option<String>,
    delimiter: u8,
) -> Result<()> {
    // Collect all unique keys from all objects, flattening nested structures
    let mut all_keys: Vec<String> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            collect_flattened_keys(obj, "", &mut all_keys, &mut seen_keys);
        }
    }

    // Create CSV writer
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(vec![]);

    // Write header
    wtr.write_record(&all_keys)?;

    // Write rows
    for item in arr {
        let flattened = flatten_object(item);
        let row: Vec<String> = all_keys
            .iter()
            .map(|key| flattened.get(key).map(value_to_cell).unwrap_or_default())
            .collect();
        wtr.write_record(&row)?;
    }

    let output = String::from_utf8(wtr.into_inner()?)?;
    write_output(output.trim_end(), output_path)
}

/// Collect flattened keys from an object using dot notation
pub(crate) fn collect_flattened_keys(
    obj: &serde_json::Map<String, serde_json::Value>,
    prefix: &str,
    keys: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            serde_json::Value::Object(nested) => {
                // Recursively flatten nested objects
                collect_flattened_keys(nested, &full_key, keys, seen);
            }
            _ => {
                // Add leaf key
                if seen.insert(full_key.clone()) {
                    keys.push(full_key);
                }
            }
        }
    }
}

/// Flatten a JSON value into a map with dot-notation keys
pub(crate) fn flatten_object(value: &serde_json::Value) -> BTreeMap<String, serde_json::Value> {
    let mut result = BTreeMap::new();
    flatten_value_recursive(value, "", &mut result);
    result
}

/// Recursively flatten a value
fn flatten_value_recursive(
    value: &serde_json::Value,
    prefix: &str,
    result: &mut BTreeMap<String, serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value_recursive(val, &new_prefix, result);
            }
        }
        _ => {
            if !prefix.is_empty() {
                result.insert(prefix.to_string(), value.clone());
            }
        }
    }
}

/// Convert a JSON value to a CSV cell string
fn value_to_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        // Arrays and objects get JSON-encoded in the cell
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}

// =============================================================================
// Table Output
// =============================================================================

/// Output as a formatted table
pub(crate) fn output_as_table(
    value: &serde_json::Value,
    output_path: &Option<String>,
    style: &str,
    color_mode: &ColorMode,
) -> Result<()> {
    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => output_path.is_none() && atty::is(atty::Stream::Stdout),
    };

    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return write_output("(empty array)", output_path);
            }

            // Check if all items are objects
            let all_objects = arr.iter().all(|v| v.is_object());

            if all_objects {
                output_objects_as_table(arr, output_path, style, use_color)
            } else {
                // Array of primitives or mixed - single column table
                output_primitives_as_table(arr, output_path, style)
            }
        }
        serde_json::Value::Object(_) => {
            // Single object - output as key/value table
            output_objects_as_table(std::slice::from_ref(value), output_path, style, use_color)
        }
        _ => Err(anyhow::anyhow!(
            "Table output requires an array or object, got {}",
            get_type_name(value)
        )),
    }
}

/// Output array of objects as a table
fn output_objects_as_table(
    arr: &[serde_json::Value],
    output_path: &Option<String>,
    style: &str,
    use_color: bool,
) -> Result<()> {
    use tabled::Table;

    // Collect all unique keys from all objects (flattened)
    let mut all_keys: Vec<String> = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for item in arr {
        if let serde_json::Value::Object(obj) = item {
            collect_flattened_keys(obj, "", &mut all_keys, &mut seen_keys);
        }
    }

    if all_keys.is_empty() {
        return write_output("(no columns)", output_path);
    }

    // Build rows
    let mut rows: Vec<Vec<String>> = Vec::new();

    // Header row
    let header: Vec<String> = if use_color {
        all_keys.iter().map(|k| k.bold().to_string()).collect()
    } else {
        all_keys.clone()
    };
    rows.push(header);

    // Data rows
    for item in arr {
        let flattened = flatten_object(item);
        let row: Vec<String> = all_keys
            .iter()
            .map(|key| {
                flattened
                    .get(key)
                    .map(|v| value_to_table_cell(v, use_color))
                    .unwrap_or_default()
            })
            .collect();
        rows.push(row);
    }

    // Create table
    let mut table = Table::from_iter(rows);

    // Apply style
    apply_table_style(&mut table, style);

    write_output(&table.to_string(), output_path)
}

/// Output array of primitives as a single-column table
fn output_primitives_as_table(
    arr: &[serde_json::Value],
    output_path: &Option<String>,
    style: &str,
) -> Result<()> {
    use tabled::Table;

    let mut rows: Vec<Vec<String>> = Vec::new();
    rows.push(vec!["Value".to_string()]);

    for item in arr {
        rows.push(vec![value_to_table_cell(item, false)]);
    }

    let mut table = Table::from_iter(rows);

    apply_table_style(&mut table, style);

    write_output(&table.to_string(), output_path)
}

fn apply_table_style(table: &mut tabled::Table, style: &str) {
    use tabled::settings::Style;

    match style.to_lowercase().as_str() {
        "ascii" => {
            table.with(Style::ascii());
        }
        "markdown" | "md" => {
            table.with(Style::markdown());
        }
        "plain" | "blank" => {
            table.with(Style::blank());
        }
        "rounded" => {
            table.with(Style::rounded());
        }
        "sharp" => {
            table.with(Style::sharp());
        }
        "modern" => {
            table.with(Style::modern());
        }
        _ => {
            table.with(Style::rounded());
        } // unicode/default
    };
}

/// Convert a JSON value to a table cell string
fn value_to_table_cell(value: &serde_json::Value, use_color: bool) -> String {
    match value {
        serde_json::Value::Null => {
            if use_color {
                "null".dimmed().to_string()
            } else {
                "null".to_string()
            }
        }
        serde_json::Value::Bool(b) => {
            if use_color {
                if *b {
                    "true".green().to_string()
                } else {
                    "false".red().to_string()
                }
            } else {
                b.to_string()
            }
        }
        serde_json::Value::Number(n) => {
            if use_color {
                n.to_string().cyan().to_string()
            } else {
                n.to_string()
            }
        }
        serde_json::Value::String(s) => {
            // Truncate long strings
            if s.len() <= 40 {
                s.clone()
            } else {
                format!("{}...", &s[..37])
            }
        }
        serde_json::Value::Array(arr) => {
            if use_color {
                format!("[{} items]", arr.len()).dimmed().to_string()
            } else {
                format!("[{} items]", arr.len())
            }
        }
        serde_json::Value::Object(obj) => {
            if use_color {
                format!("{{{} keys}}", obj.len()).dimmed().to_string()
            } else {
                format!("{{{} keys}}", obj.len())
            }
        }
    }
}

/// Get JSON type name for display
pub(crate) fn get_type_name(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else {
                "number".to_string()
            }
        }
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}
