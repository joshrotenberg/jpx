//! JMESPath MCP Server
//!
//! An MCP server providing JMESPath functionality with 490+ functions.

use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use jpx_mcp::{FileAccessPolicy, FileAccessPolicyError, build_router_from_config_with_file_access};
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
#[command(about = "JMESPath MCP server with 490+ functions", long_about = None)]
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

    /// Allow evaluate_file beneath this directory (repeatable). Without this option,
    /// stdio retains unrestricted file access and HTTP disables file access.
    #[arg(long, value_name = "DIRECTORY")]
    allow_root: Vec<PathBuf>,
}

fn select_file_access_policy(
    transport: Transport,
    allowed_roots: &[PathBuf],
) -> Result<FileAccessPolicy, FileAccessPolicyError> {
    if !allowed_roots.is_empty() {
        return FileAccessPolicy::restricted(allowed_roots);
    }

    Ok(match transport {
        Transport::Stdio => FileAccessPolicy::unrestricted(),
        Transport::Http => FileAccessPolicy::disabled(),
    })
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
        engine_config.engine.strict = Some(true);
    }

    let file_access = select_file_access_policy(args.transport, &args.allow_root)?;
    tracing::info!(
        mode = file_access.mode().as_str(),
        allowed_roots = ?file_access.allowed_roots(),
        "Configured evaluate_file policy"
    );

    // Build the router
    let router = build_router_from_config_with_file_access(engine_config, file_access)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    match args.transport {
        Transport::Stdio => {
            tracing::info!("Serving over stdio");
            StdioTransport::new(router).run().await?;
        }
        Transport::Http => {
            let addr = format!("{}:{}", args.host, args.port);
            tracing::info!(%addr, "Serving over HTTP");

            let transport = HttpTransport::new(router).layer(
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

#[cfg(test)]
mod tests {
    use super::*;
    use jpx_mcp::FileAccessMode;
    use std::path::Path;

    #[test]
    fn stdio_defaults_to_unrestricted_file_access() {
        let policy = select_file_access_policy(Transport::Stdio, &[]).unwrap();
        assert_eq!(policy.mode(), FileAccessMode::Unrestricted);
    }

    #[test]
    fn http_defaults_to_disabled_file_access() {
        let policy = select_file_access_policy(Transport::Http, &[]).unwrap();
        assert_eq!(policy.mode(), FileAccessMode::Disabled);
    }

    #[test]
    fn explicit_roots_restrict_both_transports() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let roots = vec![root.to_path_buf()];

        for transport in [Transport::Stdio, Transport::Http] {
            let policy = select_file_access_policy(transport, &roots).unwrap();
            assert_eq!(policy.mode(), FileAccessMode::AllowedRoots);
            assert_eq!(policy.allowed_roots(), &[root.canonicalize().unwrap()]);
        }
    }

    #[test]
    fn allow_root_option_is_repeatable() {
        let args = Args::try_parse_from([
            "jpx-mcp",
            "--allow-root",
            "/srv/data",
            "--allow-root",
            "/srv/reports",
        ])
        .unwrap();

        assert_eq!(
            args.allow_root,
            [PathBuf::from("/srv/data"), PathBuf::from("/srv/reports")]
        );
    }
}
