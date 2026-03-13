# Sub-Plan SP2: Driver & Connection

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1-2

## Objective

Design a driver abstraction layer and implement Cassandra/ScyllaDB connectivity with authentication, SSL/TLS, connection pooling, and protocol version negotiation.

---

## Research Phase

### Tasks

1. **Evaluate `scylla` crate** — API surface, CQL type mapping, paging, prepared statements, connection pooling
2. **Evaluate `cdrs-tokio`** — Compare feature set, maintenance status, community
3. **CQL binary protocol spec** — v4 and v5 differences, startup options, error codes
4. **Authentication mechanisms** — PasswordAuthenticator, LDAP, Kerberos (if supported)
5. **SSL/TLS configuration** — Certificate formats, mutual TLS, protocol versions
6. **Connection lifecycle** — Reconnection, failover, topology awareness

### Research Deliverables

- [ ] Driver trait API design document
- [ ] scylla crate capability matrix
- [ ] CQL type mapping table (scylla Rust types <-> CQL types)
- [ ] SSL/TLS configuration matrix (Python cqlsh options -> rustls config)
- [ ] Error code catalog

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | Define `CqlDriver` trait (connect, execute, prepare, metadata) | `driver/mod.rs` | Compile-time: trait bounds |
| 2 | Define result types (`CqlResult`, `CqlRow`, `CqlValue`) | `driver/mod.rs` | Unit: type construction |
| 3 | Implement `ScyllaDriver` — basic connect | `driver/scylla.rs` | Integration: connect to container |
| 4 | Plain text authentication | `driver/scylla.rs` | Integration: auth connect |
| 5 | Execute raw CQL string | `driver/scylla.rs` | Integration: SELECT query |
| 6 | Prepared statement support | `driver/scylla.rs` | Integration: prepare + execute |
| 7 | Result set iteration with paging | `driver/scylla.rs` | Integration: large result set |
| 8 | CQL type mapping (all types) | `driver/scylla.rs` | Unit: each type |
| 9 | SSL/TLS with rustls | `driver/scylla.rs` | Integration: TLS connect |
| 10 | Connection timeout handling | `driver/scylla.rs` | Integration: timeout behavior |
| 11 | Protocol version negotiation | `driver/scylla.rs` | Integration: v4/v5 |
| 12 | Schema metadata queries | `driver/scylla.rs` | Integration: describe support |
| 13 | Keyspace switching | `session.rs` | Integration: USE keyspace |
| 14 | Consistency level management | `session.rs` | Integration: set/get consistency |
| 15 | Tracing session management | `session.rs` | Integration: tracing on/off |

### Acceptance Criteria

- [ ] Connect to Cassandra 3.11, 4.x, 5.x and ScyllaDB 5.x, 6.x
- [ ] Authenticate with username/password
- [ ] SSL/TLS connections work with all cert options from cqlshrc
- [ ] All CQL types can be queried and returned
- [ ] Paging works for large result sets
- [ ] Connection timeouts match `--connect-timeout` configuration
- [ ] Request timeouts match `--request-timeout` configuration

### Estimated Effort

- Research: 3 days
- Implementation: 5 days
- Testing: 3 days
- **Total: 11 days**

---

## Skills Required

- Async Rust / Tokio (S2)
- `scylla` crate API (C1)
- CQL protocol (S3)
- SSL/TLS with `rustls` (S8)

---

## Key Decisions

| Decision | Options | Recommendation |
|----------|---------|---------------|
| Primary driver | `scylla` vs `cdrs-tokio` | `scylla` (better maintained, ScyllaDB team) |
| Driver trait necessity | Trait vs direct scylla usage | Trait (testability, future flexibility) |
| TLS implementation | `rustls` vs `native-tls` | `rustls` (pure Rust, no system deps) |
| Connection pooling | scylla built-in vs custom | scylla built-in |
