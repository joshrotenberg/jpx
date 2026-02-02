//! Mock MCP Server CLI
//!
//! Run a mock MCP server for testing jpx discovery protocol.
//!
//! # Usage
//!
//! ```bash
//! # Run with a preset configuration
//! mock-mcp-server --preset redis
//!
//! # Run with a custom name
//! mock-mcp-server --name my-server
//!
//! # Run with a JSON config file
//! mock-mcp-server --config server.json
//!
//! # Run with inline JSON
//! mock-mcp-server --json '{"name": "test", "tools": [{"name": "foo", "description": "bar"}]}'
//! ```

use clap::{Parser, ValueEnum};
use mock_mcp_server::{MockMcpServer, MockToolConfig, ParamConfig, ServerConfig, presets};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Debug, Clone, ValueEnum)]
enum Preset {
    /// Redis-like management server
    Redis,
    /// PostgreSQL-like database server
    Postgres,
    /// GitHub-like API server
    Github,
    /// Minimal server with just echo/info tools
    Minimal,
}

#[derive(Parser, Debug)]
#[command(name = "mock-mcp-server")]
#[command(about = "Mock MCP server for testing jpx discovery protocol")]
#[command(version)]
struct Args {
    /// Use a preset server configuration
    #[arg(short, long)]
    preset: Option<Preset>,

    /// Server name (for custom configuration)
    #[arg(short, long)]
    name: Option<String>,

    /// Number of random tools to generate (for stress testing)
    #[arg(long)]
    random_tools: Option<usize>,

    /// Load configuration from a JSON file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// Inline JSON configuration
    #[arg(long)]
    json: Option<String>,

    /// Server version
    #[arg(long, default_value = "1.0.0")]
    version: String,

    /// Server description
    #[arg(long, default_value = "")]
    description: String,
}

fn build_config(args: &Args) -> anyhow::Result<ServerConfig> {
    // Priority: config file > json > preset > name > default

    if let Some(path) = &args.config {
        let content = std::fs::read_to_string(path)?;
        return Ok(serde_json::from_str(&content)?);
    }

    if let Some(json) = &args.json {
        return Ok(serde_json::from_str(json)?);
    }

    if let Some(preset) = &args.preset {
        return Ok(match preset {
            Preset::Redis => presets::redis_server(),
            Preset::Postgres => presets::database_server(),
            Preset::Github => presets::github_server(),
            Preset::Minimal => ServerConfig::new("mock-minimal")
                .with_version("1.0.0")
                .with_description("Minimal mock server"),
        });
    }

    // Build custom config from args
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| "mock-server".to_string());
    let mut config = ServerConfig::new(&name)
        .with_version(&args.version)
        .with_description(&args.description);

    // Add random tools if requested
    if let Some(count) = args.random_tools {
        let tools = generate_random_tools(count, &name);
        config = config.with_tools(tools);
    }

    Ok(config)
}

fn generate_random_tools(count: usize, server_name: &str) -> Vec<MockToolConfig> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let categories = ["data", "config", "admin", "api", "storage"];
    let tags = ["read", "write", "admin", "safe", "destructive"];
    let verbs = ["create", "delete", "list", "get", "update", "sync"];
    let nouns = ["resource", "entity", "record", "item", "object"];

    (0..count)
        .map(|_i| {
            let verb = verbs[rng.gen_range(0..verbs.len())];
            let noun = nouns[rng.gen_range(0..nouns.len())];
            let cat = categories[rng.gen_range(0..categories.len())];
            let tool_tags: Vec<&str> = (0..rng.gen_range(1..3))
                .map(|_| tags[rng.gen_range(0..tags.len())])
                .collect();

            MockToolConfig::full(
                format!("{}_{}", verb, noun),
                format!("{} a {} in {}", verb, noun, server_name),
            )
            .with_category(cat)
            .with_tags(tool_tags)
            .with_param(
                ParamConfig::new("id")
                    .with_type("string")
                    .with_description(format!("The {} ID", noun)),
            )
        })
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing to stderr
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive("mock_mcp_server=info".parse()?))
        .init();

    let args = Args::parse();
    let config = build_config(&args)?;

    // Print config to stderr for debugging
    eprintln!("Starting mock server: {}", config.name);
    eprintln!("Tools: {}", config.tools.len());

    MockMcpServer::run(config).await
}
