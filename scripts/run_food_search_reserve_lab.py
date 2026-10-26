#!/usr/bin/env python3
"""Registered supervisor for the food-search reserve paired screen.

Requires a root-reviewed registration and root-prebuilt binary. This runner does
not build code or choose seeds/parameters.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import shutil
import statistics
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
HABITATS = {
    "v1_unique_food": ("v1", list(range(62401, 62407))),
    "meadow": ("scenarios/meadow.json", list(range(62401, 62407))),
    "sparse_resource": ("scenarios/ecology-traits-lab.json", list(range(62401, 62407))),
}
ARMS = ("baseline", "candidate")
TICKS = 300
PHASE_SECONDS = 300
CHILD_SECONDS = 60
CHILD_ADDRESS_SPACE = 1024 * 1024 * 1024
OUTPUT_CAP = 256 * 1024 * 1024
MAX_WORLD_OUTPUT = 16 * 1024 * 1024
MAX_IDENTITIES = 10_000
MAX_EVENTS_PER_TICK = 20_000
SEED_SCAN_MAX_LINE_BYTES = 64 * 1024 * 1024
PROTOCOL = "docs/FOOD_SEARCH_RESERVE_GUARD_V1_PROTOCOL_20261026.md"
SOURCE_PATHS = (
    "Cargo.toml", "Cargo.lock", "examples/food_search_reserve_lab.rs",
    "scripts/run_food_search_reserve_lab.py", "scripts/test_food_search_reserve_lab.py",
    "scripts/run_ecology_traits_lab.py",
    "scripts/evidence_archive.py", "docs/EVIDENCE_ARCHIVES.md",
    PROTOCOL, "docs/FOOD_SEARCH_RESERVE.md",
    "docs/FOOD_SEARCH_RESERVE_GUARD_V1_REGISTRATION_TEMPLATE.json",
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
        raise ValueError("registered seed roles differ from frozen 62401..62406 allocation")
    if registration.get("binary_sha256") != binary_sha:
        raise ValueError("prebuilt binary SHA-256 differs from registration")
    expected = {
        "ticks_per_world": TICKS, "arms": list(ARMS), "habitats": list(HABITATS),
        "world_count": 36, "phase_seconds": PHASE_SECONDS, "child_seconds": CHILD_SECONDS,
        "child_address_space_bytes": CHILD_ADDRESS_SPACE, "aggregate_output_bytes": OUTPUT_CAP,
        "max_world_output_bytes": MAX_WORLD_OUTPUT, "max_identities": MAX_IDENTITIES,
        "max_events_per_tick": MAX_EVENTS_PER_TICK, "mutation_per_mille": 0,
        "genetics_enabled": True, "candidate_flag": "food_search_reserve_enabled",
    }
    for key, value in expected.items():
        if registration.get(key) != value:
            raise ValueError(f"registration {key} differs from frozen protocol")


def scan_seed_roles(archive: Path, roles: list[dict]) -> dict:
    helper = ROOT / "scripts/run_ecology_traits_lab.py"
    spec = importlib.util.spec_from_file_location("food_reserve_seed_inventory", helper)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load existing seed inventory scanner")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    seeds = sorted({r["seed"] for r in roles})
    audit = mod.audit_seeds(archive, seeds, max_line_bytes=SEED_SCAN_MAX_LINE_BYTES)
    matches = audit["matches"]
    compressed_files = sorted(p for p in archive.rglob("*.jsonl.zst") if p.is_file()) if archive.exists() else []
    compressed_bytes = 0
    index_rows = []
    for path in compressed_files:
        rel = str(path.relative_to(archive))
        original = path.with_suffix("")
        index_path = Path(str(original) + ".archive.json")
        if not index_path.is_file():
            raise RuntimeError(f"compressed JSONL has no verified seed index: {rel}")
        index = json.loads(index_path.read_text(encoding="utf-8"))
        archive_rel = str(path.relative_to(ROOT))
        index_rel = str(index_path.relative_to(ROOT))
        if index.get("status") != "verified" or index.get("codec") != "zstd" or index.get("archivepath") != archive_rel:
            raise RuntimeError(f"compressed JSONL index identity/status mismatch: {index_rel}")
        if index.get("archive_size") != path.stat().st_size or index.get("archive_sha256") != sha256(path):
            raise RuntimeError(f"compressed JSONL hash/size differs from index: {rel}")
        declared = sorted(set(index.get("seeds", [])))
        if any(not isinstance(seed, int) for seed in declared):
            raise RuntimeError(f"invalid seed roles in compressed JSONL index: {index_rel}")
        for seed in set(declared).intersection(seeds):
            bucket = matches[str(seed)]
            if index_rel not in bucket:
                bucket.append(index_rel)
        compressed_bytes += path.stat().st_size
        index_rows.append({"path":archive_rel,"index":index_rel,"sha256":index["archive_sha256"],"size":index["archive_size"],"seeds":declared})
    audit.update({
        "matches": matches,
        "unseen": [seed for seed in seeds if not matches[str(seed)]],
        "scanned_compressed_jsonl_files": len(compressed_files),
        "scanned_compressed_jsonl_bytes": compressed_bytes,
        "verified_compressed_jsonl_indexes": index_rows,
        "max_line_bytes": SEED_SCAN_MAX_LINE_BYTES,
        "compressed_rule": "verify every *.jsonl.zst archive SHA-256/size against its verified sibling archive index and include the index's declared seed roles",
    })
    if audit.get("unseen") != seeds:
        raise ValueError(f"one or more registered seeds already occur in experiment inventory: {audit.get('matches')}")
    return audit


def _child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (CHILD_ADDRESS_SPACE, CHILD_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_CPU, (CHILD_SECONDS, CHILD_SECONDS))


def _atomic_json(path: Path, value: dict) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temp, path)


def _total_bytes(root: Path) -> int:
    return sum(p.stat().st_size for p in root.rglob("*") if p.is_file())


def _nonzero(value: dict) -> dict:
    return {str(k): v for k, v in value.items() if v != 0}


def _read_run(path: Path, expected_seed: int, expected_habitat: str, expected_arm: str) -> dict:
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(rows) != TICKS + 2 or rows[0].get("type") != "header" or rows[-1].get("type") != "summary":
        raise ValueError(f"malformed/incomplete run JSONL: {path.name}")
    header = rows[0]
    if (header.get("schema") != "curious-food-search-reserve-v1-jsonl" or header.get("seed") != expected_seed
            or header.get("habitat") != expected_habitat or header.get("arm") != expected_arm):
        raise ValueError(f"raw identity mismatch: {path.name}")
    if [r.get("tick") for r in rows[1:-1]] != list(range(1, TICKS + 1)):
        raise ValueError(f"tick sequence incomplete in {path.name}")
    for row in rows[1:-1]:
        if row.get("type") != "tick" or not isinstance(row.get("birth_lineage"), list) or not isinstance(row.get("mating_events"), list) or not isinstance(row.get("death_lineage"), list) or not isinstance(row.get("focal_events"), list) or not isinstance(row.get("reason_counts"), dict) or not isinstance(row.get("reserve_interventions"), list):
            raise ValueError(f"required per-tick records missing in {path.name}")
        if row.get("focal_decision_trace") is not None and row["focal_decision_trace"].get("id") != 1:
            raise ValueError(f"focal trace ID mismatch in {path.name}")
        if row["reason_counts"].get("FoodSearchReserve", 0) != len(row["reserve_interventions"]):
            raise ValueError(f"reserve intervention rows do not reconcile with reason counts: {path.name}")
        for intervention in row["reserve_interventions"]:
            required = ("decision_energy", "movement_energy_cost", "known_next_tick_maintenance", "energy_cap", "remaining_speed_cap", "applied_cap", "input_path_len", "chosen_steps", "moves_used", "effective_speed")
            if any(not isinstance(intervention.get(key), int) for key in required if key != "input_path_len") or intervention.get("input_path_len") is not None:
                raise ValueError(f"malformed reserve intervention record: {path.name}")
            energy = max(intervention["decision_energy"] - intervention["known_next_tick_maintenance"], 0)
            if intervention["movement_energy_cost"] <= 0 or intervention["energy_cap"] != energy // (2 * intervention["movement_energy_cost"]):
                raise ValueError(f"energy cap formula mismatch: {path.name}")
            speed_cap = max(intervention["effective_speed"] - intervention["moves_used"], 0)
            applied = min(intervention["energy_cap"], speed_cap)
            if intervention["remaining_speed_cap"] != speed_cap or intervention["applied_cap"] != applied or intervention["chosen_steps"] != applied:
                raise ValueError(f"applied reserve cap differs from emitted intent: {path.name}")
    summary = rows[-1]
    if summary.get("ticks_completed") != TICKS or summary.get("checkpoint_restore_valid") is not True:
        raise ValueError(f"horizon or endpoint checkpoint validation failed: {path.name}")
    if summary.get("lineage_issues") != 0 or summary.get("unknown_generation_births") != 0:
        raise ValueError(f"birth/parent/generation attribution incomplete: {path.name}")
    birth_records = [r for row in rows[1:-1] for r in row["birth_lineage"]]
    mating_records = [r for row in rows[1:-1] for r in row["mating_events"]]
    death_records = [r for row in rows[1:-1] for r in row["death_lineage"]]
    if len(birth_records) != len(mating_records):
        raise ValueError(f"birth/mating record counts differ: {path.name}")
    by_id = {}
    births_species: dict[str, int] = {}; births_gen: dict[str, int] = {}
    for b in birth_records:
        if not isinstance(b.get("id"), int) or not isinstance(b.get("generation"), int) or not isinstance(b.get("parent_ids"), list) or len(b["parent_ids"]) != 2:
            raise ValueError(f"invalid compact birth lineage: {path.name}")
        if b["id"] in by_id:
            raise ValueError(f"duplicate birth identity: {path.name}")
        by_id[b["id"]] = b
        births_species[b["species"]] = births_species.get(b["species"], 0) + 1
        key = str(b["generation"]); births_gen[key] = births_gen.get(key, 0) + 1
    mated = set()
    for m in mating_records:
        child = m.get("child")
        if child not in by_id or child in mated or set(by_id[child]["parent_ids"]) != {m.get("parent1"), m.get("parent2")}:
            raise ValueError(f"mating event does not match birth lineage: {path.name}")
        if m.get("child_parent_ids") is None or set(m["child_parent_ids"]) != set(by_id[child]["parent_ids"]):
            raise ValueError(f"child slot parent IDs differ from mating event: {path.name}")
        mated.add(child)
    if mated != set(by_id):
        raise ValueError(f"mating events do not cover birth lineage: {path.name}")
    deaths_species: dict[str, int] = {}; deaths_gen: dict[str, int] = {}
    for d in death_records:
        if not isinstance(d.get("generation"), int):
            raise ValueError(f"unknown death generation: {path.name}")
        deaths_species[d["species"]] = deaths_species.get(d["species"], 0) + 1
        key = str(d["generation"]); deaths_gen[key] = deaths_gen.get(key, 0) + 1
    if births_species != _nonzero(summary.get("births_by_species", {})) or births_gen != _nonzero(summary.get("births_by_generation", {})):
        raise ValueError(f"birth totals differ from compact lineage: {path.name}")
    if deaths_species != _nonzero(summary.get("deaths_by_species", {})) or deaths_gen != _nonzero(summary.get("deaths_by_generation", {})):
        raise ValueError(f"death totals differ from compact lineage: {path.name}")
    if summary.get("f1_births", 0) != births_gen.get("1", 0) or summary.get("f2_births", 0) != births_gen.get("2", 0):
        raise ValueError(f"F1/F2 totals differ from lineage: {path.name}")
    if sum(len(row["reserve_interventions"]) for row in rows[1:-1]) != summary.get("reserve_intervention_count"):
        raise ValueError(f"reserve intervention total differs from tick rows: {path.name}")
    if rows[-2].get("focal_zero_energy_visible_food_approach_failures") != summary.get("focal_zero_energy_visible_food_approach_failures") or rows[-2].get("focal_underfunded_visible_food_approach_failures") != summary.get("focal_underfunded_visible_food_approach_failures"):
        raise ValueError(f"focal route-lock counters differ from tick rows: {path.name}")
    initial = header.get("initial_state", {}).get("entities", [])
    births = sum(summary.get("births_by_species", {}).values())
    deaths = sum(summary.get("deaths_by_species", {}).values())
    final_live = sum(summary.get("final_alive", {}).values())
    if len(initial) + births - deaths != final_live:
        raise ValueError(f"population balance failed: {path.name}")
    return {"header": header, "summary": summary, "rows": rows[1:-1]}


def _normalized_initial(state: dict) -> dict:
    normalized = json.loads(json.dumps(state))
    policy = normalized.get("policy")
    if not isinstance(policy, dict):
        raise ValueError("initial checkpoint omits policy settings")
    policy["food_search_reserve_enabled"] = False
    return normalized


def _numeric_metrics(summary: dict) -> dict[str, int | float]:
    metrics: dict[str, int | float] = {}
    id_lists = ("f1_breeders", "living_f1_breeders", "f2_breeders", "living_f2_breeders", "living_descendant_breeders")
    count_maps = ("final_alive", "births_by_species", "deaths_by_species", "births_by_generation", "deaths_by_generation", "death_causes")
    for key, value in summary.items():
        if key in id_lists and isinstance(value, list):
            metrics[f"{key}_count"] = len(value)
        elif isinstance(value, (int, float)) and not isinstance(value, bool):
            metrics[key] = value
        elif key in count_maps and isinstance(value, dict):
            for subkey, count in value.items():
                if isinstance(count, (int, float)) and not isinstance(count, bool):
                    metrics[f"{key}.{subkey}"] = count
    return metrics


def validate_pair(baseline: dict, candidate: dict, habitat: str, seed: int) -> dict:
    if baseline["header"]["habitat"] != habitat or candidate["header"]["habitat"] != habitat or baseline["header"]["seed"] != seed or candidate["header"]["seed"] != seed:
        raise ValueError("paired raw identity mismatch")
    if baseline["header"]["arm"] != "baseline" or candidate["header"]["arm"] != "candidate":
        raise ValueError("paired arms are not baseline/candidate")
    if baseline["header"].get("candidate_policy", {}).get("food_search_reserve_enabled") is not False:
        raise ValueError("baseline did not leave candidate flag off")
    if candidate["header"].get("candidate_policy", {}).get("food_search_reserve_enabled") is not True:
        raise ValueError("candidate did not enable exactly registered flag")
    base_flag = baseline["header"]["initial_state"].get("policy", {}).get("food_search_reserve_enabled", False)
    candidate_flag = candidate["header"]["initial_state"].get("policy", {}).get("food_search_reserve_enabled", False)
    if base_flag is not False or candidate_flag is not True:
        raise ValueError("initial checkpoint policy flags disagree with paired arm headers")
    if any(row["reserve_interventions"] for row in baseline["rows"]):
        raise ValueError("baseline emitted candidate intervention records")
    if _normalized_initial(baseline["header"]["initial_state"]) != _normalized_initial(candidate["header"]["initial_state"]):
        raise ValueError("paired initial State differs beyond candidate flag")
    before = _numeric_metrics(baseline["summary"])
    after = _numeric_metrics(candidate["summary"])
    fields = sorted(set(before).union(after))
    deltas = {}
    for key in fields:
        if key in before and key in after:
            deltas[key] = after[key] - before[key]
        elif "." in key:
            # Missing species/generation/death-cause categories in count maps mean zero.
            deltas[key] = after.get(key, 0) - before.get(key, 0)
    return {"habitat": habitat, "seed": seed, "baseline": baseline["summary"], "candidate": candidate["summary"], "candidate_minus_baseline": deltas}


def _write_manifest(out: Path, manifest: dict) -> None:
    _atomic_json(out / "manifest.json", manifest)


def run(registration_path: Path, output_dir: Path, seed_archive: Path, binary: Path) -> int:
    started = time.monotonic()
    identity = source_identity(); fixtures = scenario_hashes(); binary_hash = sha256(binary)
    registration = json.loads(registration_path.read_text(encoding="utf-8"))
    validate_registration(registration, identity, fixtures, binary_hash)
    inventory = scan_seed_roles(seed_archive, registration["seed_roles"])
    if output_dir.exists():
        raise FileExistsError(f"output directory already exists: {output_dir}")
    output_dir.mkdir(parents=True, exist_ok=False)
    shutil.copy2(registration_path, output_dir / "registration.json")
    manifest = {"schema":"curious-food-search-reserve-v1-manifest","status":"running","started_unix":time.time(),"source":identity,"scenario_sha256":fixtures,"binary_sha256":binary_hash,"seed_inventory":inventory,"ticks_per_world":TICKS,"arms":list(ARMS),"worlds":[],"pairs":[],"aggregate_output_bytes":0}
    _write_manifest(output_dir, manifest)
    env = os.environ.copy()
    parsed: dict[tuple[str,int,str], dict] = {}
    try:
        for role in registration["seed_roles"]:
            habitat, seed = role["habitat"], role["seed"]
            scenario = HABITATS[habitat][0]
            for arm in ARMS:
                if time.monotonic() - started >= PHASE_SECONDS:
                    raise TimeoutError("300-second phase budget exceeded")
                stem = f"{habitat}-seed{seed}-{arm}"
                raw = output_dir / f"{stem}.jsonl"; stdout = output_dir / f"{stem}.stdout"; stderr = output_dir / f"{stem}.stderr"
                command = [str(binary), scenario, str(seed), arm, habitat, str(raw)]
                before = time.monotonic()
                with stdout.open("xb") as stdout_file, stderr.open("xb") as stderr_file:
                    proc = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stdout_file, stderr=stderr_file, preexec_fn=_child_limits, start_new_session=True)
                    while proc.poll() is None:
                        if time.monotonic() - before >= CHILD_SECONDS:
                            proc.kill(); proc.wait(); raise TimeoutError(f"60-second child limit exceeded: {stem}")
                        if time.monotonic() - started >= PHASE_SECONDS:
                            proc.kill(); proc.wait(); raise TimeoutError("300-second phase budget exceeded")
                        if _total_bytes(output_dir) > OUTPUT_CAP:
                            proc.kill(); proc.wait(); raise OSError("aggregate output cap exceeded")
                        time.sleep(0.05)
                if _total_bytes(output_dir) > OUTPUT_CAP: raise OSError("aggregate output cap exceeded after child exit")
                if proc.returncode != 0: raise RuntimeError(f"child failed ({proc.returncode}): {stem}")
                if raw.stat().st_size > MAX_WORLD_OUTPUT: raise OSError(f"per-world output cap exceeded: {stem}")
                run_data = _read_run(raw, seed, habitat, arm); parsed[(habitat,seed,arm)] = run_data
                s = run_data["summary"]
                manifest["worlds"].append({"habitat":habitat,"seed":seed,"arm":arm,"raw":raw.name,"raw_sha256":sha256(raw),"raw_bytes":raw.stat().st_size,"stdout":stdout.name,"stderr":stderr.name,"summary":{k:s.get(k) for k in ("ticks_completed","living_entity_ticks","zero_energy_alive_tick_sum","first_zero_energy_tick","focal_first_zero_energy_tick","focal_death_tick","focal_death_cause","focal_ticks_from_first_zero_to_death","plant_food_gained_energy","movement_energy_paid","plant_food_to_movement_energy_ratio","focal_approach_out_of_range_count","focal_zero_energy_visible_food_approach_failures","focal_underfunded_visible_food_approach_failures","reserve_intervention_count","final_alive","founders_alive","births_by_species","deaths_by_species","births_by_generation","deaths_by_generation","f1_births","f2_births","f1_breeders","living_f1_breeders","f2_breeders","living_f2_breeders","living_descendant_breeders","death_causes","lineage_issues","unknown_generation_births","checkpoint_restore_valid")}})
                manifest["aggregate_output_bytes"] = _total_bytes(output_dir); _write_manifest(output_dir, manifest)
        for habitat, (_, seeds) in HABITATS.items():
            for seed in seeds:
                pair = validate_pair(parsed[(habitat,seed,"baseline")], parsed[(habitat,seed,"candidate")], habitat, seed)
                manifest["pairs"].append(pair)
        fields = sorted({field for pair in manifest["pairs"] for field in pair["candidate_minus_baseline"]})
        aggregate = {}
        for habitat in HABITATS:
            habitat_pairs = [p for p in manifest["pairs"] if p["habitat"] == habitat]
            aggregate[habitat] = {}
            for field in fields:
                values = [p["candidate_minus_baseline"][field] for p in habitat_pairs if field in p["candidate_minus_baseline"]]
                aggregate[habitat][field] = {"n_pairs":len(values),"paired_deltas":values,"median":statistics.median(values) if values else None,"range":[min(values),max(values)] if values else None}
        _atomic_json(output_dir / "paired_summary.json", {"schema":"curious-food-search-reserve-v1-paired-summary","interpretation":"descriptive paired screen; no composite utility or acceptance threshold","per_seed_pairs":manifest["pairs"],"by_fixture_candidate_minus_baseline":aggregate})
        size = _total_bytes(output_dir)
        if size > OUTPUT_CAP: raise OSError("aggregate output cap exceeded before completion manifest")
        manifest.update(status="complete",completed_unix=time.time(),elapsed_seconds=round(time.monotonic()-started,3),aggregate_output_bytes=size)
        _write_manifest(output_dir, manifest)
        return 0
    except BaseException as exc:
        manifest.update(status="incomplete",stopped_unix=time.time(),elapsed_seconds=round(time.monotonic()-started,3),error=f"{type(exc).__name__}: {exc}",aggregate_output_bytes=_total_bytes(output_dir))
        _write_manifest(output_dir, manifest)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registration", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--seed-archive", type=Path, default=ROOT / "data/experiments")
    parser.add_argument("--binary", required=True, type=Path, help="root-prebuilt executable matching registered SHA-256")
    args = parser.parse_args()
    try:
        return run(args.registration.resolve(), args.output_dir.resolve(), args.seed_archive.resolve(), args.binary.resolve())
    except Exception as exc:
        print(f"run_food_search_reserve_lab: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
