//! Data validation functions.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// Constant validation patterns, compiled once instead of on every call (these
// were previously recompiled per invocation, e.g. once per element in a map).
static EMAIL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());
static URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());
static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .unwrap()
});
static PHONE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\+?[\d\s\-\(\)\.]{7,}$").unwrap());

/// Register validation functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "is_email", enabled, Box::new(IsEmailFn::new()));
    register_if_enabled(runtime, "is_url", enabled, Box::new(IsUrlFn::new()));
    register_if_enabled(runtime, "is_uuid", enabled, Box::new(IsUuidFn::new()));
    register_if_enabled(runtime, "is_phone", enabled, Box::new(IsPhoneFn::new()));
    register_if_enabled(runtime, "is_ipv4", enabled, Box::new(IsIpv4Fn::new()));
    register_if_enabled(runtime, "is_ipv6", enabled, Box::new(IsIpv6Fn::new()));
    register_if_enabled(runtime, "luhn_check", enabled, Box::new(LuhnCheckFn::new()));
    register_if_enabled(
        runtime,
        "is_credit_card",
        enabled,
        Box::new(IsCreditCardFn::new()),
    );
    register_if_enabled(runtime, "is_jwt", enabled, Box::new(IsJwtFn::new()));
    register_if_enabled(
        runtime,
        "is_iso_date",
        enabled,
        Box::new(IsIsoDateFn::new()),
    );
    register_if_enabled(runtime, "is_json", enabled, Box::new(IsJsonFn::new()));
    register_if_enabled(runtime, "is_base64", enabled, Box::new(IsBase64Fn::new()));
    register_if_enabled(runtime, "is_hex", enabled, Box::new(IsHexFn::new()));
}

// =============================================================================
// is_email(string) -> boolean
// =============================================================================

defn!(IsEmailFn, vec![arg!(string)], None);

impl Function for IsEmailFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        Ok(Value::Bool(EMAIL_RE.is_match(s)))
    }
}

// =============================================================================
// is_url(string) -> boolean
// =============================================================================

defn!(IsUrlFn, vec![arg!(string)], None);

impl Function for IsUrlFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        Ok(Value::Bool(URL_RE.is_match(s)))
    }
}

// =============================================================================
// is_uuid(string) -> boolean
// =============================================================================

defn!(IsUuidFn, vec![arg!(string)], None);

impl Function for IsUuidFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        Ok(Value::Bool(UUID_RE.is_match(s)))
    }
}

// =============================================================================
// is_ipv4(string) -> boolean
// =============================================================================

defn!(IsIpv4Fn, vec![arg!(string)], None);

impl Function for IsIpv4Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        let is_valid = s.parse::<std::net::Ipv4Addr>().is_ok();
        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// is_ipv6(string) -> boolean
// =============================================================================

defn!(IsIpv6Fn, vec![arg!(string)], None);

impl Function for IsIpv6Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        let is_valid = s.parse::<std::net::Ipv6Addr>().is_ok();
        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// luhn_check(string) -> boolean
// =============================================================================

defn!(LuhnCheckFn, vec![arg!(string)], None);

impl Function for LuhnCheckFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        Ok(Value::Bool(luhn_validate(s)))
    }
}

fn luhn_validate(s: &str) -> bool {
    // Remove spaces and dashes
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for c in digits.chars().rev() {
        if let Some(digit) = c.to_digit(10) {
            let mut d = digit;
            if double {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
            double = !double;
        } else {
            return false;
        }
    }

    sum % 10 == 0
}

// =============================================================================
// is_credit_card(string) -> boolean
// =============================================================================

defn!(IsCreditCardFn, vec![arg!(string)], None);

impl Function for IsCreditCardFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        // Remove spaces and dashes
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // Credit cards are typically 13-19 digits
        if digits.len() < 13 || digits.len() > 19 {
            return Ok(Value::Bool(false));
        }

        // Must pass Luhn check
        Ok(Value::Bool(luhn_validate(&digits)))
    }
}

// =============================================================================
// is_phone(string) -> boolean
// =============================================================================

defn!(IsPhoneFn, vec![arg!(string)], None);

