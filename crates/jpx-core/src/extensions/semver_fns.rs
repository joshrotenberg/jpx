//! Semantic versioning functions.

use std::collections::HashSet;

use semver_crate::{Version, VersionReq};
use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// semver_parse(s) -> object
// =============================================================================

defn!(SemverParseFn, vec![arg!(string)], None);

impl Function for SemverParseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        match Version::parse(s) {
            Ok(v) => {
                let pre = if v.pre.is_empty() {
                    Value::Null
                } else {
                    Value::String(v.pre.to_string())
                };
                let build = if v.build.is_empty() {
                    Value::Null
                } else {
                    Value::String(v.build.to_string())
                };

                let obj = serde_json::json!({
                    "major": v.major,
                    "minor": v.minor,
                    "patch": v.patch,
                    "pre": pre,
                    "build": build
                });

                Ok(obj)
            }
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// semver_major(s) -> number
// =============================================================================

defn!(SemverMajorFn, vec![arg!(string)], None);

impl Function for SemverMajorFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Value::Number(Number::from(v.major))),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// semver_minor(s) -> number
// =============================================================================

defn!(SemverMinorFn, vec![arg!(string)], None);

impl Function for SemverMinorFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Value::Number(Number::from(v.minor))),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// semver_patch(s) -> number
// =============================================================================

defn!(SemverPatchFn, vec![arg!(string)], None);

impl Function for SemverPatchFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Value::Number(Number::from(v.patch))),
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// semver_compare(v1, v2) -> number (-1, 0, 1)
// =============================================================================

defn!(SemverCompareFn, vec![arg!(string), arg!(string)], None);

impl Function for SemverCompareFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_str().unwrap();
        let s2 = args[1].as_str().unwrap();

        let v1 = match Version::parse(s1) {
            Ok(v) => v,
            Err(_) => return Ok(Value::Null),
        };
        let v2 = match Version::parse(s2) {
            Ok(v) => v,
            Err(_) => return Ok(Value::Null),
        };

        let result = match v1.cmp(&v2) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };

        Ok(number_value(result as f64))
    }
}

// =============================================================================
// semver_satisfies(version, requirement) -> bool
// =============================================================================

defn!(SemverSatisfiesFn, vec![arg!(string), arg!(string)], None);

impl Function for SemverSatisfiesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let version_str = args[0].as_str().unwrap();
        let req_str = args[1].as_str().unwrap();

        let version = match Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => return Ok(Value::Null),
        };
        let req = match VersionReq::parse(req_str) {
            Ok(r) => r,
            Err(_) => return Ok(Value::Null),
        };

        Ok(Value::Bool(req.matches(&version)))
    }
}

// =============================================================================
// semver_is_valid(s) -> bool
// =============================================================================

defn!(SemverIsValidFn, vec![arg!(string)], None);

impl Function for SemverIsValidFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_str().unwrap();
        let is_valid = Version::parse(s).is_ok();
        Ok(Value::Bool(is_valid))
    }
}

/// Register semver functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "semver_parse",
        enabled,
        Box::new(SemverParseFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_major",
        enabled,
        Box::new(SemverMajorFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_minor",
        enabled,
        Box::new(SemverMinorFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_patch",
        enabled,
        Box::new(SemverPatchFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_compare",
        enabled,
        Box::new(SemverCompareFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_satisfies",
        enabled,
        Box::new(SemverSatisfiesFn::new()),
    );
    register_if_enabled(
        runtime,
        "semver_is_valid",
        enabled,
        Box::new(SemverIsValidFn::new()),
    );
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
    fn test_semver_parse() {
        let runtime = setup_runtime();
        let data = json!("1.2.3");
        let expr = runtime.compile("semver_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("major").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(obj.get("minor").unwrap().as_f64().unwrap(), 2.0);
        assert_eq!(obj.get("patch").unwrap().as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_semver_parse_with_pre() {
        let runtime = setup_runtime();
        let data = json!("1.0.0-alpha.1");
        let expr = runtime.compile("semver_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("major").unwrap().as_f64().unwrap(), 1.0);
        assert_eq!(obj.get("pre").unwrap().as_str().unwrap(), "alpha.1");
    }

    #[test]
    fn test_semver_major() {
        let runtime = setup_runtime();
        let data = json!("2.3.4");
        let expr = runtime.compile("semver_major(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_semver_minor() {
        let runtime = setup_runtime();
        let data = json!("2.3.4");
        let expr = runtime.compile("semver_minor(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_semver_patch_fn() {
        let runtime = setup_runtime();
        let data = json!("2.3.4");
        let expr = runtime.compile("semver_patch(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_semver_compare_less() {
        let runtime = setup_runtime();
        let data = json!(["1.0.0", "2.0.0"]);
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), -1.0);
    }

    #[test]
    fn test_semver_compare_equal() {
        let runtime = setup_runtime();
        let data = json!(["1.0.0", "1.0.0"]);
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_semver_compare_greater() {
        let runtime = setup_runtime();
        let data = json!(["2.0.0", "1.0.0"]);
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_semver_satisfies_true() {
        let runtime = setup_runtime();
        let data = json!(["1.2.3", "^1.0.0"]);
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_semver_satisfies_false() {
        let runtime = setup_runtime();
        let data = json!(["2.0.0", "^1.0.0"]);
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_semver_satisfies_tilde() {
        let runtime = setup_runtime();
        let data = json!(["1.2.5", "~1.2.0"]);
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_semver_is_valid_true() {
        let runtime = setup_runtime();
        let data = json!("1.2.3");
        let expr = runtime.compile("semver_is_valid(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_semver_is_valid_false() {
        let runtime = setup_runtime();
        let data = json!("not-a-version");
        let expr = runtime.compile("semver_is_valid(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }
}
