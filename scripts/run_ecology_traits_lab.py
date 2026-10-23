#!/usr/bin/env python3
"""Run the fixed ecology-traits preregistration and preserve auditable output."""
from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import shutil
import time

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = ROOT / "docs/ECOLOGY_TRAITS_PROTOCOL.md"
FIXTURE = ROOT / "scenarios/ecology-traits-lab.json"
RUST_RUNNER = ROOT / "examples/ecology_traits_lab.rs"
EVALUATION_SEEDS = [50001, 50002, 50003, 50004, 50005, 50006, 50007, 50008]
CALIBRATION_SEEDS = [50041, 50042, 50043, 50044]
ARMS = ["control", "mobility", "vision", "stamina", "vision_stamina"]
ENVIRONMENTS = ["static", "cycle"]
MAX_OUTPUT_BYTES = 1024**3
MAX_SECONDS = 300
TEXT_EXTENSIONS = {
    ".json", ".jsonl", ".csv", ".tsv", ".txt", ".log", ".md", ".yaml",
    ".yml", ".toml", ".ini", ".cfg",
}
SOURCE_FILES = [
    "Cargo.toml", "Cargo.lock", "src/control.rs", "src/environment.rs",
    "src/scenario.rs", "src/entity/genetics.rs", "src/entity/lifestate.rs",
    "src/creatures/carnivore.rs", "src/creatures/herbivore.rs",
    "src/creatures/omnivore.rs", "src/simulation/checkpoint.rs",
    "src/simulation/event.rs", "src/systems/interaction.rs",
    "src/systems/life_cycle.rs", "src/systems/metabolism.rs",
    "examples/ecology_traits_lab.rs", "scenarios/ecology-traits-lab.json",
    "tests/ecology_traits.rs", "tests/genetics.rs",
]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_identity() -> tuple[str, str]:
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=all"],
        cwd=ROOT,
        text=True,
    )
    if status:
        raise RuntimeError("source worktree must be clean before a registered run")
    return commit, status


