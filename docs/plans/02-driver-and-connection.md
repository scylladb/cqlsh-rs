# Sub-Plan SP2: Driver & Connection

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1-2
> **Status: IN PROGRESS** — Core implementation complete (2026-03-14)

## Objective

Design a driver abstraction layer and implement Cassandra/ScyllaDB connectivity with authentication, SSL/TLS, connection pooling, and protocol version negotiation.

---

## Research Phase

### Tasks

1. **Evaluate `scylla` crate** — API surface, CQL type mapping, paging, prepared statements, connection pooling
2. **CQL binary protocol spec** — v4 and v5 differences, startup options, error codes
3. **Authentication mechanisms** — PasswordAuthenticator
4. **SSL/TLS configuration** — Certificate formats, mutual TLS, protocol versions
5. **Connection lifecycle** — Reconnection, failover, topology awareness

### Research Deliverables

- [x] Driver trait API design (`CqlDriver` trait in `driver/mod.rs`)
- [x] CQL type mapping table (scylla Rust types <-> CQL types, implemented in `driver/scylla_driver.rs`)
- [x] SSL/TLS configuration matrix (Python cqlsh options -> rustls config)
- [ ] Error code catalog

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests | Status |
|------|-------------|--------|-------|--------|
| 1 | Define `CqlDriver` trait (connect, execute, prepare, metadata) | `driver/mod.rs` | 3 unit tests (consistency) | ✅ Done |
| 2 | Define result types (`CqlResult`, `CqlRow`, `CqlValue`) | `driver/types.rs` | 14 unit tests | ✅ Done |
| 3 | Implement `ScyllaDriver` — basic connect | `driver/scylla_driver.rs` | Unit: type conversion | ✅ Done |
| 4 | Plain text authentication | `driver/scylla_driver.rs` | Wired in connect() | ✅ Done |
| 5 | Execute raw CQL string | `driver/scylla_driver.rs` | execute_unpaged() | ✅ Done |
| 6 | Prepared statement support | `driver/scylla_driver.rs` | prepare() + execute_prepared() | ✅ Done |
| 7 | Result set iteration with paging | `driver/scylla_driver.rs` | execute_paged() | ✅ Done |
| 8 | CQL type mapping (all types) | `driver/scylla_driver.rs` | 11 unit tests | ✅ Done |
| 9 | SSL/TLS with rustls | `driver/scylla_driver.rs` | build_rustls_config() | ✅ Done |
| 10 | Connection timeout handling | `driver/scylla_driver.rs` | Wired in connect() | ✅ Done |
| 11 | Protocol version negotiation | `driver/scylla_driver.rs` | Accepted via config, auto-negotiation by scylla crate | ✅ Done |
| 12 | Schema metadata queries | `driver/scylla_driver.rs` | get_keyspaces(), get_tables(), get_table_metadata() | ✅ Done |
| 13 | Keyspace switching | `session.rs` | 8 unit tests (parse_use_command) | ✅ Done |
| 14 | Consistency level management | `session.rs` | set_consistency_str(), set_serial_consistency_str() | ✅ Done |
| 15 | Tracing session management | `session.rs` | get_trace_session() | ✅ Done |

### Test Summary

| Layer | Count | Location |
|-------|-------|----------|
| Unit tests (driver trait) | 3 | `src/driver/mod.rs` |
| Unit tests (types) | 14 | `src/driver/types.rs` |
| Unit tests (scylla driver) | 16 | `src/driver/scylla_driver.rs` |
| Unit tests (session) | 8 | `src/session.rs` |
| **Total** | **41** | |

### Acceptance Criteria

- [x] Authenticate with username/password
- [x] SSL/TLS connections work with all cert options from cqlshrc (CA cert, mutual TLS, per-host certs)
- [x] All CQL types can be queried and returned (full convert_scylla_value mapping)
- [x] Paging works for large result sets (execute_paged with streaming)
- [x] Connection timeouts match `--connect-timeout` configuration
- [x] Request timeouts match `--request-timeout` configuration
- [ ] TODO: Integration tests against containerized Cassandra (requires testcontainers-rs setup)

---

## Skills Required

- Async Rust / Tokio (S2) ✅
- `scylla` crate API (C1) ✅
- CQL protocol (S3) ✅
- SSL/TLS with `rustls` (S8) ✅

---

## Key Decisions (Resolved)

| Decision | Chosen | Rationale |
|----------|--------|-----------|
| Primary driver | `scylla` crate | Better maintained by ScyllaDB team, `cdrs-tokio` rejected |
| Driver trait necessity | Trait (`CqlDriver`) | Testability and future flexibility |
| TLS implementation | `rustls` | Pure Rust, no system deps, matches project goal of zero runtime deps |
| Connection pooling | scylla built-in | No need for custom pooling |
| Module structure | `driver/mod.rs`, `driver/types.rs`, `driver/scylla_driver.rs` | Clean separation: trait + types + implementation |
| Session layer | Separate `session.rs` wrapping driver | Higher-level state (keyspace, consistency, tracing) managed here |
