# Phase 4.1–4.2 Manual Test Plan

## Prerequisites

- A running Cassandra or ScyllaDB instance on localhost:9042
- Build: `cargo run --`

---

## 4.1: Non-Interactive Mode

### Test 1: `-e` single statement
```bash
cargo run -- -e "SELECT key, cluster_name FROM system.local;"
```
- Should print banner, then tabular result, then exit
- Exit code: `echo $?` → `0`

### Test 2: `-e` multiple statements
```bash
cargo run -- -e "SELECT key FROM system.local; SELECT release_version FROM system.local;"
```
- Should print two result sets sequentially

### Test 3: `-e` with error
```bash
cargo run -- -e "SELECT * FROM nonexistent_table;"
echo $?
```
- Should print error to stderr
- Exit code → `1`

### Test 4: `-e` with color
```bash
cargo run -- -C -e "SELECT * FROM system.local;"
```
- Output should be colored (headers magenta, values by type)

### Test 5: `-e` no color when piped
```bash
cargo run -- -e "SELECT key FROM system.local;" | cat
```
- No ANSI codes in output

### Test 6: `-f` file execution
```bash
echo "SELECT key FROM system.local;" > /tmp/test.cql
cargo run -- -f /tmp/test.cql
echo $?
```
- Should print result and exit with `0`

### Test 7: `-f` multi-statement file
```bash
cat > /tmp/multi.cql << 'EOF'
CONSISTENCY;
SELECT key FROM system.local;
SHOW VERSION
EOF
cargo run -- -f /tmp/multi.cql
```
- Should show consistency level, query result, and version

### Test 8: `-f` with error in file
```bash
echo "INVALID CQL HERE;" > /tmp/bad.cql
cargo run -- -f /tmp/bad.cql
echo $?
```
- Error message on stderr, exit code `1`

---

## 4.1: Shell Commands

### Test 9: DEBUG command
```
cqlsh> DEBUG
cqlsh> DEBUG ON
cqlsh> SELECT * FROM system.local;
cqlsh> DEBUG OFF
```
- `DEBUG` shows current status
- `DEBUG ON` enables debug output
- Query with debug should show additional info
- `DEBUG OFF` disables it

### Test 10: UNICODE command
```
cqlsh> UNICODE
```
- Should print encoding info (e.g., `Encoding: utf-8`)

### Test 11: LOGIN command
```
cqlsh> LOGIN cassandra cassandra
```
- Should reconnect with new credentials
- If auth is not configured, should show connection error

### Test 12: LOGIN without password
```
cqlsh> LOGIN cassandra
```
- Should prompt for password

---

## 4.2: DESCRIBE Extensions

### Test 13: DESCRIBE FULL SCHEMA
```
cqlsh> DESCRIBE FULL SCHEMA
```
- Should output DDL for ALL keyspaces including `system`, `system_schema`, etc.
- Much longer output than `DESCRIBE SCHEMA`

### Test 14: DESCRIBE TYPES
```
cqlsh> USE system;
cqlsh:system> DESCRIBE TYPES
```
- Should list UDT names (may be empty for system keyspace)

### Test 15: DESCRIBE TYPE (if UDTs exist)
```sql
CREATE KEYSPACE IF NOT EXISTS test_ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
USE test_ks;
CREATE TYPE IF NOT EXISTS address (street text, city text, zip int);
DESCRIBE TYPE address
```
- Should show `CREATE TYPE test_ks.address (street text, city text, zip int);`

### Test 16: DESCRIBE FUNCTIONS / DESCRIBE AGGREGATES
```
cqlsh> USE system;
cqlsh:system> DESCRIBE FUNCTIONS
cqlsh:system> DESCRIBE AGGREGATES
```
- Should list function/aggregate names (may be empty)

### Test 17: DESCRIBE INDEX
```sql
CREATE KEYSPACE IF NOT EXISTS test_ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
USE test_ks;
CREATE TABLE IF NOT EXISTS users (id int PRIMARY KEY, name text, email text);
CREATE INDEX IF NOT EXISTS users_email_idx ON users (email);
DESCRIBE INDEX users_email_idx
```
- Should show `CREATE INDEX users_email_idx ON test_ks.users (email);`

### Test 18: DESCRIBE MATERIALIZED VIEW
```sql
USE test_ks;
CREATE MATERIALIZED VIEW IF NOT EXISTS users_by_name AS
  SELECT * FROM users WHERE name IS NOT NULL AND id IS NOT NULL
  PRIMARY KEY (name, id);
DESCRIBE MATERIALIZED VIEW users_by_name
```
- Should show the full CREATE MATERIALIZED VIEW DDL

### Test 19: Qualified names
```
cqlsh> DESCRIBE INDEX test_ks.users_email_idx
cqlsh> DESCRIBE TYPE test_ks.address
```
- Should work without USE keyspace

### Test 20: Non-existent objects
```
cqlsh> DESCRIBE INDEX nonexistent_idx
cqlsh> DESCRIBE TYPE nonexistent_type
```
- Should print error message, not crash

---

## Cleanup

```sql
DROP MATERIALIZED VIEW IF EXISTS test_ks.users_by_name;
DROP TABLE IF EXISTS test_ks.users;
DROP TYPE IF EXISTS test_ks.address;
DROP KEYSPACE IF EXISTS test_ks;
```
