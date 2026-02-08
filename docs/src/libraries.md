# Libraries

jpx is built on a stack of Rust crates that can be used independently in your own projects.

## jpx-core

Self-contained JMESPath implementation with 400+ extension functions. Provides the parser, runtime, and function registry used by all other jpx crates.

- [crates.io](https://crates.io/crates/jpx-core)
- [docs.rs](https://docs.rs/jpx-core)

## jpx-engine

Query engine with introspection, function discovery, configuration, and query store. Wraps jpx-core with higher-level features for building tools and services.

- [crates.io](https://crates.io/crates/jpx-engine)
- [docs.rs](https://docs.rs/jpx-engine)

## Python Bindings

Use jpx functions in Python via the `jmespath-extensions` package.

- [PyPI](https://pypi.org/project/jmespath-extensions/)
