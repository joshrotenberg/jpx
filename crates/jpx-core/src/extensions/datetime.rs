//! Date and time functions.

use std::collections::HashSet;

use chrono::{DateTime, Datelike, NaiveDateTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde_json::Value;

use crate::functions::{Function, custom_error, number_value};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Register datetime functions filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "now", enabled, Box::new(NowFn::new()));
    register_if_enabled(runtime, "now_millis", enabled, Box::new(NowMillisFn::new()));
    register_if_enabled(runtime, "parse_date", enabled, Box::new(ParseDateFn::new()));
    register_if_enabled(
        runtime,
        "format_date",
        enabled,
        Box::new(FormatDateFn::new()),
    );
    register_if_enabled(runtime, "date_add", enabled, Box::new(DateAddFn::new()));
    register_if_enabled(runtime, "date_diff", enabled, Box::new(DateDiffFn::new()));
    register_if_enabled(
        runtime,
        "timezone_convert",
        enabled,
        Box::new(TimezoneConvertFn::new()),
    );
    register_if_enabled(runtime, "is_weekend", enabled, Box::new(IsWeekendFn::new()));
    register_if_enabled(runtime, "is_weekday", enabled, Box::new(IsWeekdayFn::new()));
    register_if_enabled(
        runtime,
        "business_days_between",
        enabled,
        Box::new(BusinessDaysBetweenFn::new()),
    );
    register_if_enabled(
        runtime,
        "relative_time",
        enabled,
        Box::new(RelativeTimeFn::new()),
    );
    register_if_enabled(runtime, "quarter", enabled, Box::new(QuarterFn::new()));
    register_if_enabled(runtime, "is_after", enabled, Box::new(IsAfterFn::new()));
    register_if_enabled(runtime, "is_before", enabled, Box::new(IsBeforeFn::new()));
    register_if_enabled(runtime, "is_between", enabled, Box::new(IsBetweenFn::new()));
    register_if_enabled(runtime, "time_ago", enabled, Box::new(TimeAgoFn::new()));
    register_if_enabled(runtime, "from_epoch", enabled, Box::new(FromEpochFn::new()));
    register_if_enabled(
        runtime,
        "from_epoch_ms",
        enabled,
        Box::new(FromEpochMsFn::new()),
    );
    register_if_enabled(runtime, "to_epoch", enabled, Box::new(ToEpochFn::new()));
    register_if_enabled(
        runtime,
        "to_epoch_ms",
        enabled,
        Box::new(ToEpochMsFn::new()),
    );
    register_if_enabled(
        runtime,
        "duration_since",
        enabled,
        Box::new(DurationSinceFn::new()),
    );
    register_if_enabled(
        runtime,
        "start_of_day",
        enabled,
        Box::new(StartOfDayFn::new()),
    );
    register_if_enabled(runtime, "end_of_day", enabled, Box::new(EndOfDayFn::new()));
    register_if_enabled(
        runtime,
        "start_of_week",
        enabled,
        Box::new(StartOfWeekFn::new()),
    );
    register_if_enabled(
        runtime,
        "start_of_month",
        enabled,
        Box::new(StartOfMonthFn::new()),
    );
    register_if_enabled(
        runtime,
        "start_of_year",
        enabled,
        Box::new(StartOfYearFn::new()),
    );
    register_if_enabled(
        runtime,
        "is_same_day",
        enabled,
        Box::new(IsSameDayFn::new()),
    );
    // epoch_ms is an alias for now_millis (common name)
    register_if_enabled(runtime, "epoch_ms", enabled, Box::new(NowMillisFn::new()));
    register_if_enabled(
        runtime,
        "parse_datetime",
        enabled,
        Box::new(ParseDatetimeFn::new()),
    );
    register_if_enabled(
        runtime,
        "parse_natural_date",
        enabled,
        Box::new(ParseNaturalDateFn::new()),
    );
}

// now() -> number
defn!(NowFn, vec![], None);

impl Function for NowFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let ts = Utc::now().timestamp();
        Ok(number_value(ts as f64))
    }
}

// now_millis() -> number
defn!(NowMillisFn, vec![], None);

impl Function for NowMillisFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let ts = Utc::now().timestamp_millis();
        Ok(number_value(ts as f64))
    }
}

// parse_date(string, format?) -> number | null
defn!(ParseDateFn, vec![arg!(string)], Some(arg!(string)));

impl Function for ParseDateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_str().unwrap();

        if args.len() > 1 {
            // Custom format provided
            let format = args[1].as_str().unwrap();
            match NaiveDateTime::parse_from_str(s, format) {
                Ok(dt) => Ok(number_value(dt.and_utc().timestamp() as f64)),
                Err(_) => Ok(Value::Null),
            }
        } else {
            // Try common formats
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Ok(number_value(dt.timestamp() as f64));
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                return Ok(number_value(dt.and_utc().timestamp() as f64));
            }
            if let Ok(dt) =
                NaiveDateTime::parse_from_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S")
            {
                return Ok(number_value(dt.and_utc().timestamp() as f64));
            }
            Ok(Value::Null)
        }
    }
}

// format_date(timestamp, format) -> string
defn!(FormatDateFn, vec![arg!(number), arg!(string)], None);

