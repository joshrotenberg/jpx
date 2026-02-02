//! # jpx-engine
//!
//! Protocol-agnostic JMESPath query engine with 400+ functions.
//!
//! This crate provides the core "brain" of jpx - everything you can do with JMESPath
//! beyond basic compile and evaluate. It's designed to be transport-agnostic, allowing
//! the CLI (`jpx`), MCP server (`jpx-server`), or any future REST/gRPC adapters to be
//! thin wrappers over this engine.
//!
//! ## Features
//!
//! | Category | Description |
//! |----------|-------------|
//! | **Evaluation** | Single, batch, and string-based evaluation with validation |
//! | **Introspection** | List functions, search by keyword, describe, find similar |
//! | **Discovery** | Cross-server tool discovery with BM25 search indexing |
//! | **Query Store** | Named queries for session-scoped reuse |
//! | **JSON Utilities** | Format, diff, patch, merge, stats, paths, keys |
//! | **Arrow** | Apache Arrow conversion (optional, via `arrow` feature) |
//!
//! ## Cargo Features
//!
//! - **`arrow`** - Enables Apache Arrow support for columnar data conversion.
//!   This adds the [`arrow`] module with functions to convert between Arrow
//!   RecordBatches and JSON Values. Used by the CLI for Parquet I/O.
//!
//! ## Quick Start
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//! use serde_json::json;
//!
//! let engine = JpxEngine::new();
//!
//! // Evaluate a JMESPath expression
//! let result = engine.evaluate("users[*].name", &json!({
//!     "users": [{"name": "alice"}, {"name": "bob"}]
//! })).unwrap();
//! assert_eq!(result, json!(["alice", "bob"]));
//! ```
//!
//! ## Evaluation
//!
//! The engine supports multiple evaluation modes:
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//! use serde_json::json;
//!
//! let engine = JpxEngine::new();
//!
//! // From parsed JSON
//! let data = json!({"items": [1, 2, 3]});
//! let result = engine.evaluate("length(items)", &data).unwrap();
//! assert_eq!(result, json!(3));
//!
//! // From JSON string
//! let result = engine.evaluate_str("length(@)", r#"[1, 2, 3]"#).unwrap();
//! assert_eq!(result, json!(3));
//!
//! // Batch evaluation (multiple expressions, same input)
//! let exprs = vec!["a".to_string(), "b".to_string()];
//! let batch = engine.batch_evaluate(&exprs, &json!({"a": 1, "b": 2}));
//! assert_eq!(batch.results[0].result, Some(json!(1)));
//!
//! // Validation without evaluation
//! let valid = engine.validate("users[*].name");
//! assert!(valid.valid);
//! ```
//!
//! ## Function Introspection
//!
//! Discover and explore the 400+ available functions:
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//!
//! let engine = JpxEngine::new();
//!
//! // List all categories
//! let categories = engine.categories();
//! assert!(categories.contains(&"String".to_string()));
//!
//! // List functions in a category
//! let string_funcs = engine.functions(Some("String"));
//! assert!(string_funcs.iter().any(|f| f.name == "upper"));
//!
//! // Search by keyword (fuzzy matching, synonyms)
//! let results = engine.search_functions("upper", 5);
//! assert!(results.iter().any(|r| r.function.name == "upper"));
//!
//! // Get detailed function info
//! let info = engine.describe_function("upper").unwrap();
//! assert_eq!(info.category, "String");
//!
//! // Find similar functions
//! let similar = engine.similar_functions("upper").unwrap();
//! assert!(!similar.same_category.is_empty());
//! ```
//!
//! ## JSON Utilities
//!
//! Beyond JMESPath evaluation, the engine provides JSON manipulation tools:
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//!
//! let engine = JpxEngine::new();
//!
//! // Pretty-print JSON
//! let formatted = engine.format_json(r#"{"a":1}"#, 2).unwrap();
//! assert!(formatted.contains('\n'));
//!
//! // Generate JSON Patch (RFC 6902)
//! let patch = engine.diff(r#"{"a": 1}"#, r#"{"a": 2}"#).unwrap();
//!
//! // Apply JSON Patch
//! let result = engine.patch(
//!     r#"{"a": 1}"#,
//!     r#"[{"op": "replace", "path": "/a", "value": 2}]"#
//! ).unwrap();
//!
//! // Apply JSON Merge Patch (RFC 7396)
//! let merged = engine.merge(
//!     r#"{"a": 1, "b": 2}"#,
//!     r#"{"b": 3, "c": 4}"#
//! ).unwrap();
//!
//! // Analyze JSON structure
//! let stats = engine.stats(r#"[1, 2, 3]"#).unwrap();
//! assert_eq!(stats.root_type, "array");
//! ```
//!
//! ## Query Store
//!
//! Store and reuse named queries within a session:
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//! use serde_json::json;
//!
//! let engine = JpxEngine::new();
//!
//! // Define a reusable query
//! engine.define_query(
//!     "active_users".to_string(),
//!     "users[?active].name".to_string(),
//!     Some("Get names of active users".to_string())
//! ).unwrap();
//!
//! // Run it by name
//! let data = json!({"users": [
//!     {"name": "alice", "active": true},
//!     {"name": "bob", "active": false}
//! ]});
//! let result = engine.run_query("active_users", &data).unwrap();
//! assert_eq!(result, json!(["alice"]));
//!
//! // List all stored queries
//! let queries = engine.list_queries().unwrap();
//! assert_eq!(queries.len(), 1);
//! ```
//!
//! ## Tool Discovery
//!
//! Register and search tools across multiple servers (for MCP integration):
//!
//! ```rust
//! use jpx_engine::{JpxEngine, DiscoverySpec};
//! use serde_json::json;
//!
//! let engine = JpxEngine::new();
//!
//! // Register a server's tools
//! let spec: DiscoverySpec = serde_json::from_value(json!({
//!     "server": {"name": "my-server", "version": "1.0.0"},
//!     "tools": [
//!         {"name": "create_user", "description": "Create a new user", "tags": ["write"]}
//!     ]
//! })).unwrap();
//!
//! let result = engine.register_discovery(spec, false).unwrap();
//! assert!(result.ok);
//!
//! // Search across registered tools
//! let tools = engine.query_tools("user", 10).unwrap();
//! assert!(!tools.is_empty());
//! ```
//!
//! ## Strict Mode
//!
//! For standard JMESPath compliance without extensions:
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//! use serde_json::json;
//!
//! let engine = JpxEngine::strict();
//! assert!(engine.is_strict());
//!
//! // Standard functions work
//! let result = engine.evaluate("length(@)", &json!([1, 2, 3])).unwrap();
//! assert_eq!(result, json!(3));
//!
//! // Extension functions are not available for evaluation
//! // (but introspection still works for documentation purposes)
//! ```
//!
//! ## Architecture
//!
//! ```text
//! jmespath-extensions    (400+ functions, registry)
//!         |
//!    jpx-engine          (this crate - evaluation, search, discovery)
//!         |
//!    +----+----+
//!    |         |
//!   jpx    jpx-server    (CLI and MCP transport)
//! ```
//!
//! ## Thread Safety
//!
//! The engine uses interior mutability (`Arc<RwLock<...>>`) for the discovery
//! registry and query store, making it safe to share across threads. The function
//! registry is immutable after construction.

