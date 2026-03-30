# [csv]

CSV parsing settings for COPY operations.

```ini
[csv]
field_size_limit = 131072
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `field_size_limit` | integer | `131072` | Maximum size in bytes of a single CSV field. Increase for tables with large text/blob columns. |
