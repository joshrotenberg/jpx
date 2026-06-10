use crate::args::ColorMode;
use crate::input::read_json_from;
use crate::output::get_type_name;
use anyhow::Result;
use colored::Colorize;

/// Show statistics about JSON data
pub(crate) fn show_stats(file_path: &Option<String>, color_mode: &ColorMode) -> Result<()> {
    let data = read_json_from(file_path.as_deref())?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => crate::util::stdout_is_terminal(),
    };

    // Helper for colored output
    let label = |s: &str| -> String {
        if use_color {
            s.dimmed().to_string()
        } else {
            s.to_string()
        }
    };

    let heading = |s: &str| -> String {
        if use_color {
            s.green().bold().to_string()
        } else {
            s.to_string()
        }
    };

    let highlight = |s: &str| -> String {
        if use_color {
            s.cyan().to_string()
        } else {
            s.to_string()
        }
    };

    let number = |n: usize| -> String {
        let s = format_with_commas(n);
        if use_color { s.yellow().to_string() } else { s }
    };

    println!();
    println!("{}", heading("DATA STATISTICS"));
    println!("{}", "═".repeat(50));
    println!();

    // Basic type info
    let type_name = get_type_name(&data);
    println!("{}  {}", label("Type:"), highlight(&type_name));

    // Size estimation
    let size_bytes = estimate_json_size(&data);
    println!("{}  {}", label("Size:"), format_bytes_human(size_bytes));

    // Depth
    let depth = calculate_depth(&data);
    println!("{} {} levels", label("Depth:"), number(depth));

    println!();

    match &data {
        serde_json::Value::Array(arr) => {
            println!("{} {}", label("Length:"), number(arr.len()));

            if !arr.is_empty() {
                // Analyze item types
                let type_counts = count_types(arr);
                println!();
                println!("{}", heading("Item Types"));
                println!("{}", "─".repeat(30));
                for (type_name, count) in &type_counts {
                    let pct = (*count as f64 / arr.len() as f64) * 100.0;
                    println!(
                        "  {:12} {} ({:.1}%)",
                        highlight(type_name),
                        number(*count),
                        pct
                    );
                }

                // If all objects, analyze keys
                if type_counts.len() == 1 && type_counts.contains_key("object") {
                    let key_stats = analyze_object_keys(arr);
                    if !key_stats.is_empty() {
                        println!();
                        println!("{}", heading("Field Analysis"));
                        println!("{}", "─".repeat(50));
                        for (key, stats) in &key_stats {
                            println!("  {}", highlight(key));
                            println!("    {} {}", label("Type:"), stats.type_name);
                            if stats.null_count > 0 {
                                println!(
                                    "    {} {} ({:.1}%)",
                                    label("Nulls:"),
                                    number(stats.null_count),
                                    (stats.null_count as f64 / arr.len() as f64) * 100.0
                                );
                            }
                            if stats.unique_count > 0 {
                                if stats.unique_count <= 10 && !stats.sample_values.is_empty() {
                                    println!(
                                        "    {} {} unique: {}",
                                        label("Values:"),
                                        number(stats.unique_count),
                                        stats.sample_values.join(", ")
                                    );
                                } else {
                                    println!(
                                        "    {} {} unique",
                                        label("Values:"),
                                        number(stats.unique_count)
                                    );
                                }
                            }
                        }
                    }
                }

                // Show sample
                println!();
                println!("{}", heading("Sample (first item)"));
                println!("{}", "─".repeat(30));
                let sample = &arr[0];
                let sample_str = serde_json::to_string_pretty(sample)?;
                // Truncate if too long
                let lines: Vec<&str> = sample_str.lines().take(8).collect();
                for line in &lines {
                    println!("  {}", line);
                }
                if sample_str.lines().count() > 8 {
                    println!("  {}", "...".dimmed());
                }
            }
        }
        serde_json::Value::Object(obj) => {
            println!("{}  {}", label("Keys:"), number(obj.len()));

            if !obj.is_empty() {
                println!();
                println!("{}", heading("Fields"));
                println!("{}", "─".repeat(50));
                for (key, value) in obj.iter().take(20) {
                    let type_name = get_type_name(value);
                    let preview = get_value_preview(value);
                    println!("  {:20} {} {}", highlight(key), label(&type_name), preview);
                }
                if obj.len() > 20 {
                    println!("  {} and {} more...", "...".dimmed(), obj.len() - 20);
                }
            }
        }
        serde_json::Value::String(s) => {
            println!("{} {} chars", label("Length:"), number(s.chars().count()));
            if s.chars().count() <= 100 {
                println!("{} \"{}\"", label("Value:"), s);
            } else {
                println!(
                    "{} \"{}\"",
                    label("Preview:"),
                    crate::util::truncate_str(s, 100)
                );
            }
        }
        serde_json::Value::Number(n) => {
            println!("{} {}", label("Value:"), n);
        }
        serde_json::Value::Bool(b) => {
            println!("{} {}", label("Value:"), b);
        }
        serde_json::Value::Null => {
            println!("{}", label("(null value)"));
        }
    }

    println!();
    Ok(())
}

pub(crate) fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

pub(crate) fn format_bytes_human(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

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

fn estimate_json_size(value: &serde_json::Value) -> usize {
    // Rough estimation based on JSON serialization
    serde_json::to_string(value).map(|s| s.len()).unwrap_or(0)
}

fn calculate_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(arr) => 1 + arr.iter().map(calculate_depth).max().unwrap_or(0),
        serde_json::Value::Object(obj) => 1 + obj.values().map(calculate_depth).max().unwrap_or(0),
        _ => 1,
    }
}

