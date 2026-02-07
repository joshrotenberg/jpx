//! Engine configuration via `jpx.toml`.
//!
//! Provides declarative configuration for the jpx engine, supporting function
//! filtering, query libraries, and engine settings. Configuration is loaded
//! from multiple sources with layered overrides:
//!
//! 1. **Defaults** -- `EngineConfig::default()` (strict=false, all functions enabled)
//! 2. **Global** -- `~/.config/jpx/jpx.toml` (via `dirs::config_dir()`)
//! 3. **Project-local** -- Walk up from CWD looking for `jpx.toml`
//! 4. **Env override** -- `$JPX_CONFIG` points to a specific file
//! 5. **Programmatic** -- CLI flags, MCP args, builder calls
//!
//! # Example
//!
//! ```toml
//! [engine]
//! strict = false
//!
//! [functions]
//! disabled_categories = ["geo", "phonetic"]
//! disabled_functions = ["env"]
//!
//! [queries]
//! libraries = ["~/.config/jpx/common.jpx"]
//!
//! [queries.inline]
//! active-users = { expression = "users[?active].name", description = "Get active user names" }
//! ```

use crate::JpxEngine;
use crate::error::EngineError;
use jpx_core::query_library::QueryLibrary;
use jpx_core::{FunctionRegistry, Runtime};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Top-level configuration for the jpx engine.
///
/// Parsed from `jpx.toml` files. All fields have sensible defaults.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Engine-level settings.
    pub engine: EngineSection,
    /// Function filtering settings.
    pub functions: FunctionsSection,
    /// Query library and inline query settings.
    pub queries: QueriesSection,
}

/// Engine-level settings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EngineSection {
    /// If true, only standard JMESPath functions are available for evaluation.
    pub strict: bool,
}

/// Function filtering configuration.
///
/// Supports two mutually exclusive approaches:
/// - **Blocklist** (default): everything enabled, opt out with `disabled_categories`/`disabled_functions`
/// - **Allowlist**: only specified categories enabled via `enabled_categories`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FunctionsSection {
    /// Categories to disable (blocklist approach).
    pub disabled_categories: Vec<String>,
    /// Individual functions to disable.
    pub disabled_functions: Vec<String>,
    /// If set, only these categories are enabled (allowlist approach).
    /// Mutually exclusive with `disabled_categories`.
    pub enabled_categories: Option<Vec<String>>,
}

/// Query configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct QueriesSection {
    /// Paths to `.jpx` query library files to load.
    pub libraries: Vec<String>,
    /// Inline named queries.
    pub inline: HashMap<String, InlineQuery>,
}

/// An inline named query defined in the config file.
#[derive(Debug, Clone, Deserialize)]
pub struct InlineQuery {
    /// The JMESPath expression.
    pub expression: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

impl EngineConfig {
    /// Parses an `EngineConfig` from a TOML file.
    pub fn from_file(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            EngineError::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;
        toml::from_str(&content).map_err(|e| {
            EngineError::ConfigError(format!("Failed to parse {}: {}", path.display(), e))
        })
    }

    /// Discovers and merges configuration from standard locations.
    ///
    /// Loads configs in order (later overrides earlier):
    /// 1. Global: `~/.config/jpx/jpx.toml`
    /// 2. Project-local: `jpx.toml` found by walking up from CWD
    /// 3. Env override: `$JPX_CONFIG`
    pub fn discover() -> crate::Result<Self> {
        let mut config = Self::default();

        // 1. Global config
        if let Some(global_path) = global_config_path()
            && global_path.exists()
        {
            let global = Self::from_file(&global_path)?;
            config = config.merge(global);
        }

        // 2. Project-local config (walk up from CWD)
        if let Some(local_path) = find_project_config() {
            let local = Self::from_file(&local_path)?;
            config = config.merge(local);
        }

        // 3. Env override
        if let Ok(env_path) = std::env::var("JPX_CONFIG") {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                let env_config = Self::from_file(&path)?;
                config = config.merge(env_config);
            }
        }

