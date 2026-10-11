//! Bounded structural inspection of JSON documents.
//!
//! [`inspect_value`] walks a parsed document in deterministic preorder and
//! returns one record per visited node: its RFC 6901 JSON Pointer, JSON type
//! and depth. Leaf values are never copied into the report. The walk stops as
//! soon as a caller limit is reached, so the work done after parsing is bounded
//! by the limits rather than by the size of the document.
//!
//! # Ordering
//!
//! Object members are visited in the parsed map's iteration order: insertion
//! (document) order when serde_json's `preserve_order` feature is enabled, as
//! it is in the jpx MCP server, and sorted key order otherwise. Only the
//! members actually returned are read from the map; keys are never globally
//! sorted or scanned. Arrays contribute a prefix sample of their elements. The
//! same input in the same build always produces the same report.
//!
//! # Byte budget
//!
//! `max_bytes` bounds the report's compact JSON *after* it has been escaped as
//! the body of a JSON string, plus a caller-supplied `framing_bytes` overhead.
//! Transports that embed the report as JSON text inside another JSON document
//! (such as an MCP `CallToolResult`) can pass their fixed wrapper size as the
//! overhead and get an exact bound on the serialized wrapper. With an overhead
//! of zero the budget is an upper bound on the report's own compact JSON.
//!
//! Parsing the input is not covered by these limits: the whole document is
//! parsed (and allocated) before the walk begins.
//!
//! # Example
//!
//! ```rust
//! use jpx_engine::{InspectLimits, inspect_value};
//! use serde_json::json;
//!
//! let doc = json!({"a/b": {"secret": "hunter2"}});
//! let limits = InspectLimits::new(3, 100, 2, 16384).unwrap();
//! let report = inspect_value(&doc, &limits, 0).unwrap();
//!
//! let pointers: Vec<&str> = report.paths.iter().map(|p| p.pointer.as_str()).collect();
//! assert_eq!(pointers, ["", "/a~1b", "/a~1b/secret"]);
//! assert!(report.complete);
//! ```

use crate::JpxEngine;
use crate::error::{EngineError, Result};
use serde::Serialize;
use serde_json::{Value, map};
use std::iter::Enumerate;
use std::slice;

/// Default maximum depth below the root that is visited.
pub const INSPECT_DEFAULT_MAX_DEPTH: usize = 3;
/// Default maximum number of path records, including the root.
pub const INSPECT_DEFAULT_MAX_PATHS: usize = 100;
/// Default number of leading elements visited per array.
pub const INSPECT_DEFAULT_ARRAY_SAMPLE: usize = 3;
/// Default byte budget for the serialized report.
pub const INSPECT_DEFAULT_MAX_BYTES: usize = 16384;

/// Hard maximum for `max_depth`.
pub const INSPECT_MAX_DEPTH: usize = 32;
/// Hard maximum for `max_paths`.
pub const INSPECT_MAX_PATHS: usize = 1000;
/// Hard maximum for `array_sample`.
pub const INSPECT_MAX_ARRAY_SAMPLE: usize = 100;
/// Hard maximum for `max_bytes`.
pub const INSPECT_MAX_BYTES: usize = 65536;

/// Minimum accepted `max_paths` (the root is always a returned path).
pub const INSPECT_MIN_PATHS: usize = 1;
/// Minimum accepted `max_bytes`.
pub const INSPECT_MIN_BYTES: usize = 512;

/// Validated limits for [`inspect_value`].
///
/// Fields are private so every instance is within the supported ranges; build
/// one with [`InspectLimits::new`], [`InspectLimits::from_json_args`] or
/// [`Default`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct InspectLimits {
    max_depth: usize,
    max_paths: usize,
    array_sample: usize,
    max_bytes: usize,
}

impl Default for InspectLimits {
    fn default() -> Self {
        Self {
            max_depth: INSPECT_DEFAULT_MAX_DEPTH,
            max_paths: INSPECT_DEFAULT_MAX_PATHS,
            array_sample: INSPECT_DEFAULT_ARRAY_SAMPLE,
            max_bytes: INSPECT_DEFAULT_MAX_BYTES,
        }
    }
}

/// Name, accepted range and default of one inspection limit.
struct LimitSpec {
    name: &'static str,
    min: usize,
    max: usize,
    default: usize,
}

