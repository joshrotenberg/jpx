# Project Management Dataset

A comprehensive example using synthetic project management data that demonstrates most jpx function categories.

## Dataset

All examples use this project management dataset:

```json
[
  {
    "projectId": "P-8801",
    "status": "active",
    "timestamps": { "created_at": "2025-01-15T08:30:00Z", "last_modified": "2025-12-19T09:05:00Z" },
    "details": {
      "name": "Quantum Migration",
      "description": "High-priority initiative to transition legacy on-premise infrastructure to a distributed cloud-native architecture.",
      "tags": ["cloud", "security", "infrastructure"]
    },
    "milestones": [{ "phase": "Discovery", "due_date": "2025-03-01", "completed": true }]
  },
  {
    "projectId": "P-9902",
    "status": "pending",
    "timestamps": { "created_at": "2025-06-10T14:20:00Z", "last_modified": "2025-11-30T11:00:00Z" },
    "details": {
      "name": "Mobile App Redesign",
      "description": "Revamping the user interface for the primary customer-facing mobile application to improve accessibility and engagement.",
      "tags": ["UX", "frontend", "accessibility"]
    },
    "milestones": []
  },
  {
    "projectId": "P-4405",
    "status": "archived",
    "timestamps": { "created_at": "2024-11-01T09:00:00Z", "last_modified": "2025-01-20T16:30:00Z" },
    "details": {
      "name": "Legacy Data Cleanup",
      "description": "Automated purging of obsolete database records older than seven years to maintain compliance with regional data privacy laws.",
      "tags": ["database", "compliance", "automation"]
    },
    "milestones": [{ "phase": "Archive Audit", "due_date": "2025-01-15", "completed": true }]
  },
  {
    "projectId": "P-5521",
    "status": "active",
    "timestamps": { "created_at": "2025-02-14T10:15:00Z", "last_modified": "2025-12-10T14:45:00Z" },
    "details": {
      "name": "Cyber Sentinel Firewall",
      "description": "Implementation of AI-driven threat detection systems to safeguard corporate networks against zero-day vulnerabilities.",
      "tags": ["AI", "security", "network"]
    },
    "milestones": [{ "phase": "Training", "due_date": "2025-05-20", "completed": true }]
  },
  {
    "projectId": "P-1033",
    "status": "on-hold",
    "timestamps": { "created_at": "2025-03-22T13:00:00Z", "last_modified": "2025-09-05T08:20:00Z" },
    "details": {
      "name": "Blockchain Supply Chain",
      "description": "Integrating distributed ledger technology to provide end-to-end transparency for global logistics and manufacturing pipelines.",
      "tags": ["blockchain", "logistics", "transparency"]
    },
    "milestones": []
  },
  {
    "projectId": "P-2294",
    "status": "active",
    "timestamps": { "created_at": "2025-04-12T16:45:00Z", "last_modified": "2025-12-18T10:00:00Z" },
    "details": {
      "name": "Green Energy Analytics",
      "description": "Analyzing power consumption patterns to optimize the efficiency of solar and wind-based energy grids.",
      "tags": ["sustainability", "analytics", "energy"]
    },
    "milestones": [{ "phase": "Sensor Install", "due_date": "2025-08-30", "completed": true }]
  },
  {
    "projectId": "P-7761",
    "status": "pending",
    "timestamps": { "created_at": "2025-05-19T11:20:00Z", "last_modified": "2025-05-19T11:20:00Z" },
    "details": {
      "name": "Internal Wiki Overhaul",
      "description": "Restructuring the company knowledge base to facilitate better cross-departmental collaboration and documentation sharing.",
      "tags": ["knowledge", "internal", "documentation"]
    },
    "milestones": []
  },
  {
    "projectId": "P-6610",
    "status": "active",
    "timestamps": { "created_at": "2025-07-04T09:00:00Z", "last_modified": "2025-12-15T15:30:00Z" },
    "details": {
      "name": "E-Commerce Personalization",
      "description": "Deploying machine learning models to provide customized product recommendations based on real-time user behavior.",
      "tags": ["retail", "ML", "personalization"]
    },
    "milestones": [{ "phase": "Model Alpha", "due_date": "2025-11-01", "completed": true }]
  },
  {
    "projectId": "P-3349",
    "status": "archived",
    "timestamps": { "created_at": "2024-08-15T14:10:00Z", "last_modified": "2025-03-12T12:00:00Z" },
    "details": {
      "name": "Legacy Email Migration",
      "description": "Relocating historical email archives from physical tape drives to a searchable secure cloud vault.",
      "tags": ["email", "storage", "archival"]
    },
    "milestones": [{ "phase": "Final Sync", "due_date": "2025-02-28", "completed": true }]
  },
  {
    "projectId": "P-1122",
    "status": "active",
    "timestamps": { "created_at": "2025-09-30T08:45:00Z", "last_modified": "2025-12-19T11:15:00Z" },
    "details": {
      "name": "HR Portal Integration",
      "description": "Syncing payroll and performance management software to create a unified dashboard for human resources staff.",
      "tags": ["HR", "integration", "software"]
    },
    "milestones": [{ "phase": "API Bridge", "due_date": "2026-01-10", "completed": false }]
  }
]
```

