# SERIAL CONSISTENCY

Get or set the serial consistency level for lightweight transactions (IF NOT EXISTS, IF conditions).

## Syntax

```
SERIAL CONSISTENCY
SERIAL CONSISTENCY <level>
```

## Levels

`SERIAL`, `LOCAL_SERIAL`

## Usage

```
cqlsh> SERIAL CONSISTENCY
Current serial consistency level is SERIAL.

cqlsh> SERIAL CONSISTENCY LOCAL_SERIAL
Serial consistency level set to LOCAL_SERIAL.
```

## Notes

- The default serial consistency level is `SERIAL`.
- Can be set at startup with `--serial-consistency-level`.
- Only applies to statements using lightweight transactions.
