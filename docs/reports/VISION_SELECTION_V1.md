# Vision selection V1 — four-pair descriptive study

**Status: all four formal seed pairs completed and passed independent audits.** This report records a bounded descriptive contrast for the accepted fixture; all raw captures are losslessly archived and verified.

## Question and directional prediction

The study asks whether expressed vision-1 changes its frequency among living
herbivore descendants that themselves reproduced at tick 500, relative to the
same inherited alleles acting as neutral markers, in the accepted qualification
fixture. The original positive directional prediction was that vision-1 could
gain representation when increased range helps locate food enough to repay its
maintenance cost. Results must be reported direction-neutrally, including
negative, null, fixed, mixed, and non-estimable outcomes.

For each paired seed, the preregistered descriptive contrast is
`D = (p_breeder,expressed - p_founder,expressed) -
(p_breeder,neutral - p_founder,neutral)`. Each frequency uses the number of
vision-1 founders or living descendant breeders divided by the corresponding
total. `NA` applies only when a breeder denominator is zero; frequency 0 or 1
with a positive denominator is valid. Individual breeders and families are
not independent replicates; the paired seed is the replication unit.

## Calibration: seen seed 123

The calibration ran once for 500 ticks per arm (1,000 physical ticks total)
on source revision `a4232c37d52d50d7e7dccf3b429be03fe1519292`. It used the
accepted scenario with SHA-256
`33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac`, mutation
zero, identical assigned founder genomes, and expressed versus neutral-marker
trait settings. The initial herbivore cohort was exactly 288 vision-0 and 288
vision-1 founders in each arm. The supplemental audit independently matched
founder IDs, positions, genomes, and endpoint denominators.

| Frozen input | SHA-256 |
| --- | --- |
| Source revision | `a4232c37d52d50d7e7dccf3b429be03fe1519292` |
| Source tree | `eb3442162129d32690dbc685c5c77ae454e89b3a` |
| Probe binary | `bca090e47482465f792958a36ce4019ebba96238b5947c3b49e31be91d2fa05d` |
| Protocol | `6b0dcbd319e41ac4ca8e2ad11a770406dcba712093db585b3c23f84afd82127b` |
| Scenario | `33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac` |
| Formal seed inventory | `71e421a91eeb3163ae17b6db160083bf36f0637cb557c22ee196d19ecd1da579` |
| Runner | `706350504614f973bab1dfc301b6e20634f103f24c32e553d23e118f72f3cc34` |
| Independent audit script | `addd465ae10bc3fc59797d4e9e5b70240f04bd24cc0495912c2abdf4e8143db0` |

| Arm | Living H/C at 500 | Births | Deaths | Living descendant breeder counts, vision 0/1 | Vision-1 breeder share |
| --- | ---: | ---: | ---: | ---: | ---: |
| Expressed | 578 / 49 | 2,891 | 2,872 | 386 / 4 (390 total) | 1.03% |
| Neutral marker | 591 / 47 | 2,902 | 2,872 | 191 / 212 (403 total) | 52.61% |

Both arms began at vision-1 founder frequency 0.5. The expressed arm changed
from 0.5 to `4/390 = 0.010256`; the neutral arm changed from 0.5 to
`212/403 = 0.526055`. Calibration `D = -0.515798` (−51.58 percentage points),
opposite the original positive directional prediction. This is a calibration
observation, not a formal result or a reason to tune or replace seeds.

Both species had living descendant breeders in both arms at tick 500 (14
carnivore descendant breeders per arm), so the preregistered biological gate
passed. The expressed arm retained only four vision-1 living descendant
breeders; low representation is observed, not a measurement failure. Family
support was strongly uneven: expressed vision-0/vision-1 breeders represented
383/4 distinct immediate parent pairs and 355/301 distinct founder ancestors;
neutral breeders represented 191/211 parent pairs and 373/374 founder
ancestors. Founder-ancestor unions can overlap and are not additive. Family
counts are dependence diagnostics, not extra replicates.

### Genotype exposure and acquired resources

The denominator is living genotype-specific individual-ticks counted from
each exact pre-tick State (`life.is_alive()` and phase not `Corpse` or
`Removed`, including sleeping individuals). Costs and actual acquisitions below
are normalized by that denominator. Plant/corpse encounter entries are
perceptions, not discoveries or meals; missing trace coverage remains unknown.

