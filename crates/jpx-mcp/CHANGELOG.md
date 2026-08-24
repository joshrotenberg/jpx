# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.2](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.5.1...jpx-mcp-v0.5.2) - 2026-08-24

### Added

- resolve prioritized CLI and MCP backlog ([#253](https://github.com/joshrotenberg/jpx/pull/253))

### Security

- disable `evaluate_file` over HTTP by default and add repeatable canonical
  allowed-root policies that reject canonical-path symlink escapes, validate
  opened regular files, and report the active policy through `engine_info`
  ([#234](https://github.com/joshrotenberg/jpx/issues/234))

## [0.5.1](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.5.0...jpx-mcp-v0.5.1) - 2026-08-13

### Fixed

- prepare the 0.5.1 point release ([#225](https://github.com/joshrotenberg/jpx/pull/225))

## [0.5.0](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.4.4...jpx-mcp-v0.5.0) - 2026-06-11

### Added

- add batch introspection (describe_functions + batch_describe MCP tool) ([#213](https://github.com/joshrotenberg/jpx/pull/213))

### Fixed

- bound MCP resource usage on untrusted input ([#204](https://github.com/joshrotenberg/jpx/pull/204))
- [**breaking**] config strict-merge now honors later layers in both directions ([#202](https://github.com/joshrotenberg/jpx/pull/202))

### Other

- release prep -- refresh counts, fix Python module, add library crate READMEs ([#215](https://github.com/joshrotenberg/jpx/pull/215))
- refresh stale function counts, wire config color, drop dead code ([#206](https://github.com/joshrotenberg/jpx/pull/206))

## [0.4.4](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.4.3...jpx-mcp-v0.4.4) - 2026-03-16

### Added

- upgrade tower-mcp to 0.8 and add title field to all MCP tools ([#173](https://github.com/joshrotenberg/jpx/pull/173))

## [0.4.2](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.4.1...jpx-mcp-v0.4.2) - 2026-02-11

### Other

- updated the following local packages: jpx-engine

## [0.4.1](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.4.0...jpx-mcp-v0.4.1) - 2026-02-11

### Other

- update Cargo.lock dependencies

## [0.4.0](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.1.4...jpx-mcp-v0.4.0) - 2026-02-08

### Added

- add suggest_function MCP tool ([#91](https://github.com/joshrotenberg/jpx/pull/91))
- strict mode rejects let expressions + MCP engine_info ([#89](https://github.com/joshrotenberg/jpx/pull/89))
- wire EngineConfig into jpx-mcp server ([#54](https://github.com/joshrotenberg/jpx/pull/54))
- structured errors, explain tool, and improved error messages ([#42](https://github.com/joshrotenberg/jpx/pull/42))

### Fixed

- strict mode reporting in engine_info and validate (#92, #93) ([#94](https://github.com/joshrotenberg/jpx/pull/94))

### Other

- consolidate MCP discovery/registration tools (32 -> 29) ([#79](https://github.com/joshrotenberg/jpx/pull/79))
- unify jpx and jpx-mcp versioning at 0.4.0 ([#76](https://github.com/joshrotenberg/jpx/pull/76))
- add missing metadata to workspace Cargo.toml files ([#75](https://github.com/joshrotenberg/jpx/pull/75))
- upgrade tower-mcp 0.3 to 0.5, remove mock-mcp-server, expand test coverage ([#59](https://github.com/joshrotenberg/jpx/pull/59))
- bump MSRV to 1.90 ([#68](https://github.com/joshrotenberg/jpx/pull/68))

## [0.1.4](https://github.com/joshrotenberg/jpx/compare/jpx-mcp-v0.1.3...jpx-mcp-v0.1.4) - 2026-02-03

### Added

- improve jpx-mcp server with CLI, HTTP transport, and tests ([#19](https://github.com/joshrotenberg/jpx/pull/19))

## [0.1.3](https://github.com/joshrotenberg/jpx/compare/jpx-server-v0.1.2...jpx-server-v0.1.3) - 2026-02-03

### Added

- initial jpx repository