mod bm25;
mod discovery;
mod error;
mod query_store;
mod types;

#[cfg(feature = "arrow")]
pub mod arrow;

pub use bm25::{Bm25Index, DocInfo, IndexOptions, SearchResult as Bm25SearchResult, TermInfo};
pub use discovery::{
    CategoryInfo, CategorySummary, DiscoveryRegistry, DiscoverySpec, ExampleSpec, IndexStats,
    ParamSpec, RegistrationResult, ReturnSpec, ServerInfo, ServerSummary, ToolQueryResult,
    ToolSpec,
};
pub use error::{EngineError, Result};
pub use query_store::{QueryStore, StoredQuery};
pub use types::{
    BatchEvaluateResult, BatchExpressionResult, EvalRequest, EvalResponse, ValidationResult,
};

use jmespath::Runtime;
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{FunctionRegistry, expand_search_terms, lookup_synonyms};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strsim::jaro_winkler;

// Re-export commonly used types from jmespath_extensions
pub use jmespath_extensions::registry::{Category, FunctionInfo};

/// Detailed information about a JMESPath function.
///
/// This struct provides a serializable representation of function metadata,
/// suitable for API responses, documentation generation, and introspection tools.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let info = engine.describe_function("upper").unwrap();
///
/// println!("Function: {}", info.name);
/// println!("Category: {}", info.category);
/// println!("Signature: {}", info.signature);
/// println!("Example: {}", info.example);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDetail {
    /// Function name (e.g., "upper", "sum", "now")
    pub name: String,
    /// Category name (e.g., "String", "Math", "Datetime")
    pub category: String,
    /// Human-readable description of what the function does
    pub description: String,
    /// Function signature showing parameter types (e.g., "string -> string")
    pub signature: String,
    /// Example usage demonstrating the function
    pub example: String,
    /// Whether this is a standard JMESPath function (vs extension)
    pub is_standard: bool,
    /// JMESPath Enhancement Proposal number, if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jep: Option<String>,
    /// Alternative names for this function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl From<&FunctionInfo> for FunctionDetail {
    fn from(info: &FunctionInfo) -> Self {
        Self {
            name: info.name.to_string(),
            category: format!("{:?}", info.category),
            description: info.description.to_string(),
            signature: info.signature.to_string(),
            example: info.example.to_string(),
            is_standard: info.is_standard,
            jep: info.jep.map(|s| s.to_string()),
            aliases: info.aliases.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Result from searching for functions.
///
/// Contains the matched function along with information about how it matched
/// the search query and its relevance score.
///
/// # Scoring
///
/// Match types and approximate scores:
/// - `exact_name` (1000): Query exactly matches function name
/// - `alias` (900): Query matches a function alias
/// - `name_prefix` (800): Function name starts with query
/// - `name_contains` (700): Function name contains query
/// - `category` (600): Query matches category name
/// - `description` (100-300): Query found in description
/// - `fuzzy_name` (variable): Jaro-Winkler similarity > 0.8
/// - `synonym` (300): Query synonym found in name/description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matched function's details
    pub function: FunctionDetail,
    /// How the function matched (e.g., "exact_name", "description")
    pub match_type: String,
    /// Relevance score (higher = better match)
    pub score: i32,
}

/// Result from finding functions similar to a given function.
///
/// Groups similar functions by relationship type: same category,
/// similar signature, or related concepts in descriptions.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let similar = engine.similar_functions("upper").unwrap();
///
/// // Functions in the same category (String)
/// for f in &similar.same_category {
///     println!("Same category: {}", f.name);
/// }
///
/// // Functions with similar signatures
/// for f in &similar.similar_signature {
///     println!("Similar signature: {}", f.name);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFunctionsResult {
    /// Functions in the same category as the target
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_category: Vec<FunctionDetail>,
    /// Functions with similar parameter/return types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub similar_signature: Vec<FunctionDetail>,
    /// Functions with overlapping description keywords
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_concepts: Vec<FunctionDetail>,
}

