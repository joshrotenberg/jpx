//! # jpx-engine
//!
//! Protocol-agnostic JMESPath query engine with 490+ functions.
//!
//! This crate provides the core "brain" of jpx - everything you can do with JMESPath
//! beyond basic compile and evaluate. It's designed to be transport-agnostic, allowing
//! the CLI (`jpx`), MCP server (`jpx-mcp`), or any future REST/gRPC adapters to be
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
//! | **Configuration** | Declarative `jpx.toml` config with layered discovery and merge |
//! | **JSON Utilities** | Format, diff, patch, merge, stats, paths, keys |
//! | **Arrow** | Apache Arrow conversion (optional, via `arrow` feature) |
//!
//! ## Cargo Features
//!
//! - **`arrow`** - Enables Apache Arrow support for columnar data conversion.
//!   This adds the `arrow` module with functions to convert between Arrow
//!   RecordBatches and JSON Values. Used by the CLI for Parquet I/O.
//! - **`let-expr`** - Enables `let` expression support (variable bindings in
//!   JMESPath expressions). Forwarded from jpx-core. Enabled by default.
//! - **`schema`** - Derives `JsonSchema` on discovery types for JSON Schema
//!   generation. Used by the MCP server for tool schemas.
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
//! Discover and explore the 490+ available functions:
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
//! ## Configuration
//!
//! Load engine settings from `jpx.toml` files with layered discovery:
//!
//! ```rust
//! use jpx_engine::{JpxEngine, EngineConfig};
//!
//! // Discover config from standard locations
//! // (~/.config/jpx/jpx.toml, ./jpx.toml, $JPX_CONFIG)
//! let config = EngineConfig::discover().unwrap();
//! let engine = JpxEngine::from_config(config).unwrap();
//!
//! // Or use the builder for programmatic configuration
//! let engine = JpxEngine::builder()
//!     .strict(false)
//!     .disable_category("geo")
//!     .disable_function("env")
//!     .build()
//!     .unwrap();
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
//!    jpx-core           (parser, runtime, 490+ functions, registry)
//!         |
//!    jpx-engine         (this crate - evaluation, search, discovery, config)
//!         |
//!    +----+----+
//!    |         |
//!   jpx    jpx-mcp     (CLI and MCP transport)
//! ```
//!
//! ## Thread Safety
//!
//! The engine uses interior mutability (`Arc<RwLock<...>>`) for the discovery
//! registry and query store, making it safe to share across threads. The function
//! registry is immutable after construction.

mod bm25;
pub mod config;
mod discovery;
mod error;
mod eval;
mod explain;
mod introspection;
mod json_utils;
mod query_store;
mod types;

#[cfg(feature = "arrow")]
pub mod arrow;

pub use bm25::{Bm25Index, DocInfo, IndexOptions, SearchResult as Bm25SearchResult, TermInfo};
pub use config::{EngineBuilder, EngineConfig, EngineSection, FunctionsSection, QueriesSection};
pub use discovery::{
    CategoryInfo, CategorySummary, DiscoveryRegistry, DiscoverySpec, ExampleSpec, IndexStats,
    ParamSpec, RegistrationResult, ReturnSpec, ServerInfo, ServerSummary, ToolQueryResult,
    ToolSpec,
};
pub use error::{EngineError, EvaluationErrorKind, Result};
pub use explain::{ExplainResult, ExplainStep, has_let_nodes};
pub use introspection::{FunctionDetail, SearchResult, SimilarFunctionsResult};
pub use json_utils::{FieldAnalysis, PathInfo, StatsResult};
pub use query_store::{QueryStore, StoredQuery};
pub use types::{
    BatchEvaluateResult, BatchExpressionResult, EvalRequest, EvalResponse, ValidationResult,
};

use discovery::DiscoveryRegistry as DiscoveryRegistryInner;
use error::EngineError as EngineErrorInner;
use query_store::QueryStore as QueryStoreInner;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Re-export commonly used types from jpx-core
pub use jpx_core::ast;
pub use jpx_core::query_library;
pub use jpx_core::{Category, Expression, FunctionInfo, FunctionRegistry, Runtime, compile, parse};

/// The JMESPath query engine.
///
/// `JpxEngine` is the main entry point for all jpx functionality. It combines:
///
/// - **JMESPath runtime** with 490+ extension functions
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
    pub(crate) runtime: Runtime,
    /// Function registry for introspection
    pub(crate) registry: FunctionRegistry,
    /// Discovery registry for cross-server tool search
    pub(crate) discovery: Arc<RwLock<DiscoveryRegistryInner>>,
    /// Query store for named queries
    pub(crate) queries: Arc<RwLock<QueryStoreInner>>,
    /// Whether to use strict mode (standard JMESPath only)
    pub(crate) strict: bool,
}