---

## Basic Extraction

### Get all values for a field

```jmespath
# All project IDs
[*].projectId
```
```json
["P-8801", "P-9902", "P-4405", "P-5521", "P-1033", "P-2294", "P-7761", "P-6610", "P-3349", "P-1122"]
```

### Navigate nested objects

```jmespath
# All project names
[*].details.name
```
```json
["Quantum Migration", "Mobile App Redesign", "Legacy Data Cleanup", "Cyber Sentinel Firewall", "Blockchain Supply Chain", "Green Energy Analytics", "Internal Wiki Overhaul", "E-Commerce Personalization", "Legacy Email Migration", "HR Portal Integration"]
```

### Access by index

```jmespath
# First project's name
[0].details.name
```
```json
"Quantum Migration"
```

---

## Filtering

### Simple equality filter

```jmespath
# Active projects only
[?status == 'active'].details.name
```
```json
["Quantum Migration", "Cyber Sentinel Firewall", "Green Energy Analytics", "E-Commerce Personalization", "HR Portal Integration"]
```

### Numeric comparison

```jmespath
# Projects with milestones
[?length(milestones) > `0`].projectId
```
```json
["P-8801", "P-4405", "P-5521", "P-2294", "P-6610", "P-3349", "P-1122"]
```

### Empty array check

```jmespath
# Projects with no milestones
[?milestones == `[]`].details.name
```
```json
["Mobile App Redesign", "Blockchain Supply Chain", "Internal Wiki Overhaul"]
```

### Nested filtering

```jmespath
# All incomplete milestones across all projects
[*].milestones[?completed == `false`][]
```
```json
[{"phase": "API Bridge", "due_date": "2026-01-10", "completed": false}]
```

---

## Flattening & Nested Access

### Flatten nested arrays

```jmespath
# All tags across all projects (flattened)
[*].details.tags[]
```
```json
["cloud", "security", "infrastructure", "UX", "frontend", "accessibility", "database", "compliance", "automation", "AI", "security", "network", "blockchain", "logistics", "transparency", "sustainability", "analytics", "energy", "knowledge", "internal", "documentation", "retail", "ML", "personalization", "email", "storage", "archival", "HR", "integration", "software"]
```

### Get unique values

```jmespath
# Unique tags (deduplicated and sorted)
unique([*].details.tags[]) | sort(@)
```
```json
["AI", "HR", "ML", "UX", "accessibility", "analytics", "archival", "automation", "blockchain", "cloud", "compliance", "database", "documentation", "email", "energy", "frontend", "infrastructure", "integration", "internal", "knowledge", "logistics", "network", "personalization", "retail", "security", "software", "storage", "sustainability", "transparency"]
```

---

## Status & Grouping

### Count by category

```jmespath
# Count projects by status
frequencies([*].status)
```
```json
{"active": 5, "pending": 2, "archived": 2, "on-hold": 1}
```

### Group into buckets

```jmespath
# Group projects by status
group_by(@, 'status') | keys(@)
```
```json
["active", "archived", "on-hold", "pending"]
```

---

## Tag Operations

### Tag frequency analysis

```jmespath
# How often each tag appears
frequencies([*].details.tags[])
```
```json
{"security": 2, "cloud": 1, "infrastructure": 1, "UX": 1, "frontend": 1, ...}
```

### Find projects by tag

```jmespath
# Projects with "security" tag
[?includes(details.tags, 'security')].{id: projectId, name: details.name}
```
```json
[
  {"id": "P-8801", "name": "Quantum Migration"},
  {"id": "P-5521", "name": "Cyber Sentinel Firewall"}
]
```

### Multiple tag filter (AND)

```jmespath
# Projects with BOTH security AND cloud tags
[?includes(details.tags, 'security') && includes(details.tags, 'cloud')].projectId
```
```json
["P-8801"]
```

### Tags as comma-separated string

```jmespath
# All unique tags as one string
join(', ', unique([*].details.tags[]) | sort(@))
```
```json
"AI, HR, ML, UX, accessibility, analytics, archival, automation, blockchain, cloud, compliance, database, documentation, email, energy, frontend, infrastructure, integration, internal, knowledge, logistics, network, personalization, retail, security, software, storage, sustainability, transparency"
```

---

## Date & Time Queries

### Filter by date prefix

