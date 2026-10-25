---
name: curious-documentation
description: Create, update, reorganize, or prune Curious project documentation and keep current-state and roadmap links accurate. Use for documentation work, not engine changes, experiments, or live operations.
---

# Curious documentation

Resolve the repository root three parent levels above this skill directory. Read
[AGENTS.md](../../../AGENTS.md), the [documentation standard](../../../docs/DOCUMENTATION_STANDARD.md),
and [docs/README.md](../../../docs/README.md). Then read the current target and
only the feature contract, evidence report, or source needed to verify its claims.

## Choose the document role

- Use maintained contracts for current behavior, design boundaries, or operating
  procedures; use the roadmap for categorized future work and the task list for
  ordered priorities.
- Keep frozen protocols and dated reports as immutable evidence. Do not turn a
  report into a current guarantee or silently edit a failed result. Correct a
  historical error in a new, linked note or report.
- Route work to one primary K1–K8 roadmap category and note material cross-area
  dependencies. Categories are not P0–P3 execution order and confer no authority
  to change code, run a World, deploy, or contact others.

## Write and verify

Check current source, manifests, or actual runtime evidence before stating a
present fact; dated snapshots remain dated. Distinguish proposed, implemented,
tested, accepted, and deployed. Link detailed evidence instead of duplicating it.
Keep outcomes in physical units and at their stated horizon; preserve Unknown,
censoring, support counts, and the limits of the measurement. Write maintained
project guidance in concise Turkish; keep portable skill instructions in English
and preserve the language of historical reports.

Update an index, README, task list, or roadmap only when the document change
actually affects it. Prune only within the user's authorized scope, after moving
live guidance to its canonical home and retaining historical evidence and Git
history. Do not remove protocols, hashes, raw data, manifests, or original
failure records as routine cleanup.

For documentation-only work, check changed relative links and anchors, confirm
archive-backed evidence points to its index/restore instructions, and run
`git diff --check`. Do not run Rust builds, tests, or experiments just to validate
prose. For skill edits, validate frontmatter, names, and reference paths with an
available validator or equivalent checks; no provider-specific tool is required.
State which checks ran and which claims remain unverified. Existing user
authorization persists within its stated scope; do not invent extra approval
steps, and do not treat this skill as permission for external actions.
