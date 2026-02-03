//! JMESPath MCP Server Library
//!
//! This library provides the MCP router and tools for JMESPath functionality.
//! Use `build_router()` to create a configured router for use with any transport.

mod tools;

pub use tools::build_router;
