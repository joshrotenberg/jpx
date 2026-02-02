# NASA Near Earth Objects

NASA's [Near Earth Object Web Service](https://api.nasa.gov/) provides data about asteroids and comets that pass close to Earth. This dataset is excellent for learning jpx because it has:

- Deeply nested data structures
- Multiple unit systems (km, miles, AU, lunar distances)
- Boolean flags for hazard classification
- Time series data (close approach dates)
- Complex orbital mechanics data

## Getting the Data

```bash
# Browse all NEOs (paginated)
curl -s "https://api.nasa.gov/neo/rest/v1/neo/browse?api_key=DEMO_KEY" > neo_browse.json

# Get NEOs by close approach date (today)
curl -s "https://api.nasa.gov/neo/rest/v1/feed?api_key=DEMO_KEY" > neo_feed.json

# Get specific date range
curl -s "https://api.nasa.gov/neo/rest/v1/feed?start_date=2024-01-01&end_date=2024-01-07&api_key=DEMO_KEY" > neo_week.json

# Look up specific asteroid
curl -s "https://api.nasa.gov/neo/rest/v1/neo/2000433?api_key=DEMO_KEY" > eros.json
```

> **Note**: Use `DEMO_KEY` for testing (limited rate). Get a free API key at [api.nasa.gov](https://api.nasa.gov/).

## Data Structure

The browse endpoint returns:

```json
{
  "links": {"self": "...", "next": "..."},
  "page": {
    "size": 20,
    "total_elements": 41839,
    "total_pages": 2092,
    "number": 0
  },
  "near_earth_objects": [
    {
      "id": "2000433",
      "name": "433 Eros (A898 PA)",
      "name_limited": "Eros",
      "nasa_jpl_url": "https://ssd.jpl.nasa.gov/...",
      "absolute_magnitude_h": 10.38,
      "estimated_diameter": {
        "kilometers": {"estimated_diameter_min": 22.31, "estimated_diameter_max": 49.89},
        "meters": {...},
        "miles": {...},
        "feet": {...}
      },
      "is_potentially_hazardous_asteroid": false,
      "close_approach_data": [
        {
          "close_approach_date": "2024-01-15",
          "relative_velocity": {
            "kilometers_per_second": "5.57",
            "kilometers_per_hour": "20083.02",
            "miles_per_hour": "12478.81"
          },
          "miss_distance": {
            "astronomical": "0.314",
            "lunar": "122.5",
            "kilometers": "47112732",
            "miles": "29274494"
          },
          "orbiting_body": "Earth"
        }
      ]
    }
  ]
}
```

---

## Basic Queries

### List Asteroid Names

```bash
jpx 'near_earth_objects[*].name' neo_browse.json
```

### Get Short Names

```bash
jpx 'near_earth_objects[*].name_limited' neo_browse.json
```

Output:
```json
["Eros", "Albert", "Alinda", "Ganymed", ...]
```

### Count NEOs in Response

```bash
jpx 'page.total_elements' neo_browse.json
```

Output: `41839` (total known NEOs!)

---

## Filtering

### Find Potentially Hazardous Asteroids (PHAs)

```bash
jpx 'near_earth_objects[?is_potentially_hazardous_asteroid == `true`].{
  name: name_limited,
  diameter_km: estimated_diameter.kilometers.estimated_diameter_max
}' neo_browse.json
```

### Find Large Asteroids (> 1km)

```bash
jpx 'near_earth_objects[?estimated_diameter.kilometers.estimated_diameter_max > `1`].{
  name: name,
  max_diameter_km: estimated_diameter.kilometers.estimated_diameter_max,
  hazardous: is_potentially_hazardous_asteroid
}' neo_browse.json
```

### Find Non-Hazardous Objects

```bash
jpx 'near_earth_objects[?is_potentially_hazardous_asteroid == `false`].name_limited' neo_browse.json
```

---

## Working with Nested Data

### Extract Diameter Estimates

The API provides diameters in multiple units:

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  min_km: estimated_diameter.kilometers.estimated_diameter_min,
  max_km: estimated_diameter.kilometers.estimated_diameter_max
}' neo_browse.json
```

### Calculate Average Diameter

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  avg_diameter_km: divide(
    add(
      estimated_diameter.kilometers.estimated_diameter_min,
      estimated_diameter.kilometers.estimated_diameter_max
    ),
    `2`
  )
}' neo_browse.json
```

