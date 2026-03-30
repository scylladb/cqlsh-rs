# USE

Switch the current keyspace. This is a CQL statement handled by the database, but cqlsh-rs also updates the prompt and tab completion context.

## Syntax

```sql
USE <keyspace>;
```

## Usage

```
cqlsh> USE my_keyspace;
cqlsh:my_keyspace>
```

## Notes

- The prompt updates to show the current keyspace.
- Tab completion for table names will use the current keyspace context.
- Can also be set at startup with `-k` / `--keyspace`.