        Ok(config)
    }

    /// Merges another config into this one.
    ///
    /// Merge semantics:
    /// - Scalars (`strict`): later wins
    /// - `disabled_categories` / `disabled_functions`: union
    /// - `enabled_categories`: later replaces
    /// - `queries.libraries`: concatenate
    /// - `queries.inline`: later keys override same-name
    pub fn merge(mut self, other: Self) -> Self {
        // Engine section: later wins
        if other.engine.strict {
            self.engine.strict = true;
        }

        // Functions section
        if let Some(enabled) = other.functions.enabled_categories {
            // Allowlist: later replaces entirely
            self.functions.enabled_categories = Some(enabled);
            // Clear blocklist when switching to allowlist
            self.functions.disabled_categories.clear();
        } else {
            // Blocklist: union
            for cat in other.functions.disabled_categories {
                if !self.functions.disabled_categories.contains(&cat) {
                    self.functions.disabled_categories.push(cat);
                }
            }
        }
        for func in other.functions.disabled_functions {
            if !self.functions.disabled_functions.contains(&func) {
                self.functions.disabled_functions.push(func);
            }
        }

        // Queries section
        self.queries.libraries.extend(other.queries.libraries);
        self.queries.inline.extend(other.queries.inline);

        self
    }
}

/// Builds a `JpxEngine` from configuration with programmatic overrides.
///
/// # Example
///
/// ```rust
/// use jpx_engine::config::EngineBuilder;
///
/// let engine = EngineBuilder::new()
///     .strict(false)
///     .disable_category("geo")
///     .disable_function("env")
///     .build()
///     .unwrap();
/// ```
pub struct EngineBuilder {
    config: EngineConfig,
}

