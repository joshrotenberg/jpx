# Datetime Functions (28)

## `business_days_between`

**Signature:** `number, number -> number`

Count business days (weekdays) between two timestamps

```
business_days_between(`1704067200`, `1705276800`) -> 10
```
_Count weekdays_

```
business_days_between(start_ts, end_ts) -> count
```
_Between two timestamps_

---

## `date_add`

**Signature:** `number, number, string -> number`

Add time to timestamp

```
date_add(`0`, `1`, 'days') -> 86400
```
_Add 1 day_

```
date_add(`0`, `2`, 'hours') -> 7200
```
_Add 2 hours_

---

## `date_diff`

**Signature:** `number, number, string -> number`

Difference between timestamps

```
date_diff(`86400`, `0`, 'days') -> 1
```
_Diff in days_

```
date_diff(`7200`, `0`, 'hours') -> 2
```
_Diff in hours_

---

## `duration_since`

**Signature:** `number|string -> object`

Get detailed duration object from timestamp to now

```
duration_since(`1702396800`) -> {days: 1, hours: 0, ...}
```
_From timestamp_

```
duration_since('2023-01-01') -> {days: N, ...}
```
_From date string_

---

## `end_of_day`

**Signature:** `number|string -> string`

Get ISO 8601 string for end of day (23:59:59)

```
end_of_day('2023-12-13T15:30:00Z') -> \"2023-12-13T23:59:59Z\"
```
_From ISO string_

```
end_of_day(`1702483200`) -> end of that day
```
_From timestamp_

---

## `epoch_ms`

**Signature:** `-> number`

Current Unix timestamp in milliseconds (alias for now_ms)

```
epoch_ms() -> 1702483200000
```
_Current time in ms_

```
epoch_ms() / `1000` -> seconds
```
_Convert to seconds_

---

## `format_date`

**Signature:** `number, string -> string`

Format timestamp to string

```
format_date(`1705276800`, '%Y-%m-%d') -> \"2024-01-15\"
```
_ISO date format_

```
format_date(ts, '%B %d, %Y') -> \"January 15, 2024\"
```
_Long date format_

---

## `from_epoch`

**Signature:** `number -> string`

Convert Unix timestamp (seconds) to ISO 8601 string

```
from_epoch(`1702483200`) -> \"2023-12-13T16:00:00Z\"
```
_Convert seconds_

```
from_epoch(`0`) -> \"1970-01-01T00:00:00Z\"
```
_Unix epoch_

---

## `from_epoch_ms`

**Signature:** `number -> string`

Convert Unix timestamp (milliseconds) to ISO 8601 string

```
from_epoch_ms(`1702483200000`) -> \"2023-12-13T16:00:00Z\"
```
_From milliseconds_

```
from_epoch_ms(`0`) -> \"1970-01-01T00:00:00Z\"
```
_Unix epoch_

---

## `is_after`

**Signature:** `number|string, number|string -> boolean`

Check if first date is after second date (accepts timestamps or date strings)

```
is_after('2024-07-15', '2024-01-01') -> true
```
_String comparison_

```
is_after(`1705276800`, `1704067200`) -> true
```
_Timestamp comparison_

---

## `is_before`

**Signature:** `number|string, number|string -> boolean`

Check if first date is before second date (accepts timestamps or date strings)

```
is_before('2024-01-01', '2024-07-15') -> true
```
_String comparison_

```
is_before(`1704067200`, `1705276800`) -> true
```
_Timestamp comparison_

---

## `is_between`

**Signature:** `number|string, number|string, number|string -> boolean`

Check if date is between start and end (inclusive, accepts timestamps or date strings)

```
is_between('2024-06-15', '2024-01-01', '2024-12-31') -> true
```
_Within range_

```
is_between('2023-06-15', '2024-01-01', '2024-12-31') -> false
```
_Before range_

---

## `is_same_day`

**Signature:** `number|string, number|string -> boolean`

Check if two timestamps/dates are on the same day

```
is_same_day('2023-12-13T10:00:00Z', '2023-12-13T23:00:00Z') -> true
```
_Same day, different time_

```
is_same_day('2023-12-13', '2023-12-14') -> false
```
_Different days_

---

## `is_weekday`

**Signature:** `number -> boolean`

Check if timestamp falls on weekday (Monday-Friday)

```
is_weekday(`1705276800`) -> true
```
_Monday is weekday_

```
is_weekday(now()) -> true/false
```
_Check current day_

---

## `is_weekend`

