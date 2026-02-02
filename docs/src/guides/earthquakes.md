# USGS Earthquake Data

The [USGS Earthquake Hazards Program](https://earthquake.usgs.gov/) provides real-time earthquake data via a public API. This dataset is excellent for learning jpx because it has:

- Nested GeoJSON structure
- Numeric data for statistical analysis (magnitude, depth)
- Geographic coordinates for geo functions
- Timestamps for date/time operations

## Getting the Data

```bash
# Fetch recent magnitude 5+ earthquakes (10 events)
curl -s "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&limit=10&minmagnitude=5" > earthquakes.json

# Fetch more data to play with (100 events)
curl -s "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&limit=100&minmagnitude=4" > earthquakes_100.json
```

## Data Structure

The API returns GeoJSON with this structure:

```json
{
  "type": "FeatureCollection",
  "metadata": {
    "generated": 1768410727000,
    "title": "USGS Earthquakes",
    "count": 10
  },
  "features": [
    {
      "type": "Feature",
      "properties": {
        "mag": 6.2,
        "place": "133 km SE of Kuril'sk, Russia",
        "time": 1768289648725,
        "sig": 591,
        "tsunami": 0,
        "type": "earthquake",
        "title": "M 6.2 - 133 km SE of Kuril'sk, Russia"
      },
      "geometry": {
        "type": "Point",
        "coordinates": [149.2768, 44.5495, 35.788]
      },
      "id": "us7000rpcg"
    }
  ]
}
```

Key fields:
- `properties.mag` - Magnitude
- `properties.place` - Human-readable location
- `properties.time` - Unix timestamp (milliseconds)
- `properties.sig` - Significance (0-1000+)
- `geometry.coordinates` - [longitude, latitude, depth_km]

---

## Basic Queries

### List All Earthquake Titles

```bash
jpx 'features[*].properties.title' earthquakes.json
```

Output:
```json
[
  "M 5.1 - 119 km N of Lae, Papua New Guinea",
  "M 5.1 - 68 km E of Severo-Kuril'sk, Russia",
  "M 6.2 - 133 km SE of Kuril'sk, Russia",
  ...
]
```

### Get Magnitudes Only

```bash
jpx 'features[*].properties.mag' earthquakes.json
```

Output:
```json
[5.1, 5.1, 6.2, 5.3, 5.6, 5.2, 5.2, 5.1, 5.0, 5.6]
```

### Count Total Events

```bash
jpx 'length(features)' earthquakes.json
```

Output: `10`

---

## Filtering

### Find Major Earthquakes (Magnitude 6+)

```bash
jpx 'features[?properties.mag >= `6`].properties.title' earthquakes.json
```

Output:
```json
["M 6.2 - 133 km SE of Kuril'sk, Russia"]
```

### Find Deep Earthquakes (> 100km depth)

The depth is the third element in the coordinates array:

```bash
jpx 'features[?geometry.coordinates[2] > `100`].{
  title: properties.title,
  depth_km: geometry.coordinates[2]
}' earthquakes.json
```

Output:
```json
[
  {"title": "M 5.1 - 119 km N of Lae, Papua New Guinea", "depth_km": 172.746},
  {"title": "M 5.3 - 72 km W of Ollagüe, Chile", "depth_km": 118.039},
  {"title": "M 5.1 - 233 km ENE of Lospalos, Timor Leste", "depth_km": 101.686}
]
```

### Find Earthquakes with Tsunami Alerts

```bash
jpx 'features[?properties.tsunami == `1`].properties.title' earthquakes.json
```

### Filter by Location (Text Match)

```bash
jpx 'features[?contains(properties.place, `Russia`)].properties.title' earthquakes.json
```

Output:
```json
[
  "M 5.1 - 68 km E of Severo-Kuril'sk, Russia",
  "M 6.2 - 133 km SE of Kuril'sk, Russia",
  "M 5.2 - 177 km S of Severo-Kuril'sk, Russia",
  "M 5.6 - 265 km N of Kuril'sk, Russia"
]
```

---

## Statistics

### Average Magnitude

```bash
jpx 'avg(features[*].properties.mag)' earthquakes.json
```

Output: `5.34`

### Magnitude Statistics

```bash
jpx '{
  count: length(features),
  min: min(features[*].properties.mag),
  max: max(features[*].properties.mag),
  avg: avg(features[*].properties.mag),
  median: median(features[*].properties.mag),
  stddev: round(stddev(features[*].properties.mag), `3`)
}' earthquakes.json
```

Output:
```json
{
  "count": 10,
  "min": 5.0,
  "max": 6.2,
  "avg": 5.34,
  "median": 5.2,
  "stddev": 0.351
}
```

### Depth Statistics

```bash
jpx '{
  shallowest: min(features[*].geometry.coordinates[2]),
  deepest: max(features[*].geometry.coordinates[2]),
  avg_depth: round(avg(features[*].geometry.coordinates[2]), `1`)
}' earthquakes.json
```

---

## Sorting

### Sort by Magnitude (Descending)

```bash
jpx 'sort_by(features, &properties.mag) | reverse(@) | [*].{
  mag: properties.mag,
  place: properties.place
}' earthquakes.json
```

Output:
```json
[
  {"mag": 6.2, "place": "133 km SE of Kuril'sk, Russia"},
  {"mag": 5.6, "place": "Easter Island region"},
  {"mag": 5.6, "place": "265 km N of Kuril'sk, Russia"},
  ...
]
```

### Top 3 by Significance

```bash
jpx 'sort_by(features, &properties.sig) | reverse(@) | [:3] | [*].{
  significance: properties.sig,
  title: properties.title
}' earthquakes.json
```

---

## Geographic Calculations

### Extract Coordinates

```bash
jpx 'features[*].{
  place: properties.place,
  lat: geometry.coordinates[1],
  lon: geometry.coordinates[0]
}' earthquakes.json
```

### Distance from a Reference Point

Calculate distance from San Francisco (37.7749, -122.4194) using the `geo_distance_km` function:

```bash
jpx 'features[*].{
  place: properties.place,
  distance_km: round(geo_distance_km(
    `37.7749`, `-122.4194`,
    geometry.coordinates[1], geometry.coordinates[0]
  ), `0`)
} | sort_by(@, &distance_km)' earthquakes.json
```

Output:
```json
[
  {"place": "Easter Island region", "distance_km": 7580},
  {"place": "72 km W of Ollagüe, Chile", "distance_km": 9513},
  {"place": "South Sandwich Islands region", "distance_km": 12748},
  ...
]
```

### Find Nearest Earthquake

```bash
jpx 'min_by(features, &geo_distance_km(
  `37.7749`, `-122.4194`,
  geometry.coordinates[1], geometry.coordinates[0]
)).properties.title' earthquakes.json
```

---

## Date/Time Operations

### Convert Timestamps to Readable Dates

The `time` field is Unix milliseconds. Convert to ISO format:

```bash
jpx 'features[*].{
  title: properties.title,
  date: from_unixtime(floor(divide(properties.time, `1000`)))
}' earthquakes.json
```

### Find Earthquakes in Last 24 Hours

```bash
jpx --let now='now()' 'features[?properties.time > multiply(subtract($now, `86400`), `1000`)].properties.title' earthquakes.json
```

### Group by Day

```bash
jpx 'features[*].{
  day: format_datetime(from_unixtime(floor(divide(properties.time, `1000`))), `%Y-%m-%d`),
  mag: properties.mag,
  place: properties.place
}' earthquakes.json
```

---

## Data Transformation

### Reshape to Flat Structure

```bash
jpx 'features[*].{
  id: id,
  magnitude: properties.mag,
  location: properties.place,
  longitude: geometry.coordinates[0],
  latitude: geometry.coordinates[1],
  depth_km: geometry.coordinates[2],
  timestamp: properties.time,
  url: properties.url
}' earthquakes.json
```

### Export to CSV

```bash
jpx --csv 'features[*].{
  magnitude: properties.mag,
  location: properties.place,
  latitude: geometry.coordinates[1],
  longitude: geometry.coordinates[0],
  depth_km: geometry.coordinates[2]
}' earthquakes.json
```

Output:
```csv
magnitude,location,latitude,longitude,depth_km
5.1,"119 km N of Lae, Papua New Guinea",-5.6511,147.12,172.746
5.1,"68 km E of Severo-Kuril'sk, Russia",50.7928,157.0843,64.364
6.2,"133 km SE of Kuril'sk, Russia",44.5495,149.2768,35.788
...
```

---

## Advanced Pipelines

### Summary Report

Create a comprehensive earthquake summary:

```bash
jpx '{
  report_title: `USGS Earthquake Summary`,
  generated: metadata.title,
  total_events: length(features),
  magnitude_stats: {
    min: min(features[*].properties.mag),
    max: max(features[*].properties.mag),
    average: round(avg(features[*].properties.mag), `2`),
    median: median(features[*].properties.mag)
  },
  depth_stats: {
    shallowest_km: min(features[*].geometry.coordinates[2]),
    deepest_km: max(features[*].geometry.coordinates[2])
  },
  major_events: features[?properties.mag >= `5.5`] | [*].{
    magnitude: properties.mag,
    location: properties.place
  },
  by_region: {
    russia: length(features[?contains(properties.place, `Russia`)]),
    indonesia: length(features[?contains(properties.place, `Indonesia`)]),
    chile: length(features[?contains(properties.place, `Chile`)])
  }
}' earthquakes.json
```

### Pipeline with Multiple Steps

```bash
# Get major earthquakes, sort by magnitude, format for display
jpx 'features 
  | [?properties.mag >= `5.5`] 
  | sort_by(@, &properties.mag) 
  | reverse(@) 
  | [*].{
      mag: properties.mag,
      where: properties.place,
      depth: join(``, [to_string(geometry.coordinates[2]), ` km`])
    }' earthquakes.json
```

---

## Try It Yourself

1. **Fetch live data**: The USGS API updates in real-time
   ```bash
   curl -s "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&limit=50&minmagnitude=4" > quakes.json
   ```

2. **Explore the API parameters**:
   - `minmagnitude` / `maxmagnitude` - Filter by magnitude
   - `starttime` / `endtime` - Date range (ISO 8601)
   - `latitude` / `longitude` / `maxradiuskm` - Geographic radius
   - `limit` - Number of results (max 20000)

3. **API Documentation**: [USGS Earthquake API](https://earthquake.usgs.gov/fdsnws/event/1/)

## Related Functions

- [`geo_distance_km`](../functions/geo.md#geo_distance_km) - Calculate distance between coordinates
- [`avg`](../functions/math.md#avg), [`median`](../functions/math.md#median), [`stddev`](../functions/math.md#stddev) - Statistical functions
- [`from_unixtime`](../functions/datetime.md#from_unixtime) - Convert timestamps
- [`sort_by`](../functions/standard.md#sort_by), [`min_by`](../functions/standard.md#min_by), [`max_by`](../functions/standard.md#max_by) - Sorting functions
