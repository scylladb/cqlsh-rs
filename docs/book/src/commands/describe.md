# DESCRIBE

Schema introspection commands. `DESC` is accepted as a shorthand.

## Syntax

```
DESCRIBE CLUSTER
DESCRIBE KEYSPACES
DESCRIBE KEYSPACE [name]
DESCRIBE TABLES
DESCRIBE TABLE <name>
DESCRIBE SCHEMA
DESCRIBE FULL SCHEMA
DESCRIBE INDEX <name>
DESCRIBE MATERIALIZED VIEW <name>
DESCRIBE TYPES
DESCRIBE TYPE <name>
DESCRIBE FUNCTIONS
DESCRIBE FUNCTION <name>
DESCRIBE AGGREGATES
DESCRIBE AGGREGATE <name>
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `CLUSTER` | Show cluster name and partitioner |
| `KEYSPACES` | List all keyspaces |
| `KEYSPACE [name]` | Show CREATE KEYSPACE statement (current keyspace if no name) |
| `TABLES` | List all tables in the current keyspace |
| `TABLE <name>` | Show CREATE TABLE statement |
| `SCHEMA` | Show CREATE statements for all user keyspaces |
| `FULL SCHEMA` | Show CREATE statements for all keyspaces (including system) |
| `INDEX <name>` | Show CREATE INDEX statement |
| `MATERIALIZED VIEW <name>` | Show CREATE MATERIALIZED VIEW statement |
| `TYPES` | List all user-defined types |
| `TYPE <name>` | Show CREATE TYPE statement |
| `FUNCTIONS` | List all user-defined functions |
| `FUNCTION <name>` | Show CREATE FUNCTION statement |
| `AGGREGATES` | List all user-defined aggregates |
| `AGGREGATE <name>` | Show CREATE AGGREGATE statement |

## Examples

```
cqlsh> DESCRIBE KEYSPACES;
system    system_auth    system_distributed    my_app

cqlsh> DESC TABLE my_app.users;
CREATE TABLE my_app.users (
    id uuid PRIMARY KEY,
    name text,
    email text
) WITH ...
```