impl Function for FormatDateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap();
        let format = args[1].as_str().unwrap();

        let dt = Utc.timestamp_opt(ts as i64, 0);
        match dt {
            chrono::LocalResult::Single(dt) => Ok(Value::String(dt.format(format).to_string())),
            _ => Ok(Value::Null),
        }
    }
}

// date_add(timestamp, amount, unit) -> number
defn!(
    DateAddFn,
    vec![arg!(number), arg!(number), arg!(string)],
    None
);

impl Function for DateAddFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap();
        let amount = args[1].as_f64().unwrap();
        let unit = args[2].as_str().unwrap();

        let duration = match unit.to_lowercase().as_str() {
            "seconds" | "second" | "s" => TimeDelta::seconds(amount as i64),
            "minutes" | "minute" | "m" => TimeDelta::minutes(amount as i64),
            "hours" | "hour" | "h" => TimeDelta::hours(amount as i64),
            "days" | "day" | "d" => TimeDelta::days(amount as i64),
            "weeks" | "week" | "w" => TimeDelta::weeks(amount as i64),
            _ => return Err(custom_error(ctx, &format!("invalid time unit: {}", unit))),
        };

        let dt = Utc.timestamp_opt(ts as i64, 0);
        match dt {
            chrono::LocalResult::Single(dt) => {
                let new_dt = dt + duration;
                Ok(number_value(new_dt.timestamp() as f64))
            }
            _ => Ok(Value::Null),
        }
    }
}

// date_diff(ts1, ts2, unit) -> number
defn!(
    DateDiffFn,
    vec![arg!(number), arg!(number), arg!(string)],
    None
);

impl Function for DateDiffFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts1 = args[0].as_f64().unwrap();
        let ts2 = args[1].as_f64().unwrap();
        let unit = args[2].as_str().unwrap();

        let diff_seconds = (ts1 - ts2) as i64;

        let result = match unit.to_lowercase().as_str() {
            "seconds" | "second" | "s" => diff_seconds as f64,
            "minutes" | "minute" | "m" => diff_seconds as f64 / 60.0,
            "hours" | "hour" | "h" => diff_seconds as f64 / 3600.0,
            "days" | "day" | "d" => diff_seconds as f64 / 86400.0,
            "weeks" | "week" | "w" => diff_seconds as f64 / 604800.0,
            _ => return Err(custom_error(ctx, &format!("invalid time unit: {}", unit))),
        };

        Ok(number_value(result))
    }
}

// timezone_convert(timestamp, from_tz, to_tz) -> string
defn!(
    TimezoneConvertFn,
    vec![arg!(string), arg!(string), arg!(string)],
    None
);

impl Function for TimezoneConvertFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let timestamp_str = args[0].as_str().unwrap();
        let from_tz_str = args[1].as_str().unwrap();
        let to_tz_str = args[2].as_str().unwrap();

        // Parse timezone strings
        let from_tz: Tz = from_tz_str
            .parse()
            .map_err(|_| custom_error(ctx, &format!("invalid timezone: {}", from_tz_str)))?;
        let to_tz: Tz = to_tz_str
            .parse()
            .map_err(|_| custom_error(ctx, &format!("invalid timezone: {}", to_tz_str)))?;

        // Parse the input timestamp (try multiple formats)
        let naive_dt =
            if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S") {
                dt
            } else if let Ok(dt) = NaiveDateTime::parse_from_str(
                &format!("{}T00:00:00", timestamp_str),
                "%Y-%m-%dT%H:%M:%S",
            ) {
                dt
            } else {
                return Err(custom_error(
                    ctx,
                    &format!("invalid timestamp format: {}", timestamp_str),
                ));
            };

        // Interpret the naive datetime in the source timezone
        let from_dt = from_tz
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| custom_error(ctx, "ambiguous or invalid local time"))?;

        // Convert to target timezone
        let to_dt = from_dt.with_timezone(&to_tz);

        // Format as ISO string without timezone suffix
        Ok(Value::String(to_dt.format("%Y-%m-%dT%H:%M:%S").to_string()))
    }
}

// is_weekend(timestamp) -> boolean
defn!(IsWeekendFn, vec![arg!(number)], None);

impl Function for IsWeekendFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap();
        let dt = Utc.timestamp_opt(ts as i64, 0);

        match dt {
            chrono::LocalResult::Single(dt) => {
                let weekday = dt.weekday();
                let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
                Ok(Value::Bool(is_weekend))
            }
            _ => Ok(Value::Null),
        }
    }
}

// is_weekday(timestamp) -> boolean
defn!(IsWeekdayFn, vec![arg!(number)], None);

impl Function for IsWeekdayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap();
        let dt = Utc.timestamp_opt(ts as i64, 0);

        match dt {
            chrono::LocalResult::Single(dt) => {
                let weekday = dt.weekday();
                let is_weekday = weekday != Weekday::Sat && weekday != Weekday::Sun;
                Ok(Value::Bool(is_weekday))
            }
            _ => Ok(Value::Null),
        }
    }
}

// business_days_between(ts1, ts2) -> number
defn!(
    BusinessDaysBetweenFn,
    vec![arg!(number), arg!(number)],
    None
);

