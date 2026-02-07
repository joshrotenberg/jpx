//! File path manipulation functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register path functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "path_basename",
        enabled,
        Box::new(PathBasenameFn::new()),
    );
    register_if_enabled(
        runtime,
        "path_dirname",
        enabled,
        Box::new(PathDirnameFn::new()),
    );
    register_if_enabled(runtime, "path_ext", enabled, Box::new(PathExtFn::new()));
    register_if_enabled(runtime, "path_join", enabled, Box::new(PathJoinFn::new()));
}

// =============================================================================
// path_basename(string) -> string
// =============================================================================

defn!(PathBasenameFn, vec![arg!(string)], None);

impl Function for PathBasenameFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        let basename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        Ok(Value::String(basename.to_string()))
    }
}

// =============================================================================
// path_dirname(string) -> string
// =============================================================================

defn!(PathDirnameFn, vec![arg!(string)], None);

impl Function for PathDirnameFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        let dirname = std::path::Path::new(path)
            .parent()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        Ok(Value::String(dirname.to_string()))
    }
}

// =============================================================================
// path_ext(string) -> string
// =============================================================================

defn!(PathExtFn, vec![arg!(string)], None);

impl Function for PathExtFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_default();
        Ok(Value::String(ext))
    }
}

// =============================================================================
// path_join(array) -> string
// =============================================================================

defn!(PathJoinFn, vec![arg!(array)], None);

impl Function for PathJoinFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let arr = args[0].as_array().unwrap();
        let mut path = std::path::PathBuf::new();
        for part in arr {
            if let Some(s) = part.as_str() {
                path.push(s);
            }
        }
        let result = path.to_str().unwrap_or("").to_string();
        Ok(Value::String(result))
    }
}
