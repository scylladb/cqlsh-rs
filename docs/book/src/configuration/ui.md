# [ui]

Display and formatting settings.

```ini
[ui]
color = on
datetimeformat = %Y-%m-%d %H:%M:%S%z
timezone = UTC
float_precision = 5
double_precision = 12
max_trace_wait = 10.0
encoding = utf-8
completekey = tab
browser = open %s
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `color` | boolean | `on` | Enable colored output. Overridden by `-C` / `--no-color`. |
| `datetimeformat` | string | `%Y-%m-%d %H:%M:%S%z` | Format for timestamp values (strftime syntax). |
| `timezone` | string | local | Timezone for displaying timestamps. |
| `float_precision` | integer | `5` | Number of decimal digits for float values. |
| `double_precision` | integer | `12` | Number of decimal digits for double values. |
| `max_trace_wait` | float | `10.0` | Maximum seconds to wait for trace data. |
| `encoding` | string | `utf-8` | Character encoding for output. Overridden by `--encoding`. |
| `completekey` | string | `tab` | Key to trigger tab completion. |
| `browser` | string | | Browser command for CQL HELP. `%s` is replaced with the URL. |
