# Migration Guide

Step-by-step guide for migrating from Python `cqlsh` to `cqlsh-rs`.

## Overview

cqlsh-rs is designed as a drop-in replacement for Python cqlsh. The same CLI flags, configuration files, and shell commands work in both versions.

## Step 1: Install cqlsh-rs

See [Installation](./installation.md) for your platform.

## Step 2: Verify your cqlshrc

cqlsh-rs reads the same `~/.cassandra/cqlshrc` file. No changes needed.

```bash
# Test with your existing config
cqlsh-rs --cqlshrc ~/.cassandra/cqlshrc
```

## Step 3: Test your connection

```bash
# Same flags as Python cqlsh
cqlsh-rs 10.0.0.1 9042 -u cassandra -p cassandra --ssl
```

## Step 4: Test your scripts

If you use `cqlsh` in scripts with `-e` or `-f`, test them:

```bash
# These should work identically
cqlsh-rs -e "SELECT * FROM system.local"
cqlsh-rs -f schema.cql
```

## Step 5: Create an alias (optional)

Once confident, alias `cqlsh` to `cqlsh-rs`:

```bash
# In ~/.bashrc or ~/.zshrc
alias cqlsh='cqlsh-rs'
```

## What stays the same

- All CLI flags (`-e`, `-f`, `-u`, `-p`, `-k`, `--ssl`, etc.)
- The `~/.cassandra/cqlshrc` config file format and all sections
- Shell commands (DESCRIBE, COPY, CONSISTENCY, EXPAND, TRACING, etc.)
- The prompt format (`username@cqlsh:keyspace>`)
- History file location (`~/.cassandra/cql_history`)
- Environment variables (`CQLSH_HOST`, `CQLSH_PORT`, `SSL_CERTFILE`, etc.)
- Multi-line input with `;` termination
- `Ctrl-C` to cancel, `Ctrl-D` to exit

## What's different

See [Known Divergences](./divergences.md) for a complete list. Key differences:

| Area | Python cqlsh | cqlsh-rs |
|------|-------------|----------|
| **Startup time** | ~1-2 seconds | ~10-50 milliseconds |
| **Binary** | Requires Python + pip | Single static binary |
| **Driver** | cassandra-driver (Python) | scylla-rust-driver |
| **COPY parallelism** | Multiprocessing | Async (tokio) |
| **Syntax highlighting** | Basic | Token-aware with CQL grammar |

## SSL/TLS migration

Python cqlsh uses Python's `ssl` module. cqlsh-rs uses `rustls`. The cqlshrc SSL configuration is compatible:

```ini
[ssl]
certfile = /path/to/ca-cert.pem
validate = true
userkey = /path/to/client-key.pem
usercert = /path/to/client-cert.pem
```

Note: `rustls` does not support SSLv3 or TLSv1.0/1.1. If your cluster requires these, you'll need to upgrade your TLS configuration.

## COPY migration

COPY TO/FROM syntax and options are identical. Performance characteristics differ because cqlsh-rs uses async I/O instead of Python multiprocessing.

## Rollback

To switch back to Python cqlsh:

```bash
# Remove the alias
unalias cqlsh

# Or use the Python version explicitly
python -m cqlshlib.cqlsh
```
