//! Unit conversion functions for temperature, length, mass, and volume.

use std::collections::HashSet;

use serde_json::Value;

use crate::functions::{Function, custom_error, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register unit conversion functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "convert_temperature",
        enabled,
        Box::new(ConvertTemperatureFn::new()),
    );
    register_if_enabled(
        runtime,
        "convert_length",
        enabled,
        Box::new(ConvertLengthFn::new()),
    );
    register_if_enabled(
        runtime,
        "convert_mass",
        enabled,
        Box::new(ConvertMassFn::new()),
    );
    register_if_enabled(
        runtime,
        "convert_volume",
        enabled,
        Box::new(ConvertVolumeFn::new()),
    );
}

// =============================================================================
// convert_temperature(value, from_unit, to_unit) -> number
// =============================================================================

defn!(
    ConvertTemperatureFn,
    vec![arg!(number), arg!(string), arg!(string)],
    None
);

fn normalize_temp_unit(unit: &str) -> Option<&'static str> {
    match unit.to_lowercase().as_str() {
        "c" | "celsius" => Some("C"),
        "f" | "fahrenheit" => Some("F"),
        "k" | "kelvin" => Some("K"),
        _ => None,
    }
}

fn to_celsius(value: f64, from: &str) -> f64 {
    match from {
        "C" => value,
        "F" => (value - 32.0) * 5.0 / 9.0,
        "K" => value - 273.15,
        _ => unreachable!(),
    }
}

fn from_celsius(value: f64, to: &str) -> f64 {
    match to {
        "C" => value,
        "F" => value * 9.0 / 5.0 + 32.0,
        "K" => value + 273.15,
        _ => unreachable!(),
    }
}

impl Function for ConvertTemperatureFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_f64().unwrap();
        let from_str = args[1].as_str().unwrap();
        let to_str = args[2].as_str().unwrap();

        let from = normalize_temp_unit(from_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown temperature unit: {}", from_str)))?;
        let to = normalize_temp_unit(to_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown temperature unit: {}", to_str)))?;

        let celsius = to_celsius(value, from);
        let result = from_celsius(celsius, to);

        Ok(number_value(result))
    }
}

// =============================================================================
// convert_length(value, from_unit, to_unit) -> number
// =============================================================================

defn!(
    ConvertLengthFn,
    vec![arg!(number), arg!(string), arg!(string)],
    None
);

/// Returns the factor to convert from the given unit to meters.
fn length_to_meters(unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "m" | "meters" => Some(1.0),
        "km" | "kilometers" => Some(1000.0),
        "cm" | "centimeters" => Some(0.01),
        "mm" | "millimeters" => Some(0.001),
        "mi" | "miles" => Some(1609.344),
        "ft" | "feet" => Some(0.3048),
        "in" | "inches" => Some(0.0254),
        "yd" | "yards" => Some(0.9144),
        "nmi" | "nautical_miles" => Some(1852.0),
        _ => None,
    }
}

impl Function for ConvertLengthFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_f64().unwrap();
        let from_str = args[1].as_str().unwrap();
        let to_str = args[2].as_str().unwrap();

        let from_factor = length_to_meters(from_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown length unit: {}", from_str)))?;
        let to_factor = length_to_meters(to_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown length unit: {}", to_str)))?;

        let meters = value * from_factor;
        let result = meters / to_factor;

        Ok(number_value(result))
    }
}

// =============================================================================
// convert_mass(value, from_unit, to_unit) -> number
// =============================================================================

defn!(
    ConvertMassFn,
    vec![arg!(number), arg!(string), arg!(string)],
    None
);

/// Returns the factor to convert from the given unit to kilograms.
fn mass_to_kg(unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "kg" | "kilograms" => Some(1.0),
        "g" | "grams" => Some(0.001),
        "mg" | "milligrams" => Some(0.000001),
        "lbs" | "pounds" => Some(0.45359237),
        "oz" | "ounces" => Some(0.028349523125),
        "t" | "tonnes" => Some(1000.0),
        "st" | "stones" => Some(6.35029318),
        _ => None,
    }
}

impl Function for ConvertMassFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_f64().unwrap();
        let from_str = args[1].as_str().unwrap();
        let to_str = args[2].as_str().unwrap();

        let from_factor = mass_to_kg(from_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown mass unit: {}", from_str)))?;
        let to_factor = mass_to_kg(to_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown mass unit: {}", to_str)))?;

        let kg = value * from_factor;
        let result = kg / to_factor;

        Ok(number_value(result))
    }
}

// =============================================================================
// convert_volume(value, from_unit, to_unit) -> number
// =============================================================================

defn!(
    ConvertVolumeFn,
    vec![arg!(number), arg!(string), arg!(string)],
    None
);

/// Returns the factor to convert from the given unit to liters.
fn volume_to_liters(unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "l" | "liters" => Some(1.0),
        "ml" | "milliliters" => Some(0.001),
        "gal" | "gallons" => Some(3.785411784),
        "qt" | "quarts" => Some(0.946352946),
        "pt" | "pints" => Some(0.473176473),
        "cup" | "cups" => Some(0.2365882365),
        "floz" | "fluid_ounces" => Some(0.0295735295625),
        "tbsp" | "tablespoons" => Some(0.0147867647813),
        "tsp" | "teaspoons" => Some(0.00492892159375),
        _ => None,
    }
}

