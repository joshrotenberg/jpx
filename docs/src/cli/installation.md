# Installation

## Docker (Quickest)

Try jpx instantly without installing anything:

```bash
# Pull the image
docker pull ghcr.io/joshrotenberg/jpx

# Run a query
echo '{"name": "Alice", "age": 30}' | docker run -i ghcr.io/joshrotenberg/jpx 'name'
# "Alice"

# Fetch and transform data from an API
curl -s 'https://hacker-news.firebaseio.com/v0/item/1.json' | \
  docker run -i ghcr.io/joshrotenberg/jpx '{title: title, by: by, score: score}'
# {"by": "pg", "score": 57, "title": "Y Combinator"}
```

Available for `linux/amd64` and `linux/arm64`.

## Homebrew (macOS/Linux)

The easiest way to install jpx on macOS or Linux:

```bash
brew tap joshrotenberg/brew
brew install jpx
```

## Pre-built Binaries

Download pre-built binaries for your platform from the [GitHub Releases](https://github.com/joshrotenberg/jmespath-extensions/releases) page.

Available platforms:
- macOS (Apple Silicon / arm64)
- macOS (Intel / x86_64)
- Linux (x86_64)
- Windows (x86_64)

## Cargo (from crates.io)

If you have Rust installed:

```bash
cargo install jpx
```

## From Source

Clone and build from source:

```bash
git clone https://github.com/joshrotenberg/jmespath-extensions
cd jmespath-extensions/jpx
cargo install --path .
```

### Without MCP Server Support

MCP support is included by default. To build without it (smaller binary):

```bash
cargo install --path . --no-default-features
```

## Verify Installation

Check that jpx is installed correctly:

```bash
jpx --version
```

You should see output like:
```
jpx 0.1.15
```

## Shell Completions

Generate shell completions for your shell:

```bash
# Bash
jpx --completions bash > ~/.local/share/bash-completion/completions/jpx

# Zsh
jpx --completions zsh > ~/.zfunc/_jpx

# Fish
jpx --completions fish > ~/.config/fish/completions/jpx.fish

# PowerShell
jpx --completions powershell > jpx.ps1
```
