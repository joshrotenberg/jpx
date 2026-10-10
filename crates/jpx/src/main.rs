mod args;
mod bench;
mod discovery;
mod explain;
mod input;
mod ops;
mod output;
mod repl;
mod stats;
mod streaming;
mod util;

use args::{Args, ConfigDefaults};
use jpx::query_library::{self, LoadResult};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use colored::Colorize;
use jpx_engine::FunctionRegistry;
use serde_json::Value;
use std::io::{self, Write};
use std::time::Instant;

fn main() {
    // Restore the default SIGPIPE disposition. Rust installs SIG_IGN at
    // startup, which turns writes to a closed pipe (e.g. `jpx ... | head`)
    // into EPIPE errors -- the non-streaming path then panics on `println!`
    // and the streaming path exits with a spurious error. Resetting to SIG_DFL
    // lets the OS terminate jpx normally (exit 141) on a closed pipe, the
    // conventional behaviour for a streaming CLI, consistently across both
    // output paths.
    //
    // Safety: a single `libc::signal` call with no preconditions, run before
    // any other work or threads are spawned.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("jpx: {err:#}");
            std::process::exit(1);
        }
    }
}

/// Check whether a JSON value is "truthy" for --exit-status purposes.
/// null and false are falsy; everything else is truthy.
fn is_truthy(value: &Value) -> bool {
    !value.is_null() && value.as_bool() != Some(false)
}