/// Statistics about JSON data structure.
///
/// Provides insights into JSON data including type, size, depth, and
/// detailed field analysis for arrays of objects.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let stats = engine.stats(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#).unwrap();
///
/// println!("Type: {}", stats.root_type);      // "object"
/// println!("Size: {}", stats.size_human);     // "52 bytes"
/// println!("Depth: {}", stats.depth);         // 3
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    /// JSON type of the root value ("object", "array", "string", etc.)
    pub root_type: String,
    /// Size of the JSON string in bytes
    pub size_bytes: usize,
    /// Human-readable size (e.g., "1.5 KB", "2.3 MB")
    pub size_human: String,
    /// Maximum nesting depth (0 for primitives)
    pub depth: usize,
    /// Number of items (arrays only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// Number of keys (objects only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_count: Option<usize>,
    /// Field analysis (arrays of objects only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldAnalysis>>,
    /// Count of each JSON type in array (arrays only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_distribution: Option<HashMap<String, usize>>,
}

/// Analysis of a field across an array of objects.
///
/// Used by [`StatsResult`] to provide insights into consistent fields
/// in arrays of objects, including type information and null counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAnalysis {
    /// Field name (key)
    pub name: String,
    /// Most common type for this field
    pub field_type: String,
    /// Number of objects where this field is null
    pub null_count: usize,
    /// Number of distinct values (omitted for high-cardinality fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_count: Option<usize>,
}

/// Information about a path in a JSON structure.
///
/// Used by [`JpxEngine::paths`] to enumerate all paths in a JSON document.
///
/// # Example
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// let engine = JpxEngine::new();
/// let paths = engine.paths(r#"{"user": {"name": "alice"}}"#, true, false).unwrap();
///
/// for path in paths {
///     println!("{}: {:?}", path.path, path.path_type);
/// }
/// // Output:
/// // @: Some("object")
/// // user: Some("object")
/// // user.name: Some("string")
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// Path in dot notation (e.g., "user.name", "items.0.id")
    pub path: String,
    /// JSON type at this path (if `include_types` was true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
    /// Value at this path (if `include_values` was true, leaf nodes only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// The JMESPath query engine.
///
/// `JpxEngine` is the main entry point for all jpx functionality. It combines:
///
/// - **JMESPath runtime** with 400+ extension functions
/// - **Function registry** for introspection and search
/// - **Discovery registry** for cross-server tool indexing
/// - **Query store** for named query management
///
/// # Construction
///
/// ```rust
/// use jpx_engine::JpxEngine;
///
/// // Full engine with all extensions
/// let engine = JpxEngine::new();
///
/// // Strict mode (standard JMESPath only)
/// let strict_engine = JpxEngine::strict();
///
/// // Or using Default
/// let default_engine = JpxEngine::default();
/// ```
///
/// # Thread Safety
///
/// The engine is designed to be shared across threads. The discovery registry
/// and query store use `Arc<RwLock<...>>` for interior mutability, while the
/// function registry is immutable after construction.
///
/// ```rust
/// use jpx_engine::JpxEngine;
/// use std::sync::Arc;
///
/// let engine = Arc::new(JpxEngine::new());
///
/// // Clone the Arc to share across threads
/// let engine_clone = Arc::clone(&engine);
/// std::thread::spawn(move || {
///     let result = engine_clone.evaluate("length(@)", &serde_json::json!([1, 2, 3]));
/// });
/// ```
pub struct JpxEngine {
    /// JMESPath runtime with all extensions registered
    runtime: Runtime,
    /// Function registry for introspection
    registry: FunctionRegistry,
    /// Discovery registry for cross-server tool search
    discovery: Arc<RwLock<DiscoveryRegistry>>,
    /// Query store for named queries
    queries: Arc<RwLock<QueryStore>>,
    /// Whether to use strict mode (standard JMESPath only)
    strict: bool,
}

impl JpxEngine {
    /// Creates a new engine with all extension functions enabled.
    ///
    /// This is the standard way to create an engine with full functionality,
    /// including all 400+ extension functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Standard JMESPath works
    /// let result = engine.evaluate("name", &json!({"name": "alice"})).unwrap();
    /// assert_eq!(result, json!("alice"));
    ///
    /// // Extension functions also work
    /// let result = engine.evaluate("upper(name)", &json!({"name": "alice"})).unwrap();
    /// assert_eq!(result, json!("ALICE"));
    /// ```
    pub fn new() -> Self {
        Self::with_options(false)
    }

    /// Creates a new engine with configurable strict mode.
    ///
    /// # Arguments
    ///
    /// * `strict` - If `true`, only standard JMESPath functions are available
    ///   for evaluation. Introspection features still show all functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// // Create engine with extensions
    /// let full_engine = JpxEngine::with_options(false);
    ///
    /// // Create strict engine (standard JMESPath only)
    /// let strict_engine = JpxEngine::with_options(true);
    /// assert!(strict_engine.is_strict());
    /// ```
    pub fn with_options(strict: bool) -> Self {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        if !strict {
            register_all(&mut runtime);
        }

        let mut registry = FunctionRegistry::new();
        registry.register_all();

        Self {
            runtime,
            registry,
            discovery: Arc::new(RwLock::new(DiscoveryRegistry::new())),
            queries: Arc::new(RwLock::new(QueryStore::new())),
            strict,
        }
    }

    /// Creates a new engine in strict mode (standard JMESPath only).
    ///
    /// Equivalent to `JpxEngine::with_options(true)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::strict();
    ///
    /// // Standard functions work
    /// let result = engine.evaluate("length(@)", &json!([1, 2, 3])).unwrap();
    /// assert_eq!(result, json!(3));
    /// ```
    pub fn strict() -> Self {
        Self::with_options(true)
    }

