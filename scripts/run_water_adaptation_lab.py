#!/usr/bin/env python3
"""Bounded supervisor and independent analyzer for the frozen water lab protocol."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import threading
import time
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
MAX_RAW = 64 * 1024 * 1024
MAX_REPLAY = 20 * 1024 * 1024
MAX_DISK = 512 * 1024 * 1024
FORK_TICK = 200
SITES = {"a": {"x": -2, "y": 0}, "b": {"x": 4, "y": 0}}
ROLES = {"calibration": [62199], "formal": [62101, 62102, 62103, 62104, 62105, 62106]}
ARMS = ("trained_online", "trained_frozen", "naive_online", "rule_based")
PROTOCOL_SPEC = {
    "version": "water-adaptation-v1", "fork_tick": FORK_TICK, "end_tick": 400,
    "seeds": ROLES, "sites": SITES, "required_prefix_choices": 5,
    "required_context_entries_at_fork": 1, "required_suffix_choices_per_trained_arm": 3,
    "max_raw_bytes": MAX_RAW, "probe_timeout_seconds": 30,
    "address_space_limit_bytes": 1024**3,
    "frozen_tables": ["values", "residuals", "context_entries"],
    "ttl_clock_frozen": False,
}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def source_identity() -> tuple[str, dict[str, str], str]:
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    names = subprocess.check_output(
        ["git", "ls-files", "-z", "src", "examples/water_adaptation_lab.rs", "Cargo.toml", "Cargo.lock"], cwd=ROOT
    ).decode().split("\0")
    paths = sorted((ROOT / n for n in names if n), key=lambda p: p.relative_to(ROOT).as_posix())
    mapping = {p.relative_to(ROOT).as_posix(): sha256_file(p) for p in paths if p.is_file()}
    runner = ROOT / "scripts/run_water_adaptation_lab.py"
    mapping[runner.relative_to(ROOT).as_posix()] = sha256_file(runner)
    protocol = ROOT / "docs/WATER_ADAPTATION_V1_PROTOCOL_20261025.md"
    mapping[protocol.relative_to(ROOT).as_posix()] = sha256_file(protocol)
    digest = hashlib.sha256(json.dumps(mapping, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return rev, mapping, digest


def require_clean_source() -> None:
    changes = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, text=True)
    if changes.strip():
        raise RuntimeError("source tree must be clean before pilot/formal run; git status is nonempty")


def canonical(obj: Any) -> str:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def without_focal_learner(state: Any) -> Any:
    """Remove only the focal learner so every physical/ordinary-memory field remains compared."""
    import copy
    clone = copy.deepcopy(state)
    actor = focal_actor(clone)
    if actor is None:
        return None
    actor.pop("learner", None)
    return clone


def pos_eq(value: Any, target: dict[str, int]) -> bool:
    if not isinstance(value, dict):
        return False
    x = value.get("x", value.get("0"))
    y = value.get("y", value.get("1"))
    return x == target["x"] and y == target["y"]


def focal_actor(state: Any) -> dict[str, Any] | None:
    if not isinstance(state, dict):
        return None
    actors = state.get("entities")
    if not isinstance(actors, list):
        return None
    return next((a for a in actors if isinstance(a, dict) and a.get("id") == 1), None)


def learner_table(state: Any) -> dict[str, Any] | None:
    actor = focal_actor(state)
    l = actor.get("learner") if actor else None
    if not isinstance(l, dict):
        return None
    return {key: l.get(key) for key in ("values", "residuals", "context_entries")}


def frozen_tables_stable(branch: dict[str, Any]) -> bool:
    initial = learner_table(branch.get("state_after_treatment_before_reversal"))
    if initial is None:
        return False
    frames = branch.get("ticks", [])
    return all(learner_table(row.get("state_after")) == initial for row in frames)


def event_rows(row: dict[str, Any]) -> list[dict[str, Any]]:
    events = row.get("events", [])
    if isinstance(events, dict):
        events = [events]
    return [e for e in events if isinstance(e, dict)] if isinstance(events, list) else []


def event_kind(e: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    if len(e) == 1:
        k, v = next(iter(e.items()))
        return k, v if isinstance(v, dict) else {}
    return str(e.get("type", "")), e


def derive_metrics(branch: dict[str, Any]) -> dict[str, Any]:
    frames = branch.get("ticks", [])
    totals = {"water_acquired": 0, "movement_energy": 0, "thirst_ticks": 0, "dehydration_ticks": 0, "alive_ticks": 0}
    first_b: int | None = None
    last_tick = None
    terminal: dict[str, Any] = {"energy": None, "health": None, "water": None}
    for row in frames:
        p = row.get("focal_physical") or {}
        totals["water_acquired"] += int(p.get("water_gained_resolver", 0) or 0)
        totals["movement_energy"] += int(p.get("movement_energy_paid_resolver", 0) or 0)
        totals["thirst_ticks"] += int(bool(p.get("thirst_tick", False)))
        totals["dehydration_ticks"] += int(p.get("water_before", 1) == 0)
        totals["alive_ticks"] += int(bool(p.get("alive_after", False)))
        tick = row.get("tick")
        if isinstance(tick, int):
            last_tick = tick
        for e in event_rows(row):
            kind, payload = event_kind(e)
            if kind != "EntityDrank" or payload.get("id") != 1:
                continue
            at = payload.get("at", payload.get("pos", payload.get("position")))
            if pos_eq(at, SITES["b"]) and int(payload.get("amount", 0) or 0) > 0 and first_b is None:
                first_b = tick - FORK_TICK if isinstance(tick, int) else None
        actor = focal_actor(row.get("state_after"))
        body = (actor or {}).get("base", {})
        life = ((actor or {}).get("life") if isinstance(actor, dict) and isinstance(actor.get("life"), dict)
                else body.get("life_state", body.get("life", body)) if isinstance(body, dict) else {})
        if isinstance(life, dict):
            for field in ("energy", "health", "water"):
                if field in life:
                    terminal[field] = life[field]
        terminal["alive"] = bool(p.get("alive_after", False))
        for source, target in (("energy_after", "energy"), ("health_after", "health"),
                               ("water_after", "water")):
            if p.get(source) is not None:
                terminal[target] = p[source]
    status = branch.get("status")
    censor_reason = None
    if first_b is None:
        censor_reason = "death" if status == "focal_died" else "horizon" if last_tick == 400 else "incomplete"
    return {**totals, "terminal": terminal,
            "first_successful_drink_b_after_fork": {"lag_ticks": first_b, "observed": first_b is not None,
                                                       "censored": first_b is None, "censor_reason": censor_reason,
                                                       "observed_suffix_ticks": len(frames)}}


def choice_count(rows: list[dict[str, Any]], start: int | None = None) -> int:
    return sum(1 for r in rows if (start is None or r.get("tick", 0) > start)
               and isinstance(r.get("focal_choice"), dict)
               and r["focal_choice"].get("fresh_two_source_choice") is True)


def context_count(state: Any) -> int:
    table = learner_table(state)
    entries = table.get("context_entries") if table else None
    return len(entries) if isinstance(entries, list) else 0


def arm_comparison(a: dict[str, Any], b: dict[str, Any]) -> dict[str, Any]:
    # Missing values are incomparable; do not impute them.
    directions = {
        "alive_ticks": 1, "water_acquired": 1, "movement_energy": -1,
        "thirst_ticks": -1, "dehydration_ticks": -1,
    }
    components: dict[str, bool | None] = {}
    for key, direction in directions.items():
        av, bv = a.get(key), b.get(key)
        components[key] = None if av is None or bv is None else (av >= bv if direction == 1 else av <= bv)
    for key in ("energy", "health", "water"):
        av, bv = a.get("terminal", {}).get(key), b.get("terminal", {}).get(key)
        components[f"terminal_{key}"] = None if av is None or bv is None else av >= bv
    known = [v for v in components.values() if v is not None]
    strict = any(
        (a.get(k, 0) > b.get(k, 0) if d == 1 else a.get(k, 0) < b.get(k, 0))
        for k, d in directions.items()
    )
    strict |= any(a.get("terminal", {}).get(k) is not None and b.get("terminal", {}).get(k) is not None
                  and a["terminal"][k] > b["terminal"][k] for k in ("energy", "health", "water"))
    return {"components_at_least_as_good": components,
            "pareto_dominates": len(known) == len(components) and all(known) and strict}


def analyze_record(record: dict[str, Any]) -> dict[str, Any]:
    prefix = record.get("prefix", [])
    forks = record.get("forks", {})
    branches = {name: forks.get(name) for name in ARMS}
    baseline = branches["trained_online"]
    valid_branch = all(isinstance(branches[n], dict) for n in ("trained_online", "trained_frozen"))
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
        "all_required_branches_present": all(isinstance(branches[n], dict) for n in ARMS),
    }
    comparisons = {}
    for candidate in ("trained_online", "naive_online"):
        if candidate in metrics:
            comparisons[candidate] = {other: arm_comparison(metrics[candidate], metrics[other])
                                      for other in ("trained_frozen", "rule_based") if other in metrics}
    return {"seed": record.get("seed"), "role": record.get("role"), "validity": validity,
            "pilot_valid": all(validity.values()), "metrics": metrics, "pareto": comparisons,
            "fresh_two_source_counts": {"prefix": choice_count(prefix), **{
                n: choice_count(b.get("ticks", [])) for n, b in branches.items() if isinstance(b, dict)}}}


def replay_for(record: dict[str, Any], source_revision: str) -> dict[str, Any]:
    arms = []
    for name in ARMS:
        b = record.get("forks", {}).get(name)
        if not isinstance(b, dict):
            continue
        frames = [{"tick": row.get("tick"), "state": row.get("state_after"),
                   "decisions": row.get("decisions", []), "events": row.get("events", [])}
                  for row in b.get("ticks", [])]
        arms.append({"name": name, "metrics": derive_metrics(b), "frames": frames})
    return {"schema": "water-adaptation-replay-v1", "seed": record["seed"],
            "source": source_revision, "intervention": record.get("water_manipulation"),
            "sites": SITES, "fork_tick": FORK_TICK,
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
    if record.get("seed") != seed or record.get("role") != role or record.get("schema") != "curious-water-adaptation-seed-v1":
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
    manifest: dict[str, Any] = {"schema": "water-adaptation-run-manifest-v1", "phase": args.phase,
                                "status": "running", "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                                "seeds": ROLES[args.phase], "raw_limit_bytes": MAX_RAW, "probe_timeout_seconds": 30,
                                "address_space_limit_bytes": 1024**3, "protocol": "water-adaptation-v1"}
    try:
        require_clean_source()
        binary = args.binary.resolve(strict=True)
        rev, source_map, source_sha = source_identity()
        binary_sha = sha256_file(binary)
        protocol_sha = sha256_file(ROOT / "docs/WATER_ADAPTATION_V1_PROTOCOL_20261025.md")
        manifest.update({"source_revision": rev, "source_sha256": source_sha, "source_sha256_by_path": source_map,
                         "binary": str(binary), "binary_sha256": binary_sha, "protocol": PROTOCOL_SPEC,
                         "protocol_sha256": protocol_sha,
                         "runner_sha256": source_map["scripts/run_water_adaptation_lab.py"],
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
        summary = {"schema": "water-adaptation-analysis-v1", "phase": args.phase,
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