| Arm/genotype | Living individual-ticks | Vision requested / paid / unpaid per individual-tick | Plant food per individual-tick | Water per individual-tick | Trace ticks / missing | Plant / corpse encounters |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Expressed v0 | 237,660 | 0 / 0 / 0 | 2.42490 | 1.23546 | 235,586 / 13,921 | 4,670,127 / 176,441 |
| Expressed v1 | 61,174 | 0.99073 / 0.92427 / 0.06647 | 2.68969 | 0.98071 | 60,438 / 5,197 | 1,542,196 / 61,790 |
| Neutral v0 | 143,684 | 0 / 0 / 0 | 2.38145 | 1.18325 | 142,351 / 9,167 | 2,798,852 / 107,096 |
| Neutral v1 | 158,766 | 0 / 0 / 0 | 2.37462 | 1.18215 | 157,297 / 10,004 | 3,100,936 / 124,024 |

Maintenance totals for expressed v1 were 60,607 requested, 56,541 paid, and
4,066 unpaid. Unpaid maintenance is a valid shortfall; payment above request
would be an audit issue. Basal metabolism is not inferred from these trait
costs or from body-energy residuals.

## Calibration audit and feasibility gate

The root audit independently recomputed the event, lineage, exposure,
founder-ancestry, breeder-endpoint, and per-tick food/water accounting for all
1,000 ticks. It verified initial arms equal except declared expression, fixed
frame event counts at ticks 1, 100, 200, and 500, and tick-100 State+RNG
serialize/restore equality for both arms. The restore check is not an
uninterrupted-path comparison or RNG transition replay. The audit scope is not
full replay or food-discovery validation. No formal fitness claim follows.

| Capture/budget measure | Observed or gate forecast | Limit / status |
| --- | ---: | --- |
| Elapsed time | 61.462 s | 300 s calibration cap |
| Peak RSS | 476,664 KiB | 1 GiB address-space cap |
| Raw JSONL | 156,261,209 bytes | 240 MiB |
| Aggregate calibration output | 159,563,648 bytes | 256 MiB |
| Forecast raw per formal pair, 20% margin | 187,513,451 bytes | 240 MiB |
| Forecast four-pair raw, 20% margin | 750,053,803 bytes | Included in aggregate |
| Forecast metadata plus four pairs | 764,312,135 bytes | 1 GiB formal aggregate |
| Forecast formal wall, including 180 s compile/audit allowance | 548.772 s | 900 s |

The calibration measurement, biological gate, and formal feasibility gate
passed. The four formal seeds `54001`, `54002`, `54003`, and `54004` were
registered before execution and each paired seed ran once in the
specified sequential order. All four formal roles are complete and both root
audit layers passed for each pair.

## Formal batch: four registered seed pairs

The four paired seed blocks are the replication units. All four formal pairs
were estimable and independently audited. Each arm began at vision-1 frequency
0.5; all deltas below retain the founder subtraction. The negative values are
opposite the original positive prediction.

| Seed | Expressed vision-1 / living descendant breeders | Neutral vision-1 / living descendant breeders | Expressed `p500` | Neutral `p500` | `D` |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 54001 | 0 / 433 | 246 / 408 | 0.000000 | 0.602941 | −0.602941 |
| 54002 | 6 / 375 | 193 / 400 | 0.016000 | 0.482500 | −0.466500 |
| 54003 | 4 / 403 | 202 / 396 | 0.009926 | 0.510101 | −0.500175 |
| 54004 | 5 / 399 | 165 / 387 | 0.012531 | 0.426357 | −0.413825 |

Descriptive across-seed summary: `N=4` paired seed blocks, mean `D=-0.495860`,
median `D=-0.483338`, range `[-0.602941, -0.413825]`. This is not a
significance test. In seed 54001 the expressed vision-1 breeder cohort had
frequency zero with a positive denominator (0/433), a valid breeder-cohort
endpoint. It does not mean vision-1 was absent from all living herbivores:
one vision-1 herbivore remained alive in the expressed arm. Report all-living
allele counts alongside breeder counts:

| Seed | Expressed living H v0/v1 | Neutral living H v0/v1 | Expressed living descendant breeders v0/v1 | Neutral living descendant breeders v0/v1 |
| ---: | ---: | ---: | ---: | ---: |
| 54001 | 600 / 1 | 227 / 354 | 433 / 0 | 162 / 246 |
| 54002 | 557 / 10 | 297 / 287 | 369 / 6 | 207 / 193 |
| 54003 | 578 / 6 | 277 / 299 | 399 / 4 | 194 / 202 |
| 54004 | 567 / 11 | 328 / 244 | 394 / 5 | 222 / 165 |

