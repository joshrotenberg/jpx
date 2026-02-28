use crate::args::Args;
use crate::input;
use anyhow::{Context, Result};
use jpx_engine::Runtime;
use serde_json::Value;
use std::io::{self, BufRead, BufWriter, Write};

/// Run streaming mode - process input line by line (NDJSON/JSON Lines).
/// Returns whether any truthy (non-null, non-false) result was produced.
pub(crate) fn run_streaming(
    expressions: &[String],
    args: &Args,
    runtime: &Runtime,
) -> Result<bool> {
    // Set up input reader
    let input: Box<dyn BufRead> = match &args.file {
        Some(path) => {
            let file = std::fs::File::open(path).map_err(|e| input::file_read_error(path, &e))?;
            Box::new(std::io::BufReader::new(file))
        }
        None => Box::new(std::io::BufReader::new(io::stdin())),
    };

    // Set up buffered output
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    // Compile expressions once
    let compiled: Vec<_> = expressions
        .iter()
        .map(|expr| {
            runtime
                .compile(expr)
                .map_err(|e| input::expression_error(expr, e))
        })
        .collect::<Result<Vec<_>>>()?;

    let quiet = args.quiet;
    let raw = args.raw;
    let mut line_count = 0u64;
    let mut had_truthy = false;

    for line in input.lines() {
        let line = line.context("Failed to read line")?;
        let trimmed = line.trim();
        line_count += 1;

        if trimmed.is_empty() && !args.raw_input {
            continue;
        }

        let data: Value = if args.raw_input {
            Value::String(trimmed.to_string())
        } else {
            match serde_json::from_str(trimmed) {
                Ok(d) => d,
                Err(e) => {
                    if !quiet {
                        eprintln!("jpx: Failed to parse JSON: {}", e);
                    }
                    continue;
                }
            }
        };

        let mut result: Value = data;
        for expr in &compiled {
            result = match expr.search(&result) {
                Ok(r) => r,
                Err(e) => {
                    if !quiet {
                        eprintln!("jpx: Expression error: {}", e);
                    }
                    continue;
                }
            };
        }

        let result = if args.sort_keys {
            crate::output::sort_value_keys(&result)
        } else {
            result
        };

        if !result.is_null() && result.as_bool() != Some(false) {
            had_truthy = true;
        }

        if result.is_null() {
            continue;
        }

        let output = if args.join_output || raw {
            if let Some(s) = result.as_str() {
                s.to_string()
            } else {
                serde_json::to_string(&result)?
            }
        } else {
            serde_json::to_string(&result)?
        };

        if args.join_output {
            write!(writer, "{}", output)?;
        } else {
            writeln!(writer, "{}", output)?;
        }
        if args.unbuffered {
            writer.flush()?;
        }
    }

    if args.verbose {
        eprintln!("Processed {} lines", line_count);
    }

    writer.flush()?;
    Ok(had_truthy)
}
