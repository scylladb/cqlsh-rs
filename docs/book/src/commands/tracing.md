# TRACING

Toggle request tracing for subsequent CQL statements.

## Syntax

```
TRACING ON
TRACING OFF
TRACING
```

## Usage

```
cqlsh> TRACING ON
Now tracing requests.

cqlsh> SELECT * FROM system.local;
-- query results --
-- trace output --
Tracing session: 12345678-1234-1234-1234-123456789abc

cqlsh> TRACING OFF
Disabled tracing.
```

## Notes

- `TRACING` with no argument disables tracing (same as `TRACING OFF`).
- Trace output is displayed after each query result.
- To view a trace later, use `SHOW SESSION <trace-uuid>`.
- The maximum wait time for trace data is controlled by `[tracing] max_trace_wait` in cqlshrc.
