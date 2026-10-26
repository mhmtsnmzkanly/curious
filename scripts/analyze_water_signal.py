#!/usr/bin/env python3
"""Diagnose information content in the frozen V2 water-adaptation raw records.

Reads one seed file at a time. This is retrospective analysis only; it never
starts a simulation or modifies the source evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path
from typing import Any


INPUT = Path("data/experiments/water-adaptation-v2-formal-20261025")
OUTPUT = Path("data/operations/water-signal-diagnosis-20261026/analysis.json")
ARMS = ("trained_online", "trained_frozen", "naive_online", "rule_based")


def position_key(pos: dict[str, int]) -> tuple[int, int]:
    return int(pos["x"]), int(pos["y"])


def trunc_div(numerator: int, denominator: int) -> int:
    """Rust signed integer division rounds toward zero (unlike Python //)."""
    return (abs(numerator) // abs(denominator)) * (-1 if (numerator < 0) != (denominator < 0) else 1)


def baseline_value(amount: int, steps_len: int, life: dict[str, Any]) -> int:
    """Integer mirror of src/entity/foraging.rs::value for water."""
    travel = max(0, steps_len - 1)
    speed = max(1, int(life["speed"]))
    ticks = max(1, (travel + speed - 1) // speed)
    capacity = max(0, int(life["max_water"]) - int(life["water"]))
    meal = min(amount, capacity, 10)  # DRINK_WATER_AMOUNT in this source revision
    cost = travel * int(life["move_energy_cost"]) + ticks
    return trunc_div((2 * meal - cost) * 100, ticks)


def context_prediction(entries: list[dict[str, Any]], pos: tuple[int, int], speed: int,
                       move_cost: int, tick: int) -> int | None:
    for entry in entries:
        if (position_key(entry["pos"]) == pos and int(entry["speed"]) == speed
                and int(entry["move_energy_cost"]) == move_cost
                and tick - int(entry["last_seen_tick"]) <= 48):
            obs = entry["observations"]
            if obs:
                return trunc_div(sum(map(int, obs)), len(obs))
    return None


def rank(candidates: list[dict[str, Any]], scores: list[int]) -> tuple[int, ...]:
    # Rust max_by_key score, then shorter route, then earlier candidate.
    return tuple(sorted(range(len(candidates)), key=lambda i:
                         (scores[i], -int(candidates[i]["steps_len"]), -i), reverse=True))


def analyze_seed(path: Path) -> dict[str, Any]:
    raw = json.loads(path.read_text())
    seed = int(raw["seed"])
    result: dict[str, Any] = {"seed": seed, "arms": {}, "training_suffix": {}}
    for arm in ARMS:
        fork = raw["forks"][arm]
        ctx_updates: list[dict[str, Any]] = []
        candidate_observations = 0
        fresh_choice_opportunities = 0
        continuation_observations = 0
        context_covered = 0
        context_covered_fresh = 0
        context_score_changed = 0
        context_rank_changed = 0
        positive_predicted = 0
        negative_predicted = 0
        nonzero_prediction_rows: list[dict[str, Any]] = []
        outcome_values: list[dict[str, Any]] = []
        seen_outcomes: set[str] = set()
        residual_zero_count = 0
        residual_count = 0
        seen_last_seen: dict[tuple[int, int, int, int], int] = {}
        for tick_record in fork["ticks"]:
            tick = int(tick_record["tick"])
            state = tick_record["state_before"]
            focal = next((e for e in state["entities"] if e["id"] == 1), None)
            if focal is None:
                continue
            learner = focal["learner"]
            for entry in learner.get("context_entries", []):
                key = (*position_key(entry["pos"]), int(entry["speed"]),
                       int(entry["move_energy_cost"]))
                seen = int(entry["last_seen_tick"])
                if seen >= 201 and seen > seen_last_seen.get(key, -1):
                    obs = entry.get("observations", [])
                    if obs:
                        ctx_updates.append({"tick": seen, "pos": [key[0], key[1]],
                                            "residual": int(obs[-1])})
                    seen_last_seen[key] = seen
            choice = tick_record.get("focal_choice")
            if choice and choice.get("base_reason") == "SeekWater":
                candidates = choice.get("candidates", [])
                if len(candidates) >= 2:
                    candidate_observations += 1
                    fresh_choice = bool(choice.get("fresh_two_source_choice"))
                    if fresh_choice:
                        fresh_choice_opportunities += 1
                    else:
                        continuation_observations += 1
                    life = focal["life"]
                    move_cost = int(life["move_energy_cost"])
                    speed = max(1, int(life["speed"]))
                    bases = [baseline_value(int(c["amount"]), int(c["steps_len"]), life)
                             for c in candidates]
                    predictions = [context_prediction(learner.get("context_entries", []),
                                                       position_key(c["pos"]), speed, move_cost, tick)
                                   for c in candidates]
                    available = [p is not None for p in predictions]
                    if any(available):
                        context_covered += 1
                        if fresh_choice:
                            context_covered_fresh += 1
                    adjusted = [base + (pred or 0) for base, pred in zip(bases, predictions)]
                    if any(p not in (None, 0) for p in predictions):
                        context_score_changed += 1
                        nonzero_prediction_rows.append({
                            "tick": tick,
                            "fresh_choice": fresh_choice,
                            "candidate_positions": [list(position_key(c["pos"])) for c in candidates],
                            "baseline_scores": bases,
                            "predicted_residuals": predictions,
                            "baseline_top": list(position_key(candidates[rank(candidates, bases)[0]]["pos"])),
                            "adjusted_top": list(position_key(candidates[rank(candidates, adjusted)[0]]["pos"])),
                            "selected_target": ([int(selected["x"]), int(selected["y"])]
                                                 if (selected := choice.get("selected_target")) else None),
                            "selection_kind": choice.get("selection_kind"),
                        })
                    if fresh_choice and rank(candidates, bases) != rank(candidates, adjusted):
                        context_rank_changed += 1
                    if any(p is not None and p > 0 for p in predictions):
                        positive_predicted += 1
                    if any(p is not None and p < 0 for p in predictions):
                        negative_predicted += 1
            # Capture each distinct completed attempt exposed in the ordinary snapshot.
            last = learner.get("last_episode")
            if last and last.get("end") in ("Reached", "Failed"):
                attempt = last.get("attempt", {})
                baseline = attempt.get("choice_baseline")
                if baseline is not None:
                    ticks = max(1, int(attempt.get("ticks", 0)))
                    actual = trunc_div((int(last.get("water", 0)) * 2
                                        - int(attempt.get("movement_energy", 0))
                                        - int(attempt.get("ticks", 0))) * 100, ticks)
                    residual = max(-1000, min(1000, actual - int(baseline)))
                    item = {"target": position_key(attempt["target"]), "ticks": int(attempt.get("ticks", 0)),
                            "water": int(last.get("water", 0)), "movement_energy": int(attempt.get("movement_energy", 0)),
                            "baseline": int(baseline), "actual": actual, "residual": residual}
                    outcome_id = json.dumps(item, sort_keys=True)
                    if outcome_id not in seen_outcomes:
                        seen_outcomes.add(outcome_id)
                        outcome_values.append(item)
        residuals = [x["residual"] for x in ctx_updates]
        result["arms"][arm] = {
            "two_candidate_observations": candidate_observations,
            "fresh_choice_opportunities": fresh_choice_opportunities,
            "ongoing_continuation_observations": continuation_observations,
            "context_covered_opportunities": context_covered,
            "context_covered_fresh_choices": context_covered_fresh,
            "nonzero_context_score_opportunities": context_score_changed,
            "context_changed_baseline_ranking": context_rank_changed,
            "opportunities_with_positive_prediction": positive_predicted,
            "opportunities_with_negative_prediction": negative_predicted,
            "nonzero_prediction_rows": nonzero_prediction_rows,
            "context_updates": len(ctx_updates),
            "context_update_residual_distribution": dict(sorted(Counter(residuals).items())),
            "context_updates_zero": sum(v == 0 for v in residuals),
            "context_outcome_rows": ctx_updates,
            "qualified_completed_outcomes_observed_in_snapshots": outcome_values,
        }
    for arm in ("trained_online", "trained_frozen", "naive_online"):
        result["training_suffix"][arm] = {"learner_treatment": raw["forks"][arm]["learner_treatment"]}
    return result


def run(input_dir: Path, output: Path) -> dict[str, Any]:
    manifest_path = input_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    seed_files = sorted(input_dir.glob("seed-*.raw.json"))
    if not seed_files:
        raise FileNotFoundError(f"no seed-*.raw.json under {input_dir}")
    seeds = []
    raw_hashes: dict[str, str] = {}
    for path in seed_files:  # deliberately release each parsed raw before opening the next
        digest = hashlib.sha256()
        with path.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                digest.update(chunk)
        actual_hash = digest.hexdigest()
        expected_hash = manifest.get("raw_sha256", {}).get(path.name)
        if expected_hash and actual_hash != expected_hash:
            raise ValueError(f"raw SHA-256 mismatch: {path.name}")
        raw_hashes[path.name] = actual_hash
        seeds.append(analyze_seed(path))
    summary: dict[str, Any] = {"schema": "water-signal-diagnosis-v1", "input": str(input_dir),
                               "manifest_sha256": hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
                               "source_revision": manifest.get("source_revision"),
                               "source_sha256": manifest.get("source_sha256"),
                               "protocol_sha256": manifest.get("protocol_sha256"),
                               "raw_sha256": raw_hashes,
                               "seed_count": len(seeds), "seeds": seeds, "aggregate": {}}
    for arm in ARMS:
        fields = ("two_candidate_observations", "fresh_choice_opportunities",
                  "ongoing_continuation_observations", "context_covered_opportunities",
                  "context_covered_fresh_choices",
                  "nonzero_context_score_opportunities", "context_changed_baseline_ranking",
                  "context_updates", "context_updates_zero")
        summary["aggregate"][arm] = {field: sum(s["arms"][arm][field] for s in seeds)
                                      for field in fields}
        dist: Counter[int] = Counter()
        for seed in seeds:
            dist.update({int(k): v for k, v in
                         seed["arms"][arm]["context_update_residual_distribution"].items()})
        summary["aggregate"][arm]["context_update_residual_distribution"] = dict(sorted(dist.items()))
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=INPUT)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    args = parser.parse_args()
    summary = run(args.input, args.output)
    print(json.dumps(summary["aggregate"], indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
