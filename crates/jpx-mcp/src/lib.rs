//! JMESPath MCP Server Library
//!
//! This library provides the MCP router and tools for JMESPath functionality.
//!
//! [`build_router`] preserves unrestricted `evaluate_file` access for backward
//! compatibility and is intended for trusted local/stdio clients. Remote
//! embedders should use [`build_router_with_file_access`] (or its config-aware
//! counterpart) with [`FileAccessPolicy::disabled`] or
//! [`FileAccessPolicy::restricted`].

mod filesystem;
mod tools;

pub use filesystem::{FileAccessMode, FileAccessPolicy, FileAccessPolicyError};
pub use tools::{
    build_router, build_router_from_config, build_router_from_config_with_file_access,
    build_router_with_file_access,
};