impl JpxEngine {
    /// Creates a new engine with all extension functions enabled.
    ///
    /// This is the standard way to create an engine with full functionality,
    /// including all 490+ extension functions.
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

        let mut registry = FunctionRegistry::new();
        registry.register_all();

        if !strict {
            registry.apply(&mut runtime);
        }

        Self {
            runtime,
            registry,
            discovery: Arc::new(RwLock::new(DiscoveryRegistryInner::new())),
            queries: Arc::new(RwLock::new(QueryStoreInner::new())),
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

    /// Creates a new engine from an [`EngineConfig`].
    ///
    /// Applies function filtering, loads query libraries and inline queries,
    /// and sets engine options from the config.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::{JpxEngine, EngineConfig};
    ///
    /// let config = EngineConfig::default();
    /// let engine = JpxEngine::from_config(config).unwrap();
    /// ```
    pub fn from_config(config: EngineConfig) -> Result<Self> {
        let strict = config.is_strict();
        let (runtime, registry) = config::build_runtime_from_config(&config.functions, strict);

        let discovery = Arc::new(RwLock::new(DiscoveryRegistryInner::new()));
        let queries = Arc::new(RwLock::new(QueryStoreInner::new()));

        // Load queries from config
        config::load_queries_into_store(&config.queries, &runtime, &queries)?;

        Ok(Self {
            runtime,
            registry,
            discovery,
            queries,
            strict,
        })
    }

    /// Returns a builder for constructing a `JpxEngine` with programmatic overrides.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::JpxEngine;
    ///
    /// let engine = JpxEngine::builder()
    ///     .strict(false)
    ///     .disable_category("geo")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> config::EngineBuilder {
        config::EngineBuilder::new()
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
    pub fn discovery(&self) -> &Arc<RwLock<DiscoveryRegistryInner>> {
        &self.discovery
    }

    /// Returns a reference to the query store.
    ///
    /// The query store manages named queries for the session.
    /// Access is through `Arc<RwLock<...>>` for thread-safe mutation.
    pub fn queries(&self) -> &Arc<RwLock<QueryStoreInner>> {
        &self.queries
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
            return Err(EngineErrorInner::InvalidExpression(
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
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .define(query)
            .pipe(Ok)
    }

    /// Get a stored query by name.
    pub fn get_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .get(name)
            .cloned())
    }

    /// Delete a stored query.
    pub fn delete_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .write()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .delete(name))
    }

    /// List all stored queries.
    pub fn list_queries(&self) -> Result<Vec<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .list()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Run a stored query.
    pub fn run_query(&self, name: &str, input: &Value) -> Result<Value> {
        let query = self
            .get_query(name)?
            .ok_or_else(|| EngineErrorInner::QueryNotFound(name.to_string()))?;

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
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .register(spec, replace))
    }

    /// Unregister a server from discovery.
    pub fn unregister_discovery(&self, server_name: &str) -> Result<bool> {
        Ok(self
            .discovery
            .write()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .unregister(server_name))
    }

    /// Query tools across registered servers.
    pub fn query_tools(&self, query: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .query(query, top_k))
    }

    /// Find tools similar to a given tool.
    pub fn similar_tools(&self, tool_id: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .similar(tool_id, top_k))
    }

    /// List all registered discovery servers.
    pub fn list_discovery_servers(&self) -> Result<Vec<ServerSummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .list_servers())
    }

    /// List discovery categories.
    pub fn list_discovery_categories(&self) -> Result<HashMap<String, CategorySummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .list_categories())
    }

    /// Get discovery index stats.
    pub fn discovery_index_stats(&self) -> Result<Option<IndexStats>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineErrorInner::Internal(e.to_string()))?
            .index_stats())
    }

    /// Get the discovery schema.
    pub fn get_discovery_schema(&self) -> Value {
        DiscoveryRegistryInner::get_schema()
    }
}

