use crate::args::Args;
use anyhow::{Context, Result};
use jmespath::Variable;
use std::io::{self, Read};

/// Create a helpful error message for JSON parse failures
pub(crate) fn json_parse_error(input: &str, err: impl std::fmt::Display) -> anyhow::Error {
    let trimmed = input.trim();
    let preview = if trimmed.len() > 40 {
        format!("{}...", &trimmed[..40])
    } else {
        trimmed.to_string()
    };

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
    } else {
        // For other JSON parse errors, include the original error
        format!(
            "Parse error: {}\n\n\
             Tip: Ensure your input is valid JSON. Common issues:\n\
             - Strings must use double quotes (\"), not single quotes (')\n\
             - Keys must be quoted: {{\"key\": \"value\"}}\n\
             - No trailing commas in arrays or objects",
            err
        )
    };

    anyhow::anyhow!("Failed to parse JSON input\n\n{}", suggestion)
}

/// Read input (from file or stdin) and parse as Variable
pub(crate) fn read_input_as_variable(args: &Args) -> Result<Variable> {
    let input = match &args.file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    if args.slurp {
        parse_slurp(&input)
    } else {
        Variable::from_json(&input).map_err(|e| json_parse_error(&input, e))
    }
}

/// Parse multiple JSON values from input into an array
fn parse_slurp(input: &str) -> Result<Variable> {
    use serde_json::Deserializer;

    let mut values: Vec<serde_json::Value> = Vec::new();
    let stream = Deserializer::from_str(input).into_iter::<serde_json::Value>();

    for result in stream {
        let value = result.context("Failed to parse JSON in slurp mode")?;
        values.push(value);
    }

    // Convert the collected values directly to Variable
    let array_value = serde_json::Value::Array(values);
    Variable::from_json(&array_value.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to create array: {}", e))
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
        std::fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", p))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    };
    serde_json::from_str(&content).context("Failed to parse JSON")
}