impl Function for BusinessDaysBetweenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts1 = args[0].as_f64().unwrap() as i64;
        let ts2 = args[1].as_f64().unwrap() as i64;

        let dt1 = match Utc.timestamp_opt(ts1, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return Ok(Value::Null),
        };
        let dt2 = match Utc.timestamp_opt(ts2, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return Ok(Value::Null),
        };

        // Ensure we iterate from earlier to later date
        let (start, end) = if dt1 <= dt2 {
            (dt1.date_naive(), dt2.date_naive())
        } else {
            (dt2.date_naive(), dt1.date_naive())
        };

        let mut count = 0i64;
        let mut current = start;

        while current < end {
            let weekday = current.weekday();
            if weekday != Weekday::Sat && weekday != Weekday::Sun {
                count += 1;
            }
            current = current.succ_opt().unwrap_or(current);
        }

        // If original order was reversed, return negative count
        let result = if ts1 > ts2 { -count } else { count };

        Ok(number_value(result as f64))
    }
}

// relative_time(timestamp) -> string
defn!(RelativeTimeFn, vec![arg!(number)], None);

impl Function for RelativeTimeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap() as i64;
        let now = Utc::now().timestamp();
        let diff = ts - now;

        let (abs_diff, is_future) = if diff >= 0 {
            (diff, true)
        } else {
            (-diff, false)
        };

        // Determine the unit and value
        let (value, unit_singular, unit_plural) = if abs_diff < 60 {
            (abs_diff, "second", "seconds")
        } else if abs_diff < 3600 {
            (abs_diff / 60, "minute", "minutes")
        } else if abs_diff < 86400 {
            (abs_diff / 3600, "hour", "hours")
        } else if abs_diff < 2592000 {
            (abs_diff / 86400, "day", "days")
        } else if abs_diff < 31536000 {
            (abs_diff / 2592000, "month", "months")
        } else {
            (abs_diff / 31536000, "year", "years")
        };

        let unit = if value == 1 {
            unit_singular
        } else {
            unit_plural
        };
        let result = if is_future {
            format!("in {} {}", value, unit)
        } else {
            format!("{} {} ago", value, unit)
        };

        Ok(Value::String(result))
    }
}

// quarter(timestamp) -> number
defn!(QuarterFn, vec![arg!(number)], None);

impl Function for QuarterFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_f64().unwrap();
        let dt = Utc.timestamp_opt(ts as i64, 0);

        match dt {
            chrono::LocalResult::Single(dt) => {
                let month = dt.month();
                let quarter = ((month - 1) / 3) + 1;
                Ok(number_value(quarter as f64))
            }
            _ => Ok(Value::Null),
        }
    }
}

/// Helper function to parse a date value that can be either a string or a number (timestamp).
/// Returns the Unix timestamp as i64, or None if parsing fails.
fn parse_date_value(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_f64().map(|f| f as i64),
        Value::String(s) => {
            // Try RFC3339 first
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Some(dt.timestamp());
            }
            // Try ISO datetime without timezone
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                return Some(dt.and_utc().timestamp());
            }
            // Try date only
            if let Ok(dt) =
                NaiveDateTime::parse_from_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S")
            {
                return Some(dt.and_utc().timestamp());
            }
            None
        }
        _ => None,
    }
}

// is_after(date1, date2) -> boolean
defn!(IsAfterFn, vec![arg!(any), arg!(any)], None);

impl Function for IsAfterFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts1 = parse_date_value(&args[0]);
        let ts2 = parse_date_value(&args[1]);

        match (ts1, ts2) {
            (Some(t1), Some(t2)) => Ok(Value::Bool(t1 > t2)),
            _ => Ok(Value::Null),
        }
    }
}

// is_before(date1, date2) -> boolean
defn!(IsBeforeFn, vec![arg!(any), arg!(any)], None);

impl Function for IsBeforeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts1 = parse_date_value(&args[0]);
        let ts2 = parse_date_value(&args[1]);

        match (ts1, ts2) {
            (Some(t1), Some(t2)) => Ok(Value::Bool(t1 < t2)),
            _ => Ok(Value::Null),
        }
    }
}

// is_between(date, start, end) -> boolean
defn!(IsBetweenFn, vec![arg!(any), arg!(any), arg!(any)], None);

impl Function for IsBetweenFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = parse_date_value(&args[0]);
        let start = parse_date_value(&args[1]);
        let end = parse_date_value(&args[2]);

        match (ts, start, end) {
            (Some(t), Some(s), Some(e)) => Ok(Value::Bool(t >= s && t <= e)),
            _ => Ok(Value::Null),
        }
    }
}

// time_ago(date) -> string
defn!(TimeAgoFn, vec![arg!(any)], None);

