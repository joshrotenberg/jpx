//! Color manipulation functions.

use std::collections::HashSet;

use serde_json::{Number, Value};

use crate::functions::Function;
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

defn!(HexToRgbFn, vec![arg!(string)], None);

impl Function for HexToRgbFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        match parse_hex_color(hex) {
            Some((r, g, b)) => {
                let mut map = serde_json::Map::new();
                map.insert("r".to_string(), Value::Number(Number::from(r)));
                map.insert("g".to_string(), Value::Number(Number::from(g)));
                map.insert("b".to_string(), Value::Number(Number::from(b)));
                Ok(Value::Object(map))
            }
            None => Ok(Value::Null),
        }
    }
}

defn!(
    RgbToHexFn,
    vec![arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for RgbToHexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let r = args[0]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number for r"))?
            as u8;

        let g = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number for g"))?
            as u8;

        let b = args[2]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number for b"))?
            as u8;

        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(Value::String(hex))
    }
}

defn!(LightenFn, vec![arg!(string), arg!(number)], None);

impl Function for LightenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let amount = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        let factor = (amount / 100.0).clamp(0.0, 1.0);
        let r = (r as f64 + (255.0 - r as f64) * factor).round() as u8;
        let g = (g as f64 + (255.0 - g as f64) * factor).round() as u8;
        let b = (b as f64 + (255.0 - b as f64) * factor).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(Value::String(result))
    }
}

defn!(DarkenFn, vec![arg!(string), arg!(number)], None);

impl Function for DarkenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let amount = args[1]
            .as_f64()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected number"))?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        let factor = 1.0 - (amount / 100.0).clamp(0.0, 1.0);
        let r = (r as f64 * factor).round() as u8;
        let g = (g as f64 * factor).round() as u8;
        let b = (b as f64 * factor).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(Value::String(result))
    }
}

defn!(
    ColorMixFn,
    vec![arg!(string), arg!(string)],
    Some(arg!(number))
);

impl Function for ColorMixFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex1 = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let hex2 = args[1]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let weight = if args.len() > 2 {
            args[2].as_f64().unwrap_or(0.5)
        } else {
            0.5
        };

        let (r1, g1, b1) = match parse_hex_color(hex1) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        let (r2, g2, b2) = match parse_hex_color(hex2) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        let w = weight.clamp(0.0, 1.0);
        let r = (r1 as f64 * (1.0 - w) + r2 as f64 * w).round() as u8;
        let g = (g1 as f64 * (1.0 - w) + g2 as f64 * w).round() as u8;
        let b = (b1 as f64 * (1.0 - w) + b2 as f64 * w).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(Value::String(result))
    }
}

defn!(ColorInvertFn, vec![arg!(string)], None);

impl Function for ColorInvertFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        let result = format!("#{:02x}{:02x}{:02x}", 255 - r, 255 - g, 255 - b);
        Ok(Value::String(result))
    }
}

defn!(ColorGrayscaleFn, vec![arg!(string)], None);

impl Function for ColorGrayscaleFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        // Use luminance formula: 0.299*R + 0.587*G + 0.114*B
        let gray = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", gray, gray, gray);
        Ok(Value::String(result))
    }
}

defn!(ColorComplementFn, vec![arg!(string)], None);

impl Function for ColorComplementFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let hex = args[0]
            .as_str()
            .ok_or_else(|| crate::functions::custom_error(ctx, "Expected string"))?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(Value::Null),
        };

        // Convert to HSL, rotate hue by 180, convert back
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let new_h = (h + 180.0) % 360.0;
        let (r, g, b) = hsl_to_rgb(new_h, s, l);

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(Value::String(result))
    }
}

// Helper functions

/// Parse a hex color string into RGB components.
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// Convert RGB to HSL.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f64::EPSILON {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < f64::EPSILON {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s, l)
}

