#!/usr/bin/env python3
"""Verify and summarize the frozen five-case resolver-order lab without running Worlds."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

RUN_DIR = Path("/root/curious/data/experiments/resolver-order-v1-runnerfix-20261026")
RAW_ROOT = Path("/root/curious/data/experiments/water-meal-priority-v1-runnerfix-20261026")
OUTPUT = Path("/root/curious/data/operations/resolver-order-20261026/analysis-final.json")
EXPECTED_RAW_MANIFEST = "db0a490b5f3a62ed9c0f9b8db72b9d5851d1659356d3423dcdec256d9d80134c"
EXPECTED_PROTOCOL = "e8357e8d2d0795ae110579befb416585f50161355f140ff5d80252dc4402f6ff"
EXPECTED_BINARY = "ff32c4fc7cad6e86ad97872e88f9c849ae282814a121c19b54b0193a27c9462b"
EXPECTED_CASES = [
    ("meadow", 62502, 94),
    ("meadow", 62503, 92),
    ("meadow", 62505, 72),
    ("sparse_resource", 62503, 55),
    ("sparse_resource", 62505, 65),
]
RESOLVER_EVENT_TYPES = {"EntityAte", "EntityDrank", "EntityMoved", "ActionRejected"}
TOTAL_FIELDS = (
    "plant_food_energy", "water", "movement_energy", "eat_rejections",
    "drink_rejections", "deaths",
)
ACTOR_FIELDS = (
    "food", "water", "movement_energy", "attack_energy", "eat_rejections",
    "drink_rejections", "deaths",
)
RAW_METRIC_FIELDS = (
    "actor_resolver_events", "plant_food_gained_energy", "water_gained",
    "movement_energy_paid", "plant_food_gained_cumulative", "water_gained_cumulative",
    "movement_energy_cumulative", "food_stock", "water_stock", "focal_decision_trace",
    "live_total", "founders_alive",
)


class AnalysisError(ValueError):
    """Raised when immutable run evidence fails an integrity check."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AnalysisError(f"cannot read JSON {path}: {exc}") from exc


def checked_file(path: Path, expected_sha256: str, expected_bytes: int, label: str) -> None:
    if not path.is_file():
        raise AnalysisError(f"missing {label}: {path}")
    actual_bytes = path.stat().st_size
    actual_hash = sha256_file(path)
    if actual_bytes != expected_bytes or actual_hash != expected_sha256:
        raise AnalysisError(
            f"{label} integrity mismatch at {path}: bytes {actual_bytes}/{expected_bytes}, "
            f"sha256 {actual_hash}/{expected_sha256}"
        )


def expected_source_world(source_manifest: dict, habitat: str, seed: int) -> dict:
    matches = [
        row for row in source_manifest["worlds"]
        if row.get("arm") == "control" and row.get("habitat") == habitat and row.get("seed") == seed
    ]
    if len(matches) != 1:
        raise AnalysisError(f"expected one immutable raw control entry for {habitat}/{seed}, found {len(matches)}")
    return matches[0]


def event_payload(event: dict) -> tuple[str, dict]:
    if not isinstance(event, dict) or len(event) != 1:
        raise AnalysisError(f"malformed resolver event: {event!r}")
    name, payload = next(iter(event.items()))
    if not isinstance(payload, dict):
        raise AnalysisError(f"malformed payload for {name}")
    return name, payload


def actor_id(name: str, payload: dict) -> int | None:
    value = payload.get("id")
    if value is None and name == "EntityAttacked":
        value = payload.get("attacker")
    return int(value) if isinstance(value, int) else None


