# Sub-Plan SP21: Unix Domain Socket Support

> **This plan has been split into two documents:**
>
> - **[SP21a — Short Range (Internal Proxy)](21a-unix-domain-socket-short-range.md)** — Immediate implementation using a built-in TCP-to-UDS proxy. Fully self-contained within cqlsh-rs, no upstream changes needed.
>
> - **[SP21b — Long Range (Native Driver UDS)](21b-unix-domain-socket-long-range.md)** — Future migration to native driver UDS support once [scylladb/scylla-rust-driver#1616](https://github.com/scylladb/scylla-rust-driver/issues/1616) is implemented (milestone 1.7.0). Replaces the proxy with direct `SessionBuilder::known_node_unix()`.
>
> **Upstream references**:
> - [scylladb/scylla-cqlsh#67](https://github.com/scylladb/scylla-cqlsh/pull/67) — Python cqlsh UDS support
> - [scylladb/scylladb#16489](https://github.com/scylladb/scylladb/issues/16489) — ScyllaDB maintenance socket
> - [scylladb/scylla-rust-driver#1616](https://github.com/scylladb/scylla-rust-driver/issues/1616) — Native driver UDS support request
