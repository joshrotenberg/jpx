# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.3.1...jpx-core-v0.4.0) - 2026-10-20

### Added

- *(core)* split extensions feature into per-category features ([#296](https://github.com/joshrotenberg/jpx/pull/296))
- resolve prioritized CLI and MCP backlog ([#253](https://github.com/joshrotenberg/jpx/pull/253))

### Fixed

- handle extreme indices and avoid repeated let-path copies ([#312](https://github.com/joshrotenberg/jpx/pull/312))
- parse JSON floats with correctly rounded binary64 conversion ([#271](https://github.com/joshrotenberg/jpx/pull/271))
- validate nanoid size before allocation ([#282](https://github.com/joshrotenberg/jpx/pull/282))
- preserve exact random integer bounds ([#279](https://github.com/joshrotenberg/jpx/pull/279))
- report effective let-expression support ([#273](https://github.com/joshrotenberg/jpx/pull/273))
- preserve exact numeric comparisons across query operations ([#268](https://github.com/joshrotenberg/jpx/pull/268))

### Other

- Move owned flatten elements instead of cloning them ([#316](https://github.com/joshrotenberg/jpx/pull/316))
- Restrict strict ordering operators to numeric operands ([#314](https://github.com/joshrotenberg/jpx/pull/314))
- *(deps)* bump rand from 0.9.5 to 0.10.3 ([#307](https://github.com/joshrotenberg/jpx/pull/307))
- *(deps)* move sha1, sha2, and hmac to the digest 0.11 family ([#298](https://github.com/joshrotenberg/jpx/pull/298))
- *(core)* borrow arguments for built-in functions ([#301](https://github.com/joshrotenberg/jpx/pull/301))
- *(core)* add reproducible differential campaigns ([#286](https://github.com/joshrotenberg/jpx/pull/286))
- *(core)* preserve borrowed arguments for length ([#280](https://github.com/joshrotenberg/jpx/pull/280))
- *(deps)* bump rand from 0.8.7 to 0.9.5 ([#230](https://github.com/joshrotenberg/jpx/pull/230))
- *(deps)* bump md-5 from 0.10.6 to 0.11.0 ([#244](https://github.com/joshrotenberg/jpx/pull/244))
- exercise isolated minimal-core features in CI ([#281](https://github.com/joshrotenberg/jpx/pull/281))
- *(deps)* update dependencies and stop-word API ([#258](https://github.com/joshrotenberg/jpx/pull/258))
- *(deps)* bump nanoid from 0.4.0 to 0.5.0 ([#252](https://github.com/joshrotenberg/jpx/pull/252))
- interpret over borrowed values ([#242](https://github.com/joshrotenberg/jpx/pull/242))
- *(deps)* bump stop-words from 0.9.0 to 0.10.0 ([#246](https://github.com/joshrotenberg/jpx/pull/246))
- expose the lexer doc-hidden for cross-implementation benchmarking ([#247](https://github.com/joshrotenberg/jpx/pull/247))
- treat error messages as a contract, and enforce it ([#243](https://github.com/joshrotenberg/jpx/pull/243))
- re-vendor compliance suite from upstream at a pinned commit ([#241](https://github.com/joshrotenberg/jpx/pull/241))
- *(deps)* bump dateparser from 0.2.1 to 0.3.1 ([#227](https://github.com/joshrotenberg/jpx/pull/227))
- *(deps)* bump toml from 0.9.12+spec-1.1.0 to 1.1.4+spec-1.1.0 ([#228](https://github.com/joshrotenberg/jpx/pull/228))
- *(deps)* bump base64 from 0.22.1 to 0.23.1 ([#229](https://github.com/joshrotenberg/jpx/pull/229))
- convert jmespath input outside the timed closure ([#239](https://github.com/joshrotenberg/jpx/pull/239))

### Added

- per-category `ext-*` features so downstream crates can compile a subset of the extension functions; `extensions` enables all of them and the default features are unchanged
- `Category::feature_name` names the feature that compiles a category in

### Changed

- `FunctionRegistry::register_category` skips categories that are not compiled in, so the registry lists only functions the runtime can call
- all 26 built-in functions and 53 extension functions receive borrowed arguments from the interpreter, so a call such as `type(items)`, `first(items)`, or `max_by(items, &id)` no longer copies `items` first; `max`, `min`, `sort_by`, `max_by_expr`, `min_by_expr`, and `get_path` also stop copying elements internally. Functions registered through `Runtime::register_function` still receive owned arguments ([#236](https://github.com/joshrotenberg/jpx/issues/236))

### Fixed

- reject non-ASCII numeric characters after `-` instead of panicking in the lexer ([#248](https://github.com/joshrotenberg/jpx/issues/248))

### Other

- broaden parser property tests with reusable grammar-directed and compliance-derived near-valid expression generators ([#250](https://github.com/joshrotenberg/jpx/issues/250))

## [0.3.1](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.3.0...jpx-core-v0.3.1) - 2026-08-13

### Fixed

- prepare the 0.5.1 point release ([#225](https://github.com/joshrotenberg/jpx/pull/225))

## [0.3.0](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.2.2...jpx-core-v0.3.0) - 2026-06-11

### Added

- register 37 implemented-but-unlisted extension functions ([#190](https://github.com/joshrotenberg/jpx/pull/190)) ([#201](https://github.com/joshrotenberg/jpx/pull/201))

### Fixed

- resolve jpx-core name collisions and from_epoch_ms pre-epoch handling ([#189](https://github.com/joshrotenberg/jpx/pull/189)) ([#211](https://github.com/joshrotenberg/jpx/pull/211))
- report error column in characters and bound slice length cast ([#210](https://github.com/joshrotenberg/jpx/pull/210))
- stop datetime panics and bound allocation in extensions ([#208](https://github.com/joshrotenberg/jpx/pull/208))
- deduplicate flatten/mask name collisions ([#209](https://github.com/joshrotenberg/jpx/pull/209))
- prevent user data from colliding with expref sentinels ([#207](https://github.com/joshrotenberg/jpx/pull/207))
- eliminate UTF-8 byte-slice panics in CLI and extensions ([#198](https://github.com/joshrotenberg/jpx/pull/198))
- bound parser and interpreter recursion depth (stack-overflow DoS) ([#197](https://github.com/joshrotenberg/jpx/pull/197))

### Other

- release-readiness audit -- refresh stale counts, add crate READMEs, fix doc warnings ([#216](https://github.com/joshrotenberg/jpx/pull/216))
- release prep -- refresh counts, fix Python module, add library crate READMEs ([#215](https://github.com/joshrotenberg/jpx/pull/215))
- mark extensible public enums non_exhaustive; thiserror for ErrorReason ([#214](https://github.com/joshrotenberg/jpx/pull/214))
- refresh stale function counts, wire config color, drop dead code ([#206](https://github.com/joshrotenberg/jpx/pull/206))
- Prepare jpx Python package for first PyPI release ([#185](https://github.com/joshrotenberg/jpx/pull/185))

## [0.2.2](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.2.1...jpx-core-v0.2.2) - 2026-03-16

### Added

- unify group_by, index_by, partition_by to accept exprefs ([#175](https://github.com/joshrotenberg/jpx/pull/175))

## [0.2.1](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.2.0...jpx-core-v0.2.1) - 2026-02-28

### Fixed

- replace chrono-english with interim (RUSTSEC-2024-0395) ([#163](https://github.com/joshrotenberg/jpx/pull/163))

## [0.2.0](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.1.3...jpx-core-v0.2.0) - 2026-02-23

### Added

- add pivot/unpivot functions for data reshaping ([#158](https://github.com/joshrotenberg/jpx/pull/158))
- add rank, dense_rank, lag, lead window functions ([#159](https://github.com/joshrotenberg/jpx/pull/159))
- add skew, kurtosis, mad statistics functions ([#160](https://github.com/joshrotenberg/jpx/pull/160))
- add unit conversion functions (temperature, length, mass, volume) ([#139](https://github.com/joshrotenberg/jpx/pull/139))

### Fixed

- correct expref signatures and examples in functions.toml ([#133](https://github.com/joshrotenberg/jpx/pull/133))

### Other

- improve coverage for thin extension modules ([#156](https://github.com/joshrotenberg/jpx/pull/156))
- add stress tests for large/extreme inputs ([#154](https://github.com/joshrotenberg/jpx/pull/154))

## [0.1.3](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.1.2...jpx-core-v0.1.3) - 2026-02-11

### Added

- *(path)* add path_stem, path_is_absolute, path_is_relative functions ([#130](https://github.com/joshrotenberg/jpx/pull/130))
- *(rand)* add random_int and random_choice functions ([#117](https://github.com/joshrotenberg/jpx/pull/117)) ([#129](https://github.com/joshrotenberg/jpx/pull/129))
- *(encoding)* add base64url_encode, base64url_decode ([#118](https://github.com/joshrotenberg/jpx/pull/118)) ([#128](https://github.com/joshrotenberg/jpx/pull/128))
- *(duration)* add duration_days, duration_add, duration_subtract ([#115](https://github.com/joshrotenberg/jpx/pull/115)) ([#127](https://github.com/joshrotenberg/jpx/pull/127))
- *(url)* add url_build, query_string_parse, query_string_build functions ([#114](https://github.com/joshrotenberg/jpx/pull/114)) ([#125](https://github.com/joshrotenberg/jpx/pull/125))
- *(regex)* add regex_split and regex_count functions ([#113](https://github.com/joshrotenberg/jpx/pull/113)) ([#124](https://github.com/joshrotenberg/jpx/pull/124))
- *(geo)* add containment, bounding box, midpoint, and geohash functions ([#123](https://github.com/joshrotenberg/jpx/pull/123))

## [0.1.2](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.1.1...jpx-core-v0.1.2) - 2026-02-11

### Other

- Add AGENTS.md for AI coding agents ([#104](https://github.com/joshrotenberg/jpx/pull/104))

## [0.1.1](https://github.com/joshrotenberg/jpx/compare/jpx-core-v0.1.0...jpx-core-v0.1.1) - 2026-02-08

### Fixed

- resolve 13 ignored tests in jpx-core ([#74](https://github.com/joshrotenberg/jpx/pull/74))

### Other

- add missing metadata to workspace Cargo.toml files ([#75](https://github.com/joshrotenberg/jpx/pull/75))
- bump MSRV to 1.90 ([#68](https://github.com/joshrotenberg/jpx/pull/68))
- comprehensive jpx-core test coverage (63 -> 985 tests) ([#55](https://github.com/joshrotenberg/jpx/pull/55))