def derive_event_metrics(events: list[dict]) -> dict:
    totals = dict.fromkeys(TOTAL_FIELDS, 0)
    actor_metrics: dict[int, dict] = {}
    event_counts: Counter[str] = Counter()
    eat_winners = []
    rejects = []
    for wrapped in events:
        name, payload = event_payload(wrapped)
        event_counts[name] += 1
        aid = actor_id(name, payload)
        actor = None
        if aid is not None:
            actor = actor_metrics.setdefault(aid, dict.fromkeys(ACTOR_FIELDS, 0))
        if name == "EntityAte":
            amount = int(payload.get("amount", 0))
            if payload.get("corpse_id") is None:
                totals["plant_food_energy"] += amount
                if actor is not None:
                    actor["food"] += amount
                eat_winners.append({"id": aid, "amount": amount, "pos": payload.get("pos")})
        elif name == "EntityDrank":
            amount = int(payload.get("amount", 0))
            totals["water"] += amount
            if actor is not None:
                actor["water"] += amount
        elif name == "EntityMoved":
            cost = int(payload.get("energy_cost", 0))
            totals["movement_energy"] += cost
            if actor is not None:
                actor["movement_energy"] += cost
        elif name == "EntityAttacked":
            if actor is not None:
                actor["attack_energy"] += int(payload.get("energy_cost", 0))
        elif name == "ActionRejected":
            action = payload.get("action")
            if action == "Eat":
                totals["eat_rejections"] += 1
                if actor is not None:
                    actor["eat_rejections"] += 1
            elif action == "Drink":
                totals["drink_rejections"] += 1
                if actor is not None:
                    actor["drink_rejections"] += 1
            rejects.append({"id": aid, "action": action, "reason": payload.get("reason"), "target": payload.get("target")})
        elif name == "EntityDied":
            totals["deaths"] += 1
            if actor is not None:
                actor["deaths"] += 1
    return {
        "totals": totals,
        "by_actor": {str(key): actor_metrics[key] for key in sorted(actor_metrics)},
        "event_counts": dict(sorted(event_counts.items())),
        "eat_winners": eat_winners,
        "eat_rejections_and_drink_rejections": rejects,
    }


def reconcile_event_totals(branch: dict) -> dict:
    derived = derive_event_metrics(branch["events"])
    recorded = branch.get("event_totals")
    if not isinstance(recorded, dict):
        raise AnalysisError("branch event_totals missing")
    for field, value in derived["totals"].items():
        if recorded.get(field) != value:
            raise AnalysisError(f"event total mismatch {field}: events={value}, recorded={recorded.get(field)}")
    recorded_actors = recorded.get("by_actor", {})
    # ID 0 is the serializer's bucket for environment/lifecycle events without an actor.
    expected_actor_ids = {key for key in derived["by_actor"] if key != "0"}
    recorded_actor_ids = {str(key) for key in recorded_actors if str(key) != "0"}
    if expected_actor_ids != recorded_actor_ids:
        raise AnalysisError(f"actor total ID mismatch: events={sorted(expected_actor_ids)}, recorded={sorted(recorded_actor_ids)}")
    for actor in expected_actor_ids:
        for field in ACTOR_FIELDS:
            actual = derived["by_actor"][actor].get(field, 0)
            seen = recorded_actors[actor].get(field, 0)
            if actual != seen:
                raise AnalysisError(f"actor {actor} {field} mismatch: events={actual}, recorded={seen}")
    return derived


def state_resource_stock(state: dict) -> dict:
    stock = {"food": 0, "water": 0}
    for resource in state.get("resources", []):
        if not isinstance(resource, list) or len(resource) != 2:
            raise AnalysisError("malformed State resource row")
        cell = resource[1]
        if isinstance(cell, dict) and len(cell) == 1:
            kind, body = next(iter(cell.items()))
            if kind in ("Food", "Water"):
                stock[kind.lower()] += int(body["amount"])
    return stock


def body_map(state: dict) -> dict[int, dict]:
    out = {}
    for entity in state.get("entities", []):
        life = entity.get("life", {})
        out[int(entity["id"])] = {
            "pos": entity.get("pos"),
            "phase": entity.get("phase"),
            "energy": life.get("energy"),
            "water": life.get("water"),
            "health": life.get("health"),
            "death_cause": life.get("death_cause"),
        }
    return out


