//! Mock MCP Server for testing discovery protocol
//!
//! This crate provides a configurable mock MCP server that can be used to test
//! jpx's discovery registration feature. Each instance can be configured with:
//!
//! - A unique server name
//! - A set of mock tools with customizable metadata
//! - Categories and tags for testing search functionality
//!
//! # Usage
//!
//! ```no_run
//! use mock_mcp_server::{MockMcpServer, MockToolConfig, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig {
//!         name: "test-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: "A test server".to_string(),
//!         tools: vec![
//!             MockToolConfig::simple("tool_one", "Does something"),
//!             MockToolConfig::full("tool_two", "Does something else")
//!                 .with_category("testing")
//!                 .with_tags(vec!["read", "safe"]),
//!         ],
//!         categories: vec![],
//!     };
//!
//!     MockMcpServer::run(config).await.unwrap();
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use tower_mcp::{CallToolResult, Error, McpRouter, StdioTransport, ToolBuilder};

/// Configuration for the mock server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name (used in discovery registration)
    pub name: String,
    /// Server version
    #[serde(default = "default_version")]
    pub version: String,
    /// Server description
    #[serde(default)]
    pub description: String,
    /// Tools to expose
    #[serde(default)]
    pub tools: Vec<MockToolConfig>,
    /// Categories metadata
    #[serde(default)]
    pub categories: Vec<CategoryConfig>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl ServerConfig {
    /// Create a new server config with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: default_version(),
            description: String::new(),
            tools: Vec::new(),
            categories: Vec::new(),
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a tool
    pub fn with_tool(mut self, tool: MockToolConfig) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add multiple tools
    pub fn with_tools(mut self, tools: Vec<MockToolConfig>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add a category
    pub fn with_category(mut self, category: CategoryConfig) -> Self {
        self.categories.push(category);
        self
    }

    /// Generate a discovery spec for this server
    pub fn to_discovery_spec(&self) -> Value {
        let tools: Vec<Value> = self.tools.iter().map(|t| t.to_spec()).collect();

        let mut spec = json!({
            "server": {
                "name": self.name,
                "version": self.version,
            },
            "tools": tools,
        });

        if !self.description.is_empty() {
            spec["server"]["description"] = json!(self.description);
        }

        if !self.categories.is_empty() {
            let cats: serde_json::Map<String, Value> = self
                .categories
                .iter()
                .map(|c| (c.name.clone(), c.to_spec()))
                .collect();
            spec["categories"] = json!(cats);
        }

        spec
    }
}

/// Configuration for a mock tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockToolConfig {
    /// Tool name
    pub name: String,
    /// Tool description
    #[serde(default)]
    pub description: String,
    /// Short summary
    #[serde(default)]
    pub summary: String,
    /// Primary category
    #[serde(default)]
    pub category: Option<String>,
    /// Subcategory
    #[serde(default)]
    pub subcategory: Option<String>,
    /// Tags for search
    #[serde(default)]
    pub tags: Vec<String>,
    /// Parameter definitions
    #[serde(default)]
    pub params: Vec<ParamConfig>,
    /// Related tools
    #[serde(default)]
    pub related: Vec<String>,
    /// Mock response to return when called
    #[serde(default)]
    pub mock_response: Option<Value>,
    /// Aliases for the tool
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl MockToolConfig {
    /// Create a simple tool with just a name and description
    pub fn simple(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            summary: String::new(),
            category: None,
            subcategory: None,
            tags: Vec::new(),
            params: Vec::new(),
            related: Vec::new(),
            mock_response: None,
            aliases: Vec::new(),
        }
    }

    /// Create a tool with full configuration capability
    pub fn full(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::simple(name, description)
    }

    /// Set the summary
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the subcategory
    pub fn with_subcategory(mut self, subcategory: impl Into<String>) -> Self {
        self.subcategory = Some(subcategory.into());
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, param: ParamConfig) -> Self {
        self.params.push(param);
        self
    }

    /// Add related tools
    pub fn with_related(mut self, related: Vec<impl Into<String>>) -> Self {
        self.related = related.into_iter().map(|r| r.into()).collect();
        self
    }

    /// Set the mock response
    pub fn with_response(mut self, response: Value) -> Self {
        self.mock_response = Some(response);
        self
    }

    /// Add aliases
    pub fn with_aliases(mut self, aliases: Vec<impl Into<String>>) -> Self {
        self.aliases = aliases.into_iter().map(|a| a.into()).collect();
        self
    }

    /// Convert to discovery spec format
    pub fn to_spec(&self) -> Value {
        let mut spec = json!({
            "name": self.name,
        });

        if !self.description.is_empty() {
            spec["description"] = json!(self.description);
        }
        if !self.summary.is_empty() {
            spec["summary"] = json!(self.summary);
        }
        if let Some(cat) = &self.category {
            spec["category"] = json!(cat);
        }
        if let Some(subcat) = &self.subcategory {
            spec["subcategory"] = json!(subcat);
        }
        if !self.tags.is_empty() {
            spec["tags"] = json!(self.tags);
        }
        if !self.params.is_empty() {
            spec["params"] = json!(self.params.iter().map(|p| p.to_spec()).collect::<Vec<_>>());
        }
        if !self.related.is_empty() {
            spec["related"] = json!(self.related);
        }
        if !self.aliases.is_empty() {
            spec["aliases"] = json!(self.aliases);
        }

        spec
    }
}

