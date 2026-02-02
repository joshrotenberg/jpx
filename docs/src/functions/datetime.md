# Date/Time Functions

Functions for working with dates and times: parsing, formatting, arithmetic, and timezone handling.

## Summary

| Function | Signature | Description |
|----------|-----------|-------------|
| [`business_days_between`](#business-days-between) | `number, number -> number` | Count business days (weekdays) between two timestamps |
| [`date_add`](#date-add) | `number, number, string -> number` | Add time to timestamp |
| [`date_diff`](#date-diff) | `number, number, string -> number` | Difference between timestamps |
| [`duration_since`](#duration-since) | `number\|string -> object` | Get detailed duration object from timestamp to now |
| [`end_of_day`](#end-of-day) | `number\|string -> string` | Get ISO 8601 string for end of day (23:59:59) |
| [`epoch_ms`](#epoch-ms) | `-> number` | Current Unix timestamp in milliseconds (alias for now_ms) |
| [`format_date`](#format-date) | `number, string -> string` | Format timestamp to string |
| [`from_epoch`](#from-epoch) | `number -> string` | Convert Unix timestamp (seconds) to ISO 8601 string |
| [`from_epoch_ms`](#from-epoch-ms) | `number -> string` | Convert Unix timestamp (milliseconds) to ISO 8601 string |
| [`is_after`](#is-after) | `number\|string, number\|string -> boolean` | Check if first date is after second date (accepts timestamps or date strings) |
| [`is_before`](#is-before) | `number\|string, number\|string -> boolean` | Check if first date is before second date (accepts timestamps or date strings) |
| [`is_between`](#is-between) | `number\|string, number\|string, number\|string -> boolean` | Check if date is between start and end (inclusive, accepts timestamps or date strings) |
| [`is_same_day`](#is-same-day) | `number\|string, number\|string -> boolean` | Check if two timestamps/dates are on the same day |
| [`is_weekday`](#is-weekday) | `number -> boolean` | Check if timestamp falls on weekday (Monday-Friday) |
| [`is_weekend`](#is-weekend) | `number -> boolean` | Check if timestamp falls on weekend (Saturday or Sunday) |
| [`parse_date`](#parse-date) | `string, string? -> number` | Parse date string to timestamp |
| [`quarter`](#quarter) | `number -> number` | Get quarter of year (1-4) from timestamp |
| [`relative_time`](#relative-time) | `number -> string` | Human-readable relative time from timestamp |
| [`start_of_day`](#start-of-day) | `number\|string -> string` | Get ISO 8601 string for start of day (00:00:00) |
| [`start_of_month`](#start-of-month) | `number\|string -> string` | Get ISO 8601 string for start of month |
| [`start_of_week`](#start-of-week) | `number\|string -> string` | Get ISO 8601 string for start of week (Monday 00:00:00) |
| [`start_of_year`](#start-of-year) | `number\|string -> string` | Get ISO 8601 string for start of year |
| [`time_ago`](#time-ago) | `number\|string -> string` | Human-readable time since date (accepts timestamps or date strings) |
| [`timezone_convert`](#timezone-convert) | `string, string, string -> string` | Convert timestamp between timezones (IANA timezone names) |
| [`to_epoch`](#to-epoch) | `number\|string -> number` | Convert date string or timestamp to Unix timestamp (seconds) |
| [`to_epoch_ms`](#to-epoch-ms) | `number\|string -> number` | Convert date string or timestamp to Unix timestamp (milliseconds) |

## Functions

### business_days_between

Count business days (weekdays) between two timestamps

**Signature:** `number, number -> number`

**Examples:**

```text
# Count weekdays
business_days_between(`1704067200`, `1705276800`) -> 10
# Between two timestamps
business_days_between(start_ts, end_ts) -> count
# One week = 5 days
business_days_between(`0`, `604800`) -> 5
# Same day
business_days_between(`0`, `0`) -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'business_days_between(`1704067200`, `1705276800`)'
```

### date_add

Add time to timestamp

**Signature:** `number, number, string -> number`

**Examples:**

```text
# Add 1 day
date_add(`0`, `1`, 'days') -> 86400
# Add 2 hours
date_add(`0`, `2`, 'hours') -> 7200
# Add 1 week
date_add(timestamp, `7`, 'days') -> next week
# Add 30 minutes
date_add(`0`, `30`, 'minutes') -> 1800
```

**CLI Usage:**

```bash
echo '{}' | jpx 'date_add(`0`, `1`, `"days"`)'
```

### date_diff

Difference between timestamps

**Signature:** `number, number, string -> number`

**Examples:**

```text
# Diff in days
date_diff(`86400`, `0`, 'days') -> 1
# Diff in hours
date_diff(`7200`, `0`, 'hours') -> 2
# Diff in minutes
date_diff(end_ts, start_ts, 'minutes') -> minutes
# Diff in weeks
date_diff(`604800`, `0`, 'weeks') -> 1
```

**CLI Usage:**

```bash
echo '{}' | jpx 'date_diff(`86400`, `0`, `"days"`)'
```

### duration_since

Get detailed duration object from timestamp to now

**Signature:** `number|string -> object`

**Examples:**

```text
# From timestamp
duration_since(`1702396800`) -> {days: 1, hours: 0, ...}
# From date string
duration_since('2023-01-01') -> {days: N, ...}
# Time since creation
duration_since(created_at) -> elapsed time
# One hour ago
duration_since(now() - 3600) -> {hours: 1, ...}
```

**CLI Usage:**

```bash
echo '{}' | jpx 'duration_since(`1702396800`)'
```

### end_of_day

Get ISO 8601 string for end of day (23:59:59)

**Signature:** `number|string -> string`

**Examples:**

```text
# From ISO string
end_of_day('2023-12-13T15:30:00Z') -> \"2023-12-13T23:59:59Z\"
# From timestamp
end_of_day(`1702483200`) -> end of that day
# From date only
end_of_day('2024-01-01') -> last second of day
# End of today
end_of_day(now()) -> today end
```

**CLI Usage:**

```bash
echo '{}' | jpx 'end_of_day(`"2023-12-13T15:30:00Z"`)'
```

### epoch_ms

Current Unix timestamp in milliseconds (alias for now_ms)

**Signature:** `-> number`

**Examples:**

```text
# Current time in ms
epoch_ms() -> 1702483200000
# Convert to seconds
epoch_ms() / `1000` -> seconds
# Calculate duration
epoch_ms() - start_ms -> elapsed
```

**CLI Usage:**

```bash
echo '{}' | jpx 'epoch_ms()'
```

### format_date

Format timestamp to string

**Signature:** `number, string -> string`

**Examples:**

```text
# ISO date format
format_date(`1705276800`, '%Y-%m-%d') -> \"2024-01-15\"
# Long date format
format_date(ts, '%B %d, %Y') -> \"January 15, 2024\"
# Time only
format_date(ts, '%H:%M:%S') -> \"10:30:00\"
# Full ISO 8601
format_date(ts, '%Y-%m-%dT%H:%M:%SZ') -> ISO string
```

**CLI Usage:**

```bash
echo '{}' | jpx 'format_date(`1705276800`, `"%Y-%m-%d"`)'
```

### from_epoch

Convert Unix timestamp (seconds) to ISO 8601 string

**Signature:** `number -> string`

**Examples:**

```text
# Convert seconds
from_epoch(`1702483200`) -> \"2023-12-13T16:00:00Z\"
# Unix epoch
from_epoch(`0`) -> \"1970-01-01T00:00:00Z\"
# Convert field
from_epoch(created_at) -> ISO string
# Current time
from_epoch(now()) -> current ISO
```

**CLI Usage:**

```bash
echo '{}' | jpx 'from_epoch(`1702483200`)'
```

### from_epoch_ms

Convert Unix timestamp (milliseconds) to ISO 8601 string

**Signature:** `number -> string`

**Examples:**

```text
# From milliseconds
from_epoch_ms(`1702483200000`) -> \"2023-12-13T16:00:00Z\"
# Unix epoch
from_epoch_ms(`0`) -> \"1970-01-01T00:00:00Z\"
# JS timestamp
from_epoch_ms(js_timestamp) -> ISO
# Current time
from_epoch_ms(epoch_ms()) -> current
```

**CLI Usage:**

```bash
echo '{}' | jpx 'from_epoch_ms(`1702483200000`)'
```

### is_after

Check if first date is after second date (accepts timestamps or date strings)

**Signature:** `number|string, number|string -> boolean`

**Examples:**

```text
# String comparison
is_after('2024-07-15', '2024-01-01') -> true
# Timestamp comparison
is_after(`1705276800`, `1704067200`) -> true
# Field comparison
is_after(end_date, start_date) -> true/false
# Same date
is_after('2024-01-01', '2024-01-01') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_after(`"2024-07-15"`, `"2024-01-01"`)'
```

### is_before

Check if first date is before second date (accepts timestamps or date strings)

**Signature:** `number|string, number|string -> boolean`

**Examples:**

```text
# String comparison
is_before('2024-01-01', '2024-07-15') -> true
# Timestamp comparison
is_before(`1704067200`, `1705276800`) -> true
# Field comparison
is_before(start_date, end_date) -> true/false
# Same date
is_before('2024-01-01', '2024-01-01') -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_before(`"2024-01-01"`, `"2024-07-15"`)'
```

### is_between

Check if date is between start and end (inclusive, accepts timestamps or date strings)

**Signature:** `number|string, number|string, number|string -> boolean`

**Examples:**

```text
# Within range
is_between('2024-06-15', '2024-01-01', '2024-12-31') -> true
# Before range
is_between('2023-06-15', '2024-01-01', '2024-12-31') -> false
# Check event date
is_between(event_date, start, end) -> true/false
# Inclusive start
is_between('2024-01-01', '2024-01-01', '2024-12-31') -> true
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_between(`"2024-06-15"`, `"2024-01-01"`, `"2024-12-31"`)'
```

### is_same_day

Check if two timestamps/dates are on the same day

**Signature:** `number|string, number|string -> boolean`

**Examples:**

```text
# Same day, different time
is_same_day('2023-12-13T10:00:00Z', '2023-12-13T23:00:00Z') -> true
# Different days
is_same_day('2023-12-13', '2023-12-14') -> false
# Compare fields
is_same_day(created_at, updated_at) -> true/false
# Today check
is_same_day(now(), event_time) -> true/false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_same_day(`"2023-12-13T10:00:00Z"`, `"2023-12-13T23:00:00Z"`)'
```

### is_weekday

Check if timestamp falls on weekday (Monday-Friday)

**Signature:** `number -> boolean`

**Examples:**

```text
# Monday is weekday
is_weekday(`1705276800`) -> true
# Check current day
is_weekday(now()) -> true/false
# Check event day
is_weekday(event_timestamp) -> true/false
# Saturday is not weekday
is_weekday(`1705104000`) -> false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_weekday(`1705276800`)'
```

### is_weekend

Check if timestamp falls on weekend (Saturday or Sunday)

**Signature:** `number -> boolean`

**Examples:**

```text
# Saturday is weekend
is_weekend(`1705104000`) -> true
# Check current day
is_weekend(now()) -> true/false
# Monday is not weekend
is_weekend(`1705276800`) -> false
# Check event day
is_weekend(event_timestamp) -> true/false
```

**CLI Usage:**

```bash
echo '{}' | jpx 'is_weekend(`1705104000`)'
```

### parse_date

Parse date string to timestamp

**Signature:** `string, string? -> number`

**Examples:**

```text
# ISO date format
parse_date('2024-01-15', '%Y-%m-%d') -> 1705276800
# US date format
parse_date('01/15/2024', '%m/%d/%Y') -> timestamp
# Named month
parse_date('15-Jan-2024', '%d-%b-%Y') -> timestamp
# With time
parse_date('2024-01-15T10:30:00', '%Y-%m-%dT%H:%M:%S') -> timestamp
```

**CLI Usage:**

```bash
echo '{}' | jpx 'parse_date(`"2024-01-15"`, `"%Y-%m-%d"`)'
```

### quarter

Get quarter of year (1-4) from timestamp

**Signature:** `number -> number`

**Examples:**

```text
# April is Q2
quarter(`1713139200`) -> 2
# January is Q1
quarter(`1704067200`) -> 1
# July is Q3
quarter(`1719792000`) -> 3
# November is Q4
quarter(`1730419200`) -> 4
```

**CLI Usage:**

```bash
echo '{}' | jpx 'quarter(`1713139200`)'
```

### relative_time

Human-readable relative time from timestamp

**Signature:** `number -> string`

**Examples:**

```text
# One hour ago
relative_time(now() - 3600) -> \"1 hour ago\"
# One minute ago
relative_time(now() - 60) -> \"1 minute ago\"
# One day ago
relative_time(now() - 86400) -> \"1 day ago\"
# From field
relative_time(created_at) -> human readable
```

**CLI Usage:**

```bash
echo '{}' | jpx 'relative_time(now() - 3600)'
```

### start_of_day

Get ISO 8601 string for start of day (00:00:00)

**Signature:** `number|string -> string`

**Examples:**

```text
# From ISO string
start_of_day('2023-12-13T15:30:00Z') -> \"2023-12-13T00:00:00Z\"
# From timestamp
start_of_day(`1702483200`) -> start of that day
# Today start
start_of_day(now()) -> today midnight
# From date only
start_of_day('2024-01-15') -> midnight
```

**CLI Usage:**

```bash
echo '{}' | jpx 'start_of_day(`"2023-12-13T15:30:00Z"`)'
```

### start_of_month

Get ISO 8601 string for start of month

**Signature:** `number|string -> string`

**Examples:**

```text
# From ISO string
start_of_month('2023-12-13T15:30:00Z') -> \"2023-12-01T00:00:00Z\"
# Current month start
start_of_month(now()) -> first of month
# From timestamp
start_of_month(`1702483200`) -> month start
# March start
start_of_month('2024-03-15') -> \"2024-03-01T00:00:00Z\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'start_of_month(`"2023-12-13T15:30:00Z"`)'
```

### start_of_week

Get ISO 8601 string for start of week (Monday 00:00:00)

**Signature:** `number|string -> string`

**Examples:**

```text
# Wednesday to Monday
start_of_week('2023-12-13T15:30:00Z') -> \"2023-12-11T00:00:00Z\"
# Current week start
start_of_week(now()) -> this Monday
# From timestamp
start_of_week(`1702483200`) -> week start
# Monday stays Monday
start_of_week('2024-01-15') -> \"2024-01-15T00:00:00Z\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'start_of_week(`"2023-12-13T15:30:00Z"`)'
```

### start_of_year

Get ISO 8601 string for start of year

**Signature:** `number|string -> string`

**Examples:**

```text
# From ISO string
start_of_year('2023-12-13T15:30:00Z') -> \"2023-01-01T00:00:00Z\"
# Current year start
start_of_year(now()) -> Jan 1st
# From timestamp
start_of_year(`1702483200`) -> year start
# Mid-year to Jan 1
start_of_year('2024-06-15') -> \"2024-01-01T00:00:00Z\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'start_of_year(`"2023-12-13T15:30:00Z"`)'
```

### time_ago

Human-readable time since date (accepts timestamps or date strings)

**Signature:** `number|string -> string`

**Examples:**

```text
# From date string
time_ago('2020-01-01') -> \"4 years ago\"
# One hour ago
time_ago(now() - 3600) -> \"1 hour ago\"
# From field
time_ago(created_at) -> human readable
# Months ago
time_ago('2023-12-01') -> \"X months ago\"
```

**CLI Usage:**

```bash
echo '{}' | jpx 'time_ago(`"2020-01-01"`)'
```

### timezone_convert

Convert timestamp between timezones (IANA timezone names)

**Signature:** `string, string, string -> string`

**Examples:**

```text
# NY to London
timezone_convert('2024-01-15T10:00:00', 'America/New_York', 'Europe/London') -> \"2024-01-15T15:00:00\"
# UTC to Pacific
timezone_convert(time, 'UTC', 'America/Los_Angeles') -> PST time
# Tokyo to UTC
timezone_convert(time, 'Asia/Tokyo', 'UTC') -> UTC time
# Paris to Chicago
timezone_convert(time, 'Europe/Paris', 'America/Chicago') -> CST time
```

**CLI Usage:**

```bash
echo '{}' | jpx 'timezone_convert(`"2024-01-15T10:00:00"`, `"America/New_York"`, `"Europe/London"`)'
```

### to_epoch

Convert date string or timestamp to Unix timestamp (seconds)

**Signature:** `number|string -> number`

**Examples:**

```text
# From ISO string
to_epoch('2023-12-13T16:00:00Z') -> 1702483200
# From date only
to_epoch('2024-01-01') -> timestamp
# Convert field
to_epoch(date_field) -> seconds
# Unix epoch
to_epoch('1970-01-01T00:00:00Z') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_epoch(`"2023-12-13T16:00:00Z"`)'
```

### to_epoch_ms

Convert date string or timestamp to Unix timestamp (milliseconds)

**Signature:** `number|string -> number`

**Examples:**

```text
# From ISO string
to_epoch_ms('2023-12-13T16:00:00Z') -> 1702483200000
# From date only
to_epoch_ms('2024-01-01') -> timestamp_ms
# Convert field
to_epoch_ms(date_field) -> milliseconds
# Unix epoch
to_epoch_ms('1970-01-01T00:00:00Z') -> 0
```

**CLI Usage:**

```bash
echo '{}' | jpx 'to_epoch_ms(`"2023-12-13T16:00:00Z"`)'
```

