//! Geographic/geospatial functions.

use std::collections::HashSet;

use geohash::Coord;
use geoutils::Location;
use serde_json::{Value, json};

use crate::functions::{Function, custom_error, number_value};
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

// =============================================================================
// geo_bounding_box(array_of_points) -> {min_lat, max_lat, min_lon, max_lon}
// =============================================================================

defn!(GeoBoundingBoxFn, vec![arg!(array)], None);

impl Function for GeoBoundingBoxFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let points = args[0].as_array().unwrap();

        if points.is_empty() {
            return Err(custom_error(ctx, "geo_bounding_box requires a non-empty array of [lat, lon] points"));
        }

        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;
        let mut min_lon = f64::MAX;
        let mut max_lon = f64::MIN;

        for (i, point) in points.iter().enumerate() {
            let arr = point.as_array().ok_or_else(|| {
                custom_error(ctx, &format!("geo_bounding_box: element {i} is not an array"))
            })?;
            if arr.len() < 2 {
                return Err(custom_error(ctx, &format!("geo_bounding_box: element {i} must have at least 2 elements [lat, lon]")));
            }
            let lat = arr[0].as_f64().ok_or_else(|| {
                custom_error(ctx, &format!("geo_bounding_box: element {i} lat is not a number"))
            })?;
            let lon = arr[1].as_f64().ok_or_else(|| {
                custom_error(ctx, &format!("geo_bounding_box: element {i} lon is not a number"))
            })?;

            if lat < min_lat { min_lat = lat; }
            if lat > max_lat { max_lat = lat; }
            if lon < min_lon { min_lon = lon; }
            if lon > max_lon { max_lon = lon; }
        }

        Ok(json!({
            "min_lat": min_lat,
            "max_lat": max_lat,
            "min_lon": min_lon,
            "max_lon": max_lon,
        }))
    }
}

// =============================================================================
// geo_in_bbox(lat, lon, min_lat, max_lat, min_lon, max_lon) -> boolean
// =============================================================================

defn!(
    GeoInBboxFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoInBboxFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat = args[0].as_f64().unwrap();
        let lon = args[1].as_f64().unwrap();
        let min_lat = args[2].as_f64().unwrap();
        let max_lat = args[3].as_f64().unwrap();
        let min_lon = args[4].as_f64().unwrap();
        let max_lon = args[5].as_f64().unwrap();

        let inside = lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon;
        Ok(Value::Bool(inside))
    }
}

// =============================================================================
// geo_in_radius(lat, lon, center_lat, center_lon, radius_km) -> boolean
// =============================================================================

defn!(
    GeoInRadiusFn,
    vec![arg!(number), arg!(number), arg!(number), arg!(number), arg!(number)],
    None
);

impl Function for GeoInRadiusFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat = args[0].as_f64().unwrap();
        let lon = args[1].as_f64().unwrap();
        let center_lat = args[2].as_f64().unwrap();
        let center_lon = args[3].as_f64().unwrap();
        let radius_km = args[4].as_f64().unwrap();

        let point = Location::new(lat, lon);
        let center = Location::new(center_lat, center_lon);
        let distance_m = point.haversine_distance_to(&center).meters();

        Ok(Value::Bool(distance_m <= radius_km * 1000.0))
    }
}

// =============================================================================
// geo_midpoint(array_of_points) -> [lat, lon]
// =============================================================================

defn!(GeoMidpointFn, vec![arg!(array)], None);