fn run() -> Result<i32> {
    let mut args = Args::parse();

    // Load config and apply defaults (priority: config < env var < CLI flag)
    let config = args::load_config();
    config.apply_defaults(&mut args);
    args::apply_env_defaults(&mut args);

    // Handle shell completions
    if let Some(shell) = args.completions {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(0);
    }

    if !args.per_file && args.file.len() > 1 {
        return Err(anyhow::anyhow!(
            "Multiple input files require --per-file.\n\
             Add --per-file to evaluate each file independently, or pass a single -f/--file."
        ));
    }

    let single_file = args.file.first().cloned();
    let columns = parse_columns(&args)?;

    let starts_repl = args.repl || args.demo.is_some();
    if args.no_history && !starts_repl {
        return Err(anyhow::anyhow!(
            "--no-history only applies to the REPL.\n\
             Add --repl (or --demo <NAME>), or remove --no-history."
        ));
    }
    if starts_repl && args.demo.is_some() && single_file.is_some() {
        return Err(anyhow::anyhow!(
            "REPL startup accepts either --demo or -f/--file, not both.\n\
             Remove one and use .demo or .load later to switch data."
        ));
    }

    // Load engine config (jpx.toml discovery) and create runtime/registry
    let engine_config = jpx_engine::config::EngineConfig::discover().unwrap_or_default();
    let (runtime, registry) = args::create_configured_runtime(&engine_config, args.strict);

    // Handle REPL mode
    if starts_repl {
        repl::run(
            repl::ReplOptions {
                demo_name: args.demo.as_deref(),
                initial_file: single_file.as_deref(),
                color_mode: args.color,
                history_enabled: !args.no_history,
            },
            runtime,
            registry,
        )?;
        return Ok(0);
    }

    if args.list_functions {
        discovery::print_functions(&registry);
        return Ok(0);
    }

    if args.cheatsheet {
        discovery::print_cheatsheet(&registry);
        return Ok(0);
    }

    if let Some(category_name) = &args.list_category {
        discovery::print_category(&registry, category_name)?;
        return Ok(0);
    }

    if let Some(func_name) = &args.describe {
        discovery::describe_function(&registry, func_name)?;
        return Ok(0);
    }

    if let Some(query) = &args.search {
        discovery::search_functions(&registry, query);
        return Ok(0);
    }

    if let Some(func_name) = &args.similar {
        discovery::find_similar_functions(&registry, func_name)?;
        return Ok(0);
    }

    // Debug mode: show diagnostic information
    if args.debug {
        print_debug_info(&args, &registry);
    }

    // Handle --diff: generate JSON Patch from two files
    if let Some(files) = &args.diff {
        ops::diff_files(
            &files[0],
            &files[1],
            args.compact,
            &args.color,
            args.sort_keys,
        )?;
        return Ok(0);
    }

    // Handle --patch: apply JSON Patch to document
    if let Some(patch_file) = &args.patch {
        ops::apply_patch(
            &single_file,
            patch_file,
            args.compact,
            &args.color,
            args.sort_keys,
        )?;
        return Ok(0);
    }

    // Handle --merge: apply JSON Merge Patch to document
    if let Some(merge_file) = &args.merge {
        ops::apply_merge(
            &single_file,
            merge_file,
            args.compact,
            &args.color,
            args.sort_keys,
        )?;
        return Ok(0);
    }

    // Handle --stats: show data statistics
    if args.stats {
        stats::show_stats(&single_file, &args.color)?;
        return Ok(0);
    }

    // Handle --paths: list all paths in JSON
    if args.paths {
        stats::show_paths(&single_file, &args.color, args.types, args.values)?;
        return Ok(0);
    }

    // Handle --list-queries: list available queries in a .jpx file
    if args.list_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--list-queries requires -Q/--query-file"))?;
        ops::list_queries(query_path, &args.color)?;
        return Ok(0);
    }

    // Handle --check: validate queries without running
    if args.check_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--check requires -Q/--query-file"))?;
        ops::check_queries(query_path, &args.color, &runtime)?;
        return Ok(0);
    }

    resolve_positional_input_files(&mut args)?;

    // Get expressions from positional args, -e flags, or file
    let expressions: Vec<String> = if let Some(query_path) = &args.query_file {
        match query_library::load_query_expression(query_path, args.query_name.as_deref(), false)? {
            LoadResult::Expression(expr) => vec![expr],
            LoadResult::List(_) => unreachable!(), // list_mode is false
        }
    } else if !args.expressions.is_empty() {
        std::mem::take(&mut args.expressions)
    } else if !args.positional_expressions.is_empty() {
        std::mem::take(&mut args.positional_expressions)
    } else {
        vec!["@".to_string()]
    };

    // Apply --arg / --argjson variable bindings by wrapping expressions with let bindings
    let expressions = wrap_with_variable_bindings(expressions, &args)?;

    // Handle --explain: parse and show AST without evaluating
    if args.explain {
        for (i, expression) in expressions.iter().enumerate() {
            if expressions.len() > 1 {
                println!("Expression {}: {}", i + 1, expression);
                println!("{}", "=".repeat(expression.len() + 14));
            } else {
                println!("Expression: {}", expression);
                println!("{}", "=".repeat(expression.len() + 12));
            }
            println!();

            let ast = jpx_engine::parse(expression)
                .map_err(|e| input::expression_error(expression, e))?;

            explain::print_ast(&ast, 0);
            println!();
        }
        return Ok(0);
    }

    // Handle --stream or --raw-input (without slurp): process line by line.
    // Per-file raw input is read as one array of lines per file instead.
    if args.stream || (args.raw_input && !args.slurp && !args.per_file) {
        let had_truthy =
            streaming::run_streaming(&expressions, &args, &runtime, columns.as_deref())?;
        if args.exit_status && !had_truthy {
            return Ok(1);
        }
        return Ok(0);
    }

    let start = Instant::now();
    let (result, truthy_override) = if args.per_file {
        let (result, had_truthy) = evaluate_per_file(&runtime, &registry, &expressions, &args)?;
        (result, Some(had_truthy))
    } else {
        let data = if args.null_input {
            Value::Null
        } else {
            input::read_input_as_value(&args)?
        };

        if args.verbose {
            if args.strict {
                eprintln!("Mode: strict (standard JMESPath only)");
            }
            eprintln!("Input: {}", explain::describe_value(&data));
            if expressions.len() > 1 {
                eprintln!("Expressions: {} (chained)", expressions.len());
            }
            eprintln!();
        }

        if let Some(iterations) = args.bench {
            bench::run_benchmark(
                &runtime,
                &expressions,
                &data,
                iterations,
                args.warmup,
                &args.color,
            )?;
            return Ok(0);
        }

        (
            evaluate_expressions(&runtime, &registry, &expressions, data, &args)?,
            None,
        )
    };

    let total_elapsed = start.elapsed();
    if args.verbose {
        eprintln!("Total time: {:.3}ms", total_elapsed.as_secs_f64() * 1000.0);
        eprintln!();
    }

    // Determine exit code for --exit-status before output
    let is_success = truthy_override.unwrap_or_else(|| is_truthy(&result));
    let exit_code = if args.exit_status && !is_success {
        1
    } else {
        0
    };

    // Output result
    if result.is_null() {
        // Don't print anything for null results (like jq)
        return Ok(exit_code);
    }

    // Join output mode: raw output without trailing newlines
    if args.join_output {
        if let Some(s) = result.as_str() {
            print!("{}", s);
        } else {
            print!("{}", serde_json::to_string(&result)?);
        }
        return Ok(exit_code);
    }

    #[allow(clippy::collapsible_if)]
    if args.raw {
        if let Some(s) = result.as_str() {
            println!("{}", s);
            return Ok(exit_code);
        }
    }

    // Result is already a serde_json::Value
    let json_value = if args.sort_keys {
        output::sort_value_keys(&result)
    } else {
        result
    };

    // Handle alternative output formats
    if args.yaml {
        output::output_as_yaml(&json_value, &args.output)?;
        return Ok(exit_code);
    }
    if args.toml_output {
        output::output_as_toml(&json_value, &args.output)?;
        return Ok(exit_code);
    }
    if args.csv_output {
        output::output_as_csv(&json_value, &args.output, columns.as_deref())?;
        return Ok(exit_code);
    }
    if args.tsv_output {
        output::output_as_tsv(&json_value, &args.output, columns.as_deref())?;
        return Ok(exit_code);
    }
    if args.lines_output {
        output::output_as_lines(&json_value, &args.output)?;
        return Ok(exit_code);
    }
    #[cfg(feature = "parquet")]
    if args.parquet_output {
        let output_path = args
            .output
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--parquet requires --output"))?;
        jpx::parquet_support::write_json_to_parquet(&json_value, std::path::Path::new(output_path))
            .context("Failed to write parquet file")?;
        return Ok(exit_code);
    }
    if args.table {
        output::output_as_table(
            &json_value,
            &args.output,
            args.table_style.as_str(),
            &args.color,
            columns.as_deref(),
        )?;
        return Ok(exit_code);
    }

    // When writing to file, don't colorize unless explicitly requested
    let should_colorize = crate::util::should_colorize(
        &args.color,
        args.output.is_none() && crate::util::stdout_is_terminal(),
    );

    // Determine indent string
    let indent_str = if args.tab {
        "\t".to_string()
    } else if let Some(n) = args.indent {
        " ".repeat(n as usize)
    } else {
        "  ".to_string() // default: 2 spaces
    };

    let output_str = if should_colorize && !args.compact {
        // Colored pretty output with custom color scheme
        use colored_json::{ColoredFormatter, PrettyFormatter, Style, Styler};

        let styler = Styler {
            key: Style::new().blue().bold(),
            string_value: Style::new().green(),
            integer_value: Style::new().cyan(),
            float_value: Style::new().cyan(),
            bool_value: Style::new().yellow(),
            nil_value: Style::new().red().dim(),
            ..Default::default()
        };

        let pf = PrettyFormatter::with_indent(indent_str.as_bytes());
        let formatter = ColoredFormatter::with_styler(pf, styler);
        let mut writer = Vec::new();
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        use serde::Serialize;
        json_value.serialize(&mut serializer)?;
        String::from_utf8(writer)?
    } else if args.compact {
        serde_json::to_string(&json_value)?
    } else {
        let mut writer = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_str.as_bytes());
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        use serde::Serialize;
        json_value.serialize(&mut serializer)?;
        String::from_utf8(writer)?
    };

    // Write output to file or stdout
    if let Some(output_path) = &args.output {
        let mut file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path))?;
        writeln!(file, "{}", output_str)
            .with_context(|| format!("Failed to write to output file: {}", output_path))?;
    } else {
        println!("{}", output_str);
    }

    Ok(exit_code)
}

