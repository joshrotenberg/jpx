//! Discovery Protocol implementation.
//!
//! This module implements a protocol for capability registration and search
//! across servers. It uses BM25 search indexing for efficient tool discovery.
//!
//! # Discovery Spec
//!
//! Servers can register their tools using a structured discovery spec:
//!
//! ```json
//! {
//!   "server": {"name": "my-server", "version": "1.0.0"},
//!   "tools": [
//!     {"name": "my_tool", "description": "Does something useful", "tags": ["read"]}
//!   ]
//! }
//! ```

use crate::bm25::{Bm25Index, IndexOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Common English stop words to filter from search indexing.
/// These words are too common to be useful for search relevance.
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is", "it",
    "its", "of", "on", "or", "that", "the", "to", "was", "were", "will", "with", "this", "but",
    "they", "have", "had", "what", "when", "where", "who", "which", "why", "how", "all", "each",
    "every", "both", "few", "more", "most", "other", "some", "such", "no", "nor", "not", "only",
    "own", "same", "so", "than", "too", "very", "just", "can", "could", "should", "would", "may",
    "might", "must", "shall", "about", "above", "after", "again", "against", "below", "between",
    "into", "through", "during", "before", "under", "over",
];

/// Preprocess text for search indexing.
///
/// This function cleans up text before indexing to improve search relevance:
/// 1. Strips JMESPath literal syntax (backticks, escaped quotes)
/// 2. Expands common regex patterns to natural language
/// 3. Converts snake_case to separate words
/// 4. Removes noise characters
fn preprocess_for_search(text: &str) -> String {
    let mut result = text.to_string();

    // Strip JMESPath backtick literals: `"..."` -> ...
    // This handles patterns like `"\n"` -> newline, `"\\d+"` -> digits
    result = strip_jmespath_literals(&result);

    // Expand common regex patterns to natural language
    result = expand_regex_patterns(&result);

    // Convert snake_case and camelCase to separate words
    result = expand_identifiers(&result);

    // Clean up extra whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip JMESPath backtick literal syntax from text.
fn strip_jmespath_literals(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '`' {
            // Skip backtick and its contents, but extract meaningful parts
            let mut inner = String::new();
            for inner_c in chars.by_ref() {
                if inner_c == '`' {
                    break;
                }
                inner.push(inner_c);
            }
            // Extract content from JSON string if it looks like `"..."`
            let trimmed = inner.trim();
            if trimmed.starts_with('"') && trimmed.ends_with('"') {
                let content = &trimmed[1..trimmed.len() - 1];
                // Expand escape sequences to words
                let expanded = expand_escape_sequences(content);
                result.push(' ');
                result.push_str(&expanded);
                result.push(' ');
            } else {
                // Just include the inner content
                result.push(' ');
                result.push_str(trimmed);
                result.push(' ');
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Expand escape sequences to natural language.
fn expand_escape_sequences(text: &str) -> String {
    text.replace("\\n", " newline linebreak ")
        .replace("\\r", " return ")
        .replace("\\t", " tab ")
        .replace("\\s", " whitespace space ")
        .replace("\\d", " digit number numeric ")
        .replace("\\w", " word alphanumeric ")
        .replace("\\b", " boundary ")
        .replace("\\\\", " ")
}

/// Expand common regex patterns to natural language.
fn expand_regex_patterns(text: &str) -> String {
    text
        // Common regex character classes
        .replace("[0-9]", " digit number ")
        .replace("[a-z]", " letter lowercase ")
        .replace("[A-Z]", " letter uppercase ")
        .replace("[a-zA-Z]", " letter alphabetic ")
        .replace("[^>]", " ")
        .replace(".*", " any anything ")
        .replace(".+", " one more any ")
        .replace("\\d+", " digits numbers numeric ")
        .replace("\\w+", " words alphanumeric ")
        .replace("\\s+", " whitespace spaces ")
        .replace("\\S+", " nonwhitespace ")
        // Clean up regex metacharacters
        .replace(
            ['[', ']', '(', ')', '{', '}', '*', '+', '?', '^', '$', '|'],
            " ",
        )
}

/// Expand snake_case and camelCase identifiers to separate words.
fn expand_identifiers(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);

    for word in text.split_whitespace() {
        // Handle snake_case
        if word.contains('_') {
            for part in word.split('_') {
                if !part.is_empty() {
                    result.push_str(part);
                    result.push(' ');
                }
            }
            // Also keep the original for exact matches
            result.push_str(word);
            result.push(' ');
        }
        // Handle camelCase (basic implementation)
        else if word.chars().any(|c| c.is_uppercase()) && word.chars().any(|c| c.is_lowercase()) {
            let mut prev_was_upper = false;
            let mut current_word = String::new();

            for c in word.chars() {
                if c.is_uppercase() && !prev_was_upper && !current_word.is_empty() {
                    result.push_str(&current_word.to_lowercase());
                    result.push(' ');
                    current_word.clear();
                }
                current_word.push(c);
                prev_was_upper = c.is_uppercase();
            }
            if !current_word.is_empty() {
                result.push_str(&current_word.to_lowercase());
                result.push(' ');
            }
            // Also keep the original
            result.push_str(word);
            result.push(' ');
        } else {
            result.push_str(word);
            result.push(' ');
        }
    }

    result
}

/// Discovery spec - the schema MCP servers use to register their tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct DiscoverySpec {
    /// JSON Schema reference (optional)
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Server metadata
    pub server: ServerInfo,

    /// List of tools provided by this server
    pub tools: Vec<ToolSpec>,

    /// Category definitions (optional)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub categories: HashMap<String, CategoryInfo>,
}

/// Server metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ServerInfo {
    /// Server name (required)
    pub name: String,

    /// Server version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Server description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ToolSpec {
    /// Tool name (required)
    pub name: String,

    /// Alternative names/aliases
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// Primary category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Subcategory within the primary category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,

    /// Tags for filtering and search
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Short summary (for search results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Full description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter definitions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ParamSpec>,

    /// Return type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnSpec>,

    /// Usage examples
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<ExampleSpec>,

    /// Related tools (author-declared relationships)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,

    /// Version when tool was added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// Stability level (stable, beta, deprecated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<String>,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,

    /// Parameter type (string, number, boolean, object, array)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub param_type: Option<String>,

    /// Whether parameter is required
    #[serde(default)]
    pub required: bool,

    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allowed values (for enums)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Return type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ReturnSpec {
    /// Return type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    /// Description of return value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Example specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ExampleSpec {
    /// Example description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Example arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,

    /// Expected result (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Category information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct CategoryInfo {
    /// Category description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Subcategories
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcategories: Vec<String>,
}

/// Discovery registry - holds registered specs and search index
#[derive(Debug)]
pub struct DiscoveryRegistry {
    /// Registered servers: name -> spec
    servers: HashMap<String, DiscoverySpec>,

    /// All tools flattened for indexing: tool_id -> (server_name, tool_spec)
    tools: HashMap<String, (String, ToolSpec)>,

    /// BM25 search index (rebuilt on registration changes)
    index: Option<Bm25Index>,
}

impl Default for DiscoveryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            tools: HashMap::new(),
            index: None,
        }
    }

    /// Register a discovery spec
    pub fn register(&mut self, spec: DiscoverySpec, replace: bool) -> RegistrationResult {
        let server_name = spec.server.name.clone();

        // Check if server already registered
        if self.servers.contains_key(&server_name) && !replace {
            return RegistrationResult {
                ok: false,
                tools_indexed: 0,
                warnings: vec![format!(
                    "Server '{}' already registered. Use replace=true to update.",
                    server_name
                )],
            };
        }

        // Remove old tools from this server if replacing
        if replace {
            self.tools.retain(|_, (srv, _)| srv != &server_name);
        }

        // Add new tools
        let mut warnings = Vec::new();
        let mut tools_added = 0;

        for tool in &spec.tools {
            let tool_id = format!("{}:{}", server_name, tool.name);

            if self.tools.contains_key(&tool_id) && !replace {
                warnings.push(format!("Tool '{}' already exists, skipping", tool_id));
                continue;
            }

            self.tools
                .insert(tool_id, (server_name.clone(), tool.clone()));
            tools_added += 1;
        }

        // Store the spec
        self.servers.insert(server_name, spec);

        // Rebuild the search index
        self.rebuild_index();

        RegistrationResult {
            ok: true,
            tools_indexed: tools_added,
            warnings,
        }
    }

    /// Unregister a server
    pub fn unregister(&mut self, server_name: &str) -> bool {
        if self.servers.remove(server_name).is_some() {
            self.tools.retain(|_, (srv, _)| srv != server_name);
            self.rebuild_index();
            true
        } else {
            false
        }
    }

    /// Rebuild the BM25 search index from all registered tools
    fn rebuild_index(&mut self) {
        if self.tools.is_empty() {
            self.index = None;
            return;
        }

        // Convert tools to indexable documents with preprocessed text
        let docs: Vec<Value> = self
            .tools
            .iter()
            .map(|(id, (server, tool))| {
                let summary = tool.summary.as_deref().unwrap_or("");
                let description = tool.description.as_deref().unwrap_or("");

                // Preprocess text fields for better search
                let expanded_summary = preprocess_for_search(summary);
                let expanded_description = preprocess_for_search(description);

                // Also preprocess examples for searchable content
                let examples_text: String = tool
                    .examples
                    .iter()
                    .filter_map(|ex| ex.description.as_ref())
                    .map(|d| preprocess_for_search(d))
                    .collect::<Vec<_>>()
                    .join(" ");

                serde_json::json!({
                    "id": id,
                    "server": server,
                    "name": tool.name,
                    "aliases": tool.aliases.join(" "),
                    "category": tool.category.as_deref().unwrap_or(""),
                    "tags": tool.tags.join(" "),
                    "summary": summary,
                    "description": description,
                    "params": tool.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(" "),
                    // Expanded fields for better search
                    "expanded_summary": expanded_summary,
                    "expanded_description": expanded_description,
                    "expanded_examples": examples_text,
                })
            })
            .collect();

        let options = IndexOptions {
            fields: vec![
                "name".to_string(),
                "aliases".to_string(),
                "category".to_string(),
                "tags".to_string(),
                "summary".to_string(),
                "description".to_string(),
                "params".to_string(),
                // Include expanded fields in search
                "expanded_summary".to_string(),
                "expanded_description".to_string(),
                "expanded_examples".to_string(),
            ],
            id_field: Some("id".to_string()),
            stopwords: STOP_WORDS.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };

        self.index = Some(Bm25Index::build(&docs, options));
    }

    /// Query tools across all registered servers
    pub fn query(&self, query: &str, top_k: usize) -> Vec<ToolQueryResult> {
        let Some(index) = &self.index else {
            return Vec::new();
        };

        let results = index.search(query, top_k);

        results
            .into_iter()
            .filter_map(|r| {
                let (server, tool) = self.tools.get(&r.id)?;
                Some(ToolQueryResult {
                    id: r.id,
                    server: server.clone(),
                    tool: tool.clone(),
                    score: r.score,
                    matches: r.matches,
                })
            })
            .collect()
    }

    /// Find tools similar to a given tool
    pub fn similar(&self, tool_id: &str, top_k: usize) -> Vec<ToolQueryResult> {
        let Some(index) = &self.index else {
            return Vec::new();
        };

        let results = index.similar(tool_id, top_k);

        results
            .into_iter()
            .filter_map(|r| {
                let (server, tool) = self.tools.get(&r.id)?;
                Some(ToolQueryResult {
                    id: r.id,
                    server: server.clone(),
                    tool: tool.clone(),
                    score: r.score,
                    matches: r.matches,
                })
            })
            .collect()
    }

    /// List all registered servers
    pub fn list_servers(&self) -> Vec<ServerSummary> {
        self.servers
            .iter()
            .map(|(name, spec)| ServerSummary {
                name: name.clone(),
                version: spec.server.version.clone(),
                description: spec.server.description.clone(),
                tool_count: spec.tools.len(),
            })
            .collect()
    }

    /// List all categories across all servers
    pub fn list_categories(&self) -> HashMap<String, CategorySummary> {
        let mut categories: HashMap<String, CategorySummary> = HashMap::new();

        for (server, tool) in self.tools.values() {
            if let Some(cat) = &tool.category {
                let entry = categories.entry(cat.clone()).or_insert(CategorySummary {
                    name: cat.clone(),
                    tool_count: 0,
                    servers: Vec::new(),
                    subcategories: Vec::new(),
                });
                entry.tool_count += 1;
                if !entry.servers.contains(server) {
                    entry.servers.push(server.clone());
                }
                if let Some(subcat) = tool
                    .subcategory
                    .as_ref()
                    .filter(|s| !entry.subcategories.contains(s))
                {
                    entry.subcategories.push(subcat.clone());
                }
            }
        }

        categories
    }

    /// Get index statistics
    pub fn index_stats(&self) -> Option<IndexStats> {
        let index = self.index.as_ref()?;

        Some(IndexStats {
            doc_count: index.doc_count,
            term_count: index.terms.len(),
            avg_doc_length: index.avg_doc_length,
            server_count: self.servers.len(),
            top_terms: index.terms().into_iter().take(20).collect(),
        })
    }

    /// Get the discovery schema as JSON
    pub fn get_schema() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": "https://jpx.dev/schemas/mcp-discovery/v1.json",
            "title": "MCP Discovery Spec",
            "description": "Schema for registering MCP server capabilities with jpx",
            "type": "object",
            "required": ["server", "tools"],
            "properties": {
                "$schema": {
                    "type": "string",
                    "description": "JSON Schema reference"
                },
                "server": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": {"type": "string", "description": "Server name"},
                        "version": {"type": "string", "description": "Server version"},
                        "description": {"type": "string", "description": "Server description"}
                    }
                },
                "tools": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {"type": "string", "description": "Tool name"},
                            "aliases": {"type": "array", "items": {"type": "string"}},
                            "category": {"type": "string"},
                            "subcategory": {"type": "string"},
                            "tags": {"type": "array", "items": {"type": "string"}},
                            "summary": {"type": "string", "description": "Short summary"},
                            "description": {"type": "string", "description": "Full description"},
                            "params": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["name"],
                                    "properties": {
                                        "name": {"type": "string"},
                                        "type": {"type": "string"},
                                        "required": {"type": "boolean"},
                                        "description": {"type": "string"},
                                        "enum": {"type": "array", "items": {"type": "string"}},
                                        "default": {}
                                    }
                                }
                            },
                            "returns": {
                                "type": "object",
                                "properties": {
                                    "type": {"type": "string"},
                                    "description": {"type": "string"}
                                }
                            },
                            "examples": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "description": {"type": "string"},
                                        "args": {},
                                        "result": {}
                                    }
                                }
                            },
                            "related": {"type": "array", "items": {"type": "string"}},
                            "since": {"type": "string"},
                            "stability": {"type": "string", "enum": ["stable", "beta", "deprecated"]}
                        }
                    }
                },
                "categories": {
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "properties": {
                            "description": {"type": "string"},
                            "subcategories": {"type": "array", "items": {"type": "string"}}
                        }
                    }
                }
            }
        })
    }
}

