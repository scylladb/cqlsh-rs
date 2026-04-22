# Sub-Plan SP21: Unix Domain Socket Support

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2 (Driver & Connection)
>
> **This is a living document.** Update it as development progresses.
>
> **Upstream reference**: [scylladb/scylla-cqlsh#67](https://github.com/scylladb/scylla-cqlsh/pull/67) — "Make cqlsh work with unix domain sockets"
> **Upstream issue**: [scylladb/scylladb#16489](https://github.com/scylladb/scylladb/issues/16489)

## Objective

Enable cqlsh-rs to connect to Cassandra/ScyllaDB via Unix domain sockets (UDS), matching the behavior added to Python cqlsh in [scylla-cqlsh#67](https://github.com/scylladb/scylla-cqlsh/pull/67). When the user passes a filesystem path (e.g., `/var/run/scylla/maintenance.sock`) as the hostname, cqlsh-rs detects it is a socket and connects over UDS instead of TCP.

---

## Background

### What the upstream PR does (Python cqlsh)

The Python PR checks if the provided hostname is a Unix socket (`os.path.exists(hostname) and stat.S_ISSOCK(...)`). If so, it:
1. Wraps the hostname in `UnixSocketEndPoint` for the python-driver
2. Uses `WhiteListRoundRobinPolicy([UnixSocketEndPoint(hostname)])` for load balancing

This is used for ScyllaDB's **maintenance socket** — a local-only CQL endpoint exposed as a Unix domain socket for administrative operations without network overhead.

### Challenge for cqlsh-rs

The scylla-rust-driver (as of 2026) does **NOT** support Unix domain sockets natively:
- `SessionBuilder::known_node()` only accepts hostname strings resolved to `SocketAddr` (TCP)
- `SessionBuilder::known_node_addr()` only accepts `SocketAddr` (IPv4/IPv6)
- The internal connection layer (`connection.rs`) is hardcoded to `TcpSocket` / `TcpStream`
- There is no pluggable transport trait

This means we cannot simply pass a UDS path to the driver. We need a workaround.

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
| GUD-01 | Guideline | Prefer upstreaming UDS support to scylla-rust-driver long-term |
| GUD-02 | Guideline | Keep the workaround isolated so it can be removed when driver adds UDS support |

---

## Design Decisions

| Decision | Choice | Rationale | Alternatives Rejected |
|----------|--------|-----------|----------------------|
| **How to connect over UDS** | Built-in async TCP-to-UDS proxy | Zero external dependencies, transparent to user, self-contained | External `socat` proxy (requires extra tooling), Driver fork (maintenance burden), Wait for upstream (blocks feature) |
| **Proxy architecture** | Localhost ephemeral-port listener, forwards to UDS | Driver connects to `127.0.0.1:<ephemeral>` as normal TCP; proxy task bridges to `UnixStream` | Custom transport trait in driver (requires upstream changes) |
| **Detection logic** | `std::fs::metadata()` + `file_type().is_socket()` on `config.host` | Matches Python cqlsh behavior exactly | Explicit `--unix-socket` flag (breaks compatibility) |
| **Platform support** | `#[cfg(unix)]` gated, compile error with helpful message on Windows | UDS is fundamentally a Unix concept | Silent fallback to TCP (confusing) |

---

## Implementation Tasks

### Phase 1: UDS Detection & Proxy Infrastructure

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 1.1 | Add UDS detection utility | Create `fn is_unix_socket(path: &str) -> bool` in `src/driver/mod.rs` that calls `std::fs::metadata(path)` and checks `file_type().is_socket()`. Guard with `#[cfg(unix)]`. On non-unix, return `false`. | Unit test: returns `true` for a created UDS, `false` for regular file, `false` for nonexistent path |
| 1.2 | Implement TCP-to-UDS proxy | Create `src/driver/uds_proxy.rs`. Implement `async fn start_uds_proxy(socket_path: &str) -> Result<SocketAddr>` that: (a) binds a `TcpListener` on `127.0.0.1:0` (ephemeral port), (b) spawns a Tokio task that accepts TCP connections and bidirectionally copies to/from a `tokio::net::UnixStream` connected to `socket_path`, (c) returns the bound `SocketAddr`. | Unit test: proxy forwards data bidirectionally between TCP client and mock UDS server |
| 1.3 | Wire proxy into `ScyllaDriver::connect()` | In `src/driver/scylla_driver.rs`, before building the `SessionBuilder`, check `is_unix_socket(&config.host)`. If true, call `start_uds_proxy(&config.host)` and use the returned `SocketAddr` as the contact point instead. Store the proxy handle in `ScyllaDriver` for cleanup. | Integration test: connect to a local ScyllaDB via UDS (if available), or unit test with mock |
| 1.4 | Update `ConnectionConfig` | Add `pub unix_socket: bool` field to `ConnectionConfig` in `src/driver/mod.rs` to track whether UDS mode is active (set during detection). This field is used by LOGIN reconnection to preserve UDS mode. | Existing tests still compile and pass |

### Phase 2: LOGIN Reconnection & Config Plumbing

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 2.1 | Preserve UDS mode in LOGIN | In `src/main.rs` LOGIN handler (line ~530), ensure `new_config` preserves the original `host` value (which is the socket path). The detection in `connect()` handles the rest. | Manual test: `LOGIN` command reconnects over UDS when originally connected via UDS |
| 2.2 | Config documentation | Update `README.md` usage section and `src/cli.rs` help text to document UDS support: "If hostname is a Unix socket path, cqlsh-rs connects over the Unix domain socket." | README includes UDS usage example |

### Phase 3: Testing & Platform Guards

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 3.1 | Platform compilation guard | In `src/driver/uds_proxy.rs`, add `#[cfg(not(unix))]` stub that returns an error: "Unix domain sockets are not supported on this platform". | `cargo check --target x86_64-pc-windows-msvc` compiles (if cross-compilation available), or code review confirms cfg gates |
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

## Testing Strategy

| ID | Test Type | Description | Validation |
|----|-----------|-------------|------------|
| TEST-01 | Unit | `is_unix_socket()` with various path types | Returns correct bool for each case |
| TEST-02 | Unit | UDS proxy bidirectional data forwarding | Data sent from TCP client arrives at UDS server and vice versa |
| TEST-03 | Unit | Proxy returns valid ephemeral port | Returned `SocketAddr` is `127.0.0.1` with non-zero port |
| TEST-04 | Integration | Full connection through proxy to real DB (optional) | `SELECT * FROM system.local` succeeds |
| TEST-05 | Compile | Windows compilation with `#[cfg(unix)]` guards | No compile errors on non-unix targets |

---

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | Proxy adds latency to every CQL frame | Measure overhead; localhost TCP + UDS copy should be sub-millisecond. The maintenance socket use case is low-throughput admin operations. |
| RISK-02 | Proxy task leak on disconnect | Store `JoinHandle` in `ScyllaDriver`, abort on drop. Use `tokio::select!` for clean shutdown. |
| RISK-03 | scylla-rust-driver discovers nodes and tries to connect to them via TCP | For maintenance socket use case, there's typically only one node. Disable auto-discovery or accept that secondary connections may fail gracefully. |
| RISK-04 | File path that exists but is not a socket (e.g., hostname `localhost` matching a file) | Check `S_ISSOCK` explicitly, not just path existence. This matches the Python cqlsh behavior. |

---

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should we also support `--unix-socket <path>` explicit flag in addition to auto-detection? | Open | Auto-detection matches Python cqlsh behavior; explicit flag could be added later for clarity |
| 2 | Should we file an upstream issue on scylla-rust-driver for native UDS support? | Open | Yes, recommended. Would allow removing the proxy workaround eventually |
| 3 | How does auto-node-discovery interact with UDS mode? | Open | Likely need to investigate `SessionBuilder` options to disable topology refresh when in UDS mode |

---

## Long-Term Path

The TCP-to-UDS proxy is a **pragmatic workaround**. The ideal solution is native UDS support in scylla-rust-driver, which would require:
1. A `KnownNode::UnixSocket(PathBuf)` variant
2. A pluggable transport abstraction (`TcpStream` vs `UnixStream`)
3. Changes to node discovery to handle UDS endpoints

Once upstream support lands, tasks 1.2 and 1.3 can be replaced with direct `SessionBuilder` configuration, while the detection logic (1.1) and config plumbing (1.4, 2.1) remain unchanged.
