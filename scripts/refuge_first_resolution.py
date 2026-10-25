#!/usr/bin/env python3
"""Read-only extractor for one complete refuge resolver interval per start.

The real-capture CLI is deliberately pinned and requires a clean, reviewed
commit plus a new output directory. Unit tests call audit_records() on fixtures.
This tool does not run a World or fit/update a predictor.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from collections import Counter, defaultdict
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / "data/experiments/refuge-risk-baseline-v1-engineering-20261025/raw.jsonl"
SOURCE_MANIFEST = ROOT / "data/experiments/refuge-risk-baseline-v1-engineering-20261025/manifest.json"
REFERENCE_AUDIT = ROOT / "data/operations/wave9-20261025/baseline-independent-audit.json"
PROTOCOL = ROOT / "docs/REFUGE_FIRST_RESOLUTION_PROTOCOL.md"
OUTPUT_ROOT = ROOT / "data/experiments/refuge-first-resolution-v1-20261025"
RAW_SHA256 = "d9c7e00e976c2fe31e0ad07868df924cabd423b91c6f5c32540b4b2cd43b4874"
SOURCE_MANIFEST_SHA256 = "b5161962b0939b6389f07f64d19458572703e55b721a3383b7eaf0202b2aef12"
REFERENCE_AUDIT_SHA256 = "8e22728c1347a7b2fe91bd7c322591cf99424a999d6597e76a2c4361d55e00de"
SOURCE_EXPERIMENT_COMMIT = "0377d4c66d4ea81bf1ea83e0c5305c00849f963d"
SOURCE_ENGINE_COMMIT = "117a440431deead231ac6d4edbe324be2345936a"
SCENARIO_SHA256 = "9641a7a60157edcd9b3d379210b26aead5ed57df56ec242dca5c604a74d478f2"
MAX_INPUT_BYTES = 128 * 1024 * 1024
MAX_RECORDS = 2_000_000
MAX_OUTPUT_BYTES = 64 * 1024 * 1024


class AuditError(ValueError):
    """Input is inconsistent, duplicated, or outside the frozen protocol."""


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _integer(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _normalized_attacks(events: list[Any], *, local: bool) -> list[tuple[int, int, int]]:
    normalized: list[tuple[int, int, int]] = []
    for event in events:
        if not isinstance(event, dict):
            raise AuditError("malformed attack witness")
        attacker = event.get("attacker_id" if local else "attacker")
        target = event.get("target_id" if local else "target")
        damage = event.get("damage")
        if not all(_integer(value) for value in (attacker, target, damage)) or damage < 0:
            raise AuditError("attack witness has invalid attacker/target/damage")
        normalized.append((attacker, target, damage))
    return sorted(normalized)


def _start_identity(value: dict[str, Any]) -> bytes:
    # Closure copies the start payload but omits its top-level record tag.
    return canonical({key: item for key, item in value.items() if key != "record_type"})


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    size = path.stat().st_size
    if size > MAX_INPUT_BYTES:
        raise AuditError(f"input exceeds {MAX_INPUT_BYTES} bytes")
    records: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            if len(records) >= MAX_RECORDS:
                raise AuditError(f"input exceeds {MAX_RECORDS} records")
            try:
                item = json.loads(line)
            except json.JSONDecodeError as error:
                raise AuditError(f"invalid JSON on line {line_number}: {error.msg}") from None
            if not isinstance(item, dict) or not isinstance(item.get("record_type"), str):
                raise AuditError(f"line {line_number} is not a typed record")
            records.append(item)
    return records


def _record_index(records: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        grouped[record["record_type"]].append(record)
    for name in ("header", "summary"):
        if len(grouped.get(name, [])) != 1:
            raise AuditError(f"expected exactly one {name} record")
    return grouped


def _validate_header_and_global(groups: dict[str, list[dict[str, Any]]]) -> tuple[dict[int, dict[str, Any]], dict[int, list[dict[str, Any]]]]:
    header = groups["header"][0]
    if (
        header.get("protocol") != "refuge_risk_baseline_v1_engineering"
        or header.get("scenario_sha256") != SCENARIO_SHA256
        or header.get("seed") != 123
        or header.get("physical_tick_limit") != 1000
        or header.get("refuge_feature_flags_changed") is not False
    ):
        raise AuditError("header does not match the pinned baseline input identity")

    tick_records: dict[int, dict[str, Any]] = {}
    global_events: dict[int, list[dict[str, Any]]] = {}
    for record in groups.get("tick", []):
        tick = record.get("tick")
        if not _integer(tick) or tick in tick_records:
            raise AuditError("duplicate or invalid world tick record")
        tick_records[tick] = record
    for record in groups.get("resolver_events", []):
        tick = record.get("tick")
        events = record.get("events")
        if not _integer(tick) or tick in global_events or not isinstance(events, list):
            raise AuditError("duplicate or malformed global resolver witness")
        indexes: set[int] = set()
        ordered_indexes: list[int] = []
        event_count = tick_records.get(tick, {}).get("resolver_events")
        if not _integer(event_count) or event_count < 0:
            raise AuditError(f"resolver witness has no valid event count at tick {tick}")
        for event in events:
            if not isinstance(event, dict) or not _integer(event.get("event_index")):
                raise AuditError(f"malformed resolver event at tick {tick}")
            if event["event_index"] < 0 or event["event_index"] >= event_count:
                raise AuditError(f"resolver event index outside tick event count at tick {tick}")
            if event["event_index"] in indexes:
                raise AuditError(f"duplicate global event index at tick {tick}")
            indexes.add(event["event_index"])
            ordered_indexes.append(event["event_index"])
        if ordered_indexes != sorted(ordered_indexes):
            raise AuditError(f"global event indexes are not ordered at tick {tick}")
        global_events[tick] = events
    if set(global_events) - set(tick_records):
        raise AuditError("global resolver witness has no matching world tick")
    return tick_records, global_events


def _validate_starts_and_closures(
    groups: dict[str, list[dict[str, Any]]],
) -> tuple[list[dict[str, Any]], dict[tuple[int, int], dict[str, Any]]]:
    starts = groups.get("episode_start", [])
    start_by_key: dict[tuple[int, int], dict[str, Any]] = {}
    start_bytes: dict[bytes, tuple[int, int]] = {}
    for start in starts:
        owner = start.get("owner_id")
        tick = start.get("started_tick")
        if not _integer(owner) or not _integer(tick):
            raise AuditError("episode start has invalid owner/tick provenance")
        key = (owner, tick)
        encoded = _start_identity(start)
        if key in start_by_key or encoded in start_bytes:
            raise AuditError(f"duplicate episode start provenance: {key}")
        start_by_key[key] = start
        start_bytes[encoded] = key

    closure_by_key: dict[tuple[int, int], dict[str, Any]] = {}
    for closure in groups.get("episode_closure", []):
        start = closure.get("start")
        if not isinstance(start, dict):
            raise AuditError("closure is missing its start provenance")
        key = start_bytes.get(_start_identity(start))
        if key is None or key in closure_by_key:
            raise AuditError("closure has unmatched or duplicate start provenance")
        ended_tick = closure.get("ended_tick")
        if not _integer(ended_tick) or ended_tick < key[1]:
            raise AuditError(f"invalid closure tick for start {key}")
        closure_by_key[key] = closure
    if set(closure_by_key) != set(start_by_key):
        raise AuditError("start and closure provenance sets do not reconcile")
    return starts, closure_by_key


def _features(start: dict[str, Any]) -> tuple[dict[str, Any], str | None]:
    """Extract only choice-time fields; never serialize candidate alternatives."""
    body = start.get("body")
    position = start.get("start_position")
    routes = start.get("routes")
    selected_index = start.get("selected_route_index")
    if (
        not isinstance(body, dict)
        or not isinstance(position, dict)
        or not isinstance(routes, list)
        or not _integer(selected_index)
        or selected_index < 0
        or selected_index >= len(routes)
    ):
        return {}, "missing_or_invalid_choice_time_route"
    route = routes[selected_index]
    if not isinstance(route, dict):
        return {}, "malformed_selected_route"
    intent = start.get("selected_intent")
    intent_body = intent.get("Move") if isinstance(intent, dict) else None
    prefix = intent_body.get("steps") if isinstance(intent_body, dict) else None
    threat = route.get("selected_threat")
    if not isinstance(threat, dict):
        threat = {}
    if prefix is None:
        prefix_error = "missing_choice_time_prefix"
    elif not isinstance(prefix, list):
        return {}, "malformed_choice_time_prefix"
    else:
        prefix_error = None
    allowed_body = (
        "health", "max_health", "energy", "max_energy", "stamina",
        "max_stamina", "speed", "move_energy_cost", "vision_range",
    )
    body_features = {key: body.get(key) for key in allowed_body if key in body}
    required_body = ("health", "max_health", "energy", "max_energy", "stamina", "max_stamina", "speed", "move_energy_cost", "vision_range")
    if any(not _integer(body.get(key)) for key in required_body):
        feature_error = "missing_or_invalid_choice_time_body_fields"
    elif any(not _integer(position.get(key)) for key in ("x", "y")):
        feature_error = "missing_or_invalid_start_position"
    elif prefix_error:
        feature_error = prefix_error
    elif any(key not in route for key in ("endpoint", "endpoint_kind", "path", "path_steps", "turns", "observed_route_cells")):
        feature_error = "missing_selected_route_features"
    elif any(key not in threat for key in ("species", "observed_health", "observed_power", "observed_steps", "static_contact_geometry")):
        feature_error = "missing_selected_threat_features"
    else:
        feature_error = None
    selected = {
        "endpoint": route.get("endpoint"),
        "endpoint_kind": route.get("endpoint_kind"),
        "path": route.get("path"),
        "path_steps": route.get("path_steps"),
        "turns": route.get("turns"),
        "observed_route_cells": route.get("observed_route_cells"),
        "threat_species": threat.get("species"),
        "threat_observed_health": threat.get("observed_health"),
        "threat_observed_power": threat.get("observed_power"),
        "threat_observed_steps": threat.get("observed_steps"),
        "static_contact_geometry": threat.get("static_contact_geometry"),
    }
    features = {
        "decision_reason": start.get("decision_reason"),
        "body": body_features,
        "start_position": {k: position.get(k) for k in ("x", "y")},
        "selected_intent_prefix": prefix,
        "selected_route": selected,
    }
    return features, feature_error


def _selected_threat_identity_ok(start: dict[str, Any]) -> bool:
    selected_id = start.get("threat_id")
    routes = start.get("routes")
    index = start.get("selected_route_index")
    if not _integer(selected_id) or not isinstance(routes, list) or not _integer(index) or not (0 <= index < len(routes)):
        return False
    route = routes[index]
    threat = route.get("selected_threat") if isinstance(route, dict) else None
    route_threat_id = threat.get("id") if isinstance(threat, dict) else None
    return _integer(route_threat_id) and route_threat_id == selected_id


def _unknown_row(
    start: dict[str, Any],
    closure: dict[str, Any],
    raw_sha256: str,
    features: dict[str, Any],
    feature_error: str | None,
    reason: str,
    first_tick: int | None = None,
    exposure_intervals: int | None = None,
) -> dict[str, Any]:
    owner = start["owner_id"]
    started_tick = start["started_tick"]
    return {
        "provenance": {
            "raw_sha256": raw_sha256,
            "owner_id": owner,
            "start_tick": started_tick,
            "resolver_tick": first_tick,
            "selected_threat_id": start.get("threat_id"),
            "start_record_sha256": hashlib.sha256(_start_identity(start)).hexdigest(),
            "closure": closure.get("closure"),
        },
        "features": features,
        "feature_status": "complete" if feature_error is None else "unknown",
        "feature_reason": feature_error,
        "outcome": {
            "status": "Unknown",
            "reason": reason,
            "selected_threat_damage": None,
            "other_damage": None,
            "unattributed_health_loss": None,
            "exposure_resolver_intervals": exposure_intervals,
            "selected_threat_attack_events": None,
            "other_attack_events": None,
        },
    }


def audit_records(records: list[dict[str, Any]], raw_sha256: str = "fixture") -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Return one Exact/Unknown row per start plus descriptive support metadata."""
    groups = _record_index(records)
    tick_records, global_events = _validate_header_and_global(groups)
    starts, closure_by_key = _validate_starts_and_closures(groups)
    summary_record = groups["summary"][0]
    if (
        summary_record.get("started_attempts") != len(starts)
        or summary_record.get("closed_attempts") != len(closure_by_key)
        or summary_record.get("completed_world_ticks") != len(tick_records)
    ):
        raise AuditError("global summary counts do not match raw episode/tick records")
    start_lookup = {(row["owner_id"], row["started_tick"]): row for row in starts}
    ticks_by_owner: dict[int, list[dict[str, Any]]] = defaultdict(list)
    owner_tick_seen: set[tuple[int, int]] = set()
    for row in groups.get("episode_tick", []):
        owner, tick = row.get("owner_id"), row.get("tick")
        if not _integer(owner) or not _integer(tick):
            raise AuditError("episode tick has invalid owner/tick provenance")
        if (owner, tick) in owner_tick_seen:
            raise AuditError(f"duplicate episode interval for owner {owner}, tick {tick}")
        owner_tick_seen.add((owner, tick))
        ticks_by_owner[owner].append(row)
    intervals_by_start: dict[tuple[int, int], list[dict[str, Any]]] = defaultdict(list)
    for owner, rows in ticks_by_owner.items():
        for row in rows:
            elapsed = row.get("elapsed_ticks")
            tick = row["tick"]
            if not _integer(elapsed) or elapsed < 1:
                raise AuditError(f"invalid elapsed interval for owner {owner}, tick {tick}")
            start_tick = tick - elapsed + 1
            key = (owner, start_tick)
            if key not in closure_by_key:
                raise AuditError(f"episode interval does not map to a known start: {key}")
            start_record = start_lookup[key]
            intervals_by_start[key].append(row)
    for rows in intervals_by_start.values():
        rows.sort(key=lambda row: row["tick"])

    output: list[dict[str, Any]] = []
    unknown_reasons: Counter[str] = Counter()
    owner_counts: Counter[int] = Counter()
    owner_positive: Counter[int] = Counter()
    support: Counter[int] = Counter()
    continuation_rows = 0
    for start in starts:
        owner = start["owner_id"]
        start_tick = start["started_tick"]
        key = (owner, start_tick)
        closure = closure_by_key[key]
        owner_counts[owner] += 1
        candidates = intervals_by_start.get(key, [])
        features, feature_error = _features(start)
        continuation_rows += sum(
            not (row["tick"] == start_tick and row["elapsed_ticks"] == 1)
            for row in candidates
        )
        first_candidates = [
            row for row in candidates
            if row["tick"] == start_tick and row["elapsed_ticks"] == 1
        ]
        if not first_candidates:
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "missing_first_interval")
            unknown_reasons["missing_first_interval"] += 1
            output.append(row)
            continue
        if len(first_candidates) != 1:
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "ambiguous_first_interval", start_tick)
            unknown_reasons["ambiguous_first_interval"] += 1
            output.append(row)
            continue
        interval = first_candidates[0]
        first_tick = start_tick
        exposure = 1
        if not _selected_threat_identity_ok(start) or interval.get("threat_id") != start.get("threat_id"):
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "selected_threat_identity_missing_or_mismatch", first_tick, exposure)
            unknown_reasons["selected_threat_identity_missing_or_mismatch"] += 1
            output.append(row)
            continue

        global_row = tick_records.get(first_tick)
        events = global_events.get(first_tick)
        if global_row is None or events is None:
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "missing_global_resolver_witness", first_tick, exposure)
            unknown_reasons["missing_global_resolver_witness"] += 1
            output.append(row)
            continue
        owner_attacks = [
            event for event in events
            if event.get("kind") == "attacked" and event.get("target") == owner
        ]
        local_attacks = interval.get("attack_events")
        if not isinstance(local_attacks, list):
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "missing_local_attack_witness", first_tick, exposure)
            unknown_reasons["missing_local_attack_witness"] += 1
            output.append(row)
            continue
        if _normalized_attacks(owner_attacks, local=False) != _normalized_attacks(local_attacks, local=True):
            raise AuditError(f"local/global attack witness mismatch for owner {owner} at tick {first_tick}")
        selected_id = start.get("threat_id")
        selected_attacks = [event for event in owner_attacks if event.get("attacker") == selected_id]
        other_attacks = [event for event in owner_attacks if event.get("attacker") != selected_id]
        selected_damage = sum(event.get("damage", 0) for event in selected_attacks)
        other_damage = sum(event.get("damage", 0) for event in other_attacks)
        if interval.get("cumulative_threat_damage") != selected_damage:
            raise AuditError(f"selected-threat damage mismatch for owner {owner} at tick {first_tick}")
        if interval.get("cumulative_other_damage") != other_damage:
            raise AuditError(f"other damage mismatch for owner {owner} at tick {first_tick}")
        unexplained = interval.get("cumulative_unattributed_health_loss")
        health_before = interval.get("health_before_resolution")
        health_after = interval.get("health_after_resolution")
        start_health = start.get("body", {}).get("health") if isinstance(start.get("body"), dict) else None
        if not all(_integer(value) for value in (unexplained, health_before, health_after, start_health)):
            row = _unknown_row(start, closure, raw_sha256, features, feature_error, "invalid_unattributed_loss_witness", first_tick, exposure)
            unknown_reasons["invalid_unattributed_loss_witness"] += 1
            output.append(row)
            continue
        if unexplained < 0 or health_before != start_health:
            raise AuditError(f"first-interval health provenance mismatch for owner {owner}, tick {first_tick}")
        net_unattributed = max(0, health_before - health_after - selected_damage - other_damage)
        if unexplained != net_unattributed:
            raise AuditError(f"unattributed health loss mismatch for owner {owner}, tick {first_tick}")

        damage = selected_damage
        support[damage] += 1
        if damage > 0:
            owner_positive[owner] += 1
        output.append({
            "provenance": {
                "raw_sha256": raw_sha256,
                "owner_id": owner,
                "start_tick": start_tick,
                "resolver_tick": first_tick,
                "selected_threat_id": selected_id,
                "start_record_sha256": hashlib.sha256(_start_identity(start)).hexdigest(),
                "closure": closure.get("closure"),
            },
            "features": features,
            "feature_status": "complete" if feature_error is None else "unknown",
            "feature_reason": feature_error,
            "outcome": {
                "status": "Exact",
                "reason": None,
                "selected_threat_damage": damage,
                "other_damage": other_damage,
                "unattributed_health_loss": unexplained,
                "exposure_resolver_intervals": exposure,
                "selected_threat_attack_events": len(selected_attacks),
                "other_attack_events": len(other_attacks),
            },
        })

    if len(output) != len(starts):
        raise AuditError("extractor failed to preserve one row per start")
    summary = {
        "schema": "refuge_first_resolution_audit_v1",
        "estimand": "selected-threat resolver-attributed damage during first complete resolver interval after each observed shelter start",
        "raw_sha256": raw_sha256,
        "starts": len(starts),
        "closures": len(closure_by_key),
        "episode_tick_rows": len(groups.get("episode_tick", [])),
        "first_resolution_rows": len(output),
        "later_continuation_rows_excluded": continuation_rows,
        "exact_rows": sum(row["outcome"]["status"] == "Exact" for row in output),
        "unknown_rows": sum(row["outcome"]["status"] == "Unknown" for row in output),
        "unknown_reasons": dict(sorted(unknown_reasons.items())),
        "damage_support": {str(k): support[k] for k in sorted(support)},
        "positive_damage_total": sum(k * count for k, count in support.items()),
        "owners_with_starts": len(owner_counts),
        "owners_with_positive_damage": len(owner_positive),
        "per_owner_counts_metadata_only": {
            str(owner): {
                "episode_starts": owner_counts[owner],
                "exact_first_intervals": sum(
                    row["provenance"]["owner_id"] == owner and row["outcome"]["status"] == "Exact"
                    for row in output
                ),
                "positive_first_intervals": owner_positive[owner],
            }
            for owner in sorted(owner_counts)
        },
        "model_fitted": False,
        "model_state_written": False,
        "world_ticks_run": 0,
        "route_policy_or_biology_changed": False,
    }
    return output, summary


