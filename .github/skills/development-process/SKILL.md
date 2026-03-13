---
name: development-process
description: >-
  Guide the end-to-end development process for cqlsh-rs features: review plans,
  design tests, implement code, write tests, and update plan documents. Use when
  starting a new feature, picking up the next development task, or following the
  project's development workflow from plan to implementation.
---

# Development Process

Guide the structured development process for cqlsh-rs, from plan review through implementation, testing, and documentation updates.

## Workflow Overview

```
1. Review Plans  →  2. Design Tests  →  3. Implement  →  4. Test  →  5. Update Plans  →  6. Commit
```

Each feature follows this deterministic workflow. Never skip steps.

## Step 1: Review Plans

1. Read the master plan: `docs/plans/high-level-design.md`
2. Identify the next incomplete sub-plan (SP1–SP14) based on phase order
3. Read the target sub-plan fully (e.g., `docs/plans/01-cli-and-config.md`)
4. Read `docs/plans/10-testing-strategy.md` for testing requirements
5. Identify dependencies on other sub-plans

### Picking the Next Task

- Follow phase order: Phase 1 → Phase 2 → ... → Phase 6
- Within a phase, prioritize P1 items before P2, P2 before P3
- Check acceptance criteria of predecessor tasks — they must be met first
- If a sub-plan has status "IN PROGRESS", continue it; if "DONE", skip it

## Step 2: Design Tests

Before writing implementation code, design the test strategy:

1. **Unit tests** — Identify every public function and its edge cases
2. **Integration tests** — Identify interactions that need `assert_cmd` or `testcontainers`
3. **Snapshot tests** — Identify output formats that should be locked down
4. **Property tests** — Identify invariants (roundtrip, idempotency, commutativity)

### Test Design Checklist

- [ ] Happy path for each feature
- [ ] Edge cases (empty input, maximum values, Unicode, special characters)
- [ ] Error cases (invalid input, missing files, conflicts)
- [ ] Precedence/override cases (when multiple sources provide the same setting)
- [ ] Compatibility cases (behavior matches Python cqlsh)

## Step 3: Implement

### Module Structure

Follow the established module layout:

```
src/
├── main.rs              # Entry point, argument parsing, top-level orchestration
├── cli.rs               # CliArgs struct with clap derive
├── config.rs            # CqlshrcConfig, EnvConfig, MergedConfig
├── shell_completions.rs # Shell completion generation
├── driver/              # Database driver abstraction (future)
│   ├── mod.rs
│   └── scylla.rs
├── repl.rs              # REPL loop (future)
├── parser.rs            # Statement parser (future)
├── formatter.rs         # Output formatting (future)
├── colorizer.rs         # Syntax highlighting (future)
├── completer.rs         # Tab completion (future)
├── types.rs             # CQL type system (future)
└── commands/            # Built-in commands (future)
    ├── mod.rs
    ├── describe.rs
    └── ...
```

### Implementation Checklist

- [ ] Read existing code in the module area before making changes
- [ ] Follow the compatibility matrix in `high-level-design.md`
- [ ] Use `anyhow::Result` for fallible functions
- [ ] Use `thiserror` for domain-specific error types
- [ ] Keep structs and functions `pub` only when needed
- [ ] Add inline `#[cfg(test)]` module with unit tests
- [ ] Use `clap` derive API for any new CLI arguments
- [ ] Maintain case-sensitive INI parsing (`Ini::new_cs()`)

### Code Conventions

- **Error handling**: `anyhow` for application errors, `thiserror` for library errors
- **Async**: Tokio runtime (when adding async code)
- **Configuration precedence**: CLI > env > cqlshrc > defaults (always)
- **Boolean parsing**: `true/yes/on/1` → true, everything else → false (Python compat)
- **Missing config files**: Return defaults, never error

## Step 4: Test

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test cli::tests
cargo test config::tests

# Run integration tests only
cargo test --test cli_tests

# Run with output for debugging
cargo test -- --nocapture
```

### Test Quality Gates

- All tests must pass before committing
- No `#[ignore]` without a tracking issue
- Unit tests must cover happy path, edge cases, and error cases
- Integration tests must verify the binary end-to-end

### Coverage Targets (from SP10)

| Module | Target |
|--------|--------|
| `config.rs` | >95% |
| `parser.rs` | >95% |
| `types.rs` | >95% |
| `formatter.rs` | >90% |
| `completer.rs` | >90% |
| `commands/*` | >85% |
| `driver/*` | >80% |
| `repl.rs` | >70% |

## Step 5: Update Plans

After implementation, update the sub-plan document:

1. Mark completed steps with ✅
2. Update acceptance criteria checkboxes
3. Record key decisions in the "Key Decisions" table with rationale
4. Add test summary (count by layer)
5. Update status line at the top of the document
6. Remove speculative options that were not chosen (living document policy)

### Plan Update Template

```markdown
> **Status: IN PROGRESS** — [description] ([date])
```

or

```markdown
> **Status: DONE** — Completed [date], PR #XX
```

## Step 6: Commit

Use the `/conventional-commit` skill or follow this format:

```
type(scope): short description

Longer description of what was done and why.

- Key point 1
- Key point 2
```

### Commit Strategy

- **Separate commits** for code vs plan updates vs skill creation
- **Code commit**: `feat(scope):` or `fix(scope):` with implementation details
- **Plan commit**: `docs(plan):` with what was updated
- **Skill commit**: `feat(skills):` for new or updated skills
- Never mix code changes with documentation changes in one commit

## Common Patterns

### Adding a New CLI Flag

1. Add field to `CliArgs` in `src/cli.rs` with `#[arg(...)]` attribute
2. Add validation in `CliArgs::validate()` if needed
3. Add field to `MergedConfig` in `src/config.rs`
4. Update `MergedConfig::build()` with precedence logic
5. Add unit test in `cli::tests` for the flag
6. Add integration test in `tests/cli_tests.rs`
7. Update `default_cli()` helper in `config::tests`

### Adding a New cqlshrc Section

1. Create a new section struct (e.g., `NewSection`) in `src/config.rs`
2. Add field to `CqlshrcConfig`
3. Parse it in `CqlshrcConfig::from_ini()`
4. Wire relevant values into `MergedConfig::build()`
5. Add unit test for parsing the section
6. Add precedence test if the section values feed into `MergedConfig`

### Adding a New Environment Variable

1. Add field to `EnvConfig` in `src/config.rs`
2. Read it in `EnvConfig::from_env()`
3. Wire it into `MergedConfig::build()` at the env precedence level
4. Add integration test in `tests/cli_tests.rs` using `.env("VAR", "val")`

## Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `clap` | CLI argument parsing | v4 (derive) |
| `clap_complete` | Shell completion generation | v4 |
| `configparser` | INI file parsing | v3 |
| `anyhow` | Application error handling | v1 |
| `thiserror` | Custom error types | v2 |
| `dirs` | Home directory resolution | v6 |
| `assert_cmd` | CLI integration testing | v2 (dev) |
| `predicates` | Test assertions | v3 (dev) |
| `tempfile` | Temporary files in tests | v3 (dev) |
