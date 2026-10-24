#!/usr/bin/env python3
"""Read-only, streaming diagnosis of the frozen V17 water-residual formal run.

No World is constructed here. Each JSONL record is one complete seed capture and
is discarded before the next seed is read.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


ARMS = ("rule_based", "naive_online", "trained_online", "trained_frozen")
EXPECTED_SEEDS = (52001, 52002, 52003, 52004, 52005, 52006, 52007, 52008)
FORMAL_RAW_SHA256 = "c10e7d55431f6c10e96acf1f84d7c4295d67208b54ab6ab788ae210d30fcf78c"
MANIFEST_SHA256 = "d95a37b0f363e2f2172c02812c32768b0318e75a980498539ef4d46a559520c4"
PROTOCOL_SHA256 = "0410576adef7474ec23dd9e92a73eb5c9306f87c0f6450696c3ca5e424a22bc0"
FORMAL_SUMMARY_SHA256 = "255aa41c6abd12f37fce07561799c142c043580f20fe7c63ff9d0f26db242d52"
FORMAL_AUDIT_SHA256 = "9c25548db4bd66219d2f14c8c891fafa152c76ea48c79fa8036a39df2ed30601"
PROBE_SHA256 = "0c0f1aebc82baf75dc0eac925289e71741e770df46b3ba6d637a8fc59a1f7213"
CALIBRATION_RAW_SHA256 = "12f3ab2edcf092930af7c385dec457761294b7a48d88a4f4ac61962ee9beae39"
CALIBRATION_SUMMARY_SHA256 = "9b1305b8586baf5aecf4b095ee2f582194aa2724df640fca75b9d3ca854ada4c"
CALIBRATION_MANIFEST_SHA256 = "7cb7bd7f857bcea7b100e0d613ffd97ece675a6cfa232ded5cc7b5c44f2debd3"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def pos_key(pos: dict[str, int] | None) -> tuple[int, int] | None:
    if not pos:
        return None
    return int(pos["x"]), int(pos["y"])


def target_key(pos: dict[str, int] | None) -> str | None:
    key = pos_key(pos)
    return None if key is None else f"{key[0]},{key[1]}"


def best_candidate(candidates: list[dict[str, Any]], score_key: str) -> dict[str, Any] | None:
    """Mirror V17's greedy score, shorter-route, then input-order tie break."""
    if not candidates:
        return None
    return max(
        enumerate(candidates),
        key=lambda pair: (pair[1][score_key], -pair[1].get("steps_len", 0), -pair[0]),
    )[1]


def choice_diagnostics(tick: dict[str, Any], arm: str) -> dict[str, Any] | None:
    choice = tick.get("focal_choice")
    if not choice:
        return None
    candidates = choice.get("candidates") or []
    if not candidates:
        return {
            "fresh": bool(choice.get("fresh")),
            "selection_kind": choice.get("selection_kind", "unknown"),
            "selected_target": target_key(choice.get("selected_target")),
            "candidate_count": 0,
            "residual_overrode_baseline": False,
            "current_greedy_target": None,
            "baseline_only_target": None,
            "mean_spread": None,
            "context": [],
        }
    current = best_candidate(candidates, "final_score")
    baseline = best_candidate(candidates, "current_baseline")
    selected_target = target_key(choice.get("selected_target"))
    selection_kind = choice.get("selection_kind", "unknown")
    if (
        choice.get("fresh")
        and selection_kind == "greedy"
        and selected_target != target_key(current["pos"])
    ):
        raise ValueError(
            f"fresh greedy selection at tick {tick.get('tick')} in {arm} does not match "
            "the recomputed final-score ranking"
        )
    # These local rank checks describe the recorded perception only. They do not
    # replay a policy or predict a resolver outcome.
    residual_winner = target_key(current["pos"]) != target_key(baseline["pos"])
    means = [int(candidate.get("residual_mean", 0)) for candidate in candidates]
    context = []
    for candidate in candidates:
        context.append({
            "source": target_key(candidate.get("pos")),
            "amount": candidate.get("amount"),
            "route_steps": candidate.get("steps_len"),
            "baseline": candidate.get("current_baseline"),
            "residual_mean": candidate.get("residual_mean"),
            "residual_count": candidate.get("residual_count"),
            "score": candidate.get("final_score"),
        })
    before = next((d.get("before", {}) for d in tick.get("decisions", []) if d.get("id") == 1), {})
    return {
        "fresh": bool(choice.get("fresh")),
        "selection_kind": selection_kind,
        "selected_target": selected_target,
        "candidate_count": len(candidates),
        "residual_overrode_baseline": residual_winner,
        "current_greedy_target": target_key(current["pos"]),
        "baseline_only_target": target_key(baseline["pos"]),
        "mean_spread": max(means) - min(means),
        "context": context,
        "body_water": before.get("water"),
        "body_energy": before.get("energy"),
        "body_health": before.get("health"),
    }