Both species had living descendant breeders in every formal arm. Each formal
pair retained its biological result; no seed was replaced or rerun.

### Family dependence diagnostics

Counts are expressed/neutral, vision-0/vision-1 order. Parent-pair family
counts are distinct unordered immediate parent IDs among living descendant
breeders. Founder counts are distinct original herbivore ancestors represented
by those breeders; allele-specific founder unions can overlap and are not
additive. Neither measure adds independent replication units.

| Seed | Distinct parent pairs E v0/v1; N v0/v1 | Founder ancestor union E v0/v1; N v0/v1 |
| ---: | --- | --- |
| 54001 | E 432/0; N 162/246 | E 352/0; N 399/395 |
| 54002 | E 365/6; N 205/192 | E 371/299; N 384/388 |
| 54003 | E 396/4; N 194/199 | E 369/276; N 411/411 |
| 54004 | E 393/5; N 222/165 | E 368/303; N 383/383 |

### Genotype exposure, costs, and acquisitions

The exposure denominator is living genotype-specific individual-ticks from
each exact pre-tick State: `life.is_alive()` and phase not `Corpse` or
`Removed`, including sleeping individuals. Values below are per living
individual-tick. Plant/corpse encounters are recorded perceptions, not
discoveries or meals. Missing trace counts remain unknown coverage.

| Seed | Arm/genotype | Living individual-ticks | Vision request / paid / unpaid per tick | Plant food per tick | Water per tick | Trace ticks / missing |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 54001 | E v0 | 252,853 | 0 / 0 / 0 | 2.43827 | 1.23277 | 250,641 / 15,039 |
| 54001 | E v1 | 47,202 | 0.99013 / 0.91903 / 0.07110 | 2.58169 | 0.93026 | 46,608 / 4,268 |
| 54001 | N v0 | 137,856 | 0 / 0 / 0 | 2.34509 | 1.16878 | 136,543 / 8,974 |
| 54001 | N v1 | 166,303 | 0 / 0 / 0 | 2.38414 | 1.19093 | 164,778 / 10,344 |
| 54002 | E v0 | 244,999 | 0 / 0 / 0 | 2.42713 | 1.23223 | 242,851 / 14,451 |
| 54002 | E v1 | 54,216 | 0.99033 / 0.92502 / 0.06531 | 2.67807 | 0.97451 | 53,555 / 4,745 |
| 54002 | N v0 | 157,899 | 0 / 0 / 0 | 2.35776 | 1.17682 | 156,425 / 10,142 |
| 54002 | N v1 | 145,948 | 0 / 0 / 0 | 2.37959 | 1.18742 | 144,593 / 9,460 |
| 54003 | E v0 | 240,617 | 0 / 0 / 0 | 2.44729 | 1.24493 | 238,521 / 14,075 |
| 54003 | E v1 | 56,101 | 0.99059 / 0.92439 / 0.06620 | 2.65719 | 0.96922 | 55,421 / 4,915 |
| 54003 | N v0 | 144,634 | 0 / 0 / 0 | 2.37589 | 1.18543 | 143,291 / 9,235 |
| 54003 | N v1 | 158,494 | 0 / 0 / 0 | 2.37721 | 1.18452 | 157,016 / 10,103 |
| 54004 | E v0 | 247,713 | 0 / 0 / 0 | 2.42053 | 1.22823 | 245,545 / 14,468 |
| 54004 | E v1 | 52,114 | 0.99056 / 0.92246 / 0.06810 | 2.64027 | 0.95613 | 51,472 / 4,635 |
| 54004 | N v0 | 165,380 | 0 / 0 / 0 | 2.36428 | 1.18065 | 163,869 / 10,374 |
| 54004 | N v1 | 141,557 | 0 / 0 / 0 | 2.34082 | 1.17006 | 140,218 / 9,251 |

Requested-versus-paid shortfalls are retained as unpaid maintenance, not as
audit failures. Basal metabolism remains separate and unknown. Full actual
plant/corpse food and water acquisition totals, encounter counts, and complete
all-species genotype summaries are preserved in the root-audited formal
summary JSON.

### Formal audit and run totals

