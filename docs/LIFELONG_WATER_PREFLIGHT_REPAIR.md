# Lifetime water formal preflight repair

## Incident

The first formal attempt for the continuous-lifetime water lab stopped in
preflight at `2026-10-23 19:36 UTC`. It stopped before creating the formal output
directory, launching the probe, or advancing a world tick. The registered
formal seeds remain unused.

The seed freshness scanner rejected the immutable calibration `raw.jsonl`
because its valid single JSONL line is 19,443,712 bytes, larger than the
scanner's generic 16 MiB line limit. The lifetime protocol permits up to 64 MiB
per seed record, and the calibration run manifest records the same 64 MiB
`max_seed_record_bytes` bound. The scanner therefore used a narrower bound than
the preregistered run contract; this is a preflight validation defect. The
calibration line must be parsed whole under the registered limit, not skipped,
truncated, or treated as evidence that a seed was unused.

## Repair and reanalysis

Repair the shared seed scanner to accept an explicit per-call line bound while
keeping its existing 16 MiB default for other callers. The scanner reads bounded
binary lines with a `limit + 1` sentinel for both text and CSV inputs, so an
oversized line is detected and fails closed rather than being truncated. This
lab's complete freshness scan explicitly uses the registered 64 MiB per-seed
record limit, parses every available manifest and raw record, and fails closed
on parse errors. Do not change the calibration or formal seed sets, fixture,
biology, gates, or budgets.

Because the shared helper and source identity change, the previous calibration
reanalysis is not sufficient as the formal candidate's source-bound evidence.
Root will rerun validation against the original immutable calibration capture
with the updated scanner/runner, using zero world ticks, in
`data/experiments/lifetime-lab-v2-calibration-reanalysis2-20261023/`. Preserve
the original calibration run and the first reanalysis directory unchanged.
Formal preflight must bind to this second reanalysis and recheck the seed list
against the entire 64 MiB scan.

The first formal job stopped before any formal capture. After the corrected
source-bound reanalysis and full freshness scan passed, the bounded formal run
completed on the original held-out set. The final result and its limits are in
[`reports/LIFELONG_WATER_V2.md`](reports/LIFELONG_WATER_V2.md). The first
preflight failure consumed no formal seeds or simulation ticks.
