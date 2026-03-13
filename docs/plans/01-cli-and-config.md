# Sub-Plan SP1: CLI & Configuration

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1 (Bootstrap MVP)

## Objective

Implement 100% command-line argument compatibility and full `~/.cqlshrc` configuration file support, ensuring any existing Python cqlsh invocation or configuration works identically with cqlsh-rs.

---

## Research Phase

### Tasks

1. **Audit Python cqlsh CLI** — Extract every flag from `cqlsh --help` across Cassandra 3.11, 4.x, and 5.x
2. **Audit Python cqlshrc** — Read `cqlshlib/cqlsh_config.py` and document every section/key
3. **Audit environment variables** — Grep Python cqlsh source for `os.environ` and `os.getenv`
4. **Document precedence rules** — CLI > env > cqlshrc > defaults (verify in Python source)
5. **Collect real-world cqlshrc files** — Gather sample configurations from the community

### Research Deliverables

- [ ] Complete CLI flag table with default values, types, and validation rules
- [ ] Complete cqlshrc section/key table with types, defaults, and descriptions
- [ ] Precedence rule specification
- [ ] Edge case catalog (conflicting flags, invalid values, missing files)

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests |
|------|-------------|--------|-------|
| 1 | Define `CliArgs` struct with `clap` derive | `main.rs` | Unit: parse known flag combos |
| 2 | All positional args (`[host]`, `[port]`) | `main.rs` | Unit: positional parsing |
| 3 | All optional flags (see matrix in parent) | `main.rs` | Unit: each flag individually |
| 4 | Flag validation (mutually exclusive, ranges) | `main.rs` | Unit: invalid combos error |
| 5 | `CqlshrcConfig` struct with INI parsing | `config.rs` | Unit: parse sample files |
| 6 | All cqlshrc sections: `[authentication]`, `[connection]`, `[ssl]`, `[ui]`, `[csv]`, `[copy]`, `[copy-to]`, `[copy-from]`, `[tracing]` | `config.rs` | Unit: each section |
| 7 | Environment variable loading | `config.rs` | Unit: env override |
| 8 | `MergedConfig` with precedence: cli > env > cqlshrc > defaults | `config.rs` | Unit: precedence tests |
| 9 | `--cqlshrc` flag to specify custom path | `config.rs` | Unit: custom path |
| 10 | Graceful handling of missing/malformed cqlshrc | `config.rs` | Unit: error cases |

### Acceptance Criteria

- [ ] `cqlsh-rs --help` output matches Python cqlsh `--help` structure
- [ ] Every flag from the compatibility matrix is accepted
- [ ] Unrecognized flags produce helpful error messages
- [ ] Existing `~/.cqlshrc` files parse without error
- [ ] Precedence rules match Python cqlsh behavior exactly
- [ ] Environment variables override cqlshrc values
- [ ] CLI args override everything

---

## Skills Required

- `clap` v4 derive API (S5)
- INI parsing with `rust-ini` or `configparser` (S5)
- Rust error handling patterns (S1)

---

## Key Decisions

| Decision | Options | Recommendation |
|----------|---------|---------------|
| INI parser crate | `rust-ini` vs `configparser` | Prototype both; `configparser` is closer to Python's `configparser` |
| Unknown cqlshrc keys | Error vs warn vs ignore | Warn and ignore (forward compatibility) |
| `--help` format | Match Python exactly vs clap default | Customize clap help template to match |
