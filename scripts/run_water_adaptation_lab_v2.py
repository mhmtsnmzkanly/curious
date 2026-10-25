#!/usr/bin/env python3
"""V2 bounded supervisor. V1 remains immutable; shared primitives are imported from it."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import resource
import subprocess
import sys
import threading
import time
from typing import Any

import run_water_adaptation_lab as V1

ROOT = Path(__file__).resolve().parents[1]
MAX_RAW = 64 * 1024 * 1024
MAX_REPLAY = 20 * 1024 * 1024
MAX_DISK = 512 * 1024 * 1024
FORK_TICK = 200
SITES = {"a": {"x": -2, "y": 0}, "b": {"x": 4, "y": 0}}
ROLES = {"calibration": [62299], "formal": [62201, 62202, 62203, 62204, 62205, 62206]}
ARMS = ("trained_online", "trained_frozen", "naive_online", "rule_based")
PROTOCOL_SPEC = {
    "version": "water-adaptation-v2", "fork_tick": FORK_TICK, "end_tick": 400,
    "seeds": ROLES, "sites": SITES, "required_prefix_choices": 5,
    "required_context_entries_at_fork": 1, "required_suffix_choices_per_trained_arm": 3,
    "max_raw_bytes": MAX_RAW, "probe_timeout_seconds": 30,
    "address_space_limit_bytes": 1024**3,
    "frozen_tables": ["values", "residuals", "context_entries"],
    "ttl_clock_frozen": False,
}


def source_identity() -> tuple[str, dict[str, str], str]:
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    names = subprocess.check_output(
        ["git", "ls-files", "-z", "src", "Cargo.toml", "Cargo.lock"], cwd=ROOT
    ).decode().split("\0")
    paths = sorted((ROOT / n for n in names if n), key=lambda p: p.relative_to(ROOT).as_posix())
    mapping = {p.relative_to(ROOT).as_posix(): sha256_file(p) for p in paths if p.is_file()}
    for rel in ("scripts/run_water_adaptation_lab.py", "scripts/run_water_adaptation_lab_v2.py",
                "examples/water_adaptation_lab_v2.rs"):
        p = ROOT / rel
        if not p.is_file():
            raise RuntimeError(f"missing source identity input: {rel}")
        mapping[rel] = sha256_file(p)
    protocol = ROOT / "docs/WATER_ADAPTATION_V2_PROTOCOL_20261025.md"
    if not protocol.is_file():
        raise RuntimeError("missing V2 protocol")
    mapping[protocol.relative_to(ROOT).as_posix()] = sha256_file(protocol)
    digest = hashlib.sha256(json.dumps(mapping, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return rev, mapping, digest


def require_clean_source() -> None:
    changes = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True)
    if changes.strip():
        raise RuntimeError("source tree must be clean before pilot/formal run; git status is nonempty")


# Shared inspection and physical metric semantics remain delegated to V1.
sha256_file = V1.sha256_file
canonical = V1.canonical
without_focal_learner = V1.without_focal_learner
pos_eq = V1.pos_eq
focal_actor = V1.focal_actor
learner_table = V1.learner_table
frozen_tables_stable = V1.frozen_tables_stable
event_rows = V1.event_rows
event_kind = V1.event_kind
derive_metrics = V1.derive_metrics
choice_count = V1.choice_count
context_count = V1.context_count
arm_comparison = V1.arm_comparison


def analyze_record(record: dict[str, Any]) -> dict[str, Any]:
    prefix = record.get("prefix", [])
    forks = record.get("forks", {})
    branches = {name: forks.get(name) for name in ARMS}
    baseline = branches["trained_online"]
    valid_branch = all(isinstance(branches[n], dict) for n in ARMS)
    prefix_table = learner_table(record.get("checkpoint_state"))
    stable = bool(branches["trained_frozen"] and frozen_tables_stable(branches["trained_frozen"]))
    state_boundary = []
    post_treatment_physical = []
    for b in branches.values():
        if isinstance(b, dict):
            state_boundary.append(b.get("state_after_fork_interruption"))
            post_treatment_physical.append(without_focal_learner(b.get("state_after_treatment_before_reversal")))
    same_fork = bool(state_boundary) and all(canonical(s) == canonical(state_boundary[0]) for s in state_boundary)
    same_post_treatment_physics = bool(post_treatment_physical) and all(
        canonical(s) == canonical(post_treatment_physical[0]) for s in post_treatment_physical)
    metrics = {name: derive_metrics(b) for name, b in branches.items() if isinstance(b, dict)}
    checkpoint_actor = focal_actor(record.get("checkpoint_state"))
    checkpoint_body = (checkpoint_actor or {}).get("base", {})
    checkpoint_life = ((checkpoint_actor or {}).get("life") if isinstance(checkpoint_actor, dict)
                       and isinstance(checkpoint_actor.get("life"), dict)
                       else checkpoint_body.get("life_state", checkpoint_body.get("life", checkpoint_body))
                       if isinstance(checkpoint_body, dict) else {})
    validity = {
        "focal_alive_at_fork": bool(checkpoint_actor is not None and isinstance(checkpoint_life, dict)
                                      and checkpoint_life.get("health", 0) > 0),
        "prefix_fresh_two_source_choices_at_least_5": choice_count(prefix) >= 5,
        "trained_context_entries_at_fork_at_least_1": context_count(record.get("checkpoint_state")) >= 1,
        "trained_online_suffix_choices_at_least_3": bool(baseline) and choice_count(baseline.get("ticks", [])) >= 3,
        "trained_frozen_suffix_choices_at_least_3": bool(branches["trained_frozen"]) and choice_count(branches["trained_frozen"].get("ticks", [])) >= 3,
        "frozen_tables_stable": stable,
        "common_fork_state_after_censor": same_fork,
        "post_treatment_body_memory_map_rng_equal_when_learner_removed": same_post_treatment_physics,
        "all_required_branches_present": valid_branch,
    }
    comparisons = {}
    for candidate in ("trained_online", "naive_online"):
        if candidate in metrics:
            comparisons[candidate] = {other: arm_comparison(metrics[candidate], metrics[other])
                                      for other in ("trained_frozen", "rule_based") if other in metrics}
    return {"seed": record.get("seed"), "role": record.get("role"), "validity": validity,
            "pilot_valid": all(validity.values()), "metrics": metrics, "pareto": comparisons,
            "online_frozen_choice_witness": first_online_frozen_choice_witness(
                branches.get("trained_online"), branches.get("trained_frozen")),
            "fresh_two_source_counts": {"prefix": choice_count(prefix), **{
                n: choice_count(b.get("ticks", [])) for n, b in branches.items() if isinstance(b, dict)}}}


def first_online_frozen_choice_witness(online: Any, frozen: Any) -> dict[str, Any]:
    """Find the first different fresh target under matched state, RNG and options."""
    if not isinstance(online, dict) or not isinstance(frozen, dict):
        return {"status": "not_established", "reason": "missing_branch"}
    frozen_by_tick = {r.get("tick"): r for r in frozen.get("ticks", []) if isinstance(r, dict)}
    for left in online.get("ticks", []):
        if not isinstance(left, dict):
            continue
        right = frozen_by_tick.get(left.get("tick"))
        if not isinstance(right, dict):
            continue
        lc, rc = left.get("focal_choice"), right.get("focal_choice")
        if not isinstance(lc, dict) or not isinstance(rc, dict):
            continue
        if lc.get("fresh_two_source_choice") is not True or rc.get("fresh_two_source_choice") is not True:
            continue
        if canonical(without_focal_learner(left.get("state_before"))) != canonical(
                without_focal_learner(right.get("state_before"))):
            continue
        if left.get("rng_before") != right.get("rng_before"):
            continue
        def signature(choice: dict[str, Any]) -> list[tuple[str, Any, Any]]:
            return sorted((canonical(c.get("pos")), c.get("amount"), c.get("steps_len"))
                          for c in choice.get("candidates", []) if isinstance(c, dict))
        if signature(lc) != signature(rc):
            continue
        online_table = learner_table(left.get("state_before"))
        frozen_table = learner_table(right.get("state_before"))
        if online_table is None or frozen_table is None:
            continue
        online_predictions = context_predictions(online_table, left.get("state_before"), left.get("tick"), lc)
        frozen_predictions = context_predictions(frozen_table, right.get("state_before"), right.get("tick"), rc)
        differing_positions = sorted(
            key for key in set(online_predictions) | set(frozen_predictions)
            if (online_predictions.get(key) if online_predictions.get(key) is not None else 0)
            != (frozen_predictions.get(key) if frozen_predictions.get(key) is not None else 0))
        if not differing_positions:
            continue
        online_target, frozen_target = lc.get("selected_target"), rc.get("selected_target")
        if online_target != frozen_target:
            return {"status": "supported", "tick": left.get("tick"),
                    "online_selected_target": online_target, "frozen_selected_target": frozen_target,
                    "same_nonlearner_state": True, "same_rng": True, "same_candidates": True,
                    "differing_context_candidate_positions": differing_positions,
                    "online_context_predictions": online_predictions,
                    "frozen_context_predictions": frozen_predictions,
                    "context_ttl_ticks": 48,
                    "rng_kind_classification": "not_classified"}
    return {"status": "not_established", "reason": "no_matched_fresh_choice_with_different_target"}


def context_predictions(table: dict[str, Any], state: Any, tick: Any,
                        choice: dict[str, Any]) -> dict[str, int | None]:
    """Return only valid TTL context predictions matching observed choice candidates."""
    actor = focal_actor(state)
    life = actor.get("life") if actor else None
    if not isinstance(life, dict) or not isinstance(tick, int):
        return {}
    speed, move_cost = life.get("speed"), life.get("move_energy_cost")
    if not isinstance(speed, int) or not isinstance(move_cost, int):
        return {}
    entries = table.get("context_entries")
    if not isinstance(entries, list):
        return {}
    candidates = [c.get("pos") for c in choice.get("candidates", []) if isinstance(c, dict)]
    predictions: dict[str, int | None] = {}
    for pos in candidates:
        key = canonical(pos)
        match = next((entry for entry in entries if isinstance(entry, dict)
                      and canonical(entry.get("pos")) == key
                      and entry.get("speed") == speed and entry.get("move_energy_cost") == move_cost
                      and isinstance(entry.get("last_seen_tick"), int)
                      and tick - entry["last_seen_tick"] <= 48
                      and tick >= entry["last_seen_tick"]), None)
        observations = match.get("observations") if match else None
        if isinstance(observations, list) and observations:
            predictions[key] = int(sum(observations) / len(observations))
        else:
            predictions[key] = None
    return predictions


def replay_for(record: dict[str, Any], source_revision: str) -> dict[str, Any]:
    def project_state(state: Any) -> dict[str, Any]:
        if not isinstance(state, dict):
            return {}
        return {key: state.get(key) for key in ("tick", "bounds", "resources", "entities")}

    def project_decisions(decisions: Any) -> list[dict[str, Any]]:
        if not isinstance(decisions, list):
            return []
        return [{key: d.get(key) for key in ("id", "pos", "decision")}
                for d in decisions if isinstance(d, dict)]

    arms = []
    for name in ARMS:
        b = record.get("forks", {}).get(name)
        if not isinstance(b, dict):
            continue
        frames = [{"tick": row.get("tick"), "state": project_state(row.get("state_after")),
                   "decisions": project_decisions(row.get("decisions")), "events": row.get("events", [])}
                  for row in b.get("ticks", [])]
        arms.append({"name": name, "metrics": derive_metrics(b), "frames": frames})
    return {"schema": "water-adaptation-replay-v1", "experiment": "water-adaptation-v2", "seed": record["seed"],
            "source": source_revision, "intervention": record.get("water_manipulation"),
            "sites": SITES, "fork_tick": FORK_TICK,
            "projection_note": "Replay frames retain tick, bounds, resources, entities, compact decisions, and events; raw JSON preserves full State and decision traces.",
            "arms": arms}


def run_one(binary: Path, out: Path, seed: int, role: str) -> dict[str, Any]:
    stem = f"seed-{seed}"
    raw = out / f"{stem}.raw.json"
    err = out / f"{stem}.stderr"
    with raw.open("xb") as stdout_file, err.open("xb") as stderr_file:
        proc = subprocess.Popen([str(binary), str(seed), role], cwd=ROOT, stdout=subprocess.PIPE, stderr=stderr_file,
                                preexec_fn=lambda: resource.setrlimit(resource.RLIMIT_AS, (1024**3, 1024**3)))
        overflow = threading.Event()

        def drain_stdout() -> None:
            written = 0
            assert proc.stdout is not None
            while True:
                block = proc.stdout.read(64 * 1024)
                if not block:
                    break
                if written + len(block) > MAX_RAW:
                    keep = max(0, MAX_RAW - written)
                    if keep:
                        stdout_file.write(block[:keep])
                    stdout_file.flush()
                    overflow.set()
                    proc.kill()
                    break
                stdout_file.write(block)
                written += len(block)
            stdout_file.flush()

        reader = threading.Thread(target=drain_stdout, daemon=True)
        reader.start()
        try:
            rc = proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            proc.kill(); proc.wait()
            reader.join()
            raise RuntimeError(f"seed {seed} exceeded 30 second timeout; raw output retained")
        reader.join()
        if overflow.is_set():
            raise RuntimeError(f"seed {seed} raw output exceeded 64 MiB; truncated raw output retained")
    if raw.stat().st_size > MAX_RAW:
        raise RuntimeError(f"seed {seed} raw output exceeds 64 MiB")
    if rc != 0:
        raise RuntimeError(f"seed {seed} probe exited {rc}; stderr retained")
    record = json.loads(raw.read_text())
    if record.get("seed") != seed or record.get("role") != role or record.get("schema") != "curious-water-adaptation-seed-v2":
        raise RuntimeError(f"seed {seed} raw schema/identity mismatch")
    return record


def write_json(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n")


def enforce_disk_cap(out: Path) -> None:
    used = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    if used > MAX_DISK:
        raise RuntimeError("output exceeds 512 MiB total disk budget")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", required=True, type=Path)
    ap.add_argument("--phase", required=True, choices=tuple(ROLES))
    ap.add_argument("--output", required=True, type=Path)
    ap.add_argument("--calibration", type=Path, help="successful calibration manifest required for formal")
    args = ap.parse_args()
    out = args.output.resolve()
    if out.exists():
        print("water adaptation lab: output directory already exists", file=sys.stderr)
        return 2
    manifest_path = out / "manifest.json"
    manifest: dict[str, Any] = {"schema": "water-adaptation-run-manifest-v2", "phase": args.phase,
                                "status": "running", "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                                "seeds": ROLES[args.phase], "raw_limit_bytes": MAX_RAW, "probe_timeout_seconds": 30,
                                "address_space_limit_bytes": 1024**3, "protocol": "water-adaptation-v2"}
    try:
        require_clean_source()
        binary = args.binary.resolve(strict=True)
        rev, source_map, source_sha = source_identity()
        binary_sha = sha256_file(binary)
        protocol_sha = sha256_file(ROOT / "docs/WATER_ADAPTATION_V2_PROTOCOL_20261025.md")
        manifest.update({"source_revision": rev, "source_sha256": source_sha, "source_sha256_by_path": source_map,
                         "binary": str(binary), "binary_sha256": binary_sha, "protocol": PROTOCOL_SPEC,
                         "protocol_sha256": protocol_sha,
                         "runner_sha256": source_map["scripts/run_water_adaptation_lab_v2.py"],
                         "command_template": [str(binary), "SEED", args.phase],
                         "max_disk_bytes": MAX_DISK})
        if args.phase == "formal":
            if not args.calibration:
                raise RuntimeError("formal phase requires --calibration successful manifest")
            cal = json.loads(args.calibration.read_text())
            if cal.get("phase") != "calibration" or cal.get("status") != "completed" or cal.get("pilot_valid") is not True:
                raise RuntimeError("formal phase blocked: calibration manifest is not successful")
            if (cal.get("binary_sha256") != binary_sha or cal.get("source_sha256") != source_sha
                    or cal.get("protocol_sha256") != protocol_sha):
                raise RuntimeError("formal phase blocked: calibration binary/source SHA mismatch")
        out.mkdir(parents=True, exist_ok=False)
        write_json(manifest_path, manifest)
        enforce_disk_cap(out)
        records = []
        analyses = []
        for seed in ROLES[args.phase]:
            record = run_one(binary, out, seed, args.phase)
            records.append(record)
            enforce_disk_cap(out)
            analysis = analyze_record(record)
            analyses.append(analysis)
            if args.phase == "formal":
                required_invariants = (
                    "frozen_tables_stable",
                    "common_fork_state_after_censor",
                    "post_treatment_body_memory_map_rng_equal_when_learner_removed",
                )
                failed = [name for name in required_invariants if analysis["validity"].get(name) is not True]
                if failed:
                    raise RuntimeError(f"formal seed {seed} infrastructure invariant failed: {', '.join(failed)}")
        pilot_valid = all(a["pilot_valid"] for a in analyses) if args.phase == "calibration" else None
        summary = {"schema": "water-adaptation-analysis-v2", "phase": args.phase,
                   "pilot_valid": pilot_valid, "seeds": analyses,
                   "summary": summarize(analyses)}
        write_json(out / "summary.json", summary)
        enforce_disk_cap(out)
        for r in records:
            data = json.dumps(replay_for(r, rev), separators=(",", ":")).encode()
            if len(data) > MAX_REPLAY:
                raise RuntimeError(f"seed {r['seed']} replay bundle exceeds 20 MiB")
            (out / f"seed-{r['seed']}.replay.json").write_bytes(data)
            enforce_disk_cap(out)
        manifest.update({"status": "completed" if pilot_valid is not False else "failed",
                         "pilot_valid": pilot_valid,
                         "raw_sha256": {p.name: sha256_file(p) for p in sorted(out.glob("*.raw.json"))},
                         "stderr_sha256": {p.name: sha256_file(p) for p in sorted(out.glob("*.stderr"))},
                         "summary_sha256": sha256_file(out / "summary.json"),
                         "completed_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
        if pilot_valid is False:
            manifest["error"] = "calibration validity gate failed; formal phase blocked"
        write_json(manifest_path, manifest)
        return 0 if args.phase != "calibration" or pilot_valid else 3
    except Exception as exc:
        manifest.update({"status": "failed", "error": str(exc),
                         "failed_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
        if not out.exists():
            out.mkdir(parents=True, exist_ok=True)
        write_json(manifest_path, manifest)
        print(f"water adaptation lab: {exc}", file=sys.stderr)
        return 2


def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
    arms = sorted({arm for row in rows for arm in row.get("metrics", {})})
    totals: dict[str, Any] = {}
    for arm in arms:
        collected = [r["metrics"][arm] for r in rows if arm in r.get("metrics", {})]
        keys = ("alive_ticks", "water_acquired", "movement_energy", "thirst_ticks", "dehydration_ticks")
        totals[arm] = {key: sum(m.get(key, 0) for m in collected) for key in keys}
        totals[arm]["terminal"] = {key: sum(m["terminal"][key] for m in collected if m.get("terminal", {}).get(key) is not None)
                                    if all(m.get("terminal", {}).get(key) is not None for m in collected) else None
                                    for key in ("energy", "health", "water")}
        totals[arm]["all_full_suffix_alive"] = all(m.get("alive_ticks", 0) == 200 for m in collected)
    aggregate_pareto: dict[str, Any] = {}
    def as_metrics(value: dict[str, Any]) -> dict[str, Any]:
        return value
    for candidate in ("trained_online", "trained_frozen", "naive_online"):
        if candidate not in totals:
            continue
        aggregate_pareto[candidate] = {
            comparator: arm_comparison(as_metrics(totals[candidate]), as_metrics(totals[comparator]))
            for comparator in ("trained_frozen", "rule_based") if comparator in totals and comparator != candidate
        }
    full_horizon = all(totals.get(arm, {}).get("all_full_suffix_alive", False) for arm in ARMS)
    return {"physical_totals_by_arm": totals, "aggregate_componentwise_pareto_descriptive_only": aggregate_pareto,
            "all_four_arms_full_suffix_alive_for_every_seed_descriptive": full_horizon,
            "utility_claim": "per-seed results; mechanism witness needed; not acceptance"}


if __name__ == "__main__":
    raise SystemExit(main())
