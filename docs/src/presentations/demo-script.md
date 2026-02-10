# jpx Lightning Talk — Demo Script

Commands staged for live demo. Run these between slides 3 and 4.

## Pre-flight

```bash
# Cache earthquake data (in case conference WiFi is flaky)
curl -s "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson" > /tmp/quakes.json

# Verify jpx is working
jpx --version

# Terminal font: 20pt+ so the back row can read it
```

---

## 1. Quick taste (20s)

```bash
# Filter — reads like English: "users where active, get names"
echo '[{"name":"Alice","active":true},{"name":"Bob","active":false},{"name":"Charlie","active":true}]' \
  | jpx '[?active].upper(name)'
```

Expected: `["ALICE","CHARLIE"]`

---

## 2. Real API — USGS earthquakes (40s)

```bash
# Recent significant earthquakes worldwide
curl -s "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/significant_month.geojson" \
  | jpx 'features[:5].{place: properties.place, mag: properties.mag}'
```

```bash
# Full statistics — using a query file
jpx -Q examples/earthquakes.jpx:mag-stats -f /tmp/quakes.json
```

```bash
# Table output — sorted by magnitude
jpx -Q examples/earthquakes.jpx:sort-by-mag -f /tmp/quakes.json -t
```

---

## 3. Superpowers (40s)

```bash
# Geo: How far are we from home? Madrid → San Francisco
echo '{}' | jpx 'round(geo_distance_km(`40.4168`, `-3.7038`, `37.7749`, `-122.4194`), `0`)'
```

Expected: `9318` (km)

```bash
# Hashing
echo '"conference-wifi-password"' | jpx 'sha256(@)'
```

```bash
# Fuzzy matching — find names similar to "Josh"
echo '{"names":["Josh","Joseph","Joshua","Jessica","James"]}' \
  | jpx 'names[*].{name: @, score: round(jaro_winkler(@, `"Josh"`), `2`)} | [?score > `0.8`]'
```

```bash
# Let expressions — named intermediate values
echo '{"scores":[85,92,67,78,95,43,88]}' \
  | jpx 'let $s = scores in {count: length($s), avg: round(avg($s), `1`), median: median($s), stddev: round(stddev($s), `2`)}'
```

---

## 4. Function discovery (15s)

```bash
# BM25 search across 400+ functions
jpx --search "distance"
```

```bash
# Detailed docs for any function
jpx --describe geo_distance_km
```

---

## 5. If time: REPL

```bash
jpx --repl
```

Then type:
```
upper('hello madrid')
now()
format_date(now(), `"%A, %B %d %Y"`)
```

---

## Fallback commands (no network needed)

If WiFi dies, skip section 2 and use these instead:

```bash
# Use cached earthquake data
jpx 'features[:5].{place: properties.place, mag: properties.mag}' -f /tmp/quakes.json

# Stats from cached data
jpx -Q examples/earthquakes.jpx:mag-stats -f /tmp/quakes.json
```
