# CAPTURE

Capture query output to a file (tee-style — output is shown and saved).

## Syntax

```
CAPTURE '<filename>'
CAPTURE OFF
CAPTURE
```

## Usage

```
cqlsh> CAPTURE '/tmp/output.txt'
Now capturing query output to '/tmp/output.txt'.

cqlsh> SELECT * FROM system.local;
-- output shown AND written to file --

cqlsh> CAPTURE
Currently capturing to '/tmp/output.txt'.

cqlsh> CAPTURE OFF
Stopped capture. Output saved to '/tmp/output.txt'.
```

## Notes

- Disabled when `--no-file-io` is used.
- The file is created (or truncated) when CAPTURE is started.
- Tilde (`~`) in paths is expanded to the home directory.
