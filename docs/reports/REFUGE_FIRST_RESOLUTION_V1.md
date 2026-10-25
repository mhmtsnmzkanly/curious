# Refuge first-resolution extraction — 25 October 2026

## Result

The frozen read-only extractor completed and passed an independent audit. It
emitted one row for each of the 623 recorded shelter starts and exact
selected-threat damage for each first complete resolver interval: 617 zeros
and six outcomes of 20 damage each (120 total). Eight later continuation
intervals were excluded because they had no fresh choice-time perception.

This is an observed one-resolver-interval outcome conditional on the chosen
action and recorded start context. It is not expected full-escape damage,
damage for unselected routes, a counterfactual, or a calibrated predictor. The
earlier full-escape arrival analysis remains success-conditioned: 182 arrivals
had zero damage, while all six positive-damage attempts were later censored.
Expected damage for every route therefore remains **Unknown**. No model was
fit, no model state was written, and the refuge residual helper remains
unintegrated and default-off.

## Frozen inputs and output

- Frozen extractor/protocol commit: `5d648ee06227abb9cbee02d2510f5edb99f2cbb7`.
- Extractor SHA-256: `5ffe05dd56750f7e869e81b60e39b93f54b6d3952ce78597dec8890fb88ef4a4`.
- Protocol SHA-256: `a4b6c53d453b3f1b20aba3174222d45a80855a3f973ba2fe52f8b798ec713afc`.
- Raw source SHA-256: `d9c7e00e976c2fe31e0ad07868df924cabd423b91c6f5c32540b4b2cd43b4874`.
- Source manifest SHA-256: `b5161962b0939b6389f07f64d19458572703e55b721a3383b7eaf0202b2aef12`.
- Independent baseline audit SHA-256:
  `8e22728c1347a7b2fe91bd7c322591cf99424a999d6597e76a2c4361d55e00de`.

Output directory:
`data/experiments/refuge-first-resolution-v1-20261025/extraction/`

- [`rows.jsonl`](../../data/experiments/refuge-first-resolution-v1-20261025/extraction/rows.jsonl): 755,029 bytes; SHA-256 `52e8663b9b3d9ab8d3a1dd3a621ac6a9207323eb7e8b43837bd32453dc5fda78`.
- [`summary.json`](../../data/experiments/refuge-first-resolution-v1-20261025/extraction/summary.json): 8,608 bytes; SHA-256 `5384825415d04498d9c5cb5879813e0d5246416c918458f5b9a16ba2fb901ee9`.
- [`provenance.json`](../../data/experiments/refuge-first-resolution-v1-20261025/extraction/provenance.json): 1,771 bytes; SHA-256 `9112ede9ba5d2cbc60ffa7c71a97884ef652979da324bae26415f1fe0b726a83`.
- Total output: 765,408 bytes.

The extractor recorded identical before/after hashes for raw input, manifest,
independent audit, code, and protocol. It reconciled all 623 target/global
attack and net-health outcomes. The independent root audit separately checked
all 623 extracted feature rows against choice-time records and confirmed that
owner/threat IDs and closure reason remained provenance metadata, not features.
That audit passed with zero Unknown rows, 623 unique starts, 617 exact zeros,
six exact positive outcomes, and eight excluded continuations:
[`first-resolution-independent-audit.json`](../../data/operations/wave9-20261025/first-resolution-independent-audit.json),
SHA-256 `f7a28483072e59fb367c3bf8ed2cbd24ccc57775450a4efb0faa699449aac959`.

The 14 fixture tests cover the same-owner closure/start boundary, missing
intervals and Unknown handling, selected/other/unattributed damage separation,
and exclusion of closure/continuation fields from features. Root reports the
full Python suite passed 191 tests. No Rust recompilation was needed for this
Python-only extraction phase; the frozen engine was unchanged.

## Scope and limits

The extractor read the pinned raw capture only. It ran zero World ticks and
changed no live service, biological source, route policy, or RNG. Positive
first-interval outcomes came from four owners (owner 17 had three; owners 49,
53, and 39 had one each). These are sparse, single-run descriptive data, not
held-out predictor acceptance. The protocol emits only Exact or Unknown; it
does not infer bounds for incomplete intervals. Any predictor, per-owner
learning rule, full-route baseline, or use in route selection needs a separate
review. See the frozen
[`first-resolution protocol`](../REFUGE_FIRST_RESOLUTION_PROTOCOL.md) and the
[full-escape baseline report](REFUGE_RISK_BASELINE_V1.md).