const DEPTH_SPEC: LimitSpec = LimitSpec {
    name: "max_depth",
    min: 0,
    max: INSPECT_MAX_DEPTH,
    default: INSPECT_DEFAULT_MAX_DEPTH,
};
const PATHS_SPEC: LimitSpec = LimitSpec {
    name: "max_paths",
    min: INSPECT_MIN_PATHS,
    max: INSPECT_MAX_PATHS,
    default: INSPECT_DEFAULT_MAX_PATHS,
};
const SAMPLE_SPEC: LimitSpec = LimitSpec {
    name: "array_sample",
    min: 0,
    max: INSPECT_MAX_ARRAY_SAMPLE,
    default: INSPECT_DEFAULT_ARRAY_SAMPLE,
};
const BYTES_SPEC: LimitSpec = LimitSpec {
    name: "max_bytes",
    min: INSPECT_MIN_BYTES,
    max: INSPECT_MAX_BYTES,
    default: INSPECT_DEFAULT_MAX_BYTES,
};

impl LimitSpec {
    fn invalid(&self, got: &str) -> EngineError {
        EngineError::InvalidLimit(format!(
            "{} must be a whole number from {} to {} (default {}); got {got}. \
             Pass a value in that range or omit {} to use the default.",
            self.name, self.min, self.max, self.default, self.name
        ))
    }

    fn check(&self, value: usize) -> Result<usize> {
        if (self.min..=self.max).contains(&value) {
            Ok(value)
        } else {
            Err(self.invalid(&value.to_string()))
        }
    }

    fn parse(&self, value: Option<&Value>) -> Result<usize> {
        match value {
            None => Ok(self.default),
            Some(Value::Number(n)) => {
                if let Some(v) = n.as_u64() {
                    usize::try_from(v)
                        .map_err(|_| self.invalid(&v.to_string()))
                        .and_then(|v| self.check(v))
                } else if let Some(v) = n.as_i64() {
                    Err(self.invalid(&v.to_string()))
                } else {
                    Err(self.invalid("a non-integer number"))
                }
            }
            Some(Value::Null) => Err(self.invalid("null")),
            Some(other) => Err(self.invalid(&format!("a {}", json_type(other)))),
        }
    }
}

impl InspectLimits {
    /// Create limits, rejecting values outside the supported ranges.
    pub fn new(
        max_depth: usize,
        max_paths: usize,
        array_sample: usize,
        max_bytes: usize,
    ) -> Result<Self> {
        Ok(Self {
            max_depth: DEPTH_SPEC.check(max_depth)?,
            max_paths: PATHS_SPEC.check(max_paths)?,
            array_sample: SAMPLE_SPEC.check(array_sample)?,
            max_bytes: BYTES_SPEC.check(max_bytes)?,
        })
    }

    /// Create limits from optional JSON arguments.
    ///
    /// `None` (an absent argument) selects the default. Explicit `null`,
    /// negative, fractional, non-numeric and out-of-range values are rejected
    /// with a message naming the valid range and default.
    pub fn from_json_args(
        max_depth: Option<&Value>,
        max_paths: Option<&Value>,
        array_sample: Option<&Value>,
        max_bytes: Option<&Value>,
    ) -> Result<Self> {
        Ok(Self {
            max_depth: DEPTH_SPEC.parse(max_depth)?,
            max_paths: PATHS_SPEC.parse(max_paths)?,
            array_sample: SAMPLE_SPEC.parse(array_sample)?,
            max_bytes: BYTES_SPEC.parse(max_bytes)?,
        })
    }

    /// Deepest level whose nodes are returned (root is depth 0).
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Maximum number of returned path records, including the root.
    pub fn max_paths(&self) -> usize {
        self.max_paths
    }

    /// Number of leading elements visited in each array.
    pub fn array_sample(&self) -> usize {
        self.array_sample
    }

    /// Byte budget for the escaped report plus framing overhead.
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }
}

/// Why a returned node's children were not all visited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectTruncation {
    /// The node sits at `max_depth`; none of its children were visited.
    Depth,
    /// The array is longer than `array_sample`; only a prefix was visited.
    ArraySample,
}

