//! jpx library - exposed for testing and potential reuse
//!
//! This module exposes internal components for integration testing.

pub mod query_library;

#[cfg(feature = "parquet")]
pub mod parquet_support;
