mod repl;

use jpx::query_library::{self, LoadResult};

use anyhow::{Context, Result};
use clap::{ArgAction, CommandFactory, Parser, ValueEnum, builder::styling};
use clap_complete::{Shell, generate};
use colored::Colorize;
use jmespath::ast::Ast;
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionInfo, FunctionRegistry};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

// Cargo-style help coloring
const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

// =============================================================================
// Configuration File Support
// =============================================================================

/// Configuration file structure
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Config {
    /// Output settings
    verbose: Option<bool>,
    quiet: Option<bool>,
    strict: Option<bool>,
    raw: Option<bool>,
    compact: Option<bool>,
    /// Color mode (auto, always, never)
    color: Option<String>,
}

/// Get the path to the config file
fn get_config_path() -> Option<PathBuf> {
    // Check JPX_CONFIG environment variable first
    if let Ok(path) = std::env::var("JPX_CONFIG") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    // Check ~/.config/jpx/config.toml (XDG style)
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("jpx").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Check ~/.jpxrc (traditional style)
    if let Some(home_dir) = dirs::home_dir() {
        let path = home_dir.join(".jpxrc");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Create a JMESPath runtime with all extension functions registered.
fn create_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);
    runtime
}

/// Load configuration from file
fn load_config() -> Config {
    let Some(path) = get_config_path() else {
        return Config::default();
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "{}: Failed to parse config file {}: {}",
                "warning".yellow(),
                path.display(),
                e
            );
            Config::default()
        }
    }
}

/// Apply config file defaults to args (lowest priority)
fn apply_config_defaults(args: &mut Args, config: &Config) {
    // Only apply config values if CLI flag wasn't explicitly set
    // Config has lowest priority: config < env var < CLI flag
    if !args.verbose && config.verbose == Some(true) {
        args.verbose = true;
    }
    if !args.quiet && config.quiet == Some(true) {
        args.quiet = true;
    }
    if !args.strict && config.strict == Some(true) {
        args.strict = true;
    }
    if !args.raw && config.raw == Some(true) {
        args.raw = true;
    }
    if !args.compact && config.compact == Some(true) {
        args.compact = true;
    }
}

/// Check if an environment variable is set to a "truthy" value
fn env_is_true(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Apply environment variable defaults to args
/// CLI args take precedence over env vars (if CLI flag is set, don't override)
fn apply_env_defaults(args: &mut Args) {
    // Only apply env var if CLI flag wasn't explicitly set
    // Since clap sets bool flags to false by default, we check env vars
    // and set to true if the env var is truthy
    if !args.verbose && env_is_true("JPX_VERBOSE") {
        args.verbose = true;
    }
    if !args.quiet && env_is_true("JPX_QUIET") {
        args.quiet = true;
    }
    if !args.strict && env_is_true("JPX_STRICT") {
        args.strict = true;
    }
    if !args.raw && env_is_true("JPX_RAW") {
        args.raw = true;
    }
    if !args.compact && env_is_true("JPX_COMPACT") {
        args.compact = true;
    }
}

/// Color output mode
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ColorMode {
    /// Automatically detect if output is a terminal
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// JMESPath CLI with extended functions
///
/// A command-line tool for querying JSON data using JMESPath expressions
/// with 150+ additional functions beyond the standard specification.
#[derive(Parser, Debug)]
#[command(name = "jpx")]
#[command(version, about, long_about = None)]
#[command(styles = STYLES)]
#[command(disable_help_flag = true)]
#[command(after_help = "Use --help for examples and detailed documentation")]
#[command(after_long_help = concat!(
    "GETTING STARTED:\n",
    "  JMESPath basics (work everywhere):\n",
    "    .field          Access object field       {\"name\": \"jo\"} | jpx 'name' -> \"jo\"\n",
    "    [*]             Iterate arrays            [{\"a\":1},{\"a\":2}] | jpx '[*].a' -> [1,2]\n",
    "    [?expr]         Filter arrays             [1,2,3] | jpx '[?@ > `1`]' -> [2,3]\n",
    "\n",
    "  Essential standard functions:\n",
    "    length(@)       Array/string/object size\n",
    "    sort(@)         Sort arrays\n",
    "    keys(@)         Object keys\n",
    "    values(@)       Object values\n",
    "    contains(@, x)  Check if array/string contains value\n",
    "    join(', ', @)   Join array into string\n",
    "\n",
    "  Popular extensions:\n",
    "    sum(@), avg(@), min(@), max(@)     Math on arrays\n",
    "    unique(@), flatten(@), first(@)   Array manipulation\n",
    "    now(), format_date(ts, fmt)       Date/time\n",
    "    split(s, delim), upper(s)         String processing\n",
    "\n",
    "EXAMPLES:\n",
    "  Basic query:\n",
    "    echo '{\"name\": \"alice\"}' | jpx 'name'\n",
    "\n",
    "  Using extension functions:\n",
    "    echo '[1, 2, 3]' | jpx 'sum(@)'\n",
    "    echo '{\"ts\": \"2024-01-15\"}' | jpx 'format_date(ts, \"%B %d, %Y\")'\n",
    "    jpx -n 'now()'\n",
    "\n",
    "  Pipeline (multiple expressions chained):\n",
    "    cat data.json | jpx 'items[*].name' 'sort(@)' 'first(@)'\n",
    "    cat data.json | jpx -e 'items[*].name' -e 'sort(@)'\n",
    "\n",
    "  Output formats:\n",
    "    jpx -t 'users[*]' < data.json       # Table format\n",
    "    jpx --csv 'users[*]' < data.json    # CSV output\n",
    "    jpx -y 'config' < data.json         # YAML output\n",
    "\n",
    "  Discovery:\n",
    "    jpx --list-functions                # List all 400+ functions\n",
    "    jpx --search date                   # Find date-related functions\n",
    "    jpx --describe format_date          # Function documentation\n",
    "\n",
    "ENVIRONMENT VARIABLES:\n",
    "  JPX_VERBOSE=1     Enable verbose mode\n",
    "  JPX_QUIET=1       Enable quiet mode\n",
    "  JPX_STRICT=1      Disable extension functions\n",
    "  JPX_RAW=1         Output raw strings\n",
    "  JPX_COMPACT=1     Compact JSON output\n",
    "  JPX_CONFIG=PATH   Custom config file path\n",
    "\n",
    "CONFIG FILES:\n",
    "  ~/.config/jpx/config.toml  or  ~/.jpxrc\n",
    "\n",
    "Version: ", env!("CARGO_PKG_VERSION"),
    "\nDocumentation: https://docs.rs/jmespath_extensions"
))]
struct Args {
    /// Print help (use --help for more detail)
    #[arg(short = 'h', action = ArgAction::HelpShort, global = true)]
    help_short: (),

    /// Print detailed help with examples
    #[arg(long = "help", action = ArgAction::HelpLong, global = true)]
    help_long: (),