impl Function for ConvertVolumeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let value = args[0].as_f64().unwrap();
        let from_str = args[1].as_str().unwrap();
        let to_str = args[2].as_str().unwrap();

        let from_factor = volume_to_liters(from_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown volume unit: {}", from_str)))?;
        let to_factor = volume_to_liters(to_str)
            .ok_or_else(|| custom_error(ctx, &format!("unknown volume unit: {}", to_str)))?;

        let liters = value * from_factor;
        let result = liters / to_factor;

        Ok(number_value(result))
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
    // Temperature tests
    // =========================================================================

    #[test]
    fn test_celsius_to_fahrenheit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`100`, 'C', 'F')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 212.0).abs() < 1e-9);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`32`, 'F', 'C')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_celsius_to_kelvin() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`0`, 'C', 'K')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 273.15).abs() < 1e-9);
    }

    #[test]
    fn test_kelvin_to_celsius() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`273.15`, 'K', 'C')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_fahrenheit_to_kelvin() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`212`, 'fahrenheit', 'kelvin')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 373.15).abs() < 1e-9);
    }

    #[test]
    fn test_temperature_same_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`42`, 'C', 'celsius')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_temperature_invalid_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_temperature(`100`, 'C', 'invalid')")
            .unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    // =========================================================================
    // Length tests
    // =========================================================================

    #[test]
    fn test_km_to_miles() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'km', 'mi')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.621371192237334).abs() < 1e-6);
    }

    #[test]
    fn test_miles_to_km() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'mi', 'km')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 1.609344).abs() < 1e-6);
    }

    #[test]
    fn test_feet_to_meters() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'ft', 'm')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.3048).abs() < 1e-6);
    }

    #[test]
    fn test_inches_to_cm() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'in', 'cm')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 2.54).abs() < 1e-6);
    }

    #[test]
    fn test_yards_to_meters() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'yd', 'm')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.9144).abs() < 1e-6);
    }

    #[test]
    fn test_nautical_miles_to_km() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_length(`1`, 'nmi', 'km')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 1.852).abs() < 1e-6);
    }

    #[test]
    fn test_length_same_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_length(`42`, 'meters', 'm')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_length_invalid_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_length(`1`, 'm', 'furlongs')")
            .unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    // =========================================================================
    // Mass tests
    // =========================================================================

    #[test]
    fn test_kg_to_lbs() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_mass(`1`, 'kg', 'lbs')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 2.20462262185).abs() < 1e-4);
    }

    #[test]
    fn test_lbs_to_kg() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_mass(`1`, 'lbs', 'kg')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 0.45359237).abs() < 1e-6);
    }

    #[test]
    fn test_grams_to_ounces() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_mass(`1000`, 'g', 'oz')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        // 1000g = 1kg = ~35.274 oz
        assert!((result.as_f64().unwrap() - 35.27396195).abs() < 1e-3);
    }

    #[test]
    fn test_tonnes_to_kg() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_mass(`1`, 't', 'kg')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn test_stones_to_lbs() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_mass(`1`, 'st', 'lbs')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 14.0).abs() < 0.01);
    }

    #[test]
    fn test_mass_same_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_mass(`42`, 'kilograms', 'kg')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_mass_invalid_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_mass(`1`, 'kg', 'bushels')")
            .unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    // =========================================================================
    // Volume tests
    // =========================================================================

    #[test]
    fn test_gallons_to_liters() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_volume(`1`, 'gal', 'l')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 3.785411784).abs() < 1e-6);
    }

    #[test]
    fn test_liters_to_ml() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_volume(`1`, 'l', 'ml')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_cups_to_ml() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_volume(`1`, 'cup', 'ml')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 236.5882365).abs() < 0.01);
    }

    #[test]
    fn test_tbsp_to_tsp() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_volume(`1`, 'tbsp', 'tsp')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_quarts_to_pints() {
        let runtime = setup_runtime();
        let expr = runtime.compile("convert_volume(`1`, 'qt', 'pt')").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_floz_to_ml() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_volume(`1`, 'floz', 'ml')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 29.5735295625).abs() < 0.01);
    }

    #[test]
    fn test_volume_same_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_volume(`42`, 'liters', 'l')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!((result.as_f64().unwrap() - 42.0).abs() < 1e-9);
    }

    #[test]
    fn test_volume_invalid_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("convert_volume(`1`, 'l', 'barrels')")
            .unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    // =========================================================================
    // Data-driven tests
    // =========================================================================

    #[test]
    fn test_convert_from_json_data() {
        let runtime = setup_runtime();
        let data = json!({"temp": 100, "from": "C", "to": "F"});
        let expr = runtime
            .compile("convert_temperature(temp, from, to)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!((result.as_f64().unwrap() - 212.0).abs() < 1e-9);
    }
}