### Get Diameter in Different Units

```bash
jpx 'near_earth_objects[0].{
  name: name,
  in_km: estimated_diameter.kilometers,
  in_meters: estimated_diameter.meters,
  in_miles: estimated_diameter.miles
}' neo_browse.json
```

---

## Close Approach Data

### Get Next Close Approach

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  next_approach: close_approach_data[0].close_approach_date,
  miss_distance_km: close_approach_data[0].miss_distance.kilometers
}' neo_browse.json
```

### Find Closest Approaches (in Lunar Distances)

One lunar distance = 384,400 km (distance to the Moon):

```bash
jpx 'near_earth_objects[*].close_approach_data[*].{
  asteroid: @.@.@.name_limited,
  date: close_approach_date,
  lunar_distances: miss_distance.lunar
} | flatten(@)' neo_browse.json
```

### Sort by Miss Distance

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  miss_km: to_number(close_approach_data[0].miss_distance.kilometers)
} | sort_by(@, &miss_km) | [:10]' neo_browse.json
```

### Find High-Speed Approaches

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  speed_kph: to_number(close_approach_data[0].relative_velocity.kilometers_per_hour),
  date: close_approach_data[0].close_approach_date
} | [?speed_kph > `50000`]' neo_browse.json
```

---

## Statistics

### Size Statistics

```bash
jpx '{
  count: length(near_earth_objects),
  smallest_km: min(near_earth_objects[*].estimated_diameter.kilometers.estimated_diameter_min),
  largest_km: max(near_earth_objects[*].estimated_diameter.kilometers.estimated_diameter_max),
  avg_max_km: round(avg(near_earth_objects[*].estimated_diameter.kilometers.estimated_diameter_max), `2`)
}' neo_browse.json
```

### Hazard Statistics

```bash
jpx '{
  total: length(near_earth_objects),
  hazardous: length(near_earth_objects[?is_potentially_hazardous_asteroid == `true`]),
  safe: length(near_earth_objects[?is_potentially_hazardous_asteroid == `false`]),
  hazard_percentage: multiply(
    divide(
      length(near_earth_objects[?is_potentially_hazardous_asteroid == `true`]),
      length(near_earth_objects)
    ),
    `100`
  )
}' neo_browse.json
```

### Magnitude Distribution

Absolute magnitude (H) indicates brightness/size:

```bash
jpx '{
  magnitudes: near_earth_objects[*].absolute_magnitude_h,
  min_mag: min(near_earth_objects[*].absolute_magnitude_h),
  max_mag: max(near_earth_objects[*].absolute_magnitude_h),
  avg_mag: round(avg(near_earth_objects[*].absolute_magnitude_h), `2`)
}' neo_browse.json
```

---

## Sorting and Ranking

### Top 5 Largest Asteroids

```bash
jpx 'sort_by(near_earth_objects, &estimated_diameter.kilometers.estimated_diameter_max) 
  | reverse(@) 
  | [:5] 
  | [*].{
      name: name,
      max_diameter_km: round(estimated_diameter.kilometers.estimated_diameter_max, `2`),
      hazardous: is_potentially_hazardous_asteroid
    }' neo_browse.json
```

### Brightest Objects (Lowest Magnitude)

```bash
jpx 'sort_by(near_earth_objects, &absolute_magnitude_h) | [:5] | [*].{
  name: name_limited,
  magnitude: absolute_magnitude_h,
  diameter_km: estimated_diameter.kilometers.estimated_diameter_max
}' neo_browse.json
```

---

## Unit Conversions

### Convert AU to Kilometers

1 AU (Astronomical Unit) = ~149,597,870.7 km:

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  miss_au: to_number(close_approach_data[0].miss_distance.astronomical),
  miss_km: to_number(close_approach_data[0].miss_distance.kilometers)
}' neo_browse.json
```

### Velocity Comparisons

```bash
jpx 'near_earth_objects[*].{
  name: name_limited,
  km_per_sec: to_number(close_approach_data[0].relative_velocity.kilometers_per_second),
  km_per_hour: to_number(close_approach_data[0].relative_velocity.kilometers_per_hour),
  mph: to_number(close_approach_data[0].relative_velocity.miles_per_hour)
}' neo_browse.json
```

---

## Data Transformation

### Flatten for Analysis

