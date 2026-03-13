# Sub-Plan SP14: Documentation & LLM Skills

> Parent: [high-level-design.md](high-level-design.md) | Phase: 3-6 (ongoing)

## Objective

Build a comprehensive documentation system that serves four audiences: **human users** (reference docs, migration guide), **CI systems** (automated doc builds and preview deploys), **LLM agents** (structured context for AI-assisted workflows), and **skills marketplaces** (publicly registered skills that make cqlsh-rs expertise available to AI platforms).

---

## Research Phase

### Tasks

1. **Documentation generators** — Compare mdBook, Zola, Docusaurus, Sphinx for Rust projects
2. **Existing Cassandra/cqlsh docs** — Audit Apache docs, DataStax docs, ScyllaDB docs for structure and coverage to ensure our docs complement rather than duplicate
3. **CI doc pipelines** — GitHub Pages, Netlify, Cloudflare Pages; PR preview deploys
4. **LLM documentation standards** — `llms.txt` (llmstxt.org), `AGENTS.md`, MCP tool definitions, `.well-known/ai-plugin.json`
5. **Skills marketplaces** — Claude MCP server registry, OpenAI GPT Store, GitHub Copilot Extensions, LangChain Hub, Composio, Toolhouse
6. **Man page generation** — `clap_mangen`, `asciidoctor`, `pandoc` approaches for Rust CLIs

### Research Deliverables

