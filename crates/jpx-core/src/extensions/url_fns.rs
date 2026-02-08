//! URL parsing and manipulation functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register URL functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "url_encode", enabled, Box::new(UrlEncodeFn::new()));
    register_if_enabled(runtime, "url_decode", enabled, Box::new(UrlDecodeFn::new()));
    register_if_enabled(runtime, "url_parse", enabled, Box::new(UrlParseFn::new()));
}

// =============================================================================
// url_encode(string) -> string
// =============================================================================

defn!(UrlEncodeFn, vec![arg!(string)], None);

impl Function for UrlEncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = urlencoding::encode(input);
        Ok(Value::String(encoded.into_owned()))
    }
}

// =============================================================================
// url_decode(string) -> string
// =============================================================================

defn!(UrlDecodeFn, vec![arg!(string)], None);

impl Function for UrlDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match urlencoding::decode(input) {
            Ok(decoded) => Ok(Value::String(decoded.into_owned())),
            Err(_) => Err(crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Invalid URL-encoded input".to_owned()),
            )),
        }
    }
}

// =============================================================================
// url_parse(string) -> object (parse URL into components)
// =============================================================================

defn!(UrlParseFn, vec![arg!(string)], None);

impl Function for UrlParseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match url::Url::parse(input) {
            Ok(parsed) => {
                let mut result = serde_json::Map::new();

                result.insert(
                    "scheme".to_string(),
                    Value::String(parsed.scheme().to_string()),
                );

                if let Some(host) = parsed.host_str() {
                    result.insert("host".to_string(), Value::String(host.to_string()));
                } else {
                    result.insert("host".to_string(), Value::Null);
                }

                if let Some(port) = parsed.port() {
                    result.insert(
                        "port".to_string(),
                        Value::Number(serde_json::Number::from(port)),
                    );
                } else {
                    result.insert("port".to_string(), Value::Null);
                }

                result.insert("path".to_string(), Value::String(parsed.path().to_string()));

                if let Some(query) = parsed.query() {
                    result.insert("query".to_string(), Value::String(query.to_string()));
                } else {
                    result.insert("query".to_string(), Value::Null);
                }

                if let Some(fragment) = parsed.fragment() {
                    result.insert("fragment".to_string(), Value::String(fragment.to_string()));
                } else {
                    result.insert("fragment".to_string(), Value::Null);
                }

                if !parsed.username().is_empty() {
                    result.insert(
                        "username".to_string(),
                        Value::String(parsed.username().to_string()),
                    );
                }

                if let Some(password) = parsed.password() {
                    result.insert("password".to_string(), Value::String(password.to_string()));
                }

                // Add origin field (scheme + host + port)
                let origin = parsed.origin().ascii_serialization();
                result.insert("origin".to_string(), Value::String(origin));

                Ok(Value::Object(result))
            }
            // Return null for invalid URLs instead of an error
            Err(_) => Ok(Value::Null),
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
    fn test_url_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_encode(@)").unwrap();
        let data = json!("hello world");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello%20world");
    }

    #[test]
    fn test_url_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_decode(@)").unwrap();
        let data = json!("hello%20world");
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_url_parse() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = json!("https://example.com:8080/path?query=1#frag");
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("scheme").unwrap().as_str().unwrap(), "https");
        assert_eq!(obj.get("host").unwrap().as_str().unwrap(), "example.com");
        assert_eq!(obj.get("port").unwrap().as_f64().unwrap() as u16, 8080);
    }

    #[test]
    fn test_url_parse_origin() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = json!("https://example.com:8080/path");
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("origin").unwrap().as_str().unwrap(),
            "https://example.com:8080"
        );
    }

    #[test]
    fn test_url_parse_invalid_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = json!("not a valid url");
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }
}
