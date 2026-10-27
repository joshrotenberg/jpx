# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2](https://github.com/joshrotenberg/jpx/compare/jpx-v0.5.1...jpx-v0.5.2) - 2026-10-27

### Added

- embed offline query-library guide ([#284](https://github.com/joshrotenberg/jpx/pull/284))
- inspect bounded REPL history ([#283](https://github.com/joshrotenberg/jpx/pull/283))
- resolve prioritized CLI and MCP backlog ([#253](https://github.com/joshrotenberg/jpx/pull/253))

### Fixed

- *(cli)* exclude input clone and result drop from --bench timings ([#324](https://github.com/joshrotenberg/jpx/pull/324))
- infer nullable lists for Parquet export ([#310](https://github.com/joshrotenberg/jpx/pull/310))
- preserve null keys when reading Parquet ([#309](https://github.com/joshrotenberg/jpx/pull/309))
- parse JSON floats with correctly rounded binary64 conversion ([#271](https://github.com/joshrotenberg/jpx/pull/271))
- *(cli)* replace unmaintained YAML serialization dependency ([#294](https://github.com/joshrotenberg/jpx/pull/294))
- validate nanoid size before allocation ([#282](https://github.com/joshrotenberg/jpx/pull/282))
- preserve exact random integer bounds ([#279](https://github.com/joshrotenberg/jpx/pull/279))
- preserve exact numeric comparisons across query operations ([#268](https://github.com/joshrotenberg/jpx/pull/268))

### Other

- *(deps)* bump rustyline from 17.0.2 to 19.0.0 ([#318](https://github.com/joshrotenberg/jpx/pull/318))
- Restrict strict ordering operators to numeric operands ([#314](https://github.com/joshrotenberg/jpx/pull/314))
- *(deps)* bump tabled from 0.21.0 to 0.22.0 ([#303](https://github.com/joshrotenberg/jpx/pull/303))
- *(deps)* bump serde-saphyr from 1.3.0 to 2.0.0 ([#304](https://github.com/joshrotenberg/jpx/pull/304))
- *(deps)* bump dirs from 6.0.0 to 7.0.0 ([#305](https://github.com/joshrotenberg/jpx/pull/305))
- *(deps)* bump arrow and parquet to 60 ([#299](https://github.com/joshrotenberg/jpx/pull/299))
- add reproducible JMESPath compatibility audit probes ([#295](https://github.com/joshrotenberg/jpx/pull/295))
- *(cli)* test and explain bare literal compatibility ([#293](https://github.com/joshrotenberg/jpx/pull/293))
- make agent recipes correct and executable ([#269](https://github.com/joshrotenberg/jpx/pull/269))
- *(deps)* bump rand from 0.8.7 to 0.9.5 ([#230](https://github.com/joshrotenberg/jpx/pull/230))
- *(deps)* bump nanoid from 0.4.0 to 0.5.0 ([#252](https://github.com/joshrotenberg/jpx/pull/252))
- treat error messages as a contract, and enforce it ([#243](https://github.com/joshrotenberg/jpx/pull/243))
- *(deps)* bump toml from 0.9.12+spec-1.1.0 to 1.1.4+spec-1.1.0 ([#228](https://github.com/joshrotenberg/jpx/pull/228))

### Added

- add `--skills` to print an offline guide index and the `query-libraries` topic embedded in the binary, with `--json` output ([#224](https://github.com/joshrotenberg/jpx/issues/224))
- add `--per-file` corpus evaluation with repeated inputs and a `$file` path binding ([#220](https://github.com/joshrotenberg/jpx/issues/220))
- add explicit `--columns` selection and ordering for table, CSV, and TSV output ([#217](https://github.com/joshrotenberg/jpx/issues/217))
- add `--no-history` and explicit `-f/--file` startup data to the REPL ([#233](https://github.com/joshrotenberg/jpx/issues/233))
- add a `.history [count]` REPL command that lists recent entries with stable numbering ([#233](https://github.com/joshrotenberg/jpx/issues/233))

### Fixed

- make REPL exits run normal history teardown, honor color policy, derive demo help from the registry, and parse quoted load paths robustly ([#233](https://github.com/joshrotenberg/jpx/issues/233))

## [0.5.1](https://github.com/joshrotenberg/jpx/compare/jpx-v0.5.0...jpx-v0.5.1) - 2026-08-13

### Fixed

- prepare the 0.5.1 point release ([#225](https://github.com/joshrotenberg/jpx/pull/225))

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
