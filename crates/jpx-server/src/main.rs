//! JMESPath MCP Server
//!
//! This is the entry point for the jpx-server binary, which provides
//! JMESPath functionality over the MCP (Model Context Protocol).

#[cfg(feature = "mcp")]
mod mcp;

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mcp::run().await
}

#[cfg(not(feature = "mcp"))]
fn main() {
    eprintln!("Error: No transport feature enabled. Enable 'mcp' feature.");
    std::process::exit(1);
}
