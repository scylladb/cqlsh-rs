---
name: rust-error-handling
description: >-
  Apply idiomatic Rust error handling patterns using thiserror and anyhow.
  Use when asked to improve error handling, add error types, replace unwrap
  calls, design error hierarchies, or convert panics to proper error
  propagation. Follows the cqlsh-rs error handling conventions.
allowed-tools: Read, Edit, Write, Grep, Glob, Bash
---

# Rust Error Handling Guide

Apply idiomatic Rust error handling patterns for the cqlsh-rs project, using `thiserror` for library errors and `anyhow` for application-level errors.

## Core Principles

1. **Libraries expose typed errors** — Use `thiserror` for public error types
2. **Applications use dynamic errors** — Use `anyhow` at the binary/CLI level
3. **Never panic in library code** — Convert all `unwrap()`, `panic!()`, `todo!()` to `Result`
4. **Context propagates upward** — Each layer adds context via `.context()` or `.with_context()`
5. **Errors are for callers** — Error messages should help the caller understand what went wrong

## Error Type Design

### Module-level error enum (thiserror)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("failed to connect to {host}:{port}")]
    ConnectFailed {
        host: String,
        port: u16,
        #[source]
        source: std::io::Error,
    },

    #[error("authentication failed for user {user}")]
    AuthFailed {
        user: String,
        #[source]
        source: AuthError,
    },

    #[error("SSL/TLS handshake failed")]
    TlsFailed(#[from] native_tls::Error),

    #[error("connection timed out after {timeout:?}")]
    Timeout { timeout: std::time::Duration },
}
```

### Key `thiserror` attributes

| Attribute | Purpose |
|-----------|---------|
| `#[error("...")]` | Display implementation (supports format args) |
| `#[source]` | Marks the field as the error source for the chain |
| `#[from]` | Implements `From<T>` conversion + marks as source |
| `#[transparent]` | Delegates Display and source to the inner error |

## Error Handling Patterns

### Pattern 1: Propagate with `?` and add context

```rust
use anyhow::{Context, Result};

fn load_cqlshrc(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read cqlshrc from {}", path.display()))?;

    let config: Config = toml::from_str(&contents)
        .context("failed to parse cqlshrc")?;

    Ok(config)
}
```

### Pattern 2: Convert Option to Result

```rust
// Use ok_or for static errors
let host = config.get("host")
    .ok_or(ConnectionError::MissingHost)?;

// Use ok_or_else for dynamic errors
let port = config.get("port")
    .ok_or_else(|| anyhow!("missing port in config section [{}]", section))?;
```

### Pattern 3: Collect errors from iterators

```rust
// Fail on first error
let results: Vec<Row> = rows
    .into_iter()
    .map(|r| parse_row(r))
    .collect::<Result<Vec<_>>>()?;

// Collect all errors
let (successes, errors): (Vec<_>, Vec<_>) = rows
    .into_iter()
    .map(|r| parse_row(r))
    .partition_result();
```

### Pattern 4: Map external errors to domain errors

```rust
impl From<scylla::transport::errors::QueryError> for CqlError {
    fn from(err: scylla::transport::errors::QueryError) -> Self {
        match err {
            QueryError::TimeoutError => CqlError::Timeout,
            QueryError::DbError(db_err, msg) => CqlError::ServerError {
                code: db_err,
                message: msg,
            },
            other => CqlError::Driver(other.to_string()),
        }
    }
}
```

### Pattern 5: Error context at module boundaries

```rust
// In connection module — returns typed error
pub fn connect(config: &Config) -> Result<Session, ConnectionError> { ... }

// In main/CLI — wraps with anyhow context
let session = connect(&config)
    .context("failed to establish Cassandra connection")?;
```

## Anti-Patterns to Fix

### Replace `unwrap()` / `expect()` in library code

```rust
// BAD: panics on missing value
let port: u16 = config["port"].parse().unwrap();

// GOOD: propagates error with context
let port: u16 = config.get("port")
    .ok_or_else(|| anyhow!("missing 'port' in config"))?
    .parse()
    .context("invalid port number")?;
```

### Replace `panic!()` with error returns

```rust
// BAD
fn get_type(name: &str) -> CqlType {
    match name {
        "text" => CqlType::Text,
        "int" => CqlType::Int,
        _ => panic!("unknown type: {name}"),
    }
}

// GOOD
fn get_type(name: &str) -> Result<CqlType, TypeError> {
    match name {
        "text" => Ok(CqlType::Text),
        "int" => Ok(CqlType::Int),
        _ => Err(TypeError::Unknown(name.to_owned())),
    }
}
```

### Don't over-wrap errors

```rust
// BAD: redundant wrapping
Err(anyhow!("failed to connect: {}", ConnectionError::Timeout))

// GOOD: preserve the error chain
Err(ConnectionError::Timeout).context("failed to connect")
```

## Checklist

When reviewing or writing error handling:

- [ ] No `unwrap()` or `expect()` in library code (only in tests or proven-safe cases)
- [ ] All public functions return `Result` where failure is possible
- [ ] Error types use `thiserror` with descriptive `#[error()]` messages
- [ ] Error messages are lowercase and don't end with periods (Rust convention)
- [ ] Source errors are chained with `#[source]` or `#[from]`
- [ ] Context is added at module boundaries via `.context()`
- [ ] User-facing error messages are clear and actionable
- [ ] `anyhow` is only used in binary/CLI code, not library modules
