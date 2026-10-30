#!/usr/bin/env python3
"""Bounded read-only audit of refuge candidate keys versus completed rows."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, BinaryIO, Iterator


MAX_INPUT_BYTES = 128 * 1024 * 1024
MAX_LINE_BYTES = 16 * 1024 * 1024
MAX_COMPARISONS = 100_000
REFUGE_ROW_TTL_TICKS = 100


class DiagnosisError(Exception):
    pass


def sha256_file(path: Path, limit: int) -> tuple[str, int]:
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as source:
        while block := source.read(1024 * 1024):
            size += len(block)
            if size > limit:
                raise DiagnosisError(f"compressed input exceeds {limit} bytes")
            digest.update(block)
    return digest.hexdigest(), size


def open_input(path: Path, compressed_limit: int) -> tuple[BinaryIO, subprocess.Popen[bytes] | None, str | None, int | None]:
    if path.suffix != ".zst":
        return path.open("rb"), None, None, None
    archive_hash, archive_bytes = sha256_file(path, compressed_limit)
    try:
        process = subprocess.Popen(
            ["zstd", "-q", "-d", "-c", "--", str(path)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except OSError as exc:
        raise DiagnosisError(f"could not start zstd: {exc}") from exc
    assert process.stdout is not None
    return process.stdout, process, archive_hash, archive_bytes


def records(
    path: Path,
    max_input_bytes: int,
    max_line_bytes: int,
    input_meta: dict[str, Any],
) -> Iterator[dict[str, Any]]:
    stream, process, archive_hash, archive_bytes = open_input(path, max_input_bytes)
    raw_hash = hashlib.sha256()
    raw_bytes = 0
    record_number = 0
    finished = False
    try:
        while True:
            line = stream.readline(max_line_bytes + 1)
            if not line:
                break
            if len(line) > max_line_bytes:
                raise DiagnosisError(f"JSONL line exceeds {max_line_bytes} bytes")
            raw_bytes += len(line)
            if raw_bytes > max_input_bytes:
                raise DiagnosisError(f"decompressed input exceeds {max_input_bytes} bytes")
            raw_hash.update(line)
            if not line.endswith(b"\n"):
                raise DiagnosisError("input ends with an incomplete JSONL record")
            try:
                item = json.loads(line)
            except (UnicodeDecodeError, json.JSONDecodeError) as exc:
                raise DiagnosisError(f"invalid JSONL record {record_number + 1}: {exc}") from exc
            if not isinstance(item, dict):
                raise DiagnosisError(f"JSONL record {record_number + 1} is not an object")
            record_number += 1
            yield item
        finished = True
    finally:
        stream.close()
        if process is not None:
            if not finished and process.poll() is None:
                process.kill()
            stderr = process.stderr.read() if process.stderr is not None else b""
            returncode = process.wait()
            if finished and returncode:
                raise DiagnosisError(f"zstd exited {returncode}: {stderr[:500].decode(errors='replace')}")
    input_meta.update({
        "input_path": str(path),
        "raw_sha256": raw_hash.hexdigest(),
        "raw_bytes": raw_bytes,
        "archive_sha256": archive_hash,
        "archive_bytes": archive_bytes,
    })
    index_path = path.with_name(path.name[:-4] + ".archive.json") if path.suffix == ".zst" else None
    if index_path is not None and index_path.exists():
        try:
            index = json.loads(index_path.read_text())
        except (OSError, json.JSONDecodeError) as exc:
            raise DiagnosisError(f"invalid archive index {index_path}: {exc}") from exc
        input_meta["archive_index_path"] = str(index_path)
        input_meta["archive_index_status"] = index.get("status")
        if index.get("status") != "verified":
            raise DiagnosisError("archive index does not say verified")
        if index.get("original_sha256") != input_meta["raw_sha256"]:
            raise DiagnosisError("decompressed raw SHA-256 disagrees with archive index")
        if index.get("archive_sha256") != archive_hash:
            raise DiagnosisError("archive SHA-256 disagrees with archive index")
        if index.get("original_size") != raw_bytes or index.get("archive_size") != archive_bytes:
            raise DiagnosisError("archive sizes disagree with archive index")
def key_differences(left: Any, right: Any) -> list[str]:
    if not isinstance(left, dict) or not isinstance(right, dict):
        return ["<key-shape>"] if left != right else []
    return sorted(
        field
        for field in set(left) | set(right)
        if field not in left or field not in right or left[field] != right[field]
    )


def completed_samples(record: dict[str, Any]) -> list[dict[str, Any]]:
    samples: list[dict[str, Any]] = []
    events = record.get("focal_events")
    if not isinstance(events, list):
        raise DiagnosisError("tick record lacks a focal event list")
    for event in events:
        if not isinstance(event, dict):
            raise DiagnosisError("focal event list contains a non-object event")
        closure = event.get("RefugeEpisodeEnded")
        if closure is None:
            continue
        if not isinstance(closure, dict):
            raise DiagnosisError("RefugeEpisodeEnded event is not an object")
        if "generalization_mode" not in closure and (closure.get("completed") or "generalized_context" in closure):
            raise DiagnosisError("episode closure lacks its generalization mode/provenance discriminator")
        if closure.get("generalization_mode") != "Generalized":
            continue
        key = closure.get("generalized_context")
        if not closure.get("completed"):
            continue
        required = ("closure", "generalized_context", "threat_id", "ended_tick", "attribution_known")
        missing = [field for field in required if field not in closure]
        if missing:
            raise DiagnosisError(f"completed generalized closure lacks fields: {missing}")
        if closure.get("closure") != "Arrived" or not closure.get("attribution_known"):
            continue
        if (
            not isinstance(key, dict)
            or not isinstance(closure.get("threat_id"), int)
            or not isinstance(closure.get("ended_tick"), int)
        ):
            raise DiagnosisError("completed generalized closure has invalid key/source/tick fields")
        samples.append(
            {
                "key": key,
                "source_threat_id": closure.get("threat_id"),
                "completion_tick": closure.get("ended_tick"),
                "attribution_known": bool(closure.get("attribution_known")),
                "closure": closure.get("closure"),
                "arm": record.get("arm"),
            }
        )
    return samples


def row_source_ids(row: dict[str, Any], samples: list[dict[str, Any]]) -> tuple[list[int], bool, int]:
    matches = [
        sample
        for sample in samples
        if sample["key"] == row.get("key")
        and sample["completion_tick"] <= row.get("last_completed_tick", -1)
        and sample["attribution_known"]
    ]
    ids = sorted({sample["source_threat_id"] for sample in matches if isinstance(sample["source_threat_id"], int)})
    unknown = not matches or any(not isinstance(sample["source_threat_id"], int) for sample in matches)
    return ids, unknown, len(matches)


def analyze_choice(
    record: dict[str, Any],
    samples: list[dict[str, Any]],
    counters: dict[str, int],
    detail_limit: int,
) -> dict[str, Any] | None:
    choice = record.get("production_choice")
    if not isinstance(choice, dict) or not isinstance(choice.get("is_fresh"), bool):
        raise DiagnosisError("tick record lacks a production-choice witness")
    if not choice["is_fresh"]:
        return None
    tick = choice.get("choice_tick", record.get("world_tick"))
    threat_id = choice.get("threat_id")
    focal_before = record.get("focal_before", {})
    life = focal_before.get("life", {}) if isinstance(focal_before, dict) else {}
    rows = focal_before.get("generalized_outcomes", []) if isinstance(focal_before, dict) else []
    if not isinstance(life, dict) or not isinstance(life.get("age"), int) or not isinstance(life.get("max_age"), int):
        raise DiagnosisError("fresh choice lacks recorded focal age/max-age")
    if not isinstance(rows, list):
        raise DiagnosisError("fresh choice has invalid generalized-outcomes table")
    candidates = choice.get("candidates", [])
    if not isinstance(candidates, list):
        candidates = []
    output_candidates: list[dict[str, Any]] = []
    witness_mismatches: list[dict[str, Any]] = []
    candidate_row_matches: list[set[int]] = []
    all_different_id_supported = bool(candidates)
    any_different_id_exact_candidate = False
    all_supported = bool(candidates)

    for candidate_index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict) or not isinstance(candidate.get("key"), dict):
            raise DiagnosisError("fresh-choice candidate lacks a structured represented key")
        for field in ("support_completed", "support_last_completed_tick", "supported_unique_fresh", "different_threat_id_supported", "support_sources"):
            if field not in candidate:
                raise DiagnosisError(f"candidate witness lacks required field {field!r}")
        if not isinstance(candidate["support_sources"], list) or not isinstance(candidate["supported_unique_fresh"], bool) or not isinstance(candidate["different_threat_id_supported"], bool):
            raise DiagnosisError("candidate witness has invalid support/provenance field types")
        key = candidate.get("key") if isinstance(candidate, dict) else None
        comparisons: list[dict[str, Any]] = []
        exact_matches: list[int] = []
        fresh_matches: list[int] = []
        for row_index, row in enumerate(rows):
            if counters["comparisons"] >= detail_limit:
                raise DiagnosisError(f"comparison detail budget {detail_limit} exceeded")
            if not isinstance(row, dict):
                raise DiagnosisError("recorded generalized-outcomes table contains a non-object row")
            counters["comparisons"] += 1
            if not isinstance(row.get("key"), dict) or not isinstance(row.get("completed"), int) or not isinstance(row.get("last_completed_tick"), int):
                raise DiagnosisError("recorded generalized row has invalid key/count/tick fields")
            row_tick = row.get("last_completed_tick")
            age = tick - row_tick if isinstance(tick, int) and isinstance(row_tick, int) else None
            ttl_fresh = age is not None and 0 <= age <= REFUGE_ROW_TTL_TICKS
            equal = key == row.get("key")
            source_ids, source_unknown, source_event_count = row_source_ids(row, samples)
            comparison = {
                "row_index": row_index,
                "exact_key_match": equal,
                "mismatch_fields": key_differences(key, row.get("key")),
                "completed": row.get("completed"),
                "last_completed_tick": row_tick,
                "row_age_ticks": age,
                "ttl_fresh": ttl_fresh,
                "source_threat_ids": source_ids,
                "matching_completed_event_count": source_event_count,
                "row_sample_count_matches_events": row.get("completed") == source_event_count,
                "source_threat_id_unknown": source_unknown,
                "source_differs_from_current_threat": (
                    bool(source_ids) and not source_unknown and isinstance(threat_id, int) and threat_id not in source_ids
                ),
            }
            comparisons.append(comparison)
            if equal:
                exact_matches.append(row_index)
                if ttl_fresh and row["completed"] > 0:
                    fresh_matches.append(row_index)
        unique_exact = len(exact_matches) == 1
        unique_fresh = len(fresh_matches) == 1
        actual_source_ids = candidate["support_sources"]
        if any(not isinstance(source_id, int) for source_id in actual_source_ids):
            raise DiagnosisError("candidate witness contains a non-integer support source ID")
        event_source_ids = sorted({sid for comp in comparisons if comp["exact_key_match"] for sid in comp["source_threat_ids"]})
        event_source_unknown = not any(comp["exact_key_match"] and not comp["source_threat_id_unknown"] for comp in comparisons)
        diff_id = isinstance(threat_id, int) and any(source_id != threat_id for source_id in actual_source_ids)
        candidate_row_matches.append(set(fresh_matches))
        all_supported = all_supported and unique_fresh
        all_different_id_supported = all_different_id_supported and unique_fresh and diff_id
        any_different_id_exact_candidate = any_different_id_exact_candidate or diff_id
        actual_completed = candidate["support_completed"]
        if not isinstance(actual_completed, int):
            raise DiagnosisError("candidate witness support_completed is not an integer")
        calculated_completed = rows[exact_matches[0]].get("completed", 0) if unique_exact else 0
        actual_last_tick = candidate["support_last_completed_tick"]
        calculated_last_tick = rows[exact_matches[0]].get("last_completed_tick") if unique_exact else None
        calculated_diff_id = isinstance(threat_id, int) and any(source_id != threat_id for source_id in actual_source_ids)
        if unique_fresh and diff_id:
            counters["candidate_exact_different_id_supports"] += 1
        candidate_output = {
                "candidate_index": candidate_index,
                "actual_support_completed": actual_completed,
                "calculated_support_completed": calculated_completed,
                "support_count_matches": actual_completed == calculated_completed,
                "actual_support_last_completed_tick": actual_last_tick,
                "calculated_support_last_completed_tick": calculated_last_tick,
                "support_tick_matches": actual_last_tick == calculated_last_tick,
                "actual_supported_unique_fresh": candidate["supported_unique_fresh"],
                "calculated_supported_unique_fresh": unique_fresh,
                "supported_unique_fresh_matches": candidate["supported_unique_fresh"] == unique_fresh,
                "actual_different_threat_id_supported": candidate["different_threat_id_supported"],
                "calculated_different_threat_id_supported": calculated_diff_id,
                "different_id_witness_matches": candidate["different_threat_id_supported"] == calculated_diff_id,
                "actual_support_source_ids": sorted(set(actual_source_ids)),
                "event_support_source_ids_for_exact_key": event_source_ids,
                "event_support_provenance_unknown": event_source_unknown,
                "fresh_matching_row_indices": fresh_matches,
                "comparisons": comparisons,
            }
        mismatched_fields = [
            field
            for field, agrees in (
                ("support_completed", candidate_output["support_count_matches"]),
                ("support_last_completed_tick", candidate_output["support_tick_matches"]),
                ("supported_unique_fresh", candidate_output["supported_unique_fresh_matches"]),
                ("different_threat_id_supported", candidate_output["different_id_witness_matches"]),
            )
            if not agrees
        ]
        if mismatched_fields:
            witness_mismatches.append({"candidate_index": candidate_index, "fields": mismatched_fields})
        output_candidates.append(candidate_output)

    # Different candidates cannot reuse one row in a complete ranking.
    used: set[int] = set()
    for matched in candidate_row_matches:
        if len(matched) != 1 or used.intersection(matched):
            all_supported = False
        used.update(matched)
    if not all_supported:
        all_different_id_supported = False
    scope = record.get("scope")
    selection_kind = choice.get("selection_kind")
    recorded_supported = selection_kind == "generalized_supported_greedy"
    trained_online = record.get("arm") == "trained_online"
    if scope == "suffix" and trained_online and recorded_supported:
        counters["recorded_postprefix_supported_fresh_greedy_choices"] += 1
    if scope == "suffix" and trained_online and all_supported:
        counters["calculated_postprefix_all_candidates_supported_choices"] += 1
    if scope == "suffix" and trained_online and all_supported and all_different_id_supported:
        counters["calculated_postprefix_different_id_supported_choices"] += 1
    if witness_mismatches:
        counters["candidate_witness_mismatch_choices"] += 1
        counters["candidate_witness_mismatch_fields"] += sum(len(item["fields"]) for item in witness_mismatches)
    return {
        "scope": scope,
        "arm": record.get("arm"),
        "world_tick": record.get("world_tick"),
        "choice_tick": tick,
        "focal_age": life.get("age") if isinstance(life, dict) else None,
        "focal_max_age": life.get("max_age") if isinstance(life, dict) else None,
        "focal_age_margin": (
            life.get("max_age") - life.get("age")
            if isinstance(life, dict) and isinstance(life.get("max_age"), int) and isinstance(life.get("age"), int)
            else None
        ),
        "current_threat_id": threat_id,
        "selection_kind": selection_kind,
        "candidate_count": len(candidates),
        "all_candidates_calculated_supported_unique_fresh": all_supported,
        "all_candidates_calculated_supported_by_different_ids": all_different_id_supported,
        "any_candidate_with_exact_different_id_support": any_different_id_exact_candidate,
        "candidates": output_candidates,
        "candidate_witness_mismatches": witness_mismatches,
    }


def diagnose(items: Iterator[dict[str, Any]], input_meta: dict[str, Any], comparison_limit: int) -> dict[str, Any]:
    header: dict[str, Any] = {}
    raw_summary: dict[str, Any] | None = None
    prefix_samples: list[dict[str, Any]] = []
    branch_samples: dict[str, list[dict[str, Any]]] = {}
    branch_terminals: dict[str, dict[str, Any]] = {}
    treatment_arms: set[str] = set()
    details: list[dict[str, Any]] = []
    counters = {
        "comparisons": 0,
        "recorded_postprefix_supported_fresh_greedy_choices": 0,
        "calculated_postprefix_all_candidates_supported_choices": 0,
        "calculated_postprefix_different_id_supported_choices": 0,
        "candidate_exact_different_id_supports": 0,
        "candidate_witness_mismatch_choices": 0,
        "candidate_witness_mismatch_fields": 0,
    }
    counts: dict[str, int] = {}
    unique_threat_ids: set[int] = set()
    recorded_ages: dict[str, list[int]] = {"prefix": []}
    recorded_max_ages: dict[str, int] = {}
    branch_end_counts: dict[str, int] = {}
    for item in items:
        kind = item.get("record_type", "unknown")
        counts[kind] = counts.get(kind, 0) + 1
        if kind == "header":
            header = item
        elif kind == "prefix_tick":
            if item.get("scope") != "prefix":
                raise DiagnosisError(f"unexpected prefix scope {item.get('scope')!r}")
            life = item.get("focal_before", {}).get("life", {})
            if not isinstance(life, dict) or not isinstance(life.get("age"), int) or not isinstance(life.get("max_age"), int):
                raise DiagnosisError("prefix tick lacks recorded focal age/max-age")
            recorded_ages["prefix"].append(life["age"])
            recorded_max_ages["prefix"] = life["max_age"]
            detail = analyze_choice(item, prefix_samples, counters, comparison_limit)
            if detail is not None:
                details.append(detail)
                if isinstance(detail.get("current_threat_id"), int):
                    unique_threat_ids.add(detail["current_threat_id"])
            prefix_samples.extend(completed_samples(item))
        elif kind == "treatment_start":
            arm = item.get("arm")
            if isinstance(arm, str):
                treatment_arms.add(arm)
                branch_samples[arm] = copy.deepcopy(prefix_samples)
                recorded_ages[arm] = []
        elif kind == "suffix_tick":
            if item.get("scope") != "suffix":
                raise DiagnosisError(f"unexpected suffix scope {item.get('scope')!r}")
            arm = item.get("arm")
            if not isinstance(arm, str) or arm not in branch_samples:
                raise DiagnosisError(f"suffix record references uninitialized arm {arm!r}")
            life = item.get("focal_before", {}).get("life", {})
            if not isinstance(life, dict) or not isinstance(life.get("age"), int) or not isinstance(life.get("max_age"), int):
                raise DiagnosisError("suffix tick lacks recorded focal age/max-age")
            recorded_ages[arm].append(life["age"])
            recorded_max_ages[arm] = life["max_age"]
            detail = analyze_choice(item, branch_samples[arm], counters, comparison_limit)
            if detail is not None:
                details.append(detail)
                if isinstance(detail.get("current_threat_id"), int):
                    unique_threat_ids.add(detail["current_threat_id"])
            branch_samples[arm].extend(completed_samples(item))
        elif kind == "branch_end":
            arm = item.get("arm")
            terminal = item.get("focal_terminal_state", {})
            if isinstance(arm, str) and isinstance(terminal, dict):
                life = terminal.get("life", {})
                branch_terminals[arm] = {
                    "age": life.get("age") if isinstance(life, dict) else None,
                    "max_age": life.get("max_age") if isinstance(life, dict) else None,
                    "phase": terminal.get("phase"),
                }
                branch_end_counts[arm] = branch_end_counts.get(arm, 0) + 1
        elif kind == "summary":
            raw_summary = item

    if not header:
        raise DiagnosisError("missing header record")
    if raw_summary is None or counts.get("summary") != 1:
        raise DiagnosisError("complete input must contain exactly one summary record")
    required_summary = (
        "physical_ticks",
        "post_prefix_supported_fresh_greedy_choices",
        "different_threat_id_supported_choices",
        "aligned_online_frozen_greedy_contrasts",
    )
    if any(not isinstance(raw_summary.get(field), int) for field in required_summary):
        raise DiagnosisError("summary lacks required integer tick/opportunity counts")
    prefix_expected = header.get("prefix_ticks")
    suffix_expected = header.get("suffix_ticks_per_arm")
    arms_expected = 4
    if not isinstance(prefix_expected, int) or not isinstance(suffix_expected, int):
        raise DiagnosisError("header lacks valid prefix/suffix tick counts")
    if header.get("seed") != 123 or header.get("focal_id") != 15:
        raise DiagnosisError("input is not the frozen seen-seed 123/focal-15 engineering artifact")
    expected_arm_names = {"trained_online", "trained_frozen", "naive_online", "rule_based"}
    if treatment_arms != expected_arm_names or set(branch_end_counts) != expected_arm_names:
        raise DiagnosisError("treatment and branch-end arm names disagree with the frozen four-arm design")
    if counts.get("prefix_tick", 0) != prefix_expected or counts.get("suffix_tick", 0) != suffix_expected * arms_expected:
        raise DiagnosisError("tick-record counts do not match the fixed four-arm header plan")
    if counts.get("treatment_start", 0) != arms_expected or counts.get("branch_end", 0) != arms_expected:
        raise DiagnosisError("treatment-start/branch-end records do not match the four-arm design")
    if len(branch_end_counts) != arms_expected or any(count != 1 for count in branch_end_counts.values()):
        raise DiagnosisError("branch-end records are missing or duplicated by arm")
    if counts.get("prefix_tick", 0) + counts.get("suffix_tick", 0) != raw_summary["physical_ticks"]:
        raise DiagnosisError("summary physical tick count disagrees with the raw tick-record count")
    if header.get("max_physical_ticks") != raw_summary["physical_ticks"]:
        raise DiagnosisError("header physical-tick plan disagrees with summary")
    actual_supported = raw_summary["post_prefix_supported_fresh_greedy_choices"]
    actual_different_id = raw_summary["different_threat_id_supported_choices"]
    recorded_fresh_choices = counters["recorded_postprefix_supported_fresh_greedy_choices"]
    supported_reconciles = actual_supported == recorded_fresh_choices
    different_id_reconciles = actual_different_id == counters["calculated_postprefix_different_id_supported_choices"]
    witness_reconciles = counters["candidate_witness_mismatch_fields"] == 0
    return {
        "schema": "curious-refuge-opportunity-diagnosis-v1",
        "status": "witness_reconciled" if supported_reconciles and different_id_reconciles and witness_reconciles else "witness_mismatch_requires_root_review",
        "scope": "archived records only; no World ticks, policy decisions, or state mutation",
        "input": input_meta,
        "header": {
            "protocol": header.get("protocol"),
            "scenario": header.get("scenario"),
            "seed": header.get("seed"),
            "focal_id": header.get("focal_id"),
            "prefix_ticks": header.get("prefix_ticks"),
            "suffix_ticks_per_arm": header.get("suffix_ticks_per_arm"),
            "max_physical_ticks": header.get("max_physical_ticks"),
        },
        "record_counts": counts,
        "observed_selected_threat_ids": sorted(unique_threat_ids),
        "recorded_age_ranges": {
            arm: {
                "first": min(values) if values else None,
                "last": max(values) if values else None,
                "max_age": recorded_max_ages.get(arm),
                "terminal": branch_terminals.get(arm),
            }
            for arm, values in recorded_ages.items()
        },
        "support_reconciliation": {
            "raw_summary": {
                "post_prefix_supported_fresh_greedy_choices": actual_supported,
                "different_threat_id_supported_choices": actual_different_id,
                "aligned_online_frozen_greedy_contrasts": (
                    raw_summary.get("aligned_online_frozen_greedy_contrasts") if raw_summary else None
                ),
            },
            "recomputed_from_recorded_selection_kind": recorded_fresh_choices,
            "candidate_exact_different_id_supports": counters["candidate_exact_different_id_supports"],
            "calculated_postprefix_all_candidates_supported_choices": counters["calculated_postprefix_all_candidates_supported_choices"],
            "calculated_postprefix_different_id_supported_choices": counters["calculated_postprefix_different_id_supported_choices"],
            "candidate_witness_mismatch_choices": counters["candidate_witness_mismatch_choices"],
            "candidate_witness_mismatch_fields": counters["candidate_witness_mismatch_fields"],
            "candidate_row_comparisons": counters["comparisons"],
            "reconciles_supported_choice_count": supported_reconciles,
            "reconciles_different_id_choice_count": different_id_reconciles,
            "reconciles_all_candidate_support_witnesses": witness_reconciles,
            "aligned_contrast_independently_recomputed": False,
        },
        "fresh_choices": details,
        "limits": {
            "max_decompressed_input_bytes": MAX_INPUT_BYTES,
            "max_line_bytes": MAX_LINE_BYTES,
            "max_candidate_row_comparisons": comparison_limit,
            "row_ttl_ticks": REFUGE_ROW_TTL_TICKS,
        },
    }


class OpportunityDiagnosticTests(unittest.TestCase):
    def setUp(self) -> None:
        self.key = {"subject_species": "Herbivore", "threat_species": "Carnivore", "subject_energy": 29}
        self.sample_record = {
            "record_type": "prefix_tick",
            "arm": "shared_prefix_online",
            "focal_events": [
                {
                    "RefugeEpisodeEnded": {
                        "completed": True,
                        "closure": "Arrived",
                        "generalization_mode": "Generalized",
                        "generalized_context": copy.deepcopy(self.key),
                        "threat_id": 37,
                        "ended_tick": 7,
                        "attribution_known": True,
                    }
                }
            ],
        }
        self.row = {
            "key": copy.deepcopy(self.key),
            "last_completed_tick": 7,
            "completed": 1,
            "safe_arrivals": 1,
            "threat_damage": 0,
            "movement_energy": 3,
        }

    def choice_record(self, tick: int, key: dict[str, Any] | None = None, supported: bool = True) -> dict[str, Any]:
        candidate_key = copy.deepcopy(key if key is not None else self.key)
        return {
            "record_type": "suffix_tick",
            "scope": "suffix",
            "arm": "trained_online",
            "world_tick": tick,
            "focal_before": {
                "life": {"age": tick, "max_age": 105},
                "generalized_outcomes": [copy.deepcopy(self.row)],
            },
            "production_choice": {
                "choice_tick": tick,
                "threat_id": 38,
                "is_fresh": True,
                "selection_kind": "generalized_supported_greedy" if supported else "generalized_fallback_unsupported",
                "candidates": [
                    {
                        "key": candidate_key,
                        "support_completed": 1 if candidate_key == self.key else 0,
                        "support_last_completed_tick": 7 if candidate_key == self.key else None,
                        "supported_unique_fresh": supported,
                        "different_threat_id_supported": candidate_key == self.key,
                        "support_sources": [37] if candidate_key == self.key else [],
                    }
                ],
            },
        }

    def test_actual_generalized_arrival_and_table_row_schema_link_source_id(self) -> None:
        samples = completed_samples(self.sample_record)
        self.assertEqual(len(samples), 1)
        self.assertEqual(samples[0]["source_threat_id"], 37)
        ids, unknown, event_count = row_source_ids(self.row, samples)
        self.assertEqual(ids, [37])
        self.assertFalse(unknown)
        self.assertEqual(event_count, self.row["completed"])

    def test_exact_key_is_supported_by_a_different_threat_id_within_ttl(self) -> None:
        counters = {"comparisons": 0, "recorded_postprefix_supported_fresh_greedy_choices": 0,
                    "calculated_postprefix_all_candidates_supported_choices": 0,
                    "calculated_postprefix_different_id_supported_choices": 0,
                    "candidate_exact_different_id_supports": 0,
                    "candidate_witness_mismatch_choices": 0,
                    "candidate_witness_mismatch_fields": 0}
        result = analyze_choice(self.choice_record(8), completed_samples(self.sample_record), counters, 100)
        self.assertTrue(result["all_candidates_calculated_supported_unique_fresh"])
        self.assertTrue(result["all_candidates_calculated_supported_by_different_ids"])
        self.assertEqual(result["candidates"][0]["fresh_matching_row_indices"], [0])
        self.assertEqual(result["candidate_witness_mismatches"], [])

    def test_row_ttl_includes_age_100_and_excludes_age_101(self) -> None:
        samples = completed_samples(self.sample_record)
        at_limit = analyze_choice(self.choice_record(107), samples, self.empty_counters(), 100)
        expired = analyze_choice(self.choice_record(108, supported=False), samples, self.empty_counters(), 100)
        self.assertTrue(at_limit["candidates"][0]["calculated_supported_unique_fresh"])
        self.assertFalse(expired["candidates"][0]["calculated_supported_unique_fresh"])
        self.assertEqual(expired["candidates"][0]["comparisons"][0]["row_age_ticks"], 101)

    def test_exact_key_mismatch_names_field_and_removes_support(self) -> None:
        mismatch = copy.deepcopy(self.key)
        mismatch["subject_energy"] = 30
        record = self.choice_record(8, mismatch, supported=False)
        candidate = record["production_choice"]["candidates"][0]
        candidate["support_completed"] = 0
        candidate["support_last_completed_tick"] = None
        candidate["different_threat_id_supported"] = False
        candidate["support_sources"] = []
        result = analyze_choice(record, completed_samples(self.sample_record), self.empty_counters(), 100)
        comparison = result["candidates"][0]["comparisons"][0]
        self.assertEqual(comparison["mismatch_fields"], ["subject_energy"])
        self.assertFalse(result["all_candidates_calculated_supported_unique_fresh"])

    @staticmethod
    def empty_counters() -> dict[str, int]:
        return {
            "comparisons": 0,
            "recorded_postprefix_supported_fresh_greedy_choices": 0,
            "calculated_postprefix_all_candidates_supported_choices": 0,
            "calculated_postprefix_different_id_supported_choices": 0,
            "candidate_exact_different_id_supports": 0,
            "candidate_witness_mismatch_choices": 0,
            "candidate_witness_mismatch_fields": 0,
        }


def run_self_tests() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(OpportunityDiagnosticTests)
    result = unittest.TextTestRunner(stream=sys.stderr, verbosity=2).run(suite)
    return 0 if result.wasSuccessful() else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, nargs="?", help="JSONL or verified .jsonl.zst experiment artifact")
    parser.add_argument("--self-test", action="store_true", help="run hand-built parser/audit fixtures; no World is created")
    parser.add_argument("--max-input-bytes", type=int, default=MAX_INPUT_BYTES)
    parser.add_argument("--max-line-bytes", type=int, default=MAX_LINE_BYTES)
    parser.add_argument("--max-comparisons", type=int, default=MAX_COMPARISONS)
    args = parser.parse_args()
    if args.self_test:
        return run_self_tests()
    if args.input is None:
        parser.error("input is required unless --self-test is used")
    if not 1 <= args.max_input_bytes <= MAX_INPUT_BYTES:
        parser.error(f"--max-input-bytes must be between 1 and {MAX_INPUT_BYTES}")
    if not 1 <= args.max_line_bytes <= MAX_LINE_BYTES:
        parser.error(f"--max-line-bytes must be between 1 and {MAX_LINE_BYTES}")
    if not 1 <= args.max_comparisons <= MAX_COMPARISONS:
        parser.error(f"--max-comparisons must be between 1 and {MAX_COMPARISONS}")
    try:
        meta: dict[str, Any] = {}
        result = diagnose(
            records(args.input, args.max_input_bytes, args.max_line_bytes, meta),
            meta,
            args.max_comparisons,
        )
    except (DiagnosisError, OSError) as exc:
        print(f"diagnosis error: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(result, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
