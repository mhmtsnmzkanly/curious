---
name: curious-experiment
description: Design, preregister, run, and interpret bounded Curious experiments with auditable controls, seeds, source identity, and raw evidence. Use for learning, ecology, evolution, or performance claims.
---

# Curious experiment

Resolve the checkout root from this skill directory (three parent levels up).
Read [AGENTS.md](../../../AGENTS.md) and [VISION](../../../docs/VISION.md), then
the relevant maintained protocol and runner. For hunt studies start with
[HUNT_LAB](../../../docs/HUNT_LAB.md), its linked protocols, and existing manifests.
Read operational/control contracts before any run touching persistence or services.

## Freeze the question before seeing results

Specify a narrow falsifiable hypothesis, control and intervention, measured
outcomes, seed roles, body/environment settings, trial reset rules, duration,
resource limit, stopping rules, and analysis windows. Label exploratory diagnosis
separately from acceptance. Commit the protocol and clean candidate source before
registered evaluation; preserve the exact executable, source/scenario/protocol
hashes, command, flags, runner revision, start time, and output location.

Use a new experiment directory. Never overwrite a failed run or retrospectively
change its hypothesis, seed list, threshold, or reward. Consult previous manifests:
previously inspected evaluation seeds are now seen. Different seeds between
versions do not give a paired causal comparison. Repeated fixed/both tests on the
same seeds are not independent extra samples.

For individual learning, compare rule-based, naive-online, trained-online, and the
same trained table with updates frozen when that design answers the question.
Verify equal starting bodies, age, perception, environment and seed schedule;
document RNG divergence caused by different choices. Frozen updates do not imply
frozen exploration. Verify values and visits actually remain unchanged.

## Measure the physical outcome

Use normal World/resolver execution. Disclose scripted prey and body/resource
resets. Record food, movement/attack costs, received damage, deaths, pursuit time,
interruptions, attribution uncertainty, and changes in learned values. Do not use
reward alone as success. Distinguish own prey defense from third-party damage;
retain censorship when evidence is uncertain. A trial boundary is not a natural
failure. Count all episode endings exactly once, including two in one tick.

Validate totals against resolver events and body accounting. Avoid charging
metabolism or action costs twice. Check checkpoint/SQLite samples against raw
records where persistence supports the claim. Old missing audit fields are not
proof of zero cost or success.

## Execute and report honestly

Use the existing bounded runner after inspecting its options. A runtime check does
not authorize a new long sweep; remain within the user's task and resource budget.
Save raw output, stderr, manifest, status, and timestamps even on failure. Stop at
registered rejection or infrastructure failure; do not automatically tune parameters
or deploy the candidate. Resume only if source and protocol identity still match.

Report per-seed results and variation as well as aggregates, negative effects,
missing evidence, and which hypothesis was tested. Laboratory body resets do not
prove lifetime experience. Tick counts do not prove real days of uptime; generations
do not prove intelligence or adaptive evolution. Population survival alone does
not prove descendant reproduction or open-ended evolution. Treat a passed learning
lab as separate from mixed-ecosystem qualification.

Write a durable report linked to immutable evidence, note which seeds are now seen,
commit the report, and notify the user at meaningful outcomes through an available,
authorized project channel. Never delete raw evidence during build cleanup.
