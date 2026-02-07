//! Cryptographic hash functions.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

use crc32fast::Hasher as Crc32Hasher;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

// Type aliases for HMAC variants
type HmacMd5 = Hmac<Md5>;
type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Register hash functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    // Hash functions
    register_if_enabled(runtime, "md5", enabled, Box::new(Md5Fn::new()));
    register_if_enabled(runtime, "sha1", enabled, Box::new(Sha1Fn::new()));
    register_if_enabled(runtime, "sha256", enabled, Box::new(Sha256Fn::new()));
    register_if_enabled(runtime, "sha512", enabled, Box::new(Sha512Fn::new()));

    // HMAC functions
    register_if_enabled(runtime, "hmac_md5", enabled, Box::new(HmacMd5Fn::new()));
    register_if_enabled(runtime, "hmac_sha1", enabled, Box::new(HmacSha1Fn::new()));
    register_if_enabled(
        runtime,
        "hmac_sha256",
        enabled,
        Box::new(HmacSha256Fn::new()),
    );
    register_if_enabled(
        runtime,
        "hmac_sha512",
        enabled,
        Box::new(HmacSha512Fn::new()),
    );

    // Checksum functions
    register_if_enabled(runtime, "crc32", enabled, Box::new(Crc32Fn::new()));
}

// =============================================================================
// md5(string) -> string (hex-encoded MD5 hash)
// =============================================================================

defn!(Md5Fn, vec![arg!(string)], None);

impl Function for Md5Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Md5::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// sha1(string) -> string (hex-encoded SHA-1 hash)
// =============================================================================

defn!(Sha1Fn, vec![arg!(string)], None);

impl Function for Sha1Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha1::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// sha256(string) -> string (hex-encoded SHA-256 hash)
// =============================================================================

defn!(Sha256Fn, vec![arg!(string)], None);

impl Function for Sha256Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// sha512(string) -> string (hex-encoded SHA-512 hash)
// =============================================================================

defn!(Sha512Fn, vec![arg!(string)], None);

impl Function for Sha512Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_string = format!("{:x}", result);

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// hmac_md5(text, key) -> string (hex-encoded HMAC-MD5)
// =============================================================================

defn!(HmacMd5Fn, vec![arg!(string), arg!(string)], None);

impl Function for HmacMd5Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacMd5::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// hmac_sha1(text, key) -> string (hex-encoded HMAC-SHA1)
// =============================================================================

defn!(HmacSha1Fn, vec![arg!(string), arg!(string)], None);

impl Function for HmacSha1Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// hmac_sha256(text, key) -> string (hex-encoded HMAC-SHA256)
// =============================================================================

defn!(HmacSha256Fn, vec![arg!(string), arg!(string)], None);

impl Function for HmacSha256Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// hmac_sha512(text, key) -> string (hex-encoded HMAC-SHA512)
// =============================================================================

defn!(HmacSha512Fn, vec![arg!(string), arg!(string)], None);

impl Function for HmacSha512Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for text argument".to_owned()),
            )
        })?;

        let key = args[1].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string for key argument".to_owned()),
            )
        })?;

        let mut mac =
            HmacSha512::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(text.as_bytes());
        let result = mac.finalize();
        let hex_string = format!("{:x}", result.into_bytes());

        Ok(Value::String(hex_string))
    }
}

// =============================================================================
// crc32(string) -> number (CRC32 checksum as integer)
// =============================================================================

defn!(Crc32Fn, vec![arg!(string)], None);

impl Function for Crc32Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().ok_or_else(|| {
            crate::JmespathError::from_ctx(
                ctx,
                crate::ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let mut hasher = Crc32Hasher::new();
        hasher.update(input.as_bytes());
        let checksum = hasher.finalize();

        Ok(Value::Number(serde_json::Number::from(checksum)))
    }
}
