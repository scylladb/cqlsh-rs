---
name: rust-testing
description: >-
  Generate comprehensive Rust tests for cqlsh-rs modules. Use when asked
  to write tests, add test coverage, generate unit tests, create integration
  tests, or improve test coverage for Rust code. Orchestrates test creation
  following project conventions, cargo test patterns, and the cqlsh-rs
  testing strategy.
---

# Rust Test Generator

Generate comprehensive, idiomatic Rust tests for the cqlsh-rs project following the testing strategy defined in `docs/plans/10-testing-strategy.md`.

## Before Writing Tests

1. Read the source file being tested
2. Read `docs/plans/10-testing-strategy.md` for project testing conventions
3. Check existing tests in the module and `tests/` directory for patterns
4. Identify the testing layer (unit, integration, or CLI)

## Testing Layers

### Layer 1: Unit Tests (inline in modules)

Place unit tests in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name_scenario() {
        // Arrange
        let input = ...;

        // Act
        let result = function_name(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

**Naming convention**: `test_{function}_{scenario}` or `test_{function}_{input}_{expected}`

### Layer 2: Integration Tests (`tests/` directory)

Place integration tests in `tests/` directory:

```rust
// tests/integration_driver.rs
use cqlsh_rs::driver::*;

#[tokio::test]
async fn test_connection_to_cassandra() {
    // Uses testcontainers-rs for real Cassandra instance
}
```

### Layer 3: CLI Tests (`tests/` directory)

Test the binary itself using `assert_cmd`:

```rust
// tests/cli_basic.rs
use assert_cmd::Command;

#[test]
fn test_version_flag() {
    Command::cargo_bin("cqlsh-rs")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}
```

## Coverage Scope

For each function or module, generate tests covering:

1. **Happy path** — Valid inputs producing expected outputs
2. **Edge cases** — Empty inputs, boundary values, maximum sizes
3. **Error conditions** — Invalid inputs, missing data, connection failures
4. **Compatibility** — Behavior matching Python cqlsh (reference the compatibility matrix)

## Test Patterns for cqlsh-rs

### Configuration Parsing Tests

```rust
#[test]
fn test_cqlshrc_missing_file_uses_defaults() {
    let config = CqlshConfig::load(Path::new("/nonexistent"));
    assert_eq!(config.connection.timeout, Duration::from_secs(10));
}

#[test]
fn test_cqlshrc_precedence_cli_over_config() {
    // CLI args override cqlshrc values
}
```

### CQL Parsing Tests

```rust
#[test]
fn test_parse_select_simple() {
    let stmt = parse("SELECT * FROM users;");
    assert!(matches!(stmt, Statement::Select { .. }));
}

#[test]
fn test_parse_semicolon_in_string_literal() {
    // Semicolons inside quotes are NOT statement terminators
    let stmt = parse("INSERT INTO t (a) VALUES ('hello;world');");
    assert_eq!(count_statements(&stmt), 1);
}
```

### Async/Driver Tests

```rust
#[tokio::test]
async fn test_query_returns_rows() {
    let session = test_session().await;
    let rows = session.query("SELECT key FROM system.local", &[]).await.unwrap();
    assert!(!rows.is_empty());
}
```

### Snapshot Tests (with `insta`)

```rust
#[test]
fn test_describe_keyspace_output() {
    let output = format_describe_keyspace(&keyspace);
    insta::assert_snapshot!(output);
}
```

### Property Tests (with `proptest`)

```rust
proptest! {
    #[test]
    fn test_roundtrip_cql_value(value: CqlValue) {
        let formatted = format_value(&value);
        let parsed = parse_value(&formatted);
        prop_assert_eq!(value, parsed);
    }
}
```

## Running Tests

```bash
# All unit tests
cargo test

# Specific module
cargo test --lib parser::tests

# Integration tests (requires Docker for testcontainers)
cargo test --test integration_driver

# CLI tests
cargo test --test cli_basic

# With output
cargo test -- --nocapture

# Coverage report
cargo tarpaulin --out html
```

## Validation Checklist

After generating tests, verify:

- [ ] All tests pass: `cargo test`
- [ ] No compiler warnings: `cargo test 2>&1 | grep -c warning` = 0
- [ ] Tests are deterministic (no flaky behavior)
- [ ] Async tests use `#[tokio::test]`
- [ ] Test names clearly describe the scenario
- [ ] Edge cases and error paths are covered
- [ ] Compatibility with Python cqlsh behavior is verified where applicable
