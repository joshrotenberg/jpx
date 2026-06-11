# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/joshrotenberg/jpx/compare/jpx-v0.4.4...jpx-v0.5.0) - 2026-06-11

### Added

- add --cheatsheet flag for a one-page quick reference ([#212](https://github.com/joshrotenberg/jpx/pull/212))
- register 37 implemented-but-unlisted extension functions ([#190](https://github.com/joshrotenberg/jpx/pull/190)) ([#201](https://github.com/joshrotenberg/jpx/pull/201))

### Fixed

- deduplicate flatten/mask name collisions ([#209](https://github.com/joshrotenberg/jpx/pull/209))
- replace unmaintained atty with std::io::IsTerminal ([#205](https://github.com/joshrotenberg/jpx/pull/205))
- [**breaking**] config strict-merge now honors later layers in both directions ([#202](https://github.com/joshrotenberg/jpx/pull/202))
- correct streaming pipeline error handling and CLI cleanups ([#200](https://github.com/joshrotenberg/jpx/pull/200))
- terminate cleanly on a broken pipe (SIGPIPE) ([#199](https://github.com/joshrotenberg/jpx/pull/199))
- eliminate UTF-8 byte-slice panics in CLI and extensions ([#198](https://github.com/joshrotenberg/jpx/pull/198))

### Other

- release prep -- refresh counts, fix Python module, add library crate READMEs ([#215](https://github.com/joshrotenberg/jpx/pull/215))
- refresh stale function counts, wire config color, drop dead code ([#206](https://github.com/joshrotenberg/jpx/pull/206))
- Prepare jpx Python package for first PyPI release ([#185](https://github.com/joshrotenberg/jpx/pull/185))

## [0.4.4](https://github.com/joshrotenberg/jpx/compare/jpx-v0.4.3...jpx-v0.4.4) - 2026-03-16

### Added

- support CSV/TSV output in streaming mode ([#176](https://github.com/joshrotenberg/jpx/pull/176))

## [0.4.3](https://github.com/joshrotenberg/jpx/compare/jpx-v0.4.2...jpx-v0.4.3) - 2026-02-28

### Added

- add --arg and --argjson flags for variable binding ([#170](https://github.com/joshrotenberg/jpx/pull/170))
- add --indent N and --tab flags for indentation control ([#169](https://github.com/joshrotenberg/jpx/pull/169))
- add --exit-status / -x flag for shell scripting ([#166](https://github.com/joshrotenberg/jpx/pull/166))
- add --join-output / -j flag for newline-free output ([#167](https://github.com/joshrotenberg/jpx/pull/167))
- add --raw-input / -R flag for non-JSON text processing ([#165](https://github.com/joshrotenberg/jpx/pull/165))

### Fixed

- use exit_code in --join-output return path ([#168](https://github.com/joshrotenberg/jpx/pull/168))

## [0.4.2](https://github.com/joshrotenberg/jpx/compare/jpx-v0.4.1...jpx-v0.4.2) - 2026-02-11

### Other

- updated the following local packages: jpx-engine

## [0.4.1](https://github.com/joshrotenberg/jpx/compare/jpx-v0.4.0...jpx-v0.4.1) - 2026-02-11

### Other

- Add AGENTS.md for AI coding agents ([#104](https://github.com/joshrotenberg/jpx/pull/104))

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
