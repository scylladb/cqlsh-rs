# Sub-Plan SP1: CLI & Configuration

> Parent: [high-level-design.md](high-level-design.md) | Phase: 1 (Bootstrap MVP)
> **Status: IN PROGRESS** — Initial implementation complete (2026-03-13)

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

- [x] Complete CLI flag table with default values, types, and validation rules (see compatibility matrix in high-level-design.md)
- [x] Complete cqlshrc section/key table with types, defaults, and descriptions (see high-level-design.md)
- [x] Precedence rule specification: CLI > env > cqlshrc > defaults
- [x] Edge case catalog (conflicting flags, invalid values, missing files)

---

## Execution Phase

### Implementation Steps

| Step | Description | Module | Tests | Status |
|------|-------------|--------|-------|--------|
| 1 | Define `CliArgs` struct with `clap` derive | `cli.rs` | 32 unit tests | ✅ Done |
| 2 | All positional args (`[host]`, `[port]`) | `cli.rs` | `positional_host`, `positional_host_and_port` | ✅ Done |
| 3 | All optional flags (see matrix in parent) | `cli.rs` | Individual test per flag | ✅ Done |
| 4 | Flag validation (mutually exclusive, ranges) | `cli.rs` | `validate_color_conflict`, `validate_execute_and_file_conflict`, `validate_protocol_version_range` | ✅ Done |
| 5 | `CqlshrcConfig` struct with INI parsing | `config.rs` | `parse_empty_config`, `parse_full_sample_config`, `load_config_from_tempfile` | ✅ Done |
| 6 | All cqlshrc sections: `[authentication]`, `[connection]`, `[ssl]`, `[ui]`, `[csv]`, `[copy]`, `[copy-to]`, `[copy-from]`, `[tracing]`, `[certfiles]`, `[cql]` | `config.rs` | Individual section tests | ✅ Done |
| 7 | Environment variable loading (`CQLSH_HOST`, `CQLSH_PORT`, `SSL_CERTFILE`, `SSL_VALIDATE`, timeouts, `CQL_HISTORY`) | `config.rs` | `env_overrides_cqlshrc`, CLI integration tests | ✅ Done |
| 8 | `MergedConfig` with precedence: cli > env > cqlshrc > defaults | `config.rs` | `cli_overrides_everything`, `env_overrides_cqlshrc`, `cqlshrc_overrides_defaults`, `merged_defaults` | ✅ Done |
| 9 | `--cqlshrc` flag to specify custom path | `config.rs` | `resolve_cqlshrc_path_custom`, CLI integration: `custom_cqlshrc_path` | ✅ Done |
| 10 | Graceful handling of missing/malformed cqlshrc | `config.rs` | `load_nonexistent_file_returns_default`, `invalid_numeric_ignored` | ✅ Done |
| 11 | Shell completion generation (`--completions bash/zsh/fish/elvish/powershell`) | `shell_completions.rs` | 5 unit tests + 3 CLI integration tests | ✅ Done |
| 12 | CLI-level integration tests | `tests/cli_tests.rs` | 17 integration tests covering flags, env vars, config, completions | ✅ Done |

### Test Summary

| Layer | Count | Location |
|-------|-------|----------|
| Unit tests (CLI) | 32 | `src/cli.rs` |
| Unit tests (config) | 37 | `src/config.rs` |
| Unit tests (completions) | 5 | `src/shell_completions.rs` |
| Integration tests (CLI) | 17 | `tests/cli_tests.rs` |
| **Total** | **91** | |

### Acceptance Criteria

- [x] `cqlsh-rs --help` output shows all flags from the compatibility matrix
- [x] Every flag from the compatibility matrix is accepted (22 flags + 2 positional args)
- [x] Unrecognized flags produce helpful error messages (tested via `unknown_flag_produces_error`)
- [x] Existing `~/.cqlshrc` files parse without error (all 11 sections supported)
- [x] Precedence rules match Python cqlsh behavior: CLI > env > cqlshrc > defaults
- [x] Environment variables override cqlshrc values (tested)
- [x] CLI args override everything (tested)
- [x] Shell completions generated for bash, zsh, fish, elvish, PowerShell
- [ ] TODO: `--help` output template customized to match Python cqlsh layout exactly
- [ ] TODO: `auth_provider` and `protocol` sections (rarely used, lower priority)

---

## Key Decisions (Resolved)

| Decision | Chosen | Rationale |
|----------|--------|-----------|
| INI parser crate | `configparser` v3 | Closest to Python's `configparser` behavior; case-sensitive mode works well |
| Unknown cqlshrc keys | Silently ignore | Forward compatibility; unknown keys don't cause errors |
| `--help` format | Clap default (for now) | Will customize help template in a future iteration to match Python layout |
| Module structure | Separate `cli.rs`, `config.rs`, `shell_completions.rs` | Clean separation of concerns vs everything in `main.rs` |
| Boolean parsing | `true/yes/on/1` → true, else false | Matches Python cqlsh behavior |
| Missing cqlshrc | Return default config | Graceful handling, no error for missing file |
| Shell completions | `clap_complete` crate via `--completions <shell>` flag | Standard approach for Rust CLI tools |

---

## Skills Required

- `clap` v4 derive API ✅
- INI parsing with `configparser` ✅
- Rust error handling with `anyhow` + `thiserror` ✅
- CLI integration testing with `assert_cmd` + `predicates` ✅
- Shell completion generation with `clap_complete` ✅