/// Configuration for a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamConfig {
    pub name: String,
    #[serde(rename = "type", default)]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
}

impl ParamConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: "string".to_string(),
            required: false,
            description: String::new(),
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_type(mut self, t: impl Into<String>) -> Self {
        self.param_type = t.into();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn to_spec(&self) -> Value {
        json!({
            "name": self.name,
            "type": self.param_type,
            "required": self.required,
            "description": self.description,
        })
    }
}

/// Configuration for a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub subcategories: Vec<String>,
}

impl CategoryConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            subcategories: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_subcategories(mut self, subs: Vec<impl Into<String>>) -> Self {
        self.subcategories = subs.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn to_spec(&self) -> Value {
        let mut spec = json!({});
        if !self.description.is_empty() {
            spec["description"] = json!(self.description);
        }
        if !self.subcategories.is_empty() {
            spec["subcategories"] = json!(self.subcategories);
        }
        spec
    }
}

// =============================================================================
// MCP Server Implementation
// =============================================================================

/// Parameter structs for tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EmptyParams {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GenericToolParams {
    /// Any arguments (ignored, just for testing)
    #[serde(flatten)]
    args: std::collections::HashMap<String, Value>,
}

/// The mock MCP server
pub struct MockMcpServer;

impl MockMcpServer {
    /// Build an McpRouter for the given config
    pub fn build_router(config: ServerConfig) -> Result<McpRouter, tower_mcp::BoxError> {
        let config = Arc::new(config);

        // -- get_discovery_spec
        let c = config.clone();
        let get_discovery_spec = ToolBuilder::new("get_discovery_spec")
            .description("Returns the discovery spec for registering this server's tools with jpx")
            .read_only()
            .handler(move |_params: EmptyParams| {
                let config = c.clone();
                async move {
                    let spec = config.to_discovery_spec();
                    let json = serde_json::to_string_pretty(&spec)
                        .map_err(|e| Error::tool(format!("Failed to serialize: {}", e)))?;
                    Ok(CallToolResult::text(json))
                }
            })
            .build()?;

        // -- echo
        let c = config.clone();
        let echo = ToolBuilder::new("echo")
            .description("Echo back the input - useful for testing MCP connectivity")
            .read_only()
            .handler(move |params: GenericToolParams| {
                let config = c.clone();
                async move {
                    let response = json!({
                        "server": config.name,
                        "echo": params.args,
                    });
                    let json = serde_json::to_string_pretty(&response)
                        .map_err(|e| Error::tool(format!("Failed to serialize: {}", e)))?;
                    Ok(CallToolResult::text(json))
                }
            })
            .build()?;

        // -- server_info
        let c = config.clone();
        let server_info = ToolBuilder::new("server_info")
            .description("Get information about this mock server")
            .read_only()
            .handler(move |_params: EmptyParams| {
                let config = c.clone();
                async move {
                    let response = json!({
                        "name": config.name,
                        "version": config.version,
                        "description": config.description,
                        "tool_count": config.tools.len(),
                        "categories": config.categories.iter().map(|c| &c.name).collect::<Vec<_>>(),
                    });
                    let json = serde_json::to_string_pretty(&response)
                        .map_err(|e| Error::tool(format!("Failed to serialize: {}", e)))?;
                    Ok(CallToolResult::text(json))
                }
            })
            .build()?;

        let router = McpRouter::new()
            .server_info(&config.name, &config.version)
            .instructions(format!(
                "Mock MCP server '{}' for testing jpx discovery protocol. \
                 Use get_discovery_spec to get the registration payload.",
                config.name
            ))
            .tool(get_discovery_spec)
            .tool(echo)
            .tool(server_info);

        Ok(router)
    }

