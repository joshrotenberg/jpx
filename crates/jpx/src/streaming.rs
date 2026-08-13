use crate::args::Args;
use crate::input;
use crate::output::{collect_flattened_keys, flatten_object, value_to_cell};
use anyhow::{Context, Result};
use jpx_engine::Runtime;
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

/// Streaming delimited output state -- tracks headers derived from the first result.
struct DelimitedState {
    headers: Vec<String>,
    header_written: bool,
    delimiter: u8,
}

impl DelimitedState {
    fn new(delimiter: u8) -> Self {
        Self {
            headers: Vec::new(),
            header_written: false,
            delimiter,
        }
    }

    /// Write the header row and/or a data row for the given value.
    /// Returns Ok(true) if the value was written, Ok(false) if skipped (non-object primitive).
    fn write_row(&mut self, value: &Value, writer: &mut impl Write) -> Result<bool> {
        match value {
            Value::Object(obj) => {
                if !self.header_written {
                    // Derive headers from first object
                    let mut seen = std::collections::HashSet::new();
                    collect_flattened_keys(obj, "", &mut self.headers, &mut seen);
                    // Match non-streaming CSV/TSV and table output regardless of
                    // serde_json `preserve_order` feature unification.
                    self.headers.sort_unstable();

                    // Write header row
                    let mut wtr = csv::WriterBuilder::new()
                        .delimiter(self.delimiter)
                        .from_writer(Vec::new());
                    wtr.write_record(&self.headers)?;
                    writer.write_all(&wtr.into_inner()?)?;
                    self.header_written = true;
                }

                // Write data row
                let flattened = flatten_object(value);
                let cells: Vec<String> = self
                    .headers
                    .iter()
                    .map(|key| flattened.get(key).map(value_to_cell).unwrap_or_default())
                    .collect();

                let mut wtr = csv::WriterBuilder::new()
                    .delimiter(self.delimiter)
                    .from_writer(Vec::new());
                wtr.write_record(&cells)?;
                writer.write_all(&wtr.into_inner()?)?;
                Ok(true)
            }
            _ => {
                // Non-object: write as single-column value
                if !self.header_written {
                    let mut wtr = csv::WriterBuilder::new()
                        .delimiter(self.delimiter)
                        .from_writer(Vec::new());
                    wtr.write_record(["value"])?;
                    writer.write_all(&wtr.into_inner()?)?;
                    self.header_written = true;
                    self.headers = vec!["value".to_string()];
                }

                let mut wtr = csv::WriterBuilder::new()
                    .delimiter(self.delimiter)
                    .from_writer(Vec::new());
                wtr.write_record([&value_to_cell(value)])?;
                writer.write_all(&wtr.into_inner()?)?;
                Ok(true)
            }
        }
    }
}

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

    // Set up buffered output, honouring --output if given (previously the
    // streaming path always wrote to stdout and silently ignored --output).
    let sink: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(
            File::create(path).with_context(|| format!("Failed to create output file: {path}"))?,
        ),
        None => Box::new(io::stdout().lock()),
    };
    let mut writer = BufWriter::new(sink);

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

    // Set up delimited output state if CSV/TSV requested
    let mut delimited = if args.csv_output {
        Some(DelimitedState::new(b','))
    } else if args.tsv_output {
        Some(DelimitedState::new(b'\t'))
    } else {
        None
    };

    'lines: for line in input.lines() {
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
                    // Abandon the whole line: skip remaining pipeline stages and
                    // do not emit output for it (previously this `continue`
                    // skipped only the failed stage, then ran later stages on
                    // the un-transformed value and still emitted a result).
                    continue 'lines;
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

        if let Some(ref mut state) = delimited {
            // CSV/TSV streaming output
            state.write_row(&result, &mut writer)?;
        } else {
            // Default JSON streaming output
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
