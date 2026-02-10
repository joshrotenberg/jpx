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

The query:

```jmespath
let $lat = `40.4168`, $lon = `-3.7038` in
features[*].{
  place: properties.place,
  mag: properties.mag,
  km: round(geo_distance_km($lat, $lon,
    geometry.coordinates[1], geometry.coordinates[0])),
  when: format_date(divide(properties.time, `1000`), `"%b %d %H:%M"`)
} | sort_by(@, &km) | [:5]
```

> "We're in Madrid — how close were the nearest quakes this week?"
> `let` binds our coordinates, `geo_distance_km` computes great-circle
> distance for every event, `format_date` + `divide` converts epoch millis.

### 2.6 — Most-felt earthquakes

```bash
jpx -Q examples/earthquakes-madrid.jpx:felt -f /tmp/quakes_week.json -t
```

The query:

```jmespath
features[?properties.felt != null]
  | sort_by(@, &properties.felt)
  | reverse(@)
  | [:5]
  | [*].{
    place: properties.place,
    mag: properties.mag,
    felt_reports: properties.felt,
    significance: properties.sig
  }
```

> USGS "Did you feel it?" reports. Filter nulls, sort, reshape.

### 2.7 — Aggregate by region

```bash
jpx -Q examples/earthquakes-madrid.jpx:by-region -f /tmp/quakes_week.json -t
```

The query:

```jmespath
features[*].{
  region: last(split(properties.place, `", "`)),
  mag: properties.mag
} | group_by(@, 'region')
  | items(@)
  | [*].{region: [0], count: length([1]), max_mag: max([1][*].mag)}
  | sort_by(@, &count)
  | reverse(@)
  | [:10]
```

> Extracts country from place strings with `split` → `last`, groups by it,
> counts per region, sorts. One expression, no code.

### 2.8 — Full report: one query, complete picture

```bash
jpx -Q examples/earthquakes-madrid.jpx:full-report -f /tmp/quakes_week.json
```

The query:

```jmespath
let $lat = `40.4168`, $lon = `-3.7038` in {
  summary: {
    total: length(features),
    magnitude: {
      avg: round(avg(features[*].properties.mag), `2`),
      max: max(features[*].properties.mag),
      median: median(features[*].properties.mag)
    },
    depth: {
      avg_km: round(avg(features[*].geometry.coordinates[2]), `1`),
      max_km: round(max(features[*].geometry.coordinates[2]), `1`)
    }
  },
  nearest_to_madrid: features[*].{
    place: properties.place,
    km: round(geo_distance_km($lat, $lon,
      geometry.coordinates[1], geometry.coordinates[0]))
  } | sort_by(@, &km) | [:3],
  strongest: sort_by(features, &properties.mag)
    | reverse(@) | [:3]
    | [*].{place: properties.place, mag: properties.mag}
}
```

> Stats, nearest-to-Madrid, and strongest — all combined into one structured object.

### 2.9 — The query file is just text

```bash
jpx -Q examples/earthquakes-madrid.jpx --check
```

> Every query validated. Plain text, version-controlled, shareable.
> No scripts, no code — just expressions.

---

## Demo 3: MCP — let the AI drive (standalone 2-3 min segment)

This is the "wow" moment. Open Claude Code with the jpx MCP server
configured, paste one prompt, and let the audience watch the AI
explore earthquake data using MCP tool calls in real time.

### Setup

Make sure `/tmp/quakes_week.json` exists from pre-flight.
Open Claude Code in a terminal with large font.

### The prompt

Paste this into Claude Code:

```
I have USGS earthquake data (M4.5+ past week, GeoJSON format) at
/tmp/quakes_week.json. Using the jpx MCP tools:

1. Explore the data structure and give me a quick summary
2. Find the 5 earthquakes nearest to Madrid (40.42°N, 3.70°W)
   using geo functions — show as a table with distance in km
3. Which regions had the most activity? Show the top 10.
4. Save the "nearest to Madrid" query so I can reuse it later
```

### What the audience should see

The AI will make a series of MCP tool calls — talk over them:

1. **`stats`** or **`evaluate_file`** — "It's exploring the data shape first"
2. **`search`** — "It's discovering geo functions — it didn't know the name,
   it searched for it"
3. **`evaluate_file`** with `geo_distance_km` + `let` expression —
   "There's the query — geo distance from our coordinates, sorted"
4. **`evaluate_file`** with `group_by` + `items` — "Aggregation by region,
   one expression"
5. **`define_query`** — "And now it saved that query for reuse"

### Talk track

> "This is jpx as an MCP server — 29 tools that let any AI assistant
> query JSON, discover functions, and build reusable query libraries.
> I gave it one prompt and it explored the data, found the right
> functions, built the queries, and saved them. No code. No scripts."

