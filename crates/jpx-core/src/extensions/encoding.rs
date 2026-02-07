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