impl Function for TimeAgoFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };

        let now = Utc::now().timestamp();
        let diff = now - ts;
        let abs_diff = diff.abs();

        // Determine the unit and value
        let (value, unit_singular, unit_plural) = if abs_diff < 60 {
            (abs_diff, "second", "seconds")
        } else if abs_diff < 3600 {
            (abs_diff / 60, "minute", "minutes")
        } else if abs_diff < 86400 {
            (abs_diff / 3600, "hour", "hours")
        } else if abs_diff < 2592000 {
            (abs_diff / 86400, "day", "days")
        } else if abs_diff < 31536000 {
            (abs_diff / 2592000, "month", "months")
        } else {
            (abs_diff / 31536000, "year", "years")
        };

        let unit = if value == 1 {
            unit_singular
        } else {
            unit_plural
        };

        let result = if diff < 0 {
            format!("in {} {}", value, unit)
        } else {
            format!("{} {} ago", value, unit)
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// from_epoch(seconds) -> string
// =============================================================================

defn!(FromEpochFn, vec![arg!(number)], None);

impl Function for FromEpochFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let epoch = args[0].as_f64().unwrap() as i64;

        match DateTime::from_timestamp(epoch, 0) {
            Some(dt) => Ok(Value::String(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// from_epoch_ms(milliseconds) -> string
// =============================================================================

defn!(FromEpochMsFn, vec![arg!(number)], None);

impl Function for FromEpochMsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let epoch_ms = args[0].as_f64().unwrap() as i64;
        let seconds = epoch_ms / 1000;
        let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;

        match DateTime::from_timestamp(seconds, nanos) {
            Some(dt) => Ok(Value::String(
                dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            )),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// to_epoch(datetime) -> number
// =============================================================================

defn!(ToEpochFn, vec![arg!(any)], None);

impl Function for ToEpochFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        match parse_date_value(&args[0]) {
            Some(ts) => Ok(number_value(ts as f64)),
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// to_epoch_ms(datetime) -> number
// =============================================================================

defn!(ToEpochMsFn, vec![arg!(any)], None);

impl Function for ToEpochMsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        match parse_date_value(&args[0]) {
            Some(ts) => {
                let ts_ms = ts * 1000;
                Ok(number_value(ts_ms as f64))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// duration_since(datetime) -> object
// =============================================================================

defn!(DurationSinceFn, vec![arg!(any)], None);

impl Function for DurationSinceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let now = Utc::now().timestamp();
        let diff = now - ts;

        // Calculate components
        let is_future = diff < 0;
        let abs_diff = diff.abs();

        let days = abs_diff / 86400;
        let hours = (abs_diff % 86400) / 3600;
        let minutes = (abs_diff % 3600) / 60;
        let seconds = abs_diff % 60;

        // Build human-readable string
        let human = if days > 0 {
            if days == 1 {
                "1 day".to_string()
            } else {
                format!("{} days", days)
            }
        } else if hours > 0 {
            if hours == 1 {
                "1 hour".to_string()
            } else {
                format!("{} hours", hours)
            }
        } else if minutes > 0 {
            if minutes == 1 {
                "1 minute".to_string()
            } else {
                format!("{} minutes", minutes)
            }
        } else if seconds == 1 {
            "1 second".to_string()
        } else {
            format!("{} seconds", seconds)
        };

        let human_with_direction = if is_future {
            format!("in {}", human)
        } else {
            format!("{} ago", human)
        };

        // Build result object
        let mut map = serde_json::Map::new();
        map.insert(
            "seconds".to_string(),
            Value::Number(serde_json::Number::from(abs_diff)),
        );
        map.insert(
            "minutes".to_string(),
            Value::Number(serde_json::Number::from(abs_diff / 60)),
        );
        map.insert(
            "hours".to_string(),
            Value::Number(serde_json::Number::from(abs_diff / 3600)),
        );
        map.insert(
            "days".to_string(),
            Value::Number(serde_json::Number::from(abs_diff / 86400)),
        );
        map.insert("is_future".to_string(), Value::Bool(is_future));
        map.insert("human".to_string(), Value::String(human_with_direction));

        Ok(Value::Object(map))
    }
}

// =============================================================================
// start_of_day(datetime) -> string
// =============================================================================

defn!(StartOfDayFn, vec![arg!(any)], None);

impl Function for StartOfDayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let dt = DateTime::from_timestamp(ts, 0).unwrap();
        let start = dt.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

        Ok(Value::String(
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ))
    }
}

// =============================================================================
// end_of_day(datetime) -> string
// =============================================================================

defn!(EndOfDayFn, vec![arg!(any)], None);

impl Function for EndOfDayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let dt = DateTime::from_timestamp(ts, 0).unwrap();
        let end = dt.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();

        Ok(Value::String(end.format("%Y-%m-%dT%H:%M:%SZ").to_string()))
    }
}

// =============================================================================
// start_of_week(datetime) -> string
// =============================================================================

defn!(StartOfWeekFn, vec![arg!(any)], None);

impl Function for StartOfWeekFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let dt = DateTime::from_timestamp(ts, 0).unwrap();

        // Calculate days since Monday (Monday = 0)
        let days_since_monday = dt.weekday().num_days_from_monday();
        let monday = dt.date_naive() - chrono::Duration::days(days_since_monday as i64);
        let start = monday.and_hms_opt(0, 0, 0).unwrap().and_utc();

        Ok(Value::String(
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ))
    }
}

// =============================================================================
// start_of_month(datetime) -> string
// =============================================================================

defn!(StartOfMonthFn, vec![arg!(any)], None);

impl Function for StartOfMonthFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let dt = DateTime::from_timestamp(ts, 0).unwrap();

        let start = dt
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        Ok(Value::String(
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ))
    }
}

// =============================================================================
// start_of_year(datetime) -> string
// =============================================================================

defn!(StartOfYearFn, vec![arg!(any)], None);

impl Function for StartOfYearFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let dt = DateTime::from_timestamp(ts, 0).unwrap();

        let start = chrono::NaiveDate::from_ymd_opt(dt.year(), 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        Ok(Value::String(
            start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ))
    }
}