    /// JMESPath expression(s) to evaluate
    #[arg(
        short = 'e',
        long = "expression",
        conflicts_with = "query_file",
        help = "JMESPath expression(s) to evaluate",
        long_help = "JMESPath expression(s) to evaluate. Multiple -e flags are chained as a pipeline,\nwhere each expression receives the output of the previous one."
    )]
    expressions: Vec<String>,

    /// Expression(s) as positional args
    #[arg(conflicts_with_all = ["query_file", "expressions"])]
    positional_expressions: Vec<String>,

    /// Read expression from file
    #[arg(short = 'Q', long = "query-file", conflicts_with_all = ["positional_expressions", "expressions"],
          help = "Read expression from file or query library (.jpx)",
          long_help = "Read JMESPath expression from a file. Supports:\n\
            - Plain text files with a single expression\n\
            - Query libraries (.jpx) with named queries\n\n\
            For .jpx files, use colon syntax or --query:\n  \
            jpx -Q queries.jpx:my-query data.json\n  \
            jpx -Q queries.jpx --query my-query data.json")]
    query_file: Option<String>,

    /// Named query to run from a .jpx query library
    #[arg(
        long = "query",
        value_name = "NAME",
        help = "Named query to run from .jpx file",
        long_help = "Specify which named query to run from a .jpx query library.\n\
            Can also use colon syntax: -Q file.jpx:query-name"
    )]
    query_name: Option<String>,

    /// List available queries in a .jpx file
    #[arg(
        long = "list-queries",
        help = "List queries in a .jpx file",
        long_help = "List all named queries available in a .jpx query library file."
    )]
    list_queries: bool,

    /// Validate queries without running
    #[arg(
        long = "check",
        help = "Validate query file without running",
        long_help = "Parse and validate all queries in a .jpx file without executing.\n\
            Useful for CI/CD pipelines. Exit code 0 if all valid, 1 if errors."
    )]
    check_queries: bool,

    /// Input JSON file
    #[arg(
        short,
        long,
        help = "Input JSON file",
        long_help = "Input file to read JSON from. If not provided, reads from stdin.\nSupports any valid JSON file."
    )]
    file: Option<String>,

    /// Output raw strings without quotes
    #[arg(
        short = 'r',
        long,
        help = "Output raw strings without quotes",
        long_help = "Output raw strings without JSON quotes. Useful for piping string results\nto other commands. Can also be set with JPX_RAW=1 environment variable."
    )]
    raw: bool,

    /// Compact JSON output
    #[arg(
        short,
        long,
        help = "Compact JSON output (no pretty-printing)",
        long_help = "Compact output without pretty-printing or indentation.\nCan also be set with JPX_COMPACT=1 environment variable."
    )]
    compact: bool,

    /// Output as YAML
    #[arg(short = 'y', long, conflicts_with_all = ["toml_output", "csv_output", "tsv_output", "lines_output"])]
    yaml: bool,

    /// Output as TOML
    #[arg(long = "toml", conflicts_with_all = ["yaml", "csv_output", "tsv_output", "lines_output"])]
    toml_output: bool,

    /// Output as CSV
    #[arg(long = "csv", conflicts_with_all = ["yaml", "toml_output", "tsv_output", "lines_output"],
          help = "Output as CSV",
          long_help = "Output as CSV (comma-separated values). Best for arrays of objects.\nNested structures are flattened with dot notation.")]
    csv_output: bool,

    /// Output as TSV
    #[arg(long = "tsv", conflicts_with_all = ["yaml", "toml_output", "csv_output", "lines_output"],
          help = "Output as TSV",
          long_help = "Output as TSV (tab-separated values). Best for arrays of objects.\nNested structures are flattened with dot notation.")]
    tsv_output: bool,

    /// Output one value per line
    #[arg(short = 'l', long = "lines", conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "table"],
          help = "Output one JSON value per line",
          long_help = "Output one JSON value per line (NDJSON/JSON Lines format).\nUseful for streaming or piping array results to other tools.")]
    lines_output: bool,

    /// Output as formatted table
    #[arg(short = 't', long, conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "lines_output"],
          help = "Output as formatted table",
          long_help = "Output as a formatted table. Best for arrays of objects.\nUse --table-style to change appearance (unicode, ascii, markdown, plain).")]
    table: bool,

    /// Table style
    #[arg(
        long,
        value_name = "STYLE",
        default_value = "unicode",
        requires = "table",
        help = "Table style: unicode, ascii, markdown, plain",
        long_help = "Table style for --table output:\n  unicode   Box-drawing characters (default)\n  ascii     ASCII characters only\n  markdown  GitHub-flavored markdown\n  plain     No borders"
    )]
    table_style: String,

    /// Output as Parquet file (requires --output)
    #[cfg(feature = "parquet")]
    #[arg(
        long = "parquet",
        conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "lines_output", "table"],
        requires = "output",
        help = "Output as Parquet file",
        long_help = "Output as Parquet file. Requires --output to specify the file path.\nBest for arrays of objects. Uses Snappy compression."
    )]
    parquet_output: bool,

    /// Use null as input
    #[arg(
        short = 'n',
        long,
        help = "Use null as input (don't read stdin)",
        long_help = "Null input mode - use null as the input value instead of reading from stdin.\nUseful for expressions that don't need input, like: jpx -n 'now()'"
    )]
    null_input: bool,

    /// Slurp multiple inputs into array
    #[arg(
        short = 's',
        long,
        conflicts_with = "stream",
        help = "Slurp multiple JSON values into array",
        long_help = "Slurp mode - read multiple JSON values from input and combine them into\na single array. Useful for processing multiple JSON objects."
    )]
    slurp: bool,

    /// Process input line by line
    #[arg(long, visible_alias = "each", conflicts_with_all = ["slurp", "null_input"],
          help = "Process input line by line (NDJSON)",
          long_help = "Stream mode - process input line by line (for NDJSON/JSON Lines).\nEach line is parsed and evaluated independently with constant memory usage.")]
    stream: bool,

    /// Color output mode
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,

    /// Output file
    #[arg(
        short = 'o',
        long,
        help = "Write output to file",
        long_help = "Write output to a file instead of stdout."
    )]
    output: Option<String>,

    /// Suppress errors and warnings
    #[arg(
        short = 'q',
        long,
        help = "Suppress errors and warnings",
        long_help = "Quiet mode - suppress errors and warnings.\nCan also be set with JPX_QUIET=1 environment variable."
    )]
    quiet: bool,

    /// Show expression details and timing
    #[arg(
        short = 'v',
        long,
        help = "Show expression details and timing",
        long_help = "Verbose mode - show expression details, input info, and timing.\nCan also be set with JPX_VERBOSE=1 environment variable."
    )]
    verbose: bool,

    /// Disable extension functions
    #[arg(
        long,
        help = "Use only standard JMESPath functions",
        long_help = "Strict mode - only use standard JMESPath functions (no extensions).\nCan also be set with JPX_STRICT=1 environment variable."
    )]
    strict: bool,

    /// Generate shell completions
    #[arg(
        long,
        value_name = "SHELL",
        help = "Generate shell completions",
        long_help = "Generate shell completion script for the specified shell.\nSupported: bash, zsh, fish, powershell, elvish"
    )]
    completions: Option<Shell>,

    /// List all functions
    #[arg(
        long,
        help = "List all available functions",
        long_help = "List all available extension functions organized by category.\nUse --list-category for a specific category or --describe for details."
    )]
    list_functions: bool,

    /// List functions in category
    #[arg(
        long,
        value_name = "CATEGORY",
        help = "List functions in a category",
        long_help = "List functions in a specific category. Categories include:\narray, datetime, math, string, hash, encoding, object, and more."
    )]
    list_category: Option<String>,

    /// Show function details
    #[arg(
        long,
        value_name = "FUNCTION",
        help = "Show detailed info for a function",
        long_help = "Show detailed information about a specific function including\nsignature, description, and usage examples."
    )]
    describe: Option<String>,

    /// Search for functions
    #[arg(
        long,
        value_name = "QUERY",
        help = "Search functions by name or description",
        long_help = "Search functions by name, description, or category using fuzzy matching.\nResults are ranked by relevance."
    )]
    search: Option<String>,

    /// Find similar functions
    #[arg(
        long,
        value_name = "FUNCTION",
        help = "Find functions similar to another",
        long_help = "Find functions similar to the specified function based on category,\nsignature, and description keywords."
    )]
    similar: Option<String>,

    /// Generate JSON Patch from two files
    #[arg(long, num_args = 2, value_names = ["SOURCE", "TARGET"],
          help = "Generate JSON Patch from two files",
          long_help = "Generate JSON Patch (RFC 6902) that transforms SOURCE into TARGET.\nOutput can be used with --patch to apply changes.")]
    diff: Option<Vec<String>>,

    /// Apply JSON Patch
    #[arg(
        long,
        value_name = "PATCH_FILE",
        help = "Apply JSON Patch (RFC 6902)",
        long_help = "Apply JSON Patch (RFC 6902) to a document.\nReads document from -f or stdin, applies patch operations from file."
    )]
    patch: Option<String>,

    /// Apply JSON Merge Patch
    #[arg(
        long,
        value_name = "MERGE_FILE",
        help = "Apply JSON Merge Patch (RFC 7396)",
        long_help = "Apply JSON Merge Patch (RFC 7396) to a document.\nReads document from -f or stdin, merges with patch from file."
    )]
    merge: Option<String>,

    /// Show expression AST
    #[arg(
        long,
        help = "Show how expression is parsed (AST)",
        long_help = "Explain how an expression is parsed by showing the Abstract Syntax Tree.\nUseful for understanding complex expressions and debugging."
    )]
    explain: bool,

    /// Show diagnostic info
    #[arg(
        long,
        help = "Show diagnostic information",
        long_help = "Show diagnostic information for troubleshooting including\nenvironment variables, effective settings, and input info."
    )]
    debug: bool,

    /// Start interactive REPL
    #[arg(
        long,
        help = "Start interactive REPL mode",
        long_help = "Start interactive REPL (Read-Eval-Print Loop) mode.\nSupports history, tab completion, and multi-line editing."
    )]
    repl: bool,

    /// Load demo dataset
    #[arg(
        long,
        value_name = "NAME",
        help = "Load a demo dataset for REPL",
        long_help = "Load a demo dataset when starting REPL mode.\nAvailable: users, products, github, mixed"
    )]
    demo: Option<String>,

    /// Show input statistics
    #[arg(
        long,
        help = "Show statistics about input data",
        long_help = "Show statistics about the input JSON data including type, size,\ndepth, and field analysis for arrays of objects."
    )]
    stats: bool,

    /// List all JSON paths
    #[arg(
        long,
        help = "List all paths in input JSON",
        long_help = "List all paths in the input JSON document.\nUse --types and --values for additional details."
    )]
    paths: bool,

    /// Show types with paths
    #[arg(long, requires = "paths")]
    types: bool,

    /// Show values with paths
    #[arg(long, requires = "paths")]
    values: bool,

    /// Benchmark expression
    #[arg(long, value_name = "ITERATIONS", default_missing_value = "100", num_args = 0..=1,
          help = "Benchmark expression performance",
          long_help = "Benchmark expression performance over multiple iterations.\nShows statistics including mean, median, p95, p99, and throughput.")]
    bench: Option<u32>,

    /// Warmup iterations for benchmark
    #[arg(
        long,
        value_name = "COUNT",
        default_value = "5",
        requires = "bench",
        help = "Warmup iterations before benchmarking"
    )]
    warmup: u32,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("jpx: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = Args::parse();

    // Load config and apply defaults (priority: config < env var < CLI flag)
    let config = load_config();
    apply_config_defaults(&mut args, &config);
    apply_env_defaults(&mut args);

    // Handle shell completions
    if let Some(shell) = args.completions {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

    // Handle REPL mode
    if args.repl || args.demo.is_some() {
        return repl::run(args.demo.as_deref());
    }

    // Create registry for introspection
    let mut registry = FunctionRegistry::new();
    registry.register_all();

    if args.list_functions {
        print_functions(&registry);
        return Ok(());
    }

    if let Some(category_name) = &args.list_category {
        print_category(&registry, category_name)?;
        return Ok(());
    }

    if let Some(func_name) = &args.describe {
        describe_function(&registry, func_name)?;
        return Ok(());
    }

    if let Some(query) = &args.search {
        search_functions(&registry, query);
        return Ok(());
    }

    if let Some(func_name) = &args.similar {
        find_similar_functions(&registry, func_name)?;
        return Ok(());
    }

    // Debug mode: show diagnostic information
    if args.debug {
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

    // Handle --diff: generate JSON Patch from two files
    if let Some(files) = &args.diff {
        return diff_files(&files[0], &files[1], args.compact, &args.color);
    }

    // Handle --patch: apply JSON Patch to document
    if let Some(patch_file) = &args.patch {
        return apply_patch(&args.file, patch_file, args.compact, &args.color);
    }

    // Handle --merge: apply JSON Merge Patch to document
    if let Some(merge_file) = &args.merge {
        return apply_merge(&args.file, merge_file, args.compact, &args.color);
    }

    // Handle --stats: show data statistics
    if args.stats {
        return show_stats(&args.file, &args.color);
    }

    // Handle --paths: list all paths in JSON
    if args.paths {
        return show_paths(&args.file, &args.color, args.types, args.values);
    }

    // Handle --list-queries: list available queries in a .jpx file
    if args.list_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--list-queries requires -Q/--query-file"))?;
        return list_queries(query_path, &args.color);
    }

    // Handle --check: validate queries without running
    if args.check_queries {
        let query_path = args
            .query_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--check requires -Q/--query-file"))?;
        return check_queries(query_path, &args.color);
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

            let ast = jmespath::parse(expression)
                .with_context(|| format!("Failed to parse expression: {}", expression))?;

            print_ast(&ast, 0);
            println!();
        }
        return Ok(());
    }

    // Handle --stream: process input line by line (NDJSON/JSON Lines)
    if args.stream {
        return run_streaming(&expressions, &args);
    }

    // Get input data
    let data = if args.null_input {
        // Null input mode - don't read anything
        Variable::Null
    } else {
        // Check for parquet input
        #[cfg(feature = "parquet")]
        if let Some(path) = &args.file {
            if path.ends_with(".parquet") || path.ends_with(".pq") {
                let json_value =
                    jpx::parquet_support::read_parquet_to_json(std::path::Path::new(path))
                        .with_context(|| format!("Failed to read parquet file: {}", path))?;
                let json_str = serde_json::to_string(&json_value)?;
                Variable::from_json(&json_str).map_err(|e| json_parse_error(&json_str, e))?
            } else {
                read_input_as_variable(&args)?
            }
        } else {
            read_input_as_variable(&args)?
        }

        #[cfg(not(feature = "parquet"))]
        read_input_as_variable(&args)?
    };

    // Create runtime with extensions (unless strict mode)
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    if !args.strict {
        register_all(&mut runtime);
    }

    // Verbose mode: show input info
    if args.verbose {
        if args.strict {
            eprintln!("Mode: strict (standard JMESPath only)");
        }
        eprintln!("Input: {}", describe_value(&Rc::new(data.clone())));
        if expressions.len() > 1 {
            eprintln!("Expressions: {} (chained)", expressions.len());
        }
        eprintln!();
    }

    // Handle --bench: benchmark expression performance
    if let Some(iterations) = args.bench {
        return run_benchmark(
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
    let mut result: Rc<Variable> = Rc::new(data.clone());

    for (i, expression) in expressions.iter().enumerate() {
        if args.verbose {
            eprintln!("[{}] Expression: {}", i + 1, expression);
        }

        let expr = runtime
            .compile(expression)
            .with_context(|| format!("Failed to compile expression: {}", expression))?;

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
            eprintln!("[{}] Result: {}", i + 1, describe_value(&result));
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
        if let Some(s) = result.as_string() {
            println!("{}", s);
            return Ok(());
        }
    }

    // Convert to serde_json::Value for output formatting
    let json_value: serde_json::Value = serde_json::to_value(&*result)?;

    // Handle alternative output formats
    if args.yaml {
        return output_as_yaml(&json_value, &args.output);
    }
    if args.toml_output {
        return output_as_toml(&json_value, &args.output);
    }
    if args.csv_output {
        return output_as_csv(&json_value, &args.output);
    }
    if args.tsv_output {
        return output_as_tsv(&json_value, &args.output);
    }
    if args.lines_output {
        return output_as_lines(&json_value, &args.output);
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
        return output_as_table(&json_value, &args.output, &args.table_style, &args.color);
    }

    // When writing to file, don't colorize unless explicitly requested
    let should_colorize = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => args.output.is_none() && atty::is(atty::Stream::Stdout),
    };

    let output = if should_colorize && !args.compact {
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
        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path))?;
        writeln!(file, "{}", output)
            .with_context(|| format!("Failed to write to output file: {}", output_path))?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Create a helpful error message for JSON parse failures
fn json_parse_error(input: &str, err: impl std::fmt::Display) -> anyhow::Error {
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
fn read_input_as_variable(args: &Args) -> Result<Variable> {
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

fn print_functions(registry: &FunctionRegistry) {
    println!(
        "{}\n",
        "jpx - JMESPath with Extended Functions".bold().green()
    );

    // Count standard and extension functions
    let standard_count = registry.functions().filter(|f| f.is_standard).count();
    let extension_count = registry.functions().filter(|f| !f.is_standard).count();

    // Print standard functions header
    println!(
        "  {} {}:",
        "▸".dimmed(),
        format!("STANDARD ({})", standard_count).dimmed()
    );
    for func in registry.functions_in_category(Category::Standard) {
        println!(
            "    {} [{}] - {}",
            func.name.cyan().bold(),
            "std".dimmed(),
            func.description
        );
    }
    println!();

    // Print extension functions by category
    println!(
        "{} ({} available):\n",
        "Extension functions".bold(),
        extension_count.to_string().yellow()
    );

    for category in Category::all() {
        if *category == Category::Standard || !category.is_available() {
            continue;
        }

        let funcs: Vec<_> = registry.functions_in_category(*category).collect();
        if funcs.is_empty() {
            continue;
        }

        println!(
            "  {} {}:",
            "▸".dimmed(),
            format!("{} ({})", category.name().to_uppercase(), funcs.len())
                .green()
                .bold()
        );
        for func in funcs {
            println!("    {} - {}", func.name.cyan().bold(), func.description);
        }
        println!();
    }

    println!(
        "Use {} for details on a category",
        "--list-category <name>".cyan()
    );
    println!(
        "Use {} for details on a specific function",
        "--describe <function>".cyan()
    );
    println!("Use {} to find functions", "--search <query>".cyan());
    println!(
        "\nFor full documentation: {}",
        "https://docs.rs/jmespath_extensions".blue().underline()
    );
}

/// Search result with relevance score
struct SearchResult<'a> {
    func: &'a jmespath_extensions::registry::FunctionInfo,
    score: i32,
    match_type: MatchType,
}

#[derive(Debug, Clone, Copy)]
enum MatchType {
    ExactName,
    NamePrefix,
    NameContains,
    AliasMatch,
    CategoryMatch,
    DescriptionMatch,
    SignatureMatch,
}

impl MatchType {
    fn label(&self) -> &'static str {
        match self {
            MatchType::ExactName => "exact",
            MatchType::NamePrefix => "prefix",
            MatchType::NameContains => "name",
            MatchType::AliasMatch => "alias",
            MatchType::CategoryMatch => "category",
            MatchType::DescriptionMatch => "description",
            MatchType::SignatureMatch => "signature",
        }
    }
}

fn search_functions(registry: &FunctionRegistry, query: &str) {
    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for func in registry.functions() {
        let name_lower = func.name.to_lowercase();
        let desc_lower = func.description.to_lowercase();
        let sig_lower = func.signature.to_lowercase();
        let cat_lower = func.category.name().to_lowercase();

        // Scoring: higher = better match
        let (score, match_type) = if name_lower == query_lower {
            (1000, MatchType::ExactName)
        } else if name_lower.starts_with(&query_lower) {
            (800, MatchType::NamePrefix)
        } else if name_lower.contains(&query_lower) {
            (600, MatchType::NameContains)
        } else if func
            .aliases
            .iter()
            .any(|a| a.to_lowercase().contains(&query_lower))
        {
            (500, MatchType::AliasMatch)
        } else if cat_lower.contains(&query_lower) {
            (400, MatchType::CategoryMatch)
        } else if desc_lower.contains(&query_lower) {
            // Boost if query appears early in description
            let pos = desc_lower.find(&query_lower).unwrap_or(100);
            (300 - pos.min(100) as i32, MatchType::DescriptionMatch)
        } else if sig_lower.contains(&query_lower) {
            (100, MatchType::SignatureMatch)
        } else {
            continue; // No match
        };

        results.push(SearchResult {
            func,
            score,
            match_type,
        });
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.cmp(&a.score));

    if results.is_empty() {
        println!(
            "{} No functions found matching '{}'\n",
            "✗".red(),
            query.yellow()
        );
        println!("Try searching for:");
        println!("  • Function names: {}", "median, split, json".cyan());
        println!("  • Categories: {}", "math, string, array".cyan());
        println!("  • Concepts: {}", "hash, encode, date".cyan());
        return;
    }

    println!(
        "{} Found {} functions matching '{}':\n",
        "✓".green(),
        results.len().to_string().yellow(),
        query.cyan()
    );

    // Group by match type for nicer display
    let mut current_group: Option<&str> = None;

    for result in results.iter().take(25) {
        let group = result.match_type.label();
        if current_group != Some(group) {
            if current_group.is_some() {
                println!();
            }
            println!("  {} {}:", "▸".dimmed(), group.to_uppercase().dimmed());
            current_group = Some(group);
        }

        let type_badge = if result.func.is_standard {
            "std".dimmed()
        } else {
            result.func.category.name().green()
        };

        println!(
            "    {} [{}] - {}",
            result.func.name.cyan().bold(),
            type_badge,
            result.func.description
        );
    }

    if results.len() > 25 {
        println!(
            "\n  {} and {} more... (refine your search)",
            "...".dimmed(),
            (results.len() - 25).to_string().yellow()
        );
    }

    println!(
        "\nUse {} for details on a specific function",
        "--describe <function>".cyan()
    );
}

fn print_category(registry: &FunctionRegistry, category_name: &str) -> Result<()> {
    let category = Category::all()
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(category_name))
        .ok_or_else(|| {
            let available: Vec<_> = Category::all()
                .iter()
                .filter(|c| c.is_available())
                .map(|c| c.name())
                .collect();
            anyhow::anyhow!(
                "Unknown category '{}'. Available: {}",
                category_name,
                available.join(", ")
            )
        })?;

    if !category.is_available() {
        return Err(anyhow::anyhow!(
            "Category '{}' is not available (not compiled in)",
            category_name
        ));
    }

    println!(
        "{} functions:\n",
        category.name().to_uppercase().green().bold()
    );

    for func in registry.functions_in_category(*category) {
        println!("  {} - {}", func.name.cyan().bold(), func.description);
        println!("    {}: {}", "Signature".dimmed(), func.signature);
        println!("    {}: {}", "Example".dimmed(), func.example.yellow());
        println!();
    }

    Ok(())
}