impl Default for JpxEngine {
    fn default() -> Self {
        Self::new()
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

    // =========================================================================
    // Construction tests
    // =========================================================================

    #[test]
    fn test_with_options_non_strict() {
        let engine = JpxEngine::with_options(false);
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_with_options_strict() {
        let engine = JpxEngine::with_options(true);
        assert!(engine.is_strict());
    }

    #[test]
    fn test_from_config_default() {
        let config = EngineConfig::default();
        let engine = JpxEngine::from_config(config).unwrap();
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_builder_default() {
        let engine = JpxEngine::builder().build().unwrap();
        assert!(!engine.is_strict());
    }

    // =========================================================================
    // Accessor tests
    // =========================================================================

    #[test]
    fn test_runtime_accessor() {
        let engine = JpxEngine::new();
        let runtime = engine.runtime();
        // Verify we can compile an expression through the runtime reference
        let expr = runtime.compile("length(@)").unwrap();
        let data = json!([1, 2, 3]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn test_registry_accessor() {
        let engine = JpxEngine::new();
        let registry = engine.registry();
        // The registry should contain functions after register_all()
        assert!(registry.get_function("upper").is_some());
        assert!(registry.get_function("lower").is_some());
        assert!(registry.is_enabled("upper"));
    }

    #[test]
    fn test_discovery_accessor() {
        let engine = JpxEngine::new();
        let discovery = engine.discovery();
        // Should be able to acquire a read lock and inspect the empty registry
        let guard = discovery.read().unwrap();
        assert!(guard.list_servers().is_empty());
    }

    #[test]
    fn test_queries_accessor() {
        let engine = JpxEngine::new();
        let queries = engine.queries();
        // Should be able to acquire a read lock and inspect the empty store
        let guard = queries.read().unwrap();
        assert!(guard.is_empty());
    }

    // =========================================================================
    // Query store tests (via engine)
    // =========================================================================

    #[test]
    fn test_define_query_with_description() {
        let engine = JpxEngine::new();
        engine
            .define_query(
                "named".to_string(),
                "length(@)".to_string(),
                Some("Counts elements".to_string()),
            )
            .unwrap();

        let query = engine.get_query("named").unwrap().unwrap();
        assert_eq!(query.expression, "length(@)");
        assert_eq!(query.description, Some("Counts elements".to_string()));
    }

    #[test]
    fn test_define_query_invalid_expression() {
        let engine = JpxEngine::new();
        let result = engine.define_query("bad".to_string(), "invalid[".to_string(), None);
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::InvalidExpression(_) => {} // expected
            other => panic!("Expected InvalidExpression, got {:?}", other),
        }
    }

    #[test]
    fn test_define_query_overwrite() {
        let engine = JpxEngine::new();

        // First define returns None (no previous query)
        let first = engine
            .define_query("q".to_string(), "length(@)".to_string(), None)
            .unwrap();
        assert!(first.is_none());

        // Second define with same name returns Some(old)
        let second = engine
            .define_query("q".to_string(), "keys(@)".to_string(), None)
            .unwrap();
        assert!(second.is_some());
        let old = second.unwrap();
        assert_eq!(old.expression, "length(@)");

        // Current value should be the new expression
        let current = engine.get_query("q").unwrap().unwrap();
        assert_eq!(current.expression, "keys(@)");
    }

    #[test]
    fn test_get_query_nonexistent() {
        let engine = JpxEngine::new();
        let result = engine.get_query("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_query_nonexistent() {
        let engine = JpxEngine::new();
        let result = engine.delete_query("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_run_query_not_found() {
        let engine = JpxEngine::new();
        let result = engine.run_query("nonexistent", &json!({}));
        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::QueryNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected QueryNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_list_queries_empty() {
        let engine = JpxEngine::new();
        let queries = engine.list_queries().unwrap();
        assert!(queries.is_empty());
    }

    #[test]
    fn test_list_queries_multiple() {
        let engine = JpxEngine::new();
        engine
            .define_query("alpha".to_string(), "a".to_string(), None)
            .unwrap();
        engine
            .define_query("beta".to_string(), "b".to_string(), None)
            .unwrap();
        engine
            .define_query("gamma".to_string(), "c".to_string(), None)
            .unwrap();

        let queries = engine.list_queries().unwrap();
        assert_eq!(queries.len(), 3);
    }

    // =========================================================================
    // Discovery tests (via engine)
    // =========================================================================

    #[test]
    fn test_register_discovery_duplicate() {
        let engine = JpxEngine::new();

        let spec: DiscoverySpec = serde_json::from_value(json!({
            "server": {"name": "dup-server", "version": "1.0.0"},
            "tools": [
                {"name": "tool_a", "description": "Tool A", "tags": ["test"]}
            ]
        }))
        .unwrap();

        // First registration succeeds
        let first = engine.register_discovery(spec.clone(), false).unwrap();
        assert!(first.ok);

        // Second registration without replace fails
        let second = engine.register_discovery(spec, false).unwrap();
        assert!(!second.ok);
        assert!(second.warnings[0].contains("already registered"));
    }

    #[test]
    fn test_unregister_nonexistent() {
        let engine = JpxEngine::new();
        let result = engine.unregister_discovery("nonexistent").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_discovery_index_stats_empty() {
        let engine = JpxEngine::new();
        let stats = engine.discovery_index_stats().unwrap();
        assert!(stats.is_none());
    }

    #[test]
    fn test_get_discovery_schema() {
        let engine = JpxEngine::new();
        let schema = engine.get_discovery_schema();
        assert!(schema.is_object());
        assert!(schema.get("$schema").is_some());
        assert_eq!(
            schema.get("$schema").unwrap().as_str().unwrap(),
            "http://json-schema.org/draft-07/schema#"
        );
        assert!(schema.get("title").is_some());
        assert!(schema.get("properties").is_some());
    }
}
