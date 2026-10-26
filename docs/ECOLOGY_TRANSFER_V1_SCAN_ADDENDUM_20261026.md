# Ecology transfer V1 seed-scan addendum — 26 October 2026

This addendum records a preflight implementation correction for the registered
Ecology transfer V1 screen. The frozen [protocol](ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md)
remains unchanged.

## Finding and correction

The preflight seed inventory reuses `scripts/run_ecology_traits_lab.py`'s
`audit_seeds` helper. Its default maximum text-line size is 16 MiB, while the
archive contains a one-line calibration JSON of about 39 MiB. The initial
registered attempt therefore failed during seed scanning, before creating an
output directory or starting any World. That failure is scanner/input-size
handling only; it provides no ecological outcome and consumed none of the nine
registered seeds.

The transfer supervisor now passes an explicit 64 MiB line bound to the
existing scanner (`SEED_SCAN_MAX_LINE_BYTES`). A unit test supplies a line
larger than the helper's default and confirms the registered seed is detected,
proving the wider bound is applied. The bound is finite and applies to the
existing complete archive scan; no fixture, seed role, arm, feature flag,
horizon, threshold, outcome metric, or acceptance rule changes.

## Provenance and next attempt

Preserve the failed preflight's logs/status as a failed attempt; do not rewrite
it as a successful scan. A retry must use a distinct output/run identity with
the `scanfix` suffix after the updated runner and addendum are included in the
source hash. The runner still performs the same nine-seed collision audit
before output creation and refuses the phase if any seed is present. This
addendum does not itself run the scanner or any World and does not authorize
an outcome claim.