    /// Returns `true` if the engine is in strict mode.
    ///
    /// In strict mode, only standard JMESPath functions are available for
    /// evaluation. Extension functions will cause evaluation errors.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Returns a reference to the underlying JMESPath runtime.
    ///
    /// This provides access to the low-level runtime for advanced use cases.
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Returns a reference to the function registry.
    ///
    /// The registry contains metadata about all available functions and can
    /// be used for custom introspection beyond what the engine methods provide.
    pub fn registry(&self) -> &FunctionRegistry {
        &self.registry
    }

    /// Returns a reference to the discovery registry.
    ///
    /// The discovery registry manages cross-server tool indexing and search.
    /// Access is through `Arc<RwLock<...>>` for thread-safe mutation.
    pub fn discovery(&self) -> &Arc<RwLock<DiscoveryRegistry>> {
        &self.discovery
    }

    /// Returns a reference to the query store.
    ///
    /// The query store manages named queries for the session.
    /// Access is through `Arc<RwLock<...>>` for thread-safe mutation.
    pub fn queries(&self) -> &Arc<RwLock<QueryStore>> {
        &self.queries
    }

    // =========================================================================
    // Evaluation methods
    // =========================================================================

    /// Evaluates a JMESPath expression against JSON input.
    ///
    /// This is the primary method for running JMESPath queries. The expression
    /// is compiled and executed against the provided JSON value.
    ///
    /// # Arguments
    ///
    /// * `expression` - A JMESPath expression string
    /// * `input` - JSON data to query
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidExpression`] if the expression has syntax errors,
    /// or [`EngineError::EvaluationFailed`] if evaluation fails (e.g., calling an
    /// undefined function in strict mode).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Simple field access
    /// let result = engine.evaluate("name", &json!({"name": "alice"})).unwrap();
    /// assert_eq!(result, json!("alice"));
    ///
    /// // Array projection with function
    /// let result = engine.evaluate("users[*].name | sort(@)", &json!({
    ///     "users": [{"name": "charlie"}, {"name": "alice"}, {"name": "bob"}]
    /// })).unwrap();
    /// assert_eq!(result, json!(["alice", "bob", "charlie"]));
    /// ```
    pub fn evaluate(&self, expression: &str, input: &Value) -> Result<Value> {
        let expr = self
            .runtime
            .compile(expression)
            .map_err(|e| EngineError::InvalidExpression(e.to_string()))?;

        // Convert input Value to Variable for jmespath
        let var = jmespath::Variable::from_json(&input.to_string())
            .map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let result = expr
            .search(&var)
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        // Convert Rcvar to Value
        let value: Value = serde_json::to_value(result.as_ref())
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        Ok(value)
    }