impl Function for IsPhoneFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        // Basic phone pattern: optional + followed by digits, spaces, dashes, parens
        // Minimum 7 digits for a valid phone number
        if !PHONE_RE.is_match(s) {
            return Ok(Value::Bool(false));
        }

        // Count actual digits - need at least 7
        let digit_count = s.chars().filter(|c| c.is_ascii_digit()).count();
        Ok(Value::Bool((7..=15).contains(&digit_count)))
    }
}

// =============================================================================
// is_jwt(string) -> boolean
// =============================================================================

defn!(IsJwtFn, vec![arg!(string)], None);

impl Function for IsJwtFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        // JWT has 3 base64url-encoded parts separated by dots
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Ok(Value::Bool(false));
        }

        // Check each part is valid base64url (alphanumeric, -, _, no padding required)
        let is_valid = parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
        });

        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// is_iso_date(string) -> boolean
// =============================================================================

defn!(IsIsoDateFn, vec![arg!(string)], None);

impl Function for IsIsoDateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        // Try parsing as RFC3339 (subset of ISO 8601)
        if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
            return Ok(Value::Bool(true));
        }

        // Try parsing as date only (YYYY-MM-DD)
        if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
            return Ok(Value::Bool(true));
        }

        // Try parsing as datetime without timezone
        if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok() {
            return Ok(Value::Bool(true));
        }

        Ok(Value::Bool(false))
    }
}

// =============================================================================
// is_json(string) -> boolean
// =============================================================================

defn!(IsJsonFn, vec![arg!(string)], None);

impl Function for IsJsonFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();
        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// is_base64(string) -> boolean
// =============================================================================

defn!(IsBase64Fn, vec![arg!(string)], None);

impl Function for IsBase64Fn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        use base64::{Engine, engine::general_purpose::STANDARD};
        let is_valid = STANDARD.decode(s).is_ok();
        Ok(Value::Bool(is_valid))
    }
}

// =============================================================================
// is_hex(string) -> boolean
// =============================================================================

defn!(IsHexFn, vec![arg!(string)], None);

impl Function for IsHexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        // Must be non-empty and all hex chars
        let is_valid = !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit());
        Ok(Value::Bool(is_valid))
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
    fn test_is_ipv4() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_ipv4(@)").unwrap();

        let data = json!("192.168.1.1");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("not an ip");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_ipv6() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_ipv6(@)").unwrap();

        let data = json!("::1");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("2001:db8::1");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_email() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_email(@)").unwrap();

        let data = json!("test@example.com");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("not-an-email");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_luhn_check_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("luhn_check(@)").unwrap();

        // Valid Luhn number
        let data = json!("79927398713");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_luhn_check_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("luhn_check(@)").unwrap();

        let data = json!("79927398710");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_credit_card_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        // Test Visa number (passes Luhn)
        let data = json!("4111111111111111");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_credit_card_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        // Invalid number
        let data = json!("1234567890123456");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_credit_card_too_short() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        let data = json!("123456");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_phone_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_phone(@)").unwrap();

        let data = json!("+1-555-123-4567");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("(555) 123-4567");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_phone_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_phone(@)").unwrap();

        let data = json!("123");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_jwt_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_jwt(@)").unwrap();

        let data = json!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U"
        );
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_jwt_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_jwt(@)").unwrap();

        // Only two parts - invalid
        let data = json!("only.twoparts");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));

        // Contains invalid characters for base64url
        let data = json!("abc.def!ghi.jkl");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_iso_date_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_iso_date(@)").unwrap();

        let data = json!("2023-12-13T15:30:00Z");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("2023-12-13");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_iso_date_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_iso_date(@)").unwrap();

        let data = json!("12/13/2023");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_json_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_json(@)").unwrap();

        let data = json!(r#"{"a": 1, "b": [2, 3]}"#);
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_json_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_json(@)").unwrap();

        let data = json!("not json");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_base64_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_base64(@)").unwrap();

        let data = json!("SGVsbG8gV29ybGQ=");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_base64_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_base64(@)").unwrap();

        let data = json!("not valid base64!!!");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_is_hex_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_hex(@)").unwrap();

        let data = json!("deadbeef");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));

        let data = json!("ABCDEF0123456789");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_is_hex_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_hex(@)").unwrap();

        let data = json!("not hex!");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));

        let data = json!("");
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }
}
