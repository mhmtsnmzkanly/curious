# Refuge first-resolution extraction protocol

**Status: approved extraction scope; real-capture execution is pending root
review and a freeze commit.** This protocol specifies a bounded, stdlib-only,
read-only extractor for the already-recorded engineering capture. It does not
specify or authorize a predictor, model update, World run, live operation, or
route ranking.

## Frozen input identity

The only accepted raw input is
`data/experiments/refuge-risk-baseline-v1-engineering-20261025/raw.jsonl`,
SHA-256 `d9c7e00e976c2fe31e0ad07868df924cabd423b91c6f5c32540b4b2cd43b4874`.
The source manifest SHA-256 is
`b5161962b0939b6389f07f64d19458572703e55b721a3383b7eaf0202b2aef12`; the
independent audit SHA-256 is
`8e22728c1347a7b2fe91bd7c322591cf99424a999d6597e76a2c4361d55e00de`.
Manifest identity is experiment source `0377d4c66d4ea81bf1ea83e0c5305c00849f963d`,
engine source `117a440431deead231ac6d4edbe324be2345936a`, seed 123, and the
manifest's `scenario.sha256` for `mixed-refuge.json`:
`9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2`.
Any mismatch, duplicate provenance, or ambiguous global resolver event fails
closed.

The extractor accepts a required reviewed commit argument and requires that
commit to match a clean worktree. It hashes raw, manifest, independent audit,
protocol, and code before reading and again before publishing. The output must
be a new direct child of
`data/experiments/refuge-first-resolution-v1-20261025/`; existing paths and
symlinks are refused. Output is staged then atomically published. The bounded
input/output limits are 128 MiB / 64 MiB and two million records.

## Target and interval join

Emit one row for every `episode_start`. Join its closure by the complete start
payload, ignoring only the top-level `record_type` tag that the nested closure
copy omits. Join interval rows by
`(owner_id, tick - elapsed_ticks + 1)` and require the referenced start to
exist. The first target interval must have matching threat provenance,
`tick == started_tick`, and `elapsed_ticks == 1`; later rows map to the
continuing start and are counted as excluded, not repurposed as fresh
observations for a new start. This avoids borrowing an older attempt's
continuation at a same-owner decision boundary. A missing first interval
produces an `Unknown` row, never a zero.

For a complete first interval, the target is the sum of resolver-witnessed
damage from the threat selected at choice time to the episode owner. Validate
the filtered global resolver witness against the world tick's total event
count using ordered, unique, in-range event indexes. The tick count covers all
simulation events while the global witness filters event kinds, so witness
length need not equal tick count. Normalize the local
`attack_events` (`attacker_id`, `target_id`, `damage`) and global `attacked`
events (`attacker`, `target`, `damage`) to verify they agree. The global
witness is complete only when the matching tick record and unique resolver
event record exist and reconcile. A missing witness yields `Unknown`; duplicate
or inconsistent witnesses abort the extraction.

The only emitted outcome statuses are:

- `Exact`: complete first interval and consistent global/local evidence,
  including exact zero when the complete witness contains no selected-threat
  damage.
- `Unknown`: missing/ambiguous first interval or absent global witness.

No interval-valued outcome is emitted. Partial damage is never changed to zero;
bounded interval outcomes remain a future design question requiring a separate
review. Damage from other attackers and unexplained health loss are retained as
separate exact fields when present. Recompute unexplained health loss as
`max(0, health_before - health_after - selected_damage - other_damage)` and
require it to match the recorded cumulative field; also require start body
health to match the first interval's before-health. These components neither
overwrite nor invalidate known selected-threat damage. A selected-threat zero
is not a safety label.

## Features, metadata, and limits

Features are restricted to choice-time data: subject body state (excluding
entity ID), start position, decision reason, selected intent prefix, and the
selected route's endpoint/path/turns/route-cell observations and selected
threat observations. Unselected routes are not emitted. Owner ID, threat ID,
start tick, resolver tick, and closure reason are provenance metadata only.
Closure/final outcomes, future movement, later attack exposure, and all other
post-choice values are excluded from features.

The expected source support is 623 starts, 623 closures, and 631 interval rows.
The first interval has 617 exact zeros and six positive outcomes of 20 damage
each (120 total), across four owners. The eight later interval rows lack a
fresh choice-time perception and are excluded. Owner counts are descriptive
metadata only; the extractor fits no model and writes no model state. These
single-run seen-seed observations are not held-out validation and do not
calibrate full-escape damage, unselected-route risk, safety, utility, or
biology. The existing route-level damage remains `Unknown`.

The run directory contains `rows.jsonl`, `summary.json`, and `provenance.json`.
Provenance records the reviewed execution commit; source experiment/engine
revisions; raw, manifest, audit, protocol, and code hashes before/after; and
output artifact hashes. The summary records Exact/Unknown counts, per-owner
support metadata, excluded continuations, and confirms zero model fitting/state
and zero World ticks. The extractor is not to be run on the real raw capture
until root reviews the implementation and commits the frozen code and
protocol. Any predictor, per-owner learning rule, interval-censoring method,
or output use requires separate review; this protocol defines no model or
training/evaluation thresholds.
