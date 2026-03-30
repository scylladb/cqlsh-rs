# [tracing]

Request tracing settings.

```ini
[tracing]
max_trace_wait = 10.0
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_trace_wait` | float | `10.0` | Maximum seconds to wait for trace data to be available after a traced request completes. |

## Usage

Enable tracing in the shell with:

```
TRACING ON
```

Then each CQL statement will show trace output. View a specific trace with:

```
SHOW SESSION <trace-uuid>
```
