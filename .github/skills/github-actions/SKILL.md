---
name: github-actions
description: >-
  Author and maintain GitHub Actions workflows for CI/CD pipelines. Use when
  creating new workflows, modifying ci.yml, adding workflow jobs, configuring
  matrix builds, setting up caching, managing secrets, writing workflow_run
  triggers, creating release pipelines, or debugging GitHub Actions issues.
  Covers CI, benchmarking, release, and documentation workflows for cqlsh-rs.
---

# GitHub Actions Workflow Authoring

Author, maintain, and debug GitHub Actions workflows for the cqlsh-rs project. All workflows live in `.github/workflows/`.

## Before Starting

1. Read the existing CI workflow: `.github/workflows/ci.yml`
2. Check related plans for workflow requirements:
   - `docs/plans/10-testing-strategy.md` — CI matrix design
   - `docs/plans/11-benchmarking.md` — Benchmark workflow (`bench.yml`)
   - `docs/plans/12-cross-platform-release.md` — Release workflow (`release.yml`)
   - `docs/plans/14-documentation.md` — Docs workflow (`docs.yml`)
   - `docs/plans/15-ai-ci-failure-summaries.md` — Failure analysis workflow

## Planned Workflows

| Workflow | File | Trigger | Status |
|----------|------|---------|--------|
| CI | `ci.yml` | push to main, PRs | Implemented |
| CI Failure Analysis | `ci-failure-analysis.yml` | workflow_run (CI failed) | Planned (SP15) |
| Benchmarks | `bench.yml` | push to main, PR label, weekly | Planned (SP11) |
| Release | `release.yml` | tag push (v*) | Planned (SP12) |
| Documentation | `docs.yml` | push to main (docs/**) | Planned (SP14) |

## Workflow Conventions

### Standard Job Structure

Every job should follow this pattern:

```yaml
jobs:
  job-name:
    name: Human-Readable Name
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy  # as needed
      - uses: Swatinem/rust-cache@v2
      - run: cargo <command>
```

### Caching

Always use `Swatinem/rust-cache@v2` for Cargo build caching. It caches:
- `~/.cargo/registry`
- `~/.cargo/git`
- `target/`

For custom cache keys (e.g., nextest binary):
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-on-failure: true
    key: nextest  # differentiates from other jobs
```

### Environment Variables

Set project-wide env vars at the workflow level:

```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-Dwarnings"
```

### Permissions

Follow the principle of least privilege. Common patterns:

```yaml
# Read-only (default for most CI jobs)
permissions:
  contents: read

# PR commenting (failure analysis, bot comments)
permissions:
  contents: read
  pull-requests: write
  actions: read

# Release publishing
permissions:
  contents: write  # create releases
  id-token: write  # OIDC
```

### Secrets Management

| Secret | Scope | Purpose |
|--------|-------|---------|
| `GITHUB_TOKEN` | Auto-provided | GitHub API access |
| `ANTHROPIC_API_KEY` | Repository | Claude API for failure analysis |
| `CARGO_REGISTRY_TOKEN` | Repository | crates.io publishing (future) |

Never hardcode secrets. Use `${{ secrets.SECRET_NAME }}` syntax.

## Trigger Patterns

### Standard CI (every push/PR)

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

### Post-CI Analysis (workflow_run)

```yaml
on:
  workflow_run:
    workflows: ["CI"]
    types: [completed]
```

Access the triggering run's data via `github.event.workflow_run.*`.

### Conditional Execution

```yaml
# Only on failure
if: github.event.workflow_run.conclusion == 'failure'

# Only on PRs (not push to main)
if: github.event.workflow_run.event == 'pull_request'

# Only on tag push
if: startsWith(github.ref, 'refs/tags/v')

# Only when specific files changed
on:
  push:
    paths:
      - "src/**"
      - "Cargo.toml"
      - "Cargo.lock"
```

### Matrix Builds

For cross-platform or multi-version testing:

```yaml
strategy:
  fail-fast: false  # don't cancel other jobs when one fails
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
    db: [cassandra:4.0, cassandra:5.0, scylladb/scylla:6.0]
    exclude:
      - os: windows-latest
        db: cassandra:4.0  # Docker not easily available
```

**Cost rule:** never add an unconditional macOS/Windows job to PR CI. In
`ci.yml` the `test`/`build` OS matrix is Linux-only on PRs; macOS/Windows are
added by the `ci-macos` / `ci-windows` labels, and always on push to `main`
and release-plz PRs (`release-plz-*`). Reuse that matrix (`*os-matrix` YAML
anchor) for new cross-OS jobs. See CONTRIBUTING.md "CI labels".

## Rust-Specific Patterns

### Installing cargo-nextest

```yaml
- uses: taiki-e/install-action@nextest
- run: cargo nextest run --all-targets --all-features
```

### JUnit XML Output (for test reporting)

```yaml
- run: cargo nextest run --message-format junit --output-file test-results.xml
- uses: actions/upload-artifact@v4
  if: always()
  with:
    name: test-results
    path: test-results.xml
```

### Release Build with Optimizations

```yaml
- run: cargo build --release
  env:
    CARGO_PROFILE_RELEASE_LTO: thin
    CARGO_PROFILE_RELEASE_STRIP: true
```

### Cross-Compilation

```yaml
- uses: cross-rs/cross-action@v1
  with:
    command: build
    target: aarch64-unknown-linux-musl
    args: --release
```

## Artifact Patterns

### Upload/Download Within Same Workflow

```yaml
# Upload
- uses: actions/upload-artifact@v4
  with:
    name: artifact-name
    path: path/to/file
    retention-days: 30

# Download (in same workflow)
- uses: actions/download-artifact@v4
  with:
    name: artifact-name
```

### Cross-Workflow Artifact Access

For `workflow_run` triggered workflows accessing artifacts from the triggering run:

```yaml
- uses: actions/download-artifact@v4
  with:
    name: test-results
    run-id: ${{ github.event.workflow_run.id }}
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

## PR Comments from Workflows

### Using actions/github-script

```yaml
- uses: actions/github-script@v7
  with:
    script: |
      // Find existing comment to update (deduplication)
      const comments = await github.rest.issues.listComments({
        owner: context.repo.owner,
        repo: context.repo.repo,
        issue_number: prNumber
      });
      const existing = comments.data.find(c =>
        c.user.type === 'Bot' && c.body.includes('<!-- marker -->')
      );
      const body = `<!-- marker -->\n## Title\n${content}`;
      if (existing) {
        await github.rest.issues.updateComment({
          owner: context.repo.owner,
          repo: context.repo.repo,
          comment_id: existing.id,
          body
        });
      } else {
        await github.rest.issues.createComment({
          owner: context.repo.owner,
          repo: context.repo.repo,
          issue_number: prNumber,
          body
        });
      }
```

### Collapsible Sections

```markdown
<details>
<summary>Section Title (click to expand)</summary>

Content here — leave a blank line after `</summary>`.

</details>
```

## Debugging Workflows

### Common Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| `Permission denied` on PR comment | Missing `pull-requests: write` | Add to `permissions:` block |
| `workflow_run` doesn't trigger | Workflow name mismatch | Ensure `workflows: ["CI"]` matches exactly |
| Artifacts not found cross-workflow | Wrong `run-id` | Use `${{ github.event.workflow_run.id }}` |
| Cache miss every run | Different keys per job | Use shared `Swatinem/rust-cache@v2` config |
| Slow CI (>15 min) | No caching, no parallelism | Add rust-cache, split into parallel jobs |

### Viewing Logs

```bash
# View failed job logs
gh run view <run-id> --log-failed

# List recent workflow runs
gh run list --workflow ci.yml --limit 5

# Re-run failed jobs
gh run rerun <run-id> --failed
```

## Validation Checklist

Before merging workflow changes:

- [ ] Workflow YAML passes lint: `actionlint .github/workflows/*.yml` (if installed)
- [ ] All jobs have `name:` for readable UI
- [ ] Permissions follow least privilege
- [ ] `Swatinem/rust-cache@v2` is used in all Rust jobs
- [ ] Artifacts have appropriate `retention-days`
- [ ] `if: always()` is used for artifact uploads and report steps
- [ ] Secrets are not logged (no `echo ${{ secrets.* }}`)
- [ ] `fail-fast: false` is set on matrix builds (unless fast-fail is intentional)
