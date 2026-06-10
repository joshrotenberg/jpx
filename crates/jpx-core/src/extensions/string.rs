//! String manipulation functions.

use std::collections::HashSet;

use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
    ToTrainCase, ToUpperCamelCase,
};
use regex::Regex;
use serde_json::{Number, Value};

use crate::functions::{Function, custom_error};
use crate::interpreter::SearchResult;
use crate::registry::register_if_enabled;
use crate::{Context, Runtime, arg, defn};

/// Upper bound on the size of a string produced by user-controlled repetition
/// (`repeat`, `center`, ...). Prevents a single call from requesting a
/// petabyte-sized allocation (e.g. `repeat('x', `1e15`)`).
const MAX_GENERATED_STRING_BYTES: usize = 64 * 1024 * 1024;

// =============================================================================
// lower(string) -> string
// =============================================================================

defn!(LowerFn, vec![arg!(string)], None);

impl Function for LowerFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_lowercase()))
    }
}

// =============================================================================
// upper(string) -> string
// =============================================================================

defn!(UpperFn, vec![arg!(string)], None);

impl Function for UpperFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_uppercase()))
    }
}

// =============================================================================
// trim(string) -> string
// =============================================================================

defn!(TrimFn, vec![arg!(string)], None);

impl Function for TrimFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.trim().to_string()))
    }
}

// =============================================================================
// trim_start(string) -> string
// =============================================================================

defn!(TrimStartFn, vec![arg!(string)], None);

impl Function for TrimStartFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.trim_start().to_string()))
    }
}

// =============================================================================
// trim_end(string) -> string
// =============================================================================

defn!(TrimEndFn, vec![arg!(string)], None);

impl Function for TrimEndFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.trim_end().to_string()))
    }
}

// =============================================================================
// split(string, delimiter) -> array
// =============================================================================

defn!(SplitFn, vec![arg!(string), arg!(string)], None);

impl Function for SplitFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let delimiter = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string delimiter"))?;

        let parts: Vec<Value> = s
            .split(delimiter)
            .map(|part| Value::String(part.to_string()))
            .collect();

        Ok(Value::Array(parts))
    }
}

// =============================================================================
// replace(string, old, new) -> string
// =============================================================================

defn!(
    ReplaceFn,
    vec![arg!(string), arg!(string), arg!(string)],
    None
);

impl Function for ReplaceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let old = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected old string argument"))?;
        let new = args[2]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected new string argument"))?;

        Ok(Value::String(s.replace(old, new)))
    }
}

// =============================================================================
// pad_left(string, width, char) -> string
// =============================================================================

defn!(
    PadLeftFn,
    vec![arg!(string), arg!(number), arg!(string)],
    None
);

impl Function for PadLeftFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let width = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number for width"))?;
        let pad_char = args[2]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string for pad character"))?;

        let pad = pad_char.chars().next().unwrap_or(' ');
        let result = if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", pad.to_string().repeat(width - s.len()), s)
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// pad_right(string, width, char) -> string
// =============================================================================

defn!(
    PadRightFn,
    vec![arg!(string), arg!(number), arg!(string)],
    None
);

impl Function for PadRightFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let width = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number for width"))?;
        let pad_char = args[2]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string for pad character"))?;

        let pad = pad_char.chars().next().unwrap_or(' ');
        let result = if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, pad.to_string().repeat(width - s.len()))
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// substr(string, start, length?) -> string
// =============================================================================

defn!(
    SubstrFn,
    vec![arg!(string), arg!(number)],
    Some(arg!(number))
);

impl Function for SubstrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let start = args[1]
            .as_f64()
            .map(|n| n as i64)
            .ok_or_else(|| custom_error(ctx, "Expected number for start"))?;

        // Handle negative start (from end)
        let start_idx = if start < 0 {
            (s.len() as i64 + start).max(0) as usize
        } else {
            start as usize
        };

        let result = if args.len() > 2 {
            let length = args[2]
                .as_f64()
                .map(|n| n as usize)
                .ok_or_else(|| custom_error(ctx, "Expected positive number for length"))?;
            s.chars().skip(start_idx).take(length).collect()
        } else {
            s.chars().skip(start_idx).collect()
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// capitalize(string) -> string (first letter uppercase)
// =============================================================================

defn!(CapitalizeFn, vec![arg!(string)], None);

impl Function for CapitalizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let result = if s.is_empty() {
            String::new()
        } else {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + chars.as_str(),
            }
        };

        Ok(Value::String(result))
    }
}

// =============================================================================
// title(string) -> string (capitalize each word)
// =============================================================================

defn!(TitleFn, vec![arg!(string)], None);

impl Function for TitleFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let result = s
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().to_string() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Value::String(result))
    }
}

// =============================================================================
// repeat(string, count) -> string
// =============================================================================

defn!(RepeatFn, vec![arg!(string), arg!(number)], None);

impl Function for RepeatFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let count = args[1]
            .as_f64()
            .map(|n| n as usize)
            .ok_or_else(|| custom_error(ctx, "Expected positive number for count"))?;

        if count.saturating_mul(s.len()) > MAX_GENERATED_STRING_BYTES {
            return Err(custom_error(
                ctx,
                "repeat result exceeds the maximum allowed size",
            ));
        }
        Ok(Value::String(s.repeat(count)))
    }
}

// =============================================================================
// index_of(string, search, start?, end?) -> number | null (JEP-014)
// =============================================================================

defn!(
    IndexOfFn,
    vec![arg!(string), arg!(string)],
    Some(arg!(number))
);

/// Helper to normalize an index (handling negative values) within a string length
fn normalize_index(idx: i64, len: usize) -> usize {
    if idx < 0 {
        (len as i64 + idx).max(0) as usize
    } else {
        (idx as usize).min(len)
    }
}