    /// Run the mock server on stdio
    pub async fn run(config: ServerConfig) -> anyhow::Result<()> {
        use tracing::info;

        info!("Starting mock MCP server: {}", config.name);

        let router = Self::build_router(config).map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut transport = StdioTransport::new(router);

        info!("Mock MCP server running on stdio");

        transport
            .run()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }
}

// =============================================================================
// Preset Configurations for Common Test Scenarios
// =============================================================================

/// Generate preset server configurations for testing
pub mod presets {
    use super::*;

    /// A Redis-like management server with cluster/backup tools
    pub fn redis_server() -> ServerConfig {
        ServerConfig::new("mock-redis")
            .with_version("1.0.0")
            .with_description("Mock Redis Enterprise management server")
            .with_category(
                CategoryConfig::new("clusters")
                    .with_description("Cluster management operations")
                    .with_subcategories(vec!["lifecycle", "config"]),
            )
            .with_category(
                CategoryConfig::new("backups").with_description("Backup and restore operations"),
            )
            .with_tools(vec![
                MockToolConfig::full("create_cluster", "Create a new Redis cluster")
                    .with_category("clusters")
                    .with_subcategory("lifecycle")
                    .with_tags(vec!["write", "provisioning"])
                    .with_param(
                        ParamConfig::new("name")
                            .required()
                            .with_description("Cluster name"),
                    )
                    .with_param(ParamConfig::new("region").with_description("Cloud region")),
                MockToolConfig::full("delete_cluster", "Delete a Redis cluster permanently")
                    .with_category("clusters")
                    .with_subcategory("lifecycle")
                    .with_tags(vec!["write", "destructive"])
                    .with_related(vec!["create_cluster"]),
                MockToolConfig::full("list_clusters", "List all Redis clusters")
                    .with_category("clusters")
                    .with_tags(vec!["read"]),
                MockToolConfig::full("create_backup", "Create a backup of a cluster")
                    .with_category("backups")
                    .with_tags(vec!["write"])
                    .with_aliases(vec!["backup", "snapshot"]),
                MockToolConfig::full("restore_backup", "Restore a cluster from backup")
                    .with_category("backups")
                    .with_tags(vec!["write"]),
            ])
    }

    /// A database-like server (postgres style)
    pub fn database_server() -> ServerConfig {
        ServerConfig::new("mock-postgres")
            .with_version("2.0.0")
            .with_description("Mock PostgreSQL management server")
            .with_category(CategoryConfig::new("databases").with_description("Database operations"))
            .with_category(CategoryConfig::new("schema").with_description("Schema management"))
            .with_tools(vec![
                MockToolConfig::full("create_database", "Create a new PostgreSQL database")
                    .with_category("databases")
                    .with_tags(vec!["write"]),
                MockToolConfig::full("drop_database", "Drop a database")
                    .with_category("databases")
                    .with_tags(vec!["write", "destructive"]),
                MockToolConfig::full("list_tables", "List all tables in a database")
                    .with_category("schema")
                    .with_tags(vec!["read"]),
                MockToolConfig::full("create_backup", "Create a pg_dump backup")
                    .with_category("backups")
                    .with_tags(vec!["write"])
                    .with_aliases(vec!["pg_dump", "export"]),
            ])
    }

