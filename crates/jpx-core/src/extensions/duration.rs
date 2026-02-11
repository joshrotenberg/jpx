//! Duration parsing and formatting functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

defn!(ParseDurationFn, vec![arg!(string)], None);

impl Function for ParseDurationFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        match parse_duration_str(s) {
            Some(secs) => Ok(number_value(secs as f64)),
            None => Ok(Value::Null),
        }
    }
}

defn!(FormatDurationFn, vec![arg!(number)], None);

impl Function for FormatDurationFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let total_secs = num as u64;
        let formatted = format_duration_secs(total_secs);

        Ok(Value::String(formatted))
    }
}

defn!(DurationHoursFn, vec![arg!(number)], None);

impl Function for DurationHoursFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let total_secs = num as u64;
        let hours = (total_secs / 3600) % 24;

        Ok(Value::Number(Number::from(hours)))
    }
}

defn!(DurationMinutesFn, vec![arg!(number)], None);

impl Function for DurationMinutesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let total_secs = num as u64;
        let minutes = (total_secs / 60) % 60;

        Ok(Value::Number(Number::from(minutes)))
    }
}

defn!(DurationSecondsFn, vec![arg!(number)], None);

impl Function for DurationSecondsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let total_secs = num as u64;
        let seconds = total_secs % 60;

        Ok(Value::Number(Number::from(seconds)))
    }
}

defn!(DurationDaysFn, vec![arg!(number)], None);

impl Function for DurationDaysFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let num = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let total_secs = num as u64;
        let days = (total_secs / 86400) % 7;

        Ok(Value::Number(Number::from(days)))
    }
}

defn!(DurationAddFn, vec![arg!(string), arg!(string)], None);

impl Function for DurationAddFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;
        let b = args[1]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let secs_a = parse_duration_str(a)
            .ok_or_else(|| crate::functions::custom_error(ctx, "Invalid duration string"))?;
        let secs_b = parse_duration_str(b)
            .ok_or_else(|| crate::functions::custom_error(ctx, "Invalid duration string"))?;

        Ok(Value::String(format_duration_secs(secs_a + secs_b)))
    }
}

defn!(DurationSubtractFn, vec![arg!(string), arg!(string)], None);

impl Function for DurationSubtractFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;
        let b = args[1]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let secs_a = parse_duration_str(a)
            .ok_or_else(|| crate::functions::custom_error(ctx, "Invalid duration string"))?;
        let secs_b = parse_duration_str(b)
            .ok_or_else(|| crate::functions::custom_error(ctx, "Invalid duration string"))?;

        Ok(Value::String(format_duration_secs(
            secs_a.saturating_sub(secs_b),
        )))
    }
}

/// Parse a duration string into total seconds.
fn parse_duration_str(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let mut total_secs: u64 = 0;
    let mut current_num = String::new();

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_ascii_digit() {
            current_num.push(c);
            i += 1;
        } else if c.is_ascii_alphabetic() {
            let num: u64 = if current_num.is_empty() {
                return None;
            } else {
                current_num.parse().ok()?
            };
            current_num.clear();

            let mut unit = String::new();
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                unit.push(chars[i]);
                i += 1;
            }

            let multiplier = match unit.as_str() {
                "w" | "week" | "weeks" => 7 * 24 * 3600,
                "d" | "day" | "days" => 24 * 3600,
                "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
                "m" | "min" | "mins" | "minute" | "minutes" => 60,
                "s" | "sec" | "secs" | "second" | "seconds" => 1,
                _ => return None,
            };

            total_secs += num * multiplier;
        } else if c.is_whitespace() {
            i += 1;
        } else {
            return None;
        }
    }

    if !current_num.is_empty() {
        let num: u64 = current_num.parse().ok()?;
        total_secs += num;
    }

    Some(total_secs)
}

/// Format seconds as a human-readable duration string.
fn format_duration_secs(total_secs: u64) -> String {
    if total_secs == 0 {
        return "0s".to_string();
    }

    let weeks = total_secs / (7 * 24 * 3600);
    let days = (total_secs / (24 * 3600)) % 7;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs / 60) % 60;
    let seconds = total_secs % 60;

    let mut result = String::new();

    if weeks > 0 {
        result.push_str(&format!("{}w", weeks));
    }
    if days > 0 {
        result.push_str(&format!("{}d", days));
    }
    if hours > 0 {
        result.push_str(&format!("{}h", hours));
    }
    if minutes > 0 {
        result.push_str(&format!("{}m", minutes));
    }
    if seconds > 0 {
        result.push_str(&format!("{}s", seconds));
    }

    result
}