fn count_types(arr: &[serde_json::Value]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for item in arr {
        let type_name = get_type_name(item);
        *counts.entry(type_name).or_insert(0) += 1;
    }
    counts
}

struct FieldStats {
    type_name: String,
    null_count: usize,
    unique_count: usize,
    sample_values: Vec<String>,
}

fn analyze_object_keys(arr: &[serde_json::Value]) -> Vec<(String, FieldStats)> {
    use std::collections::{HashMap, HashSet};

    // Collect all keys from first few objects
    let mut all_keys: Vec<String> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    for item in arr.iter().take(100) {
        if let serde_json::Value::Object(obj) = item {
            for key in obj.keys() {
                if seen_keys.insert(key.clone()) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    // Analyze each key
    let mut results: Vec<(String, FieldStats)> = Vec::new();

    for key in all_keys.iter().take(10) {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut null_count = 0;
        let mut unique_values: HashSet<String> = HashSet::new();

        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                match obj.get(key) {
                    Some(serde_json::Value::Null) | None => {
                        null_count += 1;
                    }
                    Some(v) => {
                        let type_name = get_type_name(v);
                        *type_counts.entry(type_name).or_insert(0) += 1;

                        // Track unique values for small cardinality fields
                        if unique_values.len() < 100 {
                            let val_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => continue,
                            };
                            unique_values.insert(val_str);
                        }
                    }
                }
            }
        }

        // Determine dominant type
        let type_name = type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t.clone())
            .unwrap_or_else(|| "null".to_string());

        let sample_values: Vec<String> = unique_values.iter().take(5).cloned().collect();

        results.push((
            key.clone(),
            FieldStats {
                type_name,
                null_count,
                unique_count: unique_values.len(),
                sample_values,
            },
        ));
    }

    results
}

fn get_value_preview(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => {
            format!("\"{}\"", crate::util::truncate_str(s, 30))
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{{} keys}}", obj.len()),
    }
}

// =============================================================================
// Path Listing
// =============================================================================

/// Show all paths in the JSON data
pub(crate) fn show_paths(
    file_path: &Option<String>,
    color_mode: &ColorMode,
    show_types: bool,
    show_values: bool,
) -> Result<()> {
    let data = read_json_from(file_path.as_deref())?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => crate::util::stdout_is_terminal(),
    };

    let mut paths_info: Vec<(String, String, Option<String>)> = Vec::new();
    collect_paths(&data, String::new(), &mut paths_info);

    // Calculate max widths for alignment
    let max_path_width = paths_info
        .iter()
        .map(|(p, _, _)| p.len())
        .max()
        .unwrap_or(0);
    let max_type_width = if show_types {
        paths_info
            .iter()
            .map(|(_, t, _)| t.len())
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    for (path, type_name, value) in &paths_info {
        let path_display = if use_color {
            path.cyan().to_string()
        } else {
            path.clone()
        };

        if show_types && show_values {
            let type_display = if use_color {
                format!("{:width$}", type_name, width = max_type_width)
                    .yellow()
                    .to_string()
            } else {
                format!("{:width$}", type_name, width = max_type_width)
            };
            let value_display = value.as_deref().unwrap_or("");
            println!(
                "{:width$}  {}  {}",
                path_display,
                type_display,
                value_display,
                width = max_path_width
            );
        } else if show_types {
            let type_display = if use_color {
                type_name.yellow().to_string()
            } else {
                type_name.clone()
            };
            println!(
                "{:width$}  {}",
                path_display,
                type_display,
                width = max_path_width
            );
        } else if show_values {
            let value_display = value.as_deref().unwrap_or("");
            println!(
                "{:width$}  {}",
                path_display,
                value_display,
                width = max_path_width
            );
        } else {
            println!("{}", path_display);
        }
    }

    Ok(())
}

/// Recursively collect all paths from a JSON value
fn collect_paths(
    value: &serde_json::Value,
    current_path: String,
    paths: &mut Vec<(String, String, Option<String>)>,
) {
    let display_path = if current_path.is_empty() {
        ".".to_string()
    } else {
        current_path.clone()
    };

    match value {
        serde_json::Value::Object(obj) => {
            // Add the object path itself
            let type_name = format!("object{{{}}}", obj.len());
            paths.push((display_path, type_name, None));

            for (key, val) in obj {
                let new_path = if current_path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", current_path, key)
                };
                collect_paths(val, new_path, paths);
            }
        }
        serde_json::Value::Array(arr) => {
            // Add the array path itself
            let type_name = format!("array[{}]", arr.len());
            paths.push((display_path, type_name, None));

            for (i, val) in arr.iter().enumerate() {
                let new_path = if current_path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", current_path, i)
                };
                collect_paths(val, new_path, paths);
            }
        }
        serde_json::Value::String(s) => {
            let preview = format!("\"{}\"", crate::util::truncate_str(s, 40));
            paths.push((display_path, "string".to_string(), Some(preview)));
        }
        serde_json::Value::Number(n) => {
            paths.push((display_path, "number".to_string(), Some(n.to_string())));
        }
        serde_json::Value::Bool(b) => {
            paths.push((display_path, "boolean".to_string(), Some(b.to_string())));
        }
        serde_json::Value::Null => {
            paths.push((display_path, "null".to_string(), Some("null".to_string())));
        }
    }
}
