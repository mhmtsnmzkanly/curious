#!/usr/bin/env python3
"""Fail-closed supervisor for the preregistered three-arm resolver-order cases."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import resource
import signal
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any


RUN_ID = "resolver-order-v1-20261026"
RAW_ROOT = Path("/root/curious/data/experiments/water-meal-priority-v1-runnerfix-20261026")
RAW_MANIFEST_SHA256 = "db0a490b5f3a62ed9c0f9b8db72b9d5851d1659356d3423dcdec256d9d80134c"
PROTOCOL_SHA256 = "e8357e8d2d0795ae110579befb416585f50161355f140ff5d80252dc4402f6ff"
ADDENDUM_SHA256 = "7a271bbc907cb940b4190f1a734896b304f202a92c11282772c3719b4158a65b"
PROTOCOL_REL = Path("docs/RESOLVER_ORDER_V1_PROTOCOL_20261026.md")
ADDENDUM_REL = Path("docs/RESOLVER_ORDER_V1_REGISTRATION_ADDENDUM_20261026.md")
MAX_PHASE_SECONDS = 180.0
MAX_CHILD_SECONDS = 45.0
MAX_BYTES = 64 * 1024 * 1024
# Reserve room for the final registration update while enforcing the aggregate cap.
MAX_CHILD_ARTIFACT_BYTES = MAX_BYTES - 1024 * 1024
MAX_ADDRESS_SPACE = 1024 * 1024 * 1024
CASES = (
    ("meadow", 62502, 94),
    ("meadow", 62503, 92),
    ("meadow", 62505, 72),
    ("sparse_resource", 62503, 55),
    ("sparse_resource", 62505, 65),
)
REQUIRED_NORMALIZATIONS = {"policy.water_meal_priority_enabled only"}
REQUIRED_REFERENCE_FIELDS = {"events", "rng", "last_intents", "last_decisions", "state"}
PLAN_KINDS = {"move", "eat", "drink", "mate", "attack", "flee", "sleep"}


class RunFailure(RuntimeError):
    pass


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def atomic_json(path: Path, value: Any) -> None:
    temp = path.with_name(path.name + ".tmp")
    payload = json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False).encode("utf-8") + b"\n"
    with temp.open("xb") as f:
        f.write(payload)
        f.flush()
        os.fsync(f.fileno())
    os.replace(temp, path)


def tree_identity(root: Path) -> dict[str, Any]:
    def git(*args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(root), *args], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        ).stdout.decode("utf-8").strip()

    status = git("status", "--porcelain=v1", "--untracked-files=all")
    if status:
        raise RunFailure(f"source tree is not clean: {status[:1200]}")
    commit = git("rev-parse", "HEAD")
    paths = git("ls-files", "-z").split("\x00")
    h = hashlib.sha256()
    count = 0
    for rel in paths:
        if not rel:
            continue
        path = root / rel
        if not path.is_file():
            raise RunFailure(f"tracked source file is missing: {rel}")
        content_hash = sha256_file(path)
        h.update(rel.encode("utf-8"))
        h.update(b"\0")
        h.update(bytes.fromhex(content_hash))
        count += 1
    return {"commit": commit, "tracked_file_count": count, "tracked_tree_sha256": h.hexdigest()}


def assert_hash(path: Path, expected: str, label: str) -> str:
    if not path.is_file():
        raise RunFailure(f"{label} missing: {path}")
    actual = sha256_file(path)
    if actual != expected:
        raise RunFailure(f"{label} SHA-256 mismatch: expected {expected}, got {actual}")
    return actual


def validate_raw_inputs(raw_root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    manifest_path = raw_root / "manifest.json"
    assert_hash(manifest_path, RAW_MANIFEST_SHA256, "immutable raw manifest")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("status") != "complete" or manifest.get("schema") != "curious-water-meal-priority-v1-manifest":
        raise RunFailure("immutable raw manifest has unexpected schema/status")
    entries = {}
    for item in manifest.get("worlds", []):
        key = (item.get("habitat"), item.get("seed"), item.get("arm"))
        if key in entries:
            raise RunFailure(f"duplicate manifest world entry: {key}")
        entries[key] = item
    selected = []
    for habitat, seed, target_tick in CASES:
        key = (habitat, seed, "control")
        item = entries.get(key)
        if not item:
            raise RunFailure(f"raw manifest is missing fixed input: {key}")
        path = (raw_root / item["raw"]).resolve(strict=True)
        if path.parent != raw_root.resolve(strict=True):
            raise RunFailure(f"raw manifest input escaped root: {item['raw']}")
        size = path.stat().st_size
        digest = sha256_file(path)
        if size != item.get("raw_bytes") or digest != item.get("raw_sha256"):
            raise RunFailure(f"immutable raw input mismatch: {item['raw']}")
        header = None
        targets: set[int] = set()
        with path.open("r", encoding="utf-8") as f:
            first = f.readline()
            if len(first.encode("utf-8")) > MAX_BYTES:
                raise RunFailure(f"raw header exceeds line cap: {item['raw']}")
            header = json.loads(first)
            if header.get("arm") != "control" or header.get("habitat") != habitat or header.get("seed") != seed:
                raise RunFailure(f"raw header case identity mismatch: {item['raw']}")
            if not isinstance(header.get("initial_state"), dict):
                raise RunFailure(f"raw header lacks full initial_state: {item['raw']}")
            if "rng" not in header["initial_state"]:
                raise RunFailure(f"raw initial_state lacks RNG state: {item['raw']}")
            for line in f:
                if len(line.encode("utf-8")) > MAX_BYTES:
                    raise RunFailure(f"raw row exceeds line cap: {item['raw']}")
                row = json.loads(line)
                if row.get("type") == "tick" and row.get("tick") in (target_tick - 1, target_tick):
                    targets.add(row["tick"])
        if targets != {target_tick - 1, target_tick}:
            raise RunFailure(f"raw input lacks required prefix/target rows: {item['raw']}")
        selected.append({
            "habitat": habitat,
            "seed": seed,
            "target_tick": target_tick,
            "path": str(path),
            "bytes": size,
            "sha256": digest,
        })
    return manifest, selected


def run_dir_bytes(path: Path) -> int:
    total = 0
    for child in path.rglob("*"):
        if child.is_file():
            try:
                total += child.stat().st_size
            except FileNotFoundError:
                continue
    return total


def verify_input_copies(selected: list[dict[str, Any]], manifest_copy: Path) -> None:
    for item in selected:
        copied_input = Path(item["path"])
        if copied_input.stat().st_size != item["bytes"] or sha256_file(copied_input) != item["sha256"]:
            raise RunFailure(f"child modified immutable input copy: {copied_input.name}")
    if sha256_file(manifest_copy) != RAW_MANIFEST_SHA256:
        raise RunFailure("child modified immutable raw manifest copy")


def validate_case_json(path: Path, expected: dict[str, Any]) -> dict[str, Any]:
    if not path.is_file():
        raise RunFailure("child did not write per-case JSON")
    if path.stat().st_size > MAX_BYTES:
        raise RunFailure("per-case JSON exceeds 64 MiB")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RunFailure(f"invalid per-case JSON: {exc}") from exc
    if data.get("schema") != "curious-resolver-order-v1-case":
        raise RunFailure("per-case JSON has wrong schema")
    case = data.get("case", {})
    for key in ("habitat", "seed", "target_tick"):
        if case.get(key) != expected[key]:
            raise RunFailure(f"per-case identity mismatch for {key}")
    if Path(case.get("input_path", "")).resolve() != Path(expected["path"]).resolve():
        raise RunFailure("per-case input path mismatch")
    if case.get("input_sha256") != expected["sha256"]:
        raise RunFailure("per-case input SHA-256 mismatch")
    if data.get("status") != "complete":
        raise RunFailure(f"child case status is not complete: {data.get('failure', 'unknown')}")
    prefix = data.get("prefix_replay", {})
    validations = prefix.get("per_tick_validations")
    if not isinstance(validations, list) or len(validations) != expected["target_tick"] - 1:
        raise RunFailure("prefix replay does not record every initial-state-to-target-1 tick")
    if prefix.get("from_initial_state_through") != expected["target_tick"] - 1:
        raise RunFailure("prefix replay endpoint mismatch")
    if not isinstance(prefix.get("target_pre_state"), dict) or "target_pre_rng" not in prefix:
        raise RunFailure("prefix replay omits reconstructed target-pre State/RNG")
    per_tick_fields = {
        "tick", "actor_resolver_events", "plant_food_gained_energy", "water_gained",
        "movement_energy_paid", "plant_food_gained_cumulative", "water_gained_cumulative",
        "movement_energy_cumulative", "food_stock", "water_stock", "focal_decision_trace",
        "live_total", "founders_alive",
    }
    for expected_tick, row in enumerate(validations, 1):
        if not isinstance(row, dict) or not per_tick_fields <= set(row) or row.get("tick") != expected_tick:
            raise RunFailure("prefix replay tick validation is incomplete or out of order")
        if not isinstance(row["actor_resolver_events"], list):
            raise RunFailure("prefix replay resolver events are malformed")
        for field in per_tick_fields - {"tick", "actor_resolver_events", "focal_decision_trace"}:
            if not isinstance(row[field], int):
                raise RunFailure(f"prefix replay metric is not an integer: {field}")
    for key in ("prefix_replay_valid", "branch_roundtrip_valid"):
        if data.get(key) is not True:
            raise RunFailure(f"required case validity gate failed: {key}")
    pre_roundtrip = data.get("pre_target_roundtrip", {})
    if not all(pre_roundtrip.get(k) is True for k in ("valid", "state_equal", "rng_equal")):
        raise RunFailure("pre-target State/RNG round-trip gate failed")
    audit = data.get("candidate_b_equals_c_pre_resolution_audit", {})
    if not all(audit.get(k) is True for k in ("equal", "learner_interruptions_equal", "decision_trace_equal", "plan_content_equal")):
        raise RunFailure("B/C pre-resolution audit mismatch")
    refs = data.get("references", {})
    fields = set(refs.get("compared_fields", []))
    normalizations = set(refs.get("normalizations", []))
    if not REQUIRED_REFERENCE_FIELDS <= fields:
        raise RunFailure("native references omit required full-equivalence fields")
    if normalizations != REQUIRED_NORMALIZATIONS:
        raise RunFailure("native reference normalizations differ from preregistered policy-flag-only rule")
    branches = data.get("branches", {})
    if set(branches) != {"A", "B", "C"}:
        raise RunFailure("per-case JSON does not contain exactly branches A/B/C")
    for name in ("A", "B", "C"):
        branch = branches[name]
        for key in ("pre_state", "pre_rng", "plan_orders", "pre_actor_bodies", "events", "event_totals", "post_state", "post_rng", "post_actor_bodies"):
            if key not in branch:
                raise RunFailure(f"branch {name} omits {key}")
        if not isinstance(branch["pre_state"], dict) or not isinstance(branch["post_state"], dict):
            raise RunFailure(f"branch {name} State snapshots are not objects")
        if not isinstance(branch["plan_orders"], dict) or not isinstance(branch["events"], list):
            raise RunFailure(f"branch {name} plan/event records have wrong type")
        if not isinstance(branch["event_totals"], dict) or not isinstance(branch["pre_actor_bodies"], list) or not isinstance(branch["post_actor_bodies"], list):
            raise RunFailure(f"branch {name} physical records have wrong type")
        if set(branch["plan_orders"]) != PLAN_KINDS or any(not isinstance(v, list) for v in branch["plan_orders"].values()):
            raise RunFailure(f"branch {name} plan-order map is incomplete or malformed")
    if branches["B"]["pre_state"] != branches["C"]["pre_state"] or branches["B"]["pre_rng"] != branches["C"]["pre_rng"]:
        raise RunFailure("B/C fork State or RNG differs before resolution")
    if not (branches["A"]["pre_actor_bodies"] == branches["B"]["pre_actor_bodies"] == branches["C"]["pre_actor_bodies"]):
        raise RunFailure("fork arms do not share identical pre-resolution physical bodies")

    common = data.get("common_prepared", {})
    if not isinstance(common.get("state"), dict) or "rng" not in common:
        raise RunFailure("common preparation omits State/RNG")
    actor_inputs = common.get("actor_inputs")
    if not isinstance(actor_inputs, list) or not actor_inputs or any(not isinstance(a, dict) or not {"id", "pos", "phase", "body", "perception", "decision_trace"} <= set(a) for a in actor_inputs):
        raise RunFailure("common preparation omits full per-actor visible inputs/decision traces")
    actor_ids = [a["id"] for a in actor_inputs]
    if any(not isinstance(actor_id, int) for actor_id in actor_ids) or len(set(actor_ids)) != len(actor_ids):
        raise RunFailure("common preparation actor IDs are invalid or duplicated")
    if not isinstance(common.get("baseline_intents"), list) or not isinstance(common.get("candidate_intents"), list):
        raise RunFailure("common preparation omits baseline/candidate intents")
    if not isinstance(common.get("candidate_overrides"), list):
        raise RunFailure("common preparation omits candidate override records")
    baseline_plans = common.get("baseline_plans", {})
    candidate_plans = common.get("candidate_plans", {})
    for source, names in ((baseline_plans, {"unrotated", "rotated"}), (candidate_plans, {"unrotated", "fixed_B", "rotated_C"})):
        if not isinstance(source, dict) or not names <= set(source):
            raise RunFailure("common preparation omits required unrotated/fixed/rotated plans")
        for label in names:
            plan_set = source[label]
            if not isinstance(plan_set, dict) or set(plan_set) != PLAN_KINDS or any(not isinstance(v, list) for v in plan_set.values()):
                raise RunFailure(f"common prepared plan set is malformed: {label}")
    if branches["A"]["plan_orders"] != baseline_plans["rotated"]:
        raise RunFailure("A branch plans differ from shared natural baseline rotation")
    if branches["B"]["plan_orders"] != candidate_plans["fixed_B"]:
        raise RunFailure("B branch does not use registered fixed placement")
    if branches["C"]["plan_orders"] != candidate_plans["rotated_C"]:
        raise RunFailure("C branch does not use candidate natural rotation")
    def plan_content(plan_set: dict[str, Any]) -> dict[str, list[str]]:
        output = {}
        for kind, rows in plan_set.items():
            if any(not isinstance(row, dict) or not isinstance(row.get("id"), int) for row in rows):
                raise RunFailure(f"plan rows lack actor IDs: {kind}")
            if kind in {"mate", "attack"} and any(not isinstance(row.get("target_id"), int) for row in rows):
                raise RunFailure(f"targeted plan rows lack target IDs: {kind}")
            output[kind] = sorted(json.dumps(row, sort_keys=True, separators=(",", ":")) for row in rows)
        return output
    if plan_content(candidate_plans["fixed_B"]) != plan_content(candidate_plans["rotated_C"]):
        raise RunFailure("B/C candidate plan content differs beyond prescribed ordering")
    def normalized_state(state: Any) -> Any:
        if not isinstance(state, dict) or not isinstance(state.get("policy"), dict):
            raise RunFailure("reference State lacks the policy object")
        enabled = state["policy"].get("water_meal_priority_enabled", False)
        if not isinstance(enabled, bool):
            raise RunFailure("reference State policy flag is not boolean")
        return {**state, "policy": {k: v for k, v in state["policy"].items() if k != "water_meal_priority_enabled"}}

    native_refs = (("native_off", "A", "a_matches_native_off"), ("native_on", "C", "c_matches_native_on"))
    for reference_name, branch_name, flag_name in native_refs:
        reference = refs.get(reference_name)
        branch = branches[branch_name]
        if not isinstance(reference, dict):
            raise RunFailure(f"missing native reference payload: {reference_name}")
        reference_flag = reference.get("state", {}).get("policy", {}).get("water_meal_priority_enabled", False)
        if reference_flag is not (reference_name == "native_on"):
            raise RunFailure(f"native reference has unexpected feature flag: {reference_name}")
        for key in REQUIRED_REFERENCE_FIELDS:
            if key not in reference:
                raise RunFailure(f"native reference {reference_name} omits {key}")
        branch_values = {
            "events": branch["events"],
            "rng": branch["post_rng"],
            "last_intents": branch.get("post_last_intents"),
            "last_decisions": branch.get("post_last_decisions"),
            "state": branch["post_state"],
        }
        if branch_values["last_intents"] is None or branch_values["last_decisions"] is None:
            raise RunFailure(f"branch {branch_name} omits post-resolution intent/decision snapshots")
        equal = True
        for key in REQUIRED_REFERENCE_FIELDS:
            if key == "state":
                equal = equal and normalized_state(reference[key]) == normalized_state(branch_values[key])
            else:
                equal = equal and reference[key] == branch_values[key]
        if not equal or refs.get(flag_name) is not True:
            raise RunFailure(f"full native reference equality failed: {reference_name}")
    return data


def child_limits(max_file_size: int) -> None:
    resource.setrlimit(resource.RLIMIT_AS, (MAX_ADDRESS_SPACE, MAX_ADDRESS_SPACE))
    resource.setrlimit(resource.RLIMIT_FSIZE, (max_file_size, max_file_size))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def terminate_child(proc: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", required=True, type=Path)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--raw-root", type=Path, default=RAW_ROOT)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()

    output_dir = args.output_dir.expanduser()
    if not output_dir.is_absolute():
        parser.error("--output-dir must be absolute")
    try:
        output_dir.mkdir(mode=0o700, parents=False, exist_ok=False)
    except FileExistsError:
        parser.error("--output-dir already exists; runs are never overwritten")
    manifest_path = output_dir / "manifest.json"
    run_manifest: dict[str, Any] = {
        "run_id": RUN_ID,
        "status": "preflight",
        "started_utc_epoch": time.time(),
        "cases": [],
        "failure": None,
    }
    active_record: dict[str, Any] | None = None
    started = time.monotonic()
    atomic_json(manifest_path, run_manifest)

    def fail(message: str, case_index: int | None = None) -> int:
        if active_record is not None and active_record.get("status") == "running":
            active_record["status"] = "failed"
            active_record["failure"] = message
        if run_manifest.get("failure") is None:
            run_manifest["failure"] = {"message": message, "case_index": case_index, "time_utc_epoch": time.time()}
        run_manifest["status"] = "failed"
        run_manifest["finished_utc_epoch"] = time.time()
        run_manifest["elapsed_seconds"] = round(time.monotonic() - started, 6)
        try:
            atomic_json(manifest_path, run_manifest)
        except FileExistsError:
            temp = manifest_path.with_name("manifest.failure.tmp")
            temp.write_text(json.dumps(run_manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8")
            os.replace(temp, manifest_path)
        return 1

    try:
        source_root = args.source_root.resolve(strict=True)
        binary = args.binary.resolve(strict=True)
        raw_root = args.raw_root.resolve(strict=True)
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise RunFailure(f"binary is not executable: {binary}")
        source = tree_identity(source_root)
        protocol_path = source_root / PROTOCOL_REL
        addendum_path = source_root / ADDENDUM_REL
        protocol_sha = assert_hash(protocol_path, PROTOCOL_SHA256, "frozen protocol")
        addendum_sha = assert_hash(addendum_path, ADDENDUM_SHA256, "registration addendum")
        binary_sha = sha256_file(binary)
        raw_manifest, selected = validate_raw_inputs(raw_root)
        supervisor_path = Path(__file__).resolve()
        supervisor_source = tree_identity(supervisor_path.parents[1])
        (output_dir / "inputs").mkdir(mode=0o700)
        raw_manifest_copy = output_dir / "inputs" / "source-manifest.json"
        shutil.copyfile(raw_root / "manifest.json", raw_manifest_copy)
        if sha256_file(raw_manifest_copy) != RAW_MANIFEST_SHA256:
            raise RunFailure("copied immutable raw manifest failed hash verification")
        for item in selected:
            original = Path(item["path"])
            copy_path = output_dir / "inputs" / original.name
            shutil.copyfile(original, copy_path)
            if copy_path.stat().st_size != item["bytes"] or sha256_file(copy_path) != item["sha256"]:
                raise RunFailure(f"copied raw input failed immutable hash verification: {original.name}")
            copy_path.chmod(0o400)
            item["original_path"] = item["path"]
            item["path"] = str(copy_path.resolve())
        raw_manifest_copy.chmod(0o400)
        run_manifest.update({
            "source": {"root": str(source_root), **source},
            "supervisor": {"path": str(supervisor_path), "sha256": sha256_file(supervisor_path), **supervisor_source},
            "binary": {"path": str(binary), "sha256": binary_sha},
            "protocol": {"path": str(protocol_path), "sha256": protocol_sha},
            "registration_addendum": {"path": str(addendum_path), "sha256": addendum_sha},
            "raw_source": {
                "root": str(raw_root),
                "manifest_sha256": RAW_MANIFEST_SHA256,
                "run_copy": str(raw_manifest_copy.resolve()),
            },
            "selected_inputs": selected,
            "limits": {
                "phase_seconds": MAX_PHASE_SECONDS,
                "child_seconds": MAX_CHILD_SECONDS,
                "child_address_space_bytes": MAX_ADDRESS_SPACE,
                "child_file_size_bytes": MAX_CHILD_ARTIFACT_BYTES,
                "aggregate_child_artifact_headroom_bytes": MAX_CHILD_ARTIFACT_BYTES,
                "aggregate_run_bytes": MAX_BYTES,
                "resolved_tick_invocations": 398,
                "shared_prepared_preambles": 5,
            },
            "command_template": [str(binary), "INPUT_CONTROL.jsonl", "TARGET_TICK", "INPUT_SHA256", "OUTPUT.json"],
            "status": "running",
            "registration_utc_epoch": time.time(),
        })
        atomic_json(manifest_path, run_manifest)

        for index, item in enumerate(selected):
            elapsed = time.monotonic() - started
            if elapsed >= MAX_PHASE_SECONDS:
                raise RunFailure("180-second phase wall-clock budget exceeded")
            case_name = f"{index + 1:02d}-{item['habitat']}-seed{item['seed']}-t{item['target_tick']}"
            case_dir = output_dir / case_name
            case_dir.mkdir(mode=0o700)
            out_json = case_dir / "result.json"
            log_path = case_dir / "child.log"
            record = {**item, "status": "running", "output": str(out_json), "log": str(log_path)}
            active_record = record
            run_manifest["cases"].append(record)
            atomic_json(manifest_path, run_manifest)
            command = [str(binary), item["path"], str(item["target_tick"]), item["sha256"], str(out_json)]
            record["command"] = command
            atomic_json(manifest_path, run_manifest)
            child_file_limit = MAX_CHILD_ARTIFACT_BYTES - run_dir_bytes(output_dir)
            if child_file_limit <= 0:
                raise RunFailure("aggregate run output has no remaining child budget")
            child_elapsed = time.monotonic()
            with log_path.open("xb", buffering=0) as log:
                proc = subprocess.Popen(
                    command,
                    cwd=source_root,
                    stdin=subprocess.DEVNULL,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    close_fds=True,
                    start_new_session=True,
                    preexec_fn=lambda: child_limits(child_file_limit),
                )
                while proc.poll() is None:
                    elapsed = time.monotonic() - started
                    child_seconds = time.monotonic() - child_elapsed
                    if elapsed >= MAX_PHASE_SECONDS:
                        terminate_child(proc)
                        proc.wait()
                        verify_input_copies(selected, raw_manifest_copy)
                        raise RunFailure("180-second phase wall-clock budget exceeded",)
                    if child_seconds >= MAX_CHILD_SECONDS:
                        terminate_child(proc)
                        proc.wait()
                        verify_input_copies(selected, raw_manifest_copy)
                        raise RunFailure(f"child exceeded 45-second limit: {case_name}")
                    if run_dir_bytes(output_dir) > MAX_CHILD_ARTIFACT_BYTES:
                        terminate_child(proc)
                        proc.wait()
                        verify_input_copies(selected, raw_manifest_copy)
                        raise RunFailure(f"aggregate run artifact headroom exceeded: {case_name}")
                    time.sleep(0.05)
            verify_input_copies(selected, raw_manifest_copy)
            if proc.returncode != 0:
                raise RunFailure(f"child exited with code {proc.returncode}: {case_name}")
            if run_dir_bytes(output_dir) > MAX_CHILD_ARTIFACT_BYTES:
                raise RunFailure(f"aggregate run artifact headroom exceeded: {case_name}")
            result = validate_case_json(out_json, item)
            if time.monotonic() - started >= MAX_PHASE_SECONDS:
                raise RunFailure("180-second phase wall-clock budget exceeded")
            record["status"] = "complete"
            record["result_sha256"] = sha256_file(out_json)
            record["result_bytes"] = out_json.stat().st_size
            record["case_summary"] = {
                "prefix_replay_valid": result["prefix_replay_valid"],
                "branch_roundtrip_valid": result["branch_roundtrip_valid"],
                "a_matches_native_off": result["references"]["a_matches_native_off"],
                "c_matches_native_on": result["references"]["c_matches_native_on"],
                "candidate_b_equals_c_pre_resolution_audit": result["candidate_b_equals_c_pre_resolution_audit"]["equal"],
            }
            atomic_json(manifest_path, run_manifest)
            active_record = None
        if time.monotonic() - started >= MAX_PHASE_SECONDS:
            raise RunFailure("180-second phase wall-clock budget exceeded")
        run_manifest["status"] = "complete"
        run_manifest["finished_utc_epoch"] = time.time()
        run_manifest["elapsed_seconds"] = round(time.monotonic() - started, 6)
        for _ in range(3):
            if time.monotonic() - started >= MAX_PHASE_SECONDS:
                raise RunFailure("180-second phase wall-clock budget exceeded")
            atomic_json(manifest_path, run_manifest)
            actual_bytes = run_dir_bytes(output_dir)
            if actual_bytes > MAX_BYTES:
                raise RunFailure("aggregate run output exceeded 64 MiB")
            if run_manifest.get("aggregate_output_bytes") == actual_bytes:
                break
            run_manifest["aggregate_output_bytes"] = actual_bytes
        if time.monotonic() - started >= MAX_PHASE_SECONDS:
            raise RunFailure("180-second phase wall-clock budget exceeded")
        return 0
    except Exception as exc:
        return fail(str(exc))


if __name__ == "__main__":
    raise SystemExit(main())