impl Function for IndexOfFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let search = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected search string"))?;

        // Operate on character indices so multibyte input is handled correctly
        // and never slices through a UTF-8 code point boundary.
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        // Get optional start parameter (default: 0)
        let start = if args.len() > 2 {
            let start_val = args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for start"))?
                as i64;
            normalize_index(start_val, len)
        } else {
            0
        };

        // Get optional end parameter (default: string length)
        let end = if args.len() > 3 {
            let end_val = args[3]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for end"))?
                as i64;
            normalize_index(end_val, len)
        } else {
            len
        };

        // Search within the [start, end) character window.
        if start >= end || start >= len {
            return Ok(Value::Null);
        }

        let window: String = chars[start..end].iter().collect();
        match window.find(search) {
            Some(byte_idx) => {
                let char_idx = window[..byte_idx].chars().count();
                Ok(Value::Number(Number::from((start + char_idx) as i64)))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// last_index_of(string, search, start?, end?) -> number | null (JEP-014)
// =============================================================================

defn!(
    LastIndexOfFn,
    vec![arg!(string), arg!(string)],
    Some(arg!(number))
);

impl Function for LastIndexOfFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let search = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected search string"))?;

        // Operate on character indices so multibyte input is handled correctly
        // and never slices through a UTF-8 code point boundary.
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        // Get optional start parameter (default: 0)
        let start = if args.len() > 2 {
            let start_val = args[2]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for start"))?
                as i64;
            normalize_index(start_val, len)
        } else {
            0
        };

        // Get optional end parameter (default: string length)
        let end = if args.len() > 3 {
            let end_val = args[3]
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected number for end"))?
                as i64;
            normalize_index(end_val, len)
        } else {
            len
        };

        // Search within the [start, end) character window.
        if start >= end || start >= len {
            return Ok(Value::Null);
        }

        let window: String = chars[start..end].iter().collect();
        match window.rfind(search) {
            Some(byte_idx) => {
                let char_idx = window[..byte_idx].chars().count();
                Ok(Value::Number(Number::from((start + char_idx) as i64)))
            }
            None => Ok(Value::Null),
        }
    }
}

// =============================================================================
// slice(string, start, end?) -> string
// =============================================================================

defn!(
    SliceFn,
    vec![arg!(string), arg!(number)],
    Some(arg!(number))
);

impl Function for SliceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let len = s.len() as i64;

        let start = args[1]
            .as_f64()
            .map(|n| n as i64)
            .ok_or_else(|| custom_error(ctx, "Expected number for start"))?;

        // Handle negative indices
        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let end_idx = if args.len() > 2 {
            let end = args[2]
                .as_f64()
                .map(|n| n as i64)
                .ok_or_else(|| custom_error(ctx, "Expected number for end"))?;
            if end < 0 {
                (len + end).max(0) as usize
            } else {
                end.min(len) as usize
            }
        } else {
            len as usize
        };

        let result: String = s
            .chars()
            .skip(start_idx)
            .take(end_idx.saturating_sub(start_idx))
            .collect();

        Ok(Value::String(result))
    }
}

// =============================================================================
// concat(array_of_strings, separator?) -> string
// =============================================================================

defn!(ConcatFn, vec![arg!(array)], Some(arg!(string)));

impl Function for ConcatFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let separator = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string()).unwrap_or_default()
        } else {
            String::new()
        };

        let strings: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(Value::String(strings.join(&separator)))
    }
}

// =============================================================================
// upper_case(string) -> string (alias for upper, snake_case style)
// =============================================================================

defn!(UpperCaseFn, vec![arg!(string)], None);

impl Function for UpperCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_uppercase()))
    }
}

// =============================================================================
// lower_case(string) -> string (alias for lower, snake_case style)
// =============================================================================

defn!(LowerCaseFn, vec![arg!(string)], None);

impl Function for LowerCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_lowercase()))
    }
}

// =============================================================================
// title_case(string) -> string (alias for title, snake_case style)
// Uses heck crate for proper case conversion
// =============================================================================

defn!(TitleCaseFn, vec![arg!(string)], None);

impl Function for TitleCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_title_case()))
    }
}

// =============================================================================
// camel_case(string) -> string (helloWorld)
// Uses heck crate for proper case conversion
// =============================================================================

defn!(CamelCaseFn, vec![arg!(string)], None);

impl Function for CamelCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_lower_camel_case()))
    }
}

// =============================================================================
// snake_case(string) -> string (hello_world)
// Uses heck crate for proper case conversion
// =============================================================================

defn!(SnakeCaseFn, vec![arg!(string)], None);

impl Function for SnakeCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_snake_case()))
    }
}

// =============================================================================
// kebab_case(string) -> string (hello-world)
// Uses heck crate for proper case conversion
// =============================================================================

defn!(KebabCaseFn, vec![arg!(string)], None);

impl Function for KebabCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_kebab_case()))
    }
}

// =============================================================================
// pascal_case(string) -> string (HelloWorld)
// Uses heck crate - also known as UpperCamelCase
// =============================================================================

defn!(PascalCaseFn, vec![arg!(string)], None);

impl Function for PascalCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_upper_camel_case()))
    }
}

// =============================================================================
// shouty_snake_case(string) -> string (HELLO_WORLD)
// Uses heck crate - useful for constants
// =============================================================================

defn!(ShoutySnakeCaseFn, vec![arg!(string)], None);

impl Function for ShoutySnakeCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_shouty_snake_case()))
    }
}

// =============================================================================
// shouty_kebab_case(string) -> string (HELLO-WORLD)
// Uses heck crate
// =============================================================================

defn!(ShoutyKebabCaseFn, vec![arg!(string)], None);

impl Function for ShoutyKebabCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_shouty_kebab_case()))
    }
}

// =============================================================================
// train_case(string) -> string (Hello-World)
// Uses heck crate - like HTTP headers (Content-Type)
// =============================================================================

defn!(TrainCaseFn, vec![arg!(string)], None);

impl Function for TrainCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        Ok(Value::String(s.to_train_case()))
    }
}

// =============================================================================
// truncate(string, length, suffix?) -> string
// =============================================================================

defn!(
    TruncateFn,
    vec![arg!(string), arg!(number)],
    Some(arg!(string))
);

impl Function for TruncateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let max_len = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for length"))?
            as usize;

        let suffix = args
            .get(2)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "...".to_string());

        if s.len() <= max_len {
            Ok(Value::String(s.to_string()))
        } else {
            let truncate_at = max_len.saturating_sub(suffix.len());
            let truncated: String = s.chars().take(truncate_at).collect();
            Ok(Value::String(format!("{}{}", truncated, suffix)))
        }
    }
}

// =============================================================================
// wrap(string, width) -> string with newlines
// =============================================================================

defn!(WrapFn, vec![arg!(string), arg!(number)], None);

impl Function for WrapFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let width = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for width"))?
            as usize;

        if width == 0 {
            return Ok(Value::String(s.to_string()));
        }

        let mut lines: Vec<String> = Vec::new();

        // Process each paragraph (separated by newlines) separately
        for paragraph in s.split('\n') {
            let mut current_line = String::new();

            for word in paragraph.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line);
                    current_line = word.to_string();
                }
            }

            // Push the last line of this paragraph (even if empty to preserve blank lines)
            lines.push(current_line);
        }

        // Remove trailing empty line if the input didn't end with a newline
        if !s.ends_with('\n') && lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }

        // If we have no lines but had input, return the original
        if lines.is_empty() && !s.is_empty() {
            return Ok(Value::String(s.to_string()));
        }

        // Join lines with newlines and return as string
        Ok(Value::String(lines.join("\n")))
    }
}

