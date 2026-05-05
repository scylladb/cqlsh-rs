# SP21a: Unix Domain Socket Proxy Implementation

## TL;DR

> **Quick Summary**: Implement UDS support in cqlsh-rs via a built-in TCP-to-UDS proxy that auto-detects Unix socket paths and transparently bridges the scylla driver's TCP connections.
>
> **Deliverables**:
> - `src/driver/uds_proxy.rs` — detection + proxy module
> - Modified `src/driver/mod.rs` — module registration + ConnectionConfig update
> - Modified `src/driver/scylla_driver.rs` — wiring + AbortOnDrop cleanup
> - Updated README.md and CLI help text
>
> **Estimated Effort**: Medium (2-3 days)
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 5

---

## Context

### Original Request
Implement the short-range UDS support plan (SP21a) — all code within cqlsh-rs, no upstream driver changes.

### Research Findings
- `ScyllaDriver::connect()` at `src/driver/scylla_driver.rs:422` builds `SessionBuilder::new().known_node(&addr)`
- `ScyllaDriver` struct has no `Drop` impl — need `AbortOnDrop` pattern for proxy cleanup
- `ProxyAddressTranslator` already redirects peers to contact point — pairs well with UDS proxy
- Tokio `net` feature (already enabled) provides `UnixStream`
- The driver opens multiple TCP connections (pool) — proxy MUST accept concurrent connections

### Metis Review
**Key findings incorporated**:
- Must handle multiple concurrent connections (not single-shot)
- Must use `AbortOnDrop` pattern (dropping JoinHandle does NOT cancel tasks)
- Must error early if SSL + UDS both specified
- Must follow symlinks for detection (`std::fs::metadata`, not `symlink_metadata`)
- Must use `tokio::io::copy_bidirectional` for half-close correctness

---

## Work Objectives

### Core Objective
When `config.host` is a Unix socket path, transparently proxy the driver's TCP connections to the UDS.

### Concrete Deliverables
- `src/driver/uds_proxy.rs` (new file ~150 lines)
- `src/driver/mod.rs` changes (module registration, `ConnectionConfig.unix_socket` field)
- `src/driver/scylla_driver.rs` changes (proxy wiring, AbortOnDrop field)
- README.md UDS usage example
- `src/cli.rs` help text update

### Definition of Done
- [ ] `cargo test --lib` passes with UDS proxy unit tests
- [ ] `cargo clippy` clean
- [ ] `cargo build` succeeds on Linux/macOS

### Must Have
- Auto-detection of Unix socket paths via `std::fs::metadata().file_type().is_socket()`
- TCP-to-UDS proxy accepting multiple concurrent connections
- Proxy task cleanup on ScyllaDriver drop (AbortOnDrop)
- `#[cfg(unix)]` guards on all UDS code
- Error if both SSL and UDS are specified

### Must NOT Have (Guardrails)
- No new crate dependencies
- No modifications to the `CqlDriver` trait
- No abstract "transport" layer — concrete proxy only
- No cqlshrc config file support for UDS (CLI auto-detect only)
- No health checks, reconnection logic, or metrics in the proxy
- No Windows support (compile-time guard)
- No support for multiple UDS paths or failover

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: YES (tests-after, inline in modules)
- **Framework**: cargo test

### QA Policy
Every task includes agent-executed QA scenarios verified via `cargo test` and `cargo build`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately — independent foundation):
├── Task 1: UDS detection + proxy module (uds_proxy.rs) [deep]
├── Task 4: ConnectionConfig update (mod.rs) [quick]

Wave 2 (After Wave 1 — integration + docs + tests):
├── Task 2: Wire proxy into ScyllaDriver::connect() [deep]  (depends: 1, 4)
├── Task 3: AbortOnDrop + cleanup [quick]  (depends: 1)
├── Task 5: LOGIN reconnection preservation [quick]  (depends: 2)
├── Task 6: Documentation (README + CLI help) [quick]  (depends: 1)