/// One structural record. Never carries leaf values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectNode {
    /// RFC 6901 JSON Pointer (`""` is the root).
    pub pointer: String,
    /// JSON type: object, array, string, number, boolean or null.
    #[serde(rename = "type")]
    pub json_type: &'static str,
    /// Distance from the root (root is 0).
    pub depth: usize,
    /// Member or element count, for objects and arrays.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// Set when this node's children were skipped by depth or array sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<InspectTruncation>,
}

/// Which limits actually caused structural nodes to be omitted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct InspectOmitted {
    /// Children of nodes at `max_depth` were skipped.
    pub depth: bool,
    /// Further nodes existed after `max_paths` records.
    pub paths: bool,
    /// Array elements beyond `array_sample` were skipped.
    pub array_sample: bool,
    /// The next record did not fit in `max_bytes`.
    pub bytes: bool,
}

impl InspectOmitted {
    fn any(&self) -> bool {
        self.depth || self.paths || self.array_sample || self.bytes
    }
}

/// Result of a bounded structural inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectReport {
    /// True when every structural node in the document is in `paths`.
    pub complete: bool,
    /// Number of records in `paths` (exact; no totals are estimated).
    pub returned_paths: usize,
    /// Limits that actually omitted nodes.
    pub omitted: InspectOmitted,
    /// The limits that were applied.
    pub limits: InspectLimits,
    /// Records in deterministic preorder.
    pub paths: Vec<InspectNode>,
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Length of `s` once escaped as the body of a JSON string by serde_json.
fn escaped_len(s: &str) -> usize {
    s.bytes()
        .map(|b| match b {
            b'"' | b'\\' | b'\x08' | b'\x0c' | b'\n' | b'\r' | b'\t' => 2,
            0x00..=0x1f => 6,
            _ => 1,
        })
        .sum()
}

fn serialized_escaped_len(value: &impl Serialize) -> Result<usize> {
    let json = serde_json::to_string(value).map_err(|e| EngineError::Internal(e.to_string()))?;
    Ok(escaped_len(&json))
}

fn push_pointer_segment(pointer: &mut String, segment: &str) {
    pointer.push('/');
    for c in segment.chars() {
        match c {
            '~' => pointer.push_str("~0"),
            '/' => pointer.push_str("~1"),
            c => pointer.push(c),
        }
    }
}

fn digits(mut n: usize) -> usize {
    let mut d = 1;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// Borrowed iterator over the children of a returned container that are
/// still eligible for visiting. Entries are pulled one at a time, only when
/// they are about to be returned.
enum Children<'a> {
    /// Array prefix (already cut to `array_sample`).
    Array(Enumerate<slice::Iter<'a, Value>>),
    /// Object members in map iteration order.
    Object(map::Iter<'a>),
}

impl Children<'_> {
    fn remaining(&self) -> usize {
        match self {
            Children::Array(items) => items.len(),
            Children::Object(members) => members.len(),
        }
    }
}

struct Frame<'a> {
    pointer: String,
    depth: usize,
    children: Children<'a>,
}

/// Mutable state for one bounded walk.
struct Walker<'a> {
    limits: InspectLimits,
    /// Fixed bytes outside the escaped records (wrapper plus envelope).
    fixed_bytes: usize,
    records_bytes: usize,
    report: InspectReport,
    stack: Vec<Frame<'a>>,
    /// Number of child entries pulled from container iterators.
    entries_read: usize,
}

impl<'a> Walker<'a> {
    /// Returns true when a record of `cost` escaped bytes still fits.
    fn fits(&self, cost: usize) -> bool {
        let count = self.report.paths.len() + 1;
        let commas = count - 1;
        self.fixed_bytes
            .saturating_add(digits(count))
            .saturating_add(self.records_bytes)
            .saturating_add(commas)
            .saturating_add(cost)
            <= self.limits.max_bytes
    }

