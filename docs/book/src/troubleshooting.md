# Troubleshooting

Common issues and their solutions.

## Connection issues

### "Connection refused" or "Connection timed out"

- Verify the host and port are correct: `cqlsh-rs 10.0.0.1 9042`
- Check that Cassandra is running and listening on the native transport port
- Check firewall rules allow connections to port 9042
- Increase the connect timeout: `--connect-timeout 30`

### "Authentication failed"

- Verify username and password: `cqlsh-rs -u cassandra -p cassandra`
- Check that the authenticator is configured in `cassandra.yaml`
- Try credentials from your cqlshrc: `cat ~/.cassandra/cqlshrc`

### SSL/TLS errors

- Verify the certificate file exists and is readable
- Check certificate validity: `openssl x509 -in cert.pem -text -noout`
- Ensure TLS 1.2+ is supported (cqlsh-rs doesn't support older protocols)
- For self-signed certs, set `validate = false` in `[ssl]` (not recommended for production)

```ini
[ssl]
certfile = /path/to/ca-cert.pem
validate = true
```

### "Protocol version mismatch"

- Try specifying a protocol version: `--protocol-version 4`
- Cassandra 2.x supports protocol v1-v3
- Cassandra 3.x supports protocol v1-v4
- Cassandra 4.x+ supports protocol v4-v5

## Shell issues

### Tab completion not working

- Tab completion requires an active connection to fetch schema metadata
- Try reconnecting or running `DESCRIBE KEYSPACES` to refresh the schema cache

### History not saving

- Check that `~/.cassandra/` directory exists and is writable
- Verify `--disable-history` is not set
- Check `CQL_HISTORY` environment variable isn't set to an invalid path

### Pager not working

- Paging is automatically disabled when output is not a TTY
- Check with: `PAGING` (shows current state)
- Enable with: `PAGING ON`

## COPY issues

### "File I/O is disabled"

- Remove the `--no-file-io` flag to enable COPY, SOURCE, and CAPTURE commands

### COPY TO produces empty file

- Verify the table has data: `SELECT COUNT(*) FROM table;`
- Check the keyspace is correct (fully qualify: `COPY keyspace.table TO ...`)

### COPY FROM timeout errors

- Reduce batch size: `WITH CHUNKSIZE = 100`
- Reduce ingest rate: `WITH INGESTRATE = 10000`
- Increase request timeout: `--request-timeout 60`

## Configuration issues

### cqlshrc not being read

- Default location: `~/.cassandra/cqlshrc`
- Override with: `--cqlshrc /path/to/cqlshrc`
- Verify the file is valid INI format

### Settings not taking effect

Remember the precedence order:
1. CLI flags (highest)
2. Environment variables
3. cqlshrc file
4. Built-in defaults (lowest)

A CLI flag always overrides a cqlshrc value.

## Getting help

- File an issue: [github.com/fruch/cqlsh-rs/issues](https://github.com/fruch/cqlsh-rs/issues)
- Check existing issues for known problems
- Include `cqlsh-rs --version` output and `--debug` output when reporting issues
