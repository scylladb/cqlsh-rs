---
name: remember
description: >-
  Save lessons learned, debugging insights, and project-specific knowledge
  into persistent memory files. Use when asked to remember something,
  save a lesson, record a gotcha, document a pattern, or persist knowledge
  for future sessions. Syntax: /remember [domain] lesson content.
---

# Memory Keeper

Transform debugging sessions, workflow discoveries, and hard-won lessons into persistent, domain-organized knowledge that helps AI assistants find relevant guidance in future sessions.

## Syntax

```
/remember [domain] lesson content
```

- `domain` — Optional. Target a specific domain (e.g., `rust`, `cql`, `driver`, `testing`)
- `lesson content` — Required. The lesson to remember

### Examples

```
/remember rust prefer `thiserror` for library errors, `anyhow` for application code
/remember cql system_schema tables require ALL permission, not just SELECT
/remember driver scylla driver connection pools are per-node, not global
/remember testing testcontainers needs Docker socket access in CI
/remember always run cargo clippy before committing
```

## Memory File Locations

### Claude Code

Memory files are stored as instructions in `.claude/`:

- **General**: `.claude/memories.md`
- **Domain-specific**: `.claude/memories-{domain}.md`

### GitHub Copilot

Memory files are stored as instruction files in `.github/instructions/`:

- **General**: `.github/instructions/memory.instructions.md`
- **Domain-specific**: `.github/instructions/{domain}-memory.instructions.md`

Copilot instruction files use `applyTo` frontmatter for targeted activation:

```yaml
---
description: 'Lessons learned about Rust patterns in this project'
applyTo: '**/*.rs'
---
```

## Process

1. **Parse input** — Extract domain (if specified) and lesson content
2. **Read existing memory files** — Check what domains already exist to avoid duplication
3. **Categorize the learning**:
   - New gotcha / common mistake
   - Enhancement to existing knowledge
   - New best practice or pattern
   - Process improvement
4. **Determine target domain and file paths**:
   - If domain specified, use `{domain}-memory` files
   - If no domain, match to existing domains or use general memory
   - When uncertain, ask the user
5. **Read the target files** to avoid redundancy
6. **Write the memory** to BOTH Claude and Copilot locations for dual compatibility

## Memory File Structure

### For Claude Code (`.claude/memories-{domain}.md`)

```markdown
# {Domain} Memory

> Lessons learned and patterns for {domain} in cqlsh-rs.

## {Lesson Title}

{Concise, actionable lesson content}
```

### For Copilot (`.github/instructions/{domain}-memory.instructions.md`)

```markdown
---
description: 'Lessons learned about {domain} in this project'
applyTo: '{relevant glob pattern}'
---

# {Domain} Memory

> Lessons learned and patterns for {domain} in cqlsh-rs.

## {Lesson Title}

{Concise, actionable lesson content}
```

## Domain Mapping

| Domain | Copilot `applyTo` | Description |
|--------|-------------------|-------------|
| `rust` | `**/*.rs` | Rust language patterns and idioms |
| `cql` | `**/*.cql,**/cql*` | CQL protocol and query patterns |
| `driver` | `**/driver/**` | Cassandra/Scylla driver specifics |
| `testing` | `**/tests/**,**/*test*` | Testing patterns and gotchas |
| `ci` | `.github/workflows/**` | CI/CD configuration lessons |
| `config` | `**/config*,**/*.ini` | Configuration handling |
| `plan` | `docs/plans/**` | Planning and documentation patterns |
| (general) | `**/*` | Cross-cutting lessons |

## Writing Guidelines

- **Generalize from specifics** — Extract reusable patterns, not task-specific details
- **Be concrete** — Include code examples when relevant
- **Focus on what TO do** — Use positive reinforcement over "don't" instructions
- **Keep it scannable** — Short paragraphs, bullet points, code snippets
- **Explain the why** — Context helps the AI apply the lesson correctly
- **Remove redundancy** — If a lesson duplicates existing knowledge, merge instead
