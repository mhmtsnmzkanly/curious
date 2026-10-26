#!/usr/bin/env python3
"""Registered four-arm supervisor for the water-meal priority screen."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import resource
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
SEEDS = range(62501, 62507)
HABITATS = {
    "v1_unique_food": ("v1", list(SEEDS)),
    "meadow": ("scenarios/meadow.json", list(SEEDS)),
    "sparse_resource": ("scenarios/ecology-traits-lab.json", list(SEEDS)),
}
ARMS = ("control", "food_reserve_only", "water_meal_only", "bundle")
TICKS = 300
PHASE_SECONDS = 300
CHILD_SECONDS = 60
CHILD_ADDRESS_SPACE = 1024**3
OUTPUT_CAP = 256 * 1024**2
MAX_WORLD_OUTPUT = 16 * 1024**2
MAX_IDENTITIES = 10_000
MAX_EVENTS_PER_TICK = 20_000
SEED_SCAN_MAX_LINE_BYTES = 64 * 1024**2
PROTOCOL = "docs/WATER_MEAL_PRIORITY_V1_PROTOCOL_20261026.md"
SOURCE_PATHS = (
    "Cargo.toml", "Cargo.lock", "examples/water_meal_priority_lab.rs",
    "scripts/run_water_meal_priority_lab.py", "scripts/test_water_meal_priority_lab.py",
    "scripts/run_ecology_traits_lab.py", "scripts/evidence_archive.py",
    "docs/EVIDENCE_ARCHIVES.md", PROTOCOL,
    "docs/WATER_MEAL_PRIORITY_V1_REGISTRATION_TEMPLATE.json", "docs/README.md",
    "scenarios/meadow.json", "scenarios/ecology-traits-lab.json",
)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def git(*args: str) -> str:
    return subprocess.check_output(["git", "-C", str(ROOT), *args], text=True).strip()


def source_identity() -> dict:
    tracked = git("ls-files", "-z", "src").split("\0")
    paths = sorted(set(SOURCE_PATHS).union(p for p in tracked if p))
    files = {p: sha256(ROOT / p) for p in paths}
    digest = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return {"revision": git("rev-parse", "HEAD"), "clean": not bool(git("status", "--porcelain", "--untracked-files=all")), "file_sha256": files, "content_sha256": digest}


def scenario_hashes() -> dict[str, str | None]:
    return {name: sha256(ROOT / path) if path != "v1" else None for name, (path, _) in HABITATS.items()}


def expected_seed_roles() -> list[dict]:
    return [{"habitat": habitat, "seed": seed} for habitat, (_, seeds) in HABITATS.items() for seed in seeds]


def _verified_index_seed_metadata(index: dict, index_path: str) -> tuple[list[int], dict[str, list[int]]]:
    seeds = index.get("seeds")
    roles = index.get("seed_roles")
    if not isinstance(seeds, list) or any(not isinstance(seed, int) or isinstance(seed, bool) for seed in seeds):
        raise RuntimeError(f"invalid declared seeds in compressed JSONL index: {index_path}")
    if seeds != sorted(set(seeds)):
        raise RuntimeError(f"compressed JSONL index seeds are not unique and sorted: {index_path}")
    if not isinstance(roles, dict) or any(not isinstance(role, str) or not isinstance(values, list) for role, values in roles.items()):
        raise RuntimeError(f"invalid declared seed_roles in compressed JSONL index: {index_path}")
    for role, values in roles.items():
        if (not role or not values or any(not isinstance(seed, int) or isinstance(seed, bool) for seed in values)
                or values != sorted(set(values))):
            raise RuntimeError(f"invalid seed role values in compressed JSONL index: {index_path}")
    if sorted({seed for values in roles.values() for seed in values}) != seeds:
        raise RuntimeError(f"compressed JSONL index seeds differ from its seed_roles map: {index_path}")
    return seeds, roles


def validate_registration(registration: dict, identity: dict, fixtures: dict, binary_sha: str) -> None:
    if registration.get("status") != "registered" or registration.get("root_reviewed") is not True:
        raise ValueError("registration must be status=registered and root_reviewed=true")
    if not identity["clean"]:
        raise ValueError("registered evaluation requires a clean committed checkout")
    if registration.get("source") != identity:
        raise ValueError("registration source identity differs from current checkout")
    if registration.get("scenario_sha256") != fixtures:
        raise ValueError("fixture hashes differ from registration")
    if registration.get("seed_roles") != expected_seed_roles():
        raise ValueError("registered seed roles differ from frozen 62501..62506 allocation")
    if registration.get("binary_sha256") != binary_sha:
        raise ValueError("prebuilt binary SHA-256 differs from registration")
    expected = {
        "ticks_per_world": TICKS, "arms": list(ARMS), "habitats": list(HABITATS),
        "world_count": 72, "phase_seconds": PHASE_SECONDS, "child_seconds": CHILD_SECONDS,
        "child_address_space_bytes": CHILD_ADDRESS_SPACE, "aggregate_output_bytes": OUTPUT_CAP,
        "max_world_output_bytes": MAX_WORLD_OUTPUT, "max_identities": MAX_IDENTITIES,
        "max_events_per_tick": MAX_EVENTS_PER_TICK, "mutation_per_mille": 0,
        "genetics_enabled": True,
        "candidate_flags": {"food_search_reserve_enabled": "factorial_existing_flag", "water_meal_priority_enabled": "approved_candidate"},
    }
    for key, value in expected.items():
        if registration.get(key) != value:
            raise ValueError(f"registration {key} differs from frozen protocol")


def scan_seed_roles(archive: Path, roles: list[dict]) -> dict:
    helper = ROOT / "scripts/run_ecology_traits_lab.py"
    spec = importlib.util.spec_from_file_location("water_meal_seed_inventory", helper)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load existing seed inventory scanner")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    seeds = sorted({r["seed"] for r in roles})
    audit = mod.audit_seeds(archive, seeds, max_line_bytes=SEED_SCAN_MAX_LINE_BYTES)
    matches = audit["matches"]
    compressed = sorted(p for p in archive.rglob("*.jsonl.zst") if p.is_file()) if archive.exists() else []
    compressed_bytes, indexes = 0, []
    for path in compressed:
        rel = str(path.relative_to(archive))
        index_path = Path(str(path.with_suffix("")) + ".archive.json")
        if not index_path.is_file():
            raise RuntimeError(f"compressed JSONL has no verified seed index: {rel}")
        index = json.loads(index_path.read_text(encoding="utf-8"))
        archive_rel, index_rel = str(path.relative_to(ROOT)), str(index_path.relative_to(ROOT))
        if index.get("status") != "verified" or index.get("codec") != "zstd" or index.get("archivepath") != archive_rel:
            raise RuntimeError(f"compressed JSONL index identity/status mismatch: {index_rel}")
        if index.get("archive_size") != path.stat().st_size or index.get("archive_sha256") != sha256(path):
            raise RuntimeError(f"compressed JSONL hash/size differs from index: {rel}")
        declared, seed_roles = _verified_index_seed_metadata(index, index_rel)
        for seed in set(declared).intersection(seeds):
            if index_rel not in matches[str(seed)]:
                matches[str(seed)].append(index_rel)
        compressed_bytes += path.stat().st_size
        indexes.append({"path":archive_rel,"index":index_rel,"sha256":index["archive_sha256"],"size":index["archive_size"],"seeds":declared,"seed_roles":seed_roles})
    audit.update(matches=matches, unseen=[s for s in seeds if not matches[str(s)]],
                 scanned_compressed_jsonl_files=len(compressed), scanned_compressed_jsonl_bytes=compressed_bytes,
                 verified_compressed_jsonl_indexes=indexes, max_line_bytes=SEED_SCAN_MAX_LINE_BYTES,
                 sqlite_coverage="SQLite payloads are not decoded; declared seed roles come from retained manifests and experiment metadata.")
    if audit["unseen"] != seeds:
        raise ValueError(f"one or more registered seeds already occur in experiment inventory: {matches}")
    return audit


def _child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (CHILD_ADDRESS_SPACE, CHILD_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_CPU, (CHILD_SECONDS, CHILD_SECONDS))


def _json_bytes(value: dict) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def _atomic_json(path: Path, value: dict) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_bytes(_json_bytes(value))
    os.replace(temp, path)


def _total_bytes(root: Path) -> int:
    return sum(p.stat().st_size for p in root.rglob("*") if p.is_file())


def _nonzero(value: dict) -> dict:
    return {str(k): v for k, v in value.items() if v != 0}


def _event_totals(row: dict) -> tuple[int, int, int]:
    food = water = movement = 0
    for record in row["actor_resolver_events"]:
        event = record.get("event")
        if not isinstance(event, dict) or len(event) != 1:
            raise ValueError("malformed serialized resolver event")
        name, payload = next(iter(event.items()))
        if name == "EntityAte" and payload.get("corpse_id") is None:
            food += int(payload["amount"])
        elif name == "EntityDrank":
            water += int(payload["amount"])
        elif name == "EntityMoved":
            movement += int(payload["energy_cost"])
    return food, water, movement


def _diagnostics_for_tick(row: dict) -> dict[str, int]:
    trace = row.get("focal_decision_trace") or {}
    events = row.get("focal_events", [])
    rejected = any(
        "ActionRejected" in e and e["ActionRejected"].get("id") == 1
        and e["ActionRejected"].get("action") == "Approach"
        and e["ActionRejected"].get("reason") == "OutOfRange"
        for e in events if isinstance(e, dict)
    )
    paid = sum(e.get("EntityMoved", {}).get("energy_cost", 0) for e in events
               if isinstance(e, dict) and "EntityMoved" in e and e["EntityMoved"].get("id") == 1)
    out = {"focal_approach_out_of_range_count": int(rejected),
           "focal_matched_futile_water_approaches": 0,
           "focal_zero_paid_water_approaches": 0,
           "focal_underfunded_water_approaches": 0,
           "focal_zero_energy_visible_food_approach_failures": 0,
           "focal_underfunded_visible_food_approach_failures": 0}
    decision = trace.get("decision", {})
    intent = decision.get("intent", {}) if isinstance(decision, dict) else {}
    perception = trace.get("perception")
    before = trace.get("before", {})
    if not rejected or not isinstance(perception, dict):
        return out
    drink = intent.get("Drink") if isinstance(intent, dict) else None
    if isinstance(drink, dict):
        path = _steps(drink.get("at"))
        matched = any(w.get("amount", 0) > 0 and _steps(w.get("steps")) == path for w in perception.get("waters", []))
        required = max(len(path) - 1, 0) * before.get("move_energy_cost", 0)
        if matched:
            out["focal_matched_futile_water_approaches"] = 1
            if required > 0 and paid == 0:
                out["focal_zero_paid_water_approaches"] = 1
            elif required > 0 and 0 < paid < required:
                out["focal_underfunded_water_approaches"] = 1
    eat = intent.get("Eat") if isinstance(intent, dict) else None
    if isinstance(eat, dict) and eat.get("corpse_id") is None:
        path = _steps(eat.get("at"))
        matched = any(f.get("amount", 0) > 0 and not f.get("is_corpse") and f.get("corpse_id") is None
                      and _steps(f.get("steps")) == path for f in perception.get("foods", []))
        required = max(len(path) - 1, 0) * before.get("move_energy_cost", 0)
        if matched and required > 0 and before.get("energy") == 0 and paid == 0:
            out["focal_zero_energy_visible_food_approach_failures"] = 1
        if matched and required > 0 and before.get("energy", 0) < required and 0 < paid < required:
            out["focal_underfunded_visible_food_approach_failures"] = 1
    return out


def _steps(value) -> list:
    if isinstance(value, list):
        return value
    if isinstance(value, dict) and isinstance(value.get("0"), list):
        return value["0"]
    return []


def _validate_water_meal_intervention(record: dict, source: str) -> None:
    before = record.get("pre_choice")
    perception = record.get("perception")
    original = record.get("original_selected_decision")
    actual = record.get("actual_decision")
    if not isinstance(before, dict) or not isinstance(original, dict) or not isinstance(actual, dict):
        raise ValueError(f"incomplete water-meal intervention audit: {source}")
    if record.get("original_reason") not in ("SeekWater", "LearnedWater", "RecallWater"):
        raise ValueError(f"unapproved replaced water reason: {source}")
    if original.get("reason") != record.get("original_reason"):
        raise ValueError(f"original water decision reason mismatch: {source}")
    original_intent = original.get("intent", {})
    drink = original_intent.get("Drink") if isinstance(original_intent, dict) else None
    if not isinstance(drink, dict):
        raise ValueError(f"replaced decision was not Drink: {source}")
    selected_path = _steps(drink.get("at"))
    if len(selected_path) <= 1 or selected_path != _steps(record.get("original_selected_water_path")):
        raise ValueError(f"original selected water route missing or inconsistent: {source}")
    if not isinstance(record.get("original_water_amount"), int) or record["original_water_amount"] <= 0:
        raise ValueError(f"original selected water has no positive recorded source amount: {source}")
    if isinstance(perception, dict) and not any(w.get("amount", 0) > 0 and _steps(w.get("steps")) == selected_path for w in perception.get("waters", [])):
        raise ValueError(f"original water route does not match visible positive water: {source}")
    if actual.get("reason") != "EatBeforeWater":
        raise ValueError(f"actual decision reason is not EatBeforeWater: {source}")
    actual_intent = actual.get("intent", {})
    eat = actual_intent.get("Eat") if isinstance(actual_intent, dict) else None
    if not isinstance(eat, dict) or eat.get("corpse_id") is not None:
        raise ValueError(f"intervention actual decision is not non-corpse Eat: {source}")
    food_path = _steps(eat.get("at"))
    if len(food_path) > 1:
        raise ValueError(f"intervention meal is not immediately reachable: {source}")
    selected_food = record.get("actual_selected_food")
    if not isinstance(selected_food, dict) or selected_food.get("corpse_id") is not None or _steps(selected_food.get("steps")) != food_path:
        raise ValueError(f"recorded selected plant-food path differs from actual Eat intent: {source}")
    if not isinstance(record.get("actual_selected_food_amount"), int) or record["actual_selected_food_amount"] <= 0:
        raise ValueError(f"intervention has no positive recorded plant-food source amount: {source}")
    if isinstance(perception, dict) and not any(f.get("amount", 0) > 0 and not f.get("is_corpse") and f.get("corpse_id") is None and _steps(f.get("steps")) == food_path for f in perception.get("foods", [])):
        raise ValueError(f"intervention meal does not match visible positive plant food: {source}")
    if before.get("energy", 0) >= record.get("movement_energy_cost", 0):
        raise ValueError(f"intervention energy was not below one move cost: {source}")
    if record.get("movement_energy_cost", 0) <= 0 or record.get("effective_speed", 0) <= record.get("moves_used", 0) or record.get("remaining_moves", 0) <= 0 or record.get("stamina", 0) <= 0:
        raise ValueError(f"intervention lacked first-step movement capacity: {source}")


def _read_run(path: Path, seed: int, habitat: str, arm: str) -> dict:
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != TICKS + 2 or rows[0].get("type") != "header" or rows[-1].get("type") != "summary":
        raise ValueError(f"malformed/incomplete JSONL: {path.name}")
    header = rows[0]
    if (header.get("schema") != "curious-water-meal-priority-v1-jsonl" or header.get("seed") != seed
            or header.get("habitat") != habitat or header.get("arm") != arm):
        raise ValueError(f"raw identity mismatch: {path.name}")
    ticks = rows[1:-1]
    if [r.get("tick") for r in ticks] != list(range(1, TICKS + 1)):
        raise ValueError(f"tick sequence incomplete in {path.name}")
    sums = [0, 0, 0]
    diagnostic_sums = {k: 0 for k in ("focal_approach_out_of_range_count", "focal_matched_futile_water_approaches", "focal_zero_paid_water_approaches", "focal_underfunded_water_approaches", "focal_zero_energy_visible_food_approach_failures", "focal_underfunded_visible_food_approach_failures")}
    for row in ticks:
        if (row.get("type") != "tick" or not all(isinstance(row.get(k), list) for k in
                ("birth_lineage", "mating_events", "death_lineage", "focal_events", "actor_resolver_events", "water_meal_interventions", "reserve_interventions"))
                or not isinstance(row.get("reason_counts"), dict)):
            raise ValueError(f"required per-tick records missing: {path.name}")
        if row["reason_counts"].get("FoodSearchReserve", 0) != len(row["reserve_interventions"]):
            raise ValueError(f"food-reserve reason count mismatch: {path.name}")
        if row["reason_counts"].get("EatBeforeWater", 0) != len(row["water_meal_interventions"]):
            raise ValueError(f"water-meal reason count mismatch: {path.name}")
        for record in row["water_meal_interventions"]:
            _validate_water_meal_intervention(record, path.name)
        if row.get("focal_decision_trace") is not None and row["focal_decision_trace"].get("id") != 1:
            raise ValueError(f"focal trace identity mismatch: {path.name}")
        event_totals = _event_totals(row)
        for i, (key, value) in enumerate(zip(("plant_food_gained_energy", "water_gained", "movement_energy_paid"), event_totals)):
            if value != row.get(key):
                raise ValueError(f"resolver event/tick total mismatch for {key}: {path.name}")
            sums[i] += value
        for key, count in _diagnostics_for_tick(row).items():
            diagnostic_sums[key] += count
            if row.get(key) != diagnostic_sums[key]:
                raise ValueError(f"focal diagnostic counter mismatch for {key}: {path.name}")
    summary = rows[-1]
    if summary.get("ticks_completed") != TICKS or summary.get("checkpoint_restore_valid") is not True:
        raise ValueError(f"horizon/checkpoint validation failed: {path.name}")
    if summary.get("lineage_issues") != 0 or summary.get("unknown_generation_births") != 0:
        raise ValueError(f"lineage attribution incomplete: {path.name}")
    for key, value in zip(("plant_food_gained_energy", "water_gained", "movement_energy_paid"), sums):
        if summary.get(key) != value:
            raise ValueError(f"resolver totals/summary mismatch for {key}: {path.name}")
    for key, value in diagnostic_sums.items():
        if summary.get(key) != value:
            raise ValueError(f"focal diagnostic/summary mismatch for {key}: {path.name}")
    births = [b for row in ticks for b in row["birth_lineage"]]
    matings = [m for row in ticks for m in row["mating_events"]]
    deaths = [d for row in ticks for d in row["death_lineage"]]
    if len(births) != len(matings):
        raise ValueError(f"birth/mating counts differ: {path.name}")
    by_id, birth_species, birth_gen = {}, {}, {}
    for b in births:
        if not isinstance(b.get("id"), int) or not isinstance(b.get("generation"), int) or not isinstance(b.get("parent_ids"), list) or len(b["parent_ids"]) != 2 or b["id"] in by_id:
            raise ValueError(f"invalid birth lineage: {path.name}")
        by_id[b["id"]] = b
        birth_species[b["species"]] = birth_species.get(b["species"], 0) + 1
        birth_gen[str(b["generation"])] = birth_gen.get(str(b["generation"]), 0) + 1
    mated = set()
    for m in matings:
        child = m.get("child")
        if child not in by_id or child in mated or set(by_id[child]["parent_ids"]) != {m.get("parent1"), m.get("parent2")} or set(m.get("child_parent_ids") or []) != set(by_id[child]["parent_ids"]):
            raise ValueError(f"mating event does not reconcile with birth: {path.name}")
        mated.add(child)
    if mated != set(by_id):
        raise ValueError(f"mating stream does not cover births: {path.name}")
    death_species, death_gen = {}, {}
    for d in deaths:
        if not isinstance(d.get("generation"), int):
            raise ValueError(f"unknown death generation: {path.name}")
        death_species[d["species"]] = death_species.get(d["species"], 0) + 1
        death_gen[str(d["generation"])] = death_gen.get(str(d["generation"]), 0) + 1
    for actual, key in ((birth_species,"births_by_species"),(birth_gen,"births_by_generation"),(death_species,"deaths_by_species"),(death_gen,"deaths_by_generation")):
        if actual != _nonzero(summary.get(key, {})):
            raise ValueError(f"lineage totals mismatch for {key}: {path.name}")
    if summary.get("f1_births", 0) != birth_gen.get("1", 0) or summary.get("f2_births", 0) != birth_gen.get("2", 0):
        raise ValueError(f"generation birth totals mismatch: {path.name}")
    initial_count = len((header.get("initial_state", {}).get("entities") or []))
    final_count = sum(summary.get("final_alive", {}).values())
    if initial_count + len(births) - len(deaths) != final_count:
        raise ValueError(f"population balance failed: {path.name}")
    if (summary.get("water_meal_intervention_count") != sum(len(r["water_meal_interventions"]) for r in ticks)
            or summary.get("reserve_intervention_count") != sum(len(r["reserve_interventions"]) for r in ticks)):
        raise ValueError(f"intervention total mismatch: {path.name}")
    initial = header.get("initial_state", {})
    return {"header":header,"summary":summary,"rows":ticks,"initial":initial}


def _normalize_state(state: dict) -> dict:
    normalized = json.loads(json.dumps(state))
    policy = normalized.get("policy")
    if not isinstance(policy, dict):
        raise ValueError("initial checkpoint omits policy")
    policy["food_search_reserve_enabled"] = False
    policy["water_meal_priority_enabled"] = False
    return normalized


def _numeric_metrics(summary: dict) -> dict[str, int | float]:
    out = {}
    lists = ("f1_breeders", "living_f1_breeders", "f2_breeders", "living_f2_breeders", "living_descendant_breeders")
    maps = ("final_alive", "births_by_species", "deaths_by_species", "births_by_generation", "deaths_by_generation", "death_causes")
    for key, value in summary.items():
        if key in lists and isinstance(value, list):
            out[f"{key}_count"] = len(value)
        elif isinstance(value, (int, float)) and not isinstance(value, bool):
            out[key] = value
        elif key in maps and isinstance(value, dict):
            for k, count in value.items():
                if isinstance(count, (int, float)) and not isinstance(count, bool):
                    out[f"{key}.{k}"] = count
    return out


def _compact_summary(summary: dict) -> dict:
    """Keep report values in manifests; raw JSONL owns the full checkpoint State."""
    return {k: v for k, v in summary.items() if k != "final_state"}


def validate_block(worlds: dict, habitat: str, seed: int) -> dict:
    expected = {"control", "food_reserve_only", "water_meal_only", "bundle"}
    if set(worlds) != expected:
        raise ValueError("factorial block does not contain exactly four arms")
    normalized = None
    metrics = {}
    for arm, run in worlds.items():
        if run["header"].get("habitat") != habitat or run["header"].get("seed") != seed or run["header"].get("arm") != arm:
            raise ValueError("factorial block identity mismatch")
        expected_flags = {
            "control": (False, False), "food_reserve_only": (True, False),
            "water_meal_only": (False, True), "bundle": (True, True),
        }[arm]
        actual = run["header"].get("candidate_policy", {})
        if (actual.get("food_search_reserve_enabled"), actual.get("water_meal_priority_enabled")) != expected_flags:
            raise ValueError(f"arm {arm} flags differ from frozen factorial definition")
        policy = run["initial"].get("policy", {})
        if (policy.get("food_search_reserve_enabled", False), policy.get("water_meal_priority_enabled", False)) != expected_flags:
            raise ValueError(f"arm {arm} initial checkpoint flags mismatch")
        if any((not expected_flags[0] and r["reserve_interventions"]) or
               (not expected_flags[1] and r["water_meal_interventions"]) for r in run["rows"]):
            raise ValueError(f"arm {arm} emitted an intervention for a disabled flag")
        state = _normalize_state(run["initial"])
        if normalized is None:
            normalized = state
        elif state != normalized:
            raise ValueError("factorial initial State differs beyond registered flags")
        metrics[arm] = _numeric_metrics(run["summary"])
    keys = set.union(*(set(m) for m in metrics.values()))
    deltas = {
        arm: {
            k: metrics[arm].get(k, 0) - metrics["control"].get(k, 0)
            for k in keys
            if (k in metrics[arm] and k in metrics["control"]) or "." in k
        }
        for arm in ARMS[1:]
    }
    interaction = {k: deltas["bundle"][k] - deltas["food_reserve_only"][k] - deltas["water_meal_only"][k] for k in keys if all(k in deltas[a] for a in ARMS[1:])}
    return {"habitat":habitat,"seed":seed,"arms":{a:_compact_summary(worlds[a]["summary"]) for a in ARMS},"paired_deltas_vs_control":deltas,"factorial_interaction":interaction}


def _paired_aggregate(blocks: list[dict]) -> dict:
    fields = sorted(set().union(*(set(b["factorial_interaction"]) | set().union(*(set(d) for d in b["paired_deltas_vs_control"].values())) for b in blocks))) if blocks else []
    out = {}
    for field in fields:
        out[field] = {}
        for contrast in (*ARMS[1:], "interaction"):
            vals = [b["factorial_interaction"][field] if contrast == "interaction" else b["paired_deltas_vs_control"][contrast][field] for b in blocks if field in (b["factorial_interaction"] if contrast == "interaction" else b["paired_deltas_vs_control"][contrast])]
            out[field][contrast] = {"n_supported_pairs":len(vals),"n_registered_pairs":len(blocks),"paired_deltas":vals,"median":statistics.median(vals) if vals else None,"range":[min(vals),max(vals)] if vals else None}
    return out


def _finalize_manifest(out: Path, manifest: dict) -> int:
    other = sum(p.stat().st_size for p in out.rglob("*") if p.is_file() and p.name != "manifest.json")
    manifest_path = out / "manifest.json"
    for _ in range(8):
        manifest["aggregate_output_bytes"] = other + len(_json_bytes(manifest))
        size = manifest["aggregate_output_bytes"]
        if size > OUTPUT_CAP:
            raise OSError("aggregate output cap exceeded")
        _atomic_json(manifest_path, manifest)
        actual = _total_bytes(out)
        if actual == size:
            return actual
        other = actual - manifest_path.stat().st_size
    raise RuntimeError("manifest output-size accounting did not converge")


def run(registration_path: Path, output_dir: Path, seed_archive: Path, binary: Path) -> int:
    started = time.monotonic()
    identity, fixtures, binary_hash = source_identity(), scenario_hashes(), sha256(binary)
    registration = json.loads(registration_path.read_text(encoding="utf-8"))
    validate_registration(registration, identity, fixtures, binary_hash)
    inventory = scan_seed_roles(seed_archive, registration["seed_roles"])
    output_dir.mkdir(parents=True, exist_ok=False)
    shutil.copy2(registration_path, output_dir / "registration.json")
    manifest = {"schema":"curious-water-meal-priority-v1-manifest","status":"running","started_unix":time.time(),"source":identity,"scenario_sha256":fixtures,"protocol_sha256":sha256(ROOT/PROTOCOL),"binary_sha256":binary_hash,"seed_inventory":inventory,"ticks_per_world":TICKS,"arms":list(ARMS),"worlds":[],"blocks":[],"aggregate_output_bytes":0}
    _atomic_json(output_dir / "manifest.json", manifest)
    parsed = {}
    try:
        for role in registration["seed_roles"]:
            habitat, seed = role["habitat"], role["seed"]
            scenario = HABITATS[habitat][0]
            for arm in ARMS:
                if time.monotonic() - started >= PHASE_SECONDS:
                    raise TimeoutError("300-second phase budget exceeded")
                stem = f"{habitat}-seed{seed}-{arm}"
                raw, stdout, stderr = (output_dir / f"{stem}.{ext}" for ext in ("jsonl", "stdout", "stderr"))
                command = [str(binary), scenario, str(seed), arm, habitat, str(raw)]
                before = time.monotonic()
                with stdout.open("xb") as so, stderr.open("xb") as se:
                    proc = subprocess.Popen(command, cwd=ROOT, env=os.environ.copy(), stdout=so, stderr=se, preexec_fn=_child_limits, start_new_session=True)
                    while proc.poll() is None:
                        if time.monotonic() - before >= CHILD_SECONDS or time.monotonic() - started >= PHASE_SECONDS:
                            proc.kill(); proc.wait(); raise TimeoutError(f"world timeout: {stem}")
                        if _total_bytes(output_dir) > OUTPUT_CAP:
                            proc.kill(); proc.wait(); raise OSError("aggregate output cap exceeded")
                        time.sleep(.05)
                if proc.returncode != 0:
                    raise RuntimeError(f"child failed ({proc.returncode}): {stem}")
                if raw.stat().st_size > MAX_WORLD_OUTPUT:
                    raise OSError(f"per-world output cap exceeded: {stem}")
                run_data = _read_run(raw, seed, habitat, arm)
                parsed[(habitat, seed, arm)] = run_data
                manifest["worlds"].append({"habitat":habitat,"seed":seed,"arm":arm,"raw":raw.name,"raw_sha256":sha256(raw),"raw_bytes":raw.stat().st_size,"stdout":stdout.name,"stderr":stderr.name,"summary":_compact_summary(run_data["summary"])})
                manifest["aggregate_output_bytes"] = _total_bytes(output_dir)
                _atomic_json(output_dir / "manifest.json", manifest)
        for habitat, (_, seeds) in HABITATS.items():
            for seed in seeds:
                block = {arm:parsed[(habitat,seed,arm)] for arm in ARMS}
                manifest["blocks"].append(validate_block(block, habitat, seed))
        aggregate = {habitat:_paired_aggregate([b for b in manifest["blocks"] if b["habitat"] == habitat]) for habitat in HABITATS}
        _atomic_json(output_dir / "paired_summary.json", {"schema":"curious-water-meal-priority-v1-paired-summary","interpretation":"factorial descriptive screen; no composite utility or acceptance threshold","per_seed_blocks":manifest["blocks"],"by_fixture":aggregate})
        manifest.update(status="complete",completed_unix=time.time(),elapsed_seconds=round(time.monotonic()-started,3))
        _finalize_manifest(output_dir, manifest)
        return 0
    except BaseException as exc:
        manifest.update(status="incomplete",stopped_unix=time.time(),elapsed_seconds=round(time.monotonic()-started,3),error=f"{type(exc).__name__}: {exc}")
        try:
            _finalize_manifest(output_dir, manifest)
        except Exception:
            _atomic_json(output_dir / "manifest.json", manifest)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registration", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--seed-archive", type=Path, default=ROOT / "data/experiments")
    parser.add_argument("--binary", required=True, type=Path)
    args = parser.parse_args()
    try:
        return run(args.registration.resolve(), args.output_dir.resolve(), args.seed_archive.resolve(), args.binary.resolve())
    except Exception as exc:
        print(f"run_water_meal_priority_lab: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2

if __name__ == "__main__":
    raise SystemExit(main())