// =============================================================================
// format(template, args) -> string
// Supports:
//   - Positional with array: format('Hello {0}', ['World'])
//   - Named with object: format('Hello {name}', {name: 'World'})
//   - Variadic: format('Hello {0}', 'World')
// =============================================================================

defn!(FormatFn, vec![arg!(string)], Some(arg!(any)));

/// Convert a Value to its string representation for formatting
fn var_to_format_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
    }
}

impl Function for FormatFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let template = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected template string"))?;

        let mut result = template.to_string();

        // Check if second arg is an array or object for unified formatting
        if args.len() == 2 {
            if let Some(arr) = args[1].as_array() {
                // Array-based positional: format('Hello {0}', ['World'])
                for (i, item) in arr.iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    let value = var_to_format_string(item);
                    result = result.replace(&placeholder, &value);
                }
                return Ok(Value::String(result));
            } else if let Some(obj) = args[1].as_object() {
                // Object-based named: format('Hello {name}', {name: 'World'})
                for (key, val) in obj.iter() {
                    let placeholder = format!("{{{}}}", key);
                    let value = var_to_format_string(val);
                    result = result.replace(&placeholder, &value);
                }
                return Ok(Value::String(result));
            }
        }

        // Fallback: variadic arguments format('Hello {0}', 'World')
        for (i, arg) in args.iter().skip(1).enumerate() {
            let placeholder = format!("{{{}}}", i);
            let value = var_to_format_string(arg);
            result = result.replace(&placeholder, &value);
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// sprintf(format_string, ...args) -> string
// Printf-style formatting with %s, %d, %f, etc.
// =============================================================================

defn!(SprintfFn, vec![arg!(string)], Some(arg!(any)));

impl Function for SprintfFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let format_str = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected format string"))?;

        // Get arguments - either from array or variadic
        let format_args: Vec<&Value> = if args.len() == 2 {
            if let Some(arr) = args[1].as_array() {
                arr.iter().collect()
            } else {
                args.iter().skip(1).collect()
            }
        } else {
            args.iter().skip(1).collect()
        };

        let mut result = String::new();
        let mut arg_index = 0;
        let mut chars = format_str.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        // Escaped %
                        result.push('%');
                        chars.next();
                        continue;
                    }

                    // Parse format specifier
                    let mut width = String::new();
                    let mut precision = String::new();
                    let mut in_precision = false;

                    // Parse width and precision
                    while let Some(&ch) = chars.peek() {
                        if ch == '.' {
                            in_precision = true;
                            chars.next();
                        } else if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                            if in_precision {
                                precision.push(ch);
                            } else {
                                width.push(ch);
                            }
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // Get the format type
                    if let Some(fmt_type) = chars.next() {
                        if arg_index < format_args.len() {
                            let arg = format_args[arg_index];
                            arg_index += 1;

                            let formatted = match fmt_type {
                                's' => var_to_format_string(arg),
                                'd' | 'i' => {
                                    if let Some(n) = arg.as_f64() {
                                        format!("{}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'f' => {
                                    if let Some(n) = arg.as_f64() {
                                        let prec: usize = precision.parse().unwrap_or(6);
                                        format!("{:.prec$}", n, prec = prec)
                                    } else {
                                        "0.0".to_string()
                                    }
                                }
                                'e' => {
                                    if let Some(n) = arg.as_f64() {
                                        let prec: usize = precision.parse().unwrap_or(6);
                                        format!("{:.prec$e}", n, prec = prec)
                                    } else {
                                        "0e0".to_string()
                                    }
                                }
                                'x' => {
                                    if let Some(n) = arg.as_f64() {
                                        format!("{:x}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'X' => {
                                    if let Some(n) = arg.as_f64() {
                                        format!("{:X}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'o' => {
                                    if let Some(n) = arg.as_f64() {
                                        format!("{:o}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'b' => {
                                    if let Some(n) = arg.as_f64() {
                                        format!("{:b}", n as i64)
                                    } else {
                                        "0".to_string()
                                    }
                                }
                                'c' => {
                                    if let Some(n) = arg.as_f64() {
                                        char::from_u32(n as u32)
                                            .map(|c| c.to_string())
                                            .unwrap_or_default()
                                    } else if let Some(s) = arg.as_str() {
                                        s.chars().next().map(|c| c.to_string()).unwrap_or_default()
                                    } else {
                                        String::new()
                                    }
                                }
                                _ => {
                                    // Unknown format, just output as-is
                                    format!("%{}{}", width, fmt_type)
                                }
                            };

                            // Apply width if specified
                            if !width.is_empty() {
                                let w: i32 = width.parse().unwrap_or(0);
                                if w < 0 {
                                    // Left-align
                                    result.push_str(&format!(
                                        "{:<width$}",
                                        formatted,
                                        width = w.unsigned_abs() as usize
                                    ));
                                } else {
                                    // Right-align
                                    result.push_str(&format!(
                                        "{:>width$}",
                                        formatted,
                                        width = w as usize
                                    ));
                                }
                            } else {
                                result.push_str(&formatted);
                            }
                        } else {
                            // Not enough arguments, output placeholder
                            result.push('%');
                            result.push_str(&width);
                            if !precision.is_empty() {
                                result.push('.');
                                result.push_str(&precision);
                            }
                            result.push(fmt_type);
                        }
                    }
                } else {
                    // % at end of string
                    result.push('%');
                }
            } else {
                result.push(c);
            }
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// ltrimstr(string, prefix) -> string
// =============================================================================

defn!(LtrimstrFn, vec![arg!(string), arg!(string)], None);

impl Function for LtrimstrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let prefix = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected prefix string"))?;

        let result = s.strip_prefix(prefix).unwrap_or(s).to_string();
        Ok(Value::String(result))
    }
}

// =============================================================================
// rtrimstr(string, suffix) -> string
// =============================================================================

defn!(RtrimstrFn, vec![arg!(string), arg!(string)], None);

impl Function for RtrimstrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let suffix = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected suffix string"))?;

        let result = s.strip_suffix(suffix).unwrap_or(s).to_string();
        Ok(Value::String(result))
    }
}

// =============================================================================
// indices(string, search) -> array of indices
// =============================================================================

defn!(IndicesFn, vec![arg!(string), arg!(string)], None);

impl Function for IndicesFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let search = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected search string"))?;

        // Find all indices (including overlapping matches)
        let mut indices: Vec<Value> = Vec::new();
        if !search.is_empty() {
            let mut start = 0;
            while let Some(pos) = s[start..].find(search) {
                let actual_pos = start + pos;
                indices.push(Value::Number(Number::from(actual_pos as i64)));
                start = actual_pos + 1; // Move by 1 to find overlapping matches
            }
        }

        Ok(Value::Array(indices))
    }
}

// =============================================================================
// inside(search, string) -> boolean
// =============================================================================

defn!(InsideFn, vec![arg!(string), arg!(string)], None);

impl Function for InsideFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let search = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected search string"))?;
        let s = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        Ok(Value::Bool(s.contains(search)))
    }
}