fn parse_columns(args: &Args) -> Result<Option<Vec<String>>> {
    let Some(spec) = args.columns.as_deref() else {
        return Ok(None);
    };

    if !args.table && !args.csv_output && !args.tsv_output {
        return Err(anyhow::anyhow!(
            "--columns requires --table, --csv, or --tsv.\n\
             Add a tabular output flag, or remove --columns."
        ));
    }

    let mut columns = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in spec.split(',') {
        let column = raw.trim();
        if column.is_empty() {
            return Err(anyhow::anyhow!(
                "--columns contains an empty column name.\n\
                 Use a comma-separated list such as '--columns name,age'."
            ));
        }
        if !seen.insert(column) {
            return Err(anyhow::anyhow!(
                "Duplicate column '{column}' in --columns.\n\
                 List each column once in the desired output order."
            ));
        }
        columns.push(column.to_string());
    }

    Ok(Some(columns))
}

fn evaluate_expressions(
    runtime: &jpx_engine::Runtime,
    registry: &FunctionRegistry,
    expressions: &[String],
    mut result: Value,
    args: &Args,
) -> Result<Value> {
    for (i, expression) in expressions.iter().enumerate() {
        if args.verbose {
            eprintln!("[{}] Expression: {}", i + 1, expression);
        }

        let expr = runtime
            .compile(expression)
            .map_err(|e| input::expression_error(expression, e))?;

        if args.strict && jpx_engine::has_let_nodes(expr.as_ast()) {
            return Err(anyhow::anyhow!(
                "Let expressions are not available in strict mode (standard JMESPath only).\n\
                 Remove --strict or unset JPX_STRICT to use let expressions."
            ));
        }

        let step_start = Instant::now();
        result = match expr.search(&result) {
            Ok(result) => result,
            Err(error) => {
                let error_message = error.to_string();
                if args.strict && error_message.contains("undefined function") {
                    return Err(anyhow::anyhow!(
                        "{}\n\nHint: You are using --strict mode which only allows standard JMESPath functions.\nRemove --strict or unset JPX_STRICT to use extension functions.",
                        error_message
                    ));
                }
                let suggestion =
                    discovery::suggest_for_unknown_function(registry, &error_message, 3)
                        .unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Failed to evaluate expression: {}{}",
                    error,
                    suggestion
                ));
            }
        };

        if args.verbose {
            eprintln!("[{}] Result: {}", i + 1, explain::describe_value(&result));
            eprintln!(
                "[{}] Time: {:.3}ms",
                i + 1,
                step_start.elapsed().as_secs_f64() * 1000.0
            );
            eprintln!();
        }
    }

    Ok(result)
}

