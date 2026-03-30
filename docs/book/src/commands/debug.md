# DEBUG

Toggle debug output mode.

## Syntax

```
DEBUG
DEBUG ON
DEBUG OFF
```

## Usage

```
cqlsh> DEBUG
Debug output is currently disabled. Use DEBUG ON to enable.

cqlsh> DEBUG ON
Now printing debug output.
```

## Notes

- Can also be enabled at startup with `--debug`.
- When enabled, additional diagnostic information is printed for each query.