def source_hashes() -> tuple[str, dict[str, str]]:
    files = {name: sha256_file(ROOT / name) for name in SOURCE_FILES}
    digest = hashlib.sha256(
        json.dumps(files, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    return digest, files


SEED_KEY = r"(?<![a-z0-9])(?:[a-z0-9]+_)*seeds?(?![a-z0-9])"
SEED_FIELD = re.compile(SEED_KEY, re.IGNORECASE)
SEED_SCALAR = re.compile(SEED_KEY + r"['\"]?\s*[:=]\s*['\"]?(\d+)", re.IGNORECASE)
SEED_ARRAY = re.compile(SEED_KEY + r"['\"]?\s*[:=]\s*\[", re.IGNORECASE)
CLI_SEED = re.compile(r"(?i)--(?:world-)?seed(?:[=\s]+)(\d+)")
NUMBER = re.compile(r"(?<![0-9])([0-9]+)(?![0-9])")
FILENAME_SEED = re.compile(r"(?i)(?:^|[^a-z0-9])seed[-_]?([0-9]+)(?:$|[^0-9])")
MAX_SCAN_LINE_BYTES = 16 * 1024 * 1024


def _record_value(value: int, wanted: dict[int, re.Pattern], matches: dict[str, list[str]], record: str) -> None:
    if value in wanted and record not in matches[str(value)]:
        matches[str(value)].append(record)


def _record_matches(line: str, wanted: dict[int, re.Pattern], matches: dict[str, list[str]], record: str) -> bool:
    """Record scalar/CLI seeds and return whether a multiline seed list stays open."""
    for match in SEED_SCALAR.finditer(line):
        _record_value(int(match.group(1)), wanted, matches, record)
    for match in CLI_SEED.finditer(line):
        _record_value(int(match.group(1)), wanted, matches, record)
    array_open = False
    for match in SEED_ARRAY.finditer(line):
        tail = line[match.end():]
        close = tail.find("]")
        values = tail if close < 0 else tail[:close]
        for token in NUMBER.finditer(values):
            _record_value(int(token.group(1)), wanted, matches, record)
        array_open = close < 0
    return array_open


def _record_array_continuation(line: str, wanted: dict[int, re.Pattern], matches: dict[str, list[str]], record: str) -> bool:
    close = line.find("]")
    values = line if close < 0 else line[:close]
    for token in NUMBER.finditer(values):
        _record_value(int(token.group(1)), wanted, matches, record)
    return close < 0


def audit_seeds(root: Path, seeds: list[int]) -> dict:
    """Find declared seed usage without mistaking ticks/counts for seeds."""
    wanted = {seed: re.compile(rf"(?<![0-9]){seed}(?![0-9])") for seed in seeds}
    scanned = 0
    matches: dict[str, list[str]] = {str(seed): [] for seed in seeds}
    for seed in seeds:
        if FILENAME_SEED.search(root.name) and int(FILENAME_SEED.search(root.name).group(1)) == seed:
            matches[str(seed)].append(root.name)
    if root.exists():
        for path in sorted(p for p in root.rglob("*") if p.is_file()):
            if path.suffix.lower() not in TEXT_EXTENSIONS:
                continue
            scanned += 1
            rel = str(path.relative_to(root))
            for component in path.relative_to(root).parts:
                filename_match = FILENAME_SEED.search(component)
                if filename_match:
                    value = int(filename_match.group(1))
                    if value in wanted and rel not in matches[str(value)]:
                        matches[str(value)].append(rel)
            try:
                with path.open("r", encoding="utf-8", errors="strict") as stream:
                    csv_reader = None
                    if path.suffix.lower() in {".csv", ".tsv"}:
                        delimiter = "\t" if path.suffix.lower() == ".tsv" else ","
                        csv.field_size_limit(MAX_SCAN_LINE_BYTES)
                        csv_reader = csv.reader(stream, delimiter=delimiter)
                        header = next(csv_reader, [])
                        seed_columns = [
                            i for i, name in enumerate(header)
                            if SEED_FIELD.fullmatch(name.strip().strip("\"'"))
                        ]
                        if seed_columns:
                            for row in csv_reader:
                                if any(len(cell.encode("utf-8")) > MAX_SCAN_LINE_BYTES for cell in row):
                                    raise RuntimeError(f"oversized CSV row in {path}")
                                for index in seed_columns:
                                    if index < len(row) and row[index].strip().isdigit():
                                        value = int(row[index].strip())
                                        if value in wanted:
                                            record = f"{rel}:{csv_reader.line_num}"
                                            if record not in matches[str(value)]:
                                                matches[str(value)].append(record)
                            continue
                        stream.seek(0)
                    seed_array_open = False
                    for line_no, line in enumerate(stream, 1):
                        if len(line.encode("utf-8")) > MAX_SCAN_LINE_BYTES:
                            raise RuntimeError(f"oversized text line in {path}:{line_no}")
                        record = f"{rel}:{line_no}"
                        if seed_array_open:
                            seed_array_open = _record_array_continuation(line, wanted, matches, record)
                        if not SEED_FIELD.search(line) and not CLI_SEED.search(line):
                            continue
                        seed_array_open = _record_matches(line, wanted, matches, record) or seed_array_open
            except (OSError, UnicodeError, csv.Error, RuntimeError) as error:
                raise RuntimeError(f"cannot completely audit seed registry file {path}: {error}") from error
    return {
        "root": str(root),
        "scanned_text_files": scanned,
        "matches": matches,
        "unseen": [seed for seed in seeds if not matches[str(seed)]],
        "matching_rule": "declared seed fields, seed CSV columns, CLI --seed arguments, or seed-number filenames",
    }


def median(values: list[float]) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) / 2


def summarize(raw_path: Path, phase: str, seeds: list[int]) -> dict:
    rows = []
    mechanism = None
    run_record = None
    with raw_path.open("r", encoding="utf-8") as stream:
        for line_no, line in enumerate(stream, 1):
            try:
                row = json.loads(line)
            except json.JSONDecodeError as error:
                raise RuntimeError(f"invalid raw JSONL at line {line_no}: {error}")
            if row.get("kind") == "run":
                run_record = row
            elif row.get("kind") == "mechanism_check":
                mechanism = row
            elif row.get("kind") == "replicate":
                rows.append(row)
    expected = {(seed, arm, environment) for seed in seeds for arm in ARMS for environment in ENVIRONMENTS}
    observed = {
        (row["seed"], row["arm"]["name"], row["environment"])
        for row in rows
    }
    if len(rows) != len(expected) or observed != expected:
        raise RuntimeError(
            f"replicate set mismatch: expected {len(expected)}, observed {len(rows)}"
        )
    if mechanism is None or mechanism.get("status") != "passed":
        raise RuntimeError("schedule mechanism check did not pass")
    for row in rows:
        if row["paid_vision_maintenance"] > row["requested_vision_maintenance"]:
            raise RuntimeError("paid vision cost exceeds requested amount")
        if row["paid_stamina_maintenance"] > row["requested_stamina_maintenance"]:
            raise RuntimeError("paid stamina cost exceeds requested amount")

    measures = [
        "living_founders", "living_descendant_breeders",
        "living_f1_breeders", "f1_offspring_that_bred",
        "descendant_breeders_that_bred", "total_births",
        "f1_births", "f2_births", "max_generation", "food_eaten",
        "movement_energy", "requested_vision_maintenance",
        "requested_stamina_maintenance", "paid_vision_maintenance",
        "paid_stamina_maintenance", "resource_replenishment_events",
    ]
    groups: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        groups[f"{row['environment']}/{row['arm']['name']}"].append(row)
    summaries = {}
    for name, group in sorted(groups.items()):
        outcome = {}
        for metric in measures:
            values = [float(row[metric]) for row in group]
            outcome[metric] = {
                "per_seed": {str(row["seed"]): row[metric] for row in group},
                "median": median(values),
                "min": min(values),
                "max": max(values),
            }
        summaries[name] = outcome
    return {
        "schema": "ecology-traits-summary-v1",
        "phase": phase,
        "ticks_per_world": 2000,
        "seed_count": len(seeds),
        "world_count": len(rows),
        "run": run_record,
        "mechanism_check": mechanism,
        "groups": summaries,
        "per_seed": [
            {
                "seed": row["seed"], "environment": row["environment"],
                "arm": row["arm"]["name"], "living_founders": row["living_founders"],
                "living_descendant_breeders": row["living_descendant_breeders"],
                "living_f1_breeders": row["living_f1_breeders"],
                "f1_offspring_that_bred": row["f1_offspring_that_bred"],
                "descendant_breeders_that_bred": row["descendant_breeders_that_bred"],
                "total_births": row["total_births"], "f1_births": row["f1_births"],
                "f2_births": row["f2_births"], "max_generation": row["max_generation"],
                "genotype_counts": row["genotype_counts"],
                "death_causes": row["death_causes"],
                "food_eaten": row["food_eaten"],
                "movement_energy": row["movement_energy"],
                "resource_replenishment_events": row["resource_replenishment_events"],
                "requested_vision_maintenance": row["requested_vision_maintenance"],
                "requested_stamina_maintenance": row["requested_stamina_maintenance"],
                "paid_vision_maintenance": row["paid_vision_maintenance"],
                "paid_stamina_maintenance": row["paid_stamina_maintenance"],
            }
            for row in rows
        ],
    }


def render_report(summary: dict, manifest: dict) -> str:
    lines = [
        "# Ecology traits laboratory report",
        "",
        f"Phase: `{summary['phase']}`; revision: `{manifest['revision']}`.",
        f"Worlds: {summary['world_count']} at {summary['ticks_per_world']} ticks each.",
        f"Raw data SHA-256: `{manifest['raw_sha256']}`.",
        "",
        "The schedule mechanism check passed, including checkpoint continuation "
        "and unchanged RNG. Trait outcomes below are independent populations; "
        "they do not estimate within-population selection or allele-frequency drift.",
        "",
        "| Environment / arm | Living descendant breeders (median) | "
        "F1 births (median) | F2 births (median) | Food eaten (median) | "
        "Vision energy paid (median) | Stamina energy paid (median) |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    metrics = [
        "living_descendant_breeders", "f1_births", "f2_births",
        "food_eaten", "paid_vision_maintenance", "paid_stamina_maintenance",
    ]
    for group, data in sorted(summary["groups"].items()):
        medians = [data[metric]["median"] for metric in metrics]
        lines.append("| " + group + " | " + " | ".join(f"{value:g}" for value in medians) + " |")
    lines.extend([
        "",
        "Per-seed outcomes, death causes, genotype counts, requested/paid costs, "
        "tick samples, event rows, and final checkpoint states are in `summary.json` "
        "and `raw.jsonl`. Calibration is only a mechanism/serialization check; "
        "it cannot tune parameters. Interpret a null or negative outcome as such.",
        "",
    ])
    return "\n".join(lines)


def capture_process(command: list[str], cwd: Path, raw_path: Path, stderr_path: Path,
                    max_seconds: float, max_output_bytes: int) -> dict:
    """Stream child output without line buffering; always time out and reap it."""
    start = time.monotonic()
    digest = hashlib.sha256()
    raw_bytes = 0
    timed_out = False
    output_exceeded = False
    error_text = None
    process = None
    try:
        with raw_path.open("wb") as raw, stderr_path.open("wb") as stderr:
            try:
                process = subprocess.Popen(
                    command, cwd=cwd, stdout=subprocess.PIPE, stderr=stderr
                )
                fd = process.stdout.fileno()
                os.set_blocking(fd, False)
                with selectors.DefaultSelector() as selector:
                    selector.register(fd, selectors.EVENT_READ)
                    while True:
                        remaining = max_seconds - (time.monotonic() - start)
                        if remaining <= 0 and not timed_out:
                            timed_out = process.poll() is None or bool(selector.get_map())
                            if process.poll() is None:
                                process.kill()
                        if output_exceeded:
                            break
                        if selector.get_map():
                            for key, _ in selector.select(timeout=min(0.1, max(0.0, remaining))):
                                try:
                                    chunk = os.read(key.fd, 64 * 1024)
                                except BlockingIOError:
                                    continue
                                if not chunk:
                                    selector.unregister(key.fd)
                                    continue
                                if raw_bytes + len(chunk) > max_output_bytes:
                                    output_exceeded = True
                                    process.kill()
                                    break
                                raw_bytes += len(chunk)
                                raw.write(chunk)
                                raw.flush()
                                digest.update(chunk)
                        if process.poll() is not None and not selector.get_map():
                            break
                        if not selector.get_map() and process.poll() is None:
                            wait_for = min(0.05, max(0.001, remaining))
                            try:
                                process.wait(timeout=wait_for)
                            except subprocess.TimeoutExpired:
                                continue
                            break
                    if process.poll() is None:
                        process.kill()
                    process.wait(timeout=5)
            except Exception as error:
                error_text = f"{type(error).__name__}: {error}"
            finally:
                if process is not None:
                    if process.poll() is None:
                        process.kill()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
                    if process.stdout is not None:
                        process.stdout.close()
    except Exception as error:
        error_text = f"{type(error).__name__}: {error}"
    elapsed = time.monotonic() - start
    exit_code = process.returncode if process is not None else None
    status = "failed" if error_text or process is None or exit_code or timed_out or output_exceeded else "completed"
    try:
        stderr_hash = sha256_file(stderr_path)
    except OSError:
        stderr_hash = None
    return {
        "finished_at": datetime.now(timezone.utc).isoformat(),
        "elapsed_seconds": elapsed,
        "exit_code": exit_code,
        "timed_out": timed_out,
        "output_limit_exceeded": output_exceeded,
        "raw_bytes": raw_bytes,
        "raw_sha256": digest.hexdigest(),
        "stderr_sha256": stderr_hash,
        "status": status,
        "runner_error": error_text,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=("calibration", "evaluation"))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--seed-registry", type=Path,
        default=Path("/root/curious/data/experiments"),
    )
    args = parser.parse_args()
    output = args.output.resolve()
    archive_root = args.seed_registry.resolve()
    if not output.is_relative_to(archive_root):
        raise SystemExit(f"output must be a new directory below {archive_root}")
    if output.exists():
        raise SystemExit(f"refusing to overwrite existing output: {output}")
    seeds = CALIBRATION_SEEDS if args.phase == "calibration" else EVALUATION_SEEDS
    seed_audit = audit_seeds(args.seed_registry, seeds)
    if seed_audit["unseen"] != seeds:
        used = {seed: locations for seed, locations in seed_audit["matches"].items() if locations}
        raise SystemExit(f"preregistered seeds were already seen; stop and re-review: {used}")

    commit, _ = source_identity()
    source_sha, source_files_sha = source_hashes()
    build_command = ["cargo", "build", "--release", "--example", "ecology_traits_lab"]
    build_started = time.monotonic()
    subprocess.run(build_command, cwd=ROOT, check=True)
    build_seconds = time.monotonic() - build_started
    binary = ROOT / "target/release/examples/ecology_traits_lab"
    if not binary.is_file():
        raise RuntimeError("release lab executable was not produced")

    output.mkdir(parents=True)
    manifest_path = output / "manifest.json"
    raw_path = output / "raw.jsonl"
    stderr_path = output / "runner.stderr.log"
    probe_path = output / "lab-probe"
    protocol_copy = output / "protocol.md"
    shutil.copy2(binary, probe_path)
    shutil.copy2(PROTOCOL, protocol_copy)
    probe_path.chmod(0o555)
    started_at = datetime.now(timezone.utc).isoformat()
    manifest = {
        "schema": "curious-ecology-traits-run-v1",
        "protocol_version": 1,
        "run_name": "ecology-traits-lab",
        "phase": args.phase,
        "status": "running",
        "revision": commit,
        "source_sha256": source_sha,
        "source_files_sha256": source_files_sha,
        "source_worktree": str(ROOT),
        "binary_sha256": sha256_file(probe_path),
        "protocol_sha256": sha256_file(protocol_copy),
        "fixture_sha256": sha256_file(FIXTURE),
        "runner_sha256": sha256_file(Path(__file__)),
        "rust_runner_sha256": sha256_file(RUST_RUNNER),
        "build_command": build_command,
        "build_seconds": build_seconds,
        "seeds": seeds,
        "seed_audit": seed_audit,
        "arm_names": ARMS,
        "environments": ENVIRONMENTS,
        "ticks_per_world": 2000,
        "command": ["./lab-probe", args.phase],
        "started_at": started_at,
        "max_seconds": MAX_SECONDS,
        "max_output_bytes": MAX_OUTPUT_BYTES,
        "raw_file": "raw.jsonl",
        "summary_file": "summary.json",
        "protocol_file": "protocol.md",
        "probe_file": "lab-probe",
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    run_result = capture_process(
        [str(probe_path), args.phase], output, raw_path, stderr_path,
        MAX_SECONDS, MAX_OUTPUT_BYTES,
    )
    manifest.update(run_result)
    try:
        if run_result["output_limit_exceeded"]:
            raise RuntimeError("raw output exceeded the preregistered 1 GiB limit")
        if run_result["runner_error"]:
            raise RuntimeError(run_result["runner_error"])
        if run_result["status"] != "completed":
            raise RuntimeError("runner exited before the preregistered batch completed")
        summary = summarize(raw_path, args.phase, seeds)
        summary_path = output / "summary.json"
        summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
        manifest["summary_sha256"] = sha256_file(summary_path)
        report_path = output / "runreport.md"
        report_path.write_text(render_report(summary, manifest), encoding="utf-8")
        manifest["report_sha256"] = sha256_file(report_path)
    except Exception as error:  # Preserve raw data and failure evidence.
        manifest["status"] = "failed"
        manifest["summary_error"] = str(error)
    manifest["finished_at"] = datetime.now(timezone.utc).isoformat()
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if manifest["status"] != "completed":
        raise SystemExit(f"lab {manifest['status']}; evidence preserved in {output}")
    print(json.dumps({"status": "complete", "output": str(output), "raw_sha256": manifest["raw_sha256"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