def _git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args], check=False, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    if result.returncode:
        raise AuditError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def _pinned_inputs() -> dict[str, str]:
    expected = {
        "raw": (RAW, RAW_SHA256),
        "source_manifest": (SOURCE_MANIFEST, SOURCE_MANIFEST_SHA256),
        "independent_audit": (REFERENCE_AUDIT, REFERENCE_AUDIT_SHA256),
    }
    actual: dict[str, str] = {}
    for name, (path, expected_hash) in expected.items():
        if not path.is_file() or path.is_symlink():
            raise AuditError(f"pinned {name} input is missing or a symlink")
        value = sha256_file(path)
        if value != expected_hash:
            raise AuditError(f"pinned {name} input SHA-256 mismatch")
        actual[name] = value
    manifest = json.loads(SOURCE_MANIFEST.read_text(encoding="utf-8"))
    if (
        manifest.get("experiment_source_commit") != SOURCE_EXPERIMENT_COMMIT
        or manifest.get("engine_source_commit") != SOURCE_ENGINE_COMMIT
        or manifest.get("scenario", {}).get("sha256") != SCENARIO_SHA256
        or manifest.get("outputs_after_run", {}).get("raw", {}).get("sha256") != RAW_SHA256
    ):
        raise AuditError("source manifest identity does not match pinned raw capture")
    audit = json.loads(REFERENCE_AUDIT.read_text(encoding="utf-8"))
    if audit.get("status") != "passed" or audit.get("raw_sha256") != RAW_SHA256:
        raise AuditError("independent audit is not passed for the pinned raw capture")
    return actual