Wave FINAL (After ALL):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Cargo test + clippy verification (unspecified-high)
├── F4: Scope fidelity check (deep)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | — | 2, 3, 5, 6 |
| 4 | — | 2 |
| 2 | 1, 4 | 5 |
| 3 | 1 | — |
| 5 | 2 | — |
| 6 | 1 | — |

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 → `deep`, T4 → `quick`
- **Wave 2**: 4 tasks — T2 → `deep`, T3 → `quick`, T5 → `quick`, T6 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Create UDS detection and proxy module

  **What to do**:
  - Create `src/driver/uds_proxy.rs` with:
    - `#[cfg(unix)] pub fn is_unix_socket(path: &str) -> bool` — calls `std::fs::metadata(path)` (follows symlinks), checks `file_type().is_socket()`. Returns `false` for any error or non-socket.
    - `#[cfg(not(unix))] pub fn is_unix_socket(_path: &str) -> bool` — always returns `false`
    - `#[cfg(unix)] pub struct UdsProxy { abort_handle: tokio::task::AbortHandle }` — wrapper that aborts on drop
    - `#[cfg(unix)] impl Drop for UdsProxy { fn drop(&mut self) { self.abort_handle.abort(); } }`
    - `#[cfg(unix)] pub async fn start_uds_proxy(socket_path: &str) -> Result<(SocketAddr, UdsProxy)>` that:
      1. Binds `TcpListener` on `127.0.0.1:0`
      2. Gets the local_addr
      3. Spawns a task that loops: `listener.accept()` → spawn per-connection task that does `UnixStream::connect(socket_path)` + `tokio::io::copy_bidirectional`
      4. Returns `(local_addr, UdsProxy { abort_handle })`
    - `#[cfg(not(unix))] pub async fn start_uds_proxy(_socket_path: &str) -> Result<(SocketAddr, UdsProxy)>` — returns error "Unix domain sockets not supported on this platform"
  - Add `#[cfg(test)] mod tests` with:
    - Test `is_unix_socket` with real UDS (via `std::os::unix::net::UnixListener::bind(tempdir)`), regular file, nonexistent path, directory
    - Test proxy: create mock UDS echo server, start proxy, connect 3 TCP clients concurrently, verify bidirectional data
    - Test that dropping `UdsProxy` stops accepting connections
  - Register module in `src/driver/mod.rs`: add `pub mod uds_proxy;`

  **Must NOT do**:
  - No new dependencies
  - No abstract transport trait
  - No reconnection logic in proxy

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`rust-testing`]
    - `rust-testing`: needed for idiomatic test patterns with tokio test runtime

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 4)
  - **Blocks**: Tasks 2, 3, 5, 6
  - **Blocked By**: None

  **References**:
  - `src/driver/proxy_address_translator.rs` — Module structure and test style to follow
  - `src/driver/mod.rs:10` — Where to add `pub mod uds_proxy;`
  - Python cqlsh detection logic: `os.path.exists(hostname) and stat.S_ISSOCK(os.stat(hostname).st_mode)`

  **Acceptance Criteria**:
  - [ ] `cargo test uds_proxy` passes (≥5 tests: detection × 4, proxy × 1 multi-conn)
  - [ ] `cargo clippy -- -D warnings` clean for the new module
  - [ ] `#[cfg(unix)]` and `#[cfg(not(unix))]` stubs both present

  **QA Scenarios**:

  ```
  Scenario: Detection identifies real Unix socket
    Tool: Bash (cargo test)
    Preconditions: None (test creates temp socket internally)
    Steps:
      1. Run `cargo test --lib uds_proxy::tests::test_is_unix_socket`
      2. Assert exit code 0
    Expected Result: Test passes — returns true for socket, false for file/dir/nonexistent
    Evidence: .sisyphus/evidence/task-1-detection.txt

  Scenario: Proxy handles 3 concurrent connections
    Tool: Bash (cargo test)
    Preconditions: None (test creates mock UDS server internally)
    Steps:
      1. Run `cargo test --lib uds_proxy::tests::test_proxy_concurrent`
      2. Assert exit code 0
    Expected Result: 3 TCP clients each send/receive data through proxy to mock UDS echo server
    Evidence: .sisyphus/evidence/task-1-proxy-concurrent.txt
  ```

  **Commit**: YES
  - Message: `feat(driver): add UDS detection and TCP-to-UDS proxy module`
  - Files: `src/driver/uds_proxy.rs`, `src/driver/mod.rs`
  - Pre-commit: `cargo test --lib uds_proxy`

---

