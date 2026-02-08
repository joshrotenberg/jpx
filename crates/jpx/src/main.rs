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

use args::{Args, ColorMode, ConfigDefaults};
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
    if let Err(err) = run() {
        eprintln!("jpx: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
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
        return Ok(());
    }

    // Load engine config (jpx.toml discovery) and create runtime/registry
    let engine_config = jpx_engine::config::EngineConfig::discover().unwrap_or_default();
    let (runtime, registry) = args::create_configured_runtime(&engine_config, args.strict);

    // Handle REPL mode
    if args.repl || args.demo.is_some() {
        return repl::run(args.demo.as_deref(), runtime, registry);
    }

    if args.list_functions {
        discovery::print_functions(&registry);
        return Ok(());
    }

    if let Some(category_name) = &args.list_category {
        discovery::print_category(&registry, category_name)?;
        return Ok(());
    }

    if let Some(func_name) = &args.describe {
        discovery::describe_function(&registry, func_name)?;
        return Ok(());
    }

    if let Some(query) = &args.search {
        discovery::search_functions(&registry, query);
        return Ok(());
    }

    if let Some(func_name) = &args.similar {
        discovery::find_similar_functions(&registry, func_name)?;
        return Ok(());
    }

    // Debug mode: show diagnostic information
    if args.debug {
        print_debug_info(&args, &registry);
    }

    // Handle --diff: generate JSON Patch from two files
    if let Some(files) = &args.diff {
        return ops::diff_files(&files[0], &files[1], args.compact, &args.color);
    }

    // Handle --patch: apply JSON Patch to document
    if let Some(patch_file) = &args.patch {
        return ops::apply_patch(&args.file, patch_file, args.compact, &args.color);
    }

    // Handle --merge: apply JSON Merge Patch to document
    if let Some(merge_file) = &args.merge {
        return ops::apply_merge(&args.file, merge_file, args.compact, &args.color);
    }

    // Handle --stats: show data statistics
    if args.stats {
        return stats::show_stats(&args.file, &args.color);
    }

    // Handle --paths: list all paths in JSON
    if args.paths {
        return stats::show_paths(&args.file, &args.color, args.types, args.values);
    }

    // Handle --list-queries: list available queries in a .jpx file
    if args.list_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--list-queries requires -Q/--query-file"))?;
        return ops::list_queries(query_path, &args.color);
    }

    // Handle --check: validate queries without running
    if args.check_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--check requires -Q/--query-file"))?;
        return ops::check_queries(query_path, &args.color, &runtime);
    }

    // jq-style: if the last positional arg is an existing file and no -f was given, use it as input
    if args.file.is_none() && !args.null_input && args.positional_expressions.len() > 1 {
        let last = args.positional_expressions.last().unwrap();
        if std::path::Path::new(last).is_file() {
            let file_arg = args.positional_expressions.pop().unwrap();
            args.file = Some(file_arg);
        }
    }

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
        return Err(anyhow::anyhow!(
            "Expression required. Use --help for usage."
        ));
    };

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
        return Ok(());
    }

    // Handle --stream: process input line by line (NDJSON/JSON Lines)
    if args.stream {
        return streaming::run_streaming(&expressions, &args, &runtime);
    }

    // Get input data
    let data: Value = if args.null_input {
        // Null input mode - don't read anything
        Value::Null
    } else {
        // Check for parquet input
        #[cfg(feature = "parquet")]
        if let Some(path) = &args.file {
            if path.ends_with(".parquet") || path.ends_with(".pq") {
                jpx::parquet_support::read_parquet_to_json(std::path::Path::new(path))
                    .with_context(|| format!("Failed to read parquet file: {}", path))?
            } else {
                input::read_input_as_value(&args)?
            }
        } else {
            input::read_input_as_value(&args)?
        }

        #[cfg(not(feature = "parquet"))]
        input::read_input_as_value(&args)?
    };

    // Verbose mode: show input info
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

    // Handle --bench: benchmark expression performance
    if let Some(iterations) = args.bench {
        return bench::run_benchmark(
            &runtime,
            &expressions,
            &data,
            iterations,
            args.warmup,
            &args.color,
        );
    }

    // Compile and execute expression(s)
    let start = Instant::now();
    let mut result: Value = data.clone();

    for (i, expression) in expressions.iter().enumerate() {
        if args.verbose {
            eprintln!("[{}] Expression: {}", i + 1, expression);
        }

        let expr = runtime
            .compile(expression)
            .map_err(|e| input::expression_error(expression, e))?;

        let step_start = Instant::now();
        result = match expr.search(&result) {
            Ok(r) => r,
            Err(e) => {
                let err_msg = e.to_string();
                if args.strict && err_msg.contains("undefined function") {
                    return Err(anyhow::anyhow!(
                        "{}\n\nHint: You are using --strict mode which only allows standard JMESPath functions.\nRemove --strict or unset JPX_STRICT to use extension functions.",
                        err_msg
                    ));
                }
                return Err(anyhow::anyhow!("Failed to evaluate expression: {}", e));
            }
        };
        let step_elapsed = step_start.elapsed();

        if args.verbose {
            eprintln!("[{}] Result: {}", i + 1, explain::describe_value(&result));
            eprintln!(
                "[{}] Time: {:.3}ms",
                i + 1,
                step_elapsed.as_secs_f64() * 1000.0
            );
            eprintln!();
        }
    }

    let total_elapsed = start.elapsed();
    if args.verbose {
        eprintln!("Total time: {:.3}ms", total_elapsed.as_secs_f64() * 1000.0);
        eprintln!();
    }

    // Output result
    if result.is_null() {
        // Don't print anything for null results (like jq)
        return Ok(());
    }

    #[allow(clippy::collapsible_if)]
    if args.raw {
        if let Some(s) = result.as_str() {
            println!("{}", s);
            return Ok(());
        }
    }

    // Result is already a serde_json::Value
    let json_value = result;

    // Handle alternative output formats
    if args.yaml {
        return output::output_as_yaml(&json_value, &args.output);
    }
    if args.toml_output {
        return output::output_as_toml(&json_value, &args.output);
    }
    if args.csv_output {
        return output::output_as_csv(&json_value, &args.output);
    }
    if args.tsv_output {
        return output::output_as_tsv(&json_value, &args.output);
    }
    if args.lines_output {
        return output::output_as_lines(&json_value, &args.output);
    }
    #[cfg(feature = "parquet")]
    if args.parquet_output {
        let output_path = args.output.as_ref().expect("--parquet requires --output");
        return jpx::parquet_support::write_json_to_parquet(
            &json_value,
            std::path::Path::new(output_path),
        )
        .context("Failed to write parquet file");
    }
    if args.table {
        return output::output_as_table(&json_value, &args.output, &args.table_style, &args.color);
    }

    // When writing to file, don't colorize unless explicitly requested
    let should_colorize = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => args.output.is_none() && atty::is(atty::Stream::Stdout),
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

        let formatter = ColoredFormatter::with_styler(PrettyFormatter::new(), styler);
        let mut writer = Vec::new();
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        use serde::Serialize;
        json_value.serialize(&mut serializer)?;
        String::from_utf8(writer)?
    } else if args.compact {
        serde_json::to_string(&json_value)?
    } else {
        serde_json::to_string_pretty(&json_value)?
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

    Ok(())
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
    match &args.file {
        Some(path) => eprintln!("  source: file ({})", path),
        None if args.null_input => eprintln!("  source: null (--null-input)"),
        None => eprintln!("  source: stdin"),
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
