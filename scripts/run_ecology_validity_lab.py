#!/usr/bin/env python3
"""Source-bound, resource-capped runner for the ecology fixture validity lab.

No seed roles are embedded here. A root-reviewed registered JSON protocol must
provide them before this runner will launch a World.
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

SCENARIO_SHA256 = "33cb6ebf9d73df017b65b9ee9831be8446c2a81077f3a0e313fe06b05f0b19ac"
ACCEPTED_BASELINE = {
    "revision": "b4dcef20fdef830c2895806076d972df62741eea",
    "binary_sha256": "e1ffa03754fc964d503b6a6e04bffe19ad868b3d2090e0a8b2f9c48e3506f636",
    "scenario_sha256": SCENARIO_SHA256,
    "reserve_opportunities": True,
    "mate_rendezvous": True,
}
MAX_PHASE_SECONDS = 300
MAX_TOTAL_ADDRESS_SPACE = 1024 * 1024 * 1024
PARENT_ADDRESS_SPACE = 256 * 1024 * 1024
CHILD_ADDRESS_SPACE = MAX_TOTAL_ADDRESS_SPACE - PARENT_ADDRESS_SPACE
CANONICAL_SEED_ARCHIVE = Path("/root/curious/data/experiments")
MAX_SEED_RECORD_BYTES = 64 * 1024 * 1024
SCANNER_PATH = Path("scripts/run_ecology_traits_lab.py")
PROJECT_ROOT = Path(__file__).resolve().parents[1]
SOURCE_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "examples/ecology_validity_lab.rs",
    "examples/support/ecology_audit.rs",
    "scripts/run_ecology_traits_lab.py",
    "scripts/run_ecology_validity_lab.py",
    "scripts/test_ecology_validity_lab.py",
)
MAX_OUTPUT_BYTES = 128 * 1024 * 1024
METADATA_RESERVE_BYTES = 1024 * 1024
PHASE_TICKS = {"calibration": 200, "formal": 500}
RUN_OUTPUT_CREATED = False
ACCEPTED_PROFILE = {
    "mode": "both",
    "companions": True,
    "genetics_enabled": True,
    "mutation_per_mille": 20,
    "extra_traits_enabled": False,
    "water_learner_enabled": True,
    "water_updates_enabled": True,
    "water_residual_enabled": False,
    "hunt_learning_enabled": False,
    "refuge_outcomes_enabled": False,
}
RESOURCE_LIMITS = {
    "phase_seconds": MAX_PHASE_SECONDS,
    "preflight_address_space_bytes": MAX_TOTAL_ADDRESS_SPACE,
    "parent_address_space_bytes": PARENT_ADDRESS_SPACE,
    "child_address_space_bytes": CHILD_ADDRESS_SPACE,
    "aggregate_output_bytes": MAX_OUTPUT_BYTES,
    "identity_records": 50_000,
    "lineage_event_records": 50_000,
    "detail_issue_rows": 2_000,
    "max_events_per_tick": 20_000,
}
ACCEPTANCE = {
    "calibration": [
        "no_species_extinction",
        "zero_food_and_water_residual_ticks",
        "zero_resource_kind_replacements",
        "no_parent_unattributed_or_trait_payment_issues",
        "at_least_one_actual_birth_of_either_species",
    ],
    "formal": [
        "technical_resource_and_lineage_gates_pass",
        "no_species_extinction",
        "at_least_two_alive_of_each_species_on_every_tick_in_final_50",
        "at_least_one_living_descendant_breeder_of_each_species_at_tick_500",
        "at_least_one_birth_of_each_species_in_final_50",
    ],
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def source_identity(root: Path) -> dict[str, str | bool]:
    tracked_src = git(root, "ls-files", "-z", "src").split("\0")
    files = sorted(set(SOURCE_FILES).union(path for path in tracked_src if path))
    hashes = {path: sha256(root / path) for path in files}
    content_digest = hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return {
        "revision": git(root, "rev-parse", "HEAD"),
        "clean": not bool(git(root, "status", "--porcelain", "--untracked-files=all")),
        "content_sha256": content_digest,
        "file_sha256": hashes,
    }


def scan_seeds(archive: Path, seeds: list[int]) -> dict:
    scanner_path = Path(__file__).resolve().parents[1] / SCANNER_PATH
    spec = importlib.util.spec_from_file_location("ecology_fixture_seed_scanner", scanner_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load complete seed inventory scanner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.audit_seeds(archive, seeds, max_line_bytes=MAX_SEED_RECORD_BYTES)


def _limit_scanner_child() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (CHILD_ADDRESS_SPACE, CHILD_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_CPU, (MAX_PHASE_SECONDS, MAX_PHASE_SECONDS))


def scan_seeds_supervised(archive: Path, seeds: list[int], remaining_seconds: float) -> dict:
    if remaining_seconds <= 0:
        raise TimeoutError("phase budget elapsed before seed archive scan")
    env = os.environ.copy()
    command = [sys.executable, str(Path(__file__).resolve()), "--scan-only", "--seed-archive", str(archive), "--seeds", ",".join(map(str, seeds))]
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        env=env,
        timeout=remaining_seconds,
        check=True,
        preexec_fn=_limit_scanner_child,
    )
    audit = json.loads(result.stdout)
    if audit.get("unseen") != seeds:
        raise ValueError(f"registered phase seeds are already present in the experiment archive: {audit.get('matches')}")
    return audit


def validate_protocol(protocol: dict, scenario: Path, identity: dict) -> tuple[int, list[dict]]:
    if protocol.get("status") != "registered" or protocol.get("root_reviewed") is not True:
        raise ValueError("protocol must be registered and root_reviewed before any World run")
    if protocol.get("source") != identity:
        raise ValueError("protocol source identity does not match current clean checkout")
    if not identity["clean"]:
        raise ValueError("registered World runs require a clean committed checkout")
    if sha256(scenario) != SCENARIO_SHA256 or protocol.get("scenario_sha256") != SCENARIO_SHA256:
        raise ValueError("scenario does not match the accepted qualification fixture hash")
    if protocol.get("accepted_baseline") != ACCEPTED_BASELINE:
        raise ValueError("accepted qualification manifest identity mismatch")
    fixture = json.loads(scenario.read_text())
    founder_counts = {name: sum(founder.get("species") == name for founder in fixture.get("founders", [])) for name in ("Herbivore", "Carnivore")}
    if fixture.get("name") != "mixed-refuge 4x4 food=1 water=1 age=steady" or founder_counts != {"Herbivore": 576, "Carnivore": 32} or len(fixture.get("sites", [])) != 976 or len(fixture.get("resources", [])) != 1232:
        raise ValueError("scenario structure does not match the accepted fixture inventory")
    if protocol.get("policies") != {
        "mate_rendezvous": True,
        "reserve_opportunities": True,
        "water_residual_enabled": False,
    }:
        raise ValueError("explicit accepted-profile policy flags are required")
    if protocol.get("profile") != ACCEPTED_PROFILE:
        raise ValueError("accepted mode/genetics/learning profile mismatch")
    inventory_hash = protocol.get("seed_inventory_sha256")
    if not isinstance(inventory_hash, str) or len(inventory_hash) != 64 or any(char not in "0123456789abcdef" for char in inventory_hash):
        raise ValueError("protocol must bind the read-only seed inventory SHA-256")
    phase = protocol.get("phase")
    ticks = PHASE_TICKS.get(phase)
    roles = protocol.get("seed_roles")
    role_count = 1 if phase == "calibration" else 3 if phase == "formal" else 0
    if protocol.get("phase_world_count") != role_count or protocol.get("resource_limits") != RESOURCE_LIMITS:
        raise ValueError("registered phase count or runtime/capture limits differ from the reviewed contract")
    if protocol.get("acceptance") != ACCEPTANCE.get(phase):
        raise ValueError("registered biological/technical gates differ from the reviewed contract")
    if phase == "formal":
        budget = protocol.get("budget_forecast", {})
        if budget != {
            "calibration_ticks": 200,
            "formal_ticks_total": 1500,
            "safety_factor": 2,
            "seed_scan_analysis_reserve_seconds": 60,
            "maximum_seconds": 300,
            "formula": "calibration_probe_seconds * (1500 / calibration_ticks) * 2 + 60",
        } or protocol.get("phase_total_ticks") != 1500:
            raise ValueError("formal tick total or calibration-based forecast contract mismatch")
    if protocol.get("ticks_per_world") != ticks:
        raise ValueError(f"{phase} horizon must be exactly {ticks} ticks per world")
    if not isinstance(roles, list) or not roles:
        raise ValueError("registered seed_roles must be a nonempty list")
    names, seeds = set(), set()
    parsed = []
    for role in roles:
        if not isinstance(role, dict) or not isinstance(role.get("role"), str) or not isinstance(role.get("seed"), int):
            raise ValueError("each seed role needs a role name and integer seed")
        if role["role"] in names or role["seed"] in seeds:
            raise ValueError("role names and seed numbers must be unique")
        names.add(role["role"])
        seeds.add(role["seed"])
        parsed.append(role)
    evidence_rel = protocol.get("seed_inventory_evidence")
    if not isinstance(evidence_rel, str) or Path(evidence_rel).is_absolute() or ".." in Path(evidence_rel).parts:
        raise ValueError("seed inventory evidence must be a repository-relative file")
    inventory_path = PROJECT_ROOT / evidence_rel
    if not inventory_path.is_file() or sha256(inventory_path) != inventory_hash:
        raise ValueError("seed inventory evidence file is missing or its hash changed")
    inventory = json.loads(inventory_path.read_text())
    if inventory.get("scanned_text_files") != protocol.get("seed_inventory_text_sources_scanned"):
        raise ValueError("seed inventory scanned-file count mismatch")
    unseen = set(inventory.get("unseen", []))
    if any(role["seed"] not in unseen or inventory.get("matches", {}).get(str(role["seed"])) for role in parsed):
        raise ValueError("a registered role is not unseen in the preregistered seed inventory")
    expected_roles = phase == "calibration" and len(parsed) == 1 or phase == "formal" and len(parsed) == 3
    if not expected_roles:
        raise ValueError("phase must be calibration with one role or formal with three roles")
    return ticks, parsed


def evaluate_record(raw: dict, phase: str) -> dict:
    audit = raw["audit"]
    timeline = raw["per_tick"]
    species = ("Herbivore", "Carnivore")
    extinct = any(any(row.get("alive", {}).get(name, 0) == 0 for name in species) for row in timeline)
    parent_issues = audit["parent_event_issues"]
    residual = audit["resource_reconciliation"]
    lineage_consistent = not parent_issues and not audit["unattributed_actor_events"] and not audit["trait_payment_issues"]
    residuals_zero = residual["food_nonzero_residual_ticks"] == 0 and residual["water_nonzero_residual_ticks"] == 0
    replacement_free = audit["resource_kind_replacement_summary"]["count"] == 0
    births_total = sum(int(value) for value in audit["births_by_generation"].values())
    technical_pass = residuals_zero and replacement_free and lineage_consistent
    actual_matings = sum("EntityMated" in event.get("event", {}) for event in raw.get("lineage_events", []))
    proxy_pairs = sum(raw.get("eligible_adjacent_mating_pairs_observed", {}).values())
    if phase == "calibration":
        checks = {
            "no_extinction": not extinct,
            "resource_flow_consistent": residuals_zero and replacement_free,
            "lineage_consistent": lineage_consistent,
            "at_least_one_birth": births_total >= 1,
        }
    else:
        tail = timeline[-50:]
        alive_tail = len(tail) == 50 and all(all(row.get("alive", {}).get(name, 0) >= 2 for name in species) for row in tail)
        births_last_50 = {name: sum(row.get("births_by_species_this_tick", {}).get(name, 0) for row in tail) for name in species}
        living_bred = raw["living_bred_descendants_by_species"]
        checks = {
            "no_extinction": not extinct,
            "resource_flow_consistent": residuals_zero and replacement_free,
            "lineage_consistent": lineage_consistent,
            "at_least_two_alive_each_species_throughout_last_50": alive_tail,
            "living_descendant_breeder_each_species_at_end": all(living_bred.get(name, 0) >= 1 for name in species),
            "at_least_one_birth_each_species_last_50": all(births_last_50[name] >= 1 for name in species),
        }
    interpretation = "biological_negative_extinction" if extinct else "observed_actual_mating" if actual_matings else "nonidentifiable_no_mating_exposure_proxy" if proxy_pairs == 0 else "exposure_proxy_without_actual_mating"
    return {"checks": checks, "passed": all(checks.values()), "technical_pass": technical_pass, "extinct": extinct, "actual_mating_events": actual_matings, "eligible_adjacent_pair_exposure_proxy": proxy_pairs, "interpretation": interpretation, "births_last_50": births_last_50 if phase == "formal" else None}


def read_jsonl(path: Path) -> dict:
    header = None
    tick_rows = []
    summary = None
    with path.open() as stream:
        for line in stream:
            row = json.loads(line)
            if "header" in row:
                header = row["header"]
            elif "summary" in row:
                summary = row["summary"]
            elif "tick" in row:
                tick_rows.append(row)
    if header is None or summary is None:
        raise ValueError("raw JSONL lacks a complete header or final summary")
    summary["per_tick"] = [{key: value for key, value in row.items() if key != "audit_frame"} for row in tick_rows]
    summary["lineage_events"] = [event for row in tick_rows for event in row.get("lineage_events", [])]
    summary["audit_frames"] = [row["audit_frame"] for row in tick_rows if "audit_frame" in row]
    if header.get("schema") != "curious-ecology-validity-jsonl-v1":
        raise ValueError("raw JSONL schema mismatch")
    for key in ("seed", "requested_ticks", "scenario_name", "founders_at_tick_0", "founder_species", "explicit_policy", "profile"):
        if summary.get(key) != header.get(key):
            raise ValueError(f"raw header/summary mismatch for {key}")
    ticks_completed = summary.get("ticks_completed")
    if type(ticks_completed) is not int or ticks_completed != len(tick_rows) or [row.get("tick") for row in tick_rows] != list(range(1, ticks_completed + 1)):
        raise ValueError("raw tick rows are missing, duplicated, or out of order")
    if ticks_completed <= 0 or ticks_completed > summary.get("requested_ticks", 0):
        raise ValueError("raw completed horizon is invalid")
    if header.get("initial_state", {}).get("tick") != 0 or summary.get("initial_state") != header.get("initial_state"):
        raise ValueError("raw initial tick-zero State is missing or changed")
    extinct = any(any(row.get("alive", {}).get(name, 0) == 0 for name in ("Herbivore", "Carnivore")) for row in summary["per_tick"])
    if ticks_completed < summary["requested_ticks"] and not extinct:
        raise ValueError("raw stopped before the fixed horizon without recorded extinction")
    if summary.get("final_state", {}).get("tick") != ticks_completed or not summary["audit_frames"] or summary["audit_frames"][0].get("tick") != 1:
        raise ValueError("raw initial/final state evidence is incomplete")
    expected_frames = [tick for tick in (1, 20, 100, 200, 500) if tick <= ticks_completed]
    if [frame.get("tick") for frame in summary["audit_frames"]] != expected_frames:
        raise ValueError("raw fixed audit-frame set is incomplete")
    tick_by_number = {row["tick"]: row for row in tick_rows}
    for frame in summary["audit_frames"]:
        tick_number = frame["tick"]
        if frame.get("pre_state", {}).get("tick") != tick_number - 1 or frame.get("post_state", {}).get("tick") != tick_number:
            raise ValueError("fixed audit frame has mismatched pre/post State ticks")
        if len(frame.get("events", [])) != sum(tick_by_number[tick_number].get("event_counts_by_type", {}).values()):
            raise ValueError("fixed audit frame typed-event count does not match its tick")
    if summary["audit_frames"][0].get("pre_state") != header.get("initial_state"):
        raise ValueError("tick-1 pre-state does not match the captured tick-zero State")
    observed_events = sum(sum(row.get("event_counts_by_type", {}).values()) for row in summary["per_tick"])
    if observed_events != summary.get("capture_limits", {}).get("event_observations_cumulative"):
        raise ValueError("raw per-tick event counts do not match the cumulative total")
    lineage_count = sum(len(row.get("lineage_events", [])) for row in tick_rows)
    if lineage_count != summary.get("capture_limits", {}).get("lineage_events_retained"):
        raise ValueError("raw lineage event count does not match the captured total")
    return summary


def verify_calibration_evidence(path: Path, source: dict, scenario: Path, formal_protocol: dict) -> tuple[dict, dict, Path]:
    calibration = json.loads(path.read_text())
    if calibration.get("schema") != "curious-ecology-validity-run-v1" or calibration.get("phase") != "calibration":
        raise ValueError("formal preflight requires a calibration run manifest")
    if calibration.get("source") != source or calibration.get("scenario_sha256") != sha256(scenario):
        raise ValueError("calibration source or scenario identity mismatch")
    if calibration.get("seed_inventory_sha256") != formal_protocol.get("seed_inventory_sha256"):
        raise ValueError("calibration and formal seed inventory snapshots differ")
    if calibration.get("accepted_baseline") != formal_protocol.get("accepted_baseline") or calibration.get("policies") != formal_protocol.get("policies"):
        raise ValueError("calibration and formal accepted-profile identities differ")
    protocol_snapshot = path.parent / calibration.get("protocol_snapshot", "")
    if not protocol_snapshot.is_file() or sha256(protocol_snapshot) != calibration.get("protocol_sha256"):
        raise ValueError("calibration protocol snapshot is missing or changed")
    calibration_protocol = json.loads(protocol_snapshot.read_text())
    validate_protocol(calibration_protocol, scenario, source)
    if calibration_protocol.get("phase") != "calibration" or calibration_protocol.get("seed_roles") != [
        {"role": calibration.get("seed_roles", [{}])[0].get("role"), "seed": calibration.get("seed_roles", [{}])[0].get("seed")}
    ]:
        raise ValueError("calibration protocol seed role does not match its run manifest")
    probe_snapshot = path.parent / calibration.get("probe_snapshot", "")
    if not probe_snapshot.is_file() or sha256(probe_snapshot) != calibration.get("binary_sha256"):
        raise ValueError("calibration executable snapshot is missing or changed")
    records = calibration.get("seed_roles", [])
    if len(records) != 1:
        raise ValueError("calibration must contain exactly one seed result")
    record = records[0]
    raw_path = path.parent / record.get("path", "")
    if not raw_path.is_file() or sha256(raw_path) != record.get("sha256"):
        raise ValueError("calibration raw evidence is missing or changed")
    raw = read_jsonl(raw_path)
    expected_seed = calibration_protocol["seed_roles"][0]["seed"]
    if raw.get("seed") != expected_seed or raw.get("requested_ticks") != PHASE_TICKS["calibration"] or raw.get("ticks_completed") != PHASE_TICKS["calibration"]:
        raise ValueError("calibration raw evidence did not complete the registered horizon")
    if raw.get("profile") != ACCEPTED_PROFILE or raw.get("explicit_policy") != calibration_protocol["policies"]:
        raise ValueError("calibration raw accepted-profile settings mismatch")
    recomputed = evaluate_record(raw, "calibration")
    if recomputed != record.get("outcome") or not recomputed["passed"] or calibration.get("passed") is not True:
        raise ValueError("calibration gates fail when recomputed from immutable raw evidence")
    if record.get("ticks_completed") != PHASE_TICKS["calibration"]:
        raise ValueError("calibration manifest tick count mismatch")
    return calibration, calibration_protocol, probe_snapshot


def _limit_child() -> None:
    resource.setrlimit(resource.RLIMIT_AS, (CHILD_ADDRESS_SPACE, CHILD_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_CPU, (MAX_PHASE_SECONDS, MAX_PHASE_SECONDS))


def run_phase(root: Path, scenario: Path, output_dir: Path, protocol_path: Path, binary: Path | None = None, calibration_manifest_path: Path | None = None, seed_archive: Path = CANONICAL_SEED_ARCHIVE) -> dict:
    global RUN_OUTPUT_CREATED
    RUN_OUTPUT_CREATED = False
    started = time.time()
    protocol = json.loads(protocol_path.read_text())
    identity = source_identity(root)
    ticks, roles = validate_protocol(protocol, scenario, identity)
    if binary is None:
        env = os.environ.copy()
        env["CARGO_BUILD_JOBS"] = "1"
        target_dir = root / "target-ecology-validity"
        env["CARGO_TARGET_DIR"] = str(target_dir)
        subprocess.run(["cargo", "build", "--release", "--example", "ecology_validity_lab"], cwd=root, env=env, check=True, timeout=MAX_PHASE_SECONDS)
        binary = target_dir / "release/examples/ecology_validity_lab"
    binary_sha = sha256(binary)
    # Preflight is single-process; allow up to the aggregate ceiling for the
    # complete 64-MiB-line scanner and calibration JSONL revalidation.
    resource.setrlimit(resource.RLIMIT_AS, (MAX_TOTAL_ADDRESS_SPACE, MAX_TOTAL_ADDRESS_SPACE))
    forecast_seconds = None
    calibration_evidence = None
    if protocol["phase"] == "formal":
        if calibration_manifest_path is None:
            raise ValueError("formal phase requires the passed calibration manifest for the frozen budget forecast")
        calibration, _calibration_protocol, _calibration_probe = verify_calibration_evidence(
            calibration_manifest_path, identity, scenario, protocol
        )
        calibration_evidence = calibration
        calibration_record = calibration["seed_roles"][0]
        if any(role.get("seed") == calibration_record.get("seed") for role in roles):
            raise ValueError("formal seed roles must be disjoint from calibration")
        calibration_ticks = calibration_record.get("ticks_completed")
        calibration_elapsed = calibration_record.get("probe_elapsed_seconds")
        if calibration_ticks != PHASE_TICKS["calibration"] or not isinstance(calibration_elapsed, (int, float)):
            raise ValueError("calibration did not complete the registered horizon with a measured probe time")
        forecast_seconds = estimate_formal_seconds(calibration_elapsed, calibration_ticks)
        if forecast_seconds > MAX_PHASE_SECONDS:
            raise ValueError(f"budget feasibility gate failed before formal seeds: forecast={forecast_seconds:.2f}s > {MAX_PHASE_SECONDS}s")
    role_seeds = [role["seed"] for role in roles]
    resource.setrlimit(resource.RLIMIT_AS, (PARENT_ADDRESS_SPACE, MAX_TOTAL_ADDRESS_SPACE))
    seed_audit = scan_seeds_supervised(seed_archive, role_seeds, MAX_PHASE_SECONDS - (time.time() - started))
    resource.setrlimit(resource.RLIMIT_AS, (MAX_TOTAL_ADDRESS_SPACE, MAX_TOTAL_ADDRESS_SPACE))
    if time.time() - started > MAX_PHASE_SECONDS:
        raise TimeoutError("preflight/seed inventory exceeded the phase wall-clock cap")
    if calibration_evidence is not None and binary_sha != calibration_evidence.get("binary_sha256"):
        raise ValueError("current probe binary differs from the calibrated immutable executable")
    output_dir.mkdir(parents=True, exist_ok=False)
    RUN_OUTPUT_CREATED = True
    (output_dir / "status.json").write_text(json.dumps({"status": "running", "phase": protocol["phase"], "started_epoch": started}, indent=2) + "\n")
    protocol_snapshot = output_dir / "protocol.snapshot.json"
    with protocol_snapshot.open("xb") as target:
        with protocol_path.open("rb") as source_stream:
            shutil.copyfileobj(source_stream, target)
    protocol_snapshot.chmod(0o444)
    probe_snapshot = output_dir / "probe.snapshot"
    with probe_snapshot.open("xb") as target:
        with binary.open("rb") as source_stream:
            shutil.copyfileobj(source_stream, target)
    probe_snapshot.chmod(0o555)
    binary = probe_snapshot
    binary_sha = sha256(binary)
    snapshot_bytes = sum(candidate.stat().st_size for candidate in output_dir.iterdir() if candidate.is_file())
    if snapshot_bytes + METADATA_RESERVE_BYTES > MAX_OUTPUT_BYTES:
        raise RuntimeError("protocol/probe snapshots exceed the phase output budget before World start")
    raw_records = []
    outcomes = []
    for role in roles:
        stem = f"{role['role']}-{role['seed']}"
        raw_path = output_dir / f"{stem}.jsonl"
        stderr_path = output_dir / f"{stem}.stderr.txt"
        output_used = sum(candidate.stat().st_size for candidate in output_dir.iterdir() if candidate.is_file())
        remaining_output = MAX_OUTPUT_BYTES - output_used - METADATA_RESERVE_BYTES
        if remaining_output <= 0:
            raise RuntimeError("aggregate phase output cap reached before next seed role")
        command = [str(binary), str(scenario), str(role["seed"]), str(ticks), "true", "true", str(raw_path), str(remaining_output)]
        if time.time() - started >= MAX_PHASE_SECONDS:
            raise TimeoutError("phase wall-clock cap reached before next World start")
        # The parent is capped to 256 MiB while the single child is capped to
        # 768 MiB. No other probe child runs concurrently.
        resource.setrlimit(resource.RLIMIT_AS, (PARENT_ADDRESS_SPACE, MAX_TOTAL_ADDRESS_SPACE))
        with stderr_path.open("xb") as stderr:
            child_started = time.time()
            process = subprocess.Popen(command, cwd=root, stdout=subprocess.DEVNULL, stderr=stderr, preexec_fn=_limit_child)
            while process.poll() is None:
                if time.time() - started > MAX_PHASE_SECONDS:
                    process.kill()
                    process.wait()
                    raise TimeoutError("phase wall-clock limit exceeded")
                output_bytes = sum(candidate.stat().st_size for candidate in output_dir.iterdir() if candidate.is_file())
                if output_bytes > MAX_OUTPUT_BYTES:
                    process.kill()
                    process.wait()
                    raise RuntimeError("aggregate phase output cap exceeded")
                time.sleep(0.1)
            child_elapsed = time.time() - child_started
        # No child remains while full raw JSONL validation runs.
        resource.setrlimit(resource.RLIMIT_AS, (MAX_TOTAL_ADDRESS_SPACE, MAX_TOTAL_ADDRESS_SPACE))
        if process.returncode != 0:
            (output_dir / "status.json").write_text(json.dumps({"status": "failed", "phase": protocol["phase"], "role": role, "returncode": process.returncode, "partial_raw": raw_path.name if raw_path.exists() else None, "stderr": stderr_path.name, "elapsed_seconds": round(time.time() - started, 3)}, indent=2) + "\n")
            raise RuntimeError(f"bounded lab exited {process.returncode}; inspect {stderr_path}")
        if sum(candidate.stat().st_size for candidate in output_dir.iterdir() if candidate.is_file()) > MAX_OUTPUT_BYTES:
            raise RuntimeError("aggregate phase output cap exceeded")
        raw = read_jsonl(raw_path)
        if raw.get("schema") != "curious-ecology-validity-v1" or raw.get("seed") != role["seed"] or raw.get("requested_ticks") != ticks or raw.get("profile") != ACCEPTED_PROFILE:
            raise RuntimeError("probe output identity mismatch")
        if raw.get("founders_at_tick_0") != 608 or raw.get("explicit_policy") != protocol["policies"]:
            raise RuntimeError("probe accepted-profile identity mismatch")
        if sorted(raw.get("founder_species", {}).values()).count("Herbivore") != 576 or sorted(raw.get("founder_species", {}).values()).count("Carnivore") != 32:
            raise RuntimeError("probe founder species mismatch")
        outcome = evaluate_record(raw, protocol["phase"])
        outcomes.append(outcome)
        raw_records.append({"role": role["role"], "seed": role["seed"], "path": raw_path.name, "sha256": sha256(raw_path), "ticks_completed": raw["ticks_completed"], "probe_elapsed_seconds": round(child_elapsed, 4), "command": command, "outcome": outcome})
        if protocol["phase"] == "calibration" and not outcome["passed"]:
            break
    manifest = {
        "schema": "curious-ecology-validity-run-v1",
        "status": "complete",
        "phase": protocol["phase"],
        "accepted_baseline": protocol["accepted_baseline"],
        "seed_inventory_sha256": protocol["seed_inventory_sha256"],
        "phase_ticks_total_planned": ticks * len(roles),
        "projected_formal_seconds": round(forecast_seconds, 3) if forecast_seconds is not None else None,
        "protocol_sha256": sha256(protocol_path),
        "protocol_snapshot": protocol_snapshot.name,
        "scenario_sha256": sha256(scenario),
        "source": identity,
        "source_sha256": identity["content_sha256"],
        "seed_audit": seed_audit,
        "binary_sha256": binary_sha,
        "probe_snapshot": probe_snapshot.name,
        "policies": protocol["policies"],
        "ticks_per_world": ticks,
        "seed_roles": raw_records,
        "passed": len(outcomes) == len(roles) and all(outcome["passed"] for outcome in outcomes),
        "bounded_fixture_viability": protocol["phase"] == "formal" and len(outcomes) == 3 and all(outcome["passed"] for outcome in outcomes),
        "elapsed_seconds": round(time.time() - started, 3),
        "limits": {"phase_seconds": MAX_PHASE_SECONDS, "parent_address_space_bytes": PARENT_ADDRESS_SPACE, "child_address_space_bytes": CHILD_ADDRESS_SPACE, "aggregate_address_space_bytes": MAX_TOTAL_ADDRESS_SPACE, "aggregate_output_bytes": MAX_OUTPUT_BYTES},
    }
    manifest_path = output_dir / "manifest.json"
    manifest_text = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    status_text = json.dumps({"status": "complete", "phase": protocol["phase"], "passed": manifest["passed"], "elapsed_seconds": manifest["elapsed_seconds"]}, indent=2) + "\n"
    existing_bytes = sum(candidate.stat().st_size for candidate in output_dir.iterdir() if candidate.is_file())
    if existing_bytes + len(manifest_text.encode()) + len(status_text.encode()) > MAX_OUTPUT_BYTES:
        raise RuntimeError("aggregate phase output cap would be exceeded by final metadata")
    manifest_path.write_text(manifest_text)
    (output_dir / "status.json").write_text(status_text)
    return manifest


def estimate_formal_seconds(calibration_probe_seconds: float, calibration_ticks: int) -> float:
    if calibration_ticks <= 0 or calibration_probe_seconds < 0:
        raise ValueError("invalid calibration timing evidence")
    return calibration_probe_seconds * (PHASE_TICKS["formal"] * 3 / calibration_ticks) * 2 + 60


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--scenario", type=Path)
    parser.add_argument("--protocol", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--calibration-manifest", type=Path)
    parser.add_argument("--seed-archive", type=Path, default=CANONICAL_SEED_ARCHIVE)
    parser.add_argument("--scan-only", action="store_true")
    parser.add_argument("--seeds")
    args = parser.parse_args()
    if args.scan_only:
        try:
            if not args.seeds:
                raise ValueError("--seeds is required with --scan-only")
            seeds = [int(value) for value in args.seeds.split(",")]
            audit = scan_seeds(args.seed_archive.resolve(), seeds)
        except Exception as error:
            print(f"ecology_validity_seed_scan: {error}", file=sys.stderr)
            return 2
        print(json.dumps(audit, sort_keys=True))
        return 0
    if args.scenario is None or args.protocol is None or args.output is None:
        parser.error("--scenario, --protocol, and --output are required for a registered phase")
    try:
        manifest = run_phase(args.root.resolve(), args.scenario.resolve(), args.output.resolve(), args.protocol.resolve(), args.binary.resolve() if args.binary else None, args.calibration_manifest.resolve() if args.calibration_manifest else None, args.seed_archive.resolve())
    except Exception as error:
        output_dir = args.output.resolve()
        if RUN_OUTPUT_CREATED and output_dir.is_dir():
            status_path = output_dir / "status.json"
            failure = {"status": "failed", "error": str(error), "phase": "unknown"}
            try:
                phase_data = json.loads(args.protocol.read_text())
                failure["phase"] = phase_data.get("phase", "unknown")
                failure["protocol_sha256"] = sha256(args.protocol.resolve())
                failure["source"] = source_identity(args.root.resolve())
            except Exception:
                pass
            if not status_path.exists():
                status_path.write_text(json.dumps(failure, indent=2, sort_keys=True) + "\n")
            failed_manifest = output_dir / "manifest.failed.json"
            if not failed_manifest.exists():
                failed_manifest.write_text(json.dumps(failure, indent=2, sort_keys=True) + "\n")
        print(f"ecology_validity_runner: {error}", file=sys.stderr)
        return 2
    print(json.dumps(manifest, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
