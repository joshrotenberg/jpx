# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.4.0...jpx-engine-v0.4.1) - 2026-08-13

### Fixed

- prepare the 0.5.1 point release ([#225](https://github.com/joshrotenberg/jpx/pull/225))

## [0.4.0](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.5...jpx-engine-v0.4.0) - 2026-06-11

### Added

- add batch introspection (describe_functions + batch_describe MCP tool) ([#213](https://github.com/joshrotenberg/jpx/pull/213))

### Fixed

- [**breaking**] config strict-merge now honors later layers in both directions ([#202](https://github.com/joshrotenberg/jpx/pull/202))

### Other

- release prep -- refresh counts, fix Python module, add library crate READMEs ([#215](https://github.com/joshrotenberg/jpx/pull/215))
- mark extensible public enums non_exhaustive; thiserror for ErrorReason ([#214](https://github.com/joshrotenberg/jpx/pull/214))
- refresh stale function counts, wire config color, drop dead code ([#206](https://github.com/joshrotenberg/jpx/pull/206))
- Prepare jpx Python package for first PyPI release ([#185](https://github.com/joshrotenberg/jpx/pull/185))

## [0.3.5](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.4...jpx-engine-v0.3.5) - 2026-03-16

### Other

- updated the following local packages: jpx-core

## [0.3.4](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.3...jpx-engine-v0.3.4) - 2026-02-28

### Other

- updated the following local packages: jpx-core

## [0.3.3](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.2...jpx-engine-v0.3.3) - 2026-02-23

### Other

- updated the following local packages: jpx-core

## [0.3.2](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.1...jpx-engine-v0.3.2) - 2026-02-11

### Fixed

- functions category filter case-sensitive mismatch ([#111](https://github.com/joshrotenberg/jpx/pull/111)) ([#121](https://github.com/joshrotenberg/jpx/pull/121))

## [0.3.1](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.3.0...jpx-engine-v0.3.1) - 2026-02-11

### Other

- Add AGENTS.md for AI coding agents ([#104](https://github.com/joshrotenberg/jpx/pull/104))

## [0.3.0](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.2.0...jpx-engine-v0.3.0) - 2026-02-08

### Added

- strict mode rejects let expressions + MCP engine_info ([#89](https://github.com/joshrotenberg/jpx/pull/89))
- add engine configuration (jpx.toml) and doc improvements ([#51](https://github.com/joshrotenberg/jpx/pull/51))
- add jpx-core and migrate all crates off jmespath/jmespath_extensions ([#45](https://github.com/joshrotenberg/jpx/pull/45))
- structured errors, explain tool, and improved error messages ([#42](https://github.com/joshrotenberg/jpx/pull/42))

### Fixed

- strict mode reporting in engine_info and validate (#92, #93) ([#94](https://github.com/joshrotenberg/jpx/pull/94))
- replace comma-counting arity detection with bracket-aware parser ([#77](https://github.com/joshrotenberg/jpx/pull/77))

### Other

- add missing metadata to workspace Cargo.toml files ([#75](https://github.com/joshrotenberg/jpx/pull/75))
- bump MSRV to 1.90 ([#68](https://github.com/joshrotenberg/jpx/pull/68))
- comprehensive jpx-engine test coverage (101 -> 264) ([#56](https://github.com/joshrotenberg/jpx/pull/56))
- remove CLI direct dependency on jpx-core ([#52](https://github.com/joshrotenberg/jpx/pull/52))
- split jpx-engine lib.rs into focused modules ([#40](https://github.com/joshrotenberg/jpx/pull/40))
- add documentation to discovery struct fields ([#24](https://github.com/joshrotenberg/jpx/pull/24))

## [0.2.0](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.1.3...jpx-engine-v0.2.0) - 2026-02-03

### Fixed

- remove broken rustdoc link to conditional arrow module ([#16](https://github.com/joshrotenberg/jpx/pull/16))

### Other

- [**breaking**] rename jpx-server to jpx-mcp ([#17](https://github.com/joshrotenberg/jpx/pull/17))

## [0.1.3](https://github.com/joshrotenberg/jpx/compare/jpx-engine-v0.1.2...jpx-engine-v0.1.3) - 2026-02-03

### Added

- initial jpx repository
