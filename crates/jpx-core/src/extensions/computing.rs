//! Computing and bitwise functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// Decimal units (SI): KB, MB, GB, TB, PB
const DECIMAL_UNITS: &[(&str, f64)] = &[
    ("PB", 1e15),
    ("TB", 1e12),
    ("GB", 1e9),
    ("MB", 1e6),
    ("KB", 1e3),
    ("B", 1.0),
];

// Binary units (IEC): KiB, MiB, GiB, TiB, PiB
const BINARY_UNITS: &[(&str, f64)] = &[
    ("PiB", 1125899906842624.0),
    ("TiB", 1099511627776.0),
    ("GiB", 1073741824.0),
    ("MiB", 1048576.0),
    ("KiB", 1024.0),
    ("B", 1.0),
];

defn!(ParseBytesFn, vec![arg!(string)], None);

impl Function for ParseBytesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        match parse_bytes_str(s) {
            Some(bytes) => Ok(number_value(bytes)),
            None => Ok(Value::Null),
        }
    }
}

defn!(FormatBytesFn, vec![arg!(number)], None);

impl Function for FormatBytesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let bytes = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let formatted = format_bytes_with_units(bytes, DECIMAL_UNITS);
        Ok(Value::String(formatted))
    }
}

defn!(FormatBytesBinaryFn, vec![arg!(number)], None);

impl Function for FormatBytesBinaryFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let bytes = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let formatted = format_bytes_with_units(bytes, BINARY_UNITS);
        Ok(Value::String(formatted))
    }
}

defn!(BitAndFn, vec![arg!(number), arg!(number)], None);

impl Function for BitAndFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        let b = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        Ok(Value::Number(Number::from(a & b)))
    }
}

defn!(BitOrFn, vec![arg!(number), arg!(number)], None);

impl Function for BitOrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        let b = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        Ok(Value::Number(Number::from(a | b)))
    }
}

defn!(BitXorFn, vec![arg!(number), arg!(number)], None);

impl Function for BitXorFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        let b = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        Ok(Value::Number(Number::from(a ^ b)))
    }
}

defn!(BitNotFn, vec![arg!(number)], None);

impl Function for BitNotFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        Ok(Value::Number(Number::from(!a)))
    }
}

defn!(BitShiftLeftFn, vec![arg!(number), arg!(number)], None);

impl Function for BitShiftLeftFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected non-negative integer"))?
            as u32;

        Ok(Value::Number(Number::from(a << n)))
    }
}

defn!(BitShiftRightFn, vec![arg!(number), arg!(number)], None);

impl Function for BitShiftRightFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let a = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected integer"))?
            as i64;

        let n = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected non-negative integer"))?
            as u32;

        Ok(Value::Number(Number::from(a >> n)))
    }
}

// Helper functions

/// Parse a byte string like "1.5 GB" or "100 MiB" into bytes.
fn parse_bytes_str(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut num_end = 0;
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' {
            num_end = i + 1;
        } else if !c.is_whitespace() {
            break;
        }
    }

    if num_end == 0 {
        return None;
    }

    let num_str: String = chars[..num_end].iter().collect();
    let num: f64 = num_str.trim().parse().ok()?;

    let unit: String = chars[num_end..].iter().collect();
    let unit = unit.trim().to_uppercase();

    if unit.is_empty() || unit == "B" || unit == "BYTES" || unit == "BYTE" {
        return Some(num);
    }

    let multiplier = match unit.as_str() {
        "PIB" | "PEBIBYTE" | "PEBIBYTES" => 1125899906842624.0,
        "TIB" | "TEBIBYTE" | "TEBIBYTES" => 1099511627776.0,
        "GIB" | "GIBIBYTE" | "GIBIBYTES" => 1073741824.0,
        "MIB" | "MEBIBYTE" | "MEBIBYTES" => 1048576.0,
        "KIB" | "KIBIBYTE" | "KIBIBYTES" => 1024.0,
        "PB" | "PETABYTE" | "PETABYTES" => 1e15,
        "TB" | "TERABYTE" | "TERABYTES" => 1e12,
        "GB" | "GIGABYTE" | "GIGABYTES" => 1e9,
        "MB" | "MEGABYTE" | "MEGABYTES" => 1e6,
        "KB" | "KILOBYTE" | "KILOBYTES" => 1e3,
        _ => return None,
    };

    Some(num * multiplier)
}

/// Format bytes with the given unit system.
fn format_bytes_with_units(bytes: f64, units: &[(&str, f64)]) -> String {
    if bytes == 0.0 {
        return "0 B".to_string();
    }

    let abs_bytes = bytes.abs();

    for (unit, threshold) in units {
        if abs_bytes >= *threshold {
            let value = bytes / threshold;
            let formatted = format!("{:.2}", value);
            let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
            return format!("{} {}", formatted, unit);
        }
    }

    format!("{} B", bytes)
}

/// Register computing functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "parse_bytes",
        enabled,
        Box::new(ParseBytesFn::new()),
    );
    register_if_enabled(
        runtime,
        "format_bytes",
        enabled,
        Box::new(FormatBytesFn::new()),
    );
    register_if_enabled(
        runtime,
        "format_bytes_binary",
        enabled,
        Box::new(FormatBytesBinaryFn::new()),
    );
    register_if_enabled(runtime, "bit_and", enabled, Box::new(BitAndFn::new()));
    register_if_enabled(runtime, "bit_or", enabled, Box::new(BitOrFn::new()));
    register_if_enabled(runtime, "bit_xor", enabled, Box::new(BitXorFn::new()));
    register_if_enabled(runtime, "bit_not", enabled, Box::new(BitNotFn::new()));
    register_if_enabled(
        runtime,
        "bit_shift_left",
        enabled,
        Box::new(BitShiftLeftFn::new()),
    );
    register_if_enabled(
        runtime,
        "bit_shift_right",
        enabled,
        Box::new(BitShiftRightFn::new()),
    );
}
