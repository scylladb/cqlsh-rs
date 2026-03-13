# Sub-Plan SP13: Skills Development

> Parent: [high-level-design.md](high-level-design.md) | All Phases
>
> **This is a living document.** Update it as development progresses — mark skills as acquired, add lessons learned, and adjust priorities based on actual experience.

## Objective

Define which skills must be developed **before** development starts (blockers), which should be developed **alongside** each phase (just-in-time), and which can be deferred. The plan prioritizes getting the right skills at the right time rather than front-loading everything.

---

## Skills Inventory

### Skill Level Definitions

| Level | Definition |
|-------|-----------|
| **Basic** | Can read and understand code; can follow tutorials; needs guidance for non-trivial tasks |
| **Intermediate** | Can implement features independently; can debug issues; understands trade-offs |
| **Advanced** | Can design systems; can optimize performance; can mentor others; deep understanding |

### Required Skills Matrix

| # | Skill | Required Level | When Needed | Develop | Status |
|---|-------|---------------|-------------|---------|--------|
| S1 | Rust (core language) | Advanced | All phases | **Before dev** | [ ] |
| S2 | Async Rust (Tokio) | Intermediate | Phase 1+ | **Before dev** | [ ] |
| S3 | CQL protocol & Cassandra | Intermediate | Phase 1+ | **Before dev** | [ ] |
| S5 | CLI design (clap) | Intermediate | Phase 1 | **Before dev** | [ ] |
| S9 | Testing methodologies | Intermediate | Phase 1+ (advanced by Phase 5) | **Before dev** (basics), upgrade during Phase 5 | [ ] |
| S11 | CI/CD (GitHub Actions) | Intermediate | Phase 1+ | **Before dev** | [ ] |
| S4 | Terminal/TUI programming | Intermediate | Phase 2 | With Phase 2 | [ ] |
| S6 | Parser design | Intermediate | Phase 1-3 | With Phase 1 | [ ] |
| S8 | SSL/TLS | Basic | Phase 2 | With Phase 2 | [ ] |
| S7 | CSV/data processing | Intermediate | Phase 4 | With Phase 4 | [ ] |
| S10 | Performance profiling | Intermediate | Phase 5 | With Phase 5 | [ ] |
| S12 | Cross-compilation | Basic | Phase 6 | With Phase 6 | [ ] |

---

## Priority 1: Before Development Starts (Gate)

> **No development work begins until these skills are validated.** These are the foundation that every subsequent phase depends on. Attempting Phase 1 without these will result in constant rework.

### Gate A: Rust Core (must-have)

**Goal:** All team members at Advanced Rust level.

| Topic | Resources | Exercise | Validation |
|-------|-----------|----------|------------|
| Ownership & borrowing | The Rust Book ch. 4, 10, 15 | Implement a linked list, arena allocator | Code review passes |
| Error handling patterns | `anyhow` + `thiserror` docs | Design error type hierarchy | Implement `error.rs` prototype |
| Traits & generics | The Rust Book ch. 10, 17 | Design driver trait with associated types | Trait compiles with mock impl |
| Async Rust | Tokio tutorial, async book | Async TCP client | Connect to Cassandra |
| Async patterns | `futures` crate, streams, channels | Concurrent task pipeline | Working demo |

**Resources:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — Chapters 4, 10, 15, 17
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

### Gate B: Domain Knowledge (must-have)

**Goal:** Deep understanding of CQL and Python cqlsh behavior. Without this, every design decision will be wrong.

| Topic | Resources | Exercise | Validation |
|-------|-----------|----------|------------|
| CQL fundamentals | Apache Cassandra docs, CQL spec | Create schema, CRUD operations | Can write non-trivial CQL |
| Cassandra data model | DataStax documentation | Design a data model | Review passes |
| Python cqlsh deep-dive | Read cqlsh source code | Document all commands and behaviors | Behavior catalog complete |
| Python cqlsh internals | Read `cqlshlib/*.py` | Understand formatting, completion, COPY | Technical notes written |