- [x] 2. Wire proxy into ScyllaDriver::connect()

  **What to do**:
  - In `src/driver/scylla_driver.rs` `connect()` method (line ~422):
    - After extracting `config.host` and `config.port`, check `uds_proxy::is_unix_socket(&config.host)`
    - If true AND `config.ssl` is true, return error: "SSL is not supported with Unix domain socket connections"
    - If true, call `uds_proxy::start_uds_proxy(&config.host).await?`
    - Use the returned `SocketAddr` as the addr for `SessionBuilder::known_node()`
    - Store the `UdsProxy` handle in the `ScyllaDriver` struct
    - Log at `debug!` level: "UDS proxy started on {addr} for socket {path}"
  - Modify `ScyllaDriver` struct to add: `#[cfg(unix)] _uds_proxy: Option<super::uds_proxy::UdsProxy>`
  - Update the `Ok(ScyllaDriver { ... })` construction to include the proxy field

  **Must NOT do**:
  - No changes to CqlDriver trait
  - No changes to connect() signature

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: [`rust-error-handling`]
    - `rust-error-handling`: SSL+UDS error case needs proper anyhow context

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 5
  - **Blocked By**: Tasks 1, 4

  **References**:
  - `src/driver/scylla_driver.rs:422-475` — The connect() method to modify
  - `src/driver/scylla_driver.rs:36-48` — ScyllaDriver struct to extend
  - `src/driver/uds_proxy.rs` — The module created in Task 1

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds
  - [ ] `cargo test --lib` passes (existing tests unbroken)
  - [ ] SSL+UDS combination returns clear error

  **QA Scenarios**:

  ```
  Scenario: Build succeeds with proxy wiring
    Tool: Bash
    Steps:
      1. Run `cargo build`
      2. Assert exit code 0
    Expected Result: No compilation errors
    Evidence: .sisyphus/evidence/task-2-build.txt

  Scenario: Existing tests still pass
    Tool: Bash
    Steps:
      1. Run `cargo test --lib`
      2. Assert exit code 0
    Expected Result: All existing tests pass, no regressions
    Evidence: .sisyphus/evidence/task-2-tests.txt
  ```

  **Commit**: YES
  - Message: `feat(driver): wire UDS proxy into ScyllaDriver::connect()`
  - Files: `src/driver/scylla_driver.rs`
  - Pre-commit: `cargo test --lib`

---

- [x] 3. Implement AbortOnDrop cleanup verification

  **What to do**:
  - Verify that `UdsProxy::drop()` (implemented in Task 1) properly aborts the proxy task
  - Add a test in `uds_proxy.rs` tests: start proxy, drop the `UdsProxy`, then verify new TCP connections to the proxy port are refused
  - This confirms the cleanup contract

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`rust-testing`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 2, 5, 6)
  - **Blocks**: None
  - **Blocked By**: Task 1

  **References**:
  - `src/driver/uds_proxy.rs` — UdsProxy struct and Drop impl from Task 1

  **Acceptance Criteria**:
  - [ ] `cargo test --lib uds_proxy::tests::test_proxy_cleanup_on_drop` passes

  **QA Scenarios**:

  ```
  Scenario: Proxy stops after UdsProxy is dropped
    Tool: Bash (cargo test)
    Steps:
      1. Run `cargo test --lib uds_proxy::tests::test_proxy_cleanup_on_drop`
      2. Assert exit code 0
    Expected Result: After drop, TCP connect to proxy port fails with connection refused
    Evidence: .sisyphus/evidence/task-3-cleanup.txt
  ```

  **Commit**: NO (groups with Task 1 or squash later)

---

- [x] 4. Update ConnectionConfig with unix_socket field

  **What to do**:
  - In `src/driver/mod.rs`, add to `ConnectionConfig` struct: `pub unix_socket: bool,`
  - Default to `false`
  - Update all places that construct `ConnectionConfig` to include `unix_socket: false` (search codebase for `ConnectionConfig {`)
  - This field is informational — set to `true` in connect() when UDS is detected (done in Task 2)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - `src/driver/mod.rs:24-46` — ConnectionConfig struct definition
  - Search for `ConnectionConfig {` across codebase to find all construction sites

  **Acceptance Criteria**:
  - [ ] `cargo build` succeeds
  - [ ] `cargo test --lib` passes (no regressions)

  **QA Scenarios**:

  ```
  Scenario: Project compiles with new field
    Tool: Bash
    Steps:
      1. Run `cargo build`
      2. Assert exit code 0
    Expected Result: No compilation errors
    Evidence: .sisyphus/evidence/task-4-build.txt
  ```

  **Commit**: YES
  - Message: `feat(driver): add unix_socket field to ConnectionConfig`
  - Files: `src/driver/mod.rs`, plus any files constructing ConnectionConfig
  - Pre-commit: `cargo build`

---

