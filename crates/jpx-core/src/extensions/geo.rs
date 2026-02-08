//! Geographic/geospatial functions.

use std::collections::HashSet;

use geoutils::Location;
use serde_json::Value;

use crate::functions::{Function, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

// =============================================================================
// geo_distance(lat1, lon1, lat2, lon2) -> number (meters)
// =============================================================================

defn!(
    GeoDistanceFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoDistanceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_f64().unwrap();
        let lon1 = args[1].as_f64().unwrap();
        let lat2 = args[2].as_f64().unwrap();
        let lon2 = args[3].as_f64().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(number_value(distance.meters()))
    }
}

// =============================================================================
// geo_distance_km(lat1, lon1, lat2, lon2) -> number (kilometers)
// =============================================================================

defn!(
    GeoDistanceKmFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoDistanceKmFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_f64().unwrap();
        let lon1 = args[1].as_f64().unwrap();
        let lat2 = args[2].as_f64().unwrap();
        let lon2 = args[3].as_f64().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(number_value(distance.meters() / 1000.0))
    }
}

// =============================================================================
// geo_distance_miles(lat1, lon1, lat2, lon2) -> number (miles)
// =============================================================================

defn!(
    GeoDistanceMilesFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoDistanceMilesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_f64().unwrap();
        let lon1 = args[1].as_f64().unwrap();
        let lat2 = args[2].as_f64().unwrap();
        let lon2 = args[3].as_f64().unwrap();

        let loc1 = Location::new(lat1, lon1);
        let loc2 = Location::new(lat2, lon2);

        // 1 meter = 0.000621371 miles
        const METERS_TO_MILES: f64 = 0.000621371;

        let distance = loc1.haversine_distance_to(&loc2);
        Ok(number_value(distance.meters() * METERS_TO_MILES))
    }
}

// =============================================================================
// geo_bearing(lat1, lon1, lat2, lon2) -> number (degrees 0-360)
// =============================================================================

defn!(
    GeoBearingFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoBearingFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat1 = args[0].as_f64().unwrap();
        let lon1 = args[1].as_f64().unwrap();
        let lat2 = args[2].as_f64().unwrap();
        let lon2 = args[3].as_f64().unwrap();

        // Calculate initial bearing using the forward azimuth formula
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let x = delta_lon.sin() * lat2_rad.cos();
        let y = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * delta_lon.cos();

        let bearing_rad = x.atan2(y);
        let mut bearing = bearing_rad.to_degrees();

        // Normalize to 0-360
        if bearing < 0.0 {
            bearing += 360.0;
        }

        Ok(number_value(bearing))
    }
}

/// Register geo functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(
        runtime,
        "geo_distance",
        enabled,
        Box::new(GeoDistanceFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_distance_km",
        enabled,
        Box::new(GeoDistanceKmFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_distance_miles",
        enabled,
        Box::new(GeoDistanceMilesFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_bearing",
        enabled,
        Box::new(GeoBearingFn::new()),
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
    fn test_geo_distance() {
        let runtime = setup_runtime();
        // NYC to LA: approximately 3940 km
        let data = json!({"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]});
        let expr = runtime
            .compile("geo_distance(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let meters = result.as_f64().unwrap();
        // Should be approximately 3940000 meters
        assert!(meters > 3900000.0 && meters < 4000000.0);
    }

    #[test]
    fn test_geo_distance_km() {
        let runtime = setup_runtime();
        let data = json!({"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]});
        let expr = runtime
            .compile("geo_distance_km(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let km = result.as_f64().unwrap();
        // Should be approximately 3940 km
        assert!(km > 3900.0 && km < 4000.0);
    }

    #[test]
    fn test_geo_distance_miles() {
        let runtime = setup_runtime();
        let data = json!({"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]});
        let expr = runtime
            .compile("geo_distance_miles(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let miles = result.as_f64().unwrap();
        // Should be approximately 2450 miles
        assert!(miles > 2400.0 && miles < 2500.0);
    }

    #[test]
    fn test_geo_bearing() {
        let runtime = setup_runtime();
        // NYC to LA should be roughly west (270 degrees)
        let data = json!({"nyc": [40.7128, -74.0060], "la": [34.0522, -118.2437]});
        let expr = runtime
            .compile("geo_bearing(nyc[0], nyc[1], la[0], la[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let bearing = result.as_f64().unwrap();
        // Should be roughly 273 degrees (west-southwest)
        assert!(bearing > 260.0 && bearing < 290.0);
    }

    #[test]
    fn test_geo_distance_same_point() {
        let runtime = setup_runtime();
        let data = json!([40.7128, -74.0060]);
        let expr = runtime
            .compile("geo_distance(@[0], @[1], @[0], @[1])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let meters = result.as_f64().unwrap();
        assert!(meters < 1.0); // Should be essentially 0
    }
}