def focal_event_totals(tick: dict[str, Any], focal_id: int = 1) -> dict[str, int]:
    totals = {"water": 0, "movement_energy": 0, "drinks": 0, "moves": 0}
    for event in tick.get("events", []):
        if not event:
            continue
        name, body = next(iter(event.items()))
        if not isinstance(body, dict) or body.get("id") != focal_id:
            continue
        if name == "EntityDrank":
            totals["water"] += int(body.get("amount", 0))
            totals["drinks"] += 1
        elif name == "EntityMoved":
            totals["movement_energy"] += int(body.get("energy_cost", 0))
            totals["moves"] += 1
    return totals


def closures_for(tick: dict[str, Any]) -> list[dict[str, Any]]:
    closure_data = tick.get("learner_closures") or {}
    return list(closure_data.get("predecision_closures") or []) + list(closure_data.get("postresolver_closures") or [])


def trunc_div(numerator: int, denominator: int) -> int:
    """Rust integer division truncates toward zero; Python // floors negatives."""
    return abs(numerator) // denominator * (-1 if numerator < 0 else 1)


def realized_residual_sample(closure: dict[str, Any]) -> int | None:
    """Recompute the registered resolver residual for an eligible Reached episode."""
    attempt = closure.get("attempt", {})
    if (
        closure.get("end") != "Reached"
        or attempt.get("outcome_overflowed", False)
        or attempt.get("candidate_censored", False)
        or attempt.get("choice_baseline") is None
    ):
        return None
    ticks = int(attempt.get("ticks", 0))
    if ticks <= 0:
        return None
    actual = trunc_div((int(closure.get("water", 0)) * 2 - int(attempt.get("movement_energy", 0)) - ticks) * 100, ticks)
    return actual - int(attempt["choice_baseline"])


def closure_censor_reasons(closure: dict[str, Any]) -> list[str]:
    """Incomplete/invalid outcomes are censored; a verified Failed result can complete."""
    reasons = []
    end = str(closure.get("end", "unknown"))
    if end in {"Interrupted", "Disabled", "Died", "TimedOut"}:
        reasons.append(end)
    attempt = closure.get("attempt", {})
    if attempt.get("outcome_overflowed", False):
        reasons.append("outcome_overflowed")
    if attempt.get("candidate_censored", False):
        reasons.append("candidate_censored")
    return reasons


def residual_table(state: dict[str, Any]) -> dict[str, tuple[int, int]]:
    entity = next((entity for entity in state.get("entities", []) if entity.get("id") == 1), {})
    return {
        target_key(item.get("pos")): (int(item.get("sum", 0)), int(item.get("visits", 0)))
        for item in (entity.get("learner", {}).get("residuals") or [])
    }


