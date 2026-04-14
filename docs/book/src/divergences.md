# Known Divergences

Intentional and unintentional differences between cqlsh-rs and Python cqlsh.

## Intentional differences

### TLS/SSL

- cqlsh-rs uses `rustls`, which does not support SSLv3, TLSv1.0, or TLSv1.1.
- The `[ssl] version` config key accepts `TLSv1_2` and `TLSv1_3` only.

### Driver

- cqlsh-rs uses the scylla-rust-driver instead of the Python cassandra-driver.
- Token-aware routing and connection pooling behavior may differ.
- Protocol version negotiation may select a different version in edge cases.

### COPY implementation

- Python cqlsh uses multiprocessing for COPY operations.
- cqlsh-rs uses async I/O (tokio), which may behave differently under heavy load.
- Error messages during COPY operations may differ in format.

### Pager

- cqlsh-rs uses a built-in pager (sapling-streampager) instead of shelling out to `less`.
- The `PAGING <N>` command accepts a page size for compatibility but uses the built-in pager.

## Minor formatting differences

- Floating-point formatting may show trailing zeros differently in edge cases.
- Timestamp formatting uses `chrono` instead of Python's `datetime`, which may produce slightly different results for edge-case timezone handling.
- Blob values are displayed identically (`0x...`) but internal formatting may differ for very large blobs.

## Missing features (planned)

These features from Python cqlsh are not yet implemented but are planned:

- COPY FROM (partially implemented)
- Per-topic HELP text (stub implementation)
- CQL type-specific formatting customization

## Reporting divergences

If you find a behavior difference not listed here, please [open an issue](https://github.com/scylladb/cqlsh-rs/issues).
