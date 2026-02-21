//! Extension functions for JMESPath.
//!
//! This module contains 400+ additional functions organized by category.
//! Each submodule provides a `register_filtered` function that registers
//! only the enabled functions with the runtime.
//!
//! Extension functions are gated behind the `extensions` compile-time feature.

pub mod array;
pub mod color;
pub mod computing;
pub mod datetime;
pub mod discovery;
pub mod duration;
pub mod encoding;
pub mod expression;
pub mod format;
pub mod fuzzy;
pub mod geo;
pub mod hash;
pub mod ids;
pub mod jsonpatch;
pub mod language;
pub mod math;
pub mod multi_match;
pub mod network;
pub mod object;
pub mod path;
pub mod phonetic;
pub mod random;
pub mod regex_fns;
pub mod semver_fns;
pub mod string;
pub mod text;
pub mod type_conv;
pub mod units;
pub mod url_fns;
pub mod utility;
pub mod validation;