// =============================================================================
// humanize(string) -> string
// Converts a camelCase, snake_case, or kebab-case string to a human-readable form
// =============================================================================

defn!(HumanizeFn, vec![arg!(string)], None);

impl Function for HumanizeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Split on underscores, hyphens, and camelCase boundaries
        let mut result = String::new();
        let mut prev_was_lower = false;
        let mut word_start = true;

        for c in s.chars() {
            if c == '_' || c == '-' {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                word_start = true;
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary
                result.push(' ');
                if word_start {
                    result.push(c); // Keep first letter of sentence uppercase
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                word_start = false;
                prev_was_lower = false;
            } else {
                if word_start && result.is_empty() {
                    // First character of the string - capitalize it
                    result.push(c.to_uppercase().next().unwrap_or(c));
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                prev_was_lower = c.is_lowercase();
                word_start = false;
            }
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// deburr(string) -> string
// Removes diacritical marks (accents) from characters
// =============================================================================

defn!(DeburrrFn, vec![arg!(string)], None);

impl Function for DeburrrFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Remove diacritical marks by mapping accented characters to ASCII
        let result: String = s
            .chars()
            .map(|c| match c {
                '\u{00C0}' | '\u{00C1}' | '\u{00C2}' | '\u{00C3}' | '\u{00C4}' | '\u{00C5}' => 'A',
                '\u{00C6}' => 'A', // Could be "AE" but keeping single char
                '\u{00C7}' => 'C',
                '\u{00C8}' | '\u{00C9}' | '\u{00CA}' | '\u{00CB}' => 'E',
                '\u{00CC}' | '\u{00CD}' | '\u{00CE}' | '\u{00CF}' => 'I',
                '\u{00D0}' => 'D',
                '\u{00D1}' => 'N',
                '\u{00D2}' | '\u{00D3}' | '\u{00D4}' | '\u{00D5}' | '\u{00D6}' | '\u{00D8}' => 'O',
                '\u{00D9}' | '\u{00DA}' | '\u{00DB}' | '\u{00DC}' => 'U',
                '\u{00DD}' => 'Y',
                '\u{00DE}' => 'T', // Thorn
                '\u{00DF}' => 's', // German sharp s
                '\u{00E0}' | '\u{00E1}' | '\u{00E2}' | '\u{00E3}' | '\u{00E4}' | '\u{00E5}' => 'a',
                '\u{00E6}' => 'a', // Could be "ae" but keeping single char
                '\u{00E7}' => 'c',
                '\u{00E8}' | '\u{00E9}' | '\u{00EA}' | '\u{00EB}' => 'e',
                '\u{00EC}' | '\u{00ED}' | '\u{00EE}' | '\u{00EF}' => 'i',
                '\u{00F0}' => 'd',
                '\u{00F1}' => 'n',
                '\u{00F2}' | '\u{00F3}' | '\u{00F4}' | '\u{00F5}' | '\u{00F6}' | '\u{00F8}' => 'o',
                '\u{00F9}' | '\u{00FA}' | '\u{00FB}' | '\u{00FC}' => 'u',
                '\u{00FD}' | '\u{00FF}' => 'y',
                '\u{00FE}' => 't', // Thorn
                _ => c,
            })
            .collect();

        Ok(Value::String(result))
    }
}

// =============================================================================
// words(string) -> array
// Splits string into an array of words
// =============================================================================

defn!(WordsFn, vec![arg!(string)], None);

impl Function for WordsFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Split on word boundaries: spaces, underscores, hyphens, and camelCase
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut prev_was_lower = false;

        for c in s.chars() {
            if c.is_whitespace() || c == '_' || c == '-' {
                if !current_word.is_empty() {
                    words.push(Value::String(current_word.clone()));
                    current_word.clear();
                }
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary
                if !current_word.is_empty() {
                    words.push(Value::String(current_word.clone()));
                    current_word.clear();
                }
                current_word.push(c);
                prev_was_lower = false;
            } else {
                current_word.push(c);
                prev_was_lower = c.is_lowercase();
            }
        }

        if !current_word.is_empty() {
            words.push(Value::String(current_word));
        }

        Ok(Value::Array(words))
    }
}

// =============================================================================
// escape(string) -> string
// Escapes HTML entities: &, <, >, ", '
// =============================================================================

defn!(EscapeFn, vec![arg!(string)], None);

impl Function for EscapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let result = s
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;");

        Ok(Value::String(result))
    }
}

// =============================================================================
// unescape(string) -> string
// Unescapes HTML entities: &amp;, &lt;, &gt;, &quot;, &#39;
// =============================================================================

defn!(UnescapeFn, vec![arg!(string)], None);

impl Function for UnescapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let result = s
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        Ok(Value::String(result))
    }
}

// =============================================================================
// escape_regex(string) -> string
// Escapes special regex characters
// =============================================================================

defn!(EscapeRegexFn, vec![arg!(string)], None);

impl Function for EscapeRegexFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Escape regex special characters: \ ^ $ . | ? * + ( ) [ ] { }
        let mut result = String::with_capacity(s.len() * 2);
        for c in s.chars() {
            match c {
                '\\' | '^' | '$' | '.' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']' | '{'
                | '}' => {
                    result.push('\\');
                    result.push(c);
                }
                _ => result.push(c),
            }
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// start_case(string) -> string
// Converts string to Start Case (capitalize first letter of each word)
// =============================================================================

defn!(StartCaseFn, vec![arg!(string)], None);

impl Function for StartCaseFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Split on word boundaries and capitalize each word
        let mut result = String::new();
        let mut prev_was_lower = false;
        let mut word_start = true;

        for c in s.chars() {
            if c.is_whitespace() || c == '_' || c == '-' {
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                word_start = true;
                prev_was_lower = false;
            } else if c.is_uppercase() && prev_was_lower {
                // camelCase boundary - start new word
                result.push(' ');
                result.push(c); // Keep uppercase
                word_start = false;
                prev_was_lower = false;
            } else {
                if word_start {
                    result.push(c.to_uppercase().next().unwrap_or(c));
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                prev_was_lower = c.is_lowercase();
                word_start = false;
            }
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// mask(string, visible?, char?) -> string
// Mask a string, optionally keeping the last N characters visible
// =============================================================================

defn!(MaskFn, vec![arg!(string)], Some(arg!(any)));

impl Function for MaskFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Number of characters to keep visible at the end (default: 0)
        let visible = if args.len() > 1 && !args[1].is_null() {
            args[1].as_f64().unwrap_or(0.0) as usize
        } else {
            0
        };

        // Mask character (default: '*')
        let mask_char = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_str()
                .and_then(|s| s.chars().next())
                .unwrap_or('*')
        } else {
            '*'
        };

        let char_count = s.chars().count();

        if visible >= char_count {
            // If visible >= length, return original string
            return Ok(Value::String(s.to_string()));
        }

        let mask_count = char_count - visible;
        let masked: String = std::iter::repeat_n(mask_char, mask_count)
            .chain(s.chars().skip(mask_count))
            .collect();

        Ok(Value::String(masked))
    }
}