    /// Evaluates a JMESPath expression against a JSON string.
    ///
    /// Convenience method that parses the JSON string before evaluation.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::InvalidJson`] if the input is not valid JSON,
    /// or evaluation errors as with [`evaluate`](Self::evaluate).
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let result = engine.evaluate_str("length(@)", r#"[1, 2, 3, 4, 5]"#).unwrap();
    /// assert_eq!(result, json!(5));
    /// ```
    pub fn evaluate_str(&self, expression: &str, input: &str) -> Result<Value> {
        let json: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        self.evaluate(expression, &json)
    }

    /// Evaluates multiple expressions against the same input.
    ///
    /// Useful for extracting multiple values from a document in one call.
    /// Each expression is evaluated independently; failures don't affect other expressions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    /// use serde_json::json;
    ///
    /// let engine = JpxEngine::new();
    /// let input = json!({"name": "alice", "age": 30, "active": true});
    ///
    /// let exprs = vec![
    ///     "name".to_string(),
    ///     "age".to_string(),
    ///     "missing".to_string(),  // Returns null, not an error
    /// ];
    ///
    /// let results = engine.batch_evaluate(&exprs, &input);
    /// assert_eq!(results.results[0].result, Some(json!("alice")));
    /// assert_eq!(results.results[1].result, Some(json!(30)));
    /// assert_eq!(results.results[2].result, Some(json!(null)));
    /// ```
    pub fn batch_evaluate(&self, expressions: &[String], input: &Value) -> BatchEvaluateResult {
        let results = expressions
            .iter()
            .map(|expr| match self.evaluate(expr, input) {
                Ok(result) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(e) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();

        BatchEvaluateResult { results }
    }

    /// Validates a JMESPath expression without evaluating it.
    ///
    /// Checks if an expression has valid syntax without needing input data.
    /// Useful for validating user-provided expressions before storing them.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Valid expression
    /// let result = engine.validate("users[*].name | sort(@)");
    /// assert!(result.valid);
    /// assert!(result.error.is_none());
    ///
    /// // Invalid expression (unclosed bracket)
    /// let result = engine.validate("users[*.name");
    /// assert!(!result.valid);
    /// assert!(result.error.is_some());
    /// ```
    pub fn validate(&self, expression: &str) -> ValidationResult {
        match jmespath::compile(expression) {
            Ok(_) => ValidationResult {
                valid: true,
                error: None,
            },
            Err(e) => ValidationResult {
                valid: false,
                error: Some(e.to_string()),
            },
        }
    }

    // =========================================================================
    // Introspection methods
    // =========================================================================

    /// Lists all available function categories.
    ///
    /// Returns category names like "String", "Math", "Datetime", etc.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    /// let categories = engine.categories();
    ///
    /// assert!(categories.contains(&"String".to_string()));
    /// assert!(categories.contains(&"Math".to_string()));
    /// assert!(categories.contains(&"Array".to_string()));
    /// ```
    pub fn categories(&self) -> Vec<String> {
        Category::all().iter().map(|c| format!("{:?}", c)).collect()
    }

    /// Lists functions, optionally filtered by category.
    ///
    /// # Arguments
    ///
    /// * `category` - Optional category name to filter by (case-insensitive)
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // All functions
    /// let all = engine.functions(None);
    /// assert!(all.len() > 100);
    ///
    /// // Just string functions
    /// let string_funcs = engine.functions(Some("String"));
    /// assert!(string_funcs.iter().all(|f| f.category == "String"));
    /// ```
    pub fn functions(&self, category: Option<&str>) -> Vec<FunctionDetail> {
        match category.and_then(parse_category) {
            Some(cat) => self
                .registry
                .functions_in_category(cat)
                .map(FunctionDetail::from)
                .collect(),
            None => self
                .registry
                .functions()
                .map(FunctionDetail::from)
                .collect(),
        }
    }

    /// Gets detailed information about a function by name or alias.
    ///
    /// # Arguments
    ///
    /// * `name` - Function name or alias (e.g., "upper", "md5", "len")
    ///
    /// # Returns
    ///
    /// `Some(FunctionDetail)` if found, `None` if no matching function exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// let info = engine.describe_function("upper").unwrap();
    /// assert_eq!(info.category, "String");
    /// println!("Signature: {}", info.signature);
    /// println!("Example: {}", info.example);
    ///
    /// // Also works with aliases
    /// let info = engine.describe_function("len");  // alias for "length"
    /// ```
    pub fn describe_function(&self, name: &str) -> Option<FunctionDetail> {
        self.registry.get_function(name).map(FunctionDetail::from)
    }

    /// Searches for functions matching a query string.
    ///
    /// Uses fuzzy matching, synonyms, and searches across names, descriptions,
    /// categories, and signatures. Results are ranked by relevance.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term (e.g., "hash", "string manipulation", "date")
    /// * `limit` - Maximum number of results to return
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// // Search by concept
    /// let results = engine.search_functions("hash", 10);
    /// assert!(results.iter().any(|r| r.function.name == "md5"));
    /// assert!(results.iter().any(|r| r.function.name == "sha256"));
    ///
    /// // Results are ranked by relevance
    /// for result in &results {
    ///     println!("{}: {} (score: {})",
    ///         result.function.name,
    ///         result.match_type,
    ///         result.score
    ///     );
    /// }
    /// ```
    pub fn search_functions(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();

        // Expand query terms using synonyms
        let expanded_terms = expand_search_terms(&query_lower);

        let all_functions: Vec<_> = self.registry.functions().collect();
        let mut results: Vec<SearchResult> = Vec::new();

        for info in &all_functions {
            let name_lower = info.name.to_lowercase();
            let desc_lower = info.description.to_lowercase();
            let category_lower = format!("{:?}", info.category).to_lowercase();
            let signature_lower = info.signature.to_lowercase();
            let aliases_lower: Vec<String> = info
                .aliases
                .iter()
                .map(|a: &&str| a.to_lowercase())
                .collect();

            // Calculate match score and type
            let (score, match_type) = calculate_match_score(
                &query_lower,
                &expanded_terms,
                &MatchContext {
                    name: &name_lower,
                    aliases: &aliases_lower,
                    category: &category_lower,
                    description: &desc_lower,
                    signature: &signature_lower,
                },
            );

            if score > 0 {
                results.push(SearchResult {
                    function: FunctionDetail::from(*info),
                    match_type,
                    score,
                });
            }
        }

        // Sort by score descending, then by name
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.function.name.cmp(&b.function.name))
        });

        results.truncate(limit);
        results
    }

    /// Finds functions similar to a given function.
    ///
    /// Returns functions grouped by relationship type:
    /// - Same category (e.g., other string functions if input is "upper")
    /// - Similar signature (same parameter/return types)
    /// - Related concepts (overlapping description keywords)
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the function to find similar functions for
    ///
    /// # Returns
    ///
    /// `Some(SimilarFunctionsResult)` if the function exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::new();
    ///
    /// let similar = engine.similar_functions("upper").unwrap();
    ///
    /// // Other string functions
    /// println!("Same category:");
    /// for f in &similar.same_category {
    ///     println!("  - {}", f.name);
    /// }
    /// ```
    pub fn similar_functions(&self, name: &str) -> Option<SimilarFunctionsResult> {
        let info = self.registry.get_function(name)?;
        let all_functions: Vec<_> = self.registry.functions().collect();

        // Same category
        let same_category: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| f.category == info.category && f.name != info.name)
            .take(5)
            .map(|f| FunctionDetail::from(*f))
            .collect();