/// Result of registering a discovery spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub ok: bool,
    pub tools_indexed: usize,
    pub warnings: Vec<String>,
}

/// Tool query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolQueryResult {
    pub id: String,
    pub server: String,
    pub tool: ToolSpec,
    pub score: f64,
    pub matches: HashMap<String, Vec<String>>,
}

/// Server summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSummary {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub tool_count: usize,
}

/// Category summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub name: String,
    pub tool_count: usize,
    pub servers: Vec<String>,
    pub subcategories: Vec<String>,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub doc_count: usize,
    pub term_count: usize,
    pub avg_doc_length: f64,
    pub server_count: usize,
    pub top_terms: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> DiscoverySpec {
        serde_json::from_value(serde_json::json!({
            "server": {
                "name": "redisctl",
                "version": "0.5.0",
                "description": "Redis Enterprise management"
            },
            "tools": [
                {
                    "name": "create_cluster",
                    "category": "clusters",
                    "tags": ["write", "provisioning"],
                    "summary": "Create a new Redis cluster",
                    "description": "Creates a new Redis Enterprise cluster with specified configuration"
                },
                {
                    "name": "delete_cluster",
                    "category": "clusters",
                    "tags": ["write", "destructive"],
                    "summary": "Delete a cluster",
                    "description": "Permanently deletes a Redis cluster"
                },
                {
                    "name": "list_backups",
                    "category": "backups",
                    "tags": ["read"],
                    "summary": "List all backups",
                    "description": "Lists all available backups for a cluster"
                }
            ]
        })).unwrap()
    }

    #[test]
    fn test_register_spec() {
        let mut registry = DiscoveryRegistry::new();
        let spec = sample_spec();

        let result = registry.register(spec, false);

        assert!(result.ok);
        assert_eq!(result.tools_indexed, 3);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_query_tools() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let results = registry.query("cluster", 10);

        // All tools mention cluster in their descriptions, but cluster tools rank higher
        assert!(!results.is_empty());
        // Top results should be the cluster tools (they have "cluster" in name)
        let top_names: Vec<_> = results
            .iter()
            .take(2)
            .map(|r| r.tool.name.as_str())
            .collect();
        assert!(top_names.contains(&"create_cluster"));
        assert!(top_names.contains(&"delete_cluster"));
    }

    #[test]
    fn test_query_by_tag() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let results = registry.query("read", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool.name, "list_backups");
    }

    #[test]
    fn test_list_servers() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let servers = registry.list_servers();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "redisctl");
        assert_eq!(servers[0].tool_count, 3);
    }

    #[test]
    fn test_list_categories() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let categories = registry.list_categories();

        assert_eq!(categories.len(), 2);
        assert!(categories.contains_key("clusters"));
        assert!(categories.contains_key("backups"));
        assert_eq!(categories.get("clusters").unwrap().tool_count, 2);
    }

    #[test]
    fn test_unregister() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        assert!(registry.unregister("redisctl"));
        assert!(registry.list_servers().is_empty());
        assert!(registry.query("cluster", 10).is_empty());
    }

    #[test]
    fn test_replace_registration() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        // Try to register again without replace - should fail
        let result = registry.register(sample_spec(), false);
        assert!(!result.ok);

        // With replace - should succeed
        let result = registry.register(sample_spec(), true);
        assert!(result.ok);
    }

    #[test]
    fn test_similar_tools() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let similar = registry.similar("redisctl:create_cluster", 10);

        // delete_cluster should be similar (shares "cluster" terms)
        assert!(!similar.is_empty());
        assert_eq!(similar[0].tool.name, "delete_cluster");
    }

    #[test]
    fn test_minimal_spec() {
        let minimal: DiscoverySpec = serde_json::from_value(serde_json::json!({
            "server": {"name": "minimal"},
            "tools": [{"name": "foo"}]
        }))
        .unwrap();

        let mut registry = DiscoveryRegistry::new();
        let result = registry.register(minimal, false);

        assert!(result.ok);
        assert_eq!(result.tools_indexed, 1);
    }

    #[test]
    fn test_get_schema() {
        let schema = DiscoveryRegistry::get_schema();

        assert!(schema.get("$schema").is_some());
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_index_stats() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let stats = registry.index_stats().unwrap();

        assert_eq!(stats.doc_count, 3);
        assert_eq!(stats.server_count, 1);
        assert!(stats.term_count > 0);
    }

    // Preprocessing tests

    #[test]
    fn test_strip_jmespath_literals() {
        // Basic backtick literal with JSON string
        assert!(strip_jmespath_literals(r#"split text on `"\n"` newlines"#).contains("newline"));

        // Backtick with escaped regex
        let result = strip_jmespath_literals(r#"match `"\\d+"` digits"#);
        assert!(result.contains("digit"));

        // Multiple backticks
        let result = strip_jmespath_literals(r#"use `"\t"` for tabs and `"\n"` for lines"#);
        assert!(result.contains("tab"));
        assert!(result.contains("newline"));

        // Non-string backtick content preserved
        let result = strip_jmespath_literals(r#"literal `123` number"#);
        assert!(result.contains("123"));
    }

    #[test]
    fn test_expand_escape_sequences() {
        assert!(expand_escape_sequences(r"\n").contains("newline"));
        assert!(expand_escape_sequences(r"\t").contains("tab"));
        assert!(expand_escape_sequences(r"\d").contains("digit"));
        assert!(expand_escape_sequences(r"\w").contains("word"));
        assert!(expand_escape_sequences(r"\s").contains("whitespace"));
    }

    #[test]
    fn test_expand_regex_patterns() {
        assert!(expand_regex_patterns(r"\d+").contains("digits"));
        assert!(expand_regex_patterns(r"\w+").contains("words"));
        assert!(expand_regex_patterns(r"[0-9]").contains("digit"));
        assert!(expand_regex_patterns(r"[a-zA-Z]").contains("letter"));
        assert!(expand_regex_patterns(r".*").contains("any"));

        // Metacharacters should be stripped
        let result = expand_regex_patterns(r"foo[bar]+baz");
        assert!(!result.contains('['));
        assert!(!result.contains(']'));
        assert!(!result.contains('+'));
    }

    #[test]
    fn test_expand_identifiers() {
        // snake_case should expand
        let result = expand_identifiers("get_user_info");
        assert!(result.contains("get"));
        assert!(result.contains("user"));
        assert!(result.contains("info"));
        // Original preserved for exact match
        assert!(result.contains("get_user_info"));

        // camelCase should expand
        let result = expand_identifiers("getUserInfo");
        assert!(result.contains("get"));
        assert!(result.contains("user"));
        assert!(result.contains("info"));
        // Original preserved
        assert!(result.contains("getUserInfo"));

        // Simple words unchanged
        let result = expand_identifiers("simple");
        assert!(result.contains("simple"));
    }

    #[test]
    fn test_preprocess_for_search_integration() {
        // Full preprocessing pipeline
        let input = r#"Split on `"\n"` to get lines, use regex_extract for \d+ numbers"#;
        let result = preprocess_for_search(input);

        // Should contain expanded terms
        assert!(result.contains("newline") || result.contains("linebreak"));
        assert!(result.contains("digit") || result.contains("number"));
        assert!(result.contains("regex"));
        assert!(result.contains("extract"));

        // Should not have excess whitespace
        assert!(!result.contains("  "));
    }

    #[test]
    fn test_preprocess_preserves_search_terms() {
        // Make sure useful search terms aren't lost
        let input = "Create a new database connection";
        let result = preprocess_for_search(input);

        assert!(result.contains("Create"));
        assert!(result.contains("database"));
        assert!(result.contains("connection"));
    }

    #[test]
    fn test_search_with_preprocessed_content() {
        // Test that preprocessing improves search for escape-heavy descriptions
        let spec: DiscoverySpec = serde_json::from_value(serde_json::json!({
            "server": {"name": "text-tools"},
            "tools": [
                {
                    "name": "split_lines",
                    "summary": r#"Split text on newlines using `"\n"` delimiter"#,
                    "description": r#"Splits input string on newline characters. Use split(@, `"\n"`) syntax."#
                },
                {
                    "name": "extract_numbers",
                    "summary": r#"Extract numeric patterns with regex `"\\d+"`"#,
                    "description": r#"Uses regex_extract to find all \d+ digit sequences in text."#
                }
            ]
        }))
        .unwrap();

        let mut registry = DiscoveryRegistry::new();
        registry.register(spec, false);

        // Search for "newline" should find split_lines due to preprocessing
        let results = registry.query("newline", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].tool.name, "split_lines");

        // Search for "digit" should find extract_numbers
        let results = registry.query("digit", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].tool.name, "extract_numbers");
    }
}
