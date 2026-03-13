# Sub-Plan SP13: Skills Development

> Parent: [high-level-design.md](high-level-design.md) | All Phases

## Objective

Define a structured skills development plan that ensures team members acquire all necessary technical skills and domain knowledge before and during each implementation phase. Each skill area includes learning resources, practice exercises, and validation criteria.

---

## Skills Inventory

### Skill Level Definitions

| Level | Definition |
|-------|-----------|
| **Basic** | Can read and understand code; can follow tutorials; needs guidance for non-trivial tasks |
| **Intermediate** | Can implement features independently; can debug issues; understands trade-offs |
| **Advanced** | Can design systems; can optimize performance; can mentor others; deep understanding |

### Required Skills Matrix

| # | Skill | Required Level | Phase Needed | Current Gap Assessment |
|---|-------|---------------|-------------|----------------------|
| S1 | Rust (core language) | Advanced | All | Assess per team member |
| S2 | Async Rust (Tokio) | Intermediate | 1+ | Assess per team member |
| S3 | CQL protocol & Cassandra | Intermediate | 1+ | Assess per team member |
| S4 | Terminal/TUI programming | Intermediate | 2+ | Likely gap for most |
| S5 | CLI design (clap) | Intermediate | 1 | Easy to learn |
| S6 | Parser design | Intermediate | 1-3 | May need training |
| S7 | CSV/data processing | Intermediate | 4 | Easy to learn |
| S8 | SSL/TLS | Basic | 2 | Moderate gap |
| S9 | Testing methodologies | Advanced | All | Varies |
| S10 | Performance profiling | Intermediate | 5 | Often a gap |
| S11 | CI/CD (GitHub Actions) | Intermediate | All | Usually known |
| S12 | Cross-compilation | Basic | 6 | Niche skill |

---

## Learning Plans by Phase

### Pre-Development: Foundations (2 weeks)

#### Track A: Rust Core

**Goal:** All team members at Advanced Rust level.

| Week | Topic | Resources | Exercise | Validation |
|------|-------|-----------|----------|------------|
| 1 | Ownership & borrowing review | The Rust Book ch. 4, 10, 15 | Implement a linked list, arena allocator | Code review |
| 1 | Error handling patterns | `anyhow` + `thiserror` docs | Design error type hierarchy | Implement `error.rs` |
| 1 | Traits & generics | The Rust Book ch. 10, 17 | Design driver trait | Code review |
| 2 | Async Rust | Tokio tutorial, async book | Async TCP client | Connect to Cassandra |
| 2 | Async patterns | `futures` crate, streams, channels | Concurrent task pipeline | Benchmark |

**Recommended Resources:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — Chapters 4, 10, 15, 17
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

#### Track B: Domain Knowledge

**Goal:** Deep understanding of CQL and Python cqlsh behavior.