    /// Records `value` at `pointer` if it fits in the byte budget and queues
    /// its eligible children. Returns false when the budget is exhausted.
    fn emit(&mut self, value: &'a Value, pointer: String, depth: usize) -> Result<bool> {
        let length = match value {
            Value::Array(a) => Some(a.len()),
            Value::Object(o) => Some(o.len()),
            _ => None,
        };
        let truncated = match value {
            _ if length.unwrap_or(0) == 0 => None,
            _ if depth >= self.limits.max_depth => Some(InspectTruncation::Depth),
            Value::Array(a) if a.len() > self.limits.array_sample => {
                Some(InspectTruncation::ArraySample)
            }
            _ => None,
        };
        let node = InspectNode {
            pointer,
            json_type: json_type(value),
            depth,
            length,
            truncated,
        };
        let cost = serialized_escaped_len(&node)?;
        if !self.fits(cost) {
            return Ok(false);
        }
        self.records_bytes += cost;

        let children = match (value, truncated) {
            (_, Some(InspectTruncation::Depth)) => None,
            (Value::Array(a), _) => {
                let sample = a.len().min(self.limits.array_sample);
                (sample > 0).then(|| Children::Array(a[..sample].iter().enumerate()))
            }
            (Value::Object(o), _) if !o.is_empty() => Some(Children::Object(o.iter())),
            _ => None,
        };
        match truncated {
            Some(InspectTruncation::Depth) => self.report.omitted.depth = true,
            Some(InspectTruncation::ArraySample) => self.report.omitted.array_sample = true,
            None => {}
        }
        if let Some(children) = children {
            self.stack.push(Frame {
                pointer: node.pointer.clone(),
                depth,
                children,
            });
        }
        self.report.paths.push(node);
        Ok(true)
    }

    /// Visits the next queued child. Returns false when the walk is over.
    fn step(&mut self) -> Result<bool> {
        let Some(frame) = self.stack.last_mut() else {
            return Ok(false);
        };
        if frame.children.remaining() == 0 {
            self.stack.pop();
            return Ok(true);
        }
        // A child exists but there is no allowance left to return it.
        if self.report.paths.len() >= self.limits.max_paths {
            self.report.omitted.paths = true;
            return Ok(false);
        }

        let depth = frame.depth + 1;
        let (child, key, index) = match &mut frame.children {
            Children::Array(items) => match items.next() {
                Some((index, child)) => (child, None, index),
                None => return Ok(true),
            },
            Children::Object(members) => match members.next() {
                Some((key, child)) => (child, Some(key.as_str()), 0),
                None => return Ok(true),
            },
        };
        self.entries_read += 1;
        // A pointer longer than the whole budget can never fit; don't build it.
        let segment_len = key.map_or_else(|| digits(index), str::len);
        if frame.pointer.len() + 1 + segment_len > self.limits.max_bytes {
            self.report.omitted.bytes = true;
            return Ok(false);
        }
        let mut pointer = frame.pointer.clone();
        match key {
            Some(key) => push_pointer_segment(&mut pointer, key),
            None => push_pointer_segment(&mut pointer, &index.to_string()),
        }
        if !self.emit(child, pointer, depth)? {
            self.report.omitted.bytes = true;
            return Ok(false);
        }
        Ok(true)
    }
}

/// Runs the walk and also returns how many child entries were read from
/// container iterators, so tests can check traversal stays bounded.
fn walk(
    value: &Value,
    limits: &InspectLimits,
    framing_bytes: usize,
) -> Result<(InspectReport, usize)> {
    let report = InspectReport {
        complete: false,
        returned_paths: 0,
        omitted: InspectOmitted::default(),
        limits: *limits,
        paths: Vec::new(),
    };
    // With `complete: false` and every omission flag false, this envelope is
    // exactly one byte wider than any envelope that can be returned: a
    // finished report either sets `complete` to true or sets at least one
    // omission flag, each turning a "false" into the one-byte-shorter "true".
    // The placeholder `returned_paths: 0` is replaced by the exact digit count.
    let envelope = serialized_escaped_len(&report)? - 1 - digits(0);
    let mut walker = Walker {
        limits: *limits,
        fixed_bytes: framing_bytes.saturating_add(envelope),
        records_bytes: 0,
        report,
        stack: Vec::new(),
        entries_read: 0,
    };

    if !walker.emit(value, String::new(), 0)? {
        return Err(EngineError::InvalidLimit(format!(
            "max_bytes {} is too small to hold the inspection envelope and root record. \
             Increase max_bytes (up to {INSPECT_MAX_BYTES}).",
            limits.max_bytes
        )));
    }
    while walker.step()? {}

    let mut report = walker.report;
    report.returned_paths = report.paths.len();
    report.complete = !report.omitted.any();
    Ok((report, walker.entries_read))
}

