# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
