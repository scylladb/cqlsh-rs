# Sub-Plan SP18: Unified CQL Lexer

> Parent: [high-level-design.md](high-level-design.md) | Cross-cutting
>
> **This is a living document.** Update it as development progresses.
> **Status: COMPLETED** (2026-03-31) — Unified lexer in `src/cql_lexer.rs`, all three consumers migrated.

## Objective

Build a shared CQL lexer (tokenizer) with grammar-aware position tracking, replacing the three ad-hoc implementations in colorizer, completer, and parser. This unifies CQL understanding into a single component.

---

## Motivation

Python cqlsh uses a full CQL lexer that powers syntax highlighting, tab completion, and statement parsing from one shared component. cqlsh-rs currently has three separate, simpler implementations:

| Module | Current approach | Limitation |
|--------|-----------------|------------|
| `src/colorizer.rs` | Word-level keyword matching | Highlights identifiers as keywords (USERS, KEY, SET after FROM/INTO) |
| `src/completer.rs` | Simple token splitting | Poor context detection, no priority ordering |
| `src/parser.rs` | Incremental char scanner | Works but duplicates tokenization logic |

A shared lexer fixes all three by understanding **what syntactic position** each token occupies.

## Requirements & Constraints

| ID | Type | Description |
|----|------|-------------|
| REQ-01 | Requirement | Tokenize CQL input into typed tokens (keyword, identifier, string, number, operator, etc.) |
| REQ-02 | Requirement | Track grammar state: what kind of token is expected next (table name, column list, value, etc.) |
| REQ-03 | Requirement | Handle incomplete input (for REPL line-by-line feeding) |
| REQ-04 | Requirement | Handle strings, comments, escaped characters, multi-line input |
| REQ-05 | Constraint | Must be fast enough for real-time syntax highlighting on every keystroke |
| REQ-06 | Constraint | Must not break existing functionality — migrate incrementally |
| GUD-01 | Guideline | Study Python cqlsh's `cqlshlib/cqlhandling.py` for grammar rules |

## Design Decisions

| Decision | Choice | Rationale | Alternatives Rejected |
|----------|--------|-----------|----------------------|
| Approach | Hand-written state machine | Fast, no dependencies, fits REPL constraints | pest/nom parser (too heavy for keystroke-level use) |
| Token types | Enum with ~15 variants | Covers CQL grammar without over-engineering | AST (too complex for highlighting/completion) |
| Grammar tracking | Simple state enum (expecting table, expecting column, etc.) | Good enough for highlighting and completion | Full CQL parser (overkill) |

## Token Types (Draft)

```rust
enum CqlToken {
    Keyword(String),       // SELECT, FROM, WHERE, etc.
    Identifier(String),    // table names, column names, keyspace names
    StringLiteral(String), // 'single quoted'
    NumberLiteral(String), // 42, 3.14, -1
    UuidLiteral(String),   // 550e8400-e29b-...
    BlobLiteral(String),   // 0xdeadbeef
    BooleanLiteral(bool),  // true, false
    Operator(String),      // =, <, >, <=, >=, !=
    Punctuation(char),     // (, ), ,, ;, .
    Star,                  // *
    Comment(String),       // -- comment
    Whitespace(String),    // spaces, tabs
}

enum GrammarContext {
    Start,                 // beginning of statement
    AfterSelect,           // expecting column list or *
    AfterFrom,             // expecting table name
    AfterInto,             // expecting table name
    AfterUse,              // expecting keyspace name
    AfterDot,              // expecting qualified name part
    AfterWhere,            // expecting column name
    InColumnList,          // inside column list
    InValueList,           // inside VALUES (...)
    InWithClause,          // inside WITH options
    Other,                 // default
}
```

## Implementation Tasks

| # | Task | Description | Validation |
|---|------|-------------|------------|
| 1 | Create `src/cql_lexer.rs` | Token enum, tokenize function, grammar context tracking | Unit tests for all token types |
| 2 | Migrate colorizer | Replace word-level matching with lexer tokens | Identifiers after FROM/INTO/. no longer highlighted as keywords |
| 3 | Migrate completer | Use grammar context for completion ordering | * offered first after SELECT; tables after FROM |
| 4 | Migrate parser | Reuse tokenizer for statement boundary detection | All existing parser tests still pass |
| 5 | Remove duplicated logic | Clean up ad-hoc tokenization in all three modules | No duplicate parsing code remains |

## Risks

| ID | Risk | Mitigation |
|----|------|-----------|
| RISK-01 | Performance regression on keystroke highlighting | Benchmark before/after; lexer must be O(n) single pass |
| RISK-02 | CQL grammar edge cases | Test against Python cqlsh behavior for complex queries |
| RISK-03 | Large refactor scope | Migrate one module at a time, keep old code as fallback |

## Open Questions

| # | Question | Status | Decision |
|---|----------|--------|----------|
| 1 | Should the lexer understand prepared statement placeholders (?) | Open | |
| 2 | Should it handle CQL functions (now(), uuid(), etc.)? | Open | Probably yes for completion |
| 3 | Priority vs other Phase 4/5 work? | Open | Consider after COPY TO/FROM is stable |

## Status

**COMPLETED** (2026-03-31)

- Task 1: Created `src/cql_lexer.rs` with hand-written state machine tokenizer (112 unit tests)
- Task 2: Migrated `src/colorizer.rs` to use lexer — fixes false keyword highlights on identifiers (USERS, KEY, SET after FROM/INTO)
- Task 3: Migrated `src/completer.rs` to use lexer — grammar-aware context detection via `GrammarContext`
- Task 4: Migrated `src/parser.rs` to use lexer — `strip_comments` delegates to `cql_lexer::strip_comments`
- Task 5: Removed duplicated comment-stripping code from parser; colorizer keyword list and tokenizer deleted
- All 441 library tests pass, zero clippy warnings