def alive_counts(state: dict, founders: set[int]) -> tuple[int, int]:
    live = founder_live = 0
    for entity in state.get("entities", []):
        phase = entity.get("phase")
        life = entity.get("life", {})
        is_alive = phase in ("Active",) or (isinstance(phase, dict) and "Sleeping" in phase)
        is_alive = is_alive and life.get("death_cause") is None
        if is_alive:
            live += 1
            if int(entity["id"]) in founders:
                founder_live += 1
    return live, founder_live


def validate_prefix_and_target(raw_path: Path, result: dict) -> None:
    rows = {}
    with raw_path.open(encoding="utf-8") as stream:
        header = json.loads(next(stream))
        for line in stream:
            item = json.loads(line)
            if item.get("type") == "tick":
                rows[int(item["tick"])] = item
    if not result.get("prefix_replay_valid"):
        raise AnalysisError("probe prefix_replay_valid is false")
    prefix = result["prefix_replay"]
    replayed = prefix.get("per_tick_validations", [])
    target = int(result["case"]["target_tick"])
    if len(replayed) != target - 1 or prefix.get("validated_through_tick") != target - 1:
        raise AnalysisError("prefix validation horizon does not reach target-1")
    for row in replayed:
        tick = int(row["tick"])
        if tick not in rows:
            raise AnalysisError(f"raw input lacks replay tick {tick}")
        for key in RAW_METRIC_FIELDS:
            if row.get(key) != rows[tick].get(key):
                raise AnalysisError(f"prefix raw mismatch at tick {tick} field {key}")
    target_row = rows.get(target)
    if target_row is None:
        raise AnalysisError(f"raw input lacks target tick {target}")
    target_projection = result.get("target_validation", {})
    if target_projection.get("valid") is not True:
        raise AnalysisError("normal-off target row validation is false")
    for key in RAW_METRIC_FIELDS:
        if target_projection.get(key) != target_row.get(key):
            raise AnalysisError(f"native-off target raw mismatch at tick {target} field {key}")
    if target_projection.get("raw_tick_row") != target_row:
        raise AnalysisError("stored raw target row differs from immutable input")
    if target_projection.get("raw_focal_decision_trace") != target_row.get("focal_decision_trace"):
        raise AnalysisError("stored raw target focal trace differs from immutable input")
    if header.get("arm") != "control" or int(header.get("seed", -1)) != int(result["case"]["seed"]):
        raise AnalysisError("raw control header identity mismatch")
    if int(prefix["target_pre_state"]["tick"]) != target - 1:
        raise AnalysisError("reconstructed target pre-State tick mismatch")
    if prefix.get("target_pre_rng") != prefix["target_pre_state"].get("rng"):
        raise AnalysisError("target pre-State RNG mismatch")