        // Similar signature (same arity)
        let this_arity = count_params(info.signature);
        let similar_signature: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| {
                f.name != info.name
                    && f.category != info.category
                    && count_params(f.signature) == this_arity
            })
            .take(5)
            .map(|f| FunctionDetail::from(*f))
            .collect();

        // Related concepts (description keyword overlap)
        let keywords = extract_keywords(info.description);
        let mut concept_scores: Vec<(&FunctionInfo, usize)> = all_functions
            .iter()
            .filter(|f| f.name != info.name)
            .map(|f| {
                let f_keywords = extract_keywords(f.description);
                let overlap = keywords.iter().filter(|k| f_keywords.contains(*k)).count();
                (*f, overlap)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        concept_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let related_concepts: Vec<FunctionDetail> = concept_scores
            .into_iter()
            .take(5)
            .map(|(f, _)| FunctionDetail::from(f))
            .collect();

        Some(SimilarFunctionsResult {
            same_category,
            similar_signature,
            related_concepts,
        })
    }

    // =========================================================================
    // JSON utility methods
    // =========================================================================

    /// Format JSON with indentation.
    pub fn format_json(&self, input: &str, indent: usize) -> Result<String> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        if indent == 0 {
            serde_json::to_string(&value).map_err(|e| EngineError::Internal(e.to_string()))
        } else {
            let indent_bytes = vec![b' '; indent];
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
            let mut buf = Vec::new();
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            value
                .serialize(&mut ser)
                .map_err(|e| EngineError::Internal(e.to_string()))?;
            String::from_utf8(buf).map_err(|e| EngineError::Internal(e.to_string()))
        }
    }

    /// Generate a JSON Patch (RFC 6902) from source to target.
    pub fn diff(&self, source: &str, target: &str) -> Result<Value> {
        let source_val: Value =
            serde_json::from_str(source).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let target_val: Value =
            serde_json::from_str(target).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let patch = json_patch::diff(&source_val, &target_val);
        serde_json::to_value(&patch).map_err(|e| EngineError::Internal(e.to_string()))
    }

    /// Apply a JSON Patch (RFC 6902) to a document.
    pub fn patch(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch: json_patch::Patch =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::patch(&mut doc, &patch)
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        Ok(doc)
    }

    /// Apply a JSON Merge Patch (RFC 7396) to a document.
    pub fn merge(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch_val: Value =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::merge(&mut doc, &patch_val);
        Ok(doc)
    }

    /// Extract keys from a JSON object.
    pub fn keys(&self, input: &str, recursive: bool) -> Result<Vec<String>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut keys = Vec::new();
        if recursive {
            extract_keys_recursive(&value, "", &mut keys);
        } else if let Value::Object(map) = &value {
            keys = map.keys().cloned().collect();
            keys.sort();
        }
        Ok(keys)
    }

    /// Extract all paths from JSON data.
    pub fn paths(
        &self,
        input: &str,
        include_types: bool,
        include_values: bool,
    ) -> Result<Vec<PathInfo>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut paths = Vec::new();
        extract_paths(&value, "", include_types, include_values, &mut paths);
        Ok(paths)
    }

    /// Analyze JSON data and return statistics.
    pub fn stats(&self, input: &str) -> Result<StatsResult> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let size_bytes = input.len();
        let depth = calculate_depth(&value);

        let (length, key_count, fields, type_distribution) = match &value {
            Value::Array(arr) => {
                let type_dist = calculate_type_distribution(arr);
                let field_analysis = if arr.iter().all(|v| v.is_object()) {
                    Some(analyze_array_fields(arr))
                } else {
                    None
                };
                (Some(arr.len()), None, field_analysis, Some(type_dist))
            }
            Value::Object(map) => (None, Some(map.len()), None, None),
            _ => (None, None, None, None),
        };

        Ok(StatsResult {
            root_type: json_type_name(&value).to_string(),
            size_bytes,
            size_human: format_size(size_bytes),
            depth,
            length,
            key_count,
            fields,
            type_distribution,
        })
    }

    // =========================================================================
    // Query store methods
    // =========================================================================

    /// Define (store) a named query.
    pub fn define_query(
        &self,
        name: String,
        expression: String,
        description: Option<String>,
    ) -> Result<Option<StoredQuery>> {
        // Validate expression first
        let validation = self.validate(&expression);
        if !validation.valid {
            return Err(EngineError::InvalidExpression(
                validation
                    .error
                    .unwrap_or_else(|| "Invalid expression".to_string()),
            ));
        }

        let query = StoredQuery {
            name,
            expression,
            description,
        };

        self.queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .define(query)
            .pipe(Ok)
    }

    /// Get a stored query by name.
    pub fn get_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .get(name)
            .cloned())
    }

    /// Delete a stored query.
    pub fn delete_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .delete(name))
    }

    /// List all stored queries.
    pub fn list_queries(&self) -> Result<Vec<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Run a stored query.
    pub fn run_query(&self, name: &str, input: &Value) -> Result<Value> {
        let query = self
            .get_query(name)?
            .ok_or_else(|| EngineError::QueryNotFound(name.to_string()))?;

        self.evaluate(&query.expression, input)
    }

    // =========================================================================
    // Discovery methods
    // =========================================================================

    /// Register a discovery spec.
    pub fn register_discovery(
        &self,
        spec: DiscoverySpec,
        replace: bool,
    ) -> Result<RegistrationResult> {
        Ok(self
            .discovery
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .register(spec, replace))
    }

    /// Unregister a server from discovery.
    pub fn unregister_discovery(&self, server_name: &str) -> Result<bool> {
        Ok(self
            .discovery
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .unregister(server_name))
    }

    /// Query tools across registered servers.
    pub fn query_tools(&self, query: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .query(query, top_k))
    }

    /// Find tools similar to a given tool.
    pub fn similar_tools(&self, tool_id: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .similar(tool_id, top_k))
    }

    /// List all registered discovery servers.
    pub fn list_discovery_servers(&self) -> Result<Vec<ServerSummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list_servers())
    }

    /// List discovery categories.
    pub fn list_discovery_categories(&self) -> Result<HashMap<String, CategorySummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list_categories())
    }

    /// Get discovery index stats.
    pub fn discovery_index_stats(&self) -> Result<Option<IndexStats>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .index_stats())
    }

    /// Get the discovery schema.
    pub fn get_discovery_schema(&self) -> Value {
        DiscoveryRegistry::get_schema()
    }
}

impl Default for JpxEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Context for calculating match scores
struct MatchContext<'a> {
    name: &'a str,
    aliases: &'a [String],
    category: &'a str,
    description: &'a str,
    signature: &'a str,
}