// =============================================================================
// redact(string, pattern, replacement?) -> string
// Replace all matches of a regex pattern with a replacement string
// =============================================================================

defn!(RedactFn, vec![arg!(string), arg!(string)], Some(arg!(any)));

impl Function for RedactFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let pattern = args[1]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string pattern"))?;

        // Replacement string (default: "[REDACTED]")
        let replacement = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "[REDACTED]".to_string())
        } else {
            "[REDACTED]".to_string()
        };

        let re = Regex::new(pattern)
            .map_err(|e| custom_error(ctx, &format!("Invalid regex pattern: {}", e)))?;

        let result = re.replace_all(s, replacement.as_str());
        Ok(Value::String(result.into_owned()))
    }
}

// =============================================================================
// normalize_whitespace(string) -> string
// Collapse multiple whitespace characters into a single space
// =============================================================================

defn!(NormalizeWhitespaceFn, vec![arg!(string)], None);

impl Function for NormalizeWhitespaceFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Split on whitespace and rejoin with single spaces
        let result: String = s.split_whitespace().collect::<Vec<_>>().join(" ");

        Ok(Value::String(result))
    }
}

// =============================================================================
// is_blank(string) -> boolean
// Check if string is empty or contains only whitespace
// =============================================================================

defn!(IsBlankFn, vec![arg!(string)], None);

impl Function for IsBlankFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let is_blank = s.trim().is_empty();
        Ok(Value::Bool(is_blank))
    }
}

// =============================================================================
// abbreviate(string, max_length, suffix?) -> string
// Truncate string to max length with ellipsis suffix
// =============================================================================

defn!(
    AbbreviateFn,
    vec![arg!(string), arg!(number)],
    Some(arg!(any))
);

impl Function for AbbreviateFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let max_length = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for max_length"))?
            as usize;

        // Suffix (default: "...")
        let suffix = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "...".to_string())
        } else {
            "...".to_string()
        };

        let char_count = s.chars().count();
        let suffix_len = suffix.chars().count();

        if char_count <= max_length {
            return Ok(Value::String(s.to_string()));
        }

        if max_length <= suffix_len {
            // If max_length is too small for suffix, just truncate
            let result: String = s.chars().take(max_length).collect();
            return Ok(Value::String(result));
        }

        let truncate_at = max_length - suffix_len;
        let mut result: String = s.chars().take(truncate_at).collect();
        result.push_str(&suffix);

        Ok(Value::String(result))
    }
}

// =============================================================================
// center(string, width, char?) -> string
// Center-pad a string to the given width
// =============================================================================

defn!(CenterFn, vec![arg!(string), arg!(number)], Some(arg!(any)));

impl Function for CenterFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;
        let width = args[1]
            .as_f64()
            .ok_or_else(|| custom_error(ctx, "Expected number for width"))?
            as usize;
        if width > MAX_GENERATED_STRING_BYTES {
            return Err(custom_error(
                ctx,
                "center width exceeds the maximum allowed size",
            ));
        }

        // Padding character (default: ' ')
        let pad_char = if args.len() > 2 && !args[2].is_null() {
            args[2]
                .as_str()
                .and_then(|s| s.chars().next())
                .unwrap_or(' ')
        } else {
            ' '
        };

        let char_count = s.chars().count();

        if char_count >= width {
            return Ok(Value::String(s.to_string()));
        }

        let total_padding = width - char_count;
        let left_padding = total_padding / 2;
        let right_padding = total_padding - left_padding;

        let mut result = String::with_capacity(width);
        for _ in 0..left_padding {
            result.push(pad_char);
        }
        result.push_str(s);
        for _ in 0..right_padding {
            result.push(pad_char);
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// reverse_string(string) -> string
// Reverse a string (Unicode-aware, reverses grapheme clusters ideally, chars for simplicity)
// =============================================================================

defn!(ReverseStringFn, vec![arg!(string)], None);

impl Function for ReverseStringFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let result: String = s.chars().rev().collect();
        Ok(Value::String(result))
    }
}

// =============================================================================
// explode(string) -> array
// Convert a string to an array of Unicode codepoints (integers)
// =============================================================================

defn!(ExplodeFn, vec![arg!(string)], None);

impl Function for ExplodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        let codepoints: Vec<Value> = s
            .chars()
            .map(|c| Value::Number(Number::from(c as u32)))
            .collect();

        Ok(Value::Array(codepoints))
    }
}

// =============================================================================
// implode(array) -> string
// Convert an array of Unicode codepoints (integers) back to a string
// =============================================================================

defn!(ImplodeFn, vec![arg!(array)], None);

impl Function for ImplodeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let arr = args[0]
            .as_array()
            .ok_or_else(|| custom_error(ctx, "Expected array argument"))?;

        let mut result = String::new();
        for item in arr.iter() {
            let codepoint = item
                .as_f64()
                .ok_or_else(|| custom_error(ctx, "Expected array of numbers (codepoints)"))?
                as u32;

            let c = char::from_u32(codepoint).ok_or_else(|| {
                custom_error(ctx, &format!("Invalid Unicode codepoint: {}", codepoint))
            })?;
            result.push(c);
        }

        Ok(Value::String(result))
    }
}

// =============================================================================
// shell_escape(string) -> string
// Escape a string for safe use in shell commands (POSIX sh compatible)
// =============================================================================

defn!(ShellEscapeFn, vec![arg!(string)], None);

