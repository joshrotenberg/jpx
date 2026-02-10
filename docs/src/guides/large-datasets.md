# Large Datasets & Parquet

jpx can query large datasets that would overwhelm an LLM's context window.
With Parquet support, you get columnar compression and fast reads — the same
queries work on both JSON and Parquet files.

This guide uses the [Chicago Crimes](https://data.cityofchicago.org/Public-Safety/Crimes-2001-to-Present/ijzp-q8t2)
dataset — 200,000 real crime reports with dates, locations, crime types, and
arrest records.

## Why this matters for AI agents

When an AI agent needs to analyze JSON data, it typically reads the file
into its context window. That costs tokens and has hard limits:

| Data size | Tokens (approx.) | Fits in context? |
|-----------|------------------|-----------------|
| 60 KB (85 events) | 21,000 | Yes, but expensive |
| 46 MB (50K records) | 15,000,000 | No |
| 196 MB (200K records) | 65,000,000 | Absolutely not |

With jpx as an MCP sidecar, the file stays on disk. The agent sends a
short expression and gets back only the result — typically a few hundred
tokens regardless of input size.

## Getting the data

```bash
# Download 200K recent Chicago crime records (≈196 MB)
for offset in 0 50000 100000 150000; do
  curl -s "https://data.cityofchicago.org/resource/ijzp-q8t2.json?\$limit=50000&\$offset=$offset"
done | python3 -c "
import json, sys
records = []
for line in sys.stdin:
    records.extend(json.loads(line))
json.dump(records, open('chicago_crimes.json', 'w'))
print(f'{len(records)} records')
"
```

## Converting to Parquet

Requires jpx built with `--features parquet`:

```bash
cargo install jpx --features parquet,let-expr
```

Select the fields you need and write to Parquet:

```bash
jpx '[*].{
  id: id,
  case_number: case_number,
  date: date,
  primary_type: primary_type,
  description: description,
  location_description: location_description,
  arrest: arrest,
  domestic: domestic,
  beat: beat,
  district: district,
  ward: ward,
  year: year,
  latitude: latitude,
  longitude: longitude
}' -f chicago_crimes.json --parquet -o chicago_crimes.parquet
```

The result:

```
196 MB JSON  →  6.8 MB Parquet  (29x compression)
```

jpx auto-detects `.parquet` files — no flags needed to read them.

## Data structure

Each record looks like:

```json
{
  "id": "14101518",
  "case_number": "JK138251",
  "date": "2026-02-02T00:00:00.000",
  "primary_type": "BURGLARY",
  "description": "UNLAWFUL ENTRY",
  "location_description": "RESIDENCE",
  "arrest": false,
  "domestic": false,
  "beat": "0522",
  "district": "005",
  "ward": "9",
  "year": "2026",
  "latitude": "41.679126158",
  "longitude": "-87.62632452"
}
```

## Basic exploration

```bash
# How many records?
jpx 'length(@)' -f chicago_crimes.parquet
# → 200000

# What years are covered?
jpx 'sort(unique([*].year))' -f chicago_crimes.parquet
# → ["2025", "2026"]

# Summary stats
jpx '{
  total: length(@),
  years: sort(unique([*].year)),
  arrest_rate: round(divide(length([?arrest == `true`]), length(@)), `3`)
}' -f chicago_crimes.parquet
```

## Crime type analysis

### Top 10 crime types

```bash
jpx '[*].{type: primary_type}
  | group_by(@, '\''type'\'')
  | items(@)
  | [*].{crime: [0], count: length([1])}
  | sort_by(@, &count)
  | reverse(@)
  | [:10]' -f chicago_crimes.parquet -t
```

```
╭───────┬─────────────────────╮
│ count │ crime               │
├───────┼─────────────────────┤
│ 46138 │ THEFT               │
│ 36512 │ BATTERY             │
│ 22767 │ CRIMINAL DAMAGE     │
│ 18450 │ ASSAULT             │
│ 14928 │ MOTOR VEHICLE THEFT │
│ 13613 │ OTHER OFFENSE       │
│ 11708 │ DECEPTIVE PRACTICE  │
│ 8744  │ BURGLARY            │
│ 5830  │ NARCOTICS           │
│ 4864  │ ROBBERY             │
╰───────┴─────────────────────╯
```

### Arrest rate by crime type

Which crimes actually lead to arrests?

```bash
jpx 'let $types = [*].{type: primary_type, arrested: arrest} in
  $types[]
  | group_by(@, '\''type'\'')
  | items(@)
  | [*].{
    crime: [0],
    total: length([1]),
    arrests: length([1][?arrested == `true`]),
    rate: round(divide(length([1][?arrested == `true`]), length([1])), `3`)
  }
  | sort_by(@, &total)
  | reverse(@)
  | [:10]' -f chicago_crimes.parquet -t
```

```
╭─────────┬─────────────────────┬───────┬───────╮
│ arrests │ crime               │ rate  │ total │
├─────────┼─────────────────────┼───────┼───────┤
│ 3952    │ THEFT               │ 0.086 │ 46138 │
│ 6897    │ BATTERY             │ 0.189 │ 36512 │
│ 939     │ CRIMINAL DAMAGE     │ 0.041 │ 22767 │
│ 2280    │ ASSAULT             │ 0.124 │ 18450 │
│ 467     │ MOTOR VEHICLE THEFT │ 0.031 │ 14928 │
│ 2578    │ OTHER OFFENSE       │ 0.189 │ 13613 │
│ 362     │ DECEPTIVE PRACTICE  │ 0.031 │ 11708 │
│ 345     │ BURGLARY            │ 0.039 │ 8744  │
│ 5501    │ NARCOTICS           │ 0.944 │ 5830  │
│ 420     │ ROBBERY             │ 0.086 │ 4864  │
╰─────────┴─────────────────────┴───────┴───────╯
```

Narcotics has a 94% arrest rate — these are typically arrest-initiated reports.
Motor vehicle theft is 3% — the car is usually gone.

## Geographic analysis

### Crimes farthest from downtown Chicago

```bash
jpx 'let $lat = `41.8781`, $lon = `-87.6298` in
  [?latitude != null]
  | [*].{
    type: primary_type,
    dist_km: round(geo_distance_km($lat, $lon,
      to_number(latitude), to_number(longitude)), `2`),
    date: date
  }
  | sort_by(@, &dist_km)
  | reverse(@)
  | [:5]' -f chicago_crimes.parquet -t
```

### Crime by district

```bash
jpx '[*].{district: district}
  | group_by(@, '\''district'\'')
  | items(@)
  | [*].{district: [0], count: length([1])}
  | sort_by(@, &count)
  | reverse(@)
  | [:10]' -f chicago_crimes.parquet -t
```

## Using a query file

Save your queries as a reusable `.jpx` library:

```bash
cat > chicago.jpx << 'EOF'
-- :name stats
-- :desc Summary statistics
{
  total: length(@),
  years: sort(unique([*].year)),
  arrest_rate: round(divide(length([?arrest == `true`]), length(@)), `3`)
}

-- :name top-crimes
-- :desc Top 10 crime types by count
[*].{type: primary_type}
  | group_by(@, 'type')
  | items(@)
  | [*].{crime: [0], count: length([1])}
  | sort_by(@, &count)
  | reverse(@)
  | [:10]

-- :name arrests-by-type
-- :desc Arrest rate by crime type (top 10 crimes)
let $types = [*].{type: primary_type, arrested: arrest} in
$types[]
  | group_by(@, 'type')
  | items(@)
  | [*].{
    crime: [0],
    total: length([1]),
    arrests: length([1][?arrested == `true`]),
    rate: round(divide(length([1][?arrested == `true`]), length([1])), `3`)
  }
  | sort_by(@, &total)
  | reverse(@)
  | [:10]

-- :name by-district
-- :desc Crime count by district
[*].{district: district}
  | group_by(@, 'district')
  | items(@)
  | [*].{district: [0], count: length([1])}
  | sort_by(@, &count)
  | reverse(@)
  | [:10]
EOF
```

Then run any query against JSON or Parquet:

```bash
# Same query, either format
jpx -Q chicago.jpx:top-crimes -f chicago_crimes.json -t
jpx -Q chicago.jpx:top-crimes -f chicago_crimes.parquet -t
```

## Performance

All timings on a laptop (Apple M-series), 200K records:

| Query | JSON (196 MB) | Parquet (6.8 MB) |
|-------|--------------|-----------------|
| `length(@)` | ~1s | ~1s |
| `group_by` + `items` (top crimes) | ~1.2s | ~1.2s |
| `geo_distance_km` on all rows | ~1.8s | ~1.8s |
| Arrest rate with let + nested filters | ~1.5s | ~1.5s |

Query time is similar — the JMESPath evaluation dominates.
The Parquet advantage is **storage** (29x smaller) and **transfer**
(faster to download, copy, or send to a remote agent).

## The AI sidecar argument

When jpx runs as an MCP server, `evaluate_file` reads the Parquet file
on disk and returns only the query result. The token math:

| | Tokens |
|---|---|
| 200K records as raw JSON in context | ~65,000,000 (impossible) |
| jpx expression + result | ~150 |

The data never enters the context window. The result is deterministic —
the query engine processes 200K rows, not the LLM. Same query, same
answer, every time.

## More datasets to try

- [Chicago Crimes API](https://data.cityofchicago.org/resource/ijzp-q8t2.json) — up to 8M+ records
- [GH Archive](https://www.gharchive.org/) — GitHub events, ~900 MB/hour (NDJSON)
- [NYC Open Data](https://opendata.cityofnewyork.us/) — 311 complaints, taxi trips, more
- [data.gov](https://data.gov/) — US government open data
