//! Encoding and decoding functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

use base64::{
    Engine,
    engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD as BASE64_URL_SAFE},
};

/// Register encoding functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "base64_encode",
        enabled,
        Box::new(Base64EncodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "base64_decode",
        enabled,
        Box::new(Base64DecodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "base64url_decode",
        enabled,
        Box::new(Base64UrlDecodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "base64url_encode",
        enabled,
        Box::new(Base64UrlEncodeFn::new()),
    );
    register_if_enabled(runtime, "hex_encode", enabled, Box::new(HexEncodeFn::new()));
    register_if_enabled(runtime, "hex_decode", enabled, Box::new(HexDecodeFn::new()));
    register_if_enabled(runtime, "jwt_decode", enabled, Box::new(JwtDecodeFn::new()));
    register_if_enabled(runtime, "jwt_header", enabled, Box::new(JwtHeaderFn::new()));
    register_if_enabled(
        runtime,
        "html_escape",
        enabled,
        Box::new(HtmlEscapeFn::new()),
    );
    register_if_enabled(
        runtime,
        "html_unescape",
        enabled,
        Box::new(HtmlUnescapeFn::new()),
    );
    register_if_enabled(
        runtime,
        "shell_escape",
        enabled,
        Box::new(ShellEscapeFn::new()),
    );
}

// =============================================================================
// base64_encode(string) -> string
// =============================================================================

defn!(Base64EncodeFn, vec![arg!(string)], None);

impl Function for Base64EncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = BASE64_STANDARD.encode(input.as_bytes());
        Ok(Value::String(encoded))
    }
}

// =============================================================================
// base64_decode(string) -> string
// =============================================================================

defn!(Base64DecodeFn, vec![arg!(string)], None);

impl Function for Base64DecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match BASE64_STANDARD.decode(input.as_bytes()) {
            Ok(decoded) => {
                let s = String::from_utf8(decoded).map_err(|_| {
                    crate::JmespathError::from_ctx(
                        ctx,
                        crate::ErrorReason::Parse("Decoded bytes are not valid UTF-8".to_owned()),
                    )
                })?;
                Ok(Value::String(s))
            }
            Err(_) => Err(crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Invalid base64 input".to_owned()),
            )),
        }
    }
}

// =============================================================================
// base64url_encode(string) -> string
// =============================================================================

defn!(Base64UrlEncodeFn, vec![arg!(string)], None);

impl Function for Base64UrlEncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = BASE64_URL_SAFE.encode(input.as_bytes());
        Ok(Value::String(encoded))
    }
}

// =============================================================================
// base64url_decode(string) -> string
// =============================================================================

defn!(Base64UrlDecodeFn, vec![arg!(string)], None);

impl Function for Base64UrlDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match BASE64_URL_SAFE.decode(input.as_bytes()) {
            Ok(decoded) => {
                let s = String::from_utf8(decoded).map_err(|_| {
                    crate::JmespathError::from_ctx(
                        ctx,
                        crate::ErrorReason::Parse("Decoded bytes are not valid UTF-8".to_owned()),
                    )
                })?;
                Ok(Value::String(s))
            }
            Err(_) => Err(crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Invalid base64url input".to_owned()),
            )),
        }
    }
}

// =============================================================================
// hex_encode(string) -> string
// =============================================================================

defn!(HexEncodeFn, vec![arg!(string)], None);

impl Function for HexEncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = hex::encode(input.as_bytes());
        Ok(Value::String(encoded))
    }
}

// =============================================================================
// hex_decode(string) -> string
// =============================================================================

defn!(HexDecodeFn, vec![arg!(string)], None);

