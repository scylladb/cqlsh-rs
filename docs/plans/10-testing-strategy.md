# Sub-Plan SP10: Testing Strategy

> Parent: [high-level-design.md](high-level-design.md) | Phase: 5 (ongoing from Phase 1)

## Objective

Design and implement a comprehensive testing strategy that validates 100% compatibility with Python cqlsh, covers all code paths, and runs efficiently in CI across multiple Cassandra/ScyllaDB versions and platforms.

---

## Research Phase

### Tasks

1. **Python cqlsh test suite** — How the original is tested (dtests, pytest, ccm)
2. **Cassandra testing tools** — CCM, testcontainers, docker-compose approaches
3. **Rust testing ecosystem** — assert_cmd, insta, proptest, testcontainers-rs, trycmd
4. **CI strategies** — Matrix testing across DB versions and platforms
5. **Coverage tools** — cargo-tarpaulin vs llvm-cov, grcov

### Research Deliverables

- [ ] Testing tool comparison matrix
- [ ] CI configuration design (matrix, caching, parallelism)
- [ ] Test data generation strategy
- [ ] Coverage reporting setup

---

## Execution Phase

### Test Layers

#### Layer 1: Unit Tests

**Location:** Inline in `src/` modules (`#[cfg(test)]` modules)

| Area | Test Focus | Count (est.) |
|------|-----------|-------------|
| Config parsing | CLI flags, cqlshrc sections, env vars, precedence | 50+ |
| Statement parser | Semicolons, comments, strings, multi-line, routing | 40+ |
| Type formatting | Each CQL type, precision, null, collections, UDTs | 60+ |
| Tab completion | Each context, keyword matching, schema matching | 50+ |
| Command parsing | Each command's argument parsing | 30+ |
| Color/highlighting | Token colorization | 15+ |

#### Layer 2: Integration Tests

**Location:** `tests/integration/`
**Infrastructure:** `testcontainers-rs` with Cassandra/ScyllaDB Docker images

| Test Suite | Description | DB Required |
|-----------|-------------|------------|
| `test_connect.rs` | Connection, auth, SSL, timeouts | Yes |
| `test_queries.rs` | SELECT, INSERT, UPDATE, DELETE, all types | Yes |
| `test_describe.rs` | All DESCRIBE sub-commands, DDL output | Yes |
| `test_copy.rs` | COPY TO/FROM with all options | Yes |
| `test_commands.rs` | CONSISTENCY, TRACING, PAGING, EXPAND, etc. | Yes |
| `test_completion.rs` | Tab completion against live schema | Yes |

#### Layer 3: CLI Tests

**Location:** `tests/cli/`
**Tool:** `assert_cmd` + `predicates`

| Test Suite | Description | DB Required |
|-----------|-------------|------------|
| `test_flags.rs` | All CLI flags accepted, validation errors | No (some) |
| `test_execute.rs` | `-e` flag execution and output | Yes |
| `test_file.rs` | `-f` flag file execution | Yes |
| `test_exit_codes.rs` | Exit codes for success, error, syntax error | Yes |
| `test_version.rs` | `--version` output format | No |
| `test_help.rs` | `--help` output format | No |

#### Layer 4: Snapshot Tests

**Location:** Inline or `tests/snapshots/`
**Tool:** `insta`

| Snapshot Area | Description |
|--------------|-------------|
| Table output | Exact tabular format for various result sets |
| JSON output | JSON formatting for all types |
| CSV output | CSV formatting |
| Expanded output | Vertical format |
| DESCRIBE output | DDL for various schema objects |
| HELP output | Help text for each command |
| Error output | Error messages |

#### Layer 5: Property-Based Tests

**Location:** `tests/proptest/`
**Tool:** `proptest`

| Property | Description |
|----------|-------------|
| Parser roundtrip | Any valid CQL statement is correctly delimited |
| Type format/parse | Format -> parse -> format is idempotent |
| CSV roundtrip | COPY TO -> COPY FROM preserves data |
| Config merge | Precedence rules are consistent |

#### Layer 6: Compatibility Tests

**Location:** `tests/compat/`
**Tool:** Custom harness

```
For each test case:
  1. Start Cassandra container
  2. Set up test schema
  3. Run command in Python cqlsh, capture output
  4. Run same command in cqlsh-rs, capture output
  5. Diff outputs (ignoring timing, UUIDs, etc.)
  6. Assert match (or document divergence)
```

| Test Category | Description |
|--------------|-------------|
| CLI flag behavior | Each flag produces same effect |
| Output formatting | Same table/JSON/CSV format |
| Type display | Each CQL type formats the same |
| Command output | Each command produces same output |
| Error messages | Same error text and codes |
| Tab completion | Same completions in same contexts |
| cqlshrc handling | Same config applied |

### CI Configuration

```yaml
# .github/workflows/ci.yml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
    db:
      - cassandra:3.11
      - cassandra:4.0
      - cassandra:4.1
      - cassandra:5.0
      - scylladb/scylla:5.4
      - scylladb/scylla:6.0
    exclude:
      # Windows doesn't run Docker easily
      - os: windows-latest
        db: cassandra:3.11
```

### Coverage Target

| Module | Target Coverage |
|--------|----------------|
| `config.rs` | >95% |
| `parser.rs` | >95% |
| `types.rs` | >95% |
| `formatter.rs` | >90% |
| `completer.rs` | >90% |
| `commands/*` | >85% |
| `driver/*` | >80% |
| `repl.rs` | >70% (interactive parts harder to test) |
| **Overall** | **>90%** |

### Acceptance Criteria

- [ ] Unit tests cover all public functions
- [ ] Integration tests pass on all DB versions in the matrix
- [ ] CLI tests verify all flags and exit codes
- [ ] Snapshot tests lock down output formats
- [ ] Property tests find no parser/formatter inconsistencies
- [ ] Compatibility tests show <1% output divergence from Python cqlsh
- [ ] Coverage is >90%
- [ ] CI runs in <15 minutes (with parallelism)

### Estimated Effort

- Research: 2 days
- Unit test infrastructure: 2 days
- Integration test harness: 3 days
- Compatibility test harness: 3 days
- CI configuration: 2 days
- Ongoing test writing: Throughout development
- **Total: 12 days (infrastructure) + ongoing**

---

## Skills Required

- Rust testing ecosystem (S9)
- `testcontainers-rs` (C9)
- `assert_cmd` + `predicates` (C11)
- `insta` snapshot testing (C12)
- `proptest` property testing (C10)
- CI/CD with GitHub Actions (S11)
- `cargo-tarpaulin` / `llvm-cov` (S9)
