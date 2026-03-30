# [authentication]

Credentials for connecting to the cluster.

```ini
[authentication]
username = cassandra
password = cassandra
credentials = /path/to/credentials
keyspace = my_keyspace
```

## Keys

| Key | Type | Description |
|-----|------|-------------|
| `username` | string | Authentication username. Overridden by `-u` / `--username`. |
| `password` | string | Authentication password. Overridden by `-p` / `--password`. |
| `credentials` | string | Path to a credentials file (one `username=` and `password=` line each). |
| `keyspace` | string | Default keyspace to connect to. Overridden by `-k` / `--keyspace`. |

## Notes

- CLI flags (`-u`, `-p`, `-k`) take precedence over cqlshrc values.
- The credentials file is read if `username`/`password` are not set directly.
