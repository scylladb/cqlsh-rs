# CONSISTENCY

Get or set the consistency level for subsequent CQL statements.

## Syntax

```
CONSISTENCY
CONSISTENCY <level>
```

## Levels

`ANY`, `ONE`, `TWO`, `THREE`, `QUORUM`, `ALL`, `LOCAL_QUORUM`, `EACH_QUORUM`, `LOCAL_ONE`

## Usage

```
cqlsh> CONSISTENCY
Current consistency level is ONE.

cqlsh> CONSISTENCY QUORUM
Consistency level set to QUORUM.
```

## Notes

- The default consistency level is `ONE`.
- Can be set at startup with `--consistency-level`.
- For lightweight transactions, see [SERIAL CONSISTENCY](serial-consistency.md).
