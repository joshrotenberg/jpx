use crate::args::Args;
use anyhow::{Context, Result};
use serde_json::Value;
use std::io::{self, Read};
use std::path::Path;

/// Create a helpful error message for JSON parse failures.
pub(crate) fn json_parse_error(input: &str, err: impl std::fmt::Display) -> anyhow::Error {
    let trimmed = input.trim();
    let preview = crate::util::truncate_str(trimmed, 60);

    // Detect common issues and provide specific suggestions
    let suggestion = if trimmed.is_empty() {
        "Input is empty. Use --null-input (-n) if you don't need input data.".to_string()
    } else if !trimmed.starts_with('{')
        && !trimmed.starts_with('[')
        && !trimmed.starts_with('"')
        && !trimmed.starts_with('-')
        && !trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        && trimmed != "true"
        && trimmed != "false"
        && trimmed != "null"
    {
        // Looks like plain text, not JSON
        format!(
            "Input appears to be plain text, not JSON.\n\n\
             To pass a string value, wrap it in quotes:\n\
               echo '\"{}\"' | jpx ...\n\n\
             Or use --null-input (-n) if you don't need input data.",
            preview.replace('\\', "\\\\").replace('"', "\\\"")
        )
    } else if trimmed.starts_with('\'') || trimmed.contains("': ") || trimmed.contains("':'") {
        // Single quotes - common mistake
        "JSON requires double quotes for strings, not single quotes.\n\
         Replace ' with \" in your input."
            .to_string()
    } else if has_unquoted_keys(trimmed) {
        format!(
            "Parse error: {}\n  Input: {}\n\n\
             Hint: JSON requires double-quoted keys.\n\
             Instead of {{name: \"alice\"}}, use {{\"name\": \"alice\"}}",
            err, preview
        )
    } else if has_trailing_comma(trimmed) {
        format!(
            "Parse error: {}\n  Input: {}\n\n\
             Hint: JSON does not allow trailing commas.\n\
             Remove the comma before the closing bracket or brace.",
            err, preview
        )
    } else {
        // General parse error with context
        format!(
            "Parse error: {}\n  Input: {}\n\n\
             Common JSON issues:\n\
             - Keys must be double-quoted: {{\"key\": \"value\"}}\n\
             - No trailing commas in arrays or objects\n\
             - Strings use double quotes, not single quotes",
            err, preview
        )
    };

    anyhow::anyhow!("Failed to parse JSON input\n\n{}", suggestion)
}

/// Create a helpful error message for expression compilation failures.
pub(crate) fn expression_error(expression: &str, err: impl std::fmt::Display) -> anyhow::Error {
    let err_str = err.to_string();

    // The jmespath crate already renders expression + caret in its Display output,
    // so we just add hints without duplicating that display.
    let mut parts = vec![err_str.clone()];

    // Add hints based on common issues
    if let Some(hint) = expression_hint(expression, &err_str) {
        parts.push(String::new());
        parts.push(format!("Hint: {}", hint));
    }

    anyhow::anyhow!("{}", parts.join("\n"))
}

/// Create a helpful error for file I/O failures.
pub(crate) fn file_read_error(path: &str, err: &io::Error) -> anyhow::Error {
    let display_path = Path::new(path);
    let resolved = std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(display_path))
        .unwrap_or_else(|| display_path.to_path_buf());

    let kind_desc = match err.kind() {
        io::ErrorKind::NotFound => "File not found",
        io::ErrorKind::PermissionDenied => "Permission denied",
        _ => "Failed to read file",
    };

    let mut parts = vec![format!("{}: {}", kind_desc, path)];

    if !display_path.is_absolute() {
        parts.push(format!("  Resolved to: {}", resolved.display()));
    }

    // For "not found" errors, suggest similar files
    if err.kind() == io::ErrorKind::NotFound
        && let Some(parent) = resolved.parent()
    {
        let suggestions = find_similar_files(parent, path);
        if !suggestions.is_empty() {
            parts.push(format!("  Similar files: {}", suggestions.join(", ")));
        }
    }

    anyhow::anyhow!("{}", parts.join("\n"))
}

/// Read input (from file or stdin) and parse as Value
pub(crate) fn read_input_as_value(args: &Args) -> Result<Value> {
    read_input_path_as_value(args, args.file.first().map(String::as_str))
}