fn describe_function(registry: &FunctionRegistry, func_name: &str) -> Result<()> {
    let func = registry.get_function(func_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown function '{}'. Use --list-functions to see available functions.",
            func_name
        )
    })?;

    println!("{}", func.name.cyan().bold());
    println!("{}", "=".repeat(func.name.len()).dimmed());
    println!();
    println!(
        "{}: {}",
        "Type".dimmed(),
        if func.is_standard {
            "standard JMESPath".normal()
        } else {
            "extension".green()
        }
    );
    println!("{}: {}", "Category".dimmed(), func.category.name().green());
    if let Some(jep) = func.jep {
        println!("{}: {}", "JEP".dimmed(), jep.yellow());
    }
    println!("{}: {}", "Description".dimmed(), func.description);
    println!("{}: {}", "Signature".dimmed(), func.signature.white());
    println!();
    println!("{}:", "Example".bold());
    println!("  {}", func.example.yellow());

    Ok(())
}

/// Find functions similar to the specified function
fn find_similar_functions(registry: &FunctionRegistry, func_name: &str) -> Result<()> {
    let target = registry.get_function(func_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown function '{}'. Use --list-functions to see available functions.",
            func_name
        )
    })?;

    println!("Functions similar to '{}':\n", target.name.cyan().bold());

    // Parse the target signature to extract input/output types
    let target_sig = parse_signature(target.signature);

    // 1. Same category (excluding the target itself)
    let same_category: Vec<_> = registry
        .functions_in_category(target.category)
        .filter(|f| f.name != target.name)
        .collect();

    if !same_category.is_empty() {
        println!(
            "  {} {} ({}):",
            "▸".dimmed(),
            "Same category".bold(),
            target.category.name().green()
        );
        for func in same_category.iter().take(8) {
            println!(
                "    {} - {}",
                func.name.cyan(),
                truncate_str(func.description, 50)
            );
        }
        if same_category.len() > 8 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (same_category.len() - 8).to_string().yellow()
            );
        }
        println!();
    }

    // 2. Similar signature (same input -> output pattern)
    let similar_sig: Vec<_> = registry
        .functions()
        .filter(|f| {
            f.name != target.name
                && f.category != target.category
                && signatures_match(&parse_signature(f.signature), &target_sig)
        })
        .collect();

    if !similar_sig.is_empty() {
        println!(
            "  {} {} ({}):",
            "▸".dimmed(),
            "Similar signature".bold(),
            target.signature.white()
        );
        for func in similar_sig.iter().take(8) {
            println!(
                "    {} [{}] - {}",
                func.name.cyan(),
                func.category.name().green(),
                truncate_str(func.description, 45)
            );
        }
        if similar_sig.len() > 8 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (similar_sig.len() - 8).to_string().yellow()
            );
        }
        println!();
    }

    // 3. Related by description keywords
    let keywords = extract_keywords(target.description);
    let mut related: Vec<(&FunctionInfo, usize)> = registry
        .functions()
        .filter(|f| f.name != target.name && f.category != target.category)
        .filter_map(|f| {
            let score = keywords
                .iter()
                .filter(|kw| f.description.to_lowercase().contains(&kw.to_lowercase()))
                .count();
            if score > 0 { Some((f, score)) } else { None }
        })
        .collect();

    related.sort_by(|a, b| b.1.cmp(&a.1));

    if !related.is_empty() {
        println!("  {} {}:", "▸".dimmed(), "Related concepts".bold());
        for (func, _score) in related.iter().take(6) {
            println!(
                "    {} [{}] - {}",
                func.name.cyan(),
                func.category.name().green(),
                truncate_str(func.description, 45)
            );
        }
        if related.len() > 6 {
            println!(
                "    {} and {} more...",
                "...".dimmed(),
                (related.len() - 6).to_string().yellow()
            );
        }
        println!();
    }

    println!(
        "Use {} for details on a specific function",
        "--describe <function>".cyan()
    );

    Ok(())
}