| Week | Topic | Resources | Exercise | Validation |
|------|-------|-----------|----------|------------|
| 1 | CQL fundamentals | Apache Cassandra docs, CQL spec | Create schema, CRUD operations | Knowledge quiz |
| 1 | Cassandra data model | DataStax documentation | Design a data model | Review |
| 2 | Python cqlsh deep-dive | Read cqlsh source code | Document all commands and behaviors | Behavior catalog |
| 2 | Python cqlsh internals | Read cqlshlib/*.py | Understand formatting, completion, COPY | Technical notes |

**Recommended Resources:**
- [CQL Reference](https://cassandra.apache.org/doc/latest/cassandra/cql/)
- [Python cqlsh source](https://github.com/apache/cassandra/tree/trunk/bin/cqlsh.py)
- [cqlshlib source](https://github.com/apache/cassandra/tree/trunk/pylib/cqlshlib/)
- [DataStax CQL Guide](https://docs.datastax.com/en/cql-oss/)

---

### Phase 1: Crate Mastery — CLI & Driver (During Weeks 1-3)

| Skill | Learning Path | Practice Project | Validation |
|-------|-------------|-----------------|------------|
| **clap v4** | [clap docs](https://docs.rs/clap), derive tutorial | Build a CLI app with 20+ flags, validation, help | Full CLI parser passes tests |
| **scylla driver** | [scylla docs](https://docs.rs/scylla), examples dir | Connect, query, iterate results, handle errors | Integration test passes |
| **rust-ini** | Crate docs | Parse Python cqlshrc samples | Config tests pass |
| **anyhow/thiserror** | Crate docs | Error hierarchy with context | Error module done |

**Exercises:**
1. Build a minimal `scylla-cli` that connects and runs a query
2. Parse 5 different real-world cqlshrc files
3. Build a clap app that matches `cqlsh --help` output

---

### Phase 2: Crate Mastery — REPL & Formatting (During Weeks 4-7)

| Skill | Learning Path | Practice Project | Validation |
|-------|-------------|-----------------|------------|
| **rustyline** | [API docs](https://docs.rs/rustyline), examples | REPL with history, custom prompt | REPL module done |
| **rustyline Completer** | Completer trait docs | Static keyword completer | Keyword completion works |
| **comfy-table** | [Docs](https://docs.rs/comfy-table) | Format all CQL types in table | Output matches Python cqlsh |
| **owo-colors** | Crate docs | Colored CQL syntax | Highlighting works |
| **crossterm** | Crate docs | Terminal size, clearing | Terminal ops work |
| **rustls** | Docs + examples | TLS connection to test server | SSL tests pass |

**Exercises:**
1. Build a standalone REPL that highlights SQL keywords
2. Format a heterogeneous table with proper alignment
3. Establish TLS connection to a test Cassandra cluster

---

### Phase 3: Crate Mastery — Completion & Output (During Weeks 8-11)

| Skill | Learning Path | Practice Project | Validation |
|-------|-------------|-----------------|------------|
| **CQL grammar** | CQL spec, ANTLR grammar file | Tokenizer for CQL | Context detection works |
| **Schema introspection** | system_schema.* table docs | Query and cache schema | Schema cache works |
| **serde_json** | Crate docs | Serialize CQL values to JSON | JSON output matches |
| **chrono** | Crate docs, strftime format | Timestamp formatting | Timestamp tests pass |

**Exercises:**
1. Write a CQL tokenizer that identifies statement context
2. Build a schema browser that queries system_schema tables
3. Format various timestamps with configurable patterns

---

### Phase 4: Crate Mastery — COPY & Advanced (During Weeks 12-16)

| Skill | Learning Path | Practice Project | Validation |
|-------|-------------|-----------------|------------|
| **csv crate** | [Docs](https://docs.rs/csv) | CSV reader/writer with all options | CSV roundtrip works |
| **Tokio channels** | Tokio docs | Producer-consumer pipeline | Parallel pipeline works |
| **Rate limiting** | `governor` or manual token bucket | Rate-limited inserter | Ingest rate matches config |
| **Concurrent patterns** | `futures::stream`, `buffer_unordered` | Concurrent page fetching | COPY TO parallelism works |

**Exercises:**
1. Build a CSV import tool with parallel batch inserts
2. Implement a token bucket rate limiter
3. Build a concurrent data exporter with configurable parallelism

---

### Phase 5: Quality Mastery — Testing & Benchmarking (During Weeks 17-20)

| Skill | Learning Path | Practice Project | Validation |
|-------|-------------|-----------------|------------|
| **testcontainers-rs** | [Docs](https://docs.rs/testcontainers) | Cassandra integration test | Tests run with containers |
| **assert_cmd** | [Docs](https://docs.rs/assert_cmd) | CLI binary tests | CLI tests pass |
| **insta** | [Docs](https://docs.rs/insta) | Snapshot tests for output | Snapshots created |
| **proptest** | [Book](https://altsysrq.github.io/proptest-book/) | Property tests for parser | Properties hold |
| **criterion** | [User guide](https://bheisler.github.io/criterion.rs/book/) | Benchmark suite | Benchmarks run |
| **cargo-tarpaulin** | Tool docs | Coverage report | >90% coverage |
| **flamegraph** | `cargo flamegraph` | Profile a hot path | Flamegraph generated |

**Exercises:**
1. Write integration tests that spin up a Cassandra container
2. Create snapshot tests for 10 different output formats
3. Write property tests that fuzz the parser with random CQL
4. Set up criterion benchmarks with HTML reports

---

## Skill Development Infrastructure

### Knowledge Base

Maintain a `docs/knowledge/` directory with:

| Document | Content |
|----------|---------|
| `cql-cheatsheet.md` | Quick reference for all CQL statements |
| `python-cqlsh-behaviors.md` | Documented behaviors from reading Python source |
| `crate-patterns.md` | Patterns and idioms for each key crate |
| `debugging-guide.md` | How to debug common issues |
| `architecture-decisions.md` | ADRs for key decisions |

### Pair Programming Schedule

| Phase | Focus | Pairs |
|-------|-------|-------|
| 1 | Driver + Config | Track A + Track B |
| 2 | REPL + Formatter | Track B + Track A |
| 3 | Completer + Colorizer | Track A + Track B |
| 4 | COPY + Commands | All tracks |
| 5 | Testing + Benchmarks | Track A + Track C |

### Code Review Focus Areas

| Phase | Review Focus |
|-------|-------------|
| 1 | Error handling, trait design, async correctness |
| 2 | User experience, output accuracy, edge cases |
| 3 | Performance, cache correctness, completion accuracy |
| 4 | Parallelism safety, error handling, data integrity |
| 5 | Test quality, coverage gaps, benchmark methodology |

---

## Validation & Assessment

### Skill Checkpoints

| Checkpoint | When | Assessment Method |
|-----------|------|------------------|
| Rust proficiency | Pre-dev | Coding challenge + code review |
| Async understanding | Week 1 | Implement working async driver |
| CQL knowledge | Week 2 | Build working query tool |
| Crate mastery (Phase 1) | Week 3 | All Phase 1 tasks complete |
| REPL competency | Week 5 | REPL with editing + completion |
| Formatting mastery | Week 7 | All CQL types format correctly |
| Completion mastery | Week 10 | All completion contexts work |
| COPY mastery | Week 15 | COPY TO/FROM pass all tests |
| Testing mastery | Week 19 | >90% coverage achieved |

### Knowledge Sharing

- Weekly 30-min tech talks on areas being developed
- Document "lessons learned" for each phase
- Maintain a shared "gotchas" document for Python cqlsh quirks
- Record screen sessions for complex debugging/profiling
