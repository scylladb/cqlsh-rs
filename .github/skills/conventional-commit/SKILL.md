---
name: conventional-commit
description: >-
  Generate standardized commit messages following the Conventional Commits
  specification. Use when asked to commit changes, write a commit message,
  create a conventional commit, or when committing code. Analyzes staged
  changes and produces properly formatted commit messages.
---

# Conventional Commit

Generate commit messages following the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification for the cqlsh-rs project.

## Workflow

1. Run `git status` to review changed files
2. Run `git diff --cached` to inspect staged changes (or `git diff` for unstaged)
3. Analyze the changes to determine the commit type and scope
4. Construct the commit message following the format below
5. Stage files if needed with `git add <files>`
6. Execute the commit

## Commit Message Format

```
type(scope): description

[optional body]

[optional footer(s)]
```

### Types

| Type | When to Use |
|------|------------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `docs` | Documentation only changes |
| `style` | Formatting, whitespace (no code change) |
| `refactor` | Code restructuring (no feature/fix) |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `build` | Build system or dependencies |
| `ci` | CI/CD configuration |
| `chore` | Maintenance tasks |
| `revert` | Reverting a previous commit |

### Scopes for cqlsh-rs

Use these project-specific scopes:

| Scope | Area |
|-------|------|
| `cli` | Command-line argument parsing (clap) |
| `config` | cqlshrc configuration file handling |
| `driver` | Cassandra/Scylla driver layer |
| `repl` | REPL loop and line editing |
| `parser` | CQL statement parsing |
| `completion` | Tab completion |
| `format` | Output formatting and coloring |
| `describe` | DESCRIBE command family |
| `copy` | COPY TO/FROM operations |
| `types` | CQL type system |
| `auth` | Authentication and SSL/TLS |
| `plan` | Design documents and plans |
| `skills` | AI assistant skills |

### Rules

- Description must use imperative mood: "add", not "added" or "adds"
- Description must be lowercase, no period at the end
- Keep the first line under 72 characters
- Body explains **why**, not what (the diff shows what)
- Footer references issues: `Fixes #123`, `Refs #456`
- Breaking changes: add `!` after type/scope and `BREAKING CHANGE:` in footer

### Examples

```
feat(cli): add --request-timeout flag support

Implements the --request-timeout CLI flag matching Python cqlsh behavior.
Default value is 10 seconds, configurable via cqlshrc [connection] section.

Refs #42
```

```
fix(parser): handle semicolons inside string literals

The statement splitter was incorrectly treating semicolons within
quoted strings as statement terminators.
```

```
docs(plan): update phase 1 tasks with implementation decisions
```

```
test(driver): add integration tests for connection pooling
```

```
feat(copy)!: change default COPY FROM batch size

BREAKING CHANGE: default CHUNKSIZE changed from 1000 to 5000 to match
upstream Python cqlsh 6.2 behavior.
```
