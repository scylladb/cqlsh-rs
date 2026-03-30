# CLI Reference

Complete reference for all `cqlsh-rs` command-line flags and arguments.

## Synopsis

```
cqlsh-rs [OPTIONS] [host] [port]
```

## Positional arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `host` | Contact point hostname or IP address | `127.0.0.1` |
| `port` | Native transport port | `9042` |

## Options

### Connection

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--ssl` | | Enable SSL/TLS connection | off |
| `--connect-timeout SECONDS` | | Connection timeout in seconds | 5 |
| `--request-timeout SECONDS` | | Per-request timeout in seconds | 10 |
| `--protocol-version VERSION` | | Native protocol version (1-6) | auto-negotiate |
| `--secure-connect-bundle BUNDLE` | `-b` | Secure connect bundle for Astra DB | |

### Authentication

| Flag | Short | Description |
|------|-------|-------------|
| `--username USERNAME` | `-u` | Authentication username |
| `--password PASSWORD` | `-p` | Authentication password |

### Execution

| Flag | Short | Description |
|------|-------|-------------|
| `--execute STATEMENT` | `-e` | Execute a CQL statement and exit |
| `--file FILE` | `-f` | Execute CQL statements from a file |
| `--keyspace KEYSPACE` | `-k` | Set the initial keyspace |

Note: `--execute` and `--file` are mutually exclusive.

### Display

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--color` | `-C` | Force colored output | auto-detect |
| `--no-color` | | Disable colored output | |
| `--encoding ENCODING` | | Character encoding | `utf-8` |
| `--tty` | `-t` | Force TTY mode | auto-detect |

Note: `--color` and `--no-color` are mutually exclusive.

### Configuration

| Flag | Description | Default |
|------|-------------|---------|
| `--cqlshrc FILE` | Path to cqlshrc configuration file | `~/.cassandra/cqlshrc` |
| `--cqlversion VERSION` | CQL version to use | auto-detect |
| `--consistency-level LEVEL` | Initial consistency level | `ONE` |
| `--serial-consistency-level LEVEL` | Initial serial consistency level | `SERIAL` |

### Behavior

| Flag | Description |
|------|-------------|
| `--no-file-io` | Disable file I/O commands (COPY, SOURCE, CAPTURE) |
| `--no_compact` | Disable compact storage interpretation |
| `--disable-history` | Disable saving of command history |
| `--debug` | Show additional debug information |
| `--browser BROWSER` | Browser for CQL HELP (unused in modern cqlsh) |

### Utility

| Flag | Description |
|------|-------------|
| `--completions SHELL` | Generate shell completion script (bash, zsh, fish, elvish, powershell) |
| `--version` | Show version and exit |
| `--help` | Show help and exit |

## Environment variables

| Variable | Description | Equivalent flag |
|----------|-------------|-----------------|
| `CQLSH_HOST` | Default contact point hostname | `host` positional |
| `CQLSH_PORT` | Default native transport port | `port` positional |
| `SSL_CERTFILE` | SSL certificate file path | `--ssl` + cqlshrc `[ssl] certfile` |
| `SSL_VALIDATE` | Enable/disable certificate validation | cqlshrc `[ssl] validate` |
| `CQLSH_DEFAULT_CONNECT_TIMEOUT_SECONDS` | Default connect timeout | `--connect-timeout` |
| `CQLSH_DEFAULT_REQUEST_TIMEOUT_SECONDS` | Default request timeout | `--request-timeout` |
| `CQL_HISTORY` | Override history file path | |

## Precedence

Configuration values are resolved in this order (highest priority first):

1. CLI flags
2. Environment variables
3. cqlshrc file
4. Built-in defaults
