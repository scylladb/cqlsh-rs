# PAGING

Configure automatic output paging through the built-in pager.

## Syntax

```
PAGING
PAGING ON
PAGING OFF
PAGING <page_size>
```

## Usage

```
cqlsh> PAGING
Query paging is currently enabled. Use PAGING OFF to disable.

cqlsh> PAGING OFF
Disabled paging.

cqlsh> PAGING ON
Now query paging is enabled.

cqlsh> PAGING 100
Now query paging is enabled.
```

## Notes

- When enabled, large result sets are automatically piped through the pager.
- Paging is automatically disabled when output is not a TTY.
- `PAGING <N>` enables paging (the page size value is accepted for compatibility with Python cqlsh).