def verify_plan_orders(result: dict) -> None:
    target = int(result["case"]["target_tick"])
    common = result["common_prepared"]
    baseline = common["baseline_plans"]
    candidate = common["candidate_plans"]
    branches = result["branches"]
    kinds = ("move", "eat", "drink", "mate", "attack", "flee", "sleep")

    def ids(plan_map: dict, kind: str) -> list[int]:
        return [int(row["id"]) for row in plan_map[kind]]

    def rotated(order: list[int]) -> list[int]:
        if not order:
            return []
        offset = target % len(order)
        return order[offset:] + order[:offset]

    for kind in kinds:
        if ids(baseline["rotated"], kind) != rotated(ids(baseline["unrotated"], kind)):
            raise AnalysisError(f"A baseline {kind} order does not match production rotation")
        if ids(candidate["rotated_C"], kind) != rotated(ids(candidate["unrotated"], kind)):
            raise AnalysisError(f"C candidate {kind} order does not match production rotation")
        b_rows = candidate["fixed_B"][kind]
        c_rows = candidate["rotated_C"][kind]
        if sorted(map(canonical, b_rows)) != sorted(map(canonical, c_rows)):
            raise AnalysisError(f"B/C {kind} plans differ in content")
        if ids(branches["A"]["plan_orders"], kind) != ids(baseline["rotated"], kind):
            raise AnalysisError(f"A branch {kind} order differs from recorded baseline rotation")
        if ids(branches["C"]["plan_orders"], kind) != ids(candidate["rotated_C"], kind):
            raise AnalysisError(f"C branch {kind} order differs from candidate rotation")

    base_eat = ids(baseline["rotated"], "eat")
    candidate_eat = ids(candidate["rotated_C"], "eat")
    expected_b_eat = [actor for actor in base_eat if actor in set(candidate_eat)]
    expected_b_eat += sorted(set(candidate_eat) - set(base_eat))
    if ids(candidate["fixed_B"], "eat") != expected_b_eat:
        raise AnalysisError("B Eat order violates baseline-survivors then sorted-new-actors rule")
    base_drink = ids(baseline["rotated"], "drink")
    expected_b_drink = [actor for actor in base_drink if actor in set(ids(candidate["fixed_B"], "drink"))]
    if ids(candidate["fixed_B"], "drink") != expected_b_drink:
        raise AnalysisError("B Drink order violates baseline-survivor order rule")
    for kind in set(kinds) - {"eat", "drink"}:
        if ids(candidate["fixed_B"], kind) != ids(baseline["rotated"], kind):
            raise AnalysisError(f"B {kind} order differs from baseline order")
    for branch in "ABC":
        if branches[branch]["plan_orders"] != (baseline["rotated"] if branch == "A" else candidate["fixed_B"] if branch == "B" else candidate["rotated_C"]):
            raise AnalysisError(f"branch {branch} full plan-order records differ from common prepared order")


def canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def branch_report(branch: dict, founders: set[int], input_row: dict | None = None) -> dict:
    derived = reconcile_event_totals(branch)
    totals = derived["totals"]
    state_pre = branch["pre_state"]
    state_post = branch["post_state"]
    bodies_pre = body_map(state_pre)
    bodies_post = body_map(state_post)
    per_actor = dict(derived["by_actor"])
    for actor in sorted(set(bodies_pre) | set(bodies_post)):
        per_actor.setdefault(str(actor), dict.fromkeys(ACTOR_FIELDS, 0))
    post_live, post_founders = alive_counts(state_post, founders)
    stocks_pre = state_resource_stock(state_pre)
    stocks_post = state_resource_stock(state_post)
    if input_row is not None:
        for total_key, raw_key in (("plant_food_energy", "plant_food_gained_energy"), ("water", "water_gained"), ("movement_energy", "movement_energy_paid")):
            if totals[total_key] != input_row.get(raw_key):
                raise AnalysisError(f"A raw target physical total mismatch: {total_key}")
        if stocks_post["food"] != input_row.get("food_stock") or stocks_post["water"] != input_row.get("water_stock"):
            raise AnalysisError("A State resource stocks differ from raw target stocks")
        if branch.get("focal_decision_trace") != input_row.get("focal_decision_trace"):
            raise AnalysisError("A focal decision trace differs from raw target trace")
        if post_live != input_row.get("live_total") or post_founders != input_row.get("founders_alive"):
            raise AnalysisError("A live/founder counts differ from raw target row")
    return {
        "resolver_event_count": len(branch["events"]),
        "resolver_event_counts": derived["event_counts"],
        "physical_totals": totals,
        "by_actor": {key: per_actor[key] for key in sorted(per_actor, key=int)},
        "eat_winners": derived["eat_winners"],
        "rejections": derived["eat_rejections_and_drink_rejections"],
        "plan_order_ids": {kind: [int(row["id"]) for row in rows] for kind, rows in branch["plan_orders"].items()},
        "resources": {
            "pre": stocks_pre,
            "post": stocks_post,
            "delta_post_minus_pre": {key: stocks_post[key] - stocks_pre[key] for key in stocks_pre},
        },
        "post_actor_bodies": bodies_post,
        "post_live_total": post_live,
        "post_founders_alive": post_founders,
        "reference_focal_trace_matches_raw": (
            branch.get("focal_decision_trace") == input_row.get("focal_decision_trace")
            if input_row is not None else None
        ),
        "body_deltas_pre_to_post": body_deltas(bodies_pre, bodies_post),
    }