def physical_metrics(ticks: list[dict[str, Any]]) -> dict[str, Any]:
    totals = Counter()
    closure_ends: Counter[str] = Counter()
    censor_reasons: Counter[str] = Counter()
    censor_count = 0
    samples: list[int] = []
    actual_table_updates = 0
    actual_table_sum_delta = 0
    for tick in ticks:
        totals.update(focal_event_totals(tick))
        for closure in closures_for(tick):
            end = str(closure.get("end", "unknown"))
            closure_ends[end] += 1
            reasons = closure_censor_reasons(closure)
            if reasons:
                censor_count += 1
                censor_reasons.update(reasons)
            sample = realized_residual_sample(closure)
            if sample is not None:
                samples.append(sample)
        before_table = residual_table(tick.get("state_before", {}))
        after_table = residual_table(tick.get("state_after", {}))
        for source in set(before_table) | set(after_table):
            old_sum, old_count = before_table.get(source, (0, 0))
            new_sum, new_count = after_table.get(source, (0, 0))
            if new_count > old_count:
                actual_table_updates += new_count - old_count
                actual_table_sum_delta += new_sum - old_sum
    terminal = ticks[-1].get("state_after", {}) if ticks else {}
    focal = next((entity for entity in terminal.get("entities", []) if entity.get("id") == 1), None)
    death_events = []
    for tick in ticks:
        for event in tick.get("events", []):
            if event and "EntityDied" in event:
                death = event["EntityDied"]
                if death.get("id") == 1:
                    death_events.append({"tick": tick.get("tick"), "event": death})
    life = (focal or {}).get("life", {})
    return {
        "ticks": len(ticks),
        "water_acquired": totals["water"],
        "movement_energy_paid": totals["movement_energy"],
        "drink_events": totals["drinks"],
        "movement_events": totals["moves"],
        "observed_survival_ticks": sum(
            1 for tick in ticks
            if any(e.get("id") == 1 and e.get("phase") == "Active" for e in tick.get("state_after", {}).get("entities", []))
        ),
        "terminal_energy": life.get("energy"),
        "terminal_water": life.get("water"),
        "terminal_health": life.get("health"),
        "terminal_phase": (focal or {}).get("phase"),
        "death_events": death_events,
        "episode_closures_by_end": dict(sorted(closure_ends.items())),
        "censored_episode_closures": censor_count,
        "censored_episode_closures_by_reason": dict(sorted(censor_reasons.items())),
        "eligible_reached_residual_samples": len(samples),
        "recomputed_residual_sample_sum": sum(samples),
        "recomputed_residual_samples_positive": sum(value > 0 for value in samples),
        "recomputed_residual_samples_zero": sum(value == 0 for value in samples),
        "recomputed_residual_samples_negative": sum(value < 0 for value in samples),
        "observed_residual_table_update_count": actual_table_updates,
        "observed_residual_table_sum_delta": actual_table_sum_delta,
    }


def normalize_state_for_witness(state: dict[str, Any]) -> dict[str, Any]:
    """Normalize only the preregistered learner tables and update flag."""
    normalized = json.loads(json.dumps(state))
    for entity in normalized.get("entities", []):
        if entity.get("id") != 1:
            continue
        learner = entity.get("learner", {})
        learner["values"] = "<normalized-table>"
        learner["residuals"] = "<normalized-table>"
        learner["update_enabled"] = "<normalized-update-setting>"
    return normalized


def own_resolver_signature(tick: dict[str, Any]) -> tuple[tuple[str, int, str | None], ...]:
    events = []
    for event in tick.get("events", []):
        if not event:
            continue
        name, body = next(iter(event.items()))
        if not isinstance(body, dict) or body.get("id") != 1:
            continue
        if name == "EntityMoved":
            events.append((name, int(body.get("energy_cost", 0)), None))
        elif name == "EntityDrank":
            events.append((name, int(body.get("amount", 0)), target_key(body.get("pos"))))
    return tuple(events)


