//! Data validation functions.

use std::collections::HashSet;

use regex::Regex;
use serde_json::Value;

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

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

        let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        Ok(Value::Bool(email_re.is_match(s)))
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

        let url_re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        Ok(Value::Bool(url_re.is_match(s)))
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

        let uuid_re = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .unwrap();
        Ok(Value::Bool(uuid_re.is_match(s)))
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
        let phone_re = Regex::new(r"^\+?[\d\s\-\(\)\.]{7,}$").unwrap();
        if !phone_re.is_match(s) {
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