impl Function for ShellEscapeFn {
    fn evaluate(&self, args: &[Value], ctx: &mut Context<'_>) -> SearchResult {
        self.signature.validate(args, ctx)?;
        let s = args[0]
            .as_str()
            .ok_or_else(|| custom_error(ctx, "Expected string argument"))?;

        // Use single quotes for shell escaping (POSIX compatible)
        // If the string contains single quotes, we need to handle them specially:
        // 'it'\''s' becomes: 'it' + \' + 's' (end quote, escaped quote, start quote)
        let escaped = if s.is_empty() {
            "''".to_string()
        } else if s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
        {
            // Safe characters that don't need quoting
            s.to_string()
        } else if !s.contains('\'') {
            // No single quotes, just wrap in single quotes
            format!("'{}'", s)
        } else {
            // Contains single quotes - use the '\'' technique
            let mut result = String::with_capacity(s.len() + 10);
            result.push('\'');
            for c in s.chars() {
                if c == '\'' {
                    result.push_str("'\\''");
                } else {
                    result.push(c);
                }
            }
            result.push('\'');
            result
        };

        Ok(Value::String(escaped))
    }
}

// =============================================================================
// register_filtered
// =============================================================================

/// Register only the string functions that are in the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled(runtime, "lower", enabled, Box::new(LowerFn::new()));
    register_if_enabled(runtime, "upper", enabled, Box::new(UpperFn::new()));
    register_if_enabled(runtime, "trim", enabled, Box::new(TrimFn::new()));
    register_if_enabled(runtime, "trim_left", enabled, Box::new(TrimStartFn::new()));
    register_if_enabled(runtime, "trim_right", enabled, Box::new(TrimEndFn::new()));
    register_if_enabled(runtime, "split", enabled, Box::new(SplitFn::new()));
    register_if_enabled(runtime, "replace", enabled, Box::new(ReplaceFn::new()));
    register_if_enabled(runtime, "pad_left", enabled, Box::new(PadLeftFn::new()));
    register_if_enabled(runtime, "pad_right", enabled, Box::new(PadRightFn::new()));
    register_if_enabled(runtime, "substr", enabled, Box::new(SubstrFn::new()));
    register_if_enabled(
        runtime,
        "capitalize",
        enabled,
        Box::new(CapitalizeFn::new()),
    );
    register_if_enabled(runtime, "title", enabled, Box::new(TitleFn::new()));
    register_if_enabled(runtime, "repeat", enabled, Box::new(RepeatFn::new()));
    register_if_enabled(runtime, "find_first", enabled, Box::new(IndexOfFn::new()));
    register_if_enabled(
        runtime,
        "find_last",
        enabled,
        Box::new(LastIndexOfFn::new()),
    );
    register_if_enabled(runtime, "slice", enabled, Box::new(SliceFn::new()));
    register_if_enabled(runtime, "concat", enabled, Box::new(ConcatFn::new()));
    register_if_enabled(runtime, "upper_case", enabled, Box::new(UpperCaseFn::new()));
    register_if_enabled(runtime, "lower_case", enabled, Box::new(LowerCaseFn::new()));
    register_if_enabled(runtime, "title_case", enabled, Box::new(TitleCaseFn::new()));
    register_if_enabled(runtime, "camel_case", enabled, Box::new(CamelCaseFn::new()));
    register_if_enabled(runtime, "snake_case", enabled, Box::new(SnakeCaseFn::new()));
    register_if_enabled(runtime, "kebab_case", enabled, Box::new(KebabCaseFn::new()));
    register_if_enabled(
        runtime,
        "pascal_case",
        enabled,
        Box::new(PascalCaseFn::new()),
    );
    register_if_enabled(
        runtime,
        "shouty_snake_case",
        enabled,
        Box::new(ShoutySnakeCaseFn::new()),
    );
    register_if_enabled(
        runtime,
        "shouty_kebab_case",
        enabled,
        Box::new(ShoutyKebabCaseFn::new()),
    );
    register_if_enabled(runtime, "train_case", enabled, Box::new(TrainCaseFn::new()));
    register_if_enabled(runtime, "truncate", enabled, Box::new(TruncateFn::new()));
    register_if_enabled(runtime, "wrap", enabled, Box::new(WrapFn::new()));
    register_if_enabled(runtime, "format", enabled, Box::new(FormatFn::new()));
    register_if_enabled(runtime, "sprintf", enabled, Box::new(SprintfFn::new()));
    register_if_enabled(runtime, "ltrimstr", enabled, Box::new(LtrimstrFn::new()));
    register_if_enabled(runtime, "rtrimstr", enabled, Box::new(RtrimstrFn::new()));
    register_if_enabled(runtime, "indices", enabled, Box::new(IndicesFn::new()));
    register_if_enabled(runtime, "inside", enabled, Box::new(InsideFn::new()));
    register_if_enabled(runtime, "humanize", enabled, Box::new(HumanizeFn::new()));
    register_if_enabled(runtime, "deburr", enabled, Box::new(DeburrrFn::new()));
    register_if_enabled(runtime, "words", enabled, Box::new(WordsFn::new()));
    register_if_enabled(runtime, "escape", enabled, Box::new(EscapeFn::new()));
    register_if_enabled(runtime, "unescape", enabled, Box::new(UnescapeFn::new()));
    register_if_enabled(
        runtime,
        "escape_regex",
        enabled,
        Box::new(EscapeRegexFn::new()),
    );
    register_if_enabled(runtime, "start_case", enabled, Box::new(StartCaseFn::new()));
    register_if_enabled(runtime, "mask", enabled, Box::new(MaskFn::new()));
    register_if_enabled(runtime, "redact", enabled, Box::new(RedactFn::new()));
    register_if_enabled(
        runtime,
        "normalize_whitespace",
        enabled,
        Box::new(NormalizeWhitespaceFn::new()),
    );
    register_if_enabled(runtime, "is_blank", enabled, Box::new(IsBlankFn::new()));
    register_if_enabled(
        runtime,
        "abbreviate",
        enabled,
        Box::new(AbbreviateFn::new()),
    );
    register_if_enabled(runtime, "center", enabled, Box::new(CenterFn::new()));
    register_if_enabled(
        runtime,
        "reverse_string",
        enabled,
        Box::new(ReverseStringFn::new()),
    );
    register_if_enabled(runtime, "explode", enabled, Box::new(ExplodeFn::new()));
    register_if_enabled(runtime, "implode", enabled, Box::new(ImplodeFn::new()));
    register_if_enabled(
        runtime,
        "shell_escape",
        enabled,
        Box::new(ShellEscapeFn::new()),
    );
}

#[cfg(test)]
mod tests {
    use crate::Runtime;
    use serde_json::json;

    fn setup_runtime() -> Runtime {
        Runtime::builder()
            .with_standard()
            .with_all_extensions()
            .build()
    }