def paired_witnesses(seed_record: dict[str, Any]) -> list[dict[str, Any]]:
    online = seed_record["forks"]["trained_online"]["ticks"]
    frozen = seed_record["forks"]["trained_frozen"]["ticks"]
    frozen_by_tick = {tick["tick"]: tick for tick in frozen}
    witnesses = []
    for online_tick in online:
        frozen_tick = frozen_by_tick.get(online_tick["tick"])
        if frozen_tick is None:
            continue
        online_choice = online_tick.get("focal_choice") or {}
        frozen_choice = frozen_tick.get("focal_choice") or {}
        if not online_choice.get("fresh") or not frozen_choice.get("fresh"):
            continue
        if online_choice.get("selection_kind") != "greedy" or frozen_choice.get("selection_kind") != "greedy":
            continue
        if normalize_state_for_witness(online_tick["state_before"]) != normalize_state_for_witness(frozen_tick["state_before"]):
            continue
        if online_tick.get("rng_before") != frozen_tick.get("rng_before"):
            continue
        focal_online = next((d for d in online_tick.get("decisions", []) if d.get("id") == 1), None)
        focal_frozen = next((d for d in frozen_tick.get("decisions", []) if d.get("id") == 1), None)
        if not focal_online or not focal_frozen or focal_online.get("perception") != focal_frozen.get("perception"):
            continue
        target_online = target_key(online_choice.get("selected_target"))
        target_frozen = target_key(frozen_choice.get("selected_target"))
        if not target_online or target_online == target_frozen:
            continue
        score_online = own_resolver_signature(online_tick)
        score_frozen = own_resolver_signature(frozen_tick)
        if score_online == score_frozen:
            continue
        witnesses.append({
            "tick": online_tick["tick"],
            "trained_online_target": target_online,
            "trained_frozen_target": target_frozen,
            "online_resolver_events": score_online,
            "frozen_resolver_events": score_frozen,
            "same_physical_prestate_perception_rng": True,
        })
    return witnesses


def summarize_arm(arm: str, ticks: list[dict[str, Any]]) -> dict[str, Any]:
    counts: Counter[str] = Counter()
    candidates_by_source: dict[str, dict[str, Any]] = defaultdict(lambda: {
        "observations": 0, "amounts": set(), "route_steps": set(), "residual_means": set(), "residual_counts": set(),
    })
    context_key_counts: Counter[tuple[Any, ...]] = Counter()
    choice_records = []
    fresh_body_water: list[int] = []
    fresh_body_energy: list[int] = []
    fresh_candidate_counts: Counter[int] = Counter()
    for tick in ticks:
        diagnostic = choice_diagnostics(tick, arm)
        if diagnostic is None:
            continue
        counts["focal_decision_records"] += 1
        counts[f"decision_selection_kind:{diagnostic['selection_kind']}"] += 1
        if diagnostic["fresh"]:
            counts["fresh_choices"] += 1
            counts[f"fresh_selection_kind:{diagnostic['selection_kind']}"] += 1
            if diagnostic.get("body_water") is not None:
                fresh_body_water.append(diagnostic["body_water"])
            if diagnostic.get("body_energy") is not None:
                fresh_body_energy.append(diagnostic["body_energy"])
            fresh_candidate_counts[diagnostic["candidate_count"]] += 1
            if diagnostic["selection_kind"] == "explore":
                counts["fresh_exploration_choices"] += 1
            elif diagnostic["selection_kind"] == "greedy":
                counts["fresh_greedy_choices"] += 1
            if (arm in {"trained_online", "trained_frozen", "naive_online"}
                    and diagnostic["selection_kind"] == "greedy"
                    and diagnostic["residual_overrode_baseline"]):
                counts["fresh_residual_changes_current_greedy_rank"] += 1
        else:
            counts["nonfresh_decisions"] += 1
        if diagnostic.get("selected_target"):
            counts["selected_targets"] += 1
        for candidate in diagnostic["context"] if diagnostic["fresh"] else []:
            source = candidate["source"]
            entry = candidates_by_source[source]
            entry["observations"] += 1
            entry["amounts"].add(candidate.get("amount"))
            entry["route_steps"].add(candidate.get("route_steps"))
            entry["residual_means"].add(candidate.get("residual_mean"))
            entry["residual_counts"].add(candidate.get("residual_count"))
            if candidate.get("residual_count", 0) > 0:
                context_key_counts[(source, candidate.get("amount"), candidate.get("route_steps"))] += 1
        if (
            diagnostic["fresh"]
            and diagnostic["selection_kind"] == "greedy"
            and diagnostic["residual_overrode_baseline"]
        ):
            choice_records.append({"tick": tick.get("tick"), **diagnostic})
    contexts = {}
    for source, values in sorted(candidates_by_source.items()):
        contexts[source] = {
            "candidate_observations": values["observations"],
            "distinct_current_amounts": sorted(x for x in values["amounts"] if x is not None),
            "distinct_route_lengths": sorted(x for x in values["route_steps"] if x is not None),
            "distinct_residual_means": sorted(x for x in values["residual_means"] if x is not None),
            "distinct_residual_counts": sorted(x for x in values["residual_counts"] if x is not None),
            "historied_current_contexts": [
                {"amount": amount, "route_steps": route, "observations": count}
                for (key, amount, route), count in sorted(context_key_counts.items()) if key == source
            ],
        }
    physical = physical_metrics(ticks)
    merged_counts = dict(counts)
    merged_counts.update(physical)
    merged_counts["fresh_choice_body_water_range"] = [min(fresh_body_water), max(fresh_body_water)] if fresh_body_water else None
    merged_counts["fresh_choice_body_energy_range"] = [min(fresh_body_energy), max(fresh_body_energy)] if fresh_body_energy else None
    merged_counts["fresh_candidate_count_distribution"] = {str(key): value for key, value in sorted(fresh_candidate_counts.items())}
    return {"arm": arm, "counts": merged_counts, "source_context": contexts, "choices": choice_records}


