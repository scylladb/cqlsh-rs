---
description: 'Cross-cutting lessons learned in cqlsh-rs development'
applyTo: '**/*'
---

# General Memory

> Cross-cutting lessons learned in cqlsh-rs development.

## Squash fixup commits into logical units of work

When asked to squash, squash into logical units of work — not just one big commit. Fix-ups should be folded into the relevant development commit in the PR/branch. Each PR should present clean, single-purpose commits — not a trail of fix-ups. Use `git reset --soft` to the base and re-commit, rather than interactive rebase, for simplicity.

## Implement a whole phase at a time

When working on the phased implementation plan (see `docs/plans/high-level-design.md`), implement all remaining items in a phase together rather than cherry-picking individual tasks. This keeps the project moving forward cohesively and avoids partial-phase states that are harder to reason about.
