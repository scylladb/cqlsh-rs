# cqlsh-rs — Claude Code Configuration

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
- Every feature must reference the compatibility matrix in the high-level design

## Progress Tracking

- **Always update `docs/progress.json`** when completing tasks or phases
- Cross-check task counts against `docs/plans/high-level-design.md` (source of truth)
- The `/development-process` skill Step 5b has detailed instructions for updating progress
- A GitHub Action (`.github/workflows/progress.yml`) auto-generates the SVG roadmap when `progress.json` changes on main

## Commit Messages

Use Conventional Commits format. See `/conventional-commit` skill for details.

## Skills

Skills are stored in `.github/skills/` and symlinked to `.claude/skills/` for dual Claude + Copilot compatibility. Available skills:

- `/skill-creator` — Guidelines for creating new skills
- `/conventional-commit` — Standardized commit messages
- `/create-implementation-plan` — Create structured implementation plans
- `/remember` — Persist lessons learned across sessions
- `/rust-testing` — Generate idiomatic Rust tests
- `/rust-clippy` — Run Clippy with strict lints and fix warnings
- `/rust-error-handling` — Idiomatic error handling with thiserror/anyhow
- `/rust-code-review` — Comprehensive multi-dimensional code review
- `/rust-performance` — Performance optimization, profiling, and benchmarking
- `/development-process` — End-to-end development workflow from plan to implementation
- `/ci-failure-analysis` — AI-powered CI failure diagnosis and PR comment workflow
- `/github-actions` — GitHub Actions workflow authoring, caching, matrix builds, and debugging

## Testing

- Unit tests: inline in modules with `#[cfg(test)]`
- Integration tests: `tests/` directory using testcontainers-rs
- CLI tests: `tests/` directory using assert_cmd
- Target: >90% code coverage