def _output_path(value: str) -> Path:
    base = OUTPUT_ROOT
    if base.exists() and base.is_symlink():
        raise AuditError("output root must not be a symlink")
    base_resolved = base.resolve()
    requested = Path(value)
    if not requested.is_absolute():
        requested = ROOT / requested
    if requested.is_symlink():
        raise AuditError("output directory must not be a symlink")
    if requested.resolve().parent != base_resolved or requested.resolve() == base_resolved:
        raise AuditError("output must be a new direct child of the fixed output root")
    if requested.exists():
        raise AuditError("refusing to overwrite an existing output directory")
    return requested


def run(output_value: str, expected_commit: str) -> Path:
    output = _output_path(output_value)
    if not expected_commit or _git(ROOT, "rev-parse", "HEAD") != expected_commit:
        raise AuditError("HEAD does not match required reviewed freeze commit")
    if _git(ROOT, "status", "--porcelain=v1", "--untracked-files=all"):
        raise AuditError("worktree must be clean before frozen extraction")
    before = _pinned_inputs()
    code_before = sha256_file(Path(__file__).resolve())
    protocol_before = sha256_file(PROTOCOL)
    records = read_jsonl(RAW)
    rows, summary = audit_records(records, before["raw"])
    after_parse = _pinned_inputs()
    code_after = sha256_file(Path(__file__).resolve())
    protocol_after = sha256_file(PROTOCOL)
    if before != after_parse or code_before != code_after or protocol_before != protocol_after:
        raise AuditError("pinned source/protocol/code/input changed during extraction")
    if summary["starts"] != 623 or summary["closures"] != 623:
        raise AuditError("pinned start/closure support differs from reviewed audit")
    if summary["first_resolution_rows"] != 623 or summary["exact_rows"] + summary["unknown_rows"] != 623:
        raise AuditError("extractor did not preserve all pinned episode starts")
    if (
        summary["episode_tick_rows"] != 631
        or summary["later_continuation_rows_excluded"] != 8
        or summary["damage_support"] != {"0": 617, "20": 6}
        or summary["positive_damage_total"] != 120
    ):
        raise AuditError("pinned first-interval support differs from reviewed read-only audit")

    base = OUTPUT_ROOT
    base.mkdir(parents=True, exist_ok=True)
    if base.is_symlink():
        raise AuditError("output root became a symlink")
    if output.exists():
        raise AuditError("output directory appeared during preflight")
    stage = Path(tempfile.mkdtemp(prefix=".refuge-first-resolution-", dir=base))
    try:
        rows_path = stage / "rows.jsonl"
        with rows_path.open("xb") as stream:
            for row in rows:
                stream.write(canonical(row) + b"\n")
            stream.flush()
            os.fsync(stream.fileno())
        summary_path = stage / "summary.json"
        with summary_path.open("xb") as stream:
            stream.write((json.dumps(summary, indent=2, sort_keys=True) + "\n").encode())
            stream.flush()
            os.fsync(stream.fileno())
        output_size = rows_path.stat().st_size + summary_path.stat().st_size
        if output_size > MAX_OUTPUT_BYTES:
            raise AuditError(f"output exceeds {MAX_OUTPUT_BYTES} bytes")
        provenance = {
            "schema": "refuge_first_resolution_run_v1",
            "status": "extraction_complete_no_model",
            "reviewed_commit": expected_commit,
            "source_experiment_commit": SOURCE_EXPERIMENT_COMMIT,
            "source_engine_commit": SOURCE_ENGINE_COMMIT,
            "protocol_path": str(PROTOCOL.relative_to(ROOT)),
            "protocol_sha256_before": protocol_before,
            "protocol_sha256_after": protocol_after,
            "code_path": str(Path(__file__).resolve().relative_to(ROOT)),
            "code_sha256_before": code_before,
            "code_sha256_after": code_after,
            "inputs_before": before,
            "inputs_after": after_parse,
            "output_sha256_before": None,
            "outputs_after": {
                "rows.jsonl": {"bytes": rows_path.stat().st_size, "sha256": sha256_file(rows_path)},
                "summary.json": {"bytes": summary_path.stat().st_size, "sha256": sha256_file(summary_path)},
            },
            "model_fitted": False,
            "model_state_written": False,
            "world_ticks_run": 0,
        }
        final_inputs = _pinned_inputs()
        final_code = sha256_file(Path(__file__).resolve())
        final_protocol = sha256_file(PROTOCOL)
        if final_inputs != before or final_code != code_before or final_protocol != protocol_before:
            raise AuditError("pinned source/protocol/code/input changed before output publish")
        provenance["inputs_after"] = final_inputs
        provenance["code_sha256_after"] = final_code
        provenance["protocol_sha256_after"] = final_protocol
        run_path = stage / "provenance.json"
        with run_path.open("xb") as stream:
            stream.write((json.dumps(provenance, indent=2, sort_keys=True) + "\n").encode())
            stream.flush()
            os.fsync(stream.fileno())
        final_inputs = _pinned_inputs()
        if (
            final_inputs != before
            or sha256_file(Path(__file__).resolve()) != code_before
            or sha256_file(PROTOCOL) != protocol_before
        ):
            raise AuditError("pinned source/protocol/code/input changed before final publish")
        if output.exists():
            raise AuditError("output directory appeared before finalization")
        total_output = sum(path.stat().st_size for path in stage.iterdir() if path.is_file())
        if total_output > MAX_OUTPUT_BYTES:
            raise AuditError(f"output exceeds {MAX_OUTPUT_BYTES} bytes")
        os.replace(stage, output)
        return output
    except Exception:
        if stage.exists():
            import shutil
            shutil.rmtree(stage)
        raise


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True, help="new direct child of the fixed output root")
    parser.add_argument("--expected-commit", required=True, help="reviewed clean commit containing this code and protocol")
    args = parser.parse_args(argv)
    try:
        output = run(args.output_dir, args.expected_commit)
    except (AuditError, OSError, json.JSONDecodeError) as error:
        print(f"refuge first-resolution extraction refused: {error}", file=sys.stderr)
        return 2
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