impl Function for GeoMidpointFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let points = args[0].as_array().unwrap();

        if points.is_empty() {
            return Err(custom_error(ctx, "geo_midpoint requires a non-empty array of [lat, lon] points"));
        }

        // Cartesian averaging on unit sphere (accurate for large distances)
        let mut x = 0.0_f64;
        let mut y = 0.0_f64;
        let mut z = 0.0_f64;

        for (i, point) in points.iter().enumerate() {
            let arr = point.as_array().ok_or_else(|| {
                custom_error(ctx, &format!("geo_midpoint: element {i} is not an array"))
            })?;
            if arr.len() < 2 {
                return Err(custom_error(ctx, &format!("geo_midpoint: element {i} must have at least 2 elements [lat, lon]")));
            }
            let lat = arr[0].as_f64().ok_or_else(|| {
                custom_error(ctx, &format!("geo_midpoint: element {i} lat is not a number"))
            })?;
            let lon = arr[1].as_f64().ok_or_else(|| {
                custom_error(ctx, &format!("geo_midpoint: element {i} lon is not a number"))
            })?;

            let lat_rad = lat.to_radians();
            let lon_rad = lon.to_radians();

            x += lat_rad.cos() * lon_rad.cos();
            y += lat_rad.cos() * lon_rad.sin();
            z += lat_rad.sin();
        }

        let n = points.len() as f64;
        x /= n;
        y /= n;
        z /= n;

        let lon_mid = y.atan2(x).to_degrees();
        let hyp = (x * x + y * y).sqrt();
        let lat_mid = z.atan2(hyp).to_degrees();

        Ok(json!([lat_mid, lon_mid]))
    }
}

// =============================================================================
// geohash_encode(lat, lon, precision?) -> string
// =============================================================================

defn!(
    GeohashEncodeFn,
    vec![arg!(number), arg!(number)],
    Some(arg!(number))
);

impl Function for GeohashEncodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let lat = args[0].as_f64().unwrap();
        let lon = args[1].as_f64().unwrap();
        let precision = if args.len() > 2 {
            args[2].as_f64().unwrap() as usize
        } else {
            12
        };

        let hash = geohash::encode(Coord { x: lon, y: lat }, precision)
            .map_err(|e| custom_error(ctx, &format!("geohash_encode: {e}")))?;
        Ok(Value::String(hash))
    }
}

// =============================================================================
// geohash_decode(hash) -> {lat, lon}
// =============================================================================

defn!(GeohashDecodeFn, vec![arg!(string)], None);

