# Nobel Prize Data

The [Nobel Prize API](https://www.nobelprize.org/about/developer-zone-2/) provides comprehensive data about Nobel laureates and prizes. This dataset is excellent for learning jpx because it has:

- Deeply nested multilingual data
- Rich biographical information
- Dates and geographic locations
- Complex relationships (laureates, prizes, affiliations)

## Getting the Data

```bash
# Fetch laureates (paginated, 25 per page by default)
curl -s "https://api.nobelprize.org/2.1/laureates?limit=50" > laureates.json

# Fetch all prizes
curl -s "https://api.nobelprize.org/2.1/nobelPrizes?limit=100" > prizes.json

# Fetch laureates by category
curl -s "https://api.nobelprize.org/2.1/laureates?nobelPrizeCategory=phy&limit=50" > physics_laureates.json
```

## Data Structure

The laureates endpoint returns:

```json
{
  "laureates": [
    {
      "id": "102",
      "knownName": {"en": "Aage N. Bohr", "se": "Aage N. Bohr"},
      "givenName": {"en": "Aage N."},
      "familyName": {"en": "Bohr"},
      "gender": "male",
      "birth": {
        "date": "1922-06-19",
        "place": {
          "city": {"en": "Copenhagen"},
          "country": {"en": "Denmark"},
          "continent": {"en": "Europe"}
        }
      },
      "death": {
        "date": "2009-09-08",
        "place": {...}
      },
      "nobelPrizes": [
        {
          "awardYear": "1975",
          "category": {"en": "Physics"},
          "portion": "1/3",
          "motivation": {"en": "for the discovery of..."},
          "prizeAmount": 630000,
          "affiliations": [...]
        }
      ]
    }
  ],
  "meta": {"count": 50, "limit": 50, "offset": 0}
}
```

---

## Basic Queries

### List All Laureate Names

```bash
jpx 'laureates[*].knownName.en' laureates.json
```

### Get Name and Prize Year

```bash
jpx 'laureates[*].{
  name: knownName.en,
  year: nobelPrizes[0].awardYear,
  category: nobelPrizes[0].category.en
}' laureates.json
```

Output:
```json
[
  {"name": "A. Michael Spence", "year": "2001", "category": "Economic Sciences"},
  {"name": "Aage N. Bohr", "year": "1975", "category": "Physics"},
  ...
]
```

### Count Laureates

```bash
jpx 'length(laureates)' laureates.json
```

---

## Filtering

### Find Physics Laureates

```bash
jpx 'laureates[?nobelPrizes[0].category.en == `Physics`].knownName.en' laureates.json
```

### Find Female Laureates

```bash
jpx 'laureates[?gender == `female`].{
  name: knownName.en,
  category: nobelPrizes[0].category.en,
  year: nobelPrizes[0].awardYear
}' laureates.json
```

### Find Living Laureates

```bash
jpx 'laureates[?death == null].{
  name: knownName.en,
  born: birth.date
}' laureates.json
```

### Find Laureates Born in a Specific Country

```bash
jpx 'laureates[?birth.place.country.en == `USA`].{
  name: knownName.en,
  city: birth.place.city.en
}' laureates.json
```

### Find Multiple Prize Winners

Some laureates have won multiple Nobel Prizes:

```bash
jpx 'laureates[?length(nobelPrizes) > `1`].{
  name: knownName.en,
  prizes: nobelPrizes[*].{
    year: awardYear,
    category: category.en
  }
}' laureates.json
```

---

## Working with Multilingual Data

The API provides data in multiple languages (en, se, no):

### Extract English Names

```bash
jpx 'laureates[*].knownName.en' laureates.json
```

### Compare Languages

```bash
jpx 'laureates[:5] | [*].{
  english: knownName.en,
  swedish: knownName.se
}' laureates.json
```

### Get Motivation in English

```bash
jpx 'laureates[*].{
  name: knownName.en,
  motivation: nobelPrizes[0].motivation.en
}' laureates.json
```

---

## Date Operations

### Extract Birth Years

```bash
jpx 'laureates[*].{
  name: knownName.en,
  birth_year: split(birth.date, `-`)[0]
}' laureates.json
```

### Calculate Age at Award

```bash
jpx 'laureates[?birth.date != null].{
  name: knownName.en,
  born: split(birth.date, `-`)[0],
  awarded: nobelPrizes[0].awardYear,
  age_at_award: subtract(
    to_number(nobelPrizes[0].awardYear),
    to_number(split(birth.date, `-`)[0])
  )
}' laureates.json
```

### Find Laureates Born Before 1900

```bash
jpx 'laureates[?to_number(split(birth.date, `-`)[0]) < `1900`].{
  name: knownName.en,
  born: birth.date
}' laureates.json
```

---

## Geographic Analysis

### Laureates by Birth Country

```bash
jpx 'laureates[*].birth.place.country.en' laureates.json | jpx -s 'group_by(@, &@) | @.{
  country: [0],
  count: length(@)
}'
```

Or using `frequencies`:

```bash
jpx 'frequencies(laureates[*].birth.place.country.en)' laureates.json
```

### Laureates by Continent

```bash
jpx 'laureates[*].{
  name: knownName.en,
  continent: birth.place.continent.en
}' laureates.json
```

### Extract Coordinates

```bash
jpx 'laureates[?birth.place.cityNow.latitude != null].{
  name: knownName.en,
  city: birth.place.city.en,
  lat: birth.place.cityNow.latitude,
  lon: birth.place.cityNow.longitude
}' laureates.json
```

---

## Prize Analysis

### Prize Amounts Over Time

```bash
jpx 'laureates[*].nobelPrizes[0].{
  year: awardYear,
  amount: prizeAmount,
  adjusted: prizeAmountAdjusted
} | sort_by(@, &year)' laureates.json
```

### Shared Prizes

The `portion` field shows how the prize was split:

```bash
jpx 'laureates[?nobelPrizes[0].portion != `1`].{
  name: knownName.en,
  portion: nobelPrizes[0].portion,
  year: nobelPrizes[0].awardYear
}' laureates.json
```

### Find Solo Winners

```bash
jpx 'laureates[?nobelPrizes[0].portion == `1`].{
  name: knownName.en,
  category: nobelPrizes[0].category.en,
  year: nobelPrizes[0].awardYear
}' laureates.json
```

---

## Affiliations

### Extract University Affiliations

```bash
jpx 'laureates[*].{
  name: knownName.en,
  university: nobelPrizes[0].affiliations[0].name.en,
  country: nobelPrizes[0].affiliations[0].country.en
}' laureates.json
```

### Find Stanford Affiliates

```bash
jpx 'laureates[?contains(to_string(nobelPrizes[*].affiliations[*].name.en), `Stanford`)].knownName.en' laureates.json
```

---

## Text Processing

### Search Motivations

Find laureates whose work involved "quantum":

```bash
jpx 'laureates[?contains(lower(to_string(nobelPrizes[*].motivation.en)), `quantum`)].{
  name: knownName.en,
  motivation: nobelPrizes[0].motivation.en
}' laureates.json
```

### Extract Key Terms from Motivations

```bash
jpx 'laureates[*].nobelPrizes[0].motivation.en' laureates.json
```

---

## Data Transformation

### Flatten for Export

```bash
jpx 'laureates[*].{
  id: id,
  name: knownName.en,
  gender: gender,
  birth_date: birth.date,
  birth_country: birth.place.country.en,
  death_date: death.date,
  prize_year: nobelPrizes[0].awardYear,
  prize_category: nobelPrizes[0].category.en,
  prize_portion: nobelPrizes[0].portion,
  motivation: nobelPrizes[0].motivation.en
}' laureates.json
```

### Export to CSV

```bash
jpx --csv 'laureates[*].{
  name: knownName.en,
  year: nobelPrizes[0].awardYear,
  category: nobelPrizes[0].category.en,
  country: birth.place.country.en
}' laureates.json
```

---

## Advanced Pipelines

### Summary by Category

```bash
jpx '{
  physics: length(laureates[?nobelPrizes[0].category.en == `Physics`]),
  chemistry: length(laureates[?nobelPrizes[0].category.en == `Chemistry`]),
  medicine: length(laureates[?contains(nobelPrizes[0].category.en, `Medicine`) || contains(nobelPrizes[0].category.en, `Physiology`)]),
  literature: length(laureates[?nobelPrizes[0].category.en == `Literature`]),
  peace: length(laureates[?nobelPrizes[0].category.en == `Peace`]),
  economics: length(laureates[?contains(nobelPrizes[0].category.en, `Economic`)])
}' laureates.json
```

### Gender Distribution by Decade

```bash
jpx 'laureates[*].{
  decade: concat(slice(nobelPrizes[0].awardYear, `0`, `3`), `0s`),
  gender: gender
}' laureates.json
```

### Youngest and Oldest Winners

```bash
jpx '{
  dataset: laureates[?birth.date != null] | [*].{
    name: knownName.en,
    age: subtract(to_number(nobelPrizes[0].awardYear), to_number(split(birth.date, `-`)[0]))
  },
  youngest: min_by(@.dataset, &age),
  oldest: max_by(@.dataset, &age)
}' laureates.json
```

---

## Try It Yourself

1. **Explore the API**:
   ```bash
   # All laureates (paginated)
   curl -s "https://api.nobelprize.org/2.1/laureates?limit=100&offset=0"
   
   # Filter by category
   curl -s "https://api.nobelprize.org/2.1/laureates?nobelPrizeCategory=lit"
   
   # Filter by year
   curl -s "https://api.nobelprize.org/2.1/nobelPrizes?nobelPrizeYear=2023"
   ```

2. **Category codes**: `phy` (Physics), `che` (Chemistry), `med` (Medicine), `lit` (Literature), `pea` (Peace), `eco` (Economics)

3. **API Documentation**: [Nobel Prize API](https://www.nobelprize.org/about/developer-zone-2/)

## Related Functions

- [`contains`](../functions/standard.md#contains), [`starts_with`](../functions/standard.md#starts_with) - Text matching
- [`split`](../functions/string.md#split), [`lower`](../functions/string.md#lower) - String manipulation
- [`group_by`](../functions/array.md#group_by), [`frequencies`](../functions/array.md#frequencies) - Aggregation
- [`to_number`](../functions/standard.md#to_number), [`subtract`](../functions/math.md#subtract) - Calculations
