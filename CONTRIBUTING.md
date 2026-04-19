# Contributing to cqlsh-rs

Thank you for your interest in contributing! Below are guidelines for development
and the automated release process.

## Development

```bash
# Clone and build
git clone https://github.com/scylladb/cqlsh-rs.git
cd cqlsh-rs
cargo build

# Run tests
cargo test

# Run clippy + fmt
cargo clippy --all-targets --all-features
cargo fmt --all -- --check
```

Design documents and implementation plans live in [`docs/plans/`](docs/plans/).
The master plan is `docs/plans/high-level-design.md` — read it before making
architectural decisions.

### Commit messages

This project uses [Conventional Commits](https://www.conventionalcommits.org).
Commit messages must follow the format:

| Commit prefix                    | Version bump  |
|----------------------------------|---------------|
| `fix:`                           | Patch (0.0.x) |
| `feat:`                          | Minor (0.x.0) |
| `feat!:` / `BREAKING CHANGE:`   | Major (x.0.0) |

## Making a release

Releases happen automatically when commits are merged to `main`. The
[CI workflow](.github/workflows/ci.yml) uses
[release-plz](https://release-plz.dev/) to analyse
[Conventional Commits](https://www.conventionalcommits.org) and determine
whether a version bump is needed.

When a new version is detected the CI will:

1. Create/update a **release PR** with the version bump in `Cargo.toml` and
   `CHANGELOG.md` updates.
2. When a maintainer **merges the release PR**, the CI detects the version change
   and automatically:
   - Creates a **git tag** (e.g., `v1.0.0`).
   - Creates a **GitHub Release** with auto-generated release notes.
   - **Publishes to crates.io** using the `CARGO_REGISTRY_TOKEN` secret.
3. Downstream jobs then:
   - Build **cross-platform binaries** (Linux x86_64/arm64, macOS x86_64/arm64,
     Windows x86_64).
   - Generate **man pages** and **shell completions**.
   - Upload all artifacts + SHA256 checksums to the GitHub Release.
   - Build and push a **multi-arch Docker image** to
     `ghcr.io/scylladb/cqlsh-rs`.

No manual tagging or token management is required for normal releases.

### Manual release (fallback)

If you need to manually trigger a release (e.g., to rebuild binaries for an
existing tag), use the [Release (Manual)](.github/workflows/release.yml)
workflow via `workflow_dispatch` in the GitHub Actions UI. Provide the existing
tag name (e.g., `v1.0.0`).

### Publishing to crates.io with the scylladb organization

The `cqlsh-rs` crate is published to [crates.io](https://crates.io/) under the
scylladb organization. To set up or verify crate ownership:

**One-time setup steps (done once per crate):**

1. Ensure the `cqlsh-rs` crate exists on crates.io. If it doesn't exist yet,
   the first `cargo publish` from CI will create it.

2. Add the scylladb GitHub team as crate owners:
   ```bash
   # Add the scylladb GitHub organization team as owners
   cargo owner --add github:scylladb:crate-publishers cqlsh-rs
   ```
   Replace `crate-publishers` with your actual GitHub team name that should have
   publish access.

3. Verify ownership:
   ```bash
   cargo owner --list cqlsh-rs
   ```

**GitHub repository setup:**

1. In the repository, go to **Settings → Secrets and variables → Actions**.
2. Create a repository secret named **`CARGO_REGISTRY_TOKEN`** with a crates.io
   API token from a user who is a member of the owning team.
   - Generate a token at <https://crates.io/settings/tokens>.
   - The token needs the `publish-update` scope.

> **Note:** Unlike PyPI, crates.io does not yet support Trusted Publishing
> (OIDC). A `CARGO_REGISTRY_TOKEN` secret is required. Track the upstream
> feature request at <https://github.com/rust-lang/crates.io/issues/7091>.

### Docker images

Multi-architecture Docker images (`linux/amd64` and `linux/arm64`) are
automatically built and pushed to GitHub Container Registry:

```bash
docker pull ghcr.io/scylladb/cqlsh-rs:latest
docker pull ghcr.io/scylladb/cqlsh-rs:1.0.0  # specific version
```

## License

[MIT](LICENSE)
