# Installation

## From source (cargo)

Requires [Rust](https://www.rust-lang.org/tools/install) 1.70+.

```bash
# Install from the repository
cargo install --git https://github.com/scylladb/cqlsh-rs.git

# Or clone and install locally
git clone https://github.com/scylladb/cqlsh-rs.git
cd cqlsh-rs
cargo install --path .
```

The binary is installed to `~/.cargo/bin/cqlsh-rs`.

## Pre-built binaries

Download pre-built binaries from [GitHub Releases](https://github.com/scylladb/cqlsh-rs/releases):

Archive names include the version, e.g. `cqlsh-rs-0.5.13-x86_64-unknown-linux-musl.tar.gz`:

| Platform | Architecture | Archive |
|----------|-------------|---------|
| Linux | x86_64 | `cqlsh-rs-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux | aarch64 | `cqlsh-rs-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS | x86_64 | `cqlsh-rs-<version>-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `cqlsh-rs-<version>-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `cqlsh-rs-<version>-x86_64-pc-windows-msvc.zip` |

The Linux archives are statically linked against musl, so they run on any
distribution regardless of its glibc version.

```bash
# Example: Linux x86_64
VERSION=0.5.13
curl -LO "https://github.com/scylladb/cqlsh-rs/releases/download/v${VERSION}/cqlsh-rs-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
tar xzf "cqlsh-rs-${VERSION}-x86_64-unknown-linux-musl.tar.gz"
sudo install "cqlsh-rs-${VERSION}-x86_64-unknown-linux-musl/cqlsh-rs" /usr/local/bin/
```

Each release also ships a `SHA256SUMS.txt` for verification:

```bash
curl -LO "https://github.com/scylladb/cqlsh-rs/releases/download/v${VERSION}/SHA256SUMS.txt"
sha256sum --check --ignore-missing SHA256SUMS.txt
```

## Homebrew (macOS/Linux)

The `cqlsh-rs` repository doubles as its own Homebrew tap — there is no separate
`homebrew-cqlsh-rs` repository, so the tap URL must be passed explicitly:

```bash
brew tap scylladb/cqlsh-rs https://github.com/scylladb/cqlsh-rs
brew install cqlsh-rs

# later
brew update && brew upgrade cqlsh-rs
```

The formula ([`Formula/cqlsh-rs.rb`](https://github.com/scylladb/cqlsh-rs/blob/main/Formula/cqlsh-rs.rb))
downloads the pre-built release binary for your platform and also installs the
`cqlsh-rs(1)` man page and bash/zsh/fish completions.

To remove it:

```bash
brew uninstall cqlsh-rs
brew untap scylladb/cqlsh-rs
```

## Docker

```bash
# Run interactively
docker run --rm -it ghcr.io/scylladb/cqlsh-rs:latest

# Connect to a specific host
docker run --rm -it ghcr.io/scylladb/cqlsh-rs:latest 10.0.0.1

# Execute a statement
docker run --rm ghcr.io/scylladb/cqlsh-rs:latest -e "SELECT * FROM system.local" 10.0.0.1
```

## Building from source

```bash
git clone https://github.com/scylladb/cqlsh-rs.git
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
