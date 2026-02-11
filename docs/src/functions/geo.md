# Geolocation Functions

Geolocation functions: distance calculation, coordinate parsing, and geographic utilities.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`geo_bearing`](#geo-bearing) | `number, number, number, number -> number` | Bearing between coordinates |
| [`geo_bounding_box`](#geo-bounding-box) | `array -> object` | Bounding box from an array of points |
| [`geo_distance`](#geo-distance) | `number, number, number, number -> number` | Haversine distance in meters |
| [`geo_distance_km`](#geo-distance-km) | `number, number, number, number -> number` | Haversine distance in kilometers |
| [`geo_distance_miles`](#geo-distance-miles) | `number, number, number, number -> number` | Haversine distance in miles |
| [`geo_in_bbox`](#geo-in-bbox) | `number, number, number, number, number, number -> boolean` | Check if point is inside bounding box |
| [`geo_in_radius`](#geo-in-radius) | `number, number, number, number, number -> boolean` | Check if point is within radius (km) |
| [`geo_midpoint`](#geo-midpoint) | `array -> array` | Geographic midpoint of points |
| [`geohash_decode`](#geohash-decode) | `string -> object` | Decode geohash to {lat, lon} |
| [`geohash_encode`](#geohash-encode) | `number, number[, number] -> string` | Encode lat/lon as geohash |

## Functions

### geo_bearing

Bearing between coordinates

**Signature:** `number, number, number, number -> number`

**Examples:**

```text
# NYC to London
geo_bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 51.2
# Due east
geo_bearing(`0`, `0`, `0`, `90`) -> 90.0
# Due north
geo_bearing(`0`, `0`, `90`, `0`) -> 0.0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)'
```

### geo_bounding_box

Bounding box from an array of [lat, lon] points

**Signature:** `array -> object`

**Examples:**

```text
# NYC and LA
geo_bounding_box(`[[40.7128, -74.0060], [34.0522, -118.2437]]`)
  -> {"max_lat": 40.7128, "max_lon": -74.006, "min_lat": 34.0522, "min_lon": -118.2437}
# Three cities
geo_bounding_box(`[[40.7128, -74.0060], [34.0522, -118.2437], [37.7749, -122.4194]]`)
  -> {"max_lat": 40.7128, "max_lon": -74.006, "min_lat": 34.0522, "min_lon": -122.4194}
```

**CLI Usage:**

```bash
echo '[[40.7128, -74.0060], [34.0522, -118.2437]]' | jpx 'geo_bounding_box(@)'
```

### geo_distance

Haversine distance in meters

**Signature:** `number, number, number, number -> number`

**Examples:**

```text
# NYC to London
geo_distance(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570222
# LA to SF
geo_distance(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 559044
# Same point
geo_distance(`0`, `0`, `0`, `0`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_distance(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)'
```

### geo_distance_km

Haversine distance in kilometers

**Signature:** `number, number, number, number -> number`

**Examples:**

```text
# NYC to London
geo_distance_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570.2
# LA to SF
geo_distance_km(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 559.0
# Paris to London
geo_distance_km(`48.8566`, `2.3522`, `51.5074`, `-0.1278`) -> 343.5
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_distance_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)'
```

### geo_distance_miles

Haversine distance in miles

**Signature:** `number, number, number, number -> number`

**Examples:**

```text
# NYC to London
geo_distance_miles(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 3461.0
# LA to SF
geo_distance_miles(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 347.4
# Paris to London
geo_distance_miles(`48.8566`, `2.3522`, `51.5074`, `-0.1278`) -> 213.4
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_distance_miles(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)'
```

### geo_in_bbox

Check if a point is inside a bounding box

**Signature:** `number, number, number, number, number, number -> boolean`

Arguments: `lat, lon, min_lat, max_lat, min_lon, max_lon`

**Examples:**

```text
# Point inside box
geo_in_bbox(`40.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`) -> true
# Point outside box
geo_in_bbox(`42.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_in_bbox(`40.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`)'
```

### geo_in_radius

Check if a point is within a radius (km) of a center point

**Signature:** `number, number, number, number, number -> boolean`

Arguments: `lat, lon, center_lat, center_lon, radius_km`

**Examples:**

```text
# Times Square within 2km of Empire State Building
geo_in_radius(`40.758`, `-73.985`, `40.748`, `-73.986`, `2`) -> true
# LA not within 100km of NYC
geo_in_radius(`34.052`, `-118.244`, `40.713`, `-74.006`, `100`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geo_in_radius(`40.758`, `-73.985`, `40.748`, `-73.986`, `2`)'
```

### geo_midpoint

Geographic midpoint of an array of [lat, lon] points using Cartesian averaging on the unit sphere (accurate for large distances)

**Signature:** `array -> array`

**Examples:**

```text
# Midpoint on equator
geo_midpoint(`[[0, 0], [0, 90]]`) -> [0.0, 45.0]
# Same point
geo_midpoint(`[[40.7128, -74.0060], [40.7128, -74.0060]]`) -> [40.7128, -74.006]
```

**CLI Usage:**

```bash
echo '[[40.7128, -74.0060], [34.0522, -118.2437]]' | jpx 'geo_midpoint(@)'
```

### geohash_decode

Decode a geohash string to a `{lat, lon}` object

**Signature:** `string -> object`

**Examples:**

```text
# Decode NYC area geohash
geohash_decode('dr5ru7') -> {"lat": 40.71, "lon": -73.99}
# Decode Paris area geohash
geohash_decode('u09tvw') -> {"lat": 48.86, "lon": 2.35}
```

**CLI Usage:**

```bash
echo '"dr5ru7"' | jpx 'geohash_decode(@)'
```

### geohash_encode

Encode latitude/longitude as a geohash string with optional precision (default 12)

**Signature:** `number, number[, number] -> string`

Arguments: `lat, lon[, precision]`

**Examples:**

```text
# Encode NYC with default precision
geohash_encode(`40.7128`, `-74.0060`) -> "dr5ru6j1yz56"
# Encode with precision 5
geohash_encode(`40.7128`, `-74.0060`, `5`) -> "dr5ru"
# Encode Paris
geohash_encode(`48.8566`, `2.3522`, `6`) -> "u09tvw"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'geohash_encode(`40.7128`, `-74.0060`)'
echo '{}' | jpx 'geohash_encode(`40.7128`, `-74.0060`, `5`)'
```

