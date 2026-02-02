# Real-World Datasets

Learn jpx by working with real data from public APIs. Each example includes:

- How to fetch the data
- Data structure overview
- Progressive examples from basic to advanced
- Practical use cases

## Available Guides

| Guide | Description | Key Features |
|-------|-------------|--------------|
| [Standard JMESPath Only](./standard-jmespath.md) | Portable queries using only spec functions | 26 built-in functions, no extensions |
| [NLP Text Processing](./nlp-text-processing.md) | Text analysis pipelines | Tokenization, stemming, stopwords, normalization |
| [Hacker News](./hacker-news.md) | Tech discussions via Algolia API | NLP on real content, topic detection, vocabulary analysis |
| [USGS Earthquakes](./earthquakes.md) | Real-time seismic data | Geo functions, statistics, filtering |
| [Nobel Prize API](./nobel-prize.md) | Laureates and prizes | Multilingual data, text processing, dates |
| [NASA Near Earth Objects](./nasa-neo.md) | Asteroids and comets | Nested data, unit conversions, risk analysis |
| [Project Management](./project-management.md) | Synthetic project data | Comprehensive function coverage, all categories |

## Quick Start

```bash
# Fetch earthquake data
curl -s "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&limit=20&minmagnitude=5" > quakes.json

# Try a query
jpx 'features[*].{mag: properties.mag, place: properties.place}' quakes.json
```

## What You'll Learn

### Filtering & Selection
- Complex filter expressions with multiple conditions
- Nested field access patterns
- Text-based filtering with `contains`, `starts_with`

### Statistics & Aggregation
- `avg`, `median`, `stddev` for numeric analysis
- `min`, `max`, `min_by`, `max_by` for extremes
- `length` and counting patterns

### Geographic Calculations
- `geo_distance_km` for distance calculations
- Coordinate extraction and formatting
- Distance-based sorting

### Date/Time Operations
- Unix timestamp conversion with `from_unixtime`
- Date formatting with `format_datetime`
- Date range filtering

### Data Transformation
- Reshaping nested structures
- Flattening for export
- CSV/TSV output for spreadsheets

### Pipeline Patterns
- Multi-step transformations
- Sorting and limiting results
- Building summary reports

## Tips for Working with APIs

1. **Save data locally** for faster iteration:
   ```bash
   curl -s "API_URL" > data.json
   jpx 'expression' data.json
   ```

2. **Explore structure first**:
   ```bash
   jpx 'keys(@)' data.json          # Top-level keys
   jpx '@[0]' data.json             # First element (arrays)
   jpx 'type(@)' data.json          # Data type
   ```

3. **Use `--compact` for pipelines**:
   ```bash
   jpx -c 'expression' data.json | jpx 'next_expression'
   ```

4. **Export for analysis**:
   ```bash
   jpx --csv 'transform' data.json > output.csv
   ```

## More Data Sources

Looking for more datasets to practice with? Check out:

- [Awesome JSON Datasets](https://github.com/jdorfman/awesome-json-datasets) - Curated list of public JSON APIs
- [Public APIs](https://github.com/public-apis/public-apis) - Collective list of free APIs
- [NASA Open APIs](https://api.nasa.gov/) - Space and Earth science data
- [OpenWeatherMap](https://openweathermap.org/api) - Weather data
- [GitHub API](https://docs.github.com/en/rest) - Repository and user data
