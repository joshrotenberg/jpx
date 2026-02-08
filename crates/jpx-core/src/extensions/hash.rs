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

    // =========================================================================
    // Hash function tests
    // =========================================================================

    #[test]
    fn test_md5() {
        let runtime = setup_runtime();
        let expr = runtime.compile("md5(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("5d41402abc4b2a76b9719d911017c592"));
    }

    #[test]
    fn test_md5_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("md5(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("d41d8cd98f00b204e9800998ecf8427e"));
    }

    #[test]
    fn test_sha1() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha1(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"));
    }

    #[test]
    fn test_sha256() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha256(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
        );
    }

    #[test]
    fn test_sha512() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha512(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!(
                "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043"
            )
        );
    }

    #[test]
    fn test_sha512_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sha512(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!(
                "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
            )
        );
    }

    // =========================================================================
    // HMAC function tests
    // =========================================================================

    #[test]
    fn test_hmac_md5() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_md5(@, `\"secret\"`)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("bade63863c61ed0b3165806ecd6acefc"));
    }

    #[test]
    fn test_hmac_sha1() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha1(@, `\"secret\"`)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!("5112055c05f944f85755efc5cd8970e194e9f45b"));
    }

    #[test]
    fn test_hmac_sha256() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"secret\"`)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!("88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b")
        );
    }

    #[test]
    fn test_hmac_sha512() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha512(@, `\"secret\"`)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!(
                "db1595ae88a62fd151ec1cba81b98c39df82daae7b4cb9820f446d5bf02f1dcfca6683d88cab3e273f5963ab8ec469a746b5b19086371239f67d1e5f99a79440"
            )
        );
    }

    #[test]
    fn test_hmac_sha256_empty_message() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"key\"`)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result,
            json!("5d5d139563c95b5967b9bd9a8c9b233a9dedb45072794cd232dc1b74832607d0")
        );
    }

    #[test]
    fn test_hmac_sha256_empty_key() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hmac_sha256(@, `\"\"`)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        // HMAC with empty key is valid
        assert!(!result.as_str().unwrap().is_empty());
    }

    // =========================================================================
    // CRC32 tests
    // =========================================================================

    #[test]
    fn test_crc32() {
        let runtime = setup_runtime();
        let expr = runtime.compile("crc32(@)").unwrap();
        let data = json!("hello");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(907060870));
    }

    #[test]
    fn test_crc32_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("crc32(@)").unwrap();
        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(0));
    }
}
