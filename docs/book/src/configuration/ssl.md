# [ssl]

SSL/TLS certificate and validation settings.

```ini
[ssl]
certfile = /path/to/ca-cert.pem
validate = true
userkey = /path/to/client-key.pem
usercert = /path/to/client-cert.pem
version = TLSv1_2
```

## Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `certfile` | string | | Path to the CA certificate file for server verification. Also set via `SSL_CERTFILE` env var. |
| `validate` | boolean | `true` | Whether to validate the server certificate. Also set via `SSL_VALIDATE` env var. |
| `userkey` | string | | Path to the client private key (for mutual TLS). |
| `usercert` | string | | Path to the client certificate (for mutual TLS). |
| `version` | string | | Minimum TLS version (`TLSv1_2`, `TLSv1_3`). |

## [certfiles] section

Map per-host certificate files:

```ini
[certfiles]
10.0.0.1 = /path/to/cert-host1.pem
10.0.0.2 = /path/to/cert-host2.pem
```

## Enabling SSL

1. Set `--ssl` on the command line, or
2. Set `factory = cqlshlib.ssl.ssl_transport_factory` in `[connection]`

## Example: mutual TLS

```ini
[connection]
factory = cqlshlib.ssl.ssl_transport_factory

[ssl]
certfile = /etc/cassandra/ssl/ca-cert.pem
userkey = /etc/cassandra/ssl/client-key.pem
usercert = /etc/cassandra/ssl/client-cert.pem
validate = true
```

```bash
cqlsh-rs --ssl 10.0.0.1
```