/// Calculate match score and type for a function
fn calculate_match_score(
    query: &str,
    expanded_terms: &[String],
    ctx: &MatchContext,
) -> (i32, String) {
    // Exact name match
    if ctx.name == query {
        return (1000, "exact_name".to_string());
    }

    // Alias match
    if ctx.aliases.iter().any(|a| a == query) {
        return (900, "alias".to_string());
    }

    // Name starts with query
    if ctx.name.starts_with(query) {
        return (800, "name_prefix".to_string());
    }

    // Name contains query
    if ctx.name.contains(query) {
        return (700, "name_contains".to_string());
    }

    // Category match
    if ctx.category == query {
        return (600, "category".to_string());
    }

    // Check expanded terms in description/signature
    let mut desc_score = 0;
    let mut matched_terms = Vec::new();

    for term in expanded_terms {
        if ctx.description.contains(term) || ctx.signature.contains(term) {
            desc_score += 100;
            matched_terms.push(term.clone());
        }
    }

    if desc_score > 0 {
        return (
            desc_score,
            format!("description ({})", matched_terms.join(", ")),
        );
    }

    // Fuzzy name match using Jaro-Winkler
    let similarity = jaro_winkler(query, ctx.name);
    if similarity > 0.8 {
        return ((similarity * 500.0) as i32, "fuzzy_name".to_string());
    }

    // Check synonyms
    if let Some(synonyms) = lookup_synonyms(query) {
        for syn in synonyms {
            if ctx.name.contains(syn) || ctx.description.contains(syn) {
                return (300, format!("synonym ({})", syn));
            }
        }
    }

    (0, String::new())
}

/// Parse category string to Category enum
fn parse_category(name: &str) -> Option<Category> {
    Category::all()
        .iter()
        .find(|cat| format!("{:?}", cat).to_lowercase() == name.to_lowercase())
        .copied()
}

/// Count parameters in a function signature
fn count_params(signature: &str) -> usize {
    signature.matches(',').count() + 1
}

/// Extract keywords from a description for related concept matching
fn extract_keywords(description: &str) -> Vec<&str> {
    let stopwords = [
        "a",
        "an",
        "the",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "must",
        "shall",
        "can",
        "to",
        "of",
        "in",
        "for",
        "on",
        "with",
        "at",
        "by",
        "from",
        "or",
        "and",
        "as",
        "if",
        "that",
        "which",
        "this",
        "these",
        "those",
        "it",
        "its",
        "such",
        "when",
        "where",
        "how",
        "all",
        "each",
        "every",
        "both",
        "few",
        "more",
        "most",
        "other",
        "some",
        "any",
        "no",
        "not",
        "only",
        "same",
        "than",
        "very",
        "just",
        "also",
        "into",
        "over",
        "after",
        "before",
        "between",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "why",
        "because",
        "while",
        "although",
        "though",
        "unless",
        "until",
        "whether",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "input",
        "output",
        "function",
        "functions",
        "used",
        "using",
        "use",
    ];

    description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(&w.to_lowercase().as_str()))
        .collect()
}

/// Extract keys recursively from a JSON value
fn extract_keys_recursive(value: &Value, prefix: &str, keys: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                keys.push(path.clone());
                extract_keys_recursive(v, &path, keys);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let path = format!("{}.{}", prefix, i);
                extract_keys_recursive(v, &path, keys);
            }
        }
        _ => {}
    }
}

