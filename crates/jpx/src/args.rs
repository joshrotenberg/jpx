use clap::{ArgAction, Parser, ValueEnum, builder::styling};
use colored::Colorize;
use serde::Deserialize;
use std::path::PathBuf;

// Cargo-style help coloring
pub(crate) const STYLES: styling::Styles = styling::Styles::styled()
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
    #[allow(dead_code)]
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

/// Load configuration from file
pub(crate) fn load_config() -> impl ConfigDefaults {
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

pub(crate) trait ConfigDefaults {
    fn apply_defaults(&self, args: &mut Args);
}

impl ConfigDefaults for Config {
    fn apply_defaults(&self, args: &mut Args) {
        if !args.verbose && self.verbose == Some(true) {
            args.verbose = true;
        }
        if !args.quiet && self.quiet == Some(true) {
            args.quiet = true;
        }
        if !args.strict && self.strict == Some(true) {
            args.strict = true;
        }
        if !args.raw && self.raw == Some(true) {
            args.raw = true;
        }
        if !args.compact && self.compact == Some(true) {
            args.compact = true;
        }
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
pub(crate) fn apply_env_defaults(args: &mut Args) {
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
pub(crate) enum ColorMode {
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
pub(crate) struct Args {
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
    pub(crate) expressions: Vec<String>,

    /// Expression(s) as positional args
    #[arg(conflicts_with_all = ["query_file", "expressions"])]
    pub(crate) positional_expressions: Vec<String>,

    /// Read expression from file
    #[arg(short = 'Q', long = "query-file", conflicts_with_all = ["positional_expressions", "expressions"],
          help = "Read expression from file or query library (.jpx)",
          long_help = "Read JMESPath expression from a file. Supports:\n\
            - Plain text files with a single expression\n\
            - Query libraries (.jpx) with named queries\n\n\
            For .jpx files, use colon syntax or --query:\n  \
            jpx -Q queries.jpx:my-query data.json\n  \
            jpx -Q queries.jpx --query my-query data.json")]
    pub(crate) query_file: Option<String>,

    /// Named query to run from a .jpx query library
    #[arg(
        long = "query",
        value_name = "NAME",
        help = "Named query to run from .jpx file",
        long_help = "Specify which named query to run from a .jpx query library.\n\
            Can also use colon syntax: -Q file.jpx:query-name"
    )]
    pub(crate) query_name: Option<String>,

    /// List available queries in a .jpx file
    #[arg(
        long = "list-queries",
        help = "List queries in a .jpx file",
        long_help = "List all named queries available in a .jpx query library file."
    )]
    pub(crate) list_queries: bool,

    /// Validate queries without running
    #[arg(
        long = "check",
        help = "Validate query file without running",
        long_help = "Parse and validate all queries in a .jpx file without executing.\n\
            Useful for CI/CD pipelines. Exit code 0 if all valid, 1 if errors."
    )]
    pub(crate) check_queries: bool,

    /// Bind a string variable for use in expressions
    #[arg(
        long = "arg",
        value_name = "NAME VALUE",
        num_args = 2,
        action = clap::ArgAction::Append,
        help = "Bind a string variable: --arg name value",
        long_help = "Bind a string variable for use in expressions as $name.\nThe value is always treated as a string.\n\n\
            Example: jpx --arg name Alice '[?name == $name]' data.json"
    )]
    pub(crate) arg: Vec<String>,

    /// Bind a JSON variable for use in expressions
    #[arg(
        long = "argjson",
        value_name = "NAME VALUE",
        num_args = 2,
        action = clap::ArgAction::Append,
        help = "Bind a JSON variable: --argjson name '{\"key\":1}'",
        long_help = "Bind a JSON variable for use in expressions as $name.\nThe value is parsed as JSON (numbers, booleans, arrays, objects, etc.).\n\n\
            Example: jpx --argjson threshold 42 '[?score > $threshold]' data.json"
    )]
    pub(crate) argjson: Vec<String>,

    /// Input JSON file
    #[arg(
        short,
        long,
        help = "Input JSON file",
        long_help = "Input file to read JSON from. If not provided, reads from stdin.\nSupports any valid JSON file.\n\n\
            The file can also be passed as a trailing positional argument:\n  \
            jpx 'users[*].name' data.json\n\n\
            When the last positional argument is an existing file, it is used as input\n\
            instead of being treated as an expression."
    )]
    pub(crate) file: Option<String>,

    /// Output raw strings without quotes
    #[arg(
        short = 'r',
        long,
        help = "Output raw strings without quotes",
        long_help = "Output raw strings without JSON quotes. Useful for piping string results\nto other commands. Can also be set with JPX_RAW=1 environment variable."
    )]
    pub(crate) raw: bool,

    /// Like --raw but without trailing newlines
    #[arg(
        short = 'j',
        long = "join-output",
        help = "Raw output without trailing newlines",
        long_help = "Join output mode - like --raw but without a trailing newline after each output.\nUseful for building shell variables, URL-safe strings, or output for xargs -0."
    )]
    pub(crate) join_output: bool,

    /// Compact JSON output
    #[arg(
        short,
        long,
        help = "Compact JSON output (no pretty-printing)",
        long_help = "Compact output without pretty-printing or indentation.\nCan also be set with JPX_COMPACT=1 environment variable."
    )]
    pub(crate) compact: bool,

    /// Number of spaces for indentation
    #[arg(
        long,
        value_name = "N",
        conflicts_with_all = ["compact", "tab"],
        value_parser = clap::value_parser!(u8).range(0..=8),
        help = "Set indentation to N spaces (0-8)",
        long_help = "Set the number of spaces for indentation in pretty-printed output.\nValid range: 0-8. Default is 2 spaces. Use 0 for no indentation\n(but still with newlines, unlike --compact)."
    )]
    pub(crate) indent: Option<u8>,

    /// Use tabs for indentation
    #[arg(
        long,
        conflicts_with_all = ["compact", "indent"],
        help = "Use tab characters for indentation"
    )]
    pub(crate) tab: bool,

    /// Sort object keys in output
    #[arg(
        short = 'S',
        long = "sort-keys",
        help = "Sort object keys alphabetically in output"
    )]
    pub(crate) sort_keys: bool,

    /// Output as YAML
    #[arg(short = 'y', long, conflicts_with_all = ["toml_output", "csv_output", "tsv_output", "lines_output"])]
    pub(crate) yaml: bool,

    /// Output as TOML
    #[arg(long = "toml", conflicts_with_all = ["yaml", "csv_output", "tsv_output", "lines_output"])]
    pub(crate) toml_output: bool,

    /// Output as CSV
    #[arg(long = "csv", conflicts_with_all = ["yaml", "toml_output", "tsv_output", "lines_output"],
          help = "Output as CSV",
          long_help = "Output as CSV (comma-separated values). Best for arrays of objects.\nNested structures are flattened with dot notation.")]
    pub(crate) csv_output: bool,

    /// Output as TSV
    #[arg(long = "tsv", conflicts_with_all = ["yaml", "toml_output", "csv_output", "lines_output"],
          help = "Output as TSV",
          long_help = "Output as TSV (tab-separated values). Best for arrays of objects.\nNested structures are flattened with dot notation.")]
    pub(crate) tsv_output: bool,

    /// Output one value per line
    #[arg(short = 'l', long = "lines", conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "table"],
          help = "Output one JSON value per line",
          long_help = "Output one JSON value per line (NDJSON/JSON Lines format).\nUseful for streaming or piping array results to other tools.")]
    pub(crate) lines_output: bool,

    /// Output as formatted table
    #[arg(short = 't', long, conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "lines_output"],
          help = "Output as formatted table",
          long_help = "Output as a formatted table. Best for arrays of objects.\nUse --table-style to change appearance (unicode, ascii, markdown, plain).")]
    pub(crate) table: bool,

    /// Table style
    #[arg(
        long,
        value_name = "STYLE",
        default_value = "unicode",
        requires = "table",
        help = "Table style: unicode, ascii, markdown, plain",
        long_help = "Table style for --table output:\n  unicode   Box-drawing characters (default)\n  ascii     ASCII characters only\n  markdown  GitHub-flavored markdown\n  plain     No borders"
    )]
    pub(crate) table_style: String,

    /// Output as Parquet file (requires --output)
    #[cfg(feature = "parquet")]
    #[arg(
        long = "parquet",
        conflicts_with_all = ["yaml", "toml_output", "csv_output", "tsv_output", "lines_output", "table", "stream"],
        requires = "output",
        help = "Output as Parquet file",
        long_help = "Output as Parquet file. Requires --output to specify the file path.\nBest for arrays of objects. Uses Snappy compression."
    )]
    pub(crate) parquet_output: bool,

    /// Use null as input
    #[arg(
        short = 'n',
        long,
        help = "Use null as input (don't read stdin)",
        long_help = "Null input mode - use null as the input value instead of reading from stdin.\nUseful for expressions that don't need input, like: jpx -n 'now()'"
    )]
    pub(crate) null_input: bool,

    /// Slurp multiple inputs into array
    #[arg(
        short = 's',
        long,
        conflicts_with = "stream",
        help = "Slurp multiple JSON values into array",
        long_help = "Slurp mode - read multiple JSON values from input and combine them into\na single array. Useful for processing multiple JSON objects."
    )]
    pub(crate) slurp: bool,

    /// Read input as raw text lines, not JSON
    #[arg(
        short = 'R',
        long = "raw-input",
        conflicts_with_all = ["stream", "null_input"],
        help = "Read input as raw text lines, not JSON",
        long_help = "Raw input mode - read each line as a JSON string rather than parsing as JSON.\nCombined with --slurp, reads the entire input as an array of strings.\nUseful for processing log files, CSV data, and other non-JSON text."
    )]
    pub(crate) raw_input: bool,

    /// Process input line by line
    #[arg(long, visible_alias = "each", conflicts_with_all = ["slurp", "null_input", "table", "yaml", "toml_output", "lines_output"],
          help = "Process input line by line (NDJSON)",
          long_help = "Stream mode - process input line by line (for NDJSON/JSON Lines).\nEach line is parsed and evaluated independently with constant memory usage.\nSupports --csv and --tsv output (headers derived from first result).")]
    pub(crate) stream: bool,

    /// Flush output after each record in streaming mode
    #[arg(long, help = "Flush output after each record in streaming mode")]
    pub(crate) unbuffered: bool,

    /// Color output mode
    #[arg(long, value_enum, default_value = "auto")]
    pub(crate) color: ColorMode,

    /// Output file
    #[arg(
        short = 'o',
        long,
        help = "Write output to file",
        long_help = "Write output to a file instead of stdout."
    )]
    pub(crate) output: Option<String>,

    /// Set exit status based on result
    #[arg(
        short = 'x',
        long = "exit-status",
        help = "Exit with 1 if result is false or null",
        long_help = "Exit status mode - set the process exit code based on the result value.\nExits with 0 if the result is truthy (non-null, non-false),\nexits with 1 if the result is false or null.\nUseful for shell scripting conditionals."
    )]
    pub(crate) exit_status: bool,

    /// Suppress errors and warnings
    #[arg(
        short = 'q',
        long,
        help = "Suppress errors and warnings",
        long_help = "Quiet mode - suppress errors and warnings.\nCan also be set with JPX_QUIET=1 environment variable."
    )]
    pub(crate) quiet: bool,

    /// Show expression details and timing
    #[arg(
        short = 'v',
        long,
        help = "Show expression details and timing",
        long_help = "Verbose mode - show expression details, input info, and timing.\nCan also be set with JPX_VERBOSE=1 environment variable."
    )]
    pub(crate) verbose: bool,

    /// Disable extension functions
    #[arg(
        long,
        help = "Use only standard JMESPath functions",
        long_help = "Strict mode - only use standard JMESPath functions (no extensions).\nCan also be set with JPX_STRICT=1 environment variable."
    )]
    pub(crate) strict: bool,

    /// Generate shell completions
    #[arg(
        long,
        value_name = "SHELL",
        help = "Generate shell completions",
        long_help = "Generate shell completion script for the specified shell.\nSupported: bash, zsh, fish, powershell, elvish"
    )]
    pub(crate) completions: Option<clap_complete::Shell>,

    /// List all functions
    #[arg(
        long,
        help = "List all available functions",
        long_help = "List all available extension functions organized by category.\nUse --list-category for a specific category or --describe for details."
    )]
    pub(crate) list_functions: bool,

    /// List functions in category
    #[arg(
        long,
        value_name = "CATEGORY",
        help = "List functions in a category",
        long_help = "List functions in a specific category. Categories include:\narray, datetime, math, string, hash, encoding, object, and more."
    )]
    pub(crate) list_category: Option<String>,

    /// Show function details
    #[arg(
        long,
        value_name = "FUNCTION",
        help = "Show detailed info for a function",
        long_help = "Show detailed information about a specific function including\nsignature, description, and usage examples."
    )]
    pub(crate) describe: Option<String>,

    /// Search for functions
    #[arg(
        long,
        value_name = "QUERY",
        help = "Search functions by name or description",
        long_help = "Search functions by name, description, or category using fuzzy matching.\nResults are ranked by relevance."
    )]
    pub(crate) search: Option<String>,

    /// Find similar functions
    #[arg(
        long,
        value_name = "FUNCTION",
        help = "Find functions similar to another",
        long_help = "Find functions similar to the specified function based on category,\nsignature, and description keywords."
    )]
    pub(crate) similar: Option<String>,

    /// Generate JSON Patch from two files
    #[arg(long, num_args = 2, value_names = ["SOURCE", "TARGET"],
          help = "Generate JSON Patch from two files",
          long_help = "Generate JSON Patch (RFC 6902) that transforms SOURCE into TARGET.\nOutput can be used with --patch to apply changes.")]
    pub(crate) diff: Option<Vec<String>>,

    /// Apply JSON Patch
    #[arg(
        long,
        value_name = "PATCH_FILE",
        help = "Apply JSON Patch (RFC 6902)",
        long_help = "Apply JSON Patch (RFC 6902) to a document.\nReads document from -f or stdin, applies patch operations from file."
    )]
    pub(crate) patch: Option<String>,

    /// Apply JSON Merge Patch
    #[arg(
        long,
        value_name = "MERGE_FILE",
        help = "Apply JSON Merge Patch (RFC 7396)",
        long_help = "Apply JSON Merge Patch (RFC 7396) to a document.\nReads document from -f or stdin, merges with patch from file."
    )]
    pub(crate) merge: Option<String>,

    /// Show expression AST
    #[arg(
        long,
        help = "Show how expression is parsed (AST)",
        long_help = "Explain how an expression is parsed by showing the Abstract Syntax Tree.\nUseful for understanding complex expressions and debugging."
    )]
    pub(crate) explain: bool,

    /// Show diagnostic info
    #[arg(
        long,
        help = "Show diagnostic information",
        long_help = "Show diagnostic information for troubleshooting including\nenvironment variables, effective settings, and input info."
    )]
    pub(crate) debug: bool,

    /// Start interactive REPL
    #[arg(
        long,
        help = "Start interactive REPL mode",
        long_help = "Start interactive REPL (Read-Eval-Print Loop) mode.\nSupports history, tab completion, and multi-line editing."
    )]
    pub(crate) repl: bool,

    /// Load demo dataset
    #[arg(
        long,
        value_name = "NAME",
        help = "Load a demo dataset for REPL",
        long_help = "Load a demo dataset when starting REPL mode.\nAvailable: users, products, github, mixed"
    )]
    pub(crate) demo: Option<String>,

    /// Show input statistics
    #[arg(
        long,
        help = "Show statistics about input data",
        long_help = "Show statistics about the input JSON data including type, size,\ndepth, and field analysis for arrays of objects."
    )]
    pub(crate) stats: bool,

    /// List all JSON paths
    #[arg(
        long,
        help = "List all paths in input JSON",
        long_help = "List all paths in the input JSON document.\nUse --types and --values for additional details."
    )]
    pub(crate) paths: bool,

    /// Show types with paths
    #[arg(long, requires = "paths")]
    pub(crate) types: bool,

    /// Show values with paths
    #[arg(long, requires = "paths")]
    pub(crate) values: bool,

    /// Benchmark expression
    #[arg(long, value_name = "ITERATIONS", default_missing_value = "100", num_args = 0..=1,
          help = "Benchmark expression performance",
          long_help = "Benchmark expression performance over multiple iterations.\nShows statistics including mean, median, p95, p99, and throughput.")]
    pub(crate) bench: Option<u32>,

    /// Warmup iterations for benchmark
    #[arg(
        long,
        value_name = "COUNT",
        default_value = "5",
        requires = "bench",
        help = "Warmup iterations before benchmarking"
    )]
    pub(crate) warmup: u32,
}

/// Create a configured runtime and registry from engine config.
/// If strict is true, extension functions are not applied to the runtime.
pub(crate) fn create_configured_runtime(
    engine_config: &jpx_engine::config::EngineConfig,
    strict: bool,
) -> (jpx_engine::Runtime, jpx_engine::FunctionRegistry) {
    let effective_strict = strict || engine_config.engine.strict;
    jpx_engine::config::build_runtime_from_config(&engine_config.functions, effective_strict)
}
