//! Extension functions for JMESPath.
//!
//! This module contains 470+ additional functions organized by category.
//! Each submodule provides a `register_filtered` function that registers
//! only the enabled functions with the runtime.
//!
//! Each module is gated behind an `ext-*` compile-time feature (see the
//! crate README for the list). The `extensions` feature enables all of them.

#[cfg(feature = "ext-core")]
pub mod array;
#[cfg(feature = "ext-core")]
pub mod color;
#[cfg(feature = "ext-core")]
pub mod computing;
#[cfg(feature = "ext-time")]
pub mod datetime;
#[cfg(feature = "ext-text")]
pub mod discovery;
#[cfg(feature = "ext-core")]
pub mod duration;
#[cfg(feature = "ext-encoding")]
pub mod encoding;
#[cfg(feature = "ext-core")]
pub mod expression;
#[cfg(feature = "ext-format")]
pub mod format;
#[cfg(feature = "ext-text")]
pub mod fuzzy;
#[cfg(feature = "ext-geo")]
pub mod geo;
#[cfg(feature = "ext-hash")]
pub mod hash;
#[cfg(feature = "ext-ids")]
pub mod ids;
#[cfg(feature = "ext-jsonpatch")]
pub mod jsonpatch;
#[cfg(feature = "ext-lang")]
pub mod language;
#[cfg(feature = "ext-core")]
pub mod math;
#[cfg(feature = "ext-text")]
pub mod multi_match;
#[cfg(feature = "ext-net")]
pub mod network;
#[cfg(feature = "ext-core")]
pub mod object;
#[cfg(feature = "ext-core")]
pub mod path;
#[cfg(feature = "ext-text")]
pub mod phonetic;
#[cfg(feature = "ext-ids")]
pub mod random;
#[cfg(feature = "ext-regex")]
pub mod regex_fns;
#[cfg(feature = "ext-semver")]
pub mod semver_fns;
#[cfg(feature = "ext-strings")]
pub mod string;
#[cfg(feature = "ext-text")]
pub mod text;
#[cfg(feature = "ext-core")]
pub mod type_conv;
#[cfg(feature = "ext-core")]
pub mod units;
#[cfg(feature = "ext-encoding")]
pub mod url_fns;
#[cfg(feature = "ext-core")]
pub mod utility;
#[cfg(feature = "ext-validation")]
pub mod validation;