/// Evaluate the expression pipeline independently for every input file.
/// Results are collected into one outer array so `--lines` emits exactly one
/// JSON value per file, including nulls and nested-array results.
fn evaluate_per_file(
    runtime: &jpx_engine::Runtime,
    registry: &FunctionRegistry,
    expressions: &[String],
    args: &Args,
) -> Result<(Value, bool)> {
    let uses_file_binding = expressions
        .iter()
        .any(|expression| expression_uses_file_binding(expression));
    if args.strict && uses_file_binding {
        return Err(anyhow::anyhow!(
            "The $file binding is not available in --strict mode.\n\
             Remove --strict or unset JPX_STRICT, or remove $file from the expression."
        ));
    }

    let mut results = Vec::with_capacity(args.file.len());
    let mut had_truthy = false;

    for path in &args.file {
        let data = input::read_input_path_as_value(args, Some(path)).with_context(|| {
            format!(
                "Failed to process input file '{path}'. Check the path and JSON format; use -s/--slurp for JSONL."
            )
        })?;

        if args.verbose {
            eprintln!("File: {path}");
            eprintln!("Input: {}", explain::describe_value(&data));
        }

        let file_expressions;
        let active_expressions = if uses_file_binding {
            file_expressions = wrap_with_file_binding(expressions, path)?;
            &file_expressions
        } else {
            expressions
        };

        let result = evaluate_expressions(runtime, registry, active_expressions, data, args)
            .with_context(|| {
                format!(
                    "Evaluation failed for input file '{path}'. Fix the query or this input and retry."
                )
            })?;
        had_truthy |= is_truthy(&result);
        results.push(result);
    }

    Ok((Value::Array(results), had_truthy))
}

