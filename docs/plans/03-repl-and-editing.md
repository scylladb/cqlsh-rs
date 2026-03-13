# Sub-Plan SP3: REPL & Line Editing

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1-2

## Objective

Implement an interactive REPL with line editing, history, multi-line input support, and prompt management that behaves identically to Python cqlsh's interactive mode.

---

## Research Phase

### Tasks

1. **Python cqlsh REPL behavior** — Prompt format, multi-line continuation, history file format
2. **`rustyline` vs `reedline`** — Feature comparison, API for custom completers/highlighters
3. **Key bindings** — Emacs-mode defaults, Ctrl-C/Ctrl-D/Ctrl-L behavior
4. **History behavior** — File location, max size, dedup, multi-line entries
5. **Terminal detection** — TTY vs pipe, `--tty` flag behavior

### Research Deliverables

- [ ] Prompt format specification (all states)
- [ ] Key binding behavior catalog
- [ ] History file format and behavior spec
- [ ] rustyline vs reedline comparison matrix

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | Basic REPL loop (stdin readline, no editing) | `repl.rs` | Unit: loop lifecycle |
| 2 | rustyline integration with default config | `repl.rs` | Manual: editing works |
| 3 | Primary prompt: `cqlsh>` | `repl.rs` | Unit: prompt format |
| 4 | Keyspace prompt: `cqlsh:ks>` | `repl.rs` | Unit: keyspace in prompt |
| 5 | Continuation prompt: `   ...` for multi-line | `repl.rs` | Unit: multi-line prompt |
| 6 | Username@host prompt (when connected) | `repl.rs` | Unit: connected prompt |
| 7 | Persistent history (`~/.cqlsh_history`) | `repl.rs` | Integration: history persists |
| 8 | History max size (configurable) | `repl.rs` | Unit: config integration |
| 9 | Ctrl-C handling (cancel current input) | `repl.rs` | Manual: interrupt works |
| 10 | Ctrl-D handling (exit on empty line) | `repl.rs` | Manual: exit works |
| 11 | Ctrl-L handling (clear screen) | `repl.rs` | Manual: clear works |
| 12 | Multi-line input buffering (delegate to parser) | `repl.rs`, `parser.rs` | Unit: semicolon detection |
| 13 | Non-interactive mode detection (pipe/redirect) | `repl.rs` | Unit: TTY detection |
| 14 | `--tty` flag override | `repl.rs` | Unit: force TTY mode |

### Acceptance Criteria

- [ ] Prompt matches Python cqlsh format in all states
- [ ] Line editing (arrow keys, Home/End, word movement) works
- [ ] History persists across sessions in `~/.cqlsh_history`
- [ ] Multi-line input shows continuation prompt
- [ ] Ctrl-C cancels input without exiting
- [ ] Ctrl-D exits on empty line
- [ ] Pipe/redirect mode works without editing features
- [ ] `--tty` forces interactive mode even in pipe

---

## Skills Required

- `rustyline` API (C3)
- Terminal programming (S4)
- Signal handling in Rust (S1)
