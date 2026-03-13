---
name: rust-clippy
description: >-
  Run Clippy with strict lint settings and fix warnings for Rust code.
  Use when asked to lint code, fix clippy warnings, enforce Rust idioms,
  check code quality, or run static analysis. Applies project-specific
  lint configuration and explains fixes.
allowed-tools: Bash, Read, Edit, Grep, Glob
---

# Rust Clippy Lint Enforcer

Run Clippy with strict settings, analyze warnings, and apply idiomatic fixes for the cqlsh-rs project.

## Workflow

1. Run Clippy with strict settings to identify all warnings
2. Categorize warnings by severity and type
3. Apply fixes automatically where safe, manually where judgment is needed
4. Verify fixes compile and pass tests

## Running Clippy

### Full project scan

```bash
cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic -W clippy::nursery 2>&1
```

### Specific module

```bash
cargo clippy --lib -- -W clippy::all 2>&1
```

### Auto-fix safe suggestions

```bash
cargo clippy --fix --allow-dirty --allow-staged -- -W clippy::all 2>&1
```

## Lint Categories

Apply these lint levels for cqlsh-rs:

### Always fix (deny)

- `clippy::unwrap_used` — Use `expect()` with context or propagate with `?`
- `clippy::panic` — Replace with proper error handling
- `clippy::todo` — Implement or convert to tracked issue
- `clippy::unimplemented` — Implement or return error
- `clippy::dbg_macro` — Remove before commit
- `clippy::print_stdout` / `clippy::print_stderr` — Use `tracing` or `eprintln!` for user-facing output

### Strongly recommended (warn → fix)

- `clippy::needless_pass_by_value` — Take references where ownership isn't needed
- `clippy::redundant_clone` — Remove unnecessary `.clone()` calls
- `clippy::large_enum_variant` — Box large variants to reduce enum size
- `clippy::inefficient_to_string` — Use `to_owned()` for `&str` to `String`
- `clippy::manual_let_else` — Use `let...else` pattern (Rust 1.65+)
- `clippy::needless_borrow` — Remove unnecessary `&` or `&mut`
- `clippy::explicit_iter_loop` — Use `for x in &collection` instead of `.iter()`
- `clippy::match_wildcard_for_single_variants` — Name the variant explicitly
- `clippy::implicit_clone` — Use explicit `.clone()` for clarity
- `clippy::uninlined_format_args` — Use inline format args `{var}` instead of `{}, var`
- `clippy::semicolon_if_nothing_returned` — Add semicolon to unit-returning expressions

### Context-dependent (evaluate case by case)

- `clippy::module_name_repetitions` — Allow if it matches CQL/Cassandra terminology
- `clippy::must_use_candidate` — Add `#[must_use]` to pure functions returning values
- `clippy::missing_errors_doc` — Add error docs to public API functions
- `clippy::missing_panics_doc` — Document panic conditions

## Fix Patterns

### Replace `unwrap()` with context

```rust
// Before
let value = map.get("key").unwrap();

// After
let value = map.get("key").expect("key must exist after validation");

// Or propagate
let value = map.get("key").ok_or_else(|| anyhow!("missing key"))?;
```

### Fix needless clones

```rust
// Before
fn process(data: String) { /* only reads data */ }
process(my_string.clone());

// After
fn process(data: &str) { /* only reads data */ }
process(&my_string);
```

### Use let-else for early returns

```rust
// Before
let value = match optional {
    Some(v) => v,
    None => return Err(Error::Missing),
};

// After
let Some(value) = optional else {
    return Err(Error::Missing);
};
```

### Box large enum variants

```rust
// Before — large variant bloats all variants
enum Command {
    Simple(u8),
    Complex { data: [u8; 1024], metadata: HashMap<String, String> },
}

// After
enum Command {
    Simple(u8),
    Complex(Box<ComplexCommand>),
}
```

## Output Format

After running Clippy, report:

1. **Summary**: Total warnings by category
2. **Critical**: Items that must be fixed (deny-level)
3. **Recommended**: Items that should be fixed (warn-level)
4. **Skipped**: Items intentionally allowed with rationale
5. **Verification**: Confirm `cargo check` and `cargo test` pass after fixes