    #[test]
    fn test_lower() {
        let runtime = setup_runtime();
        let expr = runtime.compile("lower(@)").unwrap();
        let result = expr.search(&json!("HELLO")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_repeat_rejects_excessive_output() {
        // A petabyte-sized repeat must error instead of attempting the alloc.
        let runtime = setup_runtime();
        let expr = runtime.compile("repeat('x', `1000000000000`)").unwrap();
        assert!(expr.search(&json!(null)).is_err());
        // A normal repeat still works.
        let expr = runtime.compile("repeat('ab', `3`)").unwrap();
        assert_eq!(expr.search(&json!(null)).unwrap(), json!("ababab"));
    }

    #[test]
    fn test_upper() {
        let runtime = setup_runtime();
        let expr = runtime.compile("upper(@)").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        assert_eq!(result.as_str().unwrap(), "HELLO");
    }

    #[test]
    fn test_trim() {
        let runtime = setup_runtime();
        let expr = runtime.compile("trim(@)").unwrap();
        let result = expr.search(&json!("  hello  ")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_split() {
        let runtime = setup_runtime();
        let expr = runtime.compile("split(@, ',')").unwrap();
        let result = expr.search(&json!("a,b,c")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str().unwrap(), "a");
    }

    #[test]
    fn test_camel_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("camel_case(@)").unwrap();
        let result = expr.search(&json!("hello_world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "helloWorld");
    }

    #[test]
    fn test_snake_case() {
        let runtime = setup_runtime();
        let expr = runtime.compile("snake_case(@)").unwrap();
        let result = expr.search(&json!("helloWorld")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello_world");
    }

    #[test]
    fn test_wrap_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `5`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_wrap_preserves_newlines() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `100`)").unwrap();
        let result = expr.search(&json!("hello\nworld")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello\nworld");
    }

    #[test]
    fn test_wrap_wide_width() {
        let runtime = setup_runtime();
        let expr = runtime.compile("wrap(@, `100`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_ltrimstr() {
        let runtime = setup_runtime();
        let expr = runtime.compile("ltrimstr(@, 'hello ')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "world");
    }

    #[test]
    fn test_ltrimstr_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("ltrimstr(@, 'foo')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_rtrimstr() {
        let runtime = setup_runtime();
        let expr = runtime.compile("rtrimstr(@, ' world')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_rtrimstr_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("rtrimstr(@, 'foo')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_indices() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'l')").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_f64().unwrap() as i64, 2);
        assert_eq!(arr[1].as_f64().unwrap() as i64, 3);
    }

    #[test]
    fn test_indices_no_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'x')").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_indices_overlapping() {
        let runtime = setup_runtime();
        let expr = runtime.compile("indices(@, 'aa')").unwrap();
        let result = expr.search(&json!("aaa")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_f64().unwrap() as i64, 0);
        assert_eq!(arr[1].as_f64().unwrap() as i64, 1);
    }

    #[test]
    fn test_inside() {
        let runtime = setup_runtime();
        let expr = runtime.compile("inside('world', @)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_inside_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("inside('foo', @)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    #[test]
    fn test_sprintf_string() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Hello, %s!', @)").unwrap();
        let result = expr.search(&json!("World")).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_sprintf_integer() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('%d + %d = %d', @)").unwrap();
        let data = json!([1, 2, 3]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "1 + 2 = 3");
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_sprintf_float_precision() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Pi is %.2f', @)").unwrap();
        let data = json!(3.14159);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Pi is 3.14");
    }

    #[test]
    fn test_sprintf_hex() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('Hex: %x', @)").unwrap();
        let data = json!(255);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hex: ff");
    }

    #[test]
    fn test_sprintf_width() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('%10s', @)").unwrap();
        let result = expr.search(&json!("hi")).unwrap();
        assert_eq!(result.as_str().unwrap(), "        hi");
    }

    #[test]
    fn test_sprintf_escaped_percent() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sprintf('100%% done', @)").unwrap();
        let result = expr.search(&json!(null)).unwrap();
        assert_eq!(result.as_str().unwrap(), "100% done");
    }

    // JEP-014 find_first tests
    #[test]
    fn test_find_first_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'world')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 6);
    }

