# EXPAND

Toggle expanded (vertical) output mode. In expanded mode, each row is displayed as a vertical list of column-value pairs instead of a horizontal table.

## Syntax

```
EXPAND
EXPAND ON
EXPAND OFF
```

## Usage

```
cqlsh> EXPAND ON
Now printing expanded output.

cqlsh> SELECT * FROM system.local;
@ Row 1
-----------+---------------------------
 key       | local
 ...

cqlsh> EXPAND OFF
Disabled expanded output.
```

## Notes

- Useful for tables with many columns that don't fit horizontally.
- Similar to `\x` in PostgreSQL's psql.