// =============================================================================
// is_same_day(datetime1, datetime2) -> boolean
// =============================================================================

defn!(IsSameDayFn, vec![arg!(any), arg!(any)], None);

impl Function for IsSameDayFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let ts1 = match parse_date_value(&args[0]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };
        let ts2 = match parse_date_value(&args[1]) {
            Some(t) => t,
            None => return Ok(Value::Null),
        };

        let dt1 = match DateTime::from_timestamp(ts1, 0) {
            Some(dt) => dt,
            None => return Ok(Value::Null),
        };
        let dt2 = match DateTime::from_timestamp(ts2, 0) {
            Some(dt) => dt,
            None => return Ok(Value::Null),
        };

        let same_day = dt1.date_naive() == dt2.date_naive();

        Ok(Value::Bool(same_day))
    }
}

// =============================================================================
// parse_datetime(date_string) -> string
// =============================================================================

defn!(ParseDatetimeFn, vec![arg!(string)], None);

impl Function for ParseDatetimeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().unwrap();

        match dateparser::parse_with_timezone(input, &Utc) {
            Ok(dt) => {
                let iso = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                Ok(Value::String(iso))
            }
            Err(_) => Ok(Value::Null),
        }
    }
}

// =============================================================================
// parse_natural_date(expression) -> string
// =============================================================================

defn!(ParseNaturalDateFn, vec![arg!(string)], None);