impl Function for HexDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match hex::decode(input) {
            Ok(decoded) => {
                // Return null if decoded bytes are not valid UTF-8
                match String::from_utf8(decoded) {
                    Ok(s) => Ok(Value::String(s)),
                    Err(_) => Ok(Value::Null),
                }
            }
            // Return null for invalid hex input
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// JWT Helper Functions
// =============================================================================

/// Decode a base64url-encoded JWT part (header or payload) to JSON
fn decode_jwt_part(part: &str) -> Option<serde_json::Value> {
    // JWT uses base64url encoding (no padding)
    let decoded = BASE64_URL_SAFE.decode(part).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    serde_json::from_str(&json_str).ok()
}

// =============================================================================
// jwt_decode(token) -> object (JWT payload/claims)
// =============================================================================

defn!(JwtDecodeFn, vec![arg!(string)], None);

impl Function for JwtDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let token = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(Value::Null);
        }

        // Decode the payload (second part)
        match decode_jwt_part(parts[1]) {
            Some(json) => Ok(json),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// jwt_header(token) -> object (JWT header)
// =============================================================================

defn!(JwtHeaderFn, vec![arg!(string)], None);

impl Function for JwtHeaderFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let token = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(Value::Null);
        }

        // Decode the header (first part)
        match decode_jwt_part(parts[0]) {
            Some(json) => Ok(json),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// html_escape(string) -> string
// =============================================================================

defn!(HtmlEscapeFn, vec![arg!(string)], None);

impl Function for HtmlEscapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let escaped = s
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        Ok(Value::String(escaped))
    }
}

// =============================================================================
// html_unescape(string) -> string
// =============================================================================

defn!(HtmlUnescapeFn, vec![arg!(string)], None);

impl Function for HtmlUnescapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Order matters: decode &amp; last to avoid double-decoding
        let unescaped = s
            .replace("&#x27;", "'")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
            .replace("&quot;", "\"")
            .replace("&gt;", ">")
            .replace("&lt;", "<")
            .replace("&amp;", "&");

        Ok(Value::String(unescaped))
    }
}

// =============================================================================
// shell_escape(string) -> string
// =============================================================================

defn!(ShellEscapeFn, vec![arg!(string)], None);