```jmespath
# Projects created in 2025
[?starts_with(timestamps.created_at, '2025')].projectId
```
```json
["P-8801", "P-9902", "P-5521", "P-1033", "P-2294", "P-7761", "P-6610", "P-1122"]
```

### Find most recent

```jmespath
# Most recently modified project
max_by(@, &timestamps.last_modified).details.name
```
```json
"HR Portal Integration"
```

### Find oldest

```jmespath
# Oldest project by creation date
min_by(@, &timestamps.created_at).details.name
```
```json
"Legacy Email Migration"
```

### Human-readable time ago

```jmespath
# Time since last modification (human readable)
[*].{name: details.name, last_touch: time_ago(timestamps.last_modified)}
```
```json
[
  {"name": "Quantum Migration", "last_touch": "1 hour ago"},
  {"name": "Mobile App Redesign", "last_touch": "19 days ago"},
  ...
]
```

### Stale project detection

```jmespath
# Projects not modified in 30+ days
[?date_diff(now(), to_epoch(timestamps.last_modified), 'days') > `30`].details.name
```
```json
["Mobile App Redesign", "Blockchain Supply Chain", "Internal Wiki Overhaul", "Legacy Email Migration"]
```

### Same-day check

```jmespath
# Projects modified today
[?is_same_day(timestamps.last_modified, from_epoch(now()))].details.name
```
```json
["Quantum Migration", "HR Portal Integration"]
```

---

## Milestone Analysis

### Projects without milestones

```jmespath
# Projects with no milestones defined
[?milestones == `[]`].{id: projectId, name: details.name, status: status}
```
```json
[
  {"id": "P-9902", "name": "Mobile App Redesign", "status": "pending"},
  {"id": "P-1033", "name": "Blockchain Supply Chain", "status": "on-hold"},
  {"id": "P-7761", "name": "Internal Wiki Overhaul", "status": "pending"}
]
```

### Find incomplete work

```jmespath
# Projects with pending milestones
[?milestones[?completed == `false`]].{
  project: details.name,
  pending: milestones[?completed == `false`][].phase
}
```
```json
[{"project": "HR Portal Integration", "pending": ["API Bridge"]}]
```

### Upcoming milestones

```jmespath
# All incomplete milestones with due dates
[*].milestones[?completed == `false`][].{phase: phase, due: due_date}
```
```json
[{"phase": "API Bridge", "due": "2026-01-10"}]
```

---

## Text & Search

### Fuzzy name search

```jmespath
# Fuzzy match project names
[?fuzzy_match(details.name, 'energy')].details.name
```
```json
["Green Energy Analytics"]
```

### Keyword search in descriptions

```jmespath
# Projects mentioning AI or machine learning
[?contains(details.description, 'AI') || contains(details.description, 'machine learning')].details.name
```
```json
["Cyber Sentinel Firewall", "E-Commerce Personalization"]
```

### Case-insensitive search

```jmespath
# Projects mentioning compliance (case-insensitive)
[?contains(lower(details.description), 'compliance')].details.name
```
```json
["Legacy Data Cleanup"]
```

### Word analysis

```jmespath
# Word count in descriptions
[*].{name: details.name, words: length(words(details.description))}
```
```json
[
  {"name": "Quantum Migration", "words": 14},
  {"name": "Mobile App Redesign", "words": 16},
  ...
]
```

### Truncated summaries

```jmespath
# Shortened descriptions
[*].{name: details.name, summary: truncate(details.description, `50`)}
```
```json
[
  {"name": "Quantum Migration", "summary": "High-priority initiative to transition legacy..."},
  ...
]
```

---

## String Transformations

### URL-friendly slugs

```jmespath
# Kebab-case project names for URLs
[*].{id: projectId, slug: kebab_case(details.name)}
```
```json
[
  {"id": "P-8801", "slug": "quantum-migration"},
  {"id": "P-9902", "slug": "mobile-app-redesign"},
  ...
]
```

### Database column names

```jmespath
# Snake-case for database columns
[*].details.name | map(&snake_case(@), @)
```
```json
["quantum_migration", "mobile_app_redesign", "legacy_data_cleanup", ...]
```

### Uppercase status

```jmespath
# Uppercase status field
[*].{name: details.name, status: upper(status)}
```
```json
[
  {"name": "Quantum Migration", "status": "ACTIVE"},
  ...
]
```

---

## Projections & Reshaping

### Create summary objects

```jmespath
# Clean summary for each project
[*].{id: projectId, name: details.name, status: status}
```
```json
[
  {"id": "P-8801", "name": "Quantum Migration", "status": "active"},
  {"id": "P-9902", "name": "Mobile App Redesign", "status": "pending"},
  ...
]
```

### Computed fields