/// Read and parse one explicit input path using the CLI's selected input mode.
pub(crate) fn read_input_path_as_value(args: &Args, path: Option<&str>) -> Result<Value> {
    #[cfg(feature = "parquet")]
    if let Some(path) = path.filter(|path| path.ends_with(".parquet") || path.ends_with(".pq")) {
        return jpx::parquet_support::read_parquet_to_json(std::path::Path::new(path))
            .with_context(|| format!("Failed to read Parquet input file: {path}"));
    }

    let input = match path {
        Some(path) => std::fs::read_to_string(path).map_err(|e| file_read_error(path, &e))?,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    if args.raw_input {
        parse_raw_input(&input)
    } else if args.slurp {
        parse_slurp(&input)
    } else {
        serde_json::from_str(&input).map_err(|e| json_parse_error(&input, e))
    }
}

/// Parse raw text input into an array of strings (one per line).
/// Used for `-R -s` (raw-input + slurp) mode.
fn parse_raw_input(input: &str) -> Result<Value> {
    let lines: Vec<Value> = input
        .lines()
        .map(|line| Value::String(line.to_string()))
        .collect();
    Ok(Value::Array(lines))
}

/// Parse multiple JSON values from input into an array
fn parse_slurp(input: &str) -> Result<Value> {
    use serde_json::Deserializer;

    let mut values: Vec<Value> = Vec::new();
    let stream = Deserializer::from_str(input).into_iter::<Value>();

    for result in stream {
        let value = result.context("Failed to parse JSON in slurp mode")?;
        values.push(value);
    }

    Ok(Value::Array(values))
}

/// Read JSON from a file or stdin
pub(crate) fn read_json_from(path: Option<&str>) -> Result<serde_json::Value> {
    // Auto-detect parquet files by extension
    #[cfg(feature = "parquet")]
    if let Some(p) = path.filter(|p| p.ends_with(".parquet") || p.ends_with(".pq")) {
        return jpx::parquet_support::read_parquet_to_json(std::path::Path::new(p))
            .with_context(|| format!("Failed to read parquet file: {}", p));
    }

    let content = if let Some(p) = path {
        std::fs::read_to_string(p).map_err(|e| file_read_error(p, &e))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    };
    serde_json::from_str(&content).context("Failed to parse JSON")
}

// =============================================================================
// Helper functions
// =============================================================================

/// Generate a hint for common expression mistakes.
fn expression_hint(expression: &str, _err: &str) -> Option<String> {
    // Unmatched brackets
    let open_brackets = expression.chars().filter(|c| *c == '[').count();
    let close_brackets = expression.chars().filter(|c| *c == ']').count();
    if open_brackets > close_brackets {
        let missing = open_brackets - close_brackets;
        return Some(format!(
            "Missing {} closing bracket(s) ']'. Check that all '[' have matching ']'.",
            missing
        ));
    }
    if close_brackets > open_brackets {
        let extra = close_brackets - open_brackets;
        return Some(format!(
            "Found {} extra closing bracket(s) ']'. Check for unmatched ']'.",
            extra
        ));
    }

    // Unmatched parens (function calls)
    let open_parens = expression.chars().filter(|c| *c == '(').count();
    let close_parens = expression.chars().filter(|c| *c == ')').count();
    if open_parens > close_parens {
        return Some("Missing closing parenthesis ')' in function call.".into());
    }

    // Common typo: using . instead of | for pipe
    if expression.contains(". ") && expression.contains("sort") {
        return Some("Use '|' (pipe) to chain expressions, not '.'.".into());
    }

    None
}

/// Detect unquoted keys in JSON-like input.
fn has_unquoted_keys(input: &str) -> bool {
    let trimmed = input.trim();
    if !trimmed.starts_with('{') {
        return false;
    }
    // Simple heuristic: look for {word: or , word:
    // but not {"word": or , "word":
    let after_brace = &trimmed[1..];
    let chars: Vec<char> = after_brace.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_alphabetic() {
            // Check if this looks like an unquoted key followed by ':'
            let before = if i > 0 {
                chars.get(i - 1).copied()
            } else {
                None
            };
            let is_start = before.is_none()
                || before == Some(' ')
                || before == Some(',')
                || before == Some('{');
            if !is_start {
                continue;
            }
            // Look ahead for ':'
            let rest: String = chars[i..].iter().take(30).collect();
            if rest.contains(':') {
                let colon_pos = rest.find(':').unwrap();
                let key_candidate = &rest[..colon_pos];
                if key_candidate
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect trailing commas in JSON input.
fn has_trailing_comma(input: &str) -> bool {
    let trimmed = input.trim();
    // Look for ",}" or ",]" patterns (possibly with whitespace between)
    let no_ws: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    no_ws.contains(",}") || no_ws.contains(",]")
}

/// Find files in a directory with similar names to the target.
fn find_similar_files(dir: &Path, target: &str) -> Vec<String> {
    let target_name = Path::new(target)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(target);

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    let mut suggestions = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip directories and hidden files
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) || name_str.starts_with('.') {
            continue;
        }

        // Check various similarity criteria
        let is_similar = name_str.starts_with(target_name.split('.').next().unwrap_or(target_name))
            || target_name.starts_with(name_str.split('.').next().unwrap_or(&name_str))
            || (name_str.contains("json") && target_name.contains("json"));

        if is_similar && name_str != target_name {
            suggestions.push(name_str.to_string());
        }

        if suggestions.len() >= 5 {
            break;
        }
    }

    suggestions
}