impl EngineBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Sets strict mode.
    pub fn strict(mut self, strict: bool) -> Self {
        self.config.engine.strict = strict;
        self
    }

    /// Adds a category to the disabled list.
    pub fn disable_category(mut self, cat: &str) -> Self {
        let cat = cat.to_string();
        if !self.config.functions.disabled_categories.contains(&cat) {
            self.config.functions.disabled_categories.push(cat);
        }
        self
    }

    /// Adds a function to the disabled list.
    pub fn disable_function(mut self, name: &str) -> Self {
        let name = name.to_string();
        if !self.config.functions.disabled_functions.contains(&name) {
            self.config.functions.disabled_functions.push(name);
        }
        self
    }

    /// Sets the allowlist of enabled categories (replaces any blocklist).
    pub fn enable_categories(mut self, cats: Vec<String>) -> Self {
        self.config.functions.enabled_categories = Some(cats);
        self.config.functions.disabled_categories.clear();
        self
    }

    /// Adds a query library path.
    pub fn load_library(mut self, path: &str) -> Self {
        self.config.queries.libraries.push(path.to_string());
        self
    }

    /// Adds an inline query.
    pub fn inline_query(mut self, name: &str, expr: &str, desc: Option<&str>) -> Self {
        self.config.queries.inline.insert(
            name.to_string(),
            InlineQuery {
                expression: expr.to_string(),
                description: desc.map(|s| s.to_string()),
            },
        );
        self
    }

    /// Applies an `EngineConfig` (merges into the builder's config).
    pub fn config(mut self, config: EngineConfig) -> Self {
        self.config = self.config.merge(config);
        self
    }

    /// Builds the `JpxEngine`.
    pub fn build(self) -> crate::Result<JpxEngine> {
        JpxEngine::from_config(self.config)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Shared helpers for building Runtime + Registry from config
// =============================================================================

/// Builds a `Runtime` and `FunctionRegistry` from function configuration.
///
/// This is the shared logic used by both `JpxEngine::from_config` and the CLI's
/// `create_configured_runtime`. It handles:
/// - Registering builtin functions
/// - Applying allowlist/blocklist filtering
/// - Registering enabled extension functions on the runtime
pub fn build_runtime_from_config(
    functions_config: &FunctionsSection,
    strict: bool,
) -> (Runtime, FunctionRegistry) {
    use crate::introspection::parse_category;

    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();

    let mut registry = FunctionRegistry::new();

    if let Some(ref enabled_cats) = functions_config.enabled_categories {
        // Allowlist mode: only register specified categories
        for cat_name in enabled_cats {
            if let Some(cat) = parse_category(cat_name) {
                registry.register_category(cat);
            }
        }
        // Always include Standard category
        registry.register_category(jpx_core::Category::Standard);
    } else {
        // Default: register all, then disable
        registry.register_all();

        // Disable categories
        for cat_name in &functions_config.disabled_categories {
            if let Some(cat) = parse_category(cat_name) {
                let names: Vec<String> = registry
                    .functions_in_category(cat)
                    .map(|f| f.name.to_string())
                    .collect();
                for name in &names {
                    registry.disable_function(name);
                }
            }
        }
    }

    // Disable individual functions
    for func_name in &functions_config.disabled_functions {
        registry.disable_function(func_name);
    }

    // Apply to runtime (unless strict)
    if !strict {
        registry.apply(&mut runtime);
    }

    (runtime, registry)
}

/// Loads query libraries and inline queries into a query store.
pub fn load_queries_into_store(
    queries_config: &QueriesSection,
    runtime: &Runtime,
    queries: &Arc<RwLock<crate::QueryStore>>,
) -> crate::Result<()> {
    // Load .jpx library files
    for lib_path in &queries_config.libraries {
        let expanded = expand_tilde(lib_path);
        let path = Path::new(&expanded);
        if !path.exists() {
            continue; // silently skip missing libraries
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            EngineError::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let library = QueryLibrary::parse(&content).map_err(|e| {
            EngineError::ConfigError(format!("Failed to parse {}: {}", path.display(), e))
        })?;

        let mut store = queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?;

        for named_query in library.list() {
            // Validate expression
            if runtime.compile(&named_query.expression).is_ok() {
                store.define(crate::StoredQuery {
                    name: named_query.name.clone(),
                    expression: named_query.expression.clone(),
                    description: named_query.description.clone(),
                });
            }
        }
    }

    // Load inline queries
    if !queries_config.inline.is_empty() {
        let mut store = queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?;

        for (name, query) in &queries_config.inline {
            // Validate expression
            if runtime.compile(&query.expression).is_ok() {
                store.define(crate::StoredQuery {
                    name: name.clone(),
                    expression: query.expression.clone(),
                    description: query.description.clone(),
                });
            }
        }
    }

    Ok(())
}

// =============================================================================
// Path helpers
// =============================================================================

/// Returns the global config path: `~/.config/jpx/jpx.toml`
fn global_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("jpx").join("jpx.toml"))
}