```jmespath
# Add computed milestone count
[*].{
  name: details.name,
  status: status,
  milestone_count: length(milestones),
  has_incomplete: length(milestones[?completed == `false`]) > `0`
}
```
```json
[
  {"name": "Quantum Migration", "status": "active", "milestone_count": 1, "has_incomplete": false},
  {"name": "HR Portal Integration", "status": "active", "milestone_count": 1, "has_incomplete": true},
  ...
]
```

---

## Aggregation & Analytics

### Dashboard summary

```jmespath
# Executive dashboard
{
  total_projects: length(@),
  active: length([?status == 'active']),
  pending: length([?status == 'pending']),
  on_hold: length([?status == 'on-hold']),
  archived: length([?status == 'archived']),
  with_milestones: length([?milestones != `[]`]),
  without_milestones: length([?milestones == `[]`]),
  incomplete_milestones: length([*].milestones[?completed == `false`][]),
  newest_project: max_by(@, &timestamps.created_at).details.name,
  oldest_project: min_by(@, &timestamps.created_at).details.name,
  unique_tags: length(unique([*].details.tags[]))
}
```
```json
{
  "total_projects": 10,
  "active": 5,
  "pending": 2,
  "on_hold": 1,
  "archived": 2,
  "with_milestones": 7,
  "without_milestones": 3,
  "incomplete_milestones": 1,
  "newest_project": "HR Portal Integration",
  "oldest_project": "Legacy Email Migration",
  "unique_tags": 29
}
```

---

## Complex Filters

### Multiple conditions

```jmespath
# Active projects with completed milestones, modified this month
[?status == 'active'
  && milestones[?completed == `true`]
  && starts_with(timestamps.last_modified, '2025-12')
].details.name
```
```json
["Quantum Migration", "Cyber Sentinel Firewall", "Green Energy Analytics", "E-Commerce Personalization", "HR Portal Integration"]
```

### Stale pending projects

```jmespath
# Pending projects with no recent activity (60+ days)
[?status == 'pending'
  && date_diff(now(), to_epoch(timestamps.last_modified), 'days') > `60`
].details.name
```
```json
["Internal Wiki Overhaul"]
```

### At-risk detection

```jmespath
# Active projects without any milestones (potential risk)
[?status == 'active' && milestones == `[]`].details.name
```
```json
[]
```

---

## Export-Ready Formats

### CSV row generation

```jmespath
# Generate CSV rows
[*] | map(&to_csv([projectId, status, details.name]), @)
```
```json
[
  "P-8801,active,Quantum Migration",
  "P-9902,pending,Mobile App Redesign",
  ...
]
```

### Bullet list for reports

```jmespath
# Active project names as bullet list
[?status == 'active'].details.name | join('\n- ', @)
```
```json
"Quantum Migration\n- Cyber Sentinel Firewall\n- Green Energy Analytics\n- E-Commerce Personalization\n- HR Portal Integration"
```

### Full export with metadata

```jmespath
# Rich export format
[*].{
  id: projectId,
  title: details.name,
  status: upper(status),
  tags: join(', ', details.tags),
  age_days: date_diff(now(), to_epoch(timestamps.created_at), 'days'),
  last_touch: time_ago(timestamps.last_modified),
  slug: kebab_case(details.name)
}
```
```json
[
  {
    "id": "P-8801",
    "title": "Quantum Migration",
    "status": "ACTIVE",
    "tags": "cloud, security, infrastructure",
    "age_days": 338,
    "last_touch": "1 hour ago",
    "slug": "quantum-migration"
  },
  ...
]
```

---

## Function Categories Used

This guide demonstrates functions from these categories:

| Category | Functions Used |
|----------|----------------|
| **Array** | `first`, `last`, `unique`, `length`, `frequencies`, `group_by` |
| **String** | `upper`, `lower`, `join`, `split`, `starts_with`, `contains`, `truncate`, `kebab_case`, `snake_case`, `words` |
| **DateTime** | `now`, `to_epoch`, `from_epoch`, `time_ago`, `date_diff`, `is_same_day` |
| **Expression** | `map`, `filter_expr`, `max_by`, `min_by` |
| **Utility** | `includes`, `default` |
| **Fuzzy** | `fuzzy_match` |
| **Format** | `to_csv` |

---

## Try It Yourself

Save the sample dataset to a file and experiment:

```bash
# Save the JSON to a file
cat > projects.json << 'EOF'
[ ... paste the sample data ... ]
EOF

# Run queries with jpx
cat projects.json | jpx '[?status == '\''active'\''].details.name'
cat projects.json | jpx 'frequencies([*].status)'
cat projects.json | jpx '[*].{name: details.name, tags: join('\'', '\'', details.tags)}'
```

Or use the interactive REPL:

```bash
jpx --repl < projects.json
```
