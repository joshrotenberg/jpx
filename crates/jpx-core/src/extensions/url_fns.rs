//! URL parsing and manipulation functions.

use std::collections::HashSet;

use form_urlencoded;
use serde_json::Value;

use crate::functions::{Function, custom_error};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register URL functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "url_encode", enabled, Box::new(UrlEncodeFn::new()));
    register_if_enabled(runtime, "url_decode", enabled, Box::new(UrlDecodeFn::new()));
    register_if_enabled(runtime, "url_parse", enabled, Box::new(UrlParseFn::new()));
    register_if_enabled(runtime, "url_build", enabled, Box::new(UrlBuildFn::new()));
    register_if_enabled(
        runtime,
        "query_string_parse",
        enabled,
        Box::new(QueryStringParseFn::new()),
    );
    register_if_enabled(
        runtime,
        "query_string_build",
        enabled,
        Box::new(QueryStringBuildFn::new()),
    );
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

// =============================================================================
// url_build(object) -> string (build URL from components)
// =============================================================================

defn!(UrlBuildFn, vec![arg!(object)], None);

impl Function for UrlBuildFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let scheme = obj
            .get("scheme")
            .and_then(|v| v.as_str())
            .ok_or_else(|| custom_error(ctx, "url_build: 'scheme' is required"))?;

        let host = obj
            .get("host")
            .and_then(|v| v.as_str())
            .ok_or_else(|| custom_error(ctx, "url_build: 'host' is required"))?;

        let base = format!("{scheme}://{host}");
        let mut url = url::Url::parse(&base)
            .map_err(|e| custom_error(ctx, &format!("url_build: invalid scheme/host: {e}")))?;

        if let Some(port) = obj.get("port")
            && let Some(p) = port.as_u64()
        {
            url.set_port(Some(p as u16))
                .map_err(|()| custom_error(ctx, "url_build: cannot set port on this URL"))?;
        }

        if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
            url.set_path(path);
        }

        if let Some(query) = obj.get("query").and_then(|v| v.as_str()) {
            url.set_query(Some(query));
        }

        if let Some(fragment) = obj.get("fragment").and_then(|v| v.as_str()) {
            url.set_fragment(Some(fragment));
        }

        if let Some(username) = obj.get("username").and_then(|v| v.as_str()) {
            url.set_username(username)
                .map_err(|()| custom_error(ctx, "url_build: cannot set username on this URL"))?;
        }

        if let Some(password) = obj.get("password").and_then(|v| v.as_str()) {
            url.set_password(Some(password))
                .map_err(|()| custom_error(ctx, "url_build: cannot set password on this URL"))?;
        }

        Ok(Value::String(url.to_string()))
    }
}

// =============================================================================
// query_string_parse(string) -> object
// =============================================================================

defn!(QueryStringParseFn, vec![arg!(string)], None);

impl Function for QueryStringParseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let mut map = serde_json::Map::new();
        for (key, value) in form_urlencoded::parse(input.as_bytes()) {
            map.insert(key.into_owned(), Value::String(value.into_owned()));
        }

        Ok(Value::Object(map))
    }
}

// =============================================================================
// query_string_build(object) -> string
// =============================================================================

defn!(QueryStringBuildFn, vec![arg!(object)], None);

impl Function for QueryStringBuildFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let obj = args[0]
            .as_object()
            .ok_or_else(|| custom_error(ctx, "Expected object argument"))?;

        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in obj {
            let val_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            serializer.append_pair(key, &val_str);
        }

        Ok(Value::String(serializer.finish()))
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

    // url_build tests

    #[test]
    fn test_url_build_minimal() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_build(@)").unwrap();
        let data = json!({"scheme": "https", "host": "example.com"});
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "https://example.com/");
    }

    #[test]
    fn test_url_build_full() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_build(@)").unwrap();
        let data = json!({
            "scheme": "https",
            "host": "example.com",
            "port": 8080,
            "path": "/api/v1",
            "query": "key=value",
            "fragment": "section",
            "username": "user",
            "password": "pass"
        });
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            "https://user:pass@example.com:8080/api/v1?key=value#section"
        );
    }

    #[test]
    fn test_url_build_roundtrip() {
        let runtime = setup_runtime();
        let original = "https://example.com:8080/path?q=1#frag";
        let parse_expr = runtime.compile("url_parse(@)").unwrap();
        let parsed = parse_expr.search(&json!(original)).unwrap();

        let build_expr = runtime.compile("url_build(@)").unwrap();
        let rebuilt = build_expr.search(&parsed).unwrap();
        assert_eq!(rebuilt.as_str().unwrap(), original);
    }

    // query_string_parse tests

    #[test]
    fn test_query_string_parse_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_parse(@)").unwrap();
        let data = json!("foo=bar&baz=qux");
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("foo").unwrap().as_str().unwrap(), "bar");
        assert_eq!(obj.get("baz").unwrap().as_str().unwrap(), "qux");
    }

    #[test]
    fn test_query_string_parse_encoded() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_parse(@)").unwrap();
        let data = json!("greeting=hello%20world&special=a%2Bb");
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("greeting").unwrap().as_str().unwrap(),
            "hello world"
        );
        assert_eq!(obj.get("special").unwrap().as_str().unwrap(), "a+b");
    }

    #[test]
    fn test_query_string_parse_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_parse(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.is_empty());
    }

    // query_string_build tests

    #[test]
    fn test_query_string_build_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_build(@)").unwrap();
        let data = json!({"foo": "bar", "baz": "qux"});
        let result = expr.search(&data).unwrap();
        let qs = result.as_str().unwrap();
        // Object key order is deterministic in serde_json
        assert!(qs.contains("foo=bar"));
        assert!(qs.contains("baz=qux"));
    }

    #[test]
    fn test_query_string_build_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_build(@)").unwrap();
        let data = json!({"greeting": "hello world", "op": "a+b"});
        let result = expr.search(&data).unwrap();
        let qs = result.as_str().unwrap();
        assert!(qs.contains("greeting=hello+world"));
        assert!(qs.contains("op=a%2Bb"));
    }

    #[test]
    fn test_query_string_build_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("query_string_build(@)").unwrap();
        let data = json!({});
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "");
    }
}
