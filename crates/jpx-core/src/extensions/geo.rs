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
