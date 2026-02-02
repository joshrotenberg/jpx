# Geolocation Functions

Geolocation functions: distance calculation, coordinate parsing, and geographic utilities.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`geo_bearing`](#geo-bearing) | `number, number, number, number -> number` | Bearing between coordinates |
| [`geo_distance`](#geo-distance) | `number, number, number, number -> number` | Haversine distance in meters |
| [`geo_distance_km`](#geo-distance-km) | `number, number, number, number -> number` | Haversine distance in kilometers |
| [`geo_distance_miles`](#geo-distance-miles) | `number, number, number, number -> number` | Haversine distance in miles |

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