def body_deltas(before: dict[int, dict], after: dict[int, dict]) -> dict[str, dict]:
    out = {}
    for actor in sorted(set(before) | set(after)):
        pre = before.get(actor)
        post = after.get(actor)
        if pre is None or post is None:
            out[str(actor)] = {"missing_pre_or_post": True, "before": pre, "after": post}
            continue
        out[str(actor)] = {
            field: (post[field] - pre[field]) if isinstance(pre.get(field), int) and isinstance(post.get(field), int) else None
            for field in ("energy", "water", "health")
        }
    return out


def contrast(left: dict, right: dict) -> dict:
    """Return right-minus-left vectors for one paired single-tick case."""
    ltot = left["physical_totals"]
    rtot = right["physical_totals"]
    actor_deltas = {}
    for actor in sorted(set(left["by_actor"]) | set(right["by_actor"]), key=int):
        la = left["by_actor"].get(actor, {})
        ra = right["by_actor"].get(actor, {})
        actor_deltas[actor] = {field: int(ra.get(field, 0)) - int(la.get(field, 0)) for field in ACTOR_FIELDS}
    body_deltas_out = {}
    for actor in sorted(set(left["post_actor_bodies"]) | set(right["post_actor_bodies"])):
        lb = left["post_actor_bodies"].get(actor)
        rb = right["post_actor_bodies"].get(actor)
        body_deltas_out[str(actor)] = {
            field: (rb[field] - lb[field]) if lb is not None and rb is not None and isinstance(lb.get(field), int) and isinstance(rb.get(field), int) else None
            for field in ("energy", "water", "health")
        }
    lcell = winner_cells(left["eat_winners"])
    rcell = winner_cells(right["eat_winners"])
    cell_deltas = {}
    for cell in sorted(set(lcell) | set(rcell)):
        cell_deltas[cell] = {
            "plant_food_energy": rcell.get(cell, {}).get("plant_food_energy", 0) - lcell.get(cell, {}).get("plant_food_energy", 0),
            "left_winners": lcell.get(cell, {}).get("winners", []),
            "right_winners": rcell.get(cell, {}).get("winners", []),
        }
    return {
        "physical_totals_right_minus_left": {field: int(rtot[field]) - int(ltot[field]) for field in TOTAL_FIELDS},
        "by_actor_right_minus_left": actor_deltas,
        "post_body_right_minus_left": body_deltas_out,
        "eat_winner_cells": cell_deltas,
        "eat_winner_events_left": left["eat_winners"],
        "eat_winner_events_right": right["eat_winners"],
    }


def winner_cells(winners: list[dict]) -> dict[str, dict]:
    cells = defaultdict(lambda: {"plant_food_energy": 0, "winners": []})
    for winner in winners:
        pos = winner.get("pos") or {}
        key = f"{pos.get('x')},{pos.get('y')}"
        cells[key]["plant_food_energy"] += int(winner["amount"])
        cells[key]["winners"].append({"id": winner["id"], "amount": winner["amount"]})
    return dict(cells)


