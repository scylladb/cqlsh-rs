# [connection]

Network and timeout settings.

```ini
[connection]
hostname = 127.0.0.1
port = 9042
factory = cqlshlib.ssl.ssl_transport_factory
timeout = 10
connect_timeout = 5
request_timeout = 10
client_timeout = 120
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `hostname` | string | `127.0.0.1` | Contact point hostname or IP. Overridden by positional `host` arg or `CQLSH_HOST`. |
| `port` | integer | `9042` | Native transport port. Overridden by positional `port` arg or `CQLSH_PORT`. |
| `factory` | string | | Connection factory (for SSL, set to `cqlshlib.ssl.ssl_transport_factory`). |
| `timeout` | integer | `10` | General timeout in seconds (legacy). |
| `connect_timeout` | integer | `5` | Connection timeout in seconds. Overridden by `--connect-timeout` or `CQLSH_DEFAULT_CONNECT_TIMEOUT_SECONDS`. |
| `request_timeout` | integer | `10` | Per-request timeout in seconds. Overridden by `--request-timeout` or `CQLSH_DEFAULT_REQUEST_TIMEOUT_SECONDS`. |
| `client_timeout` | integer | `120` | Client-side timeout for long operations. |
