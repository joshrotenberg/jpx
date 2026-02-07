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
}