impl Function for ShellEscapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Shell escaping: wrap in single quotes and escape internal single quotes
        // The pattern is: replace ' with '\'' (end quote, escaped quote, start quote)
        let escaped = format!("'{}'", s.replace('\'', "'\\''"));

        Ok(Value::String(escaped))
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
    fn test_base64_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64_encode(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("aGVsbG8="));
    }

    #[test]
    fn test_base64_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64_decode(@)").unwrap();
        let data = json!("aGVsbG8=");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn test_hex_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_encode(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("68656c6c6f"));
    }

    #[test]
    fn test_hex_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = json!("68656c6c6f");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn test_hex_decode_invalid_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = json!("invalid");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_hex_decode_odd_length_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = json!("123");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));
    }

    // =========================================================================
    // JWT function tests
    // =========================================================================

    // Test JWT from jwt.io: {"sub": "1234567890", "name": "John Doe", "iat": 1516239022}
    const TEST_JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    #[test]
    fn test_jwt_decode_payload() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();
        let data = json!(TEST_JWT);
        let result = expr.search(&data).unwrap();

        // Check it's an object with expected claims
        assert_eq!(result["sub"], json!("1234567890"));
        assert_eq!(result["name"], json!("John Doe"));
        assert_eq!(result["iat"], json!(1516239022));
    }

    #[test]
    fn test_jwt_decode_extract_claim() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@).sub").unwrap();
        let data = json!(TEST_JWT);
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("1234567890"));
    }

    #[test]
    fn test_jwt_header() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_header(@)").unwrap();
        let data = json!(TEST_JWT);
        let result = expr.search(&data).unwrap();

        // Check header fields
        assert_eq!(result["alg"], json!("HS256"));
        assert_eq!(result["typ"], json!("JWT"));
    }

    #[test]
    fn test_jwt_header_extract_alg() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_header(@).alg").unwrap();
        let data = json!(TEST_JWT);
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("HS256"));
    }

    #[test]
    fn test_jwt_decode_invalid_format() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Not a valid JWT (no dots)
        let data = json!("not-a-jwt");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));

        // Only two parts
        let data = json!("part1.part2");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_jwt_decode_invalid_base64() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Three parts but invalid base64
        let data = json!("!!!.@@@.###");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_jwt_decode_invalid_json() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Valid base64 but not valid JSON - "not json" encoded
        let data = json!("eyJhbGciOiJIUzI1NiJ9.bm90IGpzb24.sig");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_html_escape_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = json!("<div class=\"test\">Hello & goodbye</div>");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!("&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;")
        );
    }

    #[test]
    fn test_html_escape_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = json!("It's a \"test\"");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("It&#x27;s a &quot;test&quot;"));
    }

    #[test]
    fn test_html_escape_no_change() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = json!("Hello World");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("Hello World"));
    }

    #[test]
    fn test_html_unescape_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_unescape(@)").unwrap();
        let data = json!("&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("<div class=\"test\">Hello & goodbye</div>"));
    }

    #[test]
    fn test_html_unescape_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_unescape(@)").unwrap();
        let data = json!("It&#x27;s a &quot;test&quot;");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("It's a \"test\""));
    }

    #[test]
    fn test_html_roundtrip() {
        let runtime = setup_runtime();
        let escape = runtime.compile("html_escape(@)").unwrap();
        let unescape = runtime.compile("html_unescape(@)").unwrap();
        let original = "<script>alert('xss')</script>";
        let data = json!(original);
        let escaped = escape.search(&data).unwrap();
        let roundtrip = unescape.search(&escaped).unwrap();
        assert_eq!(roundtrip, json!(original));
    }

    #[test]
    fn test_shell_escape_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let data = json!("hello world");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("'hello world'"));
    }

    #[test]
    fn test_shell_escape_with_single_quote() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let data = json!("it's here");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("'it'\\''s here'"));
    }

    #[test]
    fn test_shell_escape_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let data = json!("$HOME; rm -rf /");
        let result = expr.search(&data).unwrap();
        // Should be safely quoted
        assert_eq!(result, json!("'$HOME; rm -rf /'"));
    }

    #[test]
    fn test_shell_escape_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("''"));
    }

    #[test]
    fn test_shell_escape_multiple_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let data = json!("don't say 'hello'");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("'don'\\''t say '\\''hello'\\'''"));
    }

    // =========================================================================
    // base64url function tests
    // =========================================================================

    #[test]
    fn test_base64url_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_encode(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("aGVsbG8"));
    }

    #[test]
    fn test_base64url_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_decode(@)").unwrap();
        let data = json!("aGVsbG8");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn test_base64url_roundtrip() {
        let runtime = setup_runtime();
        let encode = runtime.compile("base64url_encode(@)").unwrap();
        let decode = runtime.compile("base64url_decode(@)").unwrap();
        let original = "hello world! 🌍";
        let data = json!(original);
        let encoded = encode.search(&data).unwrap();
        let roundtrip = decode.search(&encoded).unwrap();
        assert_eq!(roundtrip, json!(original));
    }

    #[test]
    fn test_base64url_no_padding() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_encode(@)").unwrap();
        // "test" in standard base64 is "dGVzdA==" — url-safe should have no padding
        let data = json!("test");
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(
            !s.contains('='),
            "base64url output should not contain padding"
        );
        assert_eq!(s, "dGVzdA");
    }

    #[test]
    fn test_base64url_uses_url_safe_chars() {
        let runtime = setup_runtime();
        let encode_url = runtime.compile("base64url_encode(@)").unwrap();
        let encode_std = runtime.compile("base64_encode(@)").unwrap();
        // Input that produces +/ in standard base64: bytes 0xFB, 0xFF, 0xBF
        // "????" encodes differently in standard vs url-safe
        // Use a known input: "\xfb\xef\xbe" → standard: "u+++" url-safe: "u--+"
        // Actually, let's use a string whose standard base64 contains + or /
        let data = json!("subjects?_d");
        let std_result = encode_std.search(&data).unwrap();
        let url_result = encode_url.search(&data).unwrap();
        let std_s = std_result.as_str().unwrap();
        let url_s = url_result.as_str().unwrap();
        // URL-safe should never contain + or /
        assert!(!url_s.contains('+'), "base64url should not contain '+'");
        assert!(!url_s.contains('/'), "base64url should not contain '/'");
        // If standard contains + or /, url-safe should have - or _ instead
        if std_s.contains('+') || std_s.contains('/') {
            assert_ne!(std_s.trim_end_matches('='), url_s);
        }
    }

    #[test]
    fn test_base64url_decode_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_decode(@)").unwrap();
        let data = json!("!!!invalid!!!");
        let result = expr.search(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_base64url_encode_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_encode(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(""));
    }

    #[test]
    fn test_base64url_decode_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64url_decode(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(""));
    }
}
