# Sub-Plan SP7: Built-in Commands

> Parent: [high-level-design.md](high-level-design.md) | Phase: 2-4

## Objective

Implement all built-in shell commands with 100% behavior parity to Python cqlsh. Each command is a self-contained module with its own parsing, execution, and output formatting.

---

## Research Phase

### Tasks

1. **Audit every Python cqlsh command** — Read `cqlshlib/` source for each command handler
2. **DESCRIBE implementation** — How Python cqlsh generates schema DDL output
3. **Help text** — Exact help text for each command
4. **Error handling** — What errors each command can produce
5. **Command parsing** — How arguments are parsed for each command

### Research Deliverables

- [ ] Per-command specification (arguments, output format, error cases)
- [ ] DESCRIBE DDL generation algorithm
- [ ] Help text catalog
- [ ] Command parsing grammar

---

## Execution Phase

### Command Implementation Matrix

| Command | Module | Phase | Complexity | Key Challenges |
|---------|--------|-------|------------|----------------|
| `HELP` / `?` | `commands/help.rs` | 2 | Low | Per-command help text |
| `QUIT` / `EXIT` | `repl.rs` | 1 | Low | Clean shutdown |
| `USE <keyspace>` | `commands/mod.rs` | 2 | Low | Prompt update, session state |
| `DESCRIBE CLUSTER` | `commands/describe.rs` | 2 | Medium | Cluster name + partitioner |
| `DESCRIBE KEYSPACES` | `commands/describe.rs` | 2 | Medium | List all keyspaces |
| `DESCRIBE KEYSPACE [name]` | `commands/describe.rs` | 2 | High | Full DDL output |
| `DESCRIBE TABLES` | `commands/describe.rs` | 2 | Medium | List tables in keyspace |
| `DESCRIBE TABLE <name>` | `commands/describe.rs` | 2 | High | Full CREATE TABLE DDL |
| `DESCRIBE SCHEMA` | `commands/describe.rs` | 2 | High | All DDL for all keyspaces |
| `DESCRIBE FULL SCHEMA` | `commands/describe.rs` | 4 | High | Include system keyspaces |
| `DESCRIBE INDEX` | `commands/describe.rs` | 4 | Medium | CREATE INDEX DDL |
| `DESCRIBE MATERIALIZED VIEW` | `commands/describe.rs` | 4 | Medium | CREATE MV DDL |
| `DESCRIBE TYPE / TYPES` | `commands/describe.rs` | 4 | Medium | UDT DDL |
| `DESCRIBE FUNCTION / FUNCTIONS` | `commands/describe.rs` | 4 | Medium | UDF DDL |
| `DESCRIBE AGGREGATE / AGGREGATES` | `commands/describe.rs` | 4 | Medium | UDA DDL |
| `CONSISTENCY [level]` | `commands/consistency.rs` | 2 | Low | Session state update |
| `SERIAL CONSISTENCY [level]` | `commands/consistency.rs` | 2 | Low | Session state update |
| `TRACING ON/OFF` | `commands/tracing_cmd.rs` | 2 | Medium | Query tracing toggle + display |
| `EXPAND ON/OFF` | `commands/expand.rs` | 3 | Low | Formatter mode toggle |
| `PAGING ON/OFF/<size>` | `commands/paging.rs` | 3 | Low | Pagination config |
| `SOURCE <file>` | `commands/source.rs` | 2 | Medium | File reading, statement execution |
| `CAPTURE <file>/OFF` | `commands/capture.rs` | 3 | Low | Output redirection to file |
| `LOGIN <user> [<pass>]` | `commands/login.rs` | 3 | Medium | Re-authentication |
| `SHOW VERSION` | `commands/show.rs` | 2 | Low | Version strings |
| `SHOW HOST` | `commands/show.rs` | 2 | Low | Connected host |
| `SHOW SESSION <uuid>` | `commands/show.rs` | 3 | Medium | Trace session display |
| `CLEAR` / `CLS` | `commands/clear.rs` | 2 | Low | Terminal clear |

### Implementation Steps (per command)

1. Parse command arguments
2. Validate arguments
3. Execute (query Cassandra, read file, update state, etc.)
4. Format output
5. Handle errors
6. Write tests (unit + integration)

### DESCRIBE Deep-Dive

The DESCRIBE command is the most complex built-in command. It requires:

1. **Schema metadata queries** to `system_schema.*` tables
2. **DDL reconstruction** — Converting metadata rows back into valid CQL DDL statements
3. **Formatting** — Proper indentation, quoting, ordering
4. **Object resolution** — Finding objects by name in the current or specified keyspace
5. **Output matching** — Exact format match with Python cqlsh output

Key system tables:
- `system_schema.keyspaces`
- `system_schema.tables`
- `system_schema.columns`
- `system_schema.indexes`
- `system_schema.views`
- `system_schema.types`
- `system_schema.functions`
- `system_schema.aggregates`
- `system_schema.triggers`

### Acceptance Criteria

- [ ] Every command in the matrix works with correct output
- [ ] HELP shows correct help text for each command
- [ ] DESCRIBE output matches Python cqlsh DDL format
- [ ] Error messages match Python cqlsh
- [ ] Commands are case-insensitive
- [ ] Invalid arguments produce helpful errors

### Estimated Effort

- Research: 3 days
- Implementation: 8 days (DESCRIBE alone is ~4 days)
- Testing: 4 days
- **Total: 15 days**

---

## Skills Required

- Cassandra schema system tables (D2)
- DDL generation / code generation (S6)
- CQL syntax (D1)
- Session state management (S1)
