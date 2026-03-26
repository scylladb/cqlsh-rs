# Phase 4.3–4.4 COPY TO/FROM Manual Test Plan

## Prerequisites

```sql
-- Setup test data
CREATE KEYSPACE IF NOT EXISTS test_ks WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};
USE test_ks;

CREATE TABLE IF NOT EXISTS users (
    id int PRIMARY KEY,
    name text,
    email text,
    age int,
    active boolean
);

INSERT INTO users (id, name, email, age, active) VALUES (1, 'Alice', 'alice@example.com', 30, true);
INSERT INTO users (id, name, email, age, active) VALUES (2, 'Bob', 'bob@example.com', 25, false);
INSERT INTO users (id, name, email, age, active) VALUES (3, 'Charlie', null, 40, true);
```

---

## COPY TO Tests

### Test 1: Basic COPY TO file
```
COPY test_ks.users TO '/tmp/users.csv';
```
- Should create `/tmp/users.csv`
- Verify: `cat /tmp/users.csv` shows CSV data
- Should print "N rows exported"

### Test 2: COPY TO with HEADER
```
COPY test_ks.users TO '/tmp/users_header.csv' WITH HEADER=true;
```
- First line should be column names: `id,name,email,age,active`

### Test 3: COPY TO with custom DELIMITER
```
COPY test_ks.users TO '/tmp/users_pipe.csv' WITH DELIMITER='|';
```
- Fields separated by `|` instead of `,`

### Test 4: COPY TO specific columns
```
COPY test_ks.users (id, name) TO '/tmp/users_partial.csv' WITH HEADER=true;
```
- Only `id` and `name` columns exported

### Test 5: COPY TO with NULL value
```
COPY test_ks.users TO '/tmp/users_null.csv' WITH NULL='N/A' AND HEADER=true;
```
- Charlie's null email should show as `N/A`

### Test 6: COPY TO STDOUT
```
COPY test_ks.users TO STDOUT WITH HEADER=true;
```
- CSV printed directly to terminal

### Test 7: COPY TO with BOOLSTYLE
```
COPY test_ks.users TO '/tmp/users_bool.csv' WITH BOOLSTYLE='yes,no' AND HEADER=true;
```
- Boolean values should be `yes`/`no` instead of `True`/`False`

### Test 8: COPY TO with MAXOUTPUTSIZE
```
COPY test_ks.users TO '/tmp/users_limit.csv' WITH MAXOUTPUTSIZE=1;
```
- Only 1 row exported

### Test 9: COPY TO with --no-file-io
```bash
cargo run -- --no-file-io
cqlsh> COPY test_ks.users TO '/tmp/blocked.csv';
```
- Should print "File I/O is disabled"

---

## COPY FROM Tests

### Test 10: Basic COPY FROM
```bash
cat > /tmp/import.csv << 'EOF'
10,Dave,dave@example.com,35,True
11,Eve,eve@example.com,28,False
EOF
```
```
COPY test_ks.users FROM '/tmp/import.csv';
```
- Verify: `SELECT * FROM test_ks.users WHERE id IN (10, 11);` shows imported rows

### Test 11: COPY FROM with HEADER
```bash
cat > /tmp/import_header.csv << 'EOF'
id,name,email,age,active
20,Frank,frank@example.com,50,True
EOF
```
```
COPY test_ks.users FROM '/tmp/import_header.csv' WITH HEADER=true;
SELECT * FROM test_ks.users WHERE id = 20;
```
- Should import Frank, skipping header row

### Test 12: COPY FROM with NULL handling
```bash
cat > /tmp/import_null.csv << 'EOF'
30,Grace,,45,True
EOF
```
```
COPY test_ks.users FROM '/tmp/import_null.csv';
SELECT * FROM test_ks.users WHERE id = 30;
```
- Grace's email should be null

### Test 13: COPY FROM with custom DELIMITER
Note: without HEADER, CSV columns must match CREATE TABLE order or specify columns explicitly.
```bash
cat > /tmp/import_pipe.csv << 'EOF'
40|Heidi|heidi@example.com|33|True
EOF
```
```
COPY test_ks.users (id, name, email, age, active) FROM '/tmp/import_pipe.csv' WITH DELIMITER='|';
SELECT * FROM test_ks.users WHERE id = 40;
```

### Test 14: COPY FROM with TTL
```
COPY test_ks.users FROM '/tmp/import.csv' WITH TTL=60;
```
- Rows should expire after 60 seconds
- Verify: `SELECT TTL(name) FROM test_ks.users WHERE id = 10;` shows TTL value

### Test 15: COPY FROM with specific columns
```bash
cat > /tmp/import_cols.csv << 'EOF'
50,Ivan
EOF
```
```
COPY test_ks.users (id, name) FROM '/tmp/import_cols.csv';
SELECT * FROM test_ks.users WHERE id = 50;
```
- Should import only id and name, other columns null

### Test 16: COPY FROM with parse errors
```bash
cat > /tmp/import_bad.csv << 'EOF'
60,Jane,jane@example.com,30,True
NOT_A_NUMBER,Bad,bad@example.com,x,y
70,Kate,kate@example.com,25,False
EOF
```
```
COPY test_ks.users FROM '/tmp/import_bad.csv' WITH MAXPARSEERRORS=1;
```
- Should import rows 60 and 70, skip bad row
- Should print error count

### Test 17: COPY FROM STDIN
```bash
echo "80,Leo,leo@example.com,22,True" | cargo run -- -e "COPY test_ks.users FROM STDIN;"
```
- Note: this may not work in `-e` mode — test interactively if needed

### Test 18: COPY FROM with ERRFILE
```
COPY test_ks.users FROM '/tmp/import_bad.csv' WITH ERRFILE='/tmp/errors.csv';
```
- Failed rows should be written to `/tmp/errors.csv`
- Verify: `cat /tmp/errors.csv`

---

## Round-Trip Test

### Test 19: Export then import
```sql
CREATE TABLE IF NOT EXISTS test_ks.roundtrip (id int PRIMARY KEY, val text);
INSERT INTO test_ks.roundtrip (id, val) VALUES (1, 'hello');
INSERT INTO test_ks.roundtrip (id, val) VALUES (2, 'world');
```
```
COPY test_ks.roundtrip TO '/tmp/roundtrip.csv' WITH HEADER=true;
TRUNCATE test_ks.roundtrip;
COPY test_ks.roundtrip FROM '/tmp/roundtrip.csv' WITH HEADER=true;
SELECT * FROM test_ks.roundtrip;
```
- Data should match original

---

## Data Type Tests

### Test 20: Various CQL types
```sql
CREATE TABLE IF NOT EXISTS test_ks.types_test (
    id int PRIMARY KEY,
    t text,
    b boolean,
    f float,
    d double,
    ts timestamp,
    u uuid,
    bl blob,
    i inet
);

INSERT INTO test_ks.types_test (id, t, b, f, d, ts, u, bl, i)
VALUES (1, 'hello', true, 3.14, 2.718281828, '2024-01-01 00:00:00+0000',
        550e8400-e29b-41d4-a716-446655440000, 0xdeadbeef, '127.0.0.1');
```
```
COPY test_ks.types_test TO '/tmp/types.csv' WITH HEADER=true;
```
- Verify each type is formatted correctly in the CSV

---

## Cleanup
```sql
DROP TABLE IF EXISTS test_ks.roundtrip;
DROP TABLE IF EXISTS test_ks.types_test;
DROP TABLE IF EXISTS test_ks.users;
DROP KEYSPACE IF EXISTS test_ks;
```
