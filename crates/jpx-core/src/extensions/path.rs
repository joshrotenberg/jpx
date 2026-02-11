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
    register_if_enabled(
        runtime,
        "path_is_absolute",
        enabled,
        Box::new(PathIsAbsoluteFn::new()),
    );
    register_if_enabled(
        runtime,
        "path_is_relative",
        enabled,
        Box::new(PathIsRelativeFn::new()),
    );
    register_if_enabled(runtime, "path_join", enabled, Box::new(PathJoinFn::new()));
    register_if_enabled(runtime, "path_stem", enabled, Box::new(PathStemFn::new()));
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
// path_is_absolute(string) -> boolean
// =============================================================================

defn!(PathIsAbsoluteFn, vec![arg!(string)], None);

impl Function for PathIsAbsoluteFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        Ok(Value::Bool(std::path::Path::new(path).is_absolute()))
    }
}

// =============================================================================
// path_is_relative(string) -> boolean
// =============================================================================

defn!(PathIsRelativeFn, vec![arg!(string)], None);

impl Function for PathIsRelativeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        Ok(Value::Bool(std::path::Path::new(path).is_relative()))
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

// =============================================================================
// path_stem(string) -> string | null
// =============================================================================

defn!(PathStemFn, vec![arg!(string)], None);

impl Function for PathStemFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let path = args[0].as_str().unwrap();
        let stem = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str());
        match stem {
            Some(s) => Ok(Value::String(s.to_string())),
            None => Ok(Value::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_path_basename() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_basename(@)").unwrap();
        let data = json!("/path/to/file.txt");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "file.txt");
    }

    #[test]
    fn test_path_dirname() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_dirname(@)").unwrap();
        let data = json!("/path/to/file.txt");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "/path/to");
    }

    #[test]
    fn test_path_ext() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_ext(@)").unwrap();
        let data = json!("/path/to/file.txt");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), ".txt");
    }

    #[test]
    fn test_path_is_absolute() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_is_absolute(@)").unwrap();

        assert_eq!(expr.search(&json!("/foo/bar")).unwrap(), json!(true));
        assert_eq!(expr.search(&json!("foo/bar")).unwrap(), json!(false));
        assert_eq!(expr.search(&json!("file.txt")).unwrap(), json!(false));
    }

    #[test]
    fn test_path_is_relative() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_is_relative(@)").unwrap();

        assert_eq!(expr.search(&json!("foo/bar")).unwrap(), json!(true));
        assert_eq!(expr.search(&json!("file.txt")).unwrap(), json!(true));
        assert_eq!(expr.search(&json!("/foo/bar")).unwrap(), json!(false));
    }

    #[test]
    fn test_path_stem() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_stem(@)").unwrap();

        assert_eq!(expr.search(&json!("file.txt")).unwrap(), json!("file"));
        assert_eq!(
            expr.search(&json!("/foo/bar.tar.gz")).unwrap(),
            json!("bar.tar")
        );
        assert_eq!(expr.search(&json!("noext")).unwrap(), json!("noext"));
        assert_eq!(expr.search(&json!("/foo/bar/")).unwrap(), json!("bar"));
    }
}
