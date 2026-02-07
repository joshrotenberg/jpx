//! Extension functions for JMESPath.
//!
//! This module contains 400+ additional functions organized by category.
//! Each submodule provides a `register_filtered` function that registers
//! only the enabled functions with the runtime.
//!
//! Extension functions are gated behind the `extensions` compile-time feature.

// Extension modules will be added here as they are ported in Phase 6.
// Each module exposes:
//   pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>)

// Stub modules for compilation - these will be replaced with real implementations.
// For now, each just provides an empty register_filtered function.

macro_rules! stub_module {
    ($name:ident) => {
        pub mod $name {
            use crate::Runtime;
            use std::collections::HashSet;

            pub fn register_filtered(_runtime: &mut Runtime, _enabled: &HashSet<&str>) {
                // Will be implemented in Phase 6
            }
        }
    };
}

stub_module!(string);
stub_module!(array);
stub_module!(object);
stub_module!(math);
stub_module!(type_conv);
stub_module!(utility);
stub_module!(validation);
stub_module!(path);
stub_module!(expression);
stub_module!(text);
stub_module!(hash);
stub_module!(encoding);
stub_module!(regex_fns);
stub_module!(url_fns);
stub_module!(random);
stub_module!(datetime);
stub_module!(fuzzy);
stub_module!(phonetic);
stub_module!(geo);
stub_module!(semver_fns);
stub_module!(network);
stub_module!(ids);
stub_module!(duration);
stub_module!(color);
stub_module!(computing);
stub_module!(multi_match);
stub_module!(jsonpatch);
stub_module!(format);
stub_module!(language);
stub_module!(discovery);