impl Function for ParseNaturalDateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_str().unwrap();

        match chrono_english::parse_date_string(input, Utc::now(), chrono_english::Dialect::Us) {
            Ok(dt) => {
                let iso = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                Ok(Value::String(iso))
            }
            Err(_) => Ok(Value::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use chrono::Utc;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_now() {
        let runtime = setup_runtime();
        let expr = runtime.compile("now()").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let ts = result.as_f64().unwrap();
        // Should be a reasonable timestamp (after 2020)
        assert!(ts > 1577836800.0);
    }

    #[test]
    fn test_now_millis() {
        let runtime = setup_runtime();
        // now_millis is registered as epoch_ms in jpx-core
        let expr = runtime.compile("epoch_ms()").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let ts = result.as_f64().unwrap();
        // Should be a reasonable timestamp in millis (after 2020)
        assert!(ts > 1577836800000.0);
    }

    #[test]
    fn test_format_date() {
        let runtime = setup_runtime();
        // 1720000000 = 2024-07-03T10:26:40Z
        let expr = runtime
            .compile("format_date(`1720000000`, '%Y-%m-%d')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "2024-07-03");
    }

    #[test]
    fn test_format_date_with_time() {
        let runtime = setup_runtime();
        // Use a known timestamp and verify output format
        let expr = runtime
            .compile("format_date(`0`, '%Y-%m-%dT%H:%M:%S')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1970-01-01T00:00:00");
    }

    #[test]
    fn test_parse_date_iso() {
        let runtime = setup_runtime();
        let data = json!("1970-01-01T00:00:00Z");
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_parse_date_date_only() {
        let runtime = setup_runtime();
        let data = json!("2024-07-03");
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // Should parse as midnight UTC
        assert_eq!(result.as_f64().unwrap(), 1719964800.0);
    }

    #[test]
    fn test_parse_date_with_format() {
        let runtime = setup_runtime();
        // Use datetime format for custom parsing
        let data = json!("03/07/2024 00:00:00");
        let expr = runtime
            .compile("parse_date(@, '%d/%m/%Y %H:%M:%S')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Should parse as 2024-07-03 midnight UTC
        assert_eq!(result.as_f64().unwrap(), 1719964800.0);
    }

    #[test]
    fn test_parse_date_invalid() {
        let runtime = setup_runtime();
        let data = json!("not a date");
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_date_add_days() {
        let runtime = setup_runtime();
        // Add 7 days to 1720000000
        let expr = runtime
            .compile("date_add(`1720000000`, `7`, 'days')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1720604800.0);
    }

    #[test]
    fn test_date_add_hours() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("date_add(`1720000000`, `24`, 'hours')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1720086400.0);
    }

    #[test]
    fn test_date_add_negative() {
        let runtime = setup_runtime();
        // Subtract 1 day
        let expr = runtime
            .compile("date_add(`1720000000`, `-1`, 'day')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1719913600.0);
    }

    #[test]
    fn test_date_diff_days() {
        let runtime = setup_runtime();
        // 7 days apart
        let expr = runtime
            .compile("date_diff(`1720604800`, `1720000000`, 'days')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 7.0);
    }

    #[test]
    fn test_date_diff_hours() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("date_diff(`1720086400`, `1720000000`, 'hours')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 24.0);
    }

    #[test]
    fn test_date_diff_negative() {
        let runtime = setup_runtime();
        // Earlier timestamp first
        let expr = runtime
            .compile("date_diff(`1720000000`, `1720604800`, 'days')")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), -7.0);
    }

    #[test]
    fn test_date_add_invalid_unit() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("date_add(`1720000000`, `1`, 'invalid')")
            .unwrap();
        let result = expr.search(&json!(null));
        assert!(result.is_err());
    }

    #[test]
    fn test_timezone_convert_ny_to_london() {
        let runtime = setup_runtime();
        let data = json!("2024-01-15T10:00:00");
        let expr = runtime
            .compile("timezone_convert(@, 'America/New_York', 'Europe/London')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // NY is UTC-5 in January, London is UTC+0, so 10:00 NY = 15:00 London
        assert_eq!(result.as_str().unwrap(), "2024-01-15T15:00:00");
    }

    #[test]
    fn test_timezone_convert_tokyo_to_la() {
        let runtime = setup_runtime();
        let data = json!("2024-07-15T09:00:00");
        let expr = runtime
            .compile("timezone_convert(@, 'Asia/Tokyo', 'America/Los_Angeles')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Tokyo is UTC+9, LA is UTC-7 in July (PDT), so 9:00 Tokyo = 17:00 previous day LA
        assert_eq!(result.as_str().unwrap(), "2024-07-14T17:00:00");
    }

    #[test]
    fn test_timezone_convert_invalid_tz() {
        let runtime = setup_runtime();
        let data = json!("2024-01-15T10:00:00");
        let expr = runtime
            .compile("timezone_convert(@, 'Invalid/Zone', 'Europe/London')")
            .unwrap();
        let result = expr.search(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_weekend_saturday() {
        let runtime = setup_runtime();
        // 2024-01-13 is a Saturday - timestamp: 1705104000
        let expr = runtime.compile("is_weekend(`1705104000`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_weekend_sunday() {
        let runtime = setup_runtime();
        // 2024-01-14 is a Sunday - timestamp: 1705190400
        let expr = runtime.compile("is_weekend(`1705190400`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_weekend_monday() {
        let runtime = setup_runtime();
        // 2024-01-15 is a Monday - timestamp: 1705276800
        let expr = runtime.compile("is_weekend(`1705276800`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_is_weekday_monday() {
        let runtime = setup_runtime();
        // 2024-01-15 is a Monday - timestamp: 1705276800
        let expr = runtime.compile("is_weekday(`1705276800`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_weekday_saturday() {
        let runtime = setup_runtime();
        // 2024-01-13 is a Saturday - timestamp: 1705104000
        let expr = runtime.compile("is_weekday(`1705104000`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_business_days_between() {
        let runtime = setup_runtime();
        // 2024-01-01 (Mon) to 2024-01-15 (Mon) - 10 business days
        // ts1: 1704067200 (2024-01-01)
        // ts2: 1705276800 (2024-01-15)
        let expr = runtime
            .compile("business_days_between(`1704067200`, `1705276800`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 10.0);
    }

    #[test]
    fn test_business_days_between_reversed() {
        let runtime = setup_runtime();
        // Same dates but reversed - should be negative
        let expr = runtime
            .compile("business_days_between(`1705276800`, `1704067200`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), -10.0);
    }

    #[test]
    fn test_business_days_between_same_day() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("business_days_between(`1705276800`, `1705276800`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_quarter_q1() {
        let runtime = setup_runtime();
        // January 15, 2024 - timestamp: 1705276800
        let expr = runtime.compile("quarter(`1705276800`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_quarter_q2() {
        let runtime = setup_runtime();
        // April 15, 2024 - timestamp: 1713139200
        let expr = runtime.compile("quarter(`1713139200`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_quarter_q3() {
        let runtime = setup_runtime();
        // July 15, 2024 - timestamp: 1721001600
        let expr = runtime.compile("quarter(`1721001600`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_quarter_q4() {
        let runtime = setup_runtime();
        // October 15, 2024 - timestamp: 1728950400
        let expr = runtime.compile("quarter(`1728950400`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_relative_time_past() {
        let runtime = setup_runtime();
        // Use a timestamp far in the past (1 year ago)
        let one_year_ago = Utc::now().timestamp() - 31536000;
        let expr_str = format!("relative_time(`{}`)", one_year_ago);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_str().unwrap().contains("ago"));
    }

    #[test]
    fn test_relative_time_future() {
        let runtime = setup_runtime();
        // Use a timestamp in the future (1 day from now)
        let one_day_future = Utc::now().timestamp() + 86400;
        let expr_str = format!("relative_time(`{}`)", one_day_future);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_str().unwrap().starts_with("in "));
    }

    // Tests for is_after

    #[test]
    fn test_is_after_with_timestamps() {
        let runtime = setup_runtime();
        // 1720000000 is after 1710000000
        let expr = runtime
            .compile("is_after(`1720000000`, `1710000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_with_timestamps_false() {
        let runtime = setup_runtime();
        // 1710000000 is not after 1720000000
        let expr = runtime
            .compile("is_after(`1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_with_date_strings() {
        let runtime = setup_runtime();
        let data = json!({"d1": "2024-07-15", "d2": "2024-01-01"});
        let expr = runtime.compile("is_after(d1, d2)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_with_iso_strings() {
        let runtime = setup_runtime();
        let data = json!({"d1": "2024-07-15T10:30:00Z", "d2": "2024-07-15T08:00:00Z"});
        let expr = runtime.compile("is_after(d1, d2)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_mixed_types() {
        let runtime = setup_runtime();
        // 1720000000 = 2024-07-03T10:26:40Z, which is after 2024-01-01
        let data = json!({"d": "2024-01-01"});
        let expr = runtime.compile("is_after(`1720000000`, d)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_equal_dates() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("is_after(`1720000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_is_after_invalid_date() {
        let runtime = setup_runtime();
        let data = json!({"d": "not-a-date"});
        let expr = runtime.compile("is_after(d, `1720000000`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    // Tests for is_before

    #[test]
    fn test_is_before_with_timestamps() {
        let runtime = setup_runtime();
        // 1710000000 is before 1720000000
        let expr = runtime
            .compile("is_before(`1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_before_with_timestamps_false() {
        let runtime = setup_runtime();
        // 1720000000 is not before 1710000000
        let expr = runtime
            .compile("is_before(`1720000000`, `1710000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_is_before_with_date_strings() {
        let runtime = setup_runtime();
        let data = json!({"d1": "2024-01-01", "d2": "2024-07-15"});
        let expr = runtime.compile("is_before(d1, d2)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_before_equal_dates() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("is_before(`1720000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    // Tests for is_between

    #[test]
    fn test_is_between_with_timestamps_true() {
        let runtime = setup_runtime();
        // 1715000000 is between 1710000000 and 1720000000
        let expr = runtime
            .compile("is_between(`1715000000`, `1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_between_with_timestamps_false() {
        let runtime = setup_runtime();
        // 1700000000 is not between 1710000000 and 1720000000
        let expr = runtime
            .compile("is_between(`1700000000`, `1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_is_between_with_date_strings() {
        let runtime = setup_runtime();
        let data = json!({"d": "2024-06-15", "start": "2024-01-01", "end": "2024-12-31"});
        let expr = runtime.compile("is_between(d, start, end)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_between_inclusive_start() {
        let runtime = setup_runtime();
        // Date equals start - should be true (inclusive)
        let expr = runtime
            .compile("is_between(`1710000000`, `1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_between_inclusive_end() {
        let runtime = setup_runtime();
        // Date equals end - should be true (inclusive)
        let expr = runtime
            .compile("is_between(`1720000000`, `1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_between_outside_range() {
        let runtime = setup_runtime();
        // Date is after end
        let expr = runtime
            .compile("is_between(`1730000000`, `1710000000`, `1720000000`)")
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    // Tests for time_ago

    #[test]
    fn test_time_ago_with_timestamp() {
        let runtime = setup_runtime();
        // Use a timestamp 1 hour ago
        let one_hour_ago = Utc::now().timestamp() - 3600;
        let expr_str = format!("time_ago(`{}`)", one_hour_ago);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1 hour ago");
    }

    #[test]
    fn test_time_ago_with_date_string() {
        let runtime = setup_runtime();
        // Use a date far in the past (over 1 year)
        let data = json!("2020-01-01");
        let expr = runtime.compile("time_ago(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_str().unwrap().contains("years ago"));
    }

    #[test]
    fn test_time_ago_plural() {
        let runtime = setup_runtime();
        // Use a timestamp 2 days ago
        let two_days_ago = Utc::now().timestamp() - 172800;
        let expr_str = format!("time_ago(`{}`)", two_days_ago);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "2 days ago");
    }

    #[test]
    fn test_time_ago_singular() {
        let runtime = setup_runtime();
        // Use a timestamp 1 day ago
        let one_day_ago = Utc::now().timestamp() - 86400;
        let expr_str = format!("time_ago(`{}`)", one_day_ago);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "1 day ago");
    }

    #[test]
    fn test_time_ago_future() {
        let runtime = setup_runtime();
        // Future dates show "in X"
        let one_day_future = Utc::now().timestamp() + 86400;
        let expr_str = format!("time_ago(`{}`)", one_day_future);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_str().unwrap().starts_with("in "));
    }

    #[test]
    fn test_time_ago_invalid_date() {
        let runtime = setup_runtime();
        let data = json!("not-a-date");
        let expr = runtime.compile("time_ago(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_from_epoch() {
        let runtime = setup_runtime();
        // 2023-12-13T00:00:00Z
        let expr = runtime.compile("from_epoch(`1702425600`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-12-13T00:00:00Z");
    }

    #[test]
    fn test_from_epoch_ms() {
        let runtime = setup_runtime();
        // 2023-12-13T00:00:00.500Z
        let expr = runtime.compile("from_epoch_ms(`1702425600500`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-12-13T00:00:00.500Z");
    }

    #[test]
    fn test_to_epoch() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T00:00:00Z");
        let expr = runtime.compile("to_epoch(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 1702425600);
    }

    #[test]
    fn test_to_epoch_ms() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T00:00:00Z");
        let expr = runtime.compile("to_epoch_ms(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 1702425600000);
    }

    #[test]
    fn test_to_epoch_from_number() {
        let runtime = setup_runtime();
        // Pass through if already a number
        let expr = runtime.compile("to_epoch(`1702425600`)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 1702425600);
    }

    #[test]
    fn test_duration_since() {
        let runtime = setup_runtime();
        // Use a timestamp 2 days ago
        let two_days_ago = Utc::now().timestamp() - 172800;
        let expr_str = format!("duration_since(`{}`)", two_days_ago);
        let expr = runtime.compile(&expr_str).unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("days").unwrap().as_i64().unwrap(), 2);
        assert!(!obj.get("is_future").unwrap().as_bool().unwrap());
        assert!(
            obj.get("human")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("2 days ago")
        );
    }

    #[test]
    fn test_start_of_day() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T15:30:45Z");
        let expr = runtime.compile("start_of_day(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-12-13T00:00:00Z");
    }

    #[test]
    fn test_end_of_day() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T15:30:45Z");
        let expr = runtime.compile("end_of_day(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-12-13T23:59:59Z");
    }

    #[test]
    fn test_start_of_week() {
        let runtime = setup_runtime();
        // 2023-12-13 is a Wednesday
        let data = json!("2023-12-13T15:30:45Z");
        let expr = runtime.compile("start_of_week(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // Monday is 2023-12-11
        assert_eq!(result.as_str().unwrap(), "2023-12-11T00:00:00Z");
    }

    #[test]
    fn test_start_of_month() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T15:30:45Z");
        let expr = runtime.compile("start_of_month(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-12-01T00:00:00Z");
    }

    #[test]
    fn test_start_of_year() {
        let runtime = setup_runtime();
        let data = json!("2023-12-13T15:30:45Z");
        let expr = runtime.compile("start_of_year(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "2023-01-01T00:00:00Z");
    }

    #[test]
    fn test_is_same_day_true() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile(r#"is_same_day(`"2023-12-13T10:00:00Z"`, `"2023-12-13T23:00:00Z"`)"#)
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_same_day_false() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile(r#"is_same_day(`"2023-12-13T10:00:00Z"`, `"2023-12-14T10:00:00Z"`)"#)
            .unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_epoch_ms_alias() {
        let runtime = setup_runtime();
        // epoch_ms should work like now_millis
        let expr = runtime.compile("epoch_ms()").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        let ts = result.as_f64().unwrap() as i64;
        // Should be a reasonable current timestamp in milliseconds
        assert!(ts > 1700000000000);
    }

    // =========================================================================
    // parse_datetime tests (structured formats)
    // =========================================================================

    #[test]
    fn test_parse_datetime_iso_date() {
        let runtime = setup_runtime();
        let data = json!("2024-01-15");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024-01-15"), "Expected 2024-01-15 in {}", s);
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_datetime_iso_datetime() {
        let runtime = setup_runtime();
        let data = json!("2024-01-15T10:30:00Z");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert_eq!(s, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_parse_datetime_iso_with_offset() {
        let runtime = setup_runtime();
        let data = json!("2024-01-15T10:30:00+05:00");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        // Should convert to UTC (10:30 +05:00 = 05:30 UTC)
        assert!(s.contains("2024-01-15"), "Expected 2024-01-15 in {}", s);
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_datetime_human_month_day_year() {
        let runtime = setup_runtime();
        let data = json!("Jan 15, 2024");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024-01-15"), "Expected 2024-01-15 in {}", s);
    }

    #[test]
    fn test_parse_datetime_human_full_month() {
        let runtime = setup_runtime();
        let data = json!("January 15, 2024");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024-01-15"), "Expected 2024-01-15 in {}", s);
    }

    #[test]
    fn test_parse_datetime_us_format() {
        let runtime = setup_runtime();
        let data = json!("01/15/2024");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024"), "Expected year 2024 in {}", s);
    }

    #[test]
    fn test_parse_datetime_rfc2822() {
        let runtime = setup_runtime();
        let data = json!("Mon, 15 Jan 2024 10:30:00 +0000");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024-01-15"), "Expected 2024-01-15 in {}", s);
    }

    #[test]
    fn test_parse_datetime_unix_timestamp() {
        let runtime = setup_runtime();
        // 1705363200 = 2024-01-16T00:00:00Z
        let data = json!("1705363200");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.contains("2024-01-16"), "Expected 2024-01-16 in {}", s);
    }

    #[test]
    fn test_parse_datetime_invalid() {
        let runtime = setup_runtime();
        let data = json!("not a date at all xyz123");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null(), "Invalid date should return null");
    }

    #[test]
    fn test_parse_datetime_empty() {
        let runtime = setup_runtime();
        let data = json!("");
        let expr = runtime.compile("parse_datetime(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null(), "Empty string should return null");
    }

    // =========================================================================
    // parse_natural_date tests (natural language, relative to now)
    // =========================================================================

    #[test]
    fn test_parse_natural_date_yesterday() {
        let runtime = setup_runtime();
        let data = json!("yesterday");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
        assert!(
            s.contains('T'),
            "Expected ISO format with T separator in {}",
            s
        );
    }

    #[test]
    fn test_parse_natural_date_tomorrow() {
        let runtime = setup_runtime();
        let data = json!("tomorrow");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_days_ago() {
        let runtime = setup_runtime();
        let data = json!("3 days ago");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_weeks_ago() {
        let runtime = setup_runtime();
        let data = json!("2 weeks ago");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_next_weekday() {
        let runtime = setup_runtime();
        let data = json!("next friday");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_last_weekday() {
        let runtime = setup_runtime();
        let data = json!("last monday");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_hours() {
        let runtime = setup_runtime();
        let data = json!("5 hours ago");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_str().unwrap();
        assert!(s.ends_with('Z'), "Expected UTC timezone marker in {}", s);
    }

    #[test]
    fn test_parse_natural_date_invalid() {
        let runtime = setup_runtime();
        let data = json!("not a natural date expression");
        let expr = runtime.compile("parse_natural_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null(), "Invalid expression should return null");
    }
}
