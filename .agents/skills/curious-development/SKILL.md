---
name: curious-development
description: Implement or fix Curious simulation, learning, persistence, and observation features while preserving modular boundaries and deterministic continuation. Use for code changes, not routine runtime checks.
---

# Curious development

Use the checkout containing this file as the project root (three parent
levels above this skill directory). Commands below run from that root.
Read [AGENTS.md](../../../AGENTS.md), [VISION](../../../docs/VISION.md),
[CONTROL](../../../docs/CONTROL.md), [ROADMAP](../../../docs/ROADMAP.md), and
[TODO](../../../TODO.md). Read feature-specific contracts before editing.
Historical notes describe different revisions; inspect current source, manifests,
and service configuration instead of treating the latest paragraph as live state.

## Route the change

Choose one primary roadmap area for the behavior being changed: [K1](../../../docs/ROADMAP.md#dunya-cevre), [K2](../../../docs/ROADMAP.md#canli-beden), [K3](../../../docs/ROADMAP.md#bireysel-ogrenme), [K4](../../../docs/ROADMAP.md#grup-sosyal), [K5](../../../docs/ROADMAP.md#ekoloji-etkilesim), [K6](../../../docs/ROADMAP.md#kalitim-evrim), [K7](../../../docs/ROADMAP.md#gozlem-arayuz), or [K8](../../../docs/ROADMAP.md#motor-isletim). This is categorization, not execution order: name the primary owner and identify cross-area interfaces. Keep world propagation, body/perception, decision, resolver physics, per-individual learning, and persistence responsibilities distinct. A roadmap or skill entry does not authorize a World run or live change.

## Implement the intended behavior

Describe the observable before/after behavior and affected boundary. Keep world
rules, body capabilities, perception, decision, memory, learning, inheritance,
and observation separable. Add a capability's physical cost in the resolver;
a learner may choose an eligible action but cannot bypass its execution rules.

A decision uses the individual's perception and established memory, not hidden
world state, future RNG, or another actor's undisclosed physiology. Individual
experience must not become an age bonus or automatically inherited knowledge.
Resource renewal follows world rules, not population rescue. Do not disguise
extinction by spawning replacements or weakening prey to make a test pass.

For learning changes, read [HUNT_LEARNING](../../../docs/HUNT_LEARNING.md) or
[LEARNING_NEXT](../../../docs/LEARNING_NEXT.md) as applicable. Maintain bounded
memory, strict disabled behavior, explicit frozen-update behavior, and audited
outcomes. Separate missing evidence from failure. Attribute a cost once to the
episode that incurred it; decision-time closure precedes the current resolver tick.

For saved-state changes, inspect checkpoint and SQLite versions independently.
Define defaults/migration, unknown-version rejection, and backward-reader limits.
A successful migration alone does not prove unchanged future behavior.

## Verify in isolation

Use fresh temporary worlds and unused ports, never an existing live data directory.
Tests should reproduce the behavioral failure and test the boundary at risk.
For code changes, follow the repository checks, including:

```sh
cargo test --all-targets
cargo build --release
python3 -m unittest discover -s scripts -p 'test_*.py'
```

For control/persistence changes, use `scripts/check_control.py` against the built
candidate. Test State plus RNG continuation for deterministic/checkpoint claims.
For observer changes, use the relevant Node checks and browser checks when needed;
verify read-only behavior, old/missing fields, and displayed versus actual state.
State exactly which checks ran; documentation-only changes need structural/link
validation, not an unrelated Rust rebuild.

Use a distinct `CARGO_TARGET_DIR` for each worktree and serialize Cargo jobs; a
reused target can contain artifacts from a different source/version. Do not treat
frozen model weights/counters as proof that the entire online World state or RNG
is frozen; exploration may continue. Test the contract actually claimed. Exact
State/RNG continuation is relevant to disabled-mode compatibility and checkpoint
resume.

Finish with behavior, validation, and limitations. Update the maintained contract
and task handoff, commit only intended files, and do not push unless authorized.
At meaningful milestones use the project's configured notification mechanism if
available and authorized; never expose credentials. Clean disposable build output
only after preserving required experiment binaries and confirming it is not in use.
Code completion does not authorize changing a live service or its frozen binary.