    /// A GitHub-like server
    pub fn github_server() -> ServerConfig {
        ServerConfig::new("mock-github")
            .with_version("1.5.0")
            .with_description("Mock GitHub API server")
            .with_tools(vec![
                MockToolConfig::full("create_issue", "Create a GitHub issue")
                    .with_category("issues")
                    .with_tags(vec!["write"]),
                MockToolConfig::full("list_issues", "List issues in a repository")
                    .with_category("issues")
                    .with_tags(vec!["read"]),
                MockToolConfig::full("create_pull_request", "Create a pull request")
                    .with_category("pulls")
                    .with_tags(vec!["write"])
                    .with_aliases(vec!["create_pr", "open_pr"]),
                MockToolConfig::full("list_pull_requests", "List pull requests")
                    .with_category("pulls")
                    .with_tags(vec!["read"]),
                MockToolConfig::full("merge_pull_request", "Merge a pull request")
                    .with_category("pulls")
                    .with_tags(vec!["write"]),
            ])
    }

    /// Generate N random servers with random tools for stress testing
    pub fn random_servers(count: usize, tools_per_server: usize) -> Vec<ServerConfig> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let categories = ["data", "config", "admin", "api", "storage", "network"];
        let tags = [
            "read",
            "write",
            "admin",
            "safe",
            "destructive",
            "fast",
            "slow",
        ];
        let verbs = [
            "create", "delete", "list", "get", "update", "sync", "export", "import",
        ];
        let nouns = [
            "resource", "entity", "record", "item", "object", "config", "setting",
        ];

        (0..count)
            .map(|i| {
                let tools: Vec<MockToolConfig> = (0..tools_per_server)
                    .map(|_| {
                        let verb = verbs[rng.gen_range(0..verbs.len())];
                        let noun = nouns[rng.gen_range(0..nouns.len())];
                        let cat = categories[rng.gen_range(0..categories.len())];
                        let tool_tags: Vec<&str> = (0..rng.gen_range(1..4))
                            .map(|_| tags[rng.gen_range(0..tags.len())])
                            .collect();

                        MockToolConfig::full(
                            format!("{}_{}", verb, noun),
                            format!("{} a {} in server {}", verb, noun, i),
                        )
                        .with_category(cat)
                        .with_tags(tool_tags)
                    })
                    .collect();

                ServerConfig::new(format!("mock-server-{}", i))
                    .with_version(format!(
                        "{}.{}.0",
                        rng.gen_range(1..5),
                        rng.gen_range(0..10)
                    ))
                    .with_description(format!("Auto-generated mock server {}", i))
                    .with_tools(tools)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tool_config() {
        let tool = MockToolConfig::simple("my_tool", "Does something");
        let spec = tool.to_spec();

        assert_eq!(spec["name"], "my_tool");
        assert_eq!(spec["description"], "Does something");
    }

    #[test]
    fn test_full_tool_config() {
        let tool = MockToolConfig::full("complex_tool", "A complex tool")
            .with_category("testing")
            .with_tags(vec!["read", "safe"])
            .with_param(ParamConfig::new("input").required());

        let spec = tool.to_spec();

        assert_eq!(spec["name"], "complex_tool");
        assert_eq!(spec["category"], "testing");
        assert_eq!(spec["tags"], json!(["read", "safe"]));
        assert!(spec["params"].is_array());
    }

    #[test]
    fn test_server_config_to_discovery_spec() {
        let config = ServerConfig::new("test-server")
            .with_version("1.0.0")
            .with_description("A test server")
            .with_tool(MockToolConfig::simple("tool_one", "First tool"))
            .with_tool(MockToolConfig::simple("tool_two", "Second tool"));

        let spec = config.to_discovery_spec();

        assert_eq!(spec["server"]["name"], "test-server");
        assert_eq!(spec["server"]["version"], "1.0.0");
        assert_eq!(spec["tools"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_redis_preset() {
        let config = presets::redis_server();
        assert_eq!(config.name, "mock-redis");
        assert_eq!(config.tools.len(), 5);

        let spec = config.to_discovery_spec();
        assert!(spec["categories"].get("clusters").is_some());
    }

    #[test]
    fn test_random_servers() {
        let servers = presets::random_servers(3, 5);
        assert_eq!(servers.len(), 3);

        for server in &servers {
            assert_eq!(server.tools.len(), 5);
        }
    }
}
