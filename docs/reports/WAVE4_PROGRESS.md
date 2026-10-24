# Wave 4 progress report

**Status: source and diagnosis stage complete; refuge engineering remains
incomplete.** One bounded engineering World run failed on an allocation error;
its retained output passed a zero-tick memory replay. No formal seed was started,
no world was redeployed, and the live world remains frozen at V12. Current source
validation, evidence archiving, and cleanup facts are recorded below.

## G — archive budget accounting

The persistence candidate was committed as `8f3f6d0` and integrated in main
`42fc7f6`. Before rejecting a write on the cached closed-shard total, the Store
recounts the selected `Store.files()` inventory using filesystem metadata only.
It does not read archive contents. A shard that disappears during the stat pass
gets one fresh inventory pass; persistent stat/enumeration errors fail closed.
Checked byte accumulation and the existing budget stop remain in place. The
active SQLite/WAL stays protected by the writer lock, and the newest shard
retention rule is unchanged.

Root reports 235 all-target Rust tests (45 summaries) and 177 Python tests, with
zero failures; release application/probe/replay builds and
HTTP/auth/writer-lock/SIGKILL checks passed. The candidate application is retained
at `bin/releases/curious-budget-8f3f6d0`. This is a soft history guard, not a hard
disk quota: WAL and temporary archive space are outside the estimate, and
externally added files are not continuously detected. No live deployment was
made; the running world remains V12 with its prior in-memory accounting.

## C — habitat and trait selection protocol

Commit `1a074ba`, integrated in main `935015c`, adds
`docs/HABITAT_SELECTION_PROTOCOL.md` as a **design draft only**. It proposes one
accepted habitat profile and one allele contrast: expressed vision allele versus
the same inherited allele as a neutral marker. No seeds are assigned, no
calibration or World run occurred, and the protocol is not preregistered.

The expressed-feature toggle consumes extra RNG during reproduction, so later
random streams and world trajectories can diverge between arms. That limits the
contrast to an expressed-feature-associated result in this implementation; it
does not isolate the pure vision mechanism. Genotype-linked food-discovery and
energy-cost measurement is also not yet implemented. The protocol needs root
review and a measurement plan before any run is considered.

## B — refuge escape engineering run

The single permitted engineering fixture used already-seen seed 123 and focal
herbivore 15. The immutable evidence is preserved under
`data/experiments/refuge-escape-v1-engineering-20261024/`; see its `manifest.json`,
`raw.jsonl.archive.json`, `stderr.log`, and `root-partial-audit.json`. The raw
JSONL was losslessly archived as `raw.jsonl.zst`; restore it with the evidence
helper before using readers that need the original path. The executed source
revision was `5ae985e4df219b2fae17d9e1497d7bc4774f1008`; the failed manifest
remains `failed` and binds binary, scenario, source-file, and protocol hashes.

The invocation planned 270 physical ticks: a 30-tick shared prefix and four
60-tick branches. It exited with allocation failure under the 1 GiB address-space
limit after 2.912 seconds. The raw file contains 49,022,469 bytes and the partial
audit contains 225 complete records; the possible number of physical ticks
executed is 225–226. The shared prefix and trained-online, trained-frozen, and
naive-online branches completed. The rule branch stopped after 15 of 60 ticks.
This is an **infrastructure-incomplete** run, not a biological failure.

The partial audit confirms continuous focal age, matching recorded provenance
replays, only declared fork differences, and unchanged frozen rows for the
recorded portion. Its physical-state FNV chain is not an independent RNG
transition proof. Online, frozen, and naive branches each recorded 60 suffix
ticks and ended with the same summarized vector: health 120, energy 34, water
27, movement energy 36, 23 movement events, 92 food, and 56 water acquired. Each
recorded one arrival; online and naive recorded one update, frozen recorded zero.
These are descriptive partial-run outcomes, not evidence of benefit or equality
outside that summary.

There were **zero supported greedy choices** in the completed online branch, so
the run did not identify learned refuge choice. It supports no utility,
usefulness, ineffectiveness, or biological rejection claim. No formal run began
and no registered seed role was consumed. Preserve the failed invocation and
partial raw evidence; do not rerun another seed or extend this run.

## Memory repair and zero-tick diagnosis

The observer memory repair is source `3c2a83d`, integrated in main `e551263`.
The previous design was `6a50312` / main `b193d27`; the executed engineering
source remains `5ae985e` / main `cf2c871`. The repair stores canonical normalized
pre-state bytes for the online/frozen comparison instead of retaining full
values for all four arms. Six pure tests passed, and release lab/replay binaries
compiled from the repair source. The hash-bound record is
`data/experiments/refuge-escape-v1-engineering-20261024/memory-fix-validation.json`.

A standalone zero-tick replay completed in 0.91 seconds under the 1 GiB cap, with
40,820 KiB peak RSS. It read the retained 225-record, 49,022,469-byte artifact,
stored 120 individual pre-states forming 60 online/frozen pairs using 26,294,126
canonical bytes (largest state 240,987 bytes), and found 14 of 60 full-state
pairs equal after normalization. It executed zero World ticks. The replay is
not a rerun of the probe, not an independent physics audit, and not a completed 270-tick
experiment: `run_complete` is false, no summary was emitted, and the rule arm
still has only 15 of 60 records. Memory behavior is validated on this recorded
workload only.

The separate zero-tick key diagnosis found only observed shelter threat ID 37.
The only recorded post-prefix online shelter choice was at tick 56, also under
threat ID 37, with four candidates and one stored row from tick 7. Each candidate
differed from that row in eight or nine key fields, including threat health,
power, bearing, distance, subject energy, endpoint, route, turns, and sometimes
estimated movement energy. This is descriptive support diagnosis, not an
ablation or causal explanation. There was no observed cross-ID exposure, so the
run cannot establish that cross-ID generalization failed or that exact-key
strictness alone caused the absence of supported choices.

## Final validation and cleanup

At main `e551263`, root passed 235 Rust tests across all targets (45 summaries)
and 177 Python tests, both with zero failures. Release app, probe, and replay
builds passed, as did HTTP/auth/writer-lock/SIGKILL checks. Six pure observer
tests passed with source `3c2a83d`.

The raw JSONL archive is 2,723,477 bytes versus 49,022,469 original bytes.
Original SHA-256 is
`15ce046035061e24fbd969cc9056ae32cc8f423926705b5e168c55a0a830c4a0`; archive
SHA-256 is `4b4c9bb594d1a9118ff2a74338bdb8a629928d786032f501d89b12dbe02c0464`.
The verified per-file index is
`data/experiments/refuge-escape-v1-engineering-20261024/raw.jsonl.archive.json`.
The original failed manifest remains unchanged. The protocol snapshot, executed
source `5ae985e`, and three binaries were preserved. Replay/source/raw/audit hash
bindings are recorded in
`data/experiments/refuge-escape-v1-engineering-20261024/memory-fix-validation.json`.

Cleanup removed 984,738,373 logical bytes (about 939.12 MiB) from three inactive
build caches. Proof: `data/operations/wave4-cleanup-20261024.json`. Services
7882/7883 and the retention timer remain active and unchanged; no Wave 4
biology/ML/formal worker or wake is pending.

## Next work

Source and diagnosis work is complete, but refuge engineering is incomplete and
does not establish benefit, ineffectiveness, or biological rejection. The only
World run remains the failed 225-record run; offline replay neither reran it nor
advanced a World. Before collecting further refuge data, define a protocol that
can expose a relevant context opportunity. Before a habitat/trait-selection run,
implement genotype-linked food and cost audits. Do not upgrade the engine without
qualification. Live V12 remains unchanged.
