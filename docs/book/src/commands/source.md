# SOURCE

Execute CQL statements from a file.

## Syntax

```
SOURCE '<filename>'
```

## Usage

```
cqlsh> SOURCE '/tmp/schema.cql'
```

## Notes

- Each statement in the file must be terminated with a semicolon (`;`).
- Disabled when `--no-file-io` is used.
- Tilde (`~`) in paths is expanded to the home directory.
- Similar to using `-f` on the command line, but can be run interactively.