/// Parse a signature string into input types and output type
fn parse_signature(sig: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = sig.split("->").collect();
    if parts.len() != 2 {
        return (vec![], String::new());
    }

    let inputs: Vec<String> = parts[0]
        .split(',')
        .map(|s| normalize_type(s.trim()))
        .collect();
    let output = normalize_type(parts[1].trim());

    (inputs, output)
}

/// Normalize type names for comparison
fn normalize_type(t: &str) -> String {
    // Remove optional markers and variadic indicators
    let t = t.trim_end_matches('?').trim_end_matches("...");
    // Simplify to base type
    match t {
        "number" | "integer" => "number".to_string(),
        "string" | "str" => "string".to_string(),
        "array" | "list" => "array".to_string(),
        "object" | "hash" | "map" => "object".to_string(),
        "boolean" | "bool" => "boolean".to_string(),
        "any" | "expression" | "expref" => "any".to_string(),
        _ => t.to_lowercase(),
    }
}

/// Check if two signatures are similar enough
fn signatures_match(a: &(Vec<String>, String), b: &(Vec<String>, String)) -> bool {
    // Must have same output type
    if a.1 != b.1 || a.1.is_empty() {
        return false;
    }

    // Input types should be similar (same count and compatible types)
    if a.0.len() != b.0.len() {
        return false;
    }

    // At least first input type should match
    if !a.0.is_empty() && !b.0.is_empty() && a.0[0] == b.0[0] {
        return true;
    }

    false
}