def pareto_relation(left: dict[str, Any], right: dict[str, Any]) -> str:
    # Movement cost is lower-is-better; all remaining components are higher-is-better.
    keys = ("water_acquired", "movement_energy_paid", "terminal_energy", "terminal_water", "terminal_health", "observed_survival_ticks")
    a = [left[k] for k in keys]
    b = [right[k] for k in keys]
    better = [x >= y if k != "movement_energy_paid" else x <= y for k, x, y in zip(keys, a, b)]
    worse = [x <= y if k != "movement_energy_paid" else x >= y for k, x, y in zip(keys, a, b)]
    strict_better = any(x > y if k != "movement_energy_paid" else x < y for k, x, y in zip(keys, a, b))
    strict_worse = any(x < y if k != "movement_energy_paid" else x > y for k, x, y in zip(keys, a, b))
    if not strict_better and not strict_worse:
        return "identical"
    if all(better) and strict_better:
        return "candidate_dominates"
    if all(worse) and strict_worse:
        return "candidate_dominated"
    return "tradeoff"


def analyze_seed(seed_record: dict[str, Any]) -> dict[str, Any]:
    if seed_record.get("schema") != "curious-water-residual-seed-v1":
        raise ValueError(f"unexpected raw schema: {seed_record.get('schema')}")
    if seed_record.get("status") != "complete":
        raise ValueError(f"seed {seed_record.get('seed')} is not complete")
    forks = seed_record.get("forks", {})
    if set(forks) != set(ARMS):
        raise ValueError(f"seed {seed_record.get('seed')} arm mismatch: {sorted(forks)}")
    arms = {arm: summarize_arm(arm, forks[arm]["ticks"]) for arm in ARMS}
    for arm in ARMS:
        fork_state = forks[arm].get("state_at_tick_201_before_world_tick", {})
        final_state = forks[arm]["ticks"][-1].get("state_after", {}) if forks[arm].get("ticks") else {}
        arms[arm]["residual_table_at_rate_reversal"] = {
            source: {"sum": value[0], "count": value[1], "mean_truncated_toward_zero": trunc_div(value[0], value[1]) if value[1] else 0}
            for source, value in sorted(residual_table(fork_state).items())
        }
        arms[arm]["residual_table_at_end"] = {
            source: {"sum": value[0], "count": value[1], "mean_truncated_toward_zero": trunc_div(value[0], value[1]) if value[1] else 0}
            for source, value in sorted(residual_table(final_state).items())
        }
    metrics = {arm: arms[arm]["counts"] for arm in ARMS}
    relations = {
        "trained_online_vs_rule_based": pareto_relation(metrics["trained_online"], metrics["rule_based"]),
        "trained_online_vs_trained_frozen": pareto_relation(metrics["trained_online"], metrics["trained_frozen"]),
        "naive_online_vs_rule_based_descriptive": pareto_relation(metrics["naive_online"], metrics["rule_based"]),
        "naive_online_vs_trained_frozen_descriptive": pareto_relation(metrics["naive_online"], metrics["trained_frozen"]),
    }
    return {
        "seed": seed_record["seed"],
        "physical_ticks": seed_record.get("physical_ticks"),
        "prefix": summarize_arm("prefix", seed_record.get("prefix", [])),
        "arms": arms,
        "pareto_relations": relations,
        "trained_online_vs_trained_frozen_greedy_witnesses": paired_witnesses(seed_record),
    }


