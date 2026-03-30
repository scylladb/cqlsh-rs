# Installation

## From source (cargo)

Requires [Rust](https://www.rust-lang.org/tools/install) 1.70+.

```bash
# Install from the repository
cargo install --git https://github.com/fruch/cqlsh-rs.git

# Or clone and install locally
git clone https://github.com/fruch/cqlsh-rs.git
cd cqlsh-rs
cargo install --path .
```

The binary is installed to `~/.cargo/bin/cqlsh-rs`.

## Pre-built binaries

Download pre-built binaries from [GitHub Releases](https://github.com/fruch/cqlsh-rs/releases):

| Platform | Architecture | Archive |
|----------|-------------|---------|
| Linux | x86_64 | `cqlsh-rs-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `cqlsh-rs-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | x86_64 | `cqlsh-rs-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `cqlsh-rs-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `cqlsh-rs-x86_64-pc-windows-msvc.zip` |

```bash
# Example: Linux x86_64
curl -LO https://github.com/fruch/cqlsh-rs/releases/latest/download/cqlsh-rs-x86_64-unknown-linux-gnu.tar.gz
tar xzf cqlsh-rs-x86_64-unknown-linux-gnu.tar.gz
sudo mv cqlsh-rs /usr/local/bin/
```

## Homebrew (macOS/Linux)

```bash
brew install fruch/tap/cqlsh-rs
```

## Docker

```bash
# Run interactively
docker run --rm -it ghcr.io/fruch/cqlsh-rs:latest

# Connect to a specific host
docker run --rm -it ghcr.io/fruch/cqlsh-rs:latest 10.0.0.1

# Execute a statement
docker run --rm ghcr.io/fruch/cqlsh-rs:latest -e "SELECT * FROM system.local" 10.0.0.1
```

## Building from source

```bash
git clone https://github.com/fruch/cqlsh-rs.git
cd cqlsh-rs
cargo build --release
```

The binary is at `target/release/cqlsh-rs`.

## Shell completions

Generate shell completion scripts for your shell:

```bash
# Bash
cqlsh-rs --completions bash > /etc/bash_completion.d/cqlsh-rs

# Zsh
cqlsh-rs --completions zsh > ~/.zfunc/_cqlsh-rs

# Fish
cqlsh-rs --completions fish > ~/.config/fish/completions/cqlsh-rs.fish

# PowerShell
cqlsh-rs --completions powershell > cqlsh-rs.ps1

# Elvish
cqlsh-rs --completions elvish > cqlsh-rs.elv
```

## Verifying the installation

```bash
cqlsh-rs --version
```