fn expression_uses_file_binding(expression: &str) -> bool {
    let mut chars = expression.char_indices().peekable();

    while let Some((offset, current)) = chars.next() {
        match current {
            '\'' | '"' | '`' => {
                let delimiter = current;
                while let Some((_, quoted)) = chars.next() {
                    if quoted == '\\' {
                        // All three JMESPath quoted forms treat the next
                        // character as part of the quoted token.
                        chars.next();
                    } else if quoted == delimiter {
                        break;
                    }
                }
            }
            '$' if expression[offset..].starts_with("$file") => {
                let next = expression[offset + "$file".len()..].chars().next();
                if next.is_none_or(|next| !next.is_ascii_alphanumeric() && next != '_') {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn wrap_with_file_binding(expressions: &[String], path: &str) -> Result<Vec<String>> {
    let literal = jmespath_json_literal(&Value::String(path.to_string()))?;
    Ok(expressions
        .iter()
        .map(|expression| {
            if expression_uses_file_binding(expression) {
                format!("let $file = `{literal}` in {expression}")
            } else {
                expression.clone()
            }
        })
        .collect())
}

/// Encode a JSON value for a JMESPath backtick literal. A literal backtick is
/// represented as a JSON Unicode escape so it cannot terminate the wrapper.
fn jmespath_json_literal(value: &Value) -> Result<String> {
    Ok(serde_json::to_string(value)?.replace('`', "\\u0060"))
}

/// Separate positional expressions from positional input paths.
///
/// Outside `--per-file`, this preserves the jq-style single trailing-file
/// heuristic. In per-file mode, `-Q` makes every positional unambiguous; when
/// an expression is positional, the first existing file starts the file list.
fn resolve_positional_input_files(args: &mut Args) -> Result<()> {
    if args.per_file {
        if args.query_file.is_some() {
            args.file.append(&mut args.positional_expressions);
        } else if let Some(first_file) = args
            .positional_expressions
            .iter()
            .position(|value| std::path::Path::new(value).is_file())
        {
            let positional_files = args.positional_expressions.split_off(first_file);
            args.file.extend(positional_files);
        }

        if args.file.is_empty() {
            return Err(anyhow::anyhow!(
                "--per-file requires at least one input file.\n\
                 Pass paths with repeated -f/--file, or as trailing paths after the expression."
            ));
        }

        let reserves_file = args
            .arg
            .chunks_exact(2)
            .chain(args.argjson.chunks_exact(2))
            .any(|pair| pair[0] == "file");
        if reserves_file {
            return Err(anyhow::anyhow!(
                "The $file variable is reserved by --per-file.\n\
                 Remove '--arg file ...' or '--argjson file ...'; use $file for the current input path."
            ));
        }

        return Ok(());
    }

    // With -Q the expression already comes from the query file, so a single
    // positional value may be the input path.
    if args.query_file.is_some() {
        if args.file.is_empty() && !args.null_input && args.positional_expressions.len() == 1 {
            let last = args.positional_expressions.last().expect("length checked");
            if std::path::Path::new(last).is_file() {
                let file_arg = args.positional_expressions.pop().expect("length checked");
                args.file.push(file_arg);
            }
        }
        if !args.positional_expressions.is_empty() {
            return Err(anyhow::anyhow!(
                "With -Q/--query-file, a positional argument must be an existing input file.\n\
                 Check the path or pass it explicitly with -f/--file."
            ));
        }
    } else if args.file.is_empty() && !args.null_input && args.positional_expressions.len() > 1 {
        let last = args.positional_expressions.last().expect("length checked");
        if std::path::Path::new(last).is_file() {
            let file_arg = args.positional_expressions.pop().expect("length checked");
            args.file.push(file_arg);
        }
    }

    Ok(())
}

/// Parse --arg and --argjson flags and wrap each expression with let bindings.
/// Returns the expressions unchanged if no variables are defined.
fn wrap_with_variable_bindings(expressions: Vec<String>, args: &Args) -> Result<Vec<String>> {
    if args.arg.is_empty() && args.argjson.is_empty() {
        return Ok(expressions);
    }

    // Collect bindings as (name, json_literal) pairs
    let mut bindings: Vec<(String, String)> = Vec::new();

    // --arg pairs: treat values as strings
    for pair in args.arg.chunks(2) {
        let name = &pair[0];
        let value = &pair[1];
        // Encode as JSON string and wrap in backticks for JMESPath literal
        let json_literal = jmespath_json_literal(&Value::String(value.clone()))?;
        bindings.push((name.clone(), json_literal));
    }

    // --argjson pairs: values are already JSON
    for pair in args.argjson.chunks(2) {
        let name = &pair[0];
        let json_value = &pair[1];
        // Validate it's valid JSON
        serde_json::from_str::<serde_json::Value>(json_value)
            .with_context(|| format!("Invalid JSON for --argjson {}: {}", name, json_value))?;
        bindings.push((name.clone(), json_value.replace('`', "\\u0060")));
    }

    // Wrap each expression with let bindings
    Ok(expressions
        .into_iter()
        .map(|expr| {
            let mut wrapped = String::new();
            for (name, json_lit) in &bindings {
                wrapped.push_str(&format!("let ${} = `{}` in ", name, json_lit));
            }
            wrapped.push_str(&expr);
            wrapped
        })
        .collect())
}

fn print_debug_info(args: &Args, registry: &FunctionRegistry) {
    eprintln!("{}", "=== jpx debug info ===".cyan().bold());
    eprintln!("{}: {}", "Version".dimmed(), env!("CARGO_PKG_VERSION"));
    eprintln!();

    // Show environment variables
    eprintln!("{}:", "Environment".cyan());
    for (var, desc) in [
        ("JPX_VERBOSE", "verbose mode"),
        ("JPX_QUIET", "quiet mode"),
        ("JPX_STRICT", "strict mode"),
        ("JPX_RAW", "raw output"),
        ("JPX_COMPACT", "compact output"),
    ] {
        let value = std::env::var(var).unwrap_or_else(|_| "(not set)".to_string());
        eprintln!("  {} = {} ({})", var, value.yellow(), desc);
    }
    eprintln!();

    // Show effective settings
    eprintln!("{}:", "Effective settings".cyan());
    eprintln!("  verbose: {}", args.verbose);
    eprintln!("  quiet: {}", args.quiet);
    eprintln!("  strict: {}", args.strict);
    eprintln!("  raw: {}", args.raw);
    eprintln!("  compact: {}", args.compact);
    eprintln!();

    // Show input source
    eprintln!("{}:", "Input".cyan());
    match args.file.as_slice() {
        [] if args.null_input => eprintln!("  source: null (--null-input)"),
        [] => eprintln!("  source: stdin"),
        [path] => eprintln!("  source: file ({})", path),
        paths => eprintln!("  source: {} files (--per-file)", paths.len()),
    }
    if args.slurp {
        eprintln!("  mode: slurp (multiple JSON values)");
    }
    eprintln!();

    // Show expressions
    let debug_expressions: Vec<&String> = args
        .expressions
        .iter()
        .chain(args.positional_expressions.iter())
        .collect();
    eprintln!("{}:", "Expressions".cyan());
    if debug_expressions.is_empty() {
        eprintln!("  (none provided - will use '@')");
    } else {
        for (i, expr) in debug_expressions.iter().enumerate() {
            eprintln!("  [{}] {}", i + 1, expr.yellow());
        }
    }
    eprintln!();

    // Show registered functions count
    eprintln!("{}:", "Functions".cyan());
    eprintln!("  registered: {}", registry.len());
    if args.strict {
        eprintln!("  mode: strict (standard JMESPath only)");
    } else {
        eprintln!("  mode: extended (all functions available)");
    }
    eprintln!("{}", "======================".cyan().bold());
    eprintln!();
}