```bash
jpx 'near_earth_objects[*].{
  id: id,
  name: name_limited,
  magnitude: absolute_magnitude_h,
  diameter_min_km: estimated_diameter.kilometers.estimated_diameter_min,
  diameter_max_km: estimated_diameter.kilometers.estimated_diameter_max,
  is_hazardous: is_potentially_hazardous_asteroid,
  next_approach_date: close_approach_data[0].close_approach_date,
  miss_distance_km: close_approach_data[0].miss_distance.kilometers,
  velocity_kph: close_approach_data[0].relative_velocity.kilometers_per_hour,
  jpl_url: nasa_jpl_url
}' neo_browse.json
```

### Export to CSV

```bash
jpx --csv 'near_earth_objects[*].{
  name: name_limited,
  magnitude: absolute_magnitude_h,
  max_diameter_km: estimated_diameter.kilometers.estimated_diameter_max,
  is_hazardous: is_potentially_hazardous_asteroid,
  approach_date: close_approach_data[0].close_approach_date
}' neo_browse.json
```

---

## Advanced Pipelines

### Risk Assessment Report

```bash
jpx '{
  report_title: `Near Earth Object Risk Assessment`,
  total_objects: page.total_elements,
  sample_size: length(near_earth_objects),
  potentially_hazardous: {
    count: length(near_earth_objects[?is_potentially_hazardous_asteroid == `true`]),
    objects: near_earth_objects[?is_potentially_hazardous_asteroid == `true`] | [*].{
      name: name,
      diameter_km: round(estimated_diameter.kilometers.estimated_diameter_max, `2`),
      next_approach: close_approach_data[0].close_approach_date,
      miss_lunar: close_approach_data[0].miss_distance.lunar
    }
  },
  largest_objects: sort_by(near_earth_objects, &estimated_diameter.kilometers.estimated_diameter_max) 
    | reverse(@) 
    | [:3] 
    | [*].{name: name_limited, km: round(estimated_diameter.kilometers.estimated_diameter_max, `1`)}
}' neo_browse.json
```

### Pipeline: Filter, Sort, Transform

```bash
jpx 'near_earth_objects
  | [?estimated_diameter.kilometers.estimated_diameter_max > `0.5`]
  | [?is_potentially_hazardous_asteroid == `true`]
  | sort_by(@, &estimated_diameter.kilometers.estimated_diameter_max)
  | reverse(@)
  | [*].{
      name: name,
      size_category: `LARGE PHA`,
      diameter_range: concat(
        to_string(round(estimated_diameter.kilometers.estimated_diameter_min, `1`)),
        ` - `,
        to_string(round(estimated_diameter.kilometers.estimated_diameter_max, `1`)),
        ` km`
      )
    }' neo_browse.json
```

---

## Working with the Feed Endpoint

The feed endpoint returns NEOs by date:

```bash
curl -s "https://api.nasa.gov/neo/rest/v1/feed?start_date=2024-01-15&end_date=2024-01-16&api_key=DEMO_KEY" > neo_day.json
```

### Count by Date

```bash
jpx 'element_count' neo_day.json
```

### List All Dates with Objects

```bash
jpx 'keys(near_earth_objects)' neo_day.json
```

### Get Objects for Specific Date

```bash
jpx 'near_earth_objects."2024-01-15"[*].name' neo_day.json
```

---

## Try It Yourself

1. **Get your free API key**: [api.nasa.gov](https://api.nasa.gov/)

2. **Interesting queries to try**:
   - Find the closest approach ever recorded
   - List all PHAs larger than 500 meters
   - Calculate average velocity of approaching objects

3. **API endpoints**:
   - `/neo/browse` - Paginated list of all NEOs
   - `/neo/rest/v1/feed` - NEOs by approach date
   - `/neo/{id}` - Specific asteroid details

4. **API Documentation**: [NASA NEO API](https://api.nasa.gov/)

## Related Functions

- [`sort_by`](../functions/standard.md#sort_by), [`reverse`](../functions/standard.md#reverse) - Sorting and ordering
- [`to_number`](../functions/standard.md#to_number) - Convert string numbers
- [`round`](../functions/math.md#round), [`divide`](../functions/math.md#divide), [`multiply`](../functions/math.md#multiply) - Math operations
- [`avg`](../functions/math.md#avg), [`min`](../functions/standard.md#min), [`max`](../functions/standard.md#max) - Statistics
- [`flatten`](../functions/array.md#flatten) - Flatten nested arrays