Root independently audited all four pairs: 4,000 physical ticks, event totals,
lineage and ancestry, pre-state living genotype individual-ticks, endpoint
counts, actual requested/paid costs, acquisitions, trace coverage, and food/water
stock flows. All root and supplemental founder audits passed. Tick-100 State+RNG
serialize/restore equality matched within each arm; this does not assert
uninterrupted trajectory equality. Complete frames were retained at ticks 1,
100, 200, and 500 per arm. The raw capture is not a full simulation replay and
food encounter traces do not validate discovery.

Formal runtime was 268.222 seconds, peak worker RSS 488,704 KiB, and raw output
623,423,952 bytes. The recorded 636,649,202-byte aggregate snapshot includes
the four completed run outputs and their audit metadata, measured after the
audits and before writing the aggregate summary and before archival. Source checks
at frozen revision `a4232c37d52d50d7e7dccf3b429be03fe1519292`: 251 Rust tests /
0 failures (48 summaries), 177 Python tests / 0 failures, release app/probe,
and scoped Rust formatting passed. Cleanup removed 616,453,126 logical bytes
from the inactive task target only; proof:
`data/operations/wave7-checks-20261024/cleanup.json`. No live deployment.
Root's source-check manifest is
`data/operations/wave7-checks-20261024/manifest.json`.

Calibration seed 123 and the earlier 200-tick seed-123 engineering run are
seen-seed calibration/engineering evidence and excluded from the formal
estimate. Formal outputs are in
`data/experiments/vision-selection-v1-formal-20261024/{54001,54002,54003,54004}/`;
calibration evidence is in
`data/experiments/vision-selection-v1-calibration-20261024/`. The root's
audited aggregate snapshot is
`data/experiments/vision-selection-v1-formal-20261024/summary.json`; its SHA-256
and all manifest/root-audit/founder-audit hashes are in the JSON companion.
All five raw JSONL captures were losslessly archived and verified. Their
individual indexes are:

- Calibration seed 123: [archive index](../../data/experiments/vision-selection-v1-calibration-20261024/raw.jsonl.archive.json), 156,261,209 original bytes → 8,780,851 compressed bytes; archive SHA-256 `a90dbc3bbfc36b9c7ae7850a54afd92d01d7dc1462911b78dde1561f0c83cbcd`.
- Formal seed 54001: [archive index](../../data/experiments/vision-selection-v1-formal-20261024/54001/raw.jsonl.archive.json), 155,564,996 → 8,840,961 bytes; SHA-256 `919ed29e806dbf44f1cac0692ba0670701bd4c8b47d26e1807a9fa29222ef31f`.
- Formal seed 54002: [archive index](../../data/experiments/vision-selection-v1-formal-20261024/54002/raw.jsonl.archive.json), 155,788,935 → 8,812,437 bytes; SHA-256 `23b111087838ebffd511f0034632250ddc56eabc139855142b539cf15c8e1b84`.
- Formal seed 54003: [archive index](../../data/experiments/vision-selection-v1-formal-20261024/54003/raw.jsonl.archive.json), 155,900,985 → 8,775,730 bytes; SHA-256 `0e86678e585ad623b56980b781d41d1156debd6b2bade8a2e5ec0b550db9a4b5`.
- Formal seed 54004: [archive index](../../data/experiments/vision-selection-v1-formal-20261024/54004/raw.jsonl.archive.json), 156,169,036 → 8,845,836 bytes; SHA-256 `b563a0d99469ad60182c5c7e46febb436c8f66e06ed1e2246961b2d1d6f21fd4`.

Across the five files, 779,685,161 original bytes were reduced losslessly to
44,055,815 bytes. The archive batch verification record is
[`archives.json`](../../data/operations/wave7-checks-20261024/archives.json),
SHA-256 `a063042e21ebc1b93a25d54da0c09e9a4f2f0ab11bc5eff472ea321e33ab51fa`.
Restore each JSONL from its verified Zstandard archive before using older
readers that require the uncompressed path.

## Limits

Expression changes RNG consumption during reproduction, so later paths can
diverge. All four valid paired seed blocks show a negative direction, opposite
the original positive prediction. This is a descriptive expressed-feature-
associated difference for this fixture; it does not isolate vision range from
maintenance cost or downstream RNG-path divergence. It is not a pure causal
effect, a general fitness result, evidence of long-term coexistence, or a
habitat-robustness claim.
