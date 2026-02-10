# jpx Lightning Talk — Demo Script

Open this page in a browser alongside your terminal.
Each section is one step — run the command(s), talk through the output, move on.

---

## Pre-flight (do this before the talk)

```bash
# Cache earthquake data so you don't depend on WiFi
curl -s "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson" \
  > /tmp/quakes_week.json

# Verify jpx is working
jpx --version

# Terminal: 20pt+ font, dark background, maximize window
```

---

## Demo 1: Slides + quick hits (between slides 3 and 4)

### 1.1 — One-liner to set the tone

```bash
echo '[{"name":"Alice","active":true},{"name":"Bob","active":false}]' \
  | jpx '[?active].upper(name)'
```

> `["ALICE"]` — filter + transform, reads like English.

### 1.2 — Geo distance: Madrid to San Francisco

```bash
echo '{}' | jpx 'round(geo_distance_km(`40.4168`, `-3.7038`, `37.7749`, `-122.4194`), `0`)'
```

> `9318` km. Built-in geo functions — no libraries, no imports.

### 1.3 — Function discovery

```bash
jpx --search "distance"
jpx --describe geo_distance_km
```

> 400+ functions, all searchable and documented from the CLI.

---

## Demo 2: Earthquake deep dive (standalone 2-3 min segment)

Real USGS data, building queries from simple to complex,
ending with a reusable query library.

### 2.1 — Orient: what's in the data?

```bash
jpx 'length(features)' -f /tmp/quakes_week.json
```

```bash
jpx '{total: length(features), max: max(features[*].properties.mag), avg: round(avg(features[*].properties.mag), `2`)}' \
  -f /tmp/quakes_week.json
```

> ~85 M4.5+ earthquakes this past week. Max around 6.1.

### 2.2 — Reshape and sort

```bash
jpx 'features[*].{place: properties.place, mag: properties.mag} | sort_by(@, &mag) | reverse(@) | [:5]' \
  -f /tmp/quakes_week.json -t
```

> Top 5 strongest quakes, as a table. Multi-select hash, pipe to sort, slice.

### 2.3 — Add timestamps (epoch → human-readable)

```bash
jpx 'features[*].{place: properties.place, mag: properties.mag, when: format_date(divide(properties.time, `1000`), `"%b %d %H:%M"`)} | sort_by(@, &mag) | reverse(@) | [:5]' \
  -f /tmp/quakes_week.json -t
```

> Same query, now with formatted dates. `divide` converts millis → seconds, `format_date` does the rest.

### 2.4 — The pivot: "I saved these as a query library"

```bash
jpx -Q examples/earthquakes-madrid.jpx --list-queries
```

> 8 named queries in a plain text file. Let me show you the good ones.

### 2.5 — Nearest to Madrid (geo + let expressions)

```bash
jpx -Q examples/earthquakes-madrid.jpx:nearest -f /tmp/quakes_week.json -t
```

> "We're in Madrid — how close were the nearest quakes this week?"
> Uses `let $lat, $lon` to bind our coordinates, `geo_distance_km` for great-circle distance.

### 2.6 — Aggregate by region

```bash
jpx -Q examples/earthquakes-madrid.jpx:by-region -f /tmp/quakes_week.json -t
```

> Extracts country from place strings with `split` → `last`, groups by it,
> counts per region, sorts. One expression, no code.

### 2.7 — Full report: one query, complete picture

```bash
jpx -Q examples/earthquakes-madrid.jpx:full-report -f /tmp/quakes_week.json
```

> Stats, nearest-to-Madrid, and strongest — all combined into one structured object.

### 2.8 — The query file is just text

```bash
jpx -Q examples/earthquakes-madrid.jpx --check
```

> Every query validated. Plain text, version-controlled, shareable.
> No scripts, no code — just expressions.

---

## Fallback (no network)

All commands use cached `/tmp/quakes_week.json` from pre-flight.
If the pre-flight curl failed, copy the file from another machine or use sample data.

---

## REPL (if there's time to fill)

```bash
jpx --repl
```

Then type:

```
upper('hello madrid')
now()
format_date(now(), `"%A, %B %d %Y"`)
```