- [ ] Doc generator comparison matrix (features, Rust ecosystem fit, theme quality)
- [ ] Existing documentation gap analysis (what's missing from current Cassandra docs)
- [ ] CI pipeline design for doc build + preview
- [ ] LLM documentation format specification
- [ ] Skills marketplace submission requirements per platform
- [ ] Man page generation approach selection

---

## Execution Phase

### Area 1: Documentation Site

#### Tool Selection

| Tool | Pros | Cons | Verdict |
|------|------|------|---------|
| **mdBook** | Rust-native, simple, `cargo install`, Rust book uses it | Limited themes, no blog | **Primary choice** — fits Rust ecosystem |
| **Zola** | Rust-native, fast, rich themes, shortcodes | More complex, overkill for docs | Alternative if mdBook is too limited |
| **Docusaurus** | Rich features, versioning, search | Node.js dep, heavy | Not recommended |

#### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | mdBook setup (`book.toml`, `src/SUMMARY.md`) | `docs/book/` |
| 2 | Getting started guide | `docs/book/src/getting-started.md` |
| 3 | Installation instructions (all platforms) | `docs/book/src/installation.md` |
| 4 | Configuration reference (all cqlshrc sections) | `docs/book/src/configuration/` |
| 5 | Command reference (all shell commands) | `docs/book/src/commands/` |
| 6 | CLI reference (all flags) | `docs/book/src/cli-reference.md` |
| 7 | Migration guide (Python cqlsh -> cqlsh-rs) | `docs/book/src/migration.md` |
| 8 | Compatibility notes & known divergences | `docs/book/src/divergences.md` |
| 9 | COPY TO/FROM guide with examples | `docs/book/src/copy.md` |
| 10 | Tab completion reference | `docs/book/src/completion.md` |
| 11 | Troubleshooting guide | `docs/book/src/troubleshooting.md` |
| 12 | Performance comparison report | `docs/book/src/performance.md` |

#### Compare with Existing Documentation

Before writing each section, audit what already exists:

| Our Section | Existing Docs to Compare | Our Value-Add |
|------------|-------------------------|---------------|
| Getting started | Apache cqlsh docs, DataStax quick start | Rust install story, single binary |
| Configuration | DataStax cqlshrc reference, Apache docs | Complete reference with all keys, examples |
| Commands | Apache CQL reference, ScyllaDB docs | Interactive examples, edge cases |
| Migration | None exists | **Unique** — step-by-step migration path |
| Performance | None exists | **Unique** — benchmark data vs Python |
| COPY guide | DataStax COPY reference | Better examples, performance tips |

---

### Area 2: CI Documentation Pipeline

#### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | GitHub Actions workflow for doc build | `.github/workflows/docs.yml` |
| 2 | Deploy to GitHub Pages on `main` push | Auto-deploy |
| 3 | PR preview deploys (Netlify/Cloudflare or `gh-pages` branch per PR) | Preview URLs in PR comments |
| 4 | Doc link checking (mdBook built-in or `lychee`) | CI check |
| 5 | Spell checking (`cspell` or `typos`) | CI check |
| 6 | API docs (`cargo doc`) published alongside book | `/api/` path |

#### CI Workflow Design

```yaml
# .github/workflows/docs.yml
name: Documentation
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install mdBook
        run: cargo install mdbook
      - name: Build book
        run: mdbook build docs/book
      - name: Build API docs
        run: cargo doc --no-deps
      - name: Check links
        uses: lycheeverse/lychee-action@v1
      - name: Deploy (main only)
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book/book

  preview:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build book
        run: |
          cargo install mdbook
          mdbook build docs/book
      - name: Deploy preview
        # Use Netlify, Cloudflare Pages, or surge.sh
        run: echo "Preview URL posted to PR"
```

---

### Area 3: LLM-Oriented Documentation

#### Purpose

Provide structured documentation optimized for LLM consumption, enabling AI assistants to understand cqlsh-rs deeply and help users effectively.

#### `llms.txt` Standard

Following the [llmstxt.org](https://llmstxt.org/) specification:

```
# cqlsh-rs

> Rust-based replacement for Python cqlsh — the CQL shell for Apache Cassandra and ScyllaDB.

## Docs

- [Getting Started](https://cqlsh-rs.dev/getting-started.md): Installation and first connection
- [CLI Reference](https://cqlsh-rs.dev/cli-reference.md): All command-line flags
- [Commands](https://cqlsh-rs.dev/commands.md): Shell commands (DESCRIBE, COPY, etc.)
- [Configuration](https://cqlsh-rs.dev/configuration.md): cqlshrc file format
- [Migration](https://cqlsh-rs.dev/migration.md): Moving from Python cqlsh
- [API Docs](https://cqlsh-rs.dev/api/): Rust API documentation

## Optional

- [Divergences](https://cqlsh-rs.dev/divergences.md): Known differences from Python cqlsh
- [Performance](https://cqlsh-rs.dev/performance.md): Benchmark comparisons
```

#### `llms-full.txt`

A single-file comprehensive context dump (~50KB target) containing:
- Complete CLI reference with all flags and defaults
- All shell commands with syntax and examples
- Full cqlshrc reference
- Tab completion rules
- CQL type formatting rules
- Common troubleshooting scenarios
- Migration steps from Python cqlsh

This file enables an LLM to answer any cqlsh-rs question without needing to fetch additional pages.

#### `AGENTS.md`

Instructions for AI coding agents (Copilot, Claude Code, Cursor, etc.):

```markdown
# Agent Instructions for cqlsh-rs

## Project Context
cqlsh-rs is a Rust implementation of the Python cqlsh tool for Apache Cassandra.

## Key Conventions
- Async runtime: Tokio
- Driver: scylla crate
- CLI: clap v4 derive API
- Error handling: anyhow (application) + thiserror (library)
- Testing: cargo test + testcontainers for integration

## When Modifying Code
- Run `cargo clippy` before committing
- Run `cargo test` to verify changes
- Update snapshot tests (`cargo insta review`) if output changes
- New commands go in `src/commands/` with a mod.rs entry
- New CLI flags go in the CliArgs struct in main.rs

## Architecture
[Brief module map and data flow]
```

#### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | Create `llms.txt` with site index | `docs/llm/llms.txt` |
| 2 | Create `llms-full.txt` with comprehensive context | `docs/llm/llms-full.txt` |
| 3 | Create `AGENTS.md` for AI coding assistants | `docs/llm/AGENTS.md` |
| 4 | Auto-generate `llms-full.txt` from mdBook source (build script) | Build integration |
| 5 | Serve `llms.txt` at `/.well-known/llms.txt` on doc site | Deploy config |
| 6 | Add to llmstxt.org directory | External submission |

---

### Area 4: Skills as Public Documentation

#### Concept

Package cqlsh-rs domain expertise as **publicly available AI skills** — reusable knowledge units that any AI assistant can invoke to help users with CQL, Cassandra operations, and cqlsh usage.

#### Skill Definitions

| Skill | Description | Target Audience |
|-------|-------------|----------------|
| **cqlsh-expert** | Deep knowledge of cqlsh commands, flags, configuration, troubleshooting | DBAs, developers using cqlsh daily |
| **cassandra-ops** | Operational expertise: backup, repair, monitoring, performance tuning via cqlsh | DevOps, SREs |
| **schema-design** | CQL data modeling guidance, partition key design, anti-patterns | Application developers |
| **migration-advisor** | Guide users from Python cqlsh to cqlsh-rs, handle edge cases | Teams migrating to cqlsh-rs |
| **cql-tutor** | Interactive CQL teaching, from basics to advanced features | Beginners, students |

#### Skill Format

Each skill is a structured markdown document containing:

```markdown
# Skill: CQL Shell Expert

## Metadata
name: cqlsh-expert
version: 1.0
domain: databases/cassandra
author: cqlsh-rs project
license: MIT
source: https://github.com/fruch/cqlsh-rs

## Description
Expert knowledge of the CQL shell (cqlsh/cqlsh-rs) for Apache Cassandra
and ScyllaDB. Can help with commands, configuration, troubleshooting,
COPY operations, tab completion setup, and migration from Python cqlsh.

## Capabilities
- Explain any cqlsh command with syntax and examples
- Debug connection issues (auth, SSL, timeouts)
- Optimize COPY TO/FROM operations
- Configure cqlshrc for specific environments
- Migrate workflows from Python cqlsh to cqlsh-rs

## Knowledge Base
[Embedded or linked reference material]

## Example Interactions
[Sample prompts and expected responses]
```

#### Skills Marketplace Targets

| Platform | Format | Registration Method | Priority |
|----------|--------|-------------------|----------|
| **Claude MCP Servers** | MCP tool definition (JSON schema) | [modelcontextprotocol.io](https://modelcontextprotocol.io) registry | P1 |
| **GitHub Copilot Extensions** | Copilot Extension manifest | GitHub Marketplace | P1 |
| **OpenAI GPT Store** | GPT configuration + knowledge files | OpenAI GPT builder | P2 |
| **LangChain Hub** | LangChain tool/chain definition | LangChain Hub submission | P2 |
| **Composio** | Tool definition | Composio app registry | P3 |
| **Toolhouse** | Tool definition | Toolhouse registry | P3 |
| **HuggingFace Spaces** | Gradio/Streamlit app | HuggingFace Hub | P3 |

#### MCP Server Implementation

Build a lightweight MCP server that exposes cqlsh-rs knowledge as tools:

```json
{
  "name": "cqlsh-rs-docs",
  "description": "CQL shell expert knowledge and documentation",
  "tools": [
    {
      "name": "cqlsh_command_help",
      "description": "Get detailed help for any cqlsh command",
      "inputSchema": {
        "type": "object",
        "properties": {
          "command": { "type": "string", "description": "Command name (e.g., COPY, DESCRIBE)" }
        }
      }
    },
    {
      "name": "cqlshrc_reference",
      "description": "Look up cqlshrc configuration options",
      "inputSchema": {
        "type": "object",
        "properties": {
          "section": { "type": "string", "description": "Config section (e.g., ssl, connection)" }
        }
      }
    },
    {
      "name": "cql_type_format",
      "description": "How a CQL type is displayed in cqlsh output",
      "inputSchema": {
        "type": "object",
        "properties": {
          "type": { "type": "string", "description": "CQL type (e.g., timestamp, map<text,int>)" }
        }
      }
    },
    {
      "name": "migration_guide",
      "description": "Get migration advice for moving from Python cqlsh to cqlsh-rs",
      "inputSchema": {
        "type": "object",
        "properties": {
          "topic": { "type": "string", "description": "Migration topic (e.g., ssl, copy, config)" }
        }
      }
    }
  ]
}
```

#### Implementation Steps

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | Define skill templates (markdown format) | `docs/llm/skills/template.md` |
| 2 | Write `cqlsh-expert` skill | `docs/llm/skills/cqlsh-expert.md` |
| 3 | Write `cassandra-ops` skill | `docs/llm/skills/cassandra-ops.md` |
| 4 | Write `schema-design` skill | `docs/llm/skills/schema-design.md` |
| 5 | Write `migration-advisor` skill | `docs/llm/skills/migration-advisor.md` |
| 6 | Write `cql-tutor` skill | `docs/llm/skills/cql-tutor.md` |
| 7 | Build MCP server for cqlsh-rs docs | `mcp-server/` |
| 8 | Submit to Claude MCP registry | External |
| 9 | Create GitHub Copilot Extension | `copilot-extension/` |
| 10 | Create OpenAI GPT with knowledge base | External |
| 11 | Submit to LangChain Hub | External |
| 12 | Auto-update skills on doc changes (CI) | `.github/workflows/skills.yml` |

---

### Area 5: Man Page & Shell Completions

| Step | Description | Deliverable |
|------|-------------|-------------|
| 1 | Man page generation from clap (`clap_mangen`) | `docs/cqlsh-rs.1` |
| 2 | Bash completion generation (clap `generate`) | `completions/cqlsh-rs.bash` |
| 3 | Zsh completion generation | `completions/_cqlsh-rs` |
| 4 | Fish completion generation | `completions/cqlsh-rs.fish` |
| 5 | Include completions in release artifacts | Release workflow |

---

## Acceptance Criteria

- [ ] Documentation site builds and deploys on every `main` push
- [ ] PR previews show doc changes before merge
- [ ] Link checking and spell checking pass in CI
- [ ] `llms.txt` and `llms-full.txt` are served at well-known paths
- [ ] `AGENTS.md` is present in repo root or docs
- [ ] At least 3 skills are published to at least 2 marketplaces
- [ ] MCP server is functional and registered
- [ ] Man pages and shell completions are included in release artifacts
- [ ] Documentation covers all commands, flags, and configuration options
- [ ] Migration guide is comprehensive enough for unassisted migration

---

## Estimated Effort

| Area | Effort |
|------|--------|
| Documentation site setup + content | 5 days |
| CI pipeline (build, preview, deploy) | 2 days |
| LLM documentation (llms.txt, AGENTS.md) | 2 days |
| Skill definitions (5 skills) | 3 days |
| MCP server | 3 days |
| Marketplace submissions | 2 days |
| Man pages + shell completions | 1 day |
| **Total** | **18 days** |

---

## Skills Required

- Technical writing (clear, concise documentation)
- mdBook authoring (S11)
- CI/CD for documentation (S11)
- LLM prompt engineering (for skill definitions)
- MCP protocol (for server implementation)
- JSON Schema (for tool definitions)
- Rust `clap_mangen` and `clap_complete` (S5)

---

## Key Decisions

| Decision | Options | Recommendation |
|----------|---------|---------------|
| Doc generator | a) mdBook, b) Zola, c) Docusaurus | (a) mdBook — Rust-native, simple |
| Hosting | a) GitHub Pages, b) Netlify, c) Cloudflare Pages | (a) GitHub Pages (free, integrated) |
| PR previews | a) Netlify deploy previews, b) surge.sh, c) Cloudflare | (a) Netlify (best preview UX) |
| LLM doc format | a) llms.txt only, b) + AGENTS.md, c) + MCP server | (c) All three — different use cases |
| Skills priority | a) All marketplaces at once, b) Claude + Copilot first | (b) Start with Claude MCP + Copilot, expand later |
| Man page gen | a) clap_mangen, b) manual, c) pandoc from md | (a) clap_mangen (auto from clap struct) |
