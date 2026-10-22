# Hunt laboratory v2 — preregistered before evaluation

This protocol evaluates only the bounded hunt learner's feedback attribution
repair. It preserves the V1 bodies, map, behavior, 20 tick episode limit, reward
coefficients, 10% exploration, arm layout, training/evaluation lengths, reversal,
and controlled trial reset. It does not test survival, reproduction, evolution,
or ecosystem balance.

## Fixed seeds and execution

The eight training seeds remain 1, 2, 3, 4, 5, 42, 12345, and 2026. New
evaluation seeds are fixed to 42001–42008, paired in that order; they do not
overlap V1 evaluation seeds 41001–41008. Both tracks use these pairs. The
runner's `--v2` flag selects this file and these seeds. It records source,
binary, and protocol hashes in a fresh output directory; V1 output is never
reused or overwritten. The Rust example accepts start seed 42001 and emits
schema `curious-hunt-lab-v2`; the validator checks seed, schema, and damage
accounting.

## Frozen interpretation

An episode ending with `Interrupted` because its still-visible target fails the
current policy's exact energy gate (`HuntLowEnergy` or `HuntTooCostly`) may use a
negative reward only when prior resolver events recorded movement or attack
energy, the episode has no external/ambiguous injury, and it has not already
killed a target awaiting a meal. The reward uses only already recorded costs
and ticks, with the existing coefficients. The cutoff decision's current tick
is not added to the old episode. A missing or vanished target, a different
decision priority, insufficient stamina, target switching, malformed damage
attribution, or a cutoff with no audited action cost remains unscored. Target
switching ends the old episode once and transfers none of its costs to the new
target.

Incoming damage from the current episode target is prey defense; damage from
another attacker is external. If no episode target can be identified in the lab
audit, damage is unattributed. A pre-resolution `Interrupted` result does not
identify that tick's active target, so damage in that tick remains unattributed
unless a new active episode identifies it. The three categories must sum to
total damage.
Any external or unattributed damage, or a mismatch between totals and their
source breakdown, keeps the learner result unscored. Prey defense receives no
learned cost because the experiment has not established a causal coefficient.

The four arms, first/last training halves, normal/reversed evaluation windows,
and body/energy audits are computed as in V1. V2 additionally checks and records
per-tick and per-trial prey-defense, external, and unattributed-damage totals. No
outcome-based seed, threshold, coefficient, or protocol changes are allowed.
