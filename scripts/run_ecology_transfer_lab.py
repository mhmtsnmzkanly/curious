#!/usr/bin/env python3
"""Registered, bounded supervisor for the ecology transfer V1 screen.

This script cannot choose or tune the protocol. It requires a root-reviewed
registration JSON whose source and fixture hashes match the clean checkout.
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
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
HABITATS = {
    "meadow": ("scenarios/meadow.json", [62301, 62302, 62303]),
    "sparse_resource": ("scenarios/ecology-traits-lab.json", [62311, 62312, 62313]),
    "mixed_refuge": ("scenarios/mixed-refuge.json", [62321, 62322, 62323]),
}
ARMS = ("control", "climate_only", "ambush_only", "alarm_only", "extra_traits_only")
TICKS = 600
PHASE_SECONDS = 300
CHILD_SECONDS = 60
CHILD_ADDRESS_SPACE = 1024 * 1024 * 1024
OUTPUT_CAP = 1024 * 1024 * 1024
MAX_IDENTITIES = 10_000
MAX_EVENTS_PER_TICK = 20_000
SOURCE_PATHS = (
    "Cargo.toml", "Cargo.lock", "examples/ecology_transfer_lab.rs",
    "scripts/run_ecology_transfer_lab.py", "scripts/run_ecology_traits_lab.py",
    "scripts/test_ecology_transfer_lab.py",
    "docs/ECOLOGY_TRANSFER_V1_PROTOCOL_20261026.md",
    "scenarios/meadow.json", "scenarios/ecology-traits-lab.json", "scenarios/mixed-refuge.json",
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


def scenario_hashes() -> dict[str, str]:
    return {name: sha256(ROOT / path) for name, (path, _) in HABITATS.items()}


def expected_seed_roles() -> list[dict]:
    return [{"habitat": habitat, "seed": seed} for habitat, (_, seeds) in HABITATS.items() for seed in seeds]


def validate_registration(registration: dict, identity: dict, fixture_hashes: dict) -> None:
    if registration.get("status") != "registered" or registration.get("root_reviewed") is not True:
        raise ValueError("registration must be status=registered and root_reviewed=true")
    if not identity["clean"]:
        raise ValueError("registered evaluation requires a clean committed checkout")
    if registration.get("source") != identity:
        raise ValueError("registration source identity differs from current checkout")
    if registration.get("scenario_sha256") != fixture_hashes:
        raise ValueError("scenario hashes differ from registration")
    if registration.get("seed_roles") != expected_seed_roles():
        raise ValueError("registered seed roles differ from the frozen 623xx allocation")
    expected = {
        "ticks_per_world": TICKS,
        "arms": list(ARMS),
        "habitats": list(HABITATS),
        "world_count": 45,
        "phase_seconds": PHASE_SECONDS,
        "child_seconds": CHILD_SECONDS,
        "child_address_space_bytes": CHILD_ADDRESS_SPACE,
        "aggregate_output_bytes": OUTPUT_CAP,
        "max_identities": MAX_IDENTITIES,
        "max_events_per_tick": MAX_EVENTS_PER_TICK,
        "mutation_per_mille": 0,
        "genetics_enabled": True,
    }
    for key, value in expected.items():
        if registration.get(key) != value:
            raise ValueError(f"registration {key} differs from frozen protocol")


def scan_seed_roles(archive: Path, roles: list[dict]) -> dict:
    helper = ROOT / "scripts/run_ecology_traits_lab.py"
    spec = importlib.util.spec_from_file_location("ecology_transfer_seed_inventory", helper)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load existing seed inventory scanner")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    seeds = [r["seed"] for r in roles]
    audit = mod.audit_seeds(archive, seeds)
    if audit.get("unseen") != seeds:
        raise ValueError(f"one or more registered seeds already occur in experiment inventory: {audit.get('matches')}")
    return audit


def _child_limits() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (CHILD_ADDRESS_SPACE, CHILD_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_CPU, (CHILD_SECONDS, CHILD_SECONDS))


def _atomic_json(path: Path, value: dict) -> None:
    temp = path.with_name(path.name + ".tmp")
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    os.replace(temp, path)


def _total_bytes(root: Path) -> int:
    return sum(p.stat().st_size for p in root.rglob("*") if p.is_file())


def _read_run(path: Path) -> dict:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if len(rows) != TICKS + 2 or rows[0].get("type") != "header" or rows[-1].get("type") != "summary":
        raise ValueError(f"malformed/incomplete run JSONL: {path.name}")
    if [r.get("tick") for r in rows[1:-1]] != list(range(1, TICKS + 1)):
        raise ValueError(f"tick sequence incomplete in {path.name}")
    summary = rows[-1]
    if summary.get("ticks_completed") != TICKS:
        raise ValueError(f"run did not complete registered horizon: {path.name}")
    if summary.get("checkpoint_restore_valid") is not True:
        raise ValueError(f"endpoint checkpoint restore did not validate: {path.name}")
    if summary.get("lineage_issues") != 0 or summary.get("unknown_generation_births") != 0:
        raise ValueError(f"birth/parent/generation attribution is incomplete: {path.name}")
    tick_rows = rows[1:-1]
    required_lineage = ("birth_lineage", "mating_events", "death_lineage")
    if any(any(not isinstance(row.get(key), list) for key in required_lineage) for row in tick_rows):
        raise ValueError(f"compact per-tick lineage evidence is missing: {path.name}")
    birth_records = [record for row in tick_rows for record in row["birth_lineage"]]
    mating_records = [record for row in tick_rows for record in row["mating_events"]]
    death_records = [record for row in tick_rows for record in row["death_lineage"]]
    def nonzero_counts(value: dict) -> dict:
        return {str(key): count for key, count in value.items() if count != 0}

    if len(birth_records) != len(mating_records):
        raise ValueError(f"birth/mating record counts differ: {path.name}")
    births_by_species: dict[str, int] = {}
    births_by_generation: dict[str, int] = {}
    births_by_id = {}
    for record in birth_records:
        entity_id = record.get("id")
        generation = record.get("generation")
        parents = record.get("parent_ids")
        if not isinstance(entity_id, int) or not isinstance(generation, int) or not isinstance(parents, list) or len(parents) != 2:
            raise ValueError(f"invalid compact birth lineage: {path.name}")
        if entity_id in births_by_id:
            raise ValueError(f"duplicate birth ID in lineage evidence: {path.name}")
        births_by_id[entity_id] = (generation, parents)
        species = record.get("species")
        births_by_species[species] = births_by_species.get(species, 0) + 1
        key = str(generation)
        births_by_generation[key] = births_by_generation.get(key, 0) + 1
    if births_by_species != nonzero_counts(summary.get("births_by_species", {})):
        raise ValueError(f"birth species totals differ from compact lineage: {path.name}")
    if births_by_generation != nonzero_counts(summary.get("births_by_generation", {})):
        raise ValueError(f"birth generation totals differ from compact lineage: {path.name}")
    f1_breeders = set()
    mated_children = set()
    for record in mating_records:
        child = record.get("child")
        if child not in births_by_id:
            raise ValueError(f"mating event has no compact birth lineage: {path.name}")
        if child in mated_children:
            raise ValueError(f"duplicate mating record for child: {path.name}")
        mated_children.add(child)
        _, parents = births_by_id[child]
        mating_parents = [record.get("parent1"), record.get("parent2")]
        if set(parents) != set(mating_parents) or record.get("child_parent_ids") is None or set(record["child_parent_ids"]) != set(parents):
            raise ValueError(f"mating event and child parent IDs disagree: {path.name}")
        if record.get("parent1_generation") == 1:
            f1_breeders.add(record["parent1"])
        if record.get("parent2_generation") == 1:
            f1_breeders.add(record["parent2"])
    if mated_children != set(births_by_id):
        raise ValueError(f"mating records do not cover every birth: {path.name}")
    if sorted(f1_breeders) != summary.get("f1_breeders", []):
        raise ValueError(f"F1 breeder IDs differ from compact mating events: {path.name}")
    deaths_by_species: dict[str, int] = {}
    deaths_by_generation: dict[str, int] = {}
    for record in death_records:
        if not isinstance(record.get("id"), int) or not isinstance(record.get("generation"), int):
            raise ValueError(f"invalid compact death lineage: {path.name}")
        species = record.get("species")
        deaths_by_species[species] = deaths_by_species.get(species, 0) + 1
        key = str(record["generation"])
        deaths_by_generation[key] = deaths_by_generation.get(key, 0) + 1
    if deaths_by_species != nonzero_counts(summary.get("deaths_by_species", {})):
        raise ValueError(f"death species totals differ from compact lineage: {path.name}")
    if deaths_by_generation != nonzero_counts(summary.get("deaths_by_generation", {})):
        raise ValueError(f"death generation totals differ from compact lineage: {path.name}")
    if (summary.get("f1_births", 0) != births_by_generation.get("1", 0)
            or summary.get("f2_births", 0) != births_by_generation.get("2", 0)):
        raise ValueError(f"F1/F2 totals differ from compact lineage: {path.name}")
    initial_count = len(rows[0].get("initial_state", {}).get("entities", []))
    births = sum(summary.get("births_by_species", {}).values())
    deaths = sum(summary.get("deaths_by_species", {}).values())
    final_count = sum(summary.get("final_alive", {}).values())
    if initial_count + births - deaths != final_count:
        raise ValueError(f"population balance failed: {path.name}")
    return summary


def _write_manifest(out: Path, manifest: dict) -> None:
    _atomic_json(out / "manifest.json", manifest)


def run(registration_path: Path, output_dir: Path, seed_archive: Path, target_dir: Path) -> int:
    started = time.monotonic()
    identity = source_identity()
    fixture_hashes = scenario_hashes()
    registration = json.loads(registration_path.read_text())
    validate_registration(registration, identity, fixture_hashes)
    inventory = scan_seed_roles(seed_archive, registration["seed_roles"])
    if output_dir.exists():
        raise FileExistsError(f"output directory already exists: {output_dir}")
    output_dir.mkdir(parents=True, exist_ok=False)
    shutil.copy2(registration_path, output_dir / "registration.json")
    manifest = {"schema":"curious-ecology-transfer-v1-manifest", "status":"running", "started_unix":time.time(), "source":identity, "scenario_sha256":fixture_hashes, "seed_inventory":inventory, "ticks_per_world":TICKS, "arms":list(ARMS), "worlds":[], "aggregate_output_bytes":0}
    _write_manifest(output_dir, manifest)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target_dir.resolve())
    remaining = PHASE_SECONDS - (time.monotonic() - started)
    if remaining <= 0:
        raise TimeoutError("phase budget elapsed before build")
    try:
        build = subprocess.run(["cargo", "build", "--locked", "--release", "--example", "ecology_transfer_lab", "--jobs", "1"], cwd=ROOT, env=env, capture_output=True, timeout=remaining)
    except BaseException as exc:
        manifest.update(status="incomplete", stopped_unix=time.time(), elapsed_seconds=round(time.monotonic()-started,3), error=f"{type(exc).__name__}: {exc}")
        _write_manifest(output_dir, manifest)
        raise
    (output_dir / "build.stdout.log").write_bytes(build.stdout)
    (output_dir / "build.stderr.log").write_bytes(build.stderr)
    if build.returncode:
        manifest.update(status="build_failed", error=f"cargo build exited {build.returncode}")
        _write_manifest(output_dir, manifest)
        return build.returncode
    binary = Path(env["CARGO_TARGET_DIR"]) / "release/examples/ecology_transfer_lab"
    if not binary.is_file():
        raise FileNotFoundError("cargo build returned success without expected runner binary")
    manifest["binary_sha256"] = sha256(binary)
    _write_manifest(output_dir, manifest)
    try:
        for role in registration["seed_roles"]:
            habitat, seed = role["habitat"], role["seed"]
            scenario_rel = HABITATS[habitat][0]
            for arm in ARMS:
                if time.monotonic() - started >= PHASE_SECONDS:
                    raise TimeoutError("300-second phase budget exceeded")
                stem = f"{habitat}-seed{seed}-{arm}"
                raw_path = output_dir / f"{stem}.jsonl"
                stderr_path = output_dir / f"{stem}.stderr"
                command = [str(binary), str(ROOT / scenario_rel), str(seed), arm, habitat, str(raw_path)]
                before = time.monotonic()
                with stderr_path.open("xb") as stderr:
                    proc = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.DEVNULL, stderr=stderr, preexec_fn=_child_limits, start_new_session=True)
                    while proc.poll() is None:
                        if time.monotonic() - before >= CHILD_SECONDS:
                            proc.kill(); proc.wait(); raise TimeoutError(f"60-second child limit exceeded: {stem}")
                        if time.monotonic() - started >= PHASE_SECONDS:
                            proc.kill(); proc.wait(); raise TimeoutError("300-second phase budget exceeded")
                        if _total_bytes(output_dir) > OUTPUT_CAP:
                            proc.kill(); proc.wait(); raise OSError("aggregate output cap exceeded")
                        time.sleep(0.05)
                if _total_bytes(output_dir) > OUTPUT_CAP:
                    raise OSError("aggregate output cap exceeded after child exit")
                if proc.returncode != 0:
                    raise RuntimeError(f"child failed ({proc.returncode}): {stem}")
                summary = _read_run(raw_path)
                manifest["worlds"].append({"habitat":habitat,"seed":seed,"arm":arm,"raw":raw_path.name,"raw_sha256":sha256(raw_path),"raw_bytes":raw_path.stat().st_size,"stderr":stderr_path.name,"summary":{k:summary.get(k) for k in ("ticks_completed","living_entity_ticks","final_alive","founders_alive","births_by_species","deaths_by_species","births_by_generation","deaths_by_generation","f1_births","f2_births","f1_breeders","living_f1_breeders","living_offspring_breeders","lineage_issues","unknown_generation_births","attacks","alarm","climate","extra_traits","samples_every_100_ticks","checkpoint_restore_valid")}})
                manifest["aggregate_output_bytes"] = _total_bytes(output_dir)
                _write_manifest(output_dir, manifest)
        final_output_bytes = _total_bytes(output_dir)
        if final_output_bytes > OUTPUT_CAP:
            raise OSError("aggregate output cap exceeded before completion manifest")
        manifest.update(status="complete", completed_unix=time.time(), elapsed_seconds=round(time.monotonic()-started,3), aggregate_output_bytes=final_output_bytes)
        _write_manifest(output_dir, manifest)
        return 0
    except BaseException as exc:
        manifest.update(status="incomplete", stopped_unix=time.time(), elapsed_seconds=round(time.monotonic()-started,3), error=f"{type(exc).__name__}: {exc}", aggregate_output_bytes=_total_bytes(output_dir))
        _write_manifest(output_dir, manifest)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registration", required=True, type=Path, help="root-reviewed registered protocol JSON")
    parser.add_argument("--output-dir", required=True, type=Path, help="new output directory; must not exist")
    parser.add_argument("--seed-archive", type=Path, default=ROOT / "data/experiments")
    parser.add_argument("--target-dir", type=Path, default=ROOT / "target/ecology-transfer-v1")
    args = parser.parse_args()
    try:
        return run(args.registration.resolve(), args.output_dir.resolve(), args.seed_archive.resolve(), args.target_dir.resolve())
    except Exception as exc:
        print(f"run_ecology_transfer_lab: {type(exc).__name__}: {exc}", file=sys.stderr)
        return 2

if __name__ == "__main__":
    raise SystemExit(main())
