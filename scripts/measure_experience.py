#!/usr/bin/env python3
"""Run the fixed water experiment once; preserve raw output and source identities."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from datetime import datetime, timezone

ARMS = ("rule_based", "naive_online", "experienced_online", "experienced_frozen")
PAIRS = [(1, 30001), (2, 30002), (3, 30003), (4, 30004),
         (5, 30005), (42, 30006), (12345, 30007), (2026, 30008)]
WINDOWS = ("before_first_100", "before_last_100", "after_first_100", "after_last_100")
METRICS = ("water", "movement_energy", "incoming_damage", "rejected_actions",
           "full_drinks", "partial_drinks", "zero_water", "unfinished_episodes",
           "scored_episodes", "new_experiences", "near_choices", "far_choices", "other_choices")


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def window(rows):
    return {
        "trials": len(rows),
        "water": sum(row["water"] for row in rows),
        "movement_energy": sum(row["movement_energy"] for row in rows),
        "incoming_damage": sum(row["incoming_damage"] for row in rows),
        "rejected_actions": sum(row["rejected_actions"] for row in rows),
        "full_drinks": sum(row["water"] == 10 for row in rows),
        "partial_drinks": sum(0 < row["water"] < 10 for row in rows),
        "zero_water": sum(row["water"] == 0 for row in rows),
        "unfinished_episodes": sum(row["unfinished_episode"] for row in rows),
        "scored_episodes": sum(row["reward"] is not None for row in rows),
        "new_experiences": sum(row["visits_after"] - row["visits_before"] for row in rows),
        "near_choices": sum(row["target"] == {"x": 1, "y": 0} for row in rows),
        "far_choices": sum(row["target"] == {"x": 0, "y": 3} for row in rows),
        "other_choices": sum(row["target"] not in ({"x": 1, "y": 0}, {"x": 0, "y": 3}) for row in rows),
    }


def summarize(records):
    if [(row["training_seed"], row["evaluation_seed"]) for row in records] != PAIRS:
        raise ValueError("Unexpected, missing or reordered seed pairs")
    for row in records:
        if (row["schema"] != "curious-experience-v1"
                or row["protocol"] != "water-competition-reversal-v1"
                or row["training_trials"] != 400
                or row["evaluation_trials_per_arm"] != 400
                or row["reversal_at"] != 200
                or row["controlled_body_reset"] is not True):
            raise ValueError("Protocol mismatch")
        if [arm["arm"] for arm in row["arms"]] != list(ARMS):
            raise ValueError("Arm mismatch")
        for arm in row["arms"]:
            trials = arm["trials"]
            if len(trials) != 400:
                raise ValueError("Trial count mismatch")
            previous = arm["initial_visits"]
            for i, trial in enumerate(trials):
                if (trial["trial"] != i or trial["reversed"] != (i >= 200)
                        or trial["visits_before"] != previous
                        or trial["visits_after"] < previous
                        or not 0 <= trial["water"] <= 10
                        or trial["success"] != (trial["water"] == 10)
                        or trial["partial"] != (0 < trial["water"] < 10)
                        or trial["zero_water"] != (trial["water"] == 0)):
                    raise ValueError("Invalid trial accounting")
                previous = trial["visits_after"]
            for i, name in enumerate(WINDOWS):
                if arm["windows"][name] != window(trials[i * 100:(i + 1) * 100]):
                    raise ValueError("Raw trials and window summary differ")
            if arm["arm"] == "experienced_frozen":
                if (arm["initial_values"] != row["training_values"]
                        or arm["final_values"] != arm["initial_values"]
                        or arm["reversal_values"] != arm["initial_values"]
                        or any(trial["visits_after"] != arm["initial_visits"] for trial in trials)):
                    raise ValueError("Frozen table changed")
    means = {}
    for index, name in enumerate(ARMS):
        means[name] = {period: {
            metric: sum(row["arms"][index]["windows"][period][metric] for row in records) / len(records)
            for metric in METRICS
        } for period in WINDOWS}
    per_seed = []
    for row in records:
        arms = {arm["arm"]: arm for arm in row["arms"]}
        per_seed.append({
            "training_seed": row["training_seed"], "evaluation_seed": row["evaluation_seed"],
            "initial_experience_water_difference": (
                arms["experienced_online"]["windows"]["before_first_100"]["water"]
                - arms["naive_online"]["windows"]["before_first_100"]["water"]),
            "late_reversal_update_water_difference": (
                arms["experienced_online"]["windows"]["after_last_100"]["water"]
                - arms["experienced_frozen"]["windows"]["after_last_100"]["water"]),
            "windows": {arm: arms[arm]["windows"] for arm in ARMS},
        })
    return {"schema": "curious-experience-summary-v1", "pairs": len(records),
            "controlled_body_reset": True, "superiority_claim": None,
            "mean_per_100_trials": means, "per_seed": per_seed}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    paths = sorted(root.joinpath("src").rglob("*.rs")) + [
        root / "Cargo.toml", root / "Cargo.lock", root / "examples/experience_trial.rs",
        root / "docs/EXPERIENCE_PROTOCOL.md", Path(__file__).resolve()]
    manifest = {"schema": "curious-experience-run-v1", "status": "running",
                "started_at": datetime.now(timezone.utc).isoformat(),
                "git_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
                "git_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True).strip()),
                "binary": str(binary), "binary_sha256": sha(binary),
                "source_sha256": {str(path.relative_to(root)): sha(path) for path in paths},
                "seed_pairs": PAIRS, "timeout_seconds": 300}
    manifest_path = output / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    try:
        with (output / "raw.jsonl").open("x") as stdout, (output / "stderr.log").open("x") as stderr:
            subprocess.run([str(binary)], cwd=root, stdout=stdout, stderr=stderr, check=True, timeout=300)
        records = [json.loads(line) for line in (output / "raw.jsonl").read_text().splitlines()]
        report = summarize(records)
        (output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
        manifest.update(status="completed", raw_sha256=sha(output / "raw.jsonl"),
                        summary_sha256=sha(output / "summary.json"))
    except Exception as error:
        manifest.update(status="failed", error=str(error))
        raise
    finally:
        manifest["finished_at"] = datetime.now(timezone.utc).isoformat()
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"output": str(output), "status": manifest["status"]}))


if __name__ == "__main__":
    main()
