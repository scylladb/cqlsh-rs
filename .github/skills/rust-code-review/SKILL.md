---
name: rust-code-review
description: >-
  Perform comprehensive Rust code review covering correctness, safety,
  performance, and idiomatic patterns. Use when asked to review code,
  check for issues, audit a module, or ensure code quality before merge.
  Covers ownership, lifetimes, concurrency, API design, and Rust idioms.
allowed-tools: Read, Grep, Glob, Bash
user-invocable: true
---

# Rust Code Review

Perform a thorough, multi-dimensional code review of Rust code in the cqlsh-rs project. Reviews cover correctness, safety, performance, idiomatic patterns, and maintainability.

## Review Process

1. **Read the code** — Read all changed or targeted files completely
2. **Understand context** — Check how the code fits into the module hierarchy
3. **Apply checklist** — Systematically evaluate against each review dimension
4. **Report findings** — Categorize by severity and provide actionable fixes

## Review Dimensions

### 1. Correctness

- [ ] Logic errors: off-by-one, boundary conditions, edge cases
- [ ] Pattern matching: exhaustive matches, no unreachable arms
- [ ] Integer arithmetic: overflow potential (use `checked_*` or `saturating_*`)
- [ ] String handling: UTF-8 assumptions, proper boundary checks
- [ ] Iterator usage: correct use of `map`, `filter`, `flat_map` semantics
- [ ] Async correctness: no blocking calls in async context, proper `await` placement

### 2. Memory Safety & Ownership

- [ ] No unnecessary `clone()` — borrow where possible
- [ ] Lifetime annotations are minimal and correct
- [ ] No dangling references possible through API misuse
- [ ] `Arc`/`Rc` used only when shared ownership is genuinely needed
- [ ] `unsafe` blocks: justified, minimal, well-documented invariants
- [ ] No memory leaks from circular `Arc`/`Rc` references

### 3. Concurrency Safety

- [ ] Shared state protected by `Mutex`/`RwLock` with minimal critical sections
- [ ] No potential deadlocks (consistent lock ordering)
- [ ] `Send`/`Sync` bounds are correct for async task spawning
- [ ] Channel usage: bounded channels preferred, unbounded justified
- [ ] Tokio: no blocking operations on async runtime (`spawn_blocking` for CPU work)

### 4. Error Handling

- [ ] No `unwrap()`/`expect()` in library code
- [ ] Errors propagated with context, not swallowed
- [ ] Error types are specific and descriptive
- [ ] `?` operator used consistently
- [ ] Fallible operations return `Result`, not `panic!`

### 5. Performance

- [ ] No unnecessary allocations (prefer `&str` over `String` in params)
- [ ] Collections pre-sized with `with_capacity` when size is known
- [ ] No `clone()` in hot paths without justification
- [ ] Iterators preferred over indexed loops
- [ ] String building uses `String::with_capacity` or `write!` for many appends
- [ ] No N+1 query patterns in database interactions
- [ ] `Cow<str>` used for conditional ownership

### 6. API Design

- [ ] Functions take the most general type (`&str` not `&String`, `&[T]` not `&Vec<T>`)
- [ ] Builder pattern for types with many optional fields
- [ ] `impl Into<T>` for flexible parameter types at public API boundaries
- [ ] `#[must_use]` on pure functions that return values
- [ ] Public types derive appropriate traits (`Debug`, `Clone`, `PartialEq`, etc.)
- [ ] Destructors (`Drop`) are infallible and non-panicking

### 7. Idiomatic Rust

- [ ] `if let` / `let...else` for single-variant pattern matches
- [ ] `matches!()` macro for boolean pattern checks
- [ ] Method chaining on iterators instead of manual loops
- [ ] `Default` trait implemented and used where appropriate
- [ ] `From`/`Into` conversions instead of ad-hoc conversion functions
- [ ] Type state pattern for compile-time state machine enforcement
- [ ] Tuple structs for newtypes, not raw type aliases

### 8. Documentation & Naming

- [ ] Public items have doc comments explaining purpose and semantics
- [ ] Examples in doc comments for non-obvious APIs
- [ ] Variable names are clear without being verbose
- [ ] Module organization follows Rust conventions
- [ ] `///` for public docs, `//` for implementation notes

## Severity Levels

| Level | Label | Action |
|-------|-------|--------|
| P0 | **Blocker** | Must fix — correctness bug, safety issue, data loss risk |
| P1 | **Major** | Should fix — performance issue, error handling gap, API misuse risk |
| P2 | **Minor** | Consider fixing — style, idiom, minor improvement |
| P3 | **Nit** | Optional — cosmetic, naming preference |

## Output Format

```markdown
## Code Review: `<module/file>`

### Summary
<1-2 sentence overview of code quality and main findings>

### Findings

#### P0 — Blockers
- **[file:line]** <description>
  ```rust
  // suggested fix
  ```

#### P1 — Major
...

#### P2 — Minor
...

### Positive Observations
- <things done well>
```

## cqlsh-rs Specific Checks

- CQL type handling matches Python cqlsh behavior exactly
- Output formatting is byte-identical to Python cqlsh where specified
- CLI flag names and short forms match Python cqlsh `--help` output
- cqlshrc parsing handles all documented sections and options
- Connection handling follows the same retry/timeout behavior as Python cqlsh