/// Convert HSL to RGB.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let h = h / 360.0;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Register color functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "hex_to_rgb", enabled, Box::new(HexToRgbFn::new()));
    register_if_enabled(runtime, "rgb_to_hex", enabled, Box::new(RgbToHexFn::new()));
    register_if_enabled(runtime, "lighten", enabled, Box::new(LightenFn::new()));
    register_if_enabled(runtime, "darken", enabled, Box::new(DarkenFn::new()));
    register_if_enabled(runtime, "color_mix", enabled, Box::new(ColorMixFn::new()));
    register_if_enabled(
        runtime,
        "color_invert",
        enabled,
        Box::new(ColorInvertFn::new()),
    );
    register_if_enabled(
        runtime,
        "color_grayscale",
        enabled,
        Box::new(ColorGrayscaleFn::new()),
    );
    register_if_enabled(
        runtime,
        "color_complement",
        enabled,
        Box::new(ColorComplementFn::new()),
    );
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
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("ff5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("#f50"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_rgb_to_hsl_roundtrip() {
        let colors = [
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 128, 128),
            (255, 128, 64),
        ];
        for (r, g, b) in colors {
            let (h, s, l) = super::rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = super::hsl_to_rgb(h, s, l);
            assert!(
                (r as i16 - r2 as i16).abs() <= 1,
                "Red mismatch: {} vs {}",
                r,
                r2
            );
            assert!(
                (g as i16 - g2 as i16).abs() <= 1,
                "Green mismatch: {} vs {}",
                g,
                g2
            );
            assert!(
                (b as i16 - b2 as i16).abs() <= 1,
                "Blue mismatch: {} vs {}",
                b,
                b2
            );
        }
    }

    #[test]
    fn test_hex_to_rgb() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_to_rgb('#ff5500')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result, json!({"r": 255, "g": 85, "b": 0}));
    }

    #[test]
    fn test_rgb_to_hex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("rgb_to_hex(`255`, `85`, `0`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ff5500");
    }

    use super::parse_hex_color;

    #[test]
    fn test_hex_to_rgb_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_to_rgb('zzzzzz')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_lighten() {
        let runtime = setup_runtime();
        // Lighten black by 50% should give #808080 (gray)
        let expr = runtime.compile("lighten('#000000', `50`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#808080");

        // Lighten by 0% should return the same color
        let expr = runtime.compile("lighten('#ff5500', `0`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ff5500");

        // Lighten by 100% should give white
        let expr = runtime.compile("lighten('#ff5500', `100`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ffffff");
    }

    #[test]
    fn test_lighten_invalid_hex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("lighten('notahex', `50`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_darken() {
        let runtime = setup_runtime();
        // Darken white by 50% should give #808080
        let expr = runtime.compile("darken('#ffffff', `50`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#808080");

        // Darken by 0% should return the same color
        let expr = runtime.compile("darken('#ff5500', `0`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ff5500");

        // Darken by 100% should give black
        let expr = runtime.compile("darken('#ff5500', `100`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#000000");
    }

    #[test]
    fn test_color_mix_default_weight() {
        let runtime = setup_runtime();
        // Mix black and white with default 50% weight
        let expr = runtime.compile("color_mix('#000000', '#ffffff')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#808080");
    }

    #[test]
    fn test_color_mix_custom_weight() {
        let runtime = setup_runtime();
        // Weight 0.0 = all first color
        let expr = runtime
            .compile("color_mix('#ff0000', '#0000ff', `0`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ff0000");

        // Weight 1.0 = all second color
        let expr = runtime
            .compile("color_mix('#ff0000', '#0000ff', `1`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#0000ff");
    }

    #[test]
    fn test_color_mix_invalid_hex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("color_mix('xyz', '#ffffff')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_color_invert() {
        let runtime = setup_runtime();
        let expr = runtime.compile("color_invert('#ff0000')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#00ffff");

        // Inverting white gives black
        let expr = runtime.compile("color_invert('#ffffff')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#000000");
    }

    #[test]
    fn test_color_grayscale() {
        let runtime = setup_runtime();
        // Pure red: 0.299*255 + 0.587*0 + 0.114*0 = 76.245 -> 76 = #4c4c4c
        let expr = runtime.compile("color_grayscale('#ff0000')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#4c4c4c");

        // White stays white
        let expr = runtime.compile("color_grayscale('#ffffff')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#ffffff");
    }

    #[test]
    fn test_color_complement() {
        let runtime = setup_runtime();
        // Complement of red (#ff0000) should be cyan (#00ffff)
        let expr = runtime.compile("color_complement('#ff0000')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "#00ffff");
    }

    #[test]
    fn test_color_complement_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("color_complement('xyz')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.is_null());
    }
}
