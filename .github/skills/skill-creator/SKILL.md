---
name: skill-creator
description: >-
  Create new skills, modify and improve existing skills for this project.
  Use when users want to create a skill from scratch, edit or optimize an
  existing skill, understand the skill format, or need guidance on writing
  effective SKILL.md files. Covers both Claude Code and GitHub Copilot
  compatible skill authoring using the Agent Skills open standard.
---

# Skill Creator

You are an expert at creating Agent Skills that work with both **Claude Code** and **GitHub Copilot**. This skill guides you through creating, testing, and refining skills using the [Agent Skills open standard](https://agentskills.io/specification).

## Skill Format Overview

Every skill is a self-contained folder with a `SKILL.md` file and optional bundled resources:

```
skill-name/
├── SKILL.md              # Required: frontmatter + instructions
├── scripts/              # Optional: executable helpers
├── references/           # Optional: docs loaded into context
└── assets/               # Optional: templates, examples
```

### SKILL.md Structure

```markdown
---
name: skill-name
description: >-
  When to trigger and what the skill does. Be specific about
  keywords and scenarios. Make descriptions slightly "pushy"
  to ensure reliable triggering.
---

# Skill Title

Instructions in markdown format...
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Lowercase letters, numbers, hyphens only. Max 64 chars. Becomes the `/slash-command` |
| `description` | Yes | Primary trigger mechanism. Max 1024 chars. Include keywords and scenarios |

**Claude Code additional fields:**
| Field | Description |
|-------|-------------|
| `allowed-tools` | Comma-separated tool list (e.g., `Bash, Read, Write, Edit`) |
| `model` | Override model for this skill |
| `disable-model-invocation` | `true` = only user can invoke (for side-effect workflows) |
| `user-invocable` | `false` = only AI can invoke (for background knowledge) |

## Dual Compatibility (Claude Code + Copilot)

This project stores skills in `.github/skills/` with a symlink at `.claude/skills` for dual compatibility:

- **GitHub Copilot** reads from `.github/skills/<name>/SKILL.md`
- **Claude Code** reads from `.claude/skills/<name>/SKILL.md`
- The symlink ensures both tools discover the same skills

### Writing Compatible Skills

1. Use only standard SKILL.md frontmatter fields (`name`, `description`) for maximum compatibility
2. Add Claude-specific fields only when needed (they are ignored by Copilot)
3. Write instructions in plain markdown — both tools parse it the same way
4. Reference bundled files with relative paths from SKILL.md
5. Keep SKILL.md body under 500 lines for optimal performance

## Creating a New Skill

### Step 1: Capture Intent

Ask and answer these questions:
- What should the skill enable the AI to do?
- When should it trigger? (What keywords/phrases?)
- What is the expected output format?
- Does it need access to specific tools? (Claude Code only)
- Should it be user-invocable, AI-invocable, or both?

### Step 2: Write the Description

The description is the **primary trigger mechanism**. Both Claude and Copilot use it to decide when to activate the skill.

**Good description patterns:**
- Start with the action: "Create...", "Generate...", "Analyze..."
- List trigger scenarios: "Use when asked to..., when users want to..."
- Include relevant keywords that users would naturally type
- Be slightly "pushy" to prevent under-triggering

**Bad description patterns:**
- Vague: "Helps with code" (too broad)
- Narrow: "Writes Python Flask REST APIs" (too specific)
- Missing triggers: "A testing tool" (no actionable keywords)

### Step 3: Write Instructions

Use imperative form in the markdown body:
- "Read the file..." not "The file should be read..."
- "Generate a report..." not "A report will be generated..."
- Define output formats explicitly with templates
- Include realistic examples
- Explain the **why** behind instructions, not just the what

### Step 4: Test the Skill

Create 2-3 realistic test prompts and verify:
1. The skill triggers when it should
2. The skill does NOT trigger when it shouldn't
3. The output matches expectations
4. The instructions are unambiguous

## Progressive Disclosure

Skills use a 3-level loading system for context efficiency:

1. **Metadata** (~100 tokens): `name` + `description` — always in context
2. **SKILL.md body** (variable): loaded when skill triggers — keep under 500 lines
3. **Bundled resources** (unlimited): loaded on-demand via file references

This means you can install many skills without context overhead. Only the ~100-token metadata is always present.

## File Placement in This Project

```
cqlsh-rs/
├── .github/
│   └── skills/           # Canonical skill location (Copilot native)
│       ├── skill-name/
│       │   └── SKILL.md
│       └── ...
└── .claude/
    └── skills -> ../.github/skills   # Symlink for Claude Code
```

## Quality Checklist

Before finalizing a skill, verify:

- [ ] `name` matches directory name, uses lowercase-hyphen format
- [ ] `description` is under 1024 characters and includes trigger keywords
- [ ] SKILL.md body is under 500 lines
- [ ] Instructions use imperative form
- [ ] Output format is explicitly defined (if applicable)
- [ ] Examples are realistic and representative
- [ ] Bundled resources are referenced with relative paths
- [ ] Tested with at least 2-3 representative prompts