/// Walk `value` in bounded preorder and return structural records.
///
/// `framing_bytes` is the fixed size of any wrapper the escaped report will be
/// embedded in (see the [module docs](self)). Returns
/// [`EngineError::InvalidLimit`] if `max_bytes` cannot hold even the root
/// record.
pub fn inspect_value(
    value: &Value,
    limits: &InspectLimits,
    framing_bytes: usize,
) -> Result<InspectReport> {
    walk(value, limits, framing_bytes).map(|(report, _)| report)
}

impl JpxEngine {
    /// Parse `input` and run a bounded structural inspection.
    ///
    /// See [`inspect_value`] for the walk and budget semantics. Parsing is
    /// bounded only by the caller's input size checks, not by `limits`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use jpx_engine::{InspectLimits, JpxEngine};
    ///
    /// let engine = JpxEngine::new();
    /// let report = engine
    ///     .inspect(r#"{"user": {"name": "alice"}}"#, &InspectLimits::default(), 0)
    ///     .unwrap();
    /// assert!(report.complete);
    /// assert_eq!(report.paths[2].pointer, "/user/name");
    /// ```
    pub fn inspect(
        &self,
        input: &str,
        limits: &InspectLimits,
        framing_bytes: usize,
    ) -> Result<InspectReport> {
        let value: Value = serde_json::from_str(input).map_err(|e| {
            EngineError::InvalidJson(format!(
                "{e}. Pass the document as valid JSON text, e.g. {{\"a\": [1, 2]}}; \
                 string documents must be quoted."
            ))
        })?;
        inspect_value(&value, limits, framing_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, json};

    const SECRET: &str = "SECRET-LEAF-VALUE-8f2c";

    fn limits(max_depth: usize, max_paths: usize, array_sample: usize) -> InspectLimits {
        InspectLimits::new(max_depth, max_paths, array_sample, INSPECT_MAX_BYTES).unwrap()
    }

    fn run(value: &Value, limits: &InspectLimits) -> InspectReport {
        let (report, entries) = walk(value, limits, 0).unwrap();
        assert_eq!(report.returned_paths, report.paths.len());
        // Every child read from a container is either returned or is the one
        // whose omission ended the walk.
        assert!(entries <= report.returned_paths, "read {entries} entries");
        let text = serde_json::to_string(&report).unwrap();
        assert!(escaped_len(&text) <= limits.max_bytes);
        assert!(!text.contains(SECRET), "leaf value leaked: {text}");
        report
    }

    fn pointers(report: &InspectReport) -> Vec<&str> {
        report.paths.iter().map(|p| p.pointer.as_str()).collect()
    }

    fn only(f: impl FnOnce(&mut InspectOmitted)) -> InspectOmitted {
        let mut omitted = InspectOmitted::default();
        f(&mut omitted);
        omitted
    }

    #[test]
    fn defaults_match_documented_values() {
        let d = InspectLimits::default();
        assert_eq!(
            (
                d.max_depth(),
                d.max_paths(),
                d.array_sample(),
                d.max_bytes()
            ),
            (3, 100, 3, 16384)
        );
        assert_eq!(
            InspectLimits::from_json_args(None, None, None, None).unwrap(),
            d
        );
    }

    #[test]
    fn scalar_roots_are_complete() {
        for (value, ty) in [
            (json!(SECRET), "string"),
            (json!(42), "number"),
            (json!(-1.5), "number"),
            (json!(true), "boolean"),
            (json!(null), "null"),
        ] {
            for depth in [0, 3] {
                let report = run(&value, &limits(depth, 1, 0));
                assert!(report.complete, "{value}");
                assert_eq!(report.omitted, InspectOmitted::default());
                assert_eq!(
                    report.paths,
                    vec![InspectNode {
                        pointer: String::new(),
                        json_type: ty,
                        depth: 0,
                        length: None,
                        truncated: None,
                    }]
                );
            }
        }
    }

    #[test]
    fn empty_containers_are_complete_at_zero_limits() {
        for (value, ty) in [(json!({}), "object"), (json!([]), "array")] {
            let report = run(&value, &limits(0, 1, 0));
            assert!(report.complete);
            assert_eq!(report.paths[0].json_type, ty);
            assert_eq!(report.paths[0].length, Some(0));
            assert_eq!(report.paths[0].truncated, None);
        }
        let report = run(&json!({"a": {}, "b": []}), &limits(1, 3, 0));
        assert!(report.complete);
        assert_eq!(pointers(&report), ["", "/a", "/b"]);
    }

    #[test]
    fn awkward_keys_use_rfc6901_escaping() {
        // Keys are listed in byte order so insertion and sorted map order agree.
        let value = json!({
            "": SECRET,
            "/": 2,
            "a~/b": 3,
            "nested": {"": {"": SECRET}},
            "~": 1,
            "~1": 4,
        });
        let report = run(&value, &InspectLimits::default());
        assert!(report.complete);
        assert_eq!(
            pointers(&report),
            [
                "",
                "/",
                "/~1",
                "/a~0~1b",
                "/nested",
                "/nested/",
                "/nested//",
                "/~0",
                "/~01"
            ]
        );
        assert_eq!(report.paths[0].depth, 0);
        assert_eq!(report.paths[1].depth, 1);
        assert_eq!(report.paths[6].depth, 3);
    }

    #[test]
    fn depth_zero_shows_root_and_reports_depth_omission() {
        let report = run(&json!({"a": SECRET}), &limits(0, 100, 3));
        assert!(!report.complete);
        assert_eq!(report.omitted, only(|o| o.depth = true));
        assert_eq!(pointers(&report), [""]);
        assert_eq!(report.paths[0].truncated, Some(InspectTruncation::Depth));
        assert_eq!(report.paths[0].length, Some(1));
    }

    #[test]
    fn nested_depth_boundary() {
        let value = json!({"a": {"b": {"c": SECRET}}});
        let exact = run(&value, &limits(3, 100, 3));
        assert!(exact.complete);
        assert_eq!(pointers(&exact), ["", "/a", "/a/b", "/a/b/c"]);
        assert_eq!(exact.paths[3].depth, 3);

        let short = run(&value, &limits(2, 100, 3));
        assert!(!short.complete);
        assert_eq!(short.omitted, only(|o| o.depth = true));
        assert_eq!(pointers(&short), ["", "/a", "/a/b"]);
        assert_eq!(short.paths[2].truncated, Some(InspectTruncation::Depth));
    }

    #[test]
    fn sample_zero_and_large_arrays() {
        let report = run(&json!([SECRET, SECRET]), &limits(3, 100, 0));
        assert!(!report.complete);
        assert_eq!(report.omitted, only(|o| o.array_sample = true));
        assert_eq!(pointers(&report), [""]);
        assert_eq!(
            report.paths[0].truncated,
            Some(InspectTruncation::ArraySample)
        );

        let big = Value::Array((0..100_000).map(|i| json!({ "id": i })).collect());
        let report = run(&big, &limits(3, 100, 3));
        assert_eq!(
            pointers(&report),
            ["", "/0", "/0/id", "/1", "/1/id", "/2", "/2/id"]
        );
        assert_eq!(report.paths[0].length, Some(100_000));
        assert_eq!(report.omitted, only(|o| o.array_sample = true));

        let exact = run(&json!([1, 2, 3]), &limits(3, 100, 3));
        assert!(exact.complete);
        assert_eq!(exact.paths[0].truncated, None);
    }

    fn wide_object(width: usize, member: Value) -> Value {
        let mut map = Map::new();
        for i in (0..width).rev() {
            map.insert(format!("k{i:05}"), member.clone());
        }
        Value::Object(map)
    }

    #[test]
    fn wide_objects_hit_path_ceiling_in_map_order() {
        let value = wide_object(5000, json!(SECRET));
        let report = run(&value, &limits(3, INSPECT_MAX_PATHS, 3));
        assert_eq!(report.returned_paths, 1000);
        assert_eq!(report.omitted, only(|o| o.paths = true));
        assert_eq!(report.paths[0].length, Some(5000));
        // The returned members are the first ones the parsed map yields.
        let expected: Vec<String> = value
            .as_object()
            .unwrap()
            .keys()
            .take(999)
            .map(|k| format!("/{k}"))
            .collect();
        assert_eq!(pointers(&report)[1..], expected);
    }

    #[test]
    fn object_traversal_reads_only_the_returned_prefix() {
        let value = wide_object(100_000, json!({ "inner": SECRET }));

        // No allowance after the root: no member is read at all.
        let (report, entries) = walk(&value, &limits(3, 1, 3), 0).unwrap();
        assert_eq!(entries, 0);
        assert_eq!(pointers(&report), [""]);
        assert_eq!(report.paths[0].length, Some(100_000));
        assert_eq!(report.omitted, only(|o| o.paths = true));
        assert!(!report.complete);

        // Each read member is returned; nested objects are read lazily too.
        for max_paths in [2, 10, 500] {
            let (report, entries) = walk(&value, &limits(3, max_paths, 3), 0).unwrap();
            assert_eq!(report.returned_paths, max_paths);
            assert_eq!(entries, max_paths - 1);
            assert_eq!(report.omitted, only(|o| o.paths = true));
        }
    }

    #[test]
    fn exact_path_limit_is_not_an_omission() {
        let value = json!({"a": [1, {"b": null}], "c": SECRET});
        let all = run(&value, &InspectLimits::default());
        assert!(all.complete);
        let n = all.returned_paths;
        assert_eq!(n, 6);

        let exact = run(&value, &limits(3, n, 3));
        assert!(exact.complete);
        assert_eq!(exact.paths, all.paths);

        let short = run(&value, &limits(3, n - 1, 3));
        assert!(!short.complete);
        assert_eq!(short.omitted, only(|o| o.paths = true));
        assert_eq!(short.returned_paths, n - 1);
        assert_eq!(short.paths[..], all.paths[..n - 1]);

        // A container emitted as the last allowed path still has unseen children.
        assert!(run(&json!({"a": 1}), &limits(3, 1, 3)).omitted.paths);
        assert!(run(&json!([1]), &limits(3, 1, 3)).omitted.paths);
        // ...but an empty one does not.
        assert!(run(&json!({"a": {}}), &limits(3, 2, 3)).complete);
    }

    #[test]
    fn order_is_deterministic_and_follows_map_iteration() {
        let input = r#"{"b": {"y": 1, "x": 2}, "a": [3]}"#;
        let value: Value = serde_json::from_str(input).unwrap();
        let first = run(&value, &InspectLimits::default());
        let again: Value = serde_json::from_str(input).unwrap();
        assert_eq!(first, run(&again, &InspectLimits::default()));

        let mut expected = vec![String::new()];
        for (key, child) in value.as_object().unwrap() {
            expected.push(format!("/{key}"));
            match child {
                Value::Object(m) => expected.extend(m.keys().map(|k| format!("/{key}/{k}"))),
                Value::Array(a) => expected.extend((0..a.len()).map(|i| format!("/{key}/{i}"))),
                _ => {}
            }
        }
        assert_eq!(pointers(&first), expected);
    }

    #[test]
    fn invalid_limits_name_range_and_default() {
        let cases: Vec<(usize, Value, &str)> = vec![
            (
                0,
                json!(-1),
                "max_depth must be a whole number from 0 to 32 (default 3); got -1",
            ),
            (0, json!(33), "got 33"),
            (0, json!(1.5), "got a non-integer number"),
            (0, json!("3"), "got a string"),
            (
                0,
                Value::Null,
                "max_depth must be a whole number from 0 to 32 (default 3); got null",
            ),
            (
                1,
                json!(0),
                "max_paths must be a whole number from 1 to 1000 (default 100); got 0",
            ),
            (1, json!(1001), "got 1001"),
            (1, json!([1]), "got a array"),
            (1, Value::Null, "(default 100); got null"),
            (
                2,
                json!(-5),
                "array_sample must be a whole number from 0 to 100 (default 3); got -5",
            ),
            (2, json!(101), "got 101"),
            (2, Value::Null, "(default 3); got null"),
            (
                3,
                json!(511),
                "max_bytes must be a whole number from 512 to 65536 (default 16384); got 511",
            ),
            (3, json!(65537), "got 65537"),
            (3, json!(true), "got a boolean"),
            (3, json!(u64::MAX), "got 18446744073709551615"),
            (3, Value::Null, "(default 16384); got null"),
        ];
        for (slot, value, expected) in cases {
            let mut args = [None, None, None, None];
            args[slot] = Some(&value);
            let err = InspectLimits::from_json_args(args[0], args[1], args[2], args[3])
                .unwrap_err()
                .to_string();
            assert!(err.contains(expected), "{err}");
            assert!(err.contains("Pass a value in that range or omit"), "{err}");
        }
    }

    #[test]
    fn unicode_escaped_and_long_keys() {
        // Keys are listed in byte order so insertion and sorted map order agree.
        let value = json!({
            "ctl\n\u{1}\t": 4,
            "quote\"back\\slash": 3,
            "é": 1,
            "日本": {"🙂": 2},
        });
        let report = run(&value, &InspectLimits::default());
        assert!(report.complete);
        assert_eq!(
            pointers(&report),
            [
                "",
                "/ctl\n\u{1}\t",
                "/quote\"back\\slash",
                "/é",
                "/日本",
                "/日本/🙂"
            ]
        );

        let long = "k".repeat(70_000);
        let report = run(&json!({ "a": 1, long: SECRET }), &limits(3, 100, 3));
        assert_eq!(pointers(&report), ["", "/a"]);
        assert_eq!(report.omitted, only(|o| o.bytes = true));
    }

    fn sized(value: &Value, max_bytes: usize, framing: usize) -> (InspectReport, usize) {
        let l = InspectLimits::new(32, 1000, 100, max_bytes).unwrap();
        let report = inspect_value(value, &l, framing).unwrap();
        let size = framing + escaped_len(&serde_json::to_string(&report).unwrap());
        assert!(size <= max_bytes, "{size} > {max_bytes}");
        (report, size)
    }

    #[test]
    fn byte_budget_is_exact_at_the_boundary() {
        // Escape-heavy keys make the escaped size differ from the raw size.
        let value = json!({"\"q\"": [SECRET, {"\\": null}], "é/~": {"\n": 1}});
        for framing in [0, 57] {
            let need = (INSPECT_MIN_BYTES..=INSPECT_MAX_BYTES)
                .find(|&b| sized(&value, b, framing).0.complete)
                .unwrap();
            assert!(need > INSPECT_MIN_BYTES, "fixture too small to probe");
            // The complete preview fills the budget exactly: nothing was
            // reserved that a complete report does not use.
            let (_, size) = sized(&value, need, framing);
            assert_eq!(size, need);

            let (report, _) = sized(&value, need - 1, framing);
            assert!(!report.complete);
            assert_eq!(report.omitted, only(|o| o.bytes = true));
            assert_eq!(report.returned_paths, 6);
        }
    }

    #[test]
    fn byte_budget_bounds_large_previews() {
        let mut map = Map::new();
        for i in 0..2000 {
            map.insert(format!("\"key\\{i}\" é"), json!([SECRET, {"x": SECRET}]));
        }
        let value = Value::Object(map);
        for max_bytes in [512, 16384, 65536] {
            let l = InspectLimits::new(32, 1000, 100, max_bytes).unwrap();
            let report = inspect_value(&value, &l, 60).unwrap();
            let text = serde_json::to_string(&report).unwrap();
            assert!(60 + escaped_len(&text) <= max_bytes);
            assert!(report.omitted.bytes);
            assert!(report.returned_paths >= 1);
            assert!(!text.contains(SECRET));
        }
    }

    #[test]
    fn root_that_cannot_fit_is_a_bounded_error() {
        let l = InspectLimits::new(3, 100, 3, 512).unwrap();
        for framing in [600, usize::MAX] {
            let err = inspect_value(&json!({}), &l, framing)
                .unwrap_err()
                .to_string();
            assert!(err.contains("Increase max_bytes"), "{err}");
            assert!(err.len() < 200);
        }
    }

    #[test]
    fn escaped_len_matches_serde_json() {
        for s in [
            "plain",
            "é日🙂",
            "\"\\",
            "\n\r\t\u{8}\u{c}",
            "\u{0}\u{1f}\u{7f}",
        ] {
            let quoted = serde_json::to_string(s).unwrap();
            assert_eq!(escaped_len(s), quoted.len() - 2, "{s:?}");
        }
    }

    #[test]
    fn engine_parses_input_string() {
        let engine = JpxEngine::new();
        let report = engine
            .inspect(r#"{"a": [1, 2]}"#, &InspectLimits::default(), 0)
            .unwrap();
        assert_eq!(pointers(&report), ["", "/a", "/a/0", "/a/1"]);
    }
}