/// Register duration functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "parse_duration",
        enabled,
        Box::new(ParseDurationFn::new()),
    );
    register_if_enabled(
        runtime,
        "format_duration",
        enabled,
        Box::new(FormatDurationFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_add",
        enabled,
        Box::new(DurationAddFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_days",
        enabled,
        Box::new(DurationDaysFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_hours",
        enabled,
        Box::new(DurationHoursFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_minutes",
        enabled,
        Box::new(DurationMinutesFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_seconds",
        enabled,
        Box::new(DurationSecondsFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_subtract",
        enabled,
        Box::new(DurationSubtractFn::new()),
    );
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    use super::{format_duration_secs, parse_duration_str};

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration_str("1h"), Some(3600));
        assert_eq!(parse_duration_str("30m"), Some(1800));
        assert_eq!(parse_duration_str("45s"), Some(45));
        assert_eq!(parse_duration_str("1h30m"), Some(5400));
        assert_eq!(parse_duration_str("2h30m45s"), Some(9045));
        assert_eq!(parse_duration_str("1d"), Some(86400));
        assert_eq!(parse_duration_str("1w"), Some(604800));
        assert_eq!(parse_duration_str("1w2d3h4m5s"), Some(788645));
        assert_eq!(parse_duration_str("1 hour 30 minutes"), Some(5400));
        assert_eq!(parse_duration_str(""), None);
        assert_eq!(parse_duration_str("invalid"), None);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration_secs(0), "0s");
        assert_eq!(format_duration_secs(45), "45s");
        assert_eq!(format_duration_secs(60), "1m");
        assert_eq!(format_duration_secs(3600), "1h");
        assert_eq!(format_duration_secs(5400), "1h30m");
        assert_eq!(format_duration_secs(86400), "1d");
        assert_eq!(format_duration_secs(90061), "1d1h1m1s");
        assert_eq!(format_duration_secs(788645), "1w2d3h4m5s");
    }

    #[test]
    fn test_roundtrip() {
        let values = [0, 45, 60, 3600, 5400, 86400, 90061, 788645];
        for &v in &values {
            let formatted = format_duration_secs(v);
            let parsed = parse_duration_str(&formatted).unwrap_or(0);
            assert_eq!(
                parsed, v,
                "Roundtrip failed for {}: {} -> {}",
                v, formatted, parsed
            );
        }
    }

    #[test]
    fn test_parse_duration_via_runtime() {
        let runtime = setup_runtime();
        let data = json!("1h30m");
        let expr = runtime.compile("parse_duration(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 5400.0);
    }

    #[test]
    fn test_format_duration_via_runtime() {
        let runtime = setup_runtime();
        let expr = runtime.compile("format_duration(`5400`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1h30m");
    }

    #[test]
    fn test_duration_hours_via_runtime() {
        let runtime = setup_runtime();
        // 90061 seconds = 1d 1h 1m 1s -> hours component is 1
        let expr = runtime.compile("duration_hours(`90061`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);
    }

    #[test]
    fn test_duration_minutes_via_runtime() {
        let runtime = setup_runtime();
        // 90061 seconds = 1d 1h 1m 1s -> minutes component is 1
        let expr = runtime.compile("duration_minutes(`90061`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);
    }

    #[test]
    fn test_duration_seconds_via_runtime() {
        let runtime = setup_runtime();
        // 90061 seconds = 1d 1h 1m 1s -> seconds component is 1
        let expr = runtime.compile("duration_seconds(`90061`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);
    }

    #[test]
    fn test_duration_days_via_runtime() {
        let runtime = setup_runtime();
        // 86400 seconds = 1d -> days component is 1
        let expr = runtime.compile("duration_days(`86400`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);

        // 90061 seconds = 1d 1h 1m 1s -> days component is 1
        let expr = runtime.compile("duration_days(`90061`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);

        // 691200 seconds = 8 days = 1w1d -> days component is 8%7 = 1
        let expr = runtime.compile("duration_days(`691200`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_i64().unwrap(), 1);
    }

    #[test]
    fn test_duration_add_via_runtime() {
        let runtime = setup_runtime();

        let expr = runtime.compile("duration_add('1h', '30m')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1h30m");

        let expr = runtime.compile("duration_add('1d', '12h')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1d12h");

        let expr = runtime.compile("duration_add('1h', '0s')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1h");
    }

    #[test]
    fn test_duration_subtract_via_runtime() {
        let runtime = setup_runtime();

        let expr = runtime.compile("duration_subtract('2h', '30m')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1h30m");

        // Equal durations -> "0s"
        let expr = runtime.compile("duration_subtract('1h', '1h')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "0s");

        // Underflow clamps to "0s"
        let expr = runtime.compile("duration_subtract('30m', '2h')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "0s");
    }
}
