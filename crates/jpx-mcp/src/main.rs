//! JMESPath MCP Server
//!
//! An MCP server providing JMESPath functionality with 400+ extended functions.

mod tools;

use std::time::Duration;

use clap::{Parser, ValueEnum};
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;
use tower_mcp::{HttpTransport, McpTracingLayer, StdioTransport};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Transport {
    Stdio,
    Http,
}

#[derive(Parser, Debug)]
#[command(name = "jpx-mcp")]
#[command(about = "JMESPath MCP server with 400+ extended functions", long_about = None)]
#[command(version)]
struct Args {
    /// Transport to use
    #[arg(short, long, default_value = "stdio")]
    transport: Transport,

    /// Strict mode - only standard JMESPath functions (no extensions)
    #[arg(long, default_value = "false")]
    strict: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// HTTP host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// HTTP port to bind to
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Request timeout in seconds (for HTTP transport)
    #[arg(long, default_value = "30")]
    request_timeout_secs: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing to stderr (stdout is used for MCP protocol in stdio mode)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::from_default_env()
                .add_directive(format!("jpx_mcp={}", args.log_level).parse()?)
                .add_directive(format!("tower_mcp={}", args.log_level).parse()?),
        )
        .init();

    tracing::info!(
        transport = ?args.transport,
        strict = args.strict,
        "Starting jpx-mcp server"
    );

    // Load engine config (jpx.toml discovery) and merge CLI flags
    let mut engine_config = jpx_engine::config::EngineConfig::discover().unwrap_or_default();
    if args.strict {
        engine_config.engine.strict = true;
    }

    // Build the router
    let router =
        tools::build_router_from_config(engine_config).map_err(|e| anyhow::anyhow!("{}", e))?;

    match args.transport {
        Transport::Stdio => {
            tracing::info!("Serving over stdio");
            StdioTransport::new(router).run().await?;
        }
        Transport::Http => {
            let addr = format!("{}:{}", args.host, args.port);
            tracing::info!(%addr, "Serving over HTTP");

            let transport = HttpTransport::new(router)
                .disable_origin_validation()
                .layer(
                    ServiceBuilder::new()
                        .layer(TimeoutLayer::new(Duration::from_secs(
                            args.request_timeout_secs,
                        )))
                        .layer(McpTracingLayer::new())
                        .into_inner(),
                );

            transport.serve(&addr).await?;
        }
    }

    Ok(())
}