/// Walks up from CWD looking for `jpx.toml`.
fn find_project_config() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        let candidate = dir.join("jpx.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}

/// Expands `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest).to_string_lossy().into_owned();
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert!(!config.engine.strict);
        assert!(config.functions.disabled_categories.is_empty());
        assert!(config.functions.disabled_functions.is_empty());
        assert!(config.functions.enabled_categories.is_none());
        assert!(config.queries.libraries.is_empty());
        assert!(config.queries.inline.is_empty());
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[engine]
strict = true

[functions]
disabled_categories = ["geo", "phonetic"]
disabled_functions = ["env"]

[queries]
libraries = ["~/.config/jpx/common.jpx"]

[queries.inline]
active-users = { expression = "users[?active].name", description = "Get active user names" }
"#;
        let config: EngineConfig = toml::from_str(toml).unwrap();
        assert!(config.engine.strict);
        assert_eq!(
            config.functions.disabled_categories,
            vec!["geo", "phonetic"]
        );
        assert_eq!(config.functions.disabled_functions, vec!["env"]);
        assert_eq!(config.queries.libraries.len(), 1);
        assert!(config.queries.inline.contains_key("active-users"));
    }

    #[test]
    fn test_merge_scalars() {
        let base = EngineConfig::default();
        let overlay = EngineConfig {
            engine: EngineSection { strict: true },
            ..Default::default()
        };
        let merged = base.merge(overlay);
        assert!(merged.engine.strict);
    }

    #[test]
    fn test_merge_disabled_union() {
        let base = EngineConfig {
            functions: FunctionsSection {
                disabled_categories: vec!["geo".to_string()],
                disabled_functions: vec!["env".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let overlay = EngineConfig {
            functions: FunctionsSection {
                disabled_categories: vec!["geo".to_string(), "phonetic".to_string()],
                disabled_functions: vec!["uuid".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let merged = base.merge(overlay);
        assert_eq!(merged.functions.disabled_categories.len(), 2); // geo + phonetic (no dups)
        assert_eq!(merged.functions.disabled_functions.len(), 2); // env + uuid
    }

    #[test]
    fn test_merge_enabled_replaces() {
        let base = EngineConfig {
            functions: FunctionsSection {
                disabled_categories: vec!["geo".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let overlay = EngineConfig {
            functions: FunctionsSection {
                enabled_categories: Some(vec!["string".to_string(), "math".to_string()]),
                ..Default::default()
            },
            ..Default::default()
        };
        let merged = base.merge(overlay);
        assert_eq!(
            merged.functions.enabled_categories,
            Some(vec!["string".to_string(), "math".to_string()])
        );
        assert!(merged.functions.disabled_categories.is_empty());
    }

    #[test]
    fn test_merge_queries_concat() {
        let base = EngineConfig {
            queries: QueriesSection {
                libraries: vec!["a.jpx".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let overlay = EngineConfig {
            queries: QueriesSection {
                libraries: vec!["b.jpx".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let merged = base.merge(overlay);
        assert_eq!(merged.queries.libraries, vec!["a.jpx", "b.jpx"]);
    }

    #[test]
    fn test_builder() {
        let engine = EngineBuilder::new()
            .strict(false)
            .disable_category("geo")
            .disable_function("env")
            .build()
            .unwrap();

        // Engine should work
        let result = engine
            .evaluate("length(@)", &serde_json::json!([1, 2, 3]))
            .unwrap();
        assert_eq!(result, serde_json::json!(3));
    }

    #[test]
    fn test_builder_strict() {
        let engine = EngineBuilder::new().strict(true).build().unwrap();
        assert!(engine.is_strict());
    }

    #[test]
    fn test_from_config_with_disabled_functions() {
        let config = EngineConfig {
            functions: FunctionsSection {
                disabled_functions: vec!["upper".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let engine = JpxEngine::from_config(config).unwrap();

        // upper should be disabled in introspection
        assert!(engine.describe_function("upper").is_none());

        // But standard functions still work
        let result = engine
            .evaluate("length(@)", &serde_json::json!([1, 2, 3]))
            .unwrap();
        assert_eq!(result, serde_json::json!(3));
    }

    #[test]
    fn test_from_config_with_inline_queries() {
        let config = EngineConfig {
            queries: QueriesSection {
                inline: {
                    let mut m = HashMap::new();
                    m.insert(
                        "count".to_string(),
                        InlineQuery {
                            expression: "length(@)".to_string(),
                            description: Some("Count items".to_string()),
                        },
                    );
                    m
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let engine = JpxEngine::from_config(config).unwrap();

        let result = engine
            .run_query("count", &serde_json::json!([1, 2, 3]))
            .unwrap();
        assert_eq!(result, serde_json::json!(3));
    }

    #[test]
    fn test_expand_tilde() {
        let result = expand_tilde("/absolute/path");
        assert_eq!(result, "/absolute/path");

        let result = expand_tilde("relative/path");
        assert_eq!(result, "relative/path");

        // ~ expansion depends on home dir being available
        let result = expand_tilde("~/some/path");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(result, home.join("some/path").to_string_lossy().as_ref());
        }
    }
}
