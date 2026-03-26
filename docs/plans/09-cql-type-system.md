# Sub-Plan SP9: CQL Type System

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2
> **Status: COMPLETED** — All 18 implementation steps done, all 25 CQL types supported (2026-03-18). Collections, UDTs, tuples, frozen types all working.

## Objective

Implement complete CQL type system mapping from Cassandra wire format to display strings, covering all native types, collections, UDTs, tuples, and frozen types with formatting that exactly matches Python cqlsh.

---

## Research Phase

### Tasks

1. **CQL type catalog** — All native types, collection types, special types
2. **Python cqlsh type formatting** — Read `cqlshlib/formatting.py` and `cqlshlib/displaying.py`
3. **scylla crate type mapping** — How `scylla::frame::value::CqlValue` maps to CQL types
4. **Edge cases** — Empty collections, nested collections, null values in collections, large blobs

### Research Deliverables

- [x] Complete type mapping table: CQL type -> scylla Rust type -> display string
- [x] Formatting rules for each type (precision, quoting, escaping)
- [x] Collection formatting rules (delimiters, nesting)
- [x] UDT formatting rules (field order, naming)
- [x] Edge case catalog

---

## Execution Phase

### Type Implementation Matrix

| CQL Type | Rust Type | Display Format | Complexity |
|----------|-----------|---------------|------------|
| `ascii` | `String` | `'text'` | Low |
| `bigint` | `i64` | `12345` | Low |
| `blob` | `Vec<u8>` | `0x0123abcd` | Low |
| `boolean` | `bool` | `True` / `False` | Low |
| `counter` | `i64` | `12345` | Low |
| `date` | `chrono::NaiveDate` | `2024-01-15` | Medium |
| `decimal` | `BigDecimal` | `3.14159` | Medium |
| `double` | `f64` | `3.141592653590` (configurable precision) | Medium |
| `duration` | Custom | `1h30m` | Medium |
| `float` | `f32` | `3.14159` (configurable precision) | Medium |
| `inet` | `IpAddr` | `192.168.1.1` / `::1` | Low |
| `int` | `i32` | `12345` | Low |
| `smallint` | `i16` | `123` | Low |
| `text` | `String` | `'text value'` | Low |
| `time` | `chrono::NaiveTime` | `13:30:54.234` | Medium |
| `timestamp` | `chrono::DateTime` | `2024-01-15 13:30:54.234+0000` (configurable) | High |
| `timeuuid` | `Uuid` | `550e8400-e29b-41d4-a716-446655440000` | Low |
| `tinyint` | `i8` | `12` | Low |
| `uuid` | `Uuid` | `550e8400-e29b-41d4-a716-446655440000` | Low |
| `varchar` | `String` | `'text value'` | Low |
| `varint` | `BigInt` | `123456789012345678901234567890` | Medium |
| `list<T>` | `Vec<CqlValue>` | `['a', 'b', 'c']` | Medium |
| `set<T>` | `BTreeSet<CqlValue>` | `{'a', 'b', 'c'}` | Medium |
| `map<K,V>` | `BTreeMap<CqlValue,CqlValue>` | `{'key1': 'val1', 'key2': 'val2'}` | Medium |
| `tuple<T...>` | `Vec<CqlValue>` | `(1, 'text', 3.14)` | Medium |
| `frozen<T>` | Same as T | Same as T | Low |
| User-Defined Type | `HashMap<String,CqlValue>` | `{field1: val1, field2: val2}` | High |

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | `CqlValue` enum with all type variants | `types.rs` | Unit: construction |
| 2 | `Display` impl for simple types (int, bigint, text, etc.) | `types.rs` | Unit: each type |
| 3 | Boolean formatting (`True`/`False` not `true`/`false`) | `types.rs` | Unit: bool format |
| 4 | Blob hex formatting (`0x` prefix) | `types.rs` | Unit: blob format |
| 5 | Timestamp formatting with configurable `datetimeformat` | `types.rs` | Unit: timestamp formats |
| 6 | Timezone-aware timestamp display | `types.rs` | Unit: timezone handling |
| 7 | Float/double with configurable precision | `types.rs` | Unit: precision control |
| 8 | Duration formatting | `types.rs` | Unit: duration display |
| 9 | List formatting with `[...]` delimiters | `types.rs` | Unit: list format |
| 10 | Set formatting with `{...}` delimiters (sorted) | `types.rs` | Unit: set format |
| 11 | Map formatting with `{k: v}` style | `types.rs` | Unit: map format |
| 12 | Nested collection formatting | `types.rs` | Unit: nested collections |
| 13 | Tuple formatting with `(...)` | `types.rs` | Unit: tuple format |
| 14 | UDT formatting with field names | `types.rs` | Unit: UDT format |
| 15 | NULL value formatting | `types.rs` | Unit: null display |
| 16 | Empty collection formatting | `types.rs` | Unit: empty collections |
| 17 | scylla `CqlValue` -> our `CqlValue` conversion | `types.rs` | Unit: conversion tests |
| 18 | Type name display (for DESCRIBE, etc.) | `types.rs` | Unit: type names |

### Acceptance Criteria

- [x] Every CQL type formats identically to Python cqlsh
- [x] Configurable timestamp format works (datetimeformat)
- [x] Configurable float/double precision works
- [x] Nested collections display correctly to arbitrary depth
- [x] UDTs show field names in correct order
- [x] NULL values display as configured
- [x] Empty collections display correctly
- [x] Frozen types display the same as their unfrozen counterparts

---

## Skills Required

- Rust type system and enums (S1)
- CQL type system (D1, D2)
- `chrono` crate for date/time (S1)
- `num-bigint` and `bigdecimal` for arbitrary precision (S1)
- `scylla` crate type mapping (C1)
