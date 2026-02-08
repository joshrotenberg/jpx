# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/joshrotenberg/jpx/compare/jpx-v0.3.0...jpx-v0.4.0) - 2026-02-08

### Added

- strict mode rejects let expressions + MCP engine_info ([#89](https://github.com/joshrotenberg/jpx/pull/89))
- suggest similar function names on evaluation failure ([#78](https://github.com/joshrotenberg/jpx/pull/78))
- add jpx-core and migrate all crates off jmespath/jmespath_extensions ([#45](https://github.com/joshrotenberg/jpx/pull/45))
- structured errors, explain tool, and improved error messages ([#42](https://github.com/joshrotenberg/jpx/pull/42))

### Fixed

- update stale jmespath-extensions references in READMEs ([#71](https://github.com/joshrotenberg/jpx/pull/71))
- support trailing positional arg as input file (jq convention) ([#64](https://github.com/joshrotenberg/jpx/pull/64))

### Other

- unify jpx and jpx-mcp versioning at 0.4.0 ([#76](https://github.com/joshrotenberg/jpx/pull/76))
- bump MSRV to 1.90 ([#68](https://github.com/joshrotenberg/jpx/pull/68))
- document trailing file argument as input source ([#65](https://github.com/joshrotenberg/jpx/pull/65))
- add assert_cmd integration test suite for jpx CLI (109 tests) ([#58](https://github.com/joshrotenberg/jpx/pull/58))
- remove CLI direct dependency on jpx-core ([#52](https://github.com/joshrotenberg/jpx/pull/52))
- split jpx CLI main.rs into focused modules ([#41](https://github.com/joshrotenberg/jpx/pull/41))

## [0.3.0](https://github.com/joshrotenberg/jpx/compare/jpx-v0.2.2...jpx-v0.3.0) - 2026-02-03

### Other

- [**breaking**] rename jpx-server to jpx-mcp ([#17](https://github.com/joshrotenberg/jpx/pull/17))

## [0.2.2](https://github.com/joshrotenberg/jpx/compare/jpx-v0.2.1...jpx-v0.2.2) - 2026-02-03

### Added

- initial jpx repository