def analyze(run_dir: Path = RUN_DIR, raw_root: Path = RAW_ROOT) -> dict:
    manifest_path = run_dir / "manifest.json"
    manifest = load_json(manifest_path)
    source_manifest_path = run_dir / "inputs/source-manifest.json"
    checked_file(source_manifest_path, EXPECTED_RAW_MANIFEST, source_manifest_path.stat().st_size, "copied source manifest")
    source_manifest = load_json(source_manifest_path)
    if manifest.get("raw_source", {}).get("manifest_sha256") != EXPECTED_RAW_MANIFEST:
        raise AnalysisError("run manifest does not name the frozen raw-source manifest hash")
    if manifest.get("status") != "complete" or manifest.get("failure") is not None:
        raise AnalysisError("registered run manifest is not complete")
    if manifest.get("protocol", {}).get("sha256") != EXPECTED_PROTOCOL:
        raise AnalysisError("run manifest protocol hash differs from frozen protocol")
    if manifest.get("binary", {}).get("sha256") != EXPECTED_BINARY:
        raise AnalysisError("run manifest binary hash differs from registered executable")
    cases = manifest.get("cases")
    if not isinstance(cases, list) or len(cases) != len(EXPECTED_CASES):
        raise AnalysisError("run manifest does not contain exactly five registered cases")
    analyses = []
    run_root = run_dir.resolve()
    for case_row, expected in zip(cases, EXPECTED_CASES, strict=True):
        habitat, seed, tick = expected
        identity = (case_row.get("habitat"), case_row.get("seed"), case_row.get("target_tick"))
        if identity != expected:
            raise AnalysisError(f"registered case identity {identity} differs from fixed allocation {expected}")
        if case_row.get("status") != "complete":
            raise AnalysisError(f"case {expected} status is not complete")
        summary = case_row.get("case_summary", {})
        if any(summary.get(flag) is not True for flag in (
            "prefix_replay_valid", "branch_roundtrip_valid", "a_matches_native_off",
            "c_matches_native_on", "candidate_b_equals_c_pre_resolution_audit",
        )):
            raise AnalysisError(f"case {expected} manifest summary has a false integrity gate")
        input_path = Path(case_row["path"])
        result_path = Path(case_row["output"])
        if not input_path.is_absolute(): input_path = run_root / input_path
        if not result_path.is_absolute(): result_path = run_root / result_path
        checked_file(input_path, case_row["sha256"], int(case_row["bytes"]), f"case {expected} raw input copy")
        original_name = Path(case_row["original_path"]).name
        source_entry = expected_source_world(source_manifest, habitat, seed)
        if source_entry.get("raw") != original_name:
            raise AnalysisError(f"case {expected} original raw name differs from frozen source manifest")
        if source_entry.get("raw_sha256") != case_row["sha256"] or source_entry.get("raw_bytes") != case_row["bytes"]:
            raise AnalysisError(f"case {expected} raw input differs from frozen source manifest entry")
        if Path(case_row["original_path"]).name != source_entry["raw"]:
            raise AnalysisError(f"case {expected} original raw path name differs from source manifest")
        checked_file(raw_root / source_entry["raw"], source_entry["raw_sha256"], int(source_entry["raw_bytes"]), f"case {expected} original raw source")
        checked_file(result_path, case_row["result_sha256"], int(case_row["result_bytes"]), f"case {expected} result artifact")
        result = load_json(result_path)
        metadata = result.get("case", {})
        if result.get("status") != "complete" or metadata.get("habitat") != habitat or metadata.get("seed") != seed or metadata.get("target_tick") != tick:
            raise AnalysisError(f"case {expected} result metadata/status mismatch")
        if metadata.get("input_sha256") != case_row["sha256"]:
            raise AnalysisError(f"case {expected} result input hash mismatch")
        if result.get("branch_roundtrip_valid") is not True or result.get("candidate_b_equals_c_pre_resolution_audit", {}).get("equal") is not True:
            raise AnalysisError(f"case {expected} result branch integrity gate failed")
        if result.get("references", {}).get("a_matches_native_off") is not True or result.get("references", {}).get("c_matches_native_on") is not True:
            raise AnalysisError(f"case {expected} native reference gate failed")
        validate_prefix_and_target(input_path, result)
        raw_target = result["target_validation"]["raw_tick_row"]
        header = json.loads(input_path.open(encoding="utf-8").readline())
        founders = {int(entity["id"]) for entity in header["initial_state"]["entities"]}
        verify_plan_orders(result)
        branch_data = {key: branch_report(result["branches"][key], founders, raw_target if key == "A" else None) for key in "ABC"}
        contrasts = {
            "B_minus_A": contrast(branch_data["A"], branch_data["B"]),
            "C_minus_B": contrast(branch_data["B"], branch_data["C"]),
            "C_minus_A": contrast(branch_data["A"], branch_data["C"]),
        }
        analyses.append({
            "case": {"habitat": habitat, "seed": seed, "target_tick": tick},
            "input": {
                "path": str(input_path), "sha256": case_row["sha256"], "bytes": case_row["bytes"],
                "original_path": case_row["original_path"], "original_sha256": source_entry["raw_sha256"],
            },
            "result": {"path": str(result_path), "sha256": case_row["result_sha256"], "bytes": case_row["result_bytes"]},
            "checks": {
                "prefix_raw_rows": tick - 1,
                "target_native_off_matches_raw": True,
                "A_matches_raw_target": True,
                "plan_order_contract": True,
                "event_totals_reconciled_for_ABC": True,
                "branch_roundtrip_valid": True,
                "A_matches_native_off": True,
                "C_matches_native_on": True,
            },
            "branches": branch_data,
            "contrasts": contrasts,
            "interpretation": {
                "B_minus_A": "Candidate membership plus the protocol's fixed placement rule.",
                "C_minus_B": "Joint natural-rotation differences across plan lists, including Eat and Drink; not a pure Eat-only effect.",
                "C_minus_A": "Candidate membership plus natural rotation relative to baseline.",
                "scope": "One selected resolver tick in this case; no lifetime, survival, population, or general benefit claim.",
            },
        })
    return {
        "schema": "curious-resolver-order-v1-offline-analysis",
        "status": "complete_descriptive_five_case_mechanism_analysis",
        "registered_run": {
            "run_id": manifest.get("run_id"),
            "manifest_path": str(manifest_path),
            "manifest_sha256": sha256_file(manifest_path),
            "protocol_sha256": manifest["protocol"]["sha256"],
            "source_commit": manifest.get("source", {}).get("commit"),
            "tracked_file_count": manifest.get("source", {}).get("tracked_file_count"),
            "tracked_tree_sha256": manifest.get("source", {}).get("tracked_tree_sha256"),
            "binary_path": manifest["binary"]["path"],
            "binary_sha256": manifest["binary"]["sha256"],
            "raw_source_manifest_sha256": manifest["raw_source"]["manifest_sha256"],
            "registration_addendum": manifest.get("registration_addendum"),
            "runner_repair_note": manifest.get("runner_repair_note"),
            "resolved_tick_invocations": manifest["limits"]["resolved_tick_invocations"],
            "shared_prepared_preambles": manifest["limits"]["shared_prepared_preambles"],
        },
        "support": {"paired_selected_single_tick_cases": len(analyses), "case_identities": [case["case"] for case in analyses]},
        "pooled_effect_estimate": None,
        "cases": analyses,
        "limits": [
            "Five selected negative diagnostic cases only; not a random or representative sample.",
            "B-C includes joint Eat and Drink (and other candidate-list rotation) order differences; it is not a pure Eat-order contrast.",
            "No long-horizon, lifetime, survival, lineage, population, or general policy benefit inference.",
            "No averages, significance tests, composite score, or acceptance threshold are defined.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-dir", type=Path, default=RUN_DIR)
    parser.add_argument("--raw-root", type=Path, default=RAW_ROOT)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    args = parser.parse_args()
    report = analyze(args.run_dir, args.raw_root)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if args.output.exists():
        raise AnalysisError(f"refusing to overwrite existing analysis output: {args.output}")
    temp = args.output.with_suffix(args.output.suffix + ".tmp")
    content = json.dumps(report, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
    with temp.open("x", encoding="utf-8") as stream:
        stream.write(content)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temp, args.output)
    print(json.dumps({"status": report["status"], "cases": len(report["cases"]), "output": str(args.output), "bytes": args.output.stat().st_size, "sha256": sha256_file(args.output)}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
