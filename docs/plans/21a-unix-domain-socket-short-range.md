# Sub-Plan SP21a: Unix Domain Socket Support — Short Range (Internal Proxy)

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2 (Driver & Connection)
>
> **This is a living document.** Update it as development progresses.
>
> **Upstream reference**: [scylladb/scylla-cqlsh#67](https://github.com/scylladb/scylla-cqlsh/pull/67) — "Make cqlsh work with unix domain sockets"
>
> **Related**: [SP21b — Long Range (Native Driver UDS)](21b-unix-domain-socket-long-range.md)

## Objective

Enable cqlsh-rs to connect to Cassandra/ScyllaDB via Unix domain sockets (UDS) using a built-in TCP-to-UDS proxy. This is the immediate solution that requires no upstream driver changes.

When the user passes a filesystem path (e.g., `/var/run/scylla/maintenance.sock`) as the hostname, cqlsh-rs detects it is a socket and connects over UDS transparently.

---

## Background

The scylla-rust-driver (as of 2026) does **NOT** support Unix domain sockets natively. The connection layer is hardcoded to TCP. This plan implements a self-contained workaround: an async TCP-to-UDS proxy on localhost that bridges the driver's TCP connection to the actual Unix socket.

See [SP21b](21b-unix-domain-socket-long-range.md) for the long-term plan to use native driver UDS support once [scylla-rust-driver#1616](https://github.com/scylladb/scylla-rust-driver/issues/1616) is implemented.

---

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | When hostname is a Unix socket path, connect over UDS |
| REQ-02 | Requirement | Auto-detect socket vs hostname (check `stat()` for `S_ISSOCK`) |
| REQ-03 | Requirement | LOGIN reconnection must preserve UDS mode |
| REQ-04 | Requirement | Work on Linux and macOS (UDS not available on Windows) |
| CON-01 | Constraint | scylla-rust-driver has no native UDS support |
| CON-02 | Constraint | Must not fork or vendor the scylla-rust-driver |
| GUD-01 | Guideline | Keep the workaround isolated so it can be removed when driver adds UDS support (SP21b) |

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **How to connect over UDS** | Built-in async TCP-to-UDS proxy | Zero external dependencies, transparent to user, self-contained |
| **Proxy architecture** | Localhost ephemeral-port listener, forwards to UDS | Driver connects to `127.0.0.1:<ephemeral>` as normal TCP; proxy task bridges to `UnixStream` |
| **Detection logic** | `std::fs::metadata()` + `file_type().is_socket()` on `config.host` | Matches Python cqlsh behavior exactly |
| **Platform support** | `#[cfg(unix)]` gated | UDS is fundamentally a Unix concept |
| **Isolation** | All proxy code in `src/driver/uds_proxy.rs` | Easy to remove/replace when native driver UDS lands |

---

## Implementation Tasks

### Phase 1: UDS Detection & Proxy Infrastructure

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 1.1 | Add UDS detection utility | Create `fn is_unix_socket(path: &str) -> bool` in `src/driver/mod.rs` that calls `std::fs::metadata(path)` and checks `file_type().is_socket()`. Guard with `#[cfg(unix)]`. On non-unix, return `false`. | Unit test: returns `true` for a created UDS, `false` for regular file, `false` for nonexistent path |
| 1.2 | Implement TCP-to-UDS proxy | Create `src/driver/uds_proxy.rs`. Implement `async fn start_uds_proxy(socket_path: &str) -> Result<(SocketAddr, JoinHandle<()>)>` that: (a) binds a `TcpListener` on `127.0.0.1:0` (ephemeral port), (b) spawns a Tokio task that accepts TCP connections and bidirectionally copies to/from a `tokio::net::UnixStream` connected to `socket_path`, (c) returns the bound `SocketAddr` and the task handle. | Unit test: proxy forwards data bidirectionally between TCP client and mock UDS server |
| 1.3 | Wire proxy into `ScyllaDriver::connect()` | In `src/driver/scylla_driver.rs`, before building the `SessionBuilder`, check `is_unix_socket(&config.host)`. If true, call `start_uds_proxy(&config.host)` and use the returned `SocketAddr` as the contact point. Store the `JoinHandle` in `ScyllaDriver` for cleanup on drop. | Integration test: connect to a local ScyllaDB via UDS (if available), or unit test with mock |
| 1.4 | Update `ConnectionConfig` | Add `pub unix_socket: bool` field to `ConnectionConfig` to track whether UDS mode is active (set during detection). Used by LOGIN reconnection to preserve UDS mode. | Existing tests still compile and pass |

### Phase 2: LOGIN Reconnection & Docs

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 2.1 | Preserve UDS mode in LOGIN | In the LOGIN handler, ensure `new_config` preserves the original `host` value (which is the socket path). The detection in `connect()` handles the rest. | Manual test: `LOGIN` command reconnects over UDS when originally connected via UDS |
| 2.2 | Config documentation | Update `README.md` usage section and `src/cli.rs` help text to document UDS support: "If hostname is a Unix socket path, cqlsh-rs connects over the Unix domain socket." | README includes UDS usage example |

### Phase 3: Testing & Platform Guards

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 3.1 | Platform compilation guard | In `src/driver/uds_proxy.rs`, add `#[cfg(not(unix))]` stub that returns an error: "Unix domain sockets are not supported on this platform". | Code review confirms cfg gates are correct |
| 3.2 | Unit tests for proxy | Test proxy data forwarding: create a UDS listener, start proxy, connect TCP client through proxy, verify bidirectional data transfer. | `cargo test uds_proxy` passes |
| 3.3 | Unit tests for detection | Test `is_unix_socket()` with: real UDS, regular file, directory, nonexistent path, TCP hostname string. | `cargo test is_unix_socket` passes |
| 3.4 | Integration test (optional) | If CI has ScyllaDB with maintenance socket enabled, test full connection flow. Otherwise, document manual test procedure. | Test passes in CI or manual test procedure documented |

---

## Dependencies

| ID | Dependency | Required By |
|----|-----------|-------------|
| DEP-01 | `tokio::net::UnixStream` / `UnixListener` (tokio, already a dependency) | Task 1.2 |
| DEP-02 | No new crate dependencies required | All tasks |

---

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | Proxy adds latency to every CQL frame | Localhost TCP + UDS copy is sub-millisecond. Maintenance socket is low-throughput admin operations. |
| RISK-02 | Proxy task leak on disconnect | Store `JoinHandle` in `ScyllaDriver`, abort on drop. Use `tokio::select!` for clean shutdown. |
| RISK-03 | Driver discovers nodes and tries TCP connections | Disable auto-discovery when in UDS mode, or accept graceful failure for secondary connections. |
| RISK-04 | File path that exists but is not a socket | Check `S_ISSOCK` explicitly, not just path existence. Matches Python cqlsh behavior. |

---

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should we also support `--unix-socket <path>` explicit flag? | Open | Auto-detection matches Python cqlsh behavior; explicit flag could be added later |
| 2 | How does auto-node-discovery interact with UDS mode? | Open | Likely need to disable topology refresh when in UDS mode |