- [x] 5. Preserve UDS mode in LOGIN reconnection

  **What to do**:
  - Find the LOGIN handler in `src/main.rs` (or wherever reconnection logic lives)
  - Ensure that when building a new `ConnectionConfig` for reconnection, the original `host` value (the socket path) is preserved
  - The detection in `connect()` handles the rest — no special UDS logic needed here, just don't overwrite host
  - Verify the LOGIN flow works by reading the code path

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: Task 2

  **References**:
  - `src/main.rs` — LOGIN handler (search for "LOGIN" or "login" command handling)
  - `src/session.rs` — May contain reconnection logic

  **Acceptance Criteria**:
  - [ ] Code review confirms host is preserved during LOGIN reconnection
  - [ ] `cargo build` succeeds

  **QA Scenarios**:

  ```
  Scenario: LOGIN preserves host path
    Tool: Bash (grep)
    Steps:
      1. Search for LOGIN reconnection code path
      2. Verify new ConnectionConfig uses original config.host (not a modified version)
    Expected Result: host field is copied from original config
    Evidence: .sisyphus/evidence/task-5-login-review.txt
  ```

  **Commit**: YES (if changes needed)
  - Message: `fix(session): preserve UDS host path during LOGIN reconnection`
  - Pre-commit: `cargo build`

---

- [x] 6. Documentation updates

  **What to do**:
  - Add UDS usage section to `README.md` under Usage:
    ```
    # Connect via Unix domain socket (ScyllaDB maintenance socket)
    cqlsh-rs /var/run/scylla/cql.sock
    ```
  - Update `src/cli.rs` help text for the host argument to mention: "Can be a hostname, IP address, or path to a Unix domain socket"
  - Note platform limitation (Linux/macOS only)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: None
  - **Blocked By**: Task 1

  **References**:
  - `README.md` — Usage section (line ~70+)
  - `src/cli.rs` — Host argument definition

  **Acceptance Criteria**:
  - [ ] README includes UDS usage example
  - [ ] `cargo build` succeeds (cli.rs changes compile)

  **QA Scenarios**:

  ```
  Scenario: README contains UDS documentation
    Tool: Bash (grep)
    Steps:
      1. Run `grep -c "unix.*socket\|Unix.*socket\|UDS\|\.sock" README.md`
      2. Assert count ≥ 2
    Expected Result: README mentions Unix socket usage
    Evidence: .sisyphus/evidence/task-6-docs.txt
  ```

  **Commit**: YES
  - Message: `docs: add Unix domain socket usage documentation`
  - Files: `README.md`, `src/cli.rs`
  - Pre-commit: `cargo build`

---

## Final Verification Wave

- [x] F1. **Plan Compliance Audit** — `oracle`
  Read SP21a plan. For each Must Have: verify implementation exists. For each Must NOT Have: search for forbidden patterns. Check all tasks completed.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | VERDICT`

- [x] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy -- -D warnings` + `cargo test --lib`. Review `uds_proxy.rs` for: proper error handling, no unwrap in non-test code, proper cfg guards, no dead code.
  Output: `Clippy [PASS/FAIL] | Tests [PASS/FAIL] | VERDICT`

- [x] F3. **Real QA** — `unspecified-high`
  Run full test suite. Verify proxy tests exercise concurrent connections. Check that dropping ScyllaDriver cleans up proxy.
  Output: `Tests [N pass/N fail] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  Verify no new dependencies added to Cargo.toml. Verify CqlDriver trait unchanged. Verify no code outside src/driver/ was modified (except README, cli.rs, main.rs).
  Output: `Scope [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

| Order | Message | Files | Pre-commit |
|-------|---------|-------|------------|
| 1 | `feat(driver): add unix_socket field to ConnectionConfig` | mod.rs + constructors | `cargo build` |
| 2 | `feat(driver): add UDS detection and TCP-to-UDS proxy module` | uds_proxy.rs, mod.rs | `cargo test --lib uds_proxy` |
| 3 | `feat(driver): wire UDS proxy into ScyllaDriver::connect()` | scylla_driver.rs | `cargo test --lib` |
| 4 | `fix(session): preserve UDS host path during LOGIN reconnection` | main.rs/session.rs | `cargo build` |
| 5 | `docs: add Unix domain socket usage documentation` | README.md, cli.rs | `cargo build` |

---

## Success Criteria

### Verification Commands
```bash
cargo build                    # Expected: success
cargo test --lib               # Expected: all tests pass
cargo clippy -- -D warnings    # Expected: no warnings
cargo test --lib uds_proxy     # Expected: ≥5 tests pass
```

### Final Checklist
- [ ] UDS auto-detection works (is_unix_socket)
- [ ] Proxy handles concurrent connections
- [ ] Proxy aborts on ScyllaDriver drop
- [ ] SSL + UDS returns clear error
- [ ] #[cfg(unix)] guards correct
- [ ] No new dependencies
- [ ] CqlDriver trait unchanged
- [ ] README documents UDS usage
