# Release Process

This document describes the release workflow for the jpx project.

## Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Push to main  │────▶│   release-plz   │────▶│  Creates PR     │
└─────────────────┘     │   (scheduled)   │     │  with bumps     │
                        └─────────────────┘     └────────┬────────┘
                                                         │
                        ┌─────────────────┐              │ merge
                        │  Creates tags   │◀─────────────┘
                        │  jpx-v0.3.0     │
                        │  jpx-mcp-v0.1.4 │
                        └────────┬────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              │                  │                  │
              ▼                  ▼                  ▼
    ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
    │    Release      │ │   Docker MCP    │ │   Release-plz   │
    │   (jpx-v*)      │ │  (jpx-mcp-v*)   │ │   (publish)     │
    └────────┬────────┘ └────────┬────────┘ └────────┬────────┘
             │                   │                   │
             ▼                   ▼                   ▼
    ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
    │ • Binaries      │ │ • Docker image  │ │ • crates.io     │
    │ • GitHub Release│ │   jpx-mcp       │ │   publish       │
    │ • Docker jpx    │ └─────────────────┘ └─────────────────┘
    │ • Homebrew      │
    └─────────────────┘
```

## Components

### 1. release-plz

**Purpose**: Automates version bumps, changelogs, and tag creation.

**Workflow**: `.github/workflows/release-plz.yml`

**Triggers**:
- Push to `main` branch
- Creates a PR with version bumps when changes are detected
- After PR merge, creates git tags for each crate

**Configuration**: `release-plz.toml`
```toml
[workspace]
changelog_update = true
git_release_enable = true
```

### 2. cargo-dist (Release workflow)

**Purpose**: Builds binaries for multiple platforms, creates GitHub releases.

**Workflow**: `.github/workflows/release.yml`

**Triggers**:
- Tag push matching `jpx-v[0-9]+.[0-9]+.[0-9]+*`
- Manual dispatch with `tag` input

**Outputs**:
- Binary artifacts (macOS arm64/x86_64, Linux x86_64, Windows x86_64)
- GitHub Release with release notes
- Docker images for `ghcr.io/joshrotenberg/jpx`
- Homebrew formula update

**Configuration**: `dist.toml` (cargo-dist config in `Cargo.toml`)

### 3. Docker MCP workflow

**Purpose**: Builds Docker images for the MCP server.

**Workflow**: `.github/workflows/docker-mcp.yml`

**Triggers**:
- Tag push matching `jpx-mcp-v[0-9]+.[0-9]+.[0-9]+*`
- Manual dispatch with `tag` input

**Outputs**:
- Multi-arch Docker image: `ghcr.io/joshrotenberg/jpx-mcp`
- Tags: `latest`, `X.Y.Z`, `X.Y`, `X` (if major > 0)

### 4. Homebrew

**Purpose**: Updates the Homebrew formula for easy installation.

**Repository**: `joshrotenberg/homebrew-brew`

**Updated by**: Release workflow after successful binary builds

**Formula**: `Formula/jpx.rb`

## Crates

| Crate | Tag Pattern | Publishes To |
|-------|-------------|--------------|
| jpx | `jpx-v*` | crates.io, GitHub Release, Docker, Homebrew |
| jpx-engine | `jpx-engine-v*` | crates.io |
| jpx-mcp | `jpx-mcp-v*` | crates.io, Docker |

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace config, cargo-dist settings |
| `release-plz.toml` | release-plz configuration |
| `.github/workflows/release-plz.yml` | Version bump automation |
| `.github/workflows/release.yml` | Binary builds, Docker, Homebrew |
| `.github/workflows/docker-mcp.yml` | MCP server Docker builds |
| `docker/cli/Dockerfile` | jpx CLI Docker image (uses pre-built binary) |
| `docker/cli/Dockerfile.build` | jpx CLI Docker image (builds from source, arm64) |
| `docker/server/Dockerfile` | jpx-mcp Docker image |

## Manual Operations

### Re-run a failed release

```bash
# Delete the existing release if it exists
gh release delete jpx-v0.3.0 --yes

# Trigger the workflow manually
gh workflow run Release -f tag=jpx-v0.3.0
```

### Re-run Docker MCP build

```bash
gh workflow run "Docker MCP" -f tag=jpx-mcp-v0.1.4
```

### Check workflow status

```bash
# List recent runs
gh run list --workflow=Release --limit 5

# Watch a specific run
gh run watch <run-id>

# View failed job logs
gh run view <run-id> --log-failed
```

### Manually publish to crates.io

If release-plz fails to publish:

```bash
# Publish in dependency order
cargo publish -p jpx-engine
cargo publish -p jpx-mcp
cargo publish -p jpx
```

## Secrets Required

| Secret | Purpose | Where to set |
|--------|---------|--------------|
| `GITHUB_TOKEN` | Default token, used for releases | Automatic |
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io | Repository secrets |
| `COMMITTER_TOKEN` | Pushing to homebrew-brew repo | Repository secrets |

## Troubleshooting

### "permission_denied: write_package" on Docker push

**Cause**: The GitHub Container Registry package doesn't have Actions access for this repository.

**Fix**:
1. Go to https://github.com/users/YOUR_USERNAME/packages/container/PACKAGE_NAME/settings
2. Under "Manage Actions access", click "Add Repository"
3. Add the repository with **Admin** or **Write** role
4. Re-run the workflow

### "a release with the same tag name already exists"

**Cause**: Re-running a workflow that already created a release.

**Fix**:
```bash
gh release delete TAG_NAME --yes
gh workflow run Release -f tag=TAG_NAME
```

### release-plz PR not created

**Cause**: No version-bump-worthy changes detected, or workflow didn't run.

**Check**:
```bash
gh run list --workflow=Release-plz --limit 5
```

**Manual trigger**:
```bash
gh workflow run Release-plz
```

### Homebrew formula not updated

**Cause**: The `update-homebrew` job only runs for `jpx-v*` tags and requires `COMMITTER_TOKEN`.

**Check**:
1. Verify the secret exists in repository settings
2. Check the job logs for auth errors

**Manual fix**:
Update `joshrotenberg/homebrew-brew` Formula/jpx.rb manually.

### Docker build timeout (arm64)

**Cause**: arm64 builds compile from source via QEMU emulation, which is slow (~40 min).

**Options**:
1. Wait - it's just slow
2. Use a self-hosted arm64 runner
3. Cross-compile instead of emulating

## Release Checklist

Before releasing:

- [ ] All CI checks pass on main
- [ ] CHANGELOG entries are meaningful
- [ ] Breaking changes are marked with `!` in commit messages
- [ ] Version bumps follow semver

After release-plz PR is merged:

- [ ] Tags are created automatically
- [ ] Release workflow completes successfully
- [ ] Docker images are pushed to GHCR
- [ ] Homebrew formula is updated
- [ ] crates.io packages are published

Verify:

```bash
# Check crates.io
cargo search jpx

# Check Docker images
docker pull ghcr.io/joshrotenberg/jpx:latest
docker pull ghcr.io/joshrotenberg/jpx-mcp:latest

# Check Homebrew
brew update && brew info jpx
```