**Signature:** `number -> boolean`

Check if timestamp falls on weekend (Saturday or Sunday)

```
is_weekend(`1705104000`) -> true
```
_Saturday is weekend_

```
is_weekend(now()) -> true/false
```
_Check current day_

---

## `parse_date`

**Signature:** `string, string? -> number`

Parse date string to timestamp

```
parse_date('2024-01-15', '%Y-%m-%d') -> 1705276800
```
_ISO date format_

```
parse_date('01/15/2024', '%m/%d/%Y') -> timestamp
```
_US date format_

---

## `parse_datetime`

**Signature:** `string -> string | null`

Parse structured date/time string to ISO 8601 UTC

```
parse_datetime("2024-01-15") -> "2024-01-15T00:00:00Z"
```
_ISO date_

```
parse_datetime("2024-01-15T10:30:00Z") -> "2024-01-15T10:30:00Z"
```
_ISO datetime_

---

## `parse_natural_date`

**Signature:** `string -> string | null`

Parse natural language date expression to ISO 8601 UTC (relative to now)

```
parse_natural_date("yesterday") -> "2026-01-25T00:00:00Z"
```
_Yesterday_

```
parse_natural_date("tomorrow") -> "2026-01-27T00:00:00Z"
```
_Tomorrow_

---

## `quarter`

**Signature:** `number -> number`

Get quarter of year (1-4) from timestamp

```
quarter(`1713139200`) -> 2
```
_April is Q2_

```
quarter(`1704067200`) -> 1
```
_January is Q1_

---

## `relative_time`

**Signature:** `number -> string`

Human-readable relative time from timestamp

```
relative_time(now() - 3600) -> \"1 hour ago\"
```
_One hour ago_

```
relative_time(now() - 60) -> \"1 minute ago\"
```
_One minute ago_

---

## `start_of_day`

**Signature:** `number|string -> string`

Get ISO 8601 string for start of day (00:00:00)

```
start_of_day('2023-12-13T15:30:00Z') -> \"2023-12-13T00:00:00Z\"
```
_From ISO string_

```
start_of_day(`1702483200`) -> start of that day
```
_From timestamp_

---

## `start_of_month`

**Signature:** `number|string -> string`

Get ISO 8601 string for start of month

```
start_of_month('2023-12-13T15:30:00Z') -> \"2023-12-01T00:00:00Z\"
```
_From ISO string_

```
start_of_month(now()) -> first of month
```
_Current month start_

---

## `start_of_week`

**Signature:** `number|string -> string`

Get ISO 8601 string for start of week (Monday 00:00:00)

```
start_of_week('2023-12-13T15:30:00Z') -> \"2023-12-11T00:00:00Z\"
```
_Wednesday to Monday_

```
start_of_week(now()) -> this Monday
```
_Current week start_

---

## `start_of_year`

**Signature:** `number|string -> string`

Get ISO 8601 string for start of year

```
start_of_year('2023-12-13T15:30:00Z') -> \"2023-01-01T00:00:00Z\"
```
_From ISO string_

```
start_of_year(now()) -> Jan 1st
```
_Current year start_

---

## `time_ago`

**Signature:** `number|string -> string`

Human-readable time since date (accepts timestamps or date strings)

```
time_ago('2020-01-01') -> \"4 years ago\"
```
_From date string_

```
time_ago(now() - 3600) -> \"1 hour ago\"
```
_One hour ago_

---

## `timezone_convert`

**Signature:** `string, string, string -> string`

Convert timestamp between timezones (IANA timezone names)

```
timezone_convert('2024-01-15T10:00:00', 'America/New_York', 'Europe/London') -> \"2024-01-15T15:00:00\"
```
_NY to London_

```
timezone_convert(time, 'UTC', 'America/Los_Angeles') -> PST time
```
_UTC to Pacific_

---

## `to_epoch`

**Signature:** `number|string -> number`

Convert date string or timestamp to Unix timestamp (seconds)

```
to_epoch('2023-12-13T16:00:00Z') -> 1702483200
```
_From ISO string_

```
to_epoch('2024-01-01') -> timestamp
```
_From date only_

---

## `to_epoch_ms`

**Signature:** `number|string -> number`

Convert date string or timestamp to Unix timestamp (milliseconds)

```
to_epoch_ms('2023-12-13T16:00:00Z') -> 1702483200000
```
_From ISO string_

```
to_epoch_ms('2024-01-01') -> timestamp_ms
```
_From date only_

---

