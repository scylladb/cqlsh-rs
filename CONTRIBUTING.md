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

### Building for distribution (glibc compatibility)

Release binaries for Linux are **statically linked via musl**. This eliminates
glibc version requirements entirely and ensures compatibility with:

- RHEL 9 / Rocky Linux 9 / UBI9 (glibc 2.34)
- Ubuntu 22.04+ (glibc 2.35)
- ScyllaDB Docker images (ubi9-minimal)
- Any Linux distribution regardless of glibc version

**If you're packaging cqlsh-rs for a distribution or embedding it in a Docker
image, always use the musl target:**

```bash
# x86_64 static binary
make release-linux-x86_64
# Binary at: target/x86_64-unknown-linux-musl/release/cqlsh-rs

# aarch64 static binary (requires cross-rs)
make release-linux-aarch64
# Binary at: target/aarch64-unknown-linux-musl/release/cqlsh-rs
```

Or use pre-built binaries from
[GitHub Releases](https://github.com/scylladb/cqlsh-rs/releases/latest) which
are already statically linked.

> **⚠️ Do not use `cargo build --release` on modern toolchains (Fedora 43+,
> glibc 2.39) for distribution.** The resulting binary will require the build
> host's glibc version and fail on older systems. Always use the musl target
> or download pre-built static binaries.

### Docker images

Multi-architecture Docker images (`linux/amd64` and `linux/arm64`) are
automatically built and pushed to GitHub Container Registry:

```bash
docker pull ghcr.io/scylladb/cqlsh-rs:latest
docker pull ghcr.io/scylladb/cqlsh-rs:1.0.0  # specific version
```

#### Testing the image

The image is covered by black-box tests that drive it through a real PTY
(`tests/docker/`, run by the `Docker Image Tests` workflow). They catch
image-level problems the Rust suite cannot see, such as a missing runtime
dependency or BusyBox standing in for a GNU tool.

These are pytest + `pexpect` rather than Rust, deliberately: nothing in them
touches cqlsh-rs internals — they only run `docker run` and assert on what a
user sees, so the language buys nothing in reuse. What they need is a
well-worn pty driver, and `pexpect` is that. The Rust options (`portable-pty`,
`expectrl`) mean hand-rolling expect-with-alternatives and the transcript
capture that makes a failure readable, for a suite that must not be compiled
into the binary's test graph anyway. The repo already runs Python for the
comparison benchmarks (`benchmarks/python_cqlsh/`), so this adds no new
toolchain — and `uv` keeps the setup to a single command.

To run them against a locally built image:

```bash
# 1. Build the image the way the release pipeline does
cargo build --release --target x86_64-unknown-linux-musl
mkdir -p docker-build
cp target/x86_64-unknown-linux-musl/release/cqlsh-rs docker-build/cqlsh-rs-amd64
docker build --build-arg TARGETARCH=amd64 -t cqlsh-rs:ci .

# 2. Start a database on a user-defined network
docker network create cqlsh-ci-net
docker run -d --name cqlsh-ci-db --network cqlsh-ci-net \
  scylladb/scylla:2026.1 --smp 1 --memory 512M --overprovisioned 1

# 3. Run the tests — uv resolves pytest/pexpect from tests/docker/pyproject.toml
cd tests/docker && uv run pytest
```

Set `CQLSH_DOCKER_IMAGE` to test an already published tag instead, e.g.
`CQLSH_DOCKER_IMAGE=ghcr.io/scylladb/cqlsh-rs:latest`. The workflow accepts the
same via its `image` input on `workflow_dispatch`.

The suite runs automatically on PRs touching `Dockerfile`, `src/**`,
`Cargo.toml` or `tests/docker/**`, and on every push to `main`. Add the
`skip-docker-image-tests` label to a PR to skip the musl build when the change
cannot affect the packaged image.

## License

[MIT](LICENSE)