**Resources:**
- [CQL Reference](https://cassandra.apache.org/doc/latest/cassandra/cql/)
- [Python cqlsh source](https://github.com/apache/cassandra/tree/trunk/bin/cqlsh.py)
- [cqlshlib source](https://github.com/apache/cassandra/tree/trunk/pylib/cqlshlib/)
- [DataStax CQL Guide](https://docs.datastax.com/en/cql-oss/)

### Gate C: Toolchain & CI (must-have)

| Topic | Resources | Exercise | Validation |
|-------|-----------|----------|------------|
| clap v4 derive API | [clap docs](https://docs.rs/clap) | Build CLI app with 20+ flags | Matches `cqlsh --help` |
| GitHub Actions basics | GH Actions docs | CI workflow with test + lint | Workflow runs green |
| cargo test patterns | Rust book testing chapter | Unit + integration test | Tests pass |

### Gate Completion Criteria

Development may begin when:
- [ ] Every team member passes a Rust coding exercise (reviewed by peer)
- [ ] Every team member can connect to Cassandra and run CQL queries
- [ ] The Python cqlsh behavior catalog is complete (all commands documented)
- [ ] A basic CI workflow is running (even on an empty project)

---

## Priority 2: Develop Alongside Each Phase (Just-in-Time)

> These skills are needed for specific phases. Learn them as part of the phase work — the phase tasks serve as the practice exercises.

### With Phase 1 — CLI & Driver Foundations

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **scylla driver** | [scylla docs](https://docs.rs/scylla), examples dir | Task 1.7: scylla driver impl | Integration test passes |
| **rust-ini** | Crate docs | Task 1.4: cqlshrc parsing | Config tests pass |
| **Statement parsing** | CQL spec, Python parser source | Task 1.10: multi-line buffering | Parser tests pass |

### With Phase 2 — REPL & Formatting

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **rustyline** | [API docs](https://docs.rs/rustyline), examples | Task 2.1: rustyline integration | REPL with editing works |
| **comfy-table** | [Docs](https://docs.rs/comfy-table) | Task 2.4: tabular formatting | Output matches Python cqlsh |
| **owo-colors** / **crossterm** | Crate docs | Tasks 3.16-3.18: coloring | Highlighted output works |
| **rustls** | Docs + examples | Task 2.22: SSL/TLS support | TLS connection works |
| **Cassandra system_schema** | Apache docs, system table descriptions | Task 2.8-2.13: DESCRIBE commands | DESCRIBE output correct |

### With Phase 3 — Tab Completion & Output

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **CQL grammar** | CQL spec, ANTLR grammar file | Task 3.1-3.10: completer | Context-aware completion works |
| **Schema introspection** | system_schema.* table docs | Task 3.2: schema cache | Cache populates and invalidates |
| **serde_json** | Crate docs | Task 3.11: JSON output | JSON output matches Python cqlsh |
| **chrono** | Crate docs, strftime format | Type formatting | Timestamps format correctly |

### With Phase 4 — COPY & Advanced

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **csv crate** | [Docs](https://docs.rs/csv) | Tasks 4.1-4.2: COPY TO basic | CSV roundtrip works |
| **Tokio channels** | Tokio docs | Task 4.8: parallel workers | Parallel COPY FROM works |
| **Rate limiting** | `governor` or manual token bucket | Task 4.9: INGESTRATE | Rate limiting accurate |
| **Concurrent patterns** | `futures::stream`, `buffer_unordered` | Task 4.4: concurrent fetching | COPY TO parallelism works |

### With Phase 5 — Testing & Benchmarking

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **testcontainers-rs** | [Docs](https://docs.rs/testcontainers) | Task 5.2: integration harness | Tests spin up containers |
| **assert_cmd** | [Docs](https://docs.rs/assert_cmd) | CLI binary tests | CLI tests pass |
| **insta** | [Docs](https://docs.rs/insta) | Snapshot tests | Snapshots created |
| **proptest** | [Book](https://altsysrq.github.io/proptest-book/) | Task 5.6: property tests | Properties hold |
| **criterion** | [User guide](https://bheisler.github.io/criterion.rs/book/) | Tasks 5.7-5.12: benchmarks | Benchmarks run with reports |
| **cargo-tarpaulin** / **llvm-cov** | Tool docs | Coverage reports | >90% coverage |
| **flamegraph** | `cargo flamegraph` | Profile hot paths | Flamegraph generated |

### With Phase 6 — Cross-Platform & Release

| Skill | Learning Path | Practice = Phase Task | Validation |
|-------|-------------|----------------------|------------|
| **Cross-compilation** | cross-rs docs, CI examples | Task 6.2: cross targets | Binaries build for all targets |
| **cargo-dist / release** | Tool docs | Task 6.1: release workflow | GitHub Releases work |
| **clap_mangen** | Crate docs | Man page generation | Man page installs |

---

## Skill Development Tracks

Three parallel learning tracks for team members. Each person should primarily own one track but cross-train on the others:

```
Track A: Core Engine          Track B: User Interface       Track C: Quality & DevOps
-------------------------     ------------------------      --------------------------
Rust + Async Rust             rustyline deep-dive           Testing frameworks
CQL protocol                  Terminal UI / ANSI            CI/CD setup
scylla driver                 Tab completion design         Benchmarking tools
Statement parsing             Syntax highlighting           Cross-compilation
COPY TO/FROM                  Color schemes                 Release automation
                              Pagination                    Coverage tooling
```

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

| Checkpoint | Gate/Phase | Assessment Method |
|-----------|-----------|------------------|
| Rust proficiency | **Gate** (before dev) | Coding challenge + code review |
| Async understanding | **Gate** (before dev) | Implement working async driver |
| CQL knowledge | **Gate** (before dev) | Build working query tool |
| Crate mastery (Phase 1) | Phase 1 complete | All Phase 1 tasks done |
| REPL competency | Phase 2 complete | REPL with editing + completion |
| Formatting mastery | Phase 2 complete | All CQL types format correctly |
| Completion mastery | Phase 3 complete | All completion contexts work |
| COPY mastery | Phase 4 complete | COPY TO/FROM pass all tests |
| Testing mastery | Phase 5 complete | >90% coverage achieved |

### Knowledge Sharing

- Weekly 30-min tech talks on areas being developed
- Document "lessons learned" for each phase
- Maintain a shared "gotchas" document for Python cqlsh quirks
- Record screen sessions for complex debugging/profiling
