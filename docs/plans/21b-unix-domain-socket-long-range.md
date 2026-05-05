# Sub-Plan SP21b: Unix Domain Socket Support — Long Range (Native Driver UDS)

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2 (Driver & Connection)
>
> **This is a living document.** Update it as development progresses.
>
> **Depends on**: [scylladb/scylla-rust-driver#1616](https://github.com/scylladb/scylla-rust-driver/issues/1616) — "Feature request: Unix domain socket support"
>
> **Replaces**: [SP21a — Short Range (Internal Proxy)](21a-unix-domain-socket-short-range.md) tasks 1.2 and 1.3

## Objective

Replace the internal TCP-to-UDS proxy (SP21a) with native driver UDS support once scylla-rust-driver implements [#1616](https://github.com/scylladb/scylla-rust-driver/issues/1616). This eliminates the proxy overhead and complexity while keeping the same user-facing behavior.

---

## Prerequisites

The scylla-rust-driver must implement (per issue #1616, milestone 1.7.0):

1. `KnownNode::UnixSocket(PathBuf)` variant
2. Transport abstraction supporting both TCP and Unix streams
3. Disabled topology refresh for UDS connections
4. `SessionBuilder::known_node_unix(path)` API

---

## What Changes from SP21a

| SP21a Component | Action | Reason |
|----------------|--------|--------|
| `src/driver/uds_proxy.rs` | **Delete entirely** | No longer needed — driver connects directly |
| `ScyllaDriver` proxy `JoinHandle` storage | **Remove** | No proxy to manage |
| `is_unix_socket()` detection (Task 1.1) | **Keep** | Still need to detect UDS vs hostname |
| `ConnectionConfig.unix_socket` (Task 1.4) | **Keep** | Still need to track UDS mode |
| LOGIN reconnection (Task 2.1) | **Keep** | Still need to preserve UDS mode |
| Platform guards (Task 3.1) | **Simplify** | Driver handles platform support |

---

## Implementation Tasks

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 1 | Upgrade scylla-rust-driver | Bump to version with UDS support (≥1.7.0 expected). Update `Cargo.toml`. | `cargo build` succeeds with new driver version |
| 2 | Replace proxy with direct UDS connection | In `ScyllaDriver::connect()`, when `is_unix_socket()` is true, use `SessionBuilder::known_node_unix(path)` instead of starting the proxy. | Connection to UDS works without proxy |
| 3 | Remove proxy infrastructure | Delete `src/driver/uds_proxy.rs`. Remove `JoinHandle` from `ScyllaDriver`. Remove proxy-related tests. | `cargo build` succeeds, no dead code warnings |
| 4 | Disable topology refresh for UDS | Configure the driver to skip node discovery when connected via UDS (driver may handle this automatically — verify). | No spurious connection attempts to discovered TCP nodes |
| 5 | Update tests | Replace proxy-based tests with direct UDS connection tests. Keep detection tests unchanged. | `cargo test` passes |
| 6 | Update documentation | Note in README that native UDS is now supported via the driver. Remove proxy architecture notes. | Docs are accurate |

---

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | Driver UDS API differs from expected | Adapt our integration code to match actual API. Detection and config plumbing remain stable. |
| RISK-02 | Driver version with UDS has breaking changes | Review changelog, update other driver usage as needed. |
| RISK-03 | Driver's UDS support is incomplete (e.g., no topology disable) | Keep manual topology disable logic from SP21a if needed. |

---

## Timeline

- **Blocked on**: scylla-rust-driver #1616 (milestone 1.7.0)
- **Effort once unblocked**: ~1-2 days (mostly deletion and simplification)
- **SP21a remains functional** until this plan is executed — no urgency
