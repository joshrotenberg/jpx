//! MCP tool implementations for JMESPath
//!
//! This module wraps jpx_engine functionality in MCP tool handlers.

use jpx_engine::{
    Category, DiscoverySpec, EngineConfig, JpxEngine, ServerInfo as DiscoveryServerInfo, ToolSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tower_mcp::{BoxError, CallToolResult, Error, McpRouter, ToolBuilder};

// =============================================================================
// Parameter structs for MCP tools
// =============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateParams {
    /// JSON input to evaluate the expression against
    pub input: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FunctionsParams {
    /// Optional category filter (e.g., "String", "Math", "Array", "Datetime")
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DescribeParams {
    /// Function name or alias to describe
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// JMESPath expression to validate
    pub expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchEvaluateParams {
    /// JSON input to evaluate the expressions against
    pub input: String,
    /// List of JMESPath expressions to evaluate
    pub expressions: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatParams {
    /// JSON string to format
    pub input: String,
    /// Number of spaces for indentation (default: 2, use 0 for compact)
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffParams {
    /// Source JSON document
    pub source: String,
    /// Target JSON document
    pub target: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchParams {
    /// JSON document to patch
    pub input: String,
    /// JSON Patch operations array (RFC 6902)
    pub patch: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeParams {
    /// JSON document to merge into
    pub input: String,
    /// JSON Merge Patch document (RFC 7396)
    pub patch: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KeysParams {
    /// JSON document to extract keys from
    pub input: String,
    /// If true, recursively extract all keys using dot notation (default: false)
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateFileParams {
    /// Path to the JSON file to read
    pub file_path: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Search query to match against function names, descriptions, categories, or signatures
    pub query: String,
    /// Maximum number of results to return (default: 20)
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarParams {
    /// Function name to find similar functions for
    pub function: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SuggestFunctionParams {
    /// Natural-language description of what you want to do (e.g., "remove duplicate values from an array")
    pub task: String,
    /// Maximum number of suggestions to return (default: 5)
    #[serde(default = "default_suggest_limit")]
    pub limit: usize,
}

fn default_suggest_limit() -> usize {
    5
}

#[derive(Debug, Serialize)]
struct Suggestion {
    name: String,
    signature: String,
    description: String,
    example: String,
    relevance: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StatsParams {
    /// JSON input to analyze
    pub input: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PathsParams {
    /// JSON input to extract paths from
    pub input: String,
    /// Include type information for each path (default: true)
    #[serde(default = "default_true")]
    pub include_types: bool,
    /// Include values for leaf paths (default: false)
    #[serde(default)]
    pub include_values: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterToolsParams {
    /// Full discovery spec (use this for detailed registration with params, examples, etc.)
    #[serde(default)]
    pub spec: Option<DiscoverySpec>,
    /// Server name (simplified registration - use with `tools` field)
    #[serde(default)]
    pub server_name: Option<String>,
    /// Server version (simplified registration)
    #[serde(default)]
    pub version: Option<String>,
    /// Simple tool list (simplified registration - use with `server_name`)
    #[serde(default)]
    pub tools: Option<Vec<SimpleTool>>,
    /// Replace existing registration if server already registered (default: true)
    #[serde(default = "default_true")]
    pub replace: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EngineInfoParams {
    /// Include discovery JSON schema in the response
    #[serde(default)]
    pub include_schema: bool,
    /// Include discovery index statistics in the response
    #[serde(default)]
    pub include_index_stats: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QueryToolsParams {
    /// Search query (searches across name, description, tags, etc.)
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarToolsParams {
    /// Tool ID in format "server:tool_name"
    pub tool_id: String,
    /// Maximum number of similar tools to return (default: 5)
    #[serde(default = "default_similar_k")]
    pub top_k: usize,
}

fn default_similar_k() -> usize {
    5
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UnregisterDiscoveryParams {
    /// Server name to unregister
    pub server_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SimpleTool {
    /// Tool name (required)
    pub name: String,
    /// Tool description (optional but recommended)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags for categorization and search (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DefineQueryParams {
    /// Unique name for the query (used to reference it later)
    pub name: String,
    /// JMESPath expression
    pub expression: String,
    /// Optional description of what the query does
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetQueryParams {
    /// Name of the query to retrieve
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteQueryParams {
    /// Name of the query to delete
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunQueryParams {
    /// Name of the stored query to run
    pub name: String,
    /// JSON input to evaluate the query against
    pub input: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExplainParams {
    /// JMESPath expression to explain
    pub expression: String,
}

// Empty params for tools that take no input
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EmptyParams {}

// =============================================================================
// Helper functions
// =============================================================================

fn text_result(content: impl Into<String>) -> CallToolResult {
    CallToolResult::text(content)
}

fn json_result(value: &impl Serialize) -> Result<CallToolResult, Error> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| Error::tool(format!("Failed to serialize: {}", e)))?;
    Ok(text_result(json))
}

fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(message)
}

/// Strip natural-language filler from a task description to extract search terms.
fn clean_task_description(task: &str) -> String {
    let prefixes: &[&str] = &[
        "i want to ",
        "i need to ",
        "i'd like to ",
        "how do i ",
        "how to ",
        "how can i ",
        "is there a way to ",
        "can i ",
        "please ",
        "help me ",
        "i'm trying to ",
    ];
    let mut s = task.trim().to_lowercase();
    for prefix in prefixes {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }
    s.trim().to_string()
}

/// Convert a match_type from search_functions into a human-readable relevance note.
fn relevance_note(match_type: &str) -> String {
    match match_type {
        "exact_name" => "direct match by name".into(),
        "alias" => "matches a function alias".into(),
        "name_prefix" => "name starts with your search term".into(),
        "name_contains" => "name contains your search term".into(),
        "category" => "in a matching category".into(),
        "description" => "description matches your task".into(),
        "fuzzy_name" => "similar function name".into(),
        "synonym" => "matches via synonym expansion".into(),
        other => format!("related ({})", other),
    }
}

// =============================================================================
// Router builder
// =============================================================================

/// Build the MCP router with all jpx tools registered.
///
/// For simple use, pass `strict: true/false`. For full config support
/// (function filtering, query libraries, etc.), use [`build_router_from_config`].
#[allow(dead_code)]
pub fn build_router(strict: bool) -> Result<McpRouter, BoxError> {
    let mut config = EngineConfig::default();
    config.engine.strict = strict;
    build_router_from_config(config)
}

/// Build the MCP router from an [`EngineConfig`].
///
/// This applies function filtering, query libraries, and other settings
/// from the config. Use [`EngineConfig::discover`] to load from `jpx.toml`.
pub fn build_router_from_config(config: EngineConfig) -> Result<McpRouter, BoxError> {
    let engine = JpxEngine::from_config(config)
        .map_err(|e| -> BoxError { format!("Failed to build engine: {}", e).into() })?;
    let engine = Arc::new(engine);

    // -- evaluate
    let e = engine.clone();
    let evaluate = ToolBuilder::new("evaluate")
        .description("Evaluate a JMESPath expression against JSON input. Returns the result of applying the expression to the input data. Supports 400+ extended functions beyond standard JMESPath.")
        .read_only()
        .handler(move |params: EvaluateParams| {
            let engine = e.clone();
            async move {
                match engine.evaluate_str(&params.expression, &params.input) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- functions
    let e = engine.clone();
    let functions = ToolBuilder::new("functions")
        .description("List available JMESPath functions. Optionally filter by category (e.g., 'String', 'Math', 'Array', 'Datetime', 'Hash', 'Encoding', etc.). Returns function names with signatures and descriptions.")
        .read_only()
        .handler(move |params: FunctionsParams| {
            let engine = e.clone();
            async move {
                let functions = engine.functions(params.category.as_deref());
                json_result(&functions)
            }
        })
        .build();

    // -- describe
    let e = engine.clone();
    let describe = ToolBuilder::new("describe")
        .description("Get detailed information about a specific JMESPath function including its signature, description, example usage, and category. Accepts function name or alias.")
        .read_only()
        .handler(move |params: DescribeParams| {
            let engine = e.clone();
            async move {
                match engine.describe_function(&params.name) {
                    Some(detail) => json_result(&detail),
                    None => Ok(error_result(format!(
                        "Unknown function '{}'. Use the 'functions' tool to list available functions.",
                        params.name
                    ))),
                }
            }
        })
        .build();

    // -- categories
    let categories = ToolBuilder::new("categories")
        .description("List all available JMESPath function categories. Use these category names with the 'functions' tool to filter by category.")
        .read_only()
        .handler(move |_params: EmptyParams| {
            async move {
                let categories: Vec<String> = Category::all()
                    .iter()
                    .filter(|c| c.is_available())
                    .map(|c| c.name().to_string())
                    .collect();
                json_result(&categories)
            }
        })
        .build();

    // -- validate
    let e = engine.clone();
    let validate = ToolBuilder::new("validate")
        .description("Validate a JMESPath expression without executing it. Returns whether the expression is syntactically valid and any parse errors.")
        .read_only()
        .handler(move |params: ValidateParams| {
            let engine = e.clone();
            async move {
                let result = engine.validate(&params.expression);
                json_result(&result)
            }
        })
        .build();

    // -- explain
    let e = engine.clone();
    let explain = ToolBuilder::new("explain")
        .description("Explain a JMESPath expression by breaking it down into steps. Returns a structured breakdown of each part of the expression including node types, descriptions, functions used, and complexity rating. Useful for understanding complex queries or debugging unexpected results. Also works for invalid expressions (returns the parse error).")
        .read_only()
        .handler(move |params: ExplainParams| {
            let engine = e.clone();
            async move {
                match engine.explain(&params.expression) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- batch_evaluate
    let e = engine.clone();
    let batch_evaluate = ToolBuilder::new("batch_evaluate")
        .description("Evaluate multiple JMESPath expressions against the same JSON input in a single call. Parses the input once and runs all expressions, returning results for each. Useful for extracting multiple values from the same data.")
        .read_only()
        .handler(move |params: BatchEvaluateParams| {
            let engine = e.clone();
            async move {
                let input: Value = serde_json::from_str(&params.input)
                    .map_err(|e| Error::tool(format!("Invalid JSON: {}", e)))?;
                let result = engine.batch_evaluate(&params.expressions, &input);
                json_result(&result)
            }
        })
        .build();

    // -- format
    let e = engine.clone();
    let format = ToolBuilder::new("format")
        .description("Format and validate JSON. Pretty-prints the input with configurable indentation. Use indent=0 for compact output. Returns an error if the input is not valid JSON.")
        .read_only()
        .handler(move |params: FormatParams| {
            let engine = e.clone();
            async move {
                match engine.format_json(&params.input, params.indent) {
                    Ok(formatted) => Ok(text_result(formatted)),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- diff
    let e = engine.clone();
    let diff = ToolBuilder::new("diff")
        .description("Generate a JSON Patch (RFC 6902) that transforms the source document into the target document. Returns an array of patch operations (add, remove, replace, move, copy, test). See https://datatracker.ietf.org/doc/html/rfc6902")
        .read_only()
        .handler(move |params: DiffParams| {
            let engine = e.clone();
            async move {
                match engine.diff(&params.source, &params.target) {
                    Ok(patch) => json_result(&patch),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- patch
    let e = engine.clone();
    let patch = ToolBuilder::new("patch")
        .description("Apply a JSON Patch (RFC 6902) to a JSON document. The patch is an array of operations (add, remove, replace, move, copy, test). Returns the patched document or an error if the patch cannot be applied. See https://datatracker.ietf.org/doc/html/rfc6902")
        .handler(move |params: PatchParams| {
            let engine = e.clone();
            async move {
                match engine.patch(&params.input, &params.patch) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- merge
    let e = engine.clone();
    let merge = ToolBuilder::new("merge")
        .description("Apply a JSON Merge Patch (RFC 7396) to a JSON document. The merge patch is a JSON document that describes changes: values are replaced, null values remove keys, and objects are merged recursively. Simpler than JSON Patch but less expressive. See https://datatracker.ietf.org/doc/html/rfc7396")
        .handler(move |params: MergeParams| {
            let engine = e.clone();
            async move {
                match engine.merge(&params.input, &params.patch) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- keys
    let e = engine.clone();
    let keys = ToolBuilder::new("keys")
        .description("Extract keys from a JSON object. By default returns top-level keys only. Set recursive=true to get all nested keys in dot notation (e.g., 'user.profile.age'). Useful for understanding JSON structure before querying.")
        .read_only()
        .handler(move |params: KeysParams| {
            let engine = e.clone();
            async move {
                match engine.keys(&params.input, params.recursive) {
                    Ok(keys) => json_result(&keys),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- evaluate_file
    let e = engine.clone();
    let evaluate_file = ToolBuilder::new("evaluate_file")
        .description("Read a JSON file from disk and evaluate a JMESPath expression against it. More efficient than passing large JSON content through the protocol. The file must exist and contain valid JSON.")
        .read_only()
        .handler(move |params: EvaluateFileParams| {
            let engine = e.clone();
            async move {
                use std::path::Path;

                let path = Path::new(&params.file_path);

                // Security validations
                if !path.is_absolute() {
                    return Err(Error::tool("File path must be absolute"));
                }

                let canonical_path = path
                    .canonicalize()
                    .map_err(|e| Error::tool(format!("Cannot resolve path: {}", e)))?;

                if !canonical_path.is_file() {
                    return Err(Error::tool(format!(
                        "Not a file: {}",
                        canonical_path.display()
                    )));
                }

                let metadata = std::fs::metadata(&canonical_path)
                    .map_err(|e| Error::tool(format!("Cannot read file metadata: {}", e)))?;

                const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(Error::tool(format!(
                        "File too large: {} bytes (max {} bytes)",
                        metadata.len(),
                        MAX_FILE_SIZE
                    )));
                }

                let content = std::fs::read_to_string(&canonical_path)
                    .map_err(|e| Error::tool(format!("Cannot read file: {}", e)))?;

                match engine.evaluate_str(&params.expression, &content) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- search
    let e = engine.clone();
    let search = ToolBuilder::new("search")
        .description("Search for JMESPath functions using fuzzy matching. Searches function names, descriptions, categories, signatures, and aliases. Returns ranked results with match type and relevance score. Essential for discovering functions when you're not sure of the exact name.")
        .read_only()
        .handler(move |params: SearchParams| {
            let engine = e.clone();
            async move {
                let results = engine.search_functions(&params.query, params.limit);
                json_result(&results)
            }
        })
        .build();

    // -- similar
    let e = engine.clone();
    let similar = ToolBuilder::new("similar")
        .description("Find functions similar to a specified function. Returns functions in the same category, functions with similar signatures (same input/output types), and functions with related concepts based on description keywords. Useful for discovering alternative approaches.")
        .read_only()
        .handler(move |params: SimilarParams| {
            let engine = e.clone();
            async move {
                match engine.similar_functions(&params.function) {
                    Some(result) => json_result(&result),
                    None => Ok(error_result(format!(
                        "Unknown function '{}'. Use the 'search' tool to find functions.",
                        params.function
                    ))),
                }
            }
        })
        .build();

    // -- suggest_function
    let e = engine.clone();
    let suggest_function = ToolBuilder::new("suggest_function")
        .description(
            "Suggest JMESPath functions for a task described in natural language. \
            Accepts a plain-English description of what you want to accomplish \
            (e.g., 'remove duplicate values from an array', 'convert to uppercase', \
            'find the maximum value') and returns ranked function suggestions with \
            relevance explanations.",
        )
        .read_only()
        .handler(move |params: SuggestFunctionParams| {
            let engine = e.clone();
            async move {
                let query = clean_task_description(&params.task);
                if query.is_empty() {
                    return Ok(error_result(
                        "Please describe what you want to do (e.g., 'remove duplicates from an array').",
                    ));
                }

                let results = engine.search_functions(&query, params.limit);
                let suggestions: Vec<Suggestion> = results
                    .into_iter()
                    .map(|r| Suggestion {
                        name: r.function.name,
                        signature: r.function.signature,
                        description: r.function.description,
                        example: r.function.example,
                        relevance: relevance_note(&r.match_type),
                    })
                    .collect();

                json_result(&suggestions)
            }
        })
        .build();

    // -- stats
    let e = engine.clone();
    let stats = ToolBuilder::new("stats")
        .description("Analyze JSON data and return statistics including type, size, depth, field analysis for arrays of objects, and type distribution. Useful for understanding data structure before writing queries.")
        .read_only()
        .handler(move |params: StatsParams| {
            let engine = e.clone();
            async move {
                match engine.stats(&params.input) {
                    Ok(stats) => json_result(&stats),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- paths
    let e = engine.clone();
    let paths = ToolBuilder::new("paths")
        .description("Extract all paths from JSON data in dot notation (e.g., 'users.0.name'). Optionally includes type information and values. Essential for understanding complex JSON structure before writing JMESPath queries.")
        .read_only()
        .handler(move |params: PathsParams| {
            let engine = e.clone();
            async move {
                match engine.paths(&params.input, params.include_types, params.include_values) {
                    Ok(paths) => json_result(&paths),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // =========================================================================
    // Discovery tools
    // =========================================================================

    // -- register_tools
    let e = engine.clone();
    let register_tools = ToolBuilder::new("register_tools")
        .description("Register an MCP server's tools for cross-server discovery. Accepts either a full discovery spec (via 'spec') or a simplified format (via 'server_name' + 'tools'). Tools are indexed for full-text search across name, description, tags, and parameters.")
        .handler(move |params: RegisterToolsParams| {
            let engine = e.clone();
            async move {
                let spec = if let Some(spec) = params.spec {
                    // Full spec provided
                    if spec.server.name.is_empty() {
                        return Err(Error::tool(
                            "server.name is required and cannot be empty",
                        ));
                    }
                    spec
                } else if let Some(server_name) = params.server_name {
                    // Simplified format
                    if server_name.is_empty() {
                        return Err(Error::tool(
                            "server_name is required and cannot be empty",
                        ));
                    }
                    let tools: Vec<ToolSpec> = params
                        .tools
                        .unwrap_or_default()
                        .into_iter()
                        .map(|t| ToolSpec {
                            name: t.name,
                            aliases: vec![],
                            category: None,
                            subcategory: None,
                            tags: t.tags,
                            summary: t.description.clone(),
                            description: t.description,
                            params: vec![],
                            returns: None,
                            examples: vec![],
                            related: vec![],
                            since: None,
                            stability: None,
                        })
                        .collect();

                    DiscoverySpec {
                        schema: None,
                        server: DiscoveryServerInfo {
                            name: server_name,
                            version: params.version,
                            description: None,
                        },
                        tools,
                        categories: std::collections::HashMap::new(),
                    }
                } else {
                    return Err(Error::tool(
                        "Provide either 'spec' (full format) or 'server_name' + 'tools' (simplified format)",
                    ));
                };

                match engine.register_discovery(spec, params.replace) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- query_tools
    let e = engine.clone();
    let query_tools = ToolBuilder::new("query_tools")
        .description("Search for tools across all registered MCP servers. Uses BM25 full-text search to find relevant tools by name, description, tags, category, or parameters. Returns ranked results with match scores.")
        .read_only()
        .handler(move |params: QueryToolsParams| {
            let engine = e.clone();
            async move {
                match engine.query_tools(&params.query, params.top_k) {
                    Ok(results) => json_result(&results),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- similar_tools
    let e = engine.clone();
    let similar_tools = ToolBuilder::new("similar_tools")
        .description("Find tools similar to a specified tool based on shared terms and concepts. Uses the tool's indexed content to find related tools across all registered servers.")
        .read_only()
        .handler(move |params: SimilarToolsParams| {
            let engine = e.clone();
            async move {
                match engine.similar_tools(&params.tool_id, params.top_k) {
                    Ok(results) => json_result(&results),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- unregister_discovery
    let e = engine.clone();
    let unregister_discovery = ToolBuilder::new("unregister_discovery")
        .description("Remove an MCP server's tools from the discovery index. Use this when a server is no longer available or to re-register with updated tools.")
        .handler(move |params: UnregisterDiscoveryParams| {
            let engine = e.clone();
            async move {
                match engine.unregister_discovery(&params.server_name) {
                    Ok(true) => {
                        json_result(&serde_json::json!({"ok": true, "message": "Server unregistered"}))
                    }
                    Ok(false) => Ok(error_result(format!(
                        "Server '{}' not found",
                        params.server_name
                    ))),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- list_discovery_servers
    let e = engine.clone();
    let list_discovery_servers = ToolBuilder::new("list_discovery_servers")
        .description("List all MCP servers that have registered their tools for discovery. Returns server names, versions, descriptions, and tool counts.")
        .read_only()
        .handler(move |_params: EmptyParams| {
            let engine = e.clone();
            async move {
                match engine.list_discovery_servers() {
                    Ok(servers) => json_result(&servers),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- list_discovery_categories
    let e = engine.clone();
    let list_discovery_categories = ToolBuilder::new("list_discovery_categories")
        .description("List all tool categories from registered MCP servers. Returns category names with tool counts and which servers provide tools in each category.")
        .read_only()
        .handler(move |_params: EmptyParams| {
            let engine = e.clone();
            async move {
                match engine.list_discovery_categories() {
                    Ok(categories) => json_result(&categories),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // =========================================================================
    // Query store tools
    // =========================================================================

    // -- define_query
    let e = engine.clone();
    let define_query = ToolBuilder::new("define_query")
        .description("Store a named JMESPath query for reuse during this session. Useful for building and refining complex queries iteratively. The query is validated before storing.")
        .handler(move |params: DefineQueryParams| {
            let engine = e.clone();
            async move {
                match engine.define_query(params.name.clone(), params.expression, params.description) {
                    Ok(prev) => {
                        let msg = if prev.is_some() {
                            format!("Query '{}' updated", params.name)
                        } else {
                            format!("Query '{}' defined", params.name)
                        };
                        json_result(&serde_json::json!({"ok": true, "message": msg}))
                    }
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- get_query
    let e = engine.clone();
    let get_query = ToolBuilder::new("get_query")
        .description(
            "Retrieve a stored query by name. Returns the expression and description if found.",
        )
        .read_only()
        .handler(move |params: GetQueryParams| {
            let engine = e.clone();
            async move {
                match engine.get_query(&params.name) {
                    Ok(Some(query)) => json_result(&query),
                    Ok(None) => Ok(error_result(format!("Query '{}' not found", params.name))),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- delete_query
    let e = engine.clone();
    let delete_query = ToolBuilder::new("delete_query")
        .description("Delete a stored query by name. Returns the deleted query if it existed.")
        .handler(move |params: DeleteQueryParams| {
            let engine = e.clone();
            async move {
                match engine.delete_query(&params.name) {
                    Ok(Some(query)) => json_result(&serde_json::json!({
                        "ok": true,
                        "deleted": query
                    })),
                    Ok(None) => Ok(error_result(format!("Query '{}' not found", params.name))),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- list_queries
    let e = engine.clone();
    let list_queries = ToolBuilder::new("list_queries")
        .description("List all named queries stored in this session. Shows query names, expressions, and descriptions.")
        .read_only()
        .handler(move |_params: EmptyParams| {
            let engine = e.clone();
            async move {
                match engine.list_queries() {
                    Ok(queries) => json_result(&queries),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // -- run_query
    let e = engine.clone();
    let run_query = ToolBuilder::new("run_query")
        .description("Execute a stored query by name against JSON input. Combines the convenience of named queries with evaluation.")
        .read_only()
        .handler(move |params: RunQueryParams| {
            let engine = e.clone();
            async move {
                let input: Value = serde_json::from_str(&params.input)
                    .map_err(|e| Error::tool(format!("Invalid JSON: {}", e)))?;

                match engine.run_query(&params.name, &input) {
                    Ok(result) => json_result(&result),
                    Err(e) => Err(Error::tool(e.to_string())),
                }
            }
        })
        .build();

    // =========================================================================
    // Engine info
    // =========================================================================

    // -- engine_info
    let e = engine.clone();
    let engine_info = ToolBuilder::new("engine_info")
        .description("Get information about the jpx engine including version, mode, function count, and current session state. Optionally include discovery schema (include_schema) and/or index statistics (include_index_stats).")
        .read_only()
        .handler(move |params: EngineInfoParams| {
            let engine = e.clone();
            async move {
                let function_count = engine.functions(None).len();
                let category_count = engine.categories().len();
                let stored_queries = engine.list_queries().unwrap_or_default().len();
                let discovery_servers = engine.list_discovery_servers().unwrap_or_default().len();

                let mut info = serde_json::json!({
                    "name": "jpx-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                    "strict_mode": engine.is_strict(),
                    "let_expressions": cfg!(feature = "let-expr") && !engine.is_strict(),
                    "function_count": function_count,
                    "category_count": category_count,
                    "stored_queries": stored_queries,
                    "registered_discovery_servers": discovery_servers
                });

                if params.include_schema {
                    info["discovery_schema"] = engine.get_discovery_schema();
                }

                if params.include_index_stats {
                    info["index_stats"] = match engine.discovery_index_stats() {
                        Ok(Some(stats)) => serde_json::to_value(stats).unwrap_or_default(),
                        Ok(None) => serde_json::json!({"message": "No tools indexed yet"}),
                        Err(e) => serde_json::json!({"error": e.to_string()}),
                    };
                }

                json_result(&info)
            }
        })
        .build();

    // =========================================================================
    // Build the router
    // =========================================================================

    let router = McpRouter::new()
        .server_info("jpx-mcp", env!("CARGO_PKG_VERSION"))
        .instructions(
            "JMESPath query tool with 400+ extended functions. \
            \n\nDISCOVERY: Use 'search' to find functions by keyword, 'similar' to find related functions, \
            'functions' to list all (optionally by category), 'describe' for function details, 'categories' to list categories. \
            \n\nDATA ANALYSIS: Use 'stats' to analyze JSON structure before querying, 'paths' to list all paths in dot notation, \
            'keys' to extract object keys (optionally recursive). \
            \n\nQUERYING: Use 'evaluate' to run JMESPath queries, 'evaluate_file' to query JSON files directly, \
            'batch_evaluate' for multiple expressions against the same input, 'validate' to check expression syntax, \
            'explain' to get a step-by-step breakdown of what an expression does. \
            \n\nJSON UTILITIES: Use 'format' to pretty-print JSON, 'diff' to generate RFC 6902 JSON Patches, \
            'patch' to apply RFC 6902 patches, 'merge' to apply RFC 7396 JSON Merge Patches. \
            \n\nLET EXPRESSIONS: JEP-18 let expressions are supported for variable bindings: \
            'let $var = expr in body'. Use for naming intermediate results and simplifying complex queries."
        )
        .tool(evaluate)
        .tool(functions)
        .tool(describe)
        .tool(categories)
        .tool(validate)
        .tool(explain)
        .tool(batch_evaluate)
        .tool(format)
        .tool(diff)
        .tool(patch)
        .tool(merge)
        .tool(keys)
        .tool(evaluate_file)
        .tool(search)
        .tool(similar)
        .tool(suggest_function)
        .tool(stats)
        .tool(paths)
        .tool(register_tools)
        .tool(query_tools)
        .tool(similar_tools)
        .tool(unregister_discovery)
        .tool(list_discovery_servers)
        .tool(list_discovery_categories)
        .tool(define_query)
        .tool(get_query)
        .tool(delete_query)
        .tool(list_queries)
        .tool(run_query)
        .tool(engine_info);

    Ok(router)
}
