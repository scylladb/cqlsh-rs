# LOGIN

Re-authenticate with new credentials without restarting the shell.

## Syntax

```
LOGIN <username> [<password>]
```

If the password is omitted, you will be prompted to enter it.

## Usage

```
cqlsh> LOGIN admin
Password: ****
Login successful.

cqlsh> LOGIN admin secretpass
Login successful.
```

## Notes

- Creates a new connection to the cluster with the provided credentials.
- The current session state (keyspace, consistency level, etc.) may be reset.