    #[test]
    fn test_find_first_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'xyz')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_find_first_with_start() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `5`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 7);
    }

    #[test]
    fn test_find_first_with_start_and_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `0`, `5`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 4);
    }

    #[test]
    fn test_find_first_with_negative_start() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_first(@, 'o', `-5`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 7);
    }

    // JEP-014 find_last tests
    #[test]
    fn test_find_last_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'o')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 7);
    }

    #[test]
    fn test_find_last_not_found() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'xyz')").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_find_last_with_start_and_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'o', `0`, `6`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 4);
    }

    #[test]
    fn test_find_last_with_negative_end() {
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last(@, 'l', `0`, `-1`)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_f64().unwrap() as i64, 9);
    }

    // =========================================================================
    // mask tests
    // Note: In jpx-core the object module's mask (with default show_last=4)
    // takes precedence over the string module's mask.
    // =========================================================================

    #[test]
    fn test_mask_default() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@)").unwrap();
        // Default shows last 4; "secret" (6 chars) -> mask 2, show 4
        let result = expr.search(&json!("secret")).unwrap();
        assert_eq!(result.as_str().unwrap(), "**cret");
    }

    #[test]
    fn test_mask_keep_last_4() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@, `4`)").unwrap();
        let result = expr.search(&json!("4111111111111111")).unwrap();
        assert_eq!(result.as_str().unwrap(), "************1111");
    }

    #[test]
    fn test_mask_visible_exceeds_length() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mask(@, `10`)").unwrap();
        // When show_last >= length, object mask returns all stars
        let result = expr.search(&json!("short")).unwrap();
        assert_eq!(result.as_str().unwrap(), "*****");
    }

    #[test]
    fn test_mask_multibyte() {
        let runtime = setup_runtime();
        // "héllo" is 5 chars / 6 bytes. show_last=4 -> mask 1 char. A byte-based
        // slice at byte 2 would land inside "é" and panic; char-based is safe.
        let expr = runtime.compile("mask(@, `4`)").unwrap();
        let result = expr.search(&json!("héllo")).unwrap();
        assert_eq!(result.as_str().unwrap(), "*éllo");
    }

    #[test]
    fn test_find_first_multibyte() {
        // IndexOfFn is registered as `find_first`.
        let runtime = setup_runtime();
        // Character indices: h=0, é=1, l=2, l=3, o=4.
        let expr = runtime.compile("find_first('héllo', 'l')").unwrap();
        assert_eq!(expr.search(&json!(null)).unwrap(), json!(2));
        // start/end window that previously sliced mid-"é" and panicked.
        let expr = runtime
            .compile("find_first('héllo', 'l', `1`, `5`)")
            .unwrap();
        assert_eq!(expr.search(&json!(null)).unwrap(), json!(2));
    }

    #[test]
    fn test_find_last_multibyte() {
        // LastIndexOfFn is registered as `find_last`.
        let runtime = setup_runtime();
        let expr = runtime.compile("find_last('héllo', 'l')").unwrap();
        assert_eq!(expr.search(&json!(null)).unwrap(), json!(3));
    }

    // =========================================================================
    // normalize_whitespace tests
    // =========================================================================

    #[test]
    fn test_normalize_whitespace_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let result = expr.search(&json!("hello    world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_normalize_whitespace_mixed() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let result = expr.search(&json!("hello\t\n  world\n\nfoo")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world foo");
    }

    #[test]
    fn test_normalize_whitespace_leading_trailing() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize_whitespace(@)").unwrap();
        let result = expr.search(&json!("  hello world  ")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    // =========================================================================
    // is_blank tests
    // =========================================================================

    #[test]
    fn test_is_blank_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let result = expr.search(&json!("")).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_blank_whitespace() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let result = expr.search(&json!("   \t\n  ")).unwrap();
        assert!(result.as_bool().unwrap());
    }

    #[test]
    fn test_is_blank_not_blank() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_blank(@)").unwrap();
        let result = expr.search(&json!("  a  ")).unwrap();
        assert!(!result.as_bool().unwrap());
    }

    // =========================================================================
    // abbreviate tests
    // =========================================================================

    #[test]
    fn test_abbreviate_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `10`)").unwrap();
        let result = expr.search(&json!("This is a very long string")).unwrap();
        assert_eq!(result.as_str().unwrap(), "This is...");
    }

    #[test]
    fn test_abbreviate_no_truncation() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `20`)").unwrap();
        let result = expr.search(&json!("short")).unwrap();
        assert_eq!(result.as_str().unwrap(), "short");
    }

    #[test]
    fn test_abbreviate_custom_suffix() {
        let runtime = setup_runtime();
        let expr = runtime.compile("abbreviate(@, `8`, `\"~\"`)").unwrap();
        let result = expr.search(&json!("Hello World")).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello W~");
    }

    // =========================================================================
    // center tests
    // =========================================================================

    #[test]
    fn test_center_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `10`)").unwrap();
        let result = expr.search(&json!("hi")).unwrap();
        assert_eq!(result.as_str().unwrap(), "    hi    ");
    }

    #[test]
    fn test_center_custom_char() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `10`, `\"-\"`)").unwrap();
        let result = expr.search(&json!("hi")).unwrap();
        assert_eq!(result.as_str().unwrap(), "----hi----");
    }

    #[test]
    fn test_center_already_wide() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `3`)").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_center_odd_padding() {
        let runtime = setup_runtime();
        let expr = runtime.compile("center(@, `7`)").unwrap();
        let result = expr.search(&json!("hi")).unwrap();
        assert_eq!(result.as_str().unwrap(), "  hi   ");
    }

    // =========================================================================
    // reverse_string tests
    // =========================================================================

    #[test]
    fn test_reverse_string_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        assert_eq!(result.as_str().unwrap(), "olleh");
    }

    #[test]
    fn test_reverse_string_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let result = expr.search(&json!("")).unwrap();
        assert_eq!(result.as_str().unwrap(), "");
    }

    #[test]
    fn test_reverse_string_palindrome() {
        let runtime = setup_runtime();
        let expr = runtime.compile("reverse_string(@)").unwrap();
        let result = expr.search(&json!("racecar")).unwrap();
        assert_eq!(result.as_str().unwrap(), "racecar");
    }

    // =========================================================================
    // explode tests
    // =========================================================================

    #[test]
    fn test_explode_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("explode(@)").unwrap();
        let result = expr.search(&json!("abc")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap() as u32, 97); // 'a'
        assert_eq!(arr[1].as_f64().unwrap() as u32, 98); // 'b'
        assert_eq!(arr[2].as_f64().unwrap() as u32, 99); // 'c'
    }

    #[test]
    fn test_explode_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("explode(@)").unwrap();
        let result = expr.search(&json!("")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_explode_unicode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("explode(@)").unwrap();
        let result = expr.search(&json!("A\u{263a}")).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_f64().unwrap() as u32, 65); // 'A'
        assert_eq!(arr[1].as_f64().unwrap() as u32, 9786); // smiley
    }

    // =========================================================================
    // implode tests
    // =========================================================================

    #[test]
    fn test_implode_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("implode(@)").unwrap();
        let data = json!([97, 98, 99]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "abc");
    }

    #[test]
    fn test_implode_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("implode(@)").unwrap();
        let data: serde_json::Value = serde_json::from_str("[]").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "");
    }

    #[test]
    fn test_implode_unicode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("implode(@)").unwrap();
        let data = json!([65, 9786]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_str().unwrap(), "A\u{263a}");
    }

    #[test]
    fn test_explode_implode_roundtrip() {
        let runtime = setup_runtime();
        let expr = runtime.compile("implode(explode(@))").unwrap();
        let result = expr.search(&json!("Hello, \u{4e16}\u{754c}!")).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello, \u{4e16}\u{754c}!");
    }

    // =========================================================================
    // shell_escape tests
    // =========================================================================

    #[test]
    fn test_shell_escape_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("hello")).unwrap();
        // Simple alphanumeric doesn't need quoting
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_shell_escape_with_spaces() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("hello world")).unwrap();
        assert_eq!(result.as_str().unwrap(), "'hello world'");
    }

    #[test]
    fn test_shell_escape_with_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("$PATH; rm -rf /")).unwrap();
        assert_eq!(result.as_str().unwrap(), "'$PATH; rm -rf /'");
    }

    #[test]
    fn test_shell_escape_with_single_quote() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("it's")).unwrap();
        // Single quotes are escaped as '\''
        assert_eq!(result.as_str().unwrap(), "'it'\\''s'");
    }

    #[test]
    fn test_shell_escape_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("")).unwrap();
        assert_eq!(result.as_str().unwrap(), "''");
    }

    #[test]
    fn test_shell_escape_path() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("/usr/local/bin")).unwrap();
        // Paths with only safe chars don't need quoting
        assert_eq!(result.as_str().unwrap(), "/usr/local/bin");
    }

    #[test]
    fn test_shell_escape_backticks() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shell_escape(@)").unwrap();
        let result = expr.search(&json!("`whoami`")).unwrap();
        assert_eq!(result.as_str().unwrap(), "'`whoami`'");
    }
}
