# Command Reference

Shell commands available in the cqlsh-rs interactive prompt. These are distinct from CQL statements — they control the shell itself.

## Quick reference

| Command | Description |
|---------|-------------|
| [CAPTURE](capture.md) | Capture output to a file |
| [CLEAR](clear.md) | Clear the terminal screen |
| [CONSISTENCY](consistency.md) | Get/set consistency level |
| [COPY TO](copy-to.md) | Export table data to CSV |
| [COPY FROM](copy-from.md) | Import CSV data into a table |
| [DEBUG](debug.md) | Toggle debug mode |
| [DESCRIBE](describe.md) | Schema introspection |
| [EXIT / QUIT](exit.md) | Exit the shell |
| [EXPAND](expand.md) | Toggle expanded (vertical) output |
| [HELP](help.md) | Show help |
| [LOGIN](login.md) | Re-authenticate with new credentials |
| [PAGING](paging.md) | Configure automatic paging |
| [SERIAL CONSISTENCY](serial-consistency.md) | Get/set serial consistency level |
| [SHOW](show.md) | Show version, host, or session trace info |
| [SOURCE](source.md) | Execute CQL from a file |
| [TRACING](tracing.md) | Toggle request tracing |
| [UNICODE](unicode.md) | Show Unicode/encoding info |
| [USE](use.md) | Switch keyspace |

## CQL statements

All standard CQL statements (SELECT, INSERT, UPDATE, DELETE, CREATE, ALTER, DROP, etc.) are sent directly to the database for execution. Terminate statements with a semicolon (`;`).