/// Extract meaningful keywords from a description
fn extract_keywords(description: &str) -> Vec<String> {
    let stopwords = [
        "a",
        "an",
        "the",
        "to",
        "of",
        "in",
        "for",
        "is",
        "are",
        "and",
        "or",
        "with",
        "from",
        "by",
        "on",
        "at",
        "as",
        "if",
        "be",
        "this",
        "that",
        "it",
        "its",
        "can",
        "will",
        "into",
        "using",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "specified",
    ];

    description
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Describe a Variable value for verbose output
fn describe_value(value: &Rc<Variable>) -> String {
    match value.as_ref() {
        Variable::Null => "null".to_string(),
        Variable::Bool(b) => format!("bool ({})", b),
        Variable::Number(n) => format!("number ({})", n),
        Variable::String(s) => {
            if s.len() > 50 {
                format!("string ({} chars)", s.len())
            } else {
                format!("string \"{}\"", s)
            }
        }
        Variable::Array(arr) => format!("array ({} items)", arr.len()),
        Variable::Object(obj) => format!("object ({} keys)", obj.len()),
        Variable::Expref(_) => "expression reference".to_string(),
    }
}

/// Print AST in a human-readable tree format
fn print_ast(node: &Ast, indent: usize) {
    let prefix = "  ".repeat(indent);
    let connector = if indent > 0 { "├─ " } else { "" };

    match node {
        Ast::Identity { .. } => {
            println!("{}{}@ (current node)", prefix, connector);
        }
        Ast::Field { name, .. } => {
            println!("{}{}Field: {}", prefix, connector, name);
        }
        Ast::Index { idx, .. } => {
            println!("{}{}Index: [{}]", prefix, connector, idx);
        }
        Ast::Slice {
            start, stop, step, ..
        } => {
            let start_str = start.map_or("".to_string(), |s| s.to_string());
            let stop_str = stop.map_or("".to_string(), |s| s.to_string());
            if *step == 1 {
                println!("{}{}Slice: [{}:{}]", prefix, connector, start_str, stop_str);
            } else {
                println!(
                    "{}{}Slice: [{}:{}:{}]",
                    prefix, connector, start_str, stop_str, step
                );
            }
        }
        Ast::Subexpr { lhs, rhs, .. } => {
            println!("{}{}Subexpression (a.b):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Projection { lhs, rhs, .. } => {
            println!("{}{}Projection (map over array):", prefix, connector);
            println!("{}  source:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  project:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::Function { name, args, .. } => {
            if args.is_empty() {
                println!("{}{}Function: {}()", prefix, connector, name);
            } else {
                println!("{}{}Function: {}", prefix, connector, name);
                for (i, arg) in args.iter().enumerate() {
                    println!("{}  arg {}:", prefix, i + 1);
                    print_ast(arg, indent + 2);
                }
            }
        }
        Ast::Literal { value, .. } => {
            let json = serde_json::to_string(&**value).unwrap_or_else(|_| "?".to_string());
            println!("{}{}Literal: `{}`", prefix, connector, json);
        }
        Ast::Comparison {
            comparator,
            lhs,
            rhs,
            ..
        } => {
            let op = match comparator {
                jmespath::ast::Comparator::Equal => "==",
                jmespath::ast::Comparator::NotEqual => "!=",
                jmespath::ast::Comparator::LessThan => "<",
                jmespath::ast::Comparator::LessThanEqual => "<=",
                jmespath::ast::Comparator::GreaterThan => ">",
                jmespath::ast::Comparator::GreaterThanEqual => ">=",
            };
            println!("{}{}Comparison: {}", prefix, connector, op);
            println!("{}  left:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  right:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::And { lhs, rhs, .. } => {
            println!("{}{}And (&&):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Or { lhs, rhs, .. } => {
            println!("{}{}Or (||):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Not { node, .. } => {
            println!("{}{}Not (!):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            println!("{}{}Filter condition ([?...]):", prefix, connector);
            println!("{}  predicate:", prefix);
            print_ast(predicate, indent + 2);
            println!("{}  then:", prefix);
            print_ast(then, indent + 2);
        }
        Ast::Flatten { node, .. } => {
            println!("{}{}Flatten ([]):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::ObjectValues { node, .. } => {
            println!("{}{}Object values (*):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::MultiList { elements, .. } => {
            println!(
                "{}{}Multi-select list ({} elements):",
                prefix,
                connector,
                elements.len()
            );
            for (i, elem) in elements.iter().enumerate() {
                println!("{}  [{}]:", prefix, i);
                print_ast(elem, indent + 2);
            }
        }
        Ast::MultiHash { elements, .. } => {
            println!(
                "{}{}Multi-select hash ({} keys):",
                prefix,
                connector,
                elements.len()
            );
            for kvp in elements {
                println!("{}  {}:", prefix, kvp.key);
                print_ast(&kvp.value, indent + 2);
            }
        }
        Ast::Expref { ast, .. } => {
            println!("{}{}Expression reference (&):", prefix, connector);
            print_ast(ast, indent + 1);
        }
    }
}

// =============================================================================
// JSON Patch / Merge operations
// =============================================================================

/// Read JSON from a file or stdin
fn read_json_from(path: Option<&str>) -> Result<serde_json::Value> {
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

/// Output JSON with optional coloring
fn output_json(value: &serde_json::Value, compact: bool, color_mode: &ColorMode) -> Result<()> {
    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    if compact {
        println!("{}", serde_json::to_string(value)?);
    } else if use_color {
        use colored_json::{ColoredFormatter, PrettyFormatter};
        let formatter = ColoredFormatter::new(PrettyFormatter::new());
        println!("{}", formatter.to_colored_json_auto(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

/// Generate JSON Patch (RFC 6902) from two files
fn diff_files(
    source_path: &str,
    target_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read source file: {}", source_path))?,
    )
    .context("Failed to parse source JSON")?;

    let target: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(target_path)
            .with_context(|| format!("Failed to read target file: {}", target_path))?,
    )
    .context("Failed to parse target JSON")?;

    let patch = json_patch::diff(&source, &target);

    if patch.0.is_empty() {
        eprintln!("{} No differences found", "✓".green());
        println!("[]");
        return Ok(());
    }

    eprintln!(
        "{} Generated {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    let patch_value = serde_json::to_value(&patch)?;
    output_json(&patch_value, compact, color_mode)
}

/// List queries in a .jpx query library file
fn list_queries(query_path: &str, color_mode: &ColorMode) -> Result<()> {
    let (file_path, _) = query_library::parse_query_path(query_path);
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read query file: {}", file_path))?;

    let library = query_library::QueryLibrary::parse(&content)
        .with_context(|| format!("Failed to parse query library: {}", file_path))?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    println!("Queries in {}:\n", file_path);

    // Calculate column widths
    let max_name_len = library
        .list()
        .iter()
        .map(|q| q.name.len())
        .max()
        .unwrap_or(4);
    let name_width = max_name_len.max(4);

    // Header
    if use_color {
        println!(
            "  {:<name_width$}  {}",
            "NAME".bold(),
            "DESCRIPTION".bold(),
            name_width = name_width
        );
    } else {
        println!(
            "  {:<name_width$}  DESCRIPTION",
            "NAME",
            name_width = name_width
        );
    }
    println!(
        "  {:-<name_width$}  {:-<40}",
        "",
        "",
        name_width = name_width
    );

    // Queries
    for query in library.list() {
        let desc = query.description.as_deref().unwrap_or("-");
        if use_color {
            println!(
                "  {:<name_width$}  {}",
                query.name.cyan(),
                desc,
                name_width = name_width
            );
        } else {
            println!(
                "  {:<name_width$}  {}",
                query.name,
                desc,
                name_width = name_width
            );
        }
    }

    println!("\nUse: jpx -Q {}:<query-name> <input>", file_path);

    Ok(())
}

/// Validate queries in a .jpx file without running them
fn check_queries(query_path: &str, color_mode: &ColorMode) -> Result<()> {
    let (file_path, _) = query_library::parse_query_path(query_path);
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read query file: {}", file_path))?;

    let library = query_library::QueryLibrary::parse(&content)
        .with_context(|| format!("Failed to parse query library: {}", file_path))?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    // Create runtime with all functions registered for proper validation
    let runtime = create_runtime();

    let mut has_errors = false;

    println!("Validating {}...\n", file_path);

    for query in library.list() {
        // Use runtime.compile() to validate function names too
        match runtime.compile(&query.expression) {
            Ok(_) => {
                if use_color {
                    println!("  {} {}", "✓".green(), query.name);
                } else {
                    println!("  [OK] {}", query.name);
                }
            }
            Err(e) => {
                has_errors = true;
                if use_color {
                    println!(
                        "  {} {} (line {})",
                        "✗".red(),
                        query.name.red(),
                        query.line_number
                    );
                    println!("    {}", e.to_string().red());
                } else {
                    println!("  [ERROR] {} (line {})", query.name, query.line_number);
                    println!("    {}", e);
                }
            }
        }
    }

    println!();

    if has_errors {
        if use_color {
            println!("{}", "Validation failed.".red().bold());
        } else {
            println!("Validation failed.");
        }
        std::process::exit(1);
    } else if use_color {
        println!("{}", "All queries valid.".green().bold());
    } else {
        println!("All queries valid.");
    }

    Ok(())
}

/// Apply JSON Patch (RFC 6902) to a document
fn apply_patch(
    doc_path: &Option<String>,
    patch_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the patch from file
    let patch_content = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Failed to read patch file: {}", patch_path))?;
    let patch_value: serde_json::Value =
        serde_json::from_str(&patch_content).context("Failed to parse patch JSON")?;

    // Convert to json_patch::Patch
    let patch: json_patch::Patch =
        serde_json::from_value(patch_value).context("Invalid JSON Patch format")?;

    // Apply the patch
    json_patch::patch(&mut doc, &patch).context("Failed to apply patch")?;

    eprintln!(
        "{} Applied {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    output_json(&doc, compact, color_mode)
}

/// Apply JSON Merge Patch (RFC 7396) to a document
fn apply_merge(
    doc_path: &Option<String>,
    merge_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the merge patch from file
    let merge_content = std::fs::read_to_string(merge_path)
        .with_context(|| format!("Failed to read merge patch file: {}", merge_path))?;
    let merge_patch: serde_json::Value =
        serde_json::from_str(&merge_content).context("Failed to parse merge patch JSON")?;

    // Apply the merge patch
    json_patch::merge(&mut doc, &merge_patch);

    eprintln!("{} Applied merge patch", "✓".green());

    output_json(&doc, compact, color_mode)
}

// =============================================================================
// Data Statistics
// =============================================================================

/// Show statistics about JSON data
fn show_stats(file_path: &Option<String>, color_mode: &ColorMode) -> Result<()> {
    let data = read_json_from(file_path.as_deref())?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
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
            println!("{} {} chars", label("Length:"), number(s.len()));
            if s.len() <= 100 {
                println!("{} \"{}\"", label("Value:"), s);
            } else {
                println!("{} \"{}...\"", label("Preview:"), &s[..100]);
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

fn format_with_commas(n: usize) -> String {
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

fn format_bytes_human(bytes: usize) -> String {
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

fn get_type_name(value: &serde_json::Value) -> String {
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
            if s.len() <= 30 {
                format!("\"{}\"", s)
            } else {
                format!("\"{}...\"", &s[..27])
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{{} keys}}", obj.len()),
    }
}

// =============================================================================
// Output Format Functions
// =============================================================================

/// Helper to write output to file or stdout
fn write_output(content: &str, output_path: &Option<String>) -> Result<()> {
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

/// Output as YAML
fn output_as_yaml(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
    let yaml = serde_yaml::to_string(value).context("Failed to serialize to YAML")?;
    // Remove trailing newline that serde_yaml adds
    let yaml = yaml.trim_end();
    write_output(yaml, output_path)
}

/// Output as TOML
fn output_as_toml(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
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
fn output_as_lines(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
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
fn output_as_csv(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
    output_as_delimited(value, output_path, b',')
}

/// Output as TSV
fn output_as_tsv(value: &serde_json::Value, output_path: &Option<String>) -> Result<()> {
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
fn collect_flattened_keys(
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
fn flatten_object(value: &serde_json::Value) -> BTreeMap<String, serde_json::Value> {
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
fn output_as_table(
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
    use tabled::{Table, settings::Style};

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

    write_output(&table.to_string(), output_path)
}

/// Output array of primitives as a single-column table
fn output_primitives_as_table(
    arr: &[serde_json::Value],
    output_path: &Option<String>,
    style: &str,
) -> Result<()> {
    use tabled::{Table, settings::Style};

    let mut rows: Vec<Vec<String>> = Vec::new();
    rows.push(vec!["Value".to_string()]);

    for item in arr {
        rows.push(vec![value_to_table_cell(item, false)]);
    }

    let mut table = Table::from_iter(rows);

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
        }
    };

    write_output(&table.to_string(), output_path)
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

// =============================================================================
// Path Listing
// =============================================================================

/// Show all paths in the JSON data
fn show_paths(
    file_path: &Option<String>,
    color_mode: &ColorMode,
    show_types: bool,
    show_values: bool,
) -> Result<()> {
    let data = read_json_from(file_path.as_deref())?;

    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
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
            let preview = if s.len() <= 40 {
                format!("\"{}\"", s)
            } else {
                format!("\"{}...\"", &s[..37])
            };
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

// =============================================================================
// Benchmarking
// =============================================================================

/// Run benchmark for expression(s)
fn run_benchmark(
    runtime: &Runtime,
    expressions: &[String],
    data: &Variable,
    iterations: u32,
    warmup: u32,
    color_mode: &ColorMode,
) -> Result<()> {
    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    // Helper for colored output
    let heading = |s: &str| -> String {
        if use_color {
            s.green().bold().to_string()
        } else {
            s.to_string()
        }
    };

    let label = |s: &str| -> String {
        if use_color {
            s.dimmed().to_string()
        } else {
            s.to_string()
        }
    };

    let highlight = |s: &str| -> String {
        if use_color {
            s.cyan().bold().to_string()
        } else {
            s.to_string()
        }
    };

    let number = |s: &str| -> String {
        if use_color {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    };

    // Calculate input size
    let input_json = serde_json::to_string(data)?;
    let input_size = input_json.len();
    let item_count = match data {
        Variable::Array(arr) => Some(arr.len()),
        Variable::Object(obj) => Some(obj.len()),
        _ => None,
    };

    // Compile all expressions first
    let compiled: Vec<_> = expressions
        .iter()
        .map(|expr| {
            runtime
                .compile(expr)
                .with_context(|| format!("Failed to compile expression: {}", expr))
        })
        .collect::<Result<Vec<_>>>()?;

    // Combined expression string for display
    let expr_display = if expressions.len() == 1 {
        expressions[0].clone()
    } else {
        expressions.join(" | ")
    };

    println!();
    println!("{}", heading("BENCHMARK"));
    println!("{}", "═".repeat(60));
    println!();

    // Show expression
    println!("{} {}", label("Expression:"), highlight(&expr_display));

    // Show input info
    let size_str = format_bytes_human(input_size);
    if let Some(count) = item_count {
        println!(
            "{} {} ({} items)",
            label("Input size:"),
            size_str,
            format_with_commas(count)
        );
    } else {
        println!("{} {}", label("Input size:"), size_str);
    }
    println!();

    // Warmup runs
    if warmup > 0 {
        print!("{} {} iterations... ", label("Warmup:"), warmup);
        io::stdout().flush()?;
        for _ in 0..warmup {
            let mut result: Rc<Variable> = Rc::new(data.clone());
            for expr in &compiled {
                result = expr.search(&result).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
        println!("done");
    }

    // Benchmark runs
    print!(
        "{} {} iterations... ",
        label("Running:"),
        number(&iterations.to_string())
    );
    io::stdout().flush()?;

    let mut timings: Vec<f64> = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let mut result: Rc<Variable> = Rc::new(data.clone());
        let start = Instant::now();
        for expr in &compiled {
            result = expr.search(&result).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        let elapsed = start.elapsed();
        timings.push(elapsed.as_secs_f64() * 1000.0); // Convert to milliseconds
    }

    println!("done");
    println!();

    // Calculate statistics
    timings.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let total: f64 = timings.iter().sum();
    let mean = total / timings.len() as f64;
    #[allow(clippy::manual_is_multiple_of)] // is_multiple_of is unstable
    let median = if timings.len() % 2 == 0 {
        (timings[timings.len() / 2 - 1] + timings[timings.len() / 2]) / 2.0
    } else {
        timings[timings.len() / 2]
    };
    let min = timings.first().copied().unwrap_or(0.0);
    let max = timings.last().copied().unwrap_or(0.0);

    // Standard deviation
    let variance: f64 =
        timings.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / timings.len() as f64;
    let stddev = variance.sqrt();

    // Percentiles
    let p95_idx = ((timings.len() as f64 * 0.95) as usize).min(timings.len() - 1);
    let p99_idx = ((timings.len() as f64 * 0.99) as usize).min(timings.len() - 1);
    let p95 = timings[p95_idx];
    let p99 = timings[p99_idx];

    // Throughput (MB/s based on mean time)
    let throughput_mbs = if mean > 0.0 {
        (input_size as f64 / 1_000_000.0) / (mean / 1000.0)
    } else {
        0.0
    };

    // Print results
    println!("{}", heading("Results"));
    println!("{}", "─".repeat(40));
    println!(
        "  {:12} {}",
        label("Iterations:"),
        number(&format_with_commas(iterations as usize))
    );
    println!("  {:12} {}", label("Total time:"), format_duration(total));
    println!();
    println!(
        "  {:12} {}",
        label("Mean:"),
        highlight(&format_duration(mean))
    );
    println!("  {:12} {}", label("Median:"), format_duration(median));
    println!("  {:12} {}", label("Std dev:"), format_duration(stddev));
    println!();
    println!("  {:12} {}", label("Min:"), format_duration(min));
    println!("  {:12} {}", label("Max:"), format_duration(max));
    println!("  {:12} {}", label("p95:"), format_duration(p95));
    println!("  {:12} {}", label("p99:"), format_duration(p99));
    println!();
    println!("  {:12} {:.2} MB/s", label("Throughput:"), throughput_mbs);

    // Show histogram if enough samples
    if iterations >= 10 {
        println!();
        println!("{}", heading("Distribution"));
        println!("{}", "─".repeat(40));
        print_histogram(&timings, use_color);
    }

    println!();

    Ok(())
}

/// Format duration in appropriate units
fn format_duration(ms: f64) -> String {
    if ms < 0.001 {
        format!("{:.3} ns", ms * 1_000_000.0)
    } else if ms < 1.0 {
        format!("{:.3} µs", ms * 1000.0)
    } else if ms < 1000.0 {
        format!("{:.3} ms", ms)
    } else {
        format!("{:.3} s", ms / 1000.0)
    }
}

/// Print a simple ASCII histogram
fn print_histogram(timings: &[f64], use_color: bool) {
    const BUCKETS: usize = 10;
    const BAR_WIDTH: usize = 30;

    let min = timings.first().copied().unwrap_or(0.0);
    let max = timings.last().copied().unwrap_or(0.0);
    let range = max - min;

    if range == 0.0 {
        println!("  (all samples identical)");
        return;
    }

    let bucket_size = range / BUCKETS as f64;
    let mut buckets = [0usize; BUCKETS];

    for &t in timings {
        let idx = ((t - min) / bucket_size) as usize;
        let idx = idx.min(BUCKETS - 1);
        buckets[idx] += 1;
    }

    let max_count = *buckets.iter().max().unwrap_or(&1);

    for (i, &count) in buckets.iter().enumerate() {
        let lower = min + (i as f64 * bucket_size);
        let upper = lower + bucket_size;
        let bar_len = (count * BAR_WIDTH) / max_count.max(1);
        let bar: String = "█".repeat(bar_len);

        let bar_display = if use_color {
            bar.cyan().to_string()
        } else {
            bar
        };

        println!(
            "  {:>8} - {:>8} │{:<width$}│ {}",
            format!("{:.2}", lower),
            format!("{:.2}", upper),
            bar_display,
            count,
            width = BAR_WIDTH
        );
    }
}

// =============================================================================
// Streaming Processing
// =============================================================================

/// Run streaming mode - process input line by line (NDJSON/JSON Lines)
fn run_streaming(expressions: &[String], args: &Args) -> Result<()> {
    use std::io::{BufRead, BufWriter, Write};

    // Set up input reader
    let input: Box<dyn BufRead> = match &args.file {
        Some(path) => {
            let file = std::fs::File::open(path)
                .with_context(|| format!("Failed to open file: {}", path))?;
            Box::new(std::io::BufReader::new(file))
        }
        None => Box::new(std::io::BufReader::new(io::stdin())),
    };

    // Set up buffered output
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    // Create runtime with extensions (unless strict mode)
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    if !args.strict {
        register_all(&mut runtime);
    }

    // Compile expressions once
    let compiled: Vec<_> = expressions
        .iter()
        .map(|expr| {
            runtime
                .compile(expr)
                .with_context(|| format!("Failed to compile expression: {}", expr))
        })
        .collect::<Result<Vec<_>>>()?;

    let quiet = args.quiet;
    let raw = args.raw;
    let mut line_count = 0u64;

    for line in input.lines() {
        let line = line.context("Failed to read line")?;
        let trimmed = line.trim();
        line_count += 1;

        if trimmed.is_empty() {
            continue;
        }

        let data = match Variable::from_json(trimmed) {
            Ok(d) => d,
            Err(e) => {
                if !quiet {
                    eprintln!("jpx: Failed to parse JSON: {}", e);
                }
                continue;
            }
        };

        let mut result: Rc<Variable> = Rc::new(data);
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

        if result.is_null() {
            continue;
        }

        let output = if raw {
            if let Some(s) = result.as_string() {
                s.to_string()
            } else {
                serde_json::to_string(&*result)?
            }
        } else {
            serde_json::to_string(&*result)?
        };

        writeln!(writer, "{}", output)?;
    }

    if args.verbose {
        eprintln!("Processed {} lines", line_count);
    }

    writer.flush()?;
    Ok(())
}