### The token argument (for the slide or Q&A)

Real numbers from the earthquake data we just queried:

| | Tokens | Notes |
|---|---|---|
| **Without jpx** — full JSON in context | 21,469 | 60 KB file, 85 events |
| **With jpx** — expression + result only | 355 | 92 (query) + 263 (result) |
| **Savings** | **98.3%** | **60x fewer tokens** |

The result size is constant — "top 5 nearest" returns 5 rows whether
the input has 85 events or 40,000. Bigger data = bigger savings.

And the query engine is deterministic. The LLM doesn't read the JSON —
it writes an expression and the engine evaluates it. No hallucinated values,
no miscounted arrays, same result every time.

### Tips

- If response is slow, narrate what's happening: "It's thinking about
  which functions to use..."
- The tool calls stream in real time in Claude Code — that's the demo
- If it errors on a query (it happens!), say: "And it self-corrects —
  it'll check the function signature and fix it"
- Consider pre-recording this as a backup video in case of WiFi/latency issues

---

## Demo 4: "What if the data doesn't fit?" — Parquet at scale

This is the closer. Everything before this used 85 events (60 KB).
Now show what happens with half a million rows.

Requires jpx built with `--features parquet` (`cargo install jpx --features parquet,let-expr`).

### Pre-flight

Generate the dataset beforehand (takes ~10s):

```bash
# Generate 500K synthetic earthquake events (95 MB JSON)
python3 -c "
import json, random, time
regions = [('Russia',55,100),('Japan',36,140),('Indonesia',-5,120),
  ('Chile',-33,-71),('Mexico',19,-99),('Philippines',12,122),
  ('Turkey',39,35),('Italy',42,13),('Greece',38,23),('Iran',33,52),
  ('India',25,80),('USA',37,-120),('Peru',-12,-77),('New Zealand',-42,173)]
events = []
base_time = int(time.time()*1000)
for i in range(500_000):
    r,lat,lon = random.choice(regions)
    events.append({'id':f'ev{i:07d}','mag':round(random.uniform(1,8),2),
      'place':f'{random.randint(10,300)} km from {r}','region':r,
      'lat':round(lat+random.uniform(-5,5),4),'lon':round(lon+random.uniform(-5,5),4),
      'depth_km':round(random.uniform(1,600),1),'time':base_time-random.randint(0,365*24*3600*1000),
      'felt':random.randint(0,500) if random.random()<0.1 else None,
      'sig':int(round(random.uniform(1,8),2)*80+random.uniform(-20,20)),
      'tsunami':1 if random.uniform(1,8)>7 and random.random()<0.3 else 0})
json.dump(events,open('/tmp/quakes_500k.json','w'))
print(f'{len(events)} events written')
"

# Convert to Parquet (Snappy compression)
jpx '@' -f /tmp/quakes_500k.json --parquet -o /tmp/quakes_500k.parquet
```

### 4.1 — The setup: show the file sizes

```bash
ls -lh /tmp/quakes_500k.json /tmp/quakes_500k.parquet
```

> 95 MB JSON → 16 MB Parquet. 6x compression, same data.

### 4.2 — Count: half a million rows

```bash
time jpx 'length(@)' -f /tmp/quakes_500k.parquet
```

> 500,000 — reads parquet natively, under 2 seconds.

### 4.3 — Aggregate stats across all 500K events

```bash
time jpx '{total: length(@), avg_mag: round(avg([*].mag), `2`), max_mag: max([*].mag), tsunami_count: length([?tsunami == `1`])}' \
  -f /tmp/quakes_500k.parquet
```

> Full statistical summary in ~2 seconds.

### 4.4 — Geo distance on every row: nearest to Madrid

```bash
time jpx 'let $lat = `40.4168`, $lon = `-3.7038` in [*].{place: place, mag: mag, km: round(geo_distance_km($lat, $lon, lat, lon))} | sort_by(@, &km) | [:5]' \
  -f /tmp/quakes_500k.parquet -t
```

> geo_distance_km computed 500,000 times, sorted, top 5 returned — ~2 seconds.

### 4.5 — The punchline: token math

> "That JSON file is 95 megabytes. That's roughly **33 million tokens**.
> No context window on earth can hold that.
> With jpx, the agent sends a 61-token expression and gets back 88 tokens.
> **The data never enters the context window. Period.**"

| | Tokens |
|---|---|
| 95 MB JSON in context | ~33,000,000 (impossible) |
| jpx expression + result | 149 |

> "And every result is deterministic. The query engine processed
> half a million rows — not the LLM. Same query, same answer, every time."

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