impl Function for GeohashDecodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let hash = args[0].as_str().unwrap();

        let (coord, _, _) = geohash::decode(hash)
            .map_err(|e| custom_error(ctx, &format!("geohash_decode: {e}")))?;

        Ok(json!({
            "lat": coord.y,
            "lon": coord.x,
        }))
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
    register_if_enabled(
        runtime,
        "geo_bounding_box",
        enabled,
        Box::new(GeoBoundingBoxFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_in_bbox",
        enabled,
        Box::new(GeoInBboxFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_in_radius",
        enabled,
        Box::new(GeoInRadiusFn::new()),
    );
    register_if_enabled(
        runtime,
        "geo_midpoint",
        enabled,
        Box::new(GeoMidpointFn::new()),
    );
    register_if_enabled(
        runtime,
        "geohash_encode",
        enabled,
        Box::new(GeohashEncodeFn::new()),
    );
    register_if_enabled(
        runtime,
        "geohash_decode",
        enabled,
        Box::new(GeohashDecodeFn::new()),
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

    #[test]
    fn test_geo_midpoint_two_points() {
        let runtime = setup_runtime();
        // Midpoint of equator points (0,0) and (0,90) should be roughly (0, 45)
        let data = json!([[0, 0], [0, 90]]);
        let expr = runtime.compile("geo_midpoint(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let lat = arr[0].as_f64().unwrap();
        let lon = arr[1].as_f64().unwrap();
        assert!(lat.abs() < 0.01, "lat should be ~0, got {lat}");
        assert!((lon - 45.0).abs() < 0.01, "lon should be ~45, got {lon}");
    }

    #[test]
    fn test_geo_midpoint_same_point() {
        let runtime = setup_runtime();
        let data = json!([[40.7128, -74.0060], [40.7128, -74.0060]]);
        let expr = runtime.compile("geo_midpoint(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let lat = arr[0].as_f64().unwrap();
        let lon = arr[1].as_f64().unwrap();
        assert!((lat - 40.7128).abs() < 0.001);
        assert!((lon - (-74.0060)).abs() < 0.001);
    }

    #[test]
    fn test_geo_bounding_box() {
        let runtime = setup_runtime();
        let data = json!([[40.7128, -74.0060], [34.0522, -118.2437], [37.7749, -122.4194]]);
        let expr = runtime.compile("geo_bounding_box(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!((obj["min_lat"].as_f64().unwrap() - 34.0522).abs() < 0.001);
        assert!((obj["max_lat"].as_f64().unwrap() - 40.7128).abs() < 0.001);
        assert!((obj["min_lon"].as_f64().unwrap() - (-122.4194)).abs() < 0.001);
        assert!((obj["max_lon"].as_f64().unwrap() - (-74.0060)).abs() < 0.001);
    }

    #[test]
    fn test_geo_bounding_box_single_point() {
        let runtime = setup_runtime();
        let data = json!([[40.7128, -74.0060]]);
        let expr = runtime.compile("geo_bounding_box(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!((obj["min_lat"].as_f64().unwrap() - 40.7128).abs() < 0.001);
        assert!((obj["max_lat"].as_f64().unwrap() - 40.7128).abs() < 0.001);
    }

    #[test]
    fn test_geo_in_radius_inside() {
        let runtime = setup_runtime();
        // Times Square is ~1.3 km from Empire State Building
        let data = json!(null);
        let expr = runtime
            .compile("geo_in_radius(`40.7580`, `-73.9855`, `40.7484`, `-73.9857`, `2`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_geo_in_radius_outside() {
        let runtime = setup_runtime();
        // NYC to LA is ~3940 km, so 100 km radius won't contain it
        let data = json!(null);
        let expr = runtime
            .compile("geo_in_radius(`34.0522`, `-118.2437`, `40.7128`, `-74.0060`, `100`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_geo_in_bbox_inside() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("geo_in_bbox(`40.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_geo_in_bbox_outside() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("geo_in_bbox(`42.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(false));
    }

    #[test]
    fn test_geo_in_bbox_boundary() {
        let runtime = setup_runtime();
        let data = json!(null);
        // Exactly on the boundary should be inside (inclusive)
        let expr = runtime
            .compile("geo_in_bbox(`39.0`, `-76.0`, `39.0`, `41.0`, `-76.0`, `-74.0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn test_geohash_encode_default_precision() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("geohash_encode(`40.7128`, `-74.0060`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let hash = result.as_str().unwrap();
        assert_eq!(hash.len(), 12);
        assert!(hash.starts_with("dr5r"));
    }

    #[test]
    fn test_geohash_encode_custom_precision() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("geohash_encode(`40.7128`, `-74.0060`, `5`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let hash = result.as_str().unwrap();
        assert_eq!(hash.len(), 5);
    }

    #[test]
    fn test_geohash_decode() {
        let runtime = setup_runtime();
        let data = json!(null);
        let expr = runtime
            .compile("geohash_decode('dr5ru7')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let lat = obj["lat"].as_f64().unwrap();
        let lon = obj["lon"].as_f64().unwrap();
        // dr5ru7 is in the NYC area
        assert!(lat > 40.0 && lat < 41.0, "lat should be ~40.7, got {lat}");
        assert!(lon > -75.0 && lon < -73.0, "lon should be ~-74, got {lon}");
    }

    #[test]
    fn test_geohash_roundtrip() {
        let runtime = setup_runtime();
        let data = json!(null);
        // Encode then decode should give back approximately the same point
        let encode_expr = runtime
            .compile("geohash_encode(`48.8566`, `2.3522`, `8`)")
            .unwrap();
        let hash_val = encode_expr.search(&data).unwrap();
        let hash = hash_val.as_str().unwrap();

        let decode_expr_str = format!("geohash_decode('{hash}')");
        let decode_expr = runtime.compile(&decode_expr_str).unwrap();
        let result = decode_expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let lat = obj["lat"].as_f64().unwrap();
        let lon = obj["lon"].as_f64().unwrap();
        assert!((lat - 48.8566).abs() < 0.01, "lat roundtrip: expected ~48.8566, got {lat}");
        assert!((lon - 2.3522).abs() < 0.01, "lon roundtrip: expected ~2.3522, got {lon}");
    }
}
