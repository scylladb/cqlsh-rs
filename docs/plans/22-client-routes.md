# Sub-Plan SP22: Client Routes (PrivateLink) Support

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2 (Driver & Connection)
>
> **This is a living document.** Update it as development progresses.
>
> **Tracking issue**: [scylladb/cqlsh-rs#182](https://github.com/scylladb/cqlsh-rs/issues/182)
>
> **Upstream references**:
> - [scylladb/scylla-cqlsh#204](https://github.com/scylladb/scylla-cqlsh/pull/204) — Python cqlsh implementation
> - [scylladb/scylla-cqlsh#207](https://github.com/scylladb/scylla-cqlsh/pull/207) — Python cqlsh follow-up fixes
> - [scylladb/scylladb#27323](https://github.com/scylladb/scylladb/pull/27323) — `system.client_routes` + `/v2/client-routes` REST API (ScyllaDB 2026.1+)
> - [Rust driver docs](https://rust-driver.docs.scylladb.com/stable/connecting/client-routes.html)

## Objective

Let cqlsh-rs connect to ScyllaDB deployments behind a proxy — AWS PrivateLink,
GCP Private Service Connect — where the node addresses advertised in
`system.peers` are unreachable from the client. Configuring one or more
connection IDs makes the driver read `system.client_routes` and translate each
node's `host_id` to its published, reachable address.

Flag and config spelling match Python cqlsh, per the project's compatibility
constraint.

---

## Background

Each node in such a deployment publishes a row in `system.client_routes`:

```
connection_id text, host_id uuid, address text, port int, tls_port int,
alternator_port int, alternator_https_port int
PRIMARY KEY (connection_id, host_id)
```

The `connection_id` is assigned by the cloud provider. The driver filters the
table by the configured connection IDs, uses the resulting addresses as its
address translation table, and follows `CLIENT_ROUTES_CHANGE` events.

---

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | `--client-route CONNECTION_ID[=ADDRESS]`, repeatable, matching Python cqlsh |
| REQ-02 | Requirement | `[client_routes]` cqlshrc section with `proxies` and `advanced_shard_awareness` |
| REQ-03 | Requirement | With no explicit host, route address overrides become the contact points |
| REQ-04 | Requirement | `LOGIN` reconnection preserves client routes |
| REQ-05 | Requirement | Reject `--ssl` + client routes at startup with a clear message |
| CON-01 | Constraint | Driver support is behind the `unstable-client-routes` cargo feature |
| CON-02 | Constraint | The driver does not support TLS with client routes |
| CON-03 | Constraint | The driver rejects a user-supplied address translator with client routes |
| CON-04 | Constraint | `ClientRoutesSessionBuilder` is a distinct type from `SessionBuilder` |
| CON-05 | Constraint | Server support requires ScyllaDB 2026.1 or later |

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Driver feature gate** | Enable `unstable-client-routes` unconditionally | Avoids `#[cfg]` noise across cli/config/session/driver; a driver minor bump may need a touch-up |
| **Spec parsing** | Shared `src/client_routes.rs`, used by both CLI and cqlshrc | One parser, one error vocabulary, matching Python's `parse_client_route_spec` |
| **CLI vs cqlshrc merge** | CLI `--client-route` replaces `proxies` wholesale | Matches Python cqlsh |
| **Builder sharing** | Generic `apply_common_options<K: SessionBuilderKind>` | Default and client-routes builders are different types but share the common option impl block |
| **Address translator** | Skipped entirely in client-routes mode | The driver installs its own; a user-supplied one is rejected at compile time and at runtime |
| **Multi-line INI values** | `Ini::set_multiline(true)` | Python's `configparser` supports continuation lines for every option; `proxies` is documented in that form |
| **Advanced shard awareness** | `disallow_shard_aware_port(false)` when requested | `ClientRoutesSessionBuilder::new` disables the shard-aware port by default |

---

## Implementation

| # | Area | Change |
|---|------|--------|
| 1 | `Cargo.toml` | Add `unstable-client-routes` to the `scylla` features; add the `test-client-routes` test feature |
| 2 | `src/client_routes.rs` | `ClientRoute`, `ClientRoutesSettings`, `parse_client_route_spec`, `parse_client_routes` |
| 3 | `src/cli.rs` | `--client-route`, `--client-routes-advanced-shard-awareness`, `--no-…`; conflict and spec validation in `validate()` |
| 4 | `src/config.rs` | `ClientRoutesSection`; `MergedConfig::{client_routes, contact_points}`; contact-point derivation; multi-line INI; post-merge SSL conflict check in `load_config` |
| 5 | `src/driver/{mod,scylla_driver}.rs` | `ConnectionConfig::{client_routes, contact_points}`; `apply_common_options`; `ClientRoutesSessionBuilder` branch; `build_client_routes_config` |
| 6 | `src/main.rs` | `--debug` lines for routes, shard awareness, and contact points |

`SOURCE` and `COPY` needed no changes: unlike Python cqlsh, cqlsh-rs runs both
against the live session rather than a subshell or worker clusters. `LOGIN`
inherits routes for free because both reconnect paths clone `MergedConfig`.

---

## Testing

| Level | Coverage |
|-------|----------|
| Unit — `src/client_routes.rs` | Spec forms, separators, whitespace, error cases |
| Unit — `src/cli.rs` | Repeats accumulate, flag conflicts, SSL conflict, bad specs |
| Unit — `src/config.rs` | Section parsing incl. continuation lines, CLI-replaces-file, shard-awareness precedence, contact-point derivation, `load_config` rejections |
| Unit — `src/driver/scylla_driver.rs` | Driver config construction from settings |
| CLI — `tests/cli_tests.rs` | Flags accepted, help listing, exit codes and messages |
| Config — `tests/cqlshrc_tests.rs` | `[client_routes]` asserted through `--debug` output |
| Integration — `tests/integration/client_routes_tests.rs` | End-to-end through two forwarders (see below) |

The integration tests place cqlsh-rs behind forwarder **A** (the contact point)
and publish a route pointing at forwarder **B**:

```
cqlsh-rs → forwarder A (contact point)        → ScyllaDB CQL port
           forwarder B (address in the table) → ScyllaDB CQL port
```

Traffic arriving at B can only be the result of address translation through
`system.client_routes` — a single-forwarder setup would pass even with routing
disabled, since the node's own address is directly reachable from the test host.
Routes are seeded by POSTing to `/v2/client-routes` on the node REST API (port
10000), so the CI job exposes that port and starts ScyllaDB with
`--api-address 0.0.0.0`. Tests skip themselves when the table is absent.

Run locally:

```bash
cargo test --features test-client-routes --test integration -- --ignored --test-threads=1
```

---

## Status

| Task | Status |
|------|--------|
| Driver feature + shared parser module | ✅ Done |
| CLI flags and validation | ✅ Done |
| cqlshrc section and merged config | ✅ Done |
| Driver client-routes session path | ✅ Done |
| Unit, CLI, and cqlshrc tests | ✅ Done |
| Integration tests + `integration-client-routes` CI job | ✅ Done |
| Documentation | ✅ Done |

## Follow-ups

- **TLS** — blocked on driver support ("TODO: support TLS for ClientRoutesMode
  once Cloud comes up with a solution" in `session_builder.rs`). The
  combination is rejected until then.
- **Unix domain sockets** — when [SP21a](21a-unix-domain-socket-short-range.md)
  lands, a UDS host combined with client routes should be rejected the same way
  `--ssl` is; the two connection models are mutually exclusive.
- The driver feature is named *unstable*; revisit on each `scylla` minor bump.
