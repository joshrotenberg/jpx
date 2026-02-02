//! MCP transport implementation for jpx-server.
//!
//! This module provides the MCP (Model Context Protocol) server implementation
//! that exposes jpx_engine functionality over stdio.

mod tools;

use anyhow::Result;
use tower_mcp::StdioTransport;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Run the MCP server.
pub async fn run() -> Result<()> {
    // Initialize tracing to stderr (stdout is used for MCP protocol)
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env().add_directive("jpx_server=info".parse()?))
        .init();

    // Check for strict mode flag
    let strict = std::env::args().any(|arg| arg == "--strict");

    info!(
        "Starting jpx MCP server{}",
        if strict { " (strict mode)" } else { "" }
    );

    // Create MCP router and run
    let router = tools::build_router(strict).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut transport = StdioTransport::new(router);

    info!("jpx MCP server running on stdio");

    transport.run().await?;

    Ok(())
}
