---
name: curious-review
description: Review Curious changes and simulation claims for architectural boundaries, credible behavior, causal attribution, reproducibility, and evidence quality. Use for requested code, experiment, or design review.
---

# Curious review

Resolve the repository root three parent levels above this skill directory. Read
[AGENTS.md](../../../AGENTS.md), [VISION](../../../docs/VISION.md), and the contract
for the affected feature. Identify the requested review scope, base/candidate
revisions, and actual diff. A review request alone does not authorize editing code,
launching a long experiment, or touching a live world.

Route the review by the affected roadmap area, then trace cross-area effects where
they matter: [K1](../../../docs/ROADMAP.md#dunya-cevre),
[K2](../../../docs/ROADMAP.md#canli-beden),
[K3](../../../docs/ROADMAP.md#bireysel-ogrenme),
[K4](../../../docs/ROADMAP.md#grup-sosyal),
[K5](../../../docs/ROADMAP.md#ekoloji-etkilesim),
[K6](../../../docs/ROADMAP.md#kalitim-evrim),
[K7](../../../docs/ROADMAP.md#gozlem-arayuz), and
[K8](../../../docs/ROADMAP.md#motor-isletim). Treat these as categories, not a
feature sequence. Check whether an outcome is supported at the declared horizon
and by the existing denominators; preserve established Unknown/censor rules
instead of inventing a new acceptance threshold during review.

## Trace intent to evidence

State what user-visible behavior is intended and trace it through perception,
memory/decision, intent, resolver validation, event attribution, learning update,
persistence, and observer display. Separate a deliberate design tradeoff from an
implementation bug. Prioritize failures that change simulation behavior or invalidate
a claim; avoid speculative generic checklists and cosmetic findings.

Check the boundaries relevant to the change:

- Decision input must be observable or established memory; no future RNG, hidden
  target physiology, or omniscient world lookup. The resolver owns physics and cost.
- Features need paid physical consequences. No age-based intelligence bonus,
  accidental learned-table inheritance, or population rescue disguised as ecology.
- Disabled learning must preserve its documented state/RNG behavior. Frozen values
  and visits stay fixed while the configured choice/exploration policy continues;
  full online/frozen State/RNG identity is not required. Verify the freeze contract
  separately from disabled-mode compatibility and deterministic checkpoint resume.
- Follow a whole episode across target switch, kill, later meal, priority change,
  external command, third-party interaction, death, and timeout. Account costs once;
  predecision closure cannot consume the current resolver tick. Killing is not food.
- Attribution must identify attacker, target and corpse where required. Missing
  evidence is not failure. Own prey defense is not third-party attack; a visible
  budget cutoff is not the same as an arbitrary interruption. Verify every ending,
  including two endings in one tick, reaches the audit stream exactly once.
- Saved state and RNG must resume deterministically for the claimed contract.
  Check migration/defaults, invalid/future versions, bounded memory, and the limits
  of older readers. Checkpoint and SQLite versions are separate interfaces.
- Observers must represent recorded facts, sparse/old fields, and uncertainty.
  A read-only viewer must not mutate the world; replay controls are not world controls.

## Audit the experiment behind a claim

Inspect raw data and manifests rather than trusting the summary. Confirm frozen
source/binary/protocol identity, declared flags, clean source, seed roles, stopping
rules, and equal controls. Recompute the small decisive metric. Look for training
leakage, changed evaluation seeds presented as paired evidence, reset bodies
presented as aging experience, and reward improvement presented as ecology.
Check births and reproducing descendants when continuity is claimed. Wall-clock
soak evidence is separate from accelerated ticks and laboratory success.

Use a minimal isolated reproduction or a relevant existing test where it can
resolve uncertainty. Never run tests against live data. Request independent review
or delegate only if the user/task or applicable instructions authorize it; no
particular model, provider, or multi-agent infrastructure is required.

## Deliver actionable findings

For each concrete finding give severity, file/location, triggering conditions,
actual versus expected effect, and supporting evidence or a minimal reproduction.
Separate confirmed defects, untested risks, and scientific limitations. If no
blocking defect is found, say so and retain material validation gaps. Do not call a
candidate deployable merely because unit tests pass. If fixes are authorized,
verify the correction against the failure and report checks actually performed.
