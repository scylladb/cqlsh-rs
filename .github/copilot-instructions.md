# cqlsh-rs — Copilot Instructions

## Project Overview

cqlsh-rs is a ground-up Rust re-implementation of the Python `cqlsh` — the official interactive CQL shell for Apache Cassandra and compatible databases (ScyllaDB, Amazon Keyspaces, Astra DB). The project targets 100% command-line and configuration compatibility with the original Python cqlsh.

## Architecture

- **Language**: Rust (2021 edition)
- **Async runtime**: Tokio
- **Driver**: scylla-rust-driver
- **CLI**: clap v4 (derive API)
- **REPL**: rustyline
- **Formatting**: comfy-table, owo-colors
- **Testing**: cargo test, testcontainers-rs, assert_cmd, insta, proptest, criterion

## Key Conventions

- All design documents live in `docs/plans/`
- The master plan is `docs/plans/high-level-design.md` — read it before making architectural decisions
- Plans are living documents: update them when decisions are made
- 100% compatibility with Python cqlsh is the primary constraint
- Use Conventional Commits for all commit messages
- Every feature must reference the compatibility matrix in the high-level design

## Code Style

- Follow `cargo fmt` and `cargo clippy` defaults
- Use `thiserror` for library error types, `anyhow` for application-level errors
- Prefer `#[derive]` over manual trait implementations
- Use `#[cfg(test)] mod tests` for inline unit tests
- Async functions use Tokio runtime
- Target >90% code coverage

## Testing

- Unit tests: inline in modules with `#[cfg(test)]`
- Integration tests: `tests/` directory using testcontainers-rs
- CLI tests: `tests/` directory using assert_cmd
- Snapshot tests: use `insta` crate
- Property tests: use `proptest` crate
