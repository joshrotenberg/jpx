# Geo Functions (10)

## `geo_bearing`

**Signature:** `number, number, number, number -> number`

Bearing between coordinates

```
geo_bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 51.2
```
_NYC to London_

```
geo_bearing(`0`, `0`, `0`, `90`) -> 90.0
```
_Due east_

---

## `geo_bounding_box`

**Signature:** `array -> object`

Bounding box from an array of [lat, lon] points

```
geo_bounding_box(`[[40.7, -74.0], [34.0, -118.2]]`) -> {"max_lat": 40.7, "max_lon": -74.0, "min_lat": 34.0, "min_lon": -118.2}
```
_NYC and LA bounding box_

---

## `geo_distance`

**Signature:** `number, number, number, number -> number`

Haversine distance in meters

```
geo_distance(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570222
```
_NYC to London_

```
geo_distance(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 559044
```
_LA to SF_

---

## `geo_distance_km`

**Signature:** `number, number, number, number -> number`

Haversine distance in kilometers

```
geo_distance_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570.2
```
_NYC to London_

```
geo_distance_km(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 559.0
```
_LA to SF_

---

## `geo_distance_miles`

**Signature:** `number, number, number, number -> number`

Haversine distance in miles

```
geo_distance_miles(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 3461.0
```
_NYC to London_

```
geo_distance_miles(`34.0522`, `-118.2437`, `37.7749`, `-122.4194`) -> 347.4
```
_LA to SF_

---

## `geo_in_bbox`

**Signature:** `number, number, number, number, number, number -> boolean`

Check if a point is inside a bounding box

```
geo_in_bbox(`40.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`) -> true
```
_Point inside box_

```
geo_in_bbox(`42.0`, `-75.0`, `39.0`, `41.0`, `-76.0`, `-74.0`) -> false
```
_Point outside box_

---

## `geo_in_radius`

**Signature:** `number, number, number, number, number -> boolean`

Check if a point is within a radius (km) of a center point

```
geo_in_radius(`40.758`, `-73.985`, `40.748`, `-73.986`, `2`) -> true
```
_Times Square within 2km of Empire State_

```
geo_in_radius(`34.052`, `-118.244`, `40.713`, `-74.006`, `100`) -> false
```
_LA not within 100km of NYC_

---

## `geo_midpoint`

**Signature:** `array -> array`

Geographic midpoint of an array of [lat, lon] points

```
geo_midpoint(`[[0, 0], [0, 90]]`) -> [0.0, 45.0]
```
_Midpoint on equator_

---

## `geohash_decode`

**Signature:** `string -> object`

Decode a geohash string to {lat, lon}

```
geohash_decode('dr5ru7') -> {"lat": 40.71, "lon": -73.99}
```
_Decode NYC geohash_

---

## `geohash_encode`

**Signature:** `number, number[, number] -> string`

Encode lat/lon as a geohash string with optional precision (default 12)

```
geohash_encode(`40.7128`, `-74.0060`) -> "dr5ru6j1yz56"
```
_Encode NYC coordinates_

```
geohash_encode(`40.7128`, `-74.0060`, `5`) -> "dr5ru"
```
_Encode with precision 5_

---