def analyze_raw(
    raw_path: Path,
    manifest_path: Path | None = None,
    source_root: Path | None = None,
    verify_hashes: bool = True,
) -> dict[str, Any]:
    raw_digest = sha256(raw_path) if verify_hashes else None
    if verify_hashes and raw_digest != FORMAL_RAW_SHA256:
        raise ValueError(f"formal raw hash mismatch: got {raw_digest}")
    manifest = None
    if manifest_path:
        manifest_digest = sha256(manifest_path) if verify_hashes else None
        if verify_hashes and manifest_digest != MANIFEST_SHA256:
            raise ValueError(f"manifest hash mismatch: got {manifest_digest}")
        manifest = json.loads(manifest_path.read_text())
        if manifest.get("raw_sha256") != FORMAL_RAW_SHA256:
            raise ValueError("manifest does not bind the registered formal raw hash")
        if manifest.get("protocol_sha256") != PROTOCOL_SHA256:
            raise ValueError("manifest protocol hash differs from registered protocol")
    verified_artifacts = {"formal_raw": raw_digest or FORMAL_RAW_SHA256}
    if verify_hashes:
        formal_dir = raw_path.parent
        required = {
            "formal_summary": (formal_dir / "summary.json", FORMAL_SUMMARY_SHA256),
            "formal_protocol": (formal_dir / "protocol.md", PROTOCOL_SHA256),
            "formal_probe": (formal_dir / "water-residual-probe", PROBE_SHA256),
            "formal_root_audit": (formal_dir / "root-formal-audit.json", FORMAL_AUDIT_SHA256),
        }
        calibration_dir = formal_dir.parent / "water-residual-v1-calibration-20261023"
        required.update({
            "calibration_raw": (calibration_dir / "raw.jsonl", CALIBRATION_RAW_SHA256),
            "calibration_summary": (calibration_dir / "summary.json", CALIBRATION_SUMMARY_SHA256),
            "calibration_manifest": (calibration_dir / "manifest.json", CALIBRATION_MANIFEST_SHA256),
        })
        for name, (path, expected) in required.items():
            if not path.is_file():
                raise ValueError(f"required frozen evidence file is missing: {path}")
            actual = sha256(path)
            if actual != expected:
                raise ValueError(f"{name} hash mismatch: got {actual}")
            verified_artifacts[name] = actual
        if manifest:
            frozen_root = source_root or Path("/root/curious-research/water-lab-wave2")
            for relative, expected in manifest.get("source_files_sha256", {}).items():
                path = frozen_root / relative
                if not path.is_file():
                    raise ValueError(f"frozen source file is missing: {path}")
                actual = sha256(path)
                if actual != expected:
                    raise ValueError(f"frozen source hash mismatch for {relative}: got {actual}")
                verified_artifacts[f"frozen_source:{relative}"] = actual
    seeds = []
    with raw_path.open("r", encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, start=1):
            if not line.strip():
                continue
            record = json.loads(line)
            seeds.append(analyze_seed(record))
    observed = tuple(item["seed"] for item in seeds)
    if observed != EXPECTED_SEEDS:
        raise ValueError(f"seed order mismatch: {observed}")
    arms = {}
    physical_keys = ("water_acquired", "movement_energy_paid", "observed_survival_ticks", "terminal_energy", "terminal_water", "terminal_health")
    for arm in ARMS:
        arms[arm] = {key: sum(seed["arms"][arm]["counts"][key] for seed in seeds) for key in physical_keys}
        arms[arm]["fresh_choices"] = sum(seed["arms"][arm]["counts"].get("fresh_choices", 0) for seed in seeds)
        arms[arm]["fresh_greedy_choices"] = sum(seed["arms"][arm]["counts"].get("fresh_greedy_choices", 0) for seed in seeds)
        arms[arm]["fresh_exploration_choices"] = sum(seed["arms"][arm]["counts"].get("fresh_exploration_choices", 0) for seed in seeds)
        arms[arm]["fresh_residual_changes_current_greedy_rank"] = sum(seed["arms"][arm]["counts"].get("fresh_residual_changes_current_greedy_rank", 0) for seed in seeds)
        arms[arm]["censored_episode_closures"] = sum(seed["arms"][arm]["counts"]["censored_episode_closures"] for seed in seeds)
    witness_count = sum(len(seed["trained_online_vs_trained_frozen_greedy_witnesses"]) for seed in seeds)
    return {
        "schema": "curious-water-residual-diagnosis-v1",
        "provenance": {
            "raw_path": str(raw_path),
            "raw_sha256": raw_digest or FORMAL_RAW_SHA256,
            "manifest_path": str(manifest_path) if manifest_path else None,
            "manifest_sha256": sha256(manifest_path) if manifest_path and verify_hashes else (MANIFEST_SHA256 if manifest_path else None),
            "protocol_sha256": PROTOCOL_SHA256,
            "verified_artifact_sha256": verified_artifacts,
            "source_revision": manifest.get("revision") if manifest else None,
            "source_tree_sha256": manifest.get("source_sha256") if manifest else None,
            "probe_sha256": manifest.get("binary_sha256") if manifest else None,
            "analysis": "read-only Python stream over each seed record; no simulation/World construction",
        },
        "formal_seed_count": len(seeds),
        "formal_ticks": sum(seed["physical_ticks"] or 0 for seed in seeds),
        "aggregate_arm_metrics": arms,
        "formal_mechanism_witness_count_recomputed": witness_count,
        "choice_rows_scope": {
            "schema": "fresh-greedy-rank-changes-only-v1",
            "count": sum(len(seed["arms"][arm]["choices"]) for seed in seeds for arm in ARMS),
            "meaning": "Only fresh greedy decisions where removing the residual changes the locally top-ranked source are included. All-choice counts, per-seed metrics, context ranges, and witnesses remain summarized; the full raw choice trace is retained in the immutable formal archive and is regenerated by the analyzer.",
        },
        "seeds": seeds,
        "interpretation_limits": [
            "Recorded-perception rank counterfactuals remove the residual from current recorded scores only; they do not resimulate decisions or world outcomes.",
            "Rule-based contrasts also change epsilon exploration and downstream RNG consumption, so they compare complete policies.",
            "Per-source residuals pool outcomes across amount, route, body, competition, and time contexts; reported context variation is descriptive and does not prove harmful aliasing.",
            "Eight fixed seeds in one 5x5 fixture do not establish general efficacy, lifetime benefit, or ecological fitness.",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--source-root", type=Path, help="frozen candidate checkout bound by the manifest")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--skip-hash-check", action="store_true", help="only for synthetic/local fixtures")
    args = parser.parse_args()
    result = analyze_raw(args.raw, args.manifest, args.source_root, verify_hashes=not args.skip_hash_check)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({
        "output": str(args.output),
        "seeds": result["formal_seed_count"],
        "ticks": result["formal_ticks"],
        "witnesses": result["formal_mechanism_witness_count_recomputed"],
        "raw_sha256": result["provenance"]["raw_sha256"],
    }, sort_keys=True))


if __name__ == "__main__":
    main()