/// Extract paths from a JSON value
fn extract_paths(
    value: &Value,
    prefix: &str,
    include_types: bool,
    include_values: bool,
    paths: &mut Vec<PathInfo>,
) {
    let current_path = if prefix.is_empty() {
        "@".to_string()
    } else {
        prefix.to_string()
    };

    match value {
        Value::Object(map) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("object".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        Value::Array(arr) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("array".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}.{}", prefix, i);
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        _ => {
            paths.push(PathInfo {
                path: current_path,
                path_type: if include_types {
                    Some(json_type_name(value).to_string())
                } else {
                    None
                },
                value: if include_values {
                    Some(value.clone())
                } else {
                    None
                },
            });
        }
    }
}

/// Calculate the nesting depth of a JSON value
fn calculate_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(calculate_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(calculate_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Get the type name of a JSON value
fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Calculate type distribution in an array
fn calculate_type_distribution(arr: &[Value]) -> HashMap<String, usize> {
    let mut dist = HashMap::new();
    for item in arr {
        *dist.entry(json_type_name(item).to_string()).or_insert(0) += 1;
    }
    dist
}

/// Analyze fields in an array of objects
fn analyze_array_fields(arr: &[Value]) -> Vec<FieldAnalysis> {
    let mut field_types: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut field_null_counts: HashMap<String, usize> = HashMap::new();
    let mut field_values: HashMap<String, Vec<Value>> = HashMap::new();

    for item in arr {
        if let Value::Object(map) = item {
            for (k, v) in map {
                let types = field_types.entry(k.clone()).or_default();
                *types.entry(json_type_name(v).to_string()).or_insert(0) += 1;

                if v.is_null() {
                    *field_null_counts.entry(k.clone()).or_insert(0) += 1;
                }

                // Track unique values for low-cardinality detection
                let values = field_values.entry(k.clone()).or_default();
                if values.len() < 100 && !values.contains(v) {
                    values.push(v.clone());
                }
            }
        }
    }

    let mut fields: Vec<FieldAnalysis> = field_types
        .into_iter()
        .map(|(name, types)| {
            let predominant_type = types
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(t, _)| t)
                .unwrap_or_else(|| "unknown".to_string());

            let null_count = field_null_counts.get(&name).copied().unwrap_or(0);
            let unique_count = field_values.get(&name).map(|v| v.len());

            FieldAnalysis {
                name,
                field_type: predominant_type,
                null_count,
                unique_count,
            }
        })
        .collect();

    fields.sort_by(|a, b| a.name.cmp(&b.name));
    fields
}

/// Format size in human-readable form
fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

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

/// Extension trait for pipe-style method chaining
trait Pipe: Sized {
    fn pipe<T, F: FnOnce(Self) -> T>(self, f: F) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_engine_creation() {
        let engine = JpxEngine::new();
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_engine_strict_mode() {
        let engine = JpxEngine::strict();
        assert!(engine.is_strict());
    }

    #[test]
    fn test_engine_default() {
        let engine = JpxEngine::default();
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
        let result = engine.evaluate("users[*].name", &input).unwrap();
        assert_eq!(result, json!(["alice", "bob"]));
    }

    #[test]
    fn test_evaluate_str() {
        let engine = JpxEngine::new();
        let result = engine.evaluate_str("length(@)", r#"[1, 2, 3]"#).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn test_batch_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"a": 1, "b": 2});
        let exprs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = engine.batch_evaluate(&exprs, &input);

        assert_eq!(result.results.len(), 3);
        assert_eq!(result.results[0].result, Some(json!(1)));
        assert_eq!(result.results[1].result, Some(json!(2)));
        assert_eq!(result.results[2].result, Some(json!(null)));
    }

    #[test]
    fn test_validate() {
        let engine = JpxEngine::new();

        let valid = engine.validate("users[*].name");
        assert!(valid.valid);
        assert!(valid.error.is_none());

        let invalid = engine.validate("users[*.name");
        assert!(!invalid.valid);
        assert!(invalid.error.is_some());
    }

    #[test]
    fn test_categories() {
        let engine = JpxEngine::new();
        let cats = engine.categories();
        assert!(!cats.is_empty());
        assert!(cats.iter().any(|c| c == "String"));
    }

    #[test]
    fn test_functions() {
        let engine = JpxEngine::new();

        // All functions
        let all = engine.functions(None);
        assert!(!all.is_empty());

        // Filtered by category
        let string_funcs = engine.functions(Some("String"));
        assert!(!string_funcs.is_empty());
        assert!(string_funcs.iter().all(|f| f.category == "String"));
    }

    #[test]
    fn test_describe_function() {
        let engine = JpxEngine::new();

        let info = engine.describe_function("upper").unwrap();
        assert_eq!(info.name, "upper");
        assert_eq!(info.category, "String");

        let missing = engine.describe_function("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_search_functions() {
        let engine = JpxEngine::new();

        let results = engine.search_functions("string", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_similar_functions() {
        let engine = JpxEngine::new();

        let result = engine.similar_functions("upper").unwrap();
        // Should have functions in same category
        assert!(!result.same_category.is_empty());
    }

    #[test]
    fn test_format_json() {
        let engine = JpxEngine::new();

        let formatted = engine.format_json(r#"{"a":1,"b":2}"#, 2).unwrap();
        assert!(formatted.contains('\n'));

        let compact = engine.format_json(r#"{"a":1,"b":2}"#, 0).unwrap();
        assert!(!compact.contains('\n'));
    }

    #[test]
    fn test_diff() {
        let engine = JpxEngine::new();

        let patch = engine.diff(r#"{"a": 1}"#, r#"{"a": 2}"#).unwrap();

        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
    }

    #[test]
    fn test_patch() {
        let engine = JpxEngine::new();

        let result = engine
            .patch(
                r#"{"a": 1}"#,
                r#"[{"op": "replace", "path": "/a", "value": 2}]"#,
            )
            .unwrap();

        assert_eq!(result, json!({"a": 2}));
    }

    #[test]
    fn test_merge() {
        let engine = JpxEngine::new();

        let result = engine
            .merge(r#"{"a": 1, "b": 2}"#, r#"{"b": 3, "c": 4}"#)
            .unwrap();

        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_keys() {
        let engine = JpxEngine::new();

        let keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, false).unwrap();
        assert_eq!(keys, vec!["a", "b"]);

        let recursive_keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, true).unwrap();
        assert!(recursive_keys.contains(&"b.c".to_string()));
    }

    #[test]
    fn test_paths() {
        let engine = JpxEngine::new();

        let paths = engine.paths(r#"{"a": 1}"#, true, false).unwrap();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_stats() {
        let engine = JpxEngine::new();

        let stats = engine.stats(r#"[1, 2, 3]"#).unwrap();
        assert_eq!(stats.root_type, "array");
        assert_eq!(stats.length, Some(3));
    }

    #[test]
    fn test_query_store() {
        let engine = JpxEngine::new();

        // Define a query
        engine
            .define_query("count".to_string(), "length(@)".to_string(), None)
            .unwrap();

        // Get it
        let query = engine.get_query("count").unwrap().unwrap();
        assert_eq!(query.expression, "length(@)");

        // Run it
        let result = engine.run_query("count", &json!([1, 2, 3])).unwrap();
        assert_eq!(result, json!(3));

        // List queries
        let queries = engine.list_queries().unwrap();
        assert_eq!(queries.len(), 1);

        // Delete it
        engine.delete_query("count").unwrap();
        assert!(engine.get_query("count").unwrap().is_none());
    }

    #[test]
    fn test_discovery() {
        let engine = JpxEngine::new();

        let spec: DiscoverySpec = serde_json::from_value(json!({
            "server": {"name": "test-server", "version": "1.0.0"},
            "tools": [
                {"name": "test_tool", "description": "A test tool", "tags": ["test"]}
            ]
        }))
        .unwrap();

        // Register
        let result = engine.register_discovery(spec, false).unwrap();
        assert!(result.ok);
        assert_eq!(result.tools_indexed, 1);

        // List servers
        let servers = engine.list_discovery_servers().unwrap();
        assert_eq!(servers.len(), 1);

        // Query tools
        let tools = engine.query_tools("test", 10).unwrap();
        assert!(!tools.is_empty());

        // Unregister
        assert!(engine.unregister_discovery("test-server").unwrap());
        assert!(engine.list_discovery_servers().unwrap().is_empty());
    }
}
