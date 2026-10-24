#!/usr/bin/env python3
"""Safely compress and restore individual experiment JSONL evidence files.

This helper deliberately has no batch mode.  Its default mode is a dry run;
``--apply`` is required before an archive operation can remove its raw source.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import subprocess
import tempfile
from typing import BinaryIO

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = Path("data/experiments")
MAX_BYTES = 1024**3
MAX_SECONDS = 120
MAX_SCAN_LINE_BYTES = 64 * 1024 * 1024
MAX_SEED_ROLES = 128
MAX_SEED_VALUES = 100_000
MAX_SEED_ROLE_BYTES = 256
CHUNK_BYTES = 1024 * 1024
CODEC = "zstd"

# Keep the seed declaration grammar aligned with run_ecology_traits_lab.py.
SEED_KEY = r"(?<![a-z0-9])(?:[a-z0-9]+_)*seeds?(?![a-z0-9])"
SEED_SCALAR = re.compile(SEED_KEY + r"['\"]?\s*[:=]\s*['\"]?(\d+)", re.IGNORECASE)
SEED_ARRAY = re.compile(SEED_KEY + r"['\"]?\s*[:=]\s*\[", re.IGNORECASE)
CLI_SEED = re.compile(r"(?i)--(?:world-)?seed(?:[=\s]+)(\d+)")
NUMBER = re.compile(r"(?<![0-9])([0-9]+)(?![0-9])")


class ArchiveError(RuntimeError):
    """A safety or integrity check prevented the requested operation."""


def _checked_path(path: Path, root: Path, *, must_exist: bool) -> Path:
    """Resolve an allowed path while rejecting symlink components and escapes."""
    root = root.resolve(strict=True)
    allow_root = root / ALLOWLIST
    # Verify the allowlist itself is not redirected through a symlink.
    _reject_symlink_components(root, allow_root)
    candidate = path if path.is_absolute() else root / path
    try:
        lexical = Path(os.path.abspath(candidate))
        lexical.relative_to(allow_root)
    except ValueError as error:
        raise ArchiveError(f"path is outside {ALLOWLIST}: {path}") from error
    _reject_symlink_components(allow_root, lexical)
    if must_exist and not lexical.exists():
        raise ArchiveError(f"path does not exist: {path}")
    if lexical.exists() and not lexical.is_file():
        raise ArchiveError(f"expected a regular file: {path}")
    return lexical


def _reject_symlink_components(parent: Path, path: Path) -> None:
    try:
        relative = path.relative_to(parent)
    except ValueError as error:
        raise ArchiveError(f"path escapes allowed directory: {path}") from error
    current = parent
    for component in relative.parts:
        if component in ("", ".", ".."):
            raise ArchiveError(f"unsafe path component in {path}")
        current = current / component
        try:
            if current.is_symlink():
                raise ArchiveError(f"symlink paths are not allowed: {current}")
        except OSError as error:
            raise ArchiveError(f"cannot inspect path {current}: {error}") from error


def _relative(path: Path, root: Path) -> str:
    return path.relative_to(root.resolve(strict=True)).as_posix()


def _hash_stream(stream: BinaryIO, *, cap: int = MAX_BYTES) -> tuple[int, str]:
    digest = hashlib.sha256()
    size = 0
    while True:
        chunk = stream.read(CHUNK_BYTES)
        if not chunk:
            break
        size += len(chunk)
        if size > cap:
            raise ArchiveError(f"file exceeds {cap} byte limit")
        digest.update(chunk)
    return size, digest.hexdigest()


def _hash_file(path: Path) -> tuple[int, str]:
    with path.open("rb") as stream:
        return _hash_stream(stream)


def _seed_metadata(path: Path) -> tuple[list[int], dict[str, list[int]]]:
    """Extract every declared scalar, array, and CLI seed from bounded lines."""
    roles: dict[str, set[int]] = {}
    array_role: str | None = None
    seed_value_count = 0

    def add(role: str, value: int) -> None:
        nonlocal seed_value_count
        if len(role.encode("utf-8")) > MAX_SEED_ROLE_BYTES:
            raise ArchiveError(f"seed role exceeds {MAX_SEED_ROLE_BYTES} bytes")
        values = roles.get(role)
        if values is None:
            if len(roles) >= MAX_SEED_ROLES:
                raise ArchiveError(f"declared seed roles exceed {MAX_SEED_ROLES}")
            values = roles.setdefault(role, set())
        if value not in values and seed_value_count >= MAX_SEED_VALUES:
            raise ArchiveError(f"declared seed values exceed {MAX_SEED_VALUES}")
        if value not in values:
            seed_value_count += 1
        values.add(value)

    with path.open("rb") as stream:
        line_number = 0
        while True:
            raw = stream.readline(MAX_SCAN_LINE_BYTES + 1)
            if not raw:
                break
            line_number += 1
            if len(raw) > MAX_SCAN_LINE_BYTES:
                raise ArchiveError(
                    f"seed scan line exceeds {MAX_SCAN_LINE_BYTES} bytes at line {line_number}"
                )
            try:
                line = raw.decode("utf-8", errors="strict")
            except UnicodeError as error:
                raise ArchiveError(f"invalid UTF-8 in JSONL at line {line_number}") from error

            if array_role is not None:
                close = line.find("]")
                values = line if close < 0 else line[:close]
                for match in NUMBER.finditer(values):
                    add(array_role, int(match.group(1)))
                if close >= 0:
                    array_role = None

            for match in SEED_SCALAR.finditer(line):
                key_match = re.search(SEED_KEY, match.group(0), re.IGNORECASE)
                role = key_match.group(0).strip("'\"").lower() if key_match else "seed"
                add(role, int(match.group(1)))
            for match in SEED_ARRAY.finditer(line):
                key_match = re.search(SEED_KEY, match.group(0), re.IGNORECASE)
                role = key_match.group(0).strip("'\"").lower() if key_match else "seeds"
                tail = line[match.end():]
                close = tail.find("]")
                values = tail if close < 0 else tail[:close]
                for number in NUMBER.finditer(values):
                    add(role, int(number.group(1)))
                if close < 0:
                    array_role = role
            for match in CLI_SEED.finditer(line):
                add("cli_seed", int(match.group(1)))

    normalized = {key: sorted(values) for key, values in sorted(roles.items()) if values}
    return sorted({value for values in normalized.values() for value in values}), normalized


def _verify_seed_metadata(path: Path, payload: dict) -> None:
    seeds, roles = _seed_metadata(path)
    if seeds != payload.get("seeds") or roles != payload.get("seed_roles"):
        raise ArchiveError("archive index seed declarations do not match original bytes")


def _limit_child() -> None:
    resource.setrlimit(resource.RLIMIT_CPU, (MAX_SECONDS, MAX_SECONDS))
    resource.setrlimit(resource.RLIMIT_FSIZE, (MAX_BYTES, MAX_BYTES))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))


def _run_zstd(command: list[str], *, stdout: BinaryIO | int, stderr: int = subprocess.PIPE) -> None:
    executable = shutil.which("zstd")
    if executable is None:
        raise ArchiveError("zstd CLI is required but was not found in PATH")
    command = [executable, *command]
    try:
        completed = subprocess.run(
            command,
            stdout=stdout,
            stderr=stderr,
            timeout=MAX_SECONDS,
            check=False,
            preexec_fn=_limit_child,
        )
    except subprocess.TimeoutExpired as error:
        raise ArchiveError(f"zstd exceeded {MAX_SECONDS} seconds") from error
    if completed.returncode != 0:
        message = (completed.stderr or b"").decode("utf-8", errors="replace").strip()
        raise ArchiveError(f"zstd failed ({completed.returncode}): {message[:500]}")


def _fsync_dir(path: Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _write_json_atomic(index_path: Path, payload: dict) -> None:
    encoded = (json.dumps(payload, sort_keys=True, indent=2) + "\n").encode("utf-8")
    if index_path.exists():
        if index_path.is_symlink() or index_path.read_bytes() != encoded:
            raise ArchiveError(f"existing archive index differs; refusing overwrite: {index_path}")
        return
    fd, temporary = tempfile.mkstemp(prefix=f".{index_path.name}.", suffix=".tmp", dir=index_path.parent)
    temp_path = Path(temporary)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(encoded)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temp_path, index_path)
        _fsync_dir(index_path.parent)
    except FileExistsError:
        if index_path.read_bytes() != encoded:
            raise ArchiveError(f"existing archive index differs; refusing overwrite: {index_path}")
    finally:
        temp_path.unlink(missing_ok=True)


def _publish_archive(temp_archive: Path, archive_path: Path) -> None:
    if archive_path.exists():
        if archive_path.is_symlink():
            raise ArchiveError(f"archive path is a symlink: {archive_path}")
        # Existing data is only accepted when it has the exact expected bytes.
        if _hash_file(archive_path) != _hash_file(temp_archive):
            raise ArchiveError(f"existing archive differs; refusing overwrite: {archive_path}")
        temp_archive.unlink()
        return
    os.link(temp_archive, archive_path)
    temp_archive.unlink()
    _fsync_dir(archive_path.parent)


def _decompress_verified(archive_path: Path, temp_raw: Path, expected_size: int, expected_hash: str) -> tuple[int, str]:
    with temp_raw.open("wb") as output:
        _run_zstd(["-q", "-d", "-c", "--single-thread", str(archive_path)], stdout=output)
        output.flush()
        os.fsync(output.fileno())
    size, digest = _hash_file(temp_raw)
    if (size, digest) != (expected_size, expected_hash):
        raise ArchiveError("decompressed archive does not match original size/SHA-256")
    return size, digest


def archive_file(path: Path, *, root: Path = ROOT, apply: bool = False) -> dict:
    root = root.resolve(strict=True)
    source = _checked_path(path, root, must_exist=False)
    if source.suffix != ".jsonl":
        raise ArchiveError("only individual .jsonl files can be archived")
    archive = _checked_path(source.with_name(source.name + ".zst"), root, must_exist=False)
    index = _checked_path(source.with_name(source.name + ".archive.json"), root, must_exist=False)
    if not source.exists():
        if not archive.exists() or not index.exists():
            raise ArchiveError(f"raw source does not exist and no complete archive is available: {source}")
        indexed_source, indexed_archive, payload = _validated_index(index, root)
        if indexed_source != source or indexed_archive != archive:
            raise ArchiveError("archive index does not prove this source path")
        if _hash_file(archive) != (payload["archive_size"], payload["archive_sha256"]):
            raise ArchiveError("archive bytes do not match archive index")
        fd, name = tempfile.mkstemp(prefix=f".{source.name}.verify.", dir=source.parent)
        os.close(fd)
        temporary = Path(name)
        try:
            _decompress_verified(archive, temporary, payload["original_size"], payload["original_sha256"])
            _verify_seed_metadata(temporary, payload)
        finally:
            temporary.unlink(missing_ok=True)
        return {**payload, "dry_run": not apply, "source_removed": True, "already_archived": True}
    if os.path.samefile(source, archive) if archive.exists() else False:
        raise ArchiveError("archive path aliases the raw source")

    source_stat = source.stat()
    if source_stat.st_size > MAX_BYTES:
        raise ArchiveError(f"source exceeds {MAX_BYTES} byte limit")
    source_size, source_hash = _hash_file(source)
    seeds, seed_roles = _seed_metadata(source)
    payload = {
        "archivepath": _relative(archive, root),
        "archive_sha256": None,
        "archive_size": None,
        "codec": CODEC,
        "originalpath": _relative(source, root),
        "original_sha256": source_hash,
        "original_size": source_size,
        "seed_roles": seed_roles,
        "seeds": seeds,
        "status": "verified",
    }
    if not apply:
        return {**payload, "dry_run": True, "source_would_be_removed": True}

    archive.parent.mkdir(parents=False, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=f".{archive.name}.", suffix=".tmp", dir=archive.parent)
    temp_archive = Path(temporary)
    try:
        with os.fdopen(fd, "wb") as output:
            _run_zstd(["-q", "-c", "--single-thread", "--check", "-3", str(source)], stdout=output)
            output.flush()
            os.fsync(output.fileno())
        archive_size, archive_hash = _hash_file(temp_archive)
        payload["archive_size"] = archive_size
        payload["archive_sha256"] = archive_hash

        # Verify the complete compressed file before publishing/removing anything.
        verify_fd, verify_name = tempfile.mkstemp(prefix=f".{source.name}.verify.", dir=source.parent)
        os.close(verify_fd)
        verify_path = Path(verify_name)
        try:
            _decompress_verified(temp_archive, verify_path, source_size, source_hash)
            _verify_seed_metadata(verify_path, payload)
        finally:
            verify_path.unlink(missing_ok=True)

        _publish_archive(temp_archive, archive)
        # Re-check the published archive and index proof before considering unlink.
        if _hash_file(archive) != (archive_size, archive_hash):
            raise ArchiveError("published archive changed during verification")
        _write_json_atomic(index, payload)
        if json.loads(index.read_text(encoding="utf-8")) != payload:
            raise ArchiveError("published archive index failed read-back verification")
        verify_fd, verify_name = tempfile.mkstemp(prefix=f".{source.name}.verify.", dir=source.parent)
        os.close(verify_fd)
        verify_path = Path(verify_name)
        try:
            _decompress_verified(archive, verify_path, source_size, source_hash)
            _verify_seed_metadata(verify_path, payload)
        finally:
            verify_path.unlink(missing_ok=True)

        # Last-moment identity and byte fingerprint guard against source changes.
        current_stat = source.stat()
        identity_fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
        if any(getattr(current_stat, key) != getattr(source_stat, key) for key in identity_fields):
            raise ArchiveError("source fingerprint changed; raw source was retained")
        if _hash_file(source) != (source_size, source_hash):
            raise ArchiveError("source bytes changed; raw source was retained")
        if archive.is_symlink() or _hash_file(archive) != (archive_size, archive_hash):
            raise ArchiveError("archive changed before source removal; raw source was retained")
        if index.is_symlink() or index.read_bytes() != (json.dumps(payload, sort_keys=True, indent=2) + "\n").encode("utf-8"):
            raise ArchiveError("archive index changed before source removal; raw source was retained")
        final_stat = source.stat()
        if (final_stat.st_dev, final_stat.st_ino) != (source_stat.st_dev, source_stat.st_ino):
            raise ArchiveError("source path was replaced; raw source was retained")
        source.unlink()
        _fsync_dir(source.parent)
        return {**payload, "dry_run": False, "source_removed": True}
    finally:
        temp_archive.unlink(missing_ok=True)


def _validated_index(index_path: Path, root: Path) -> tuple[Path, Path, dict]:
    index = _checked_path(index_path, root, must_exist=True)
    try:
        payload = json.loads(index.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ArchiveError(f"cannot read archive index: {error}") from error
    required = {
        "archivepath", "archive_sha256", "archive_size", "codec", "originalpath",
        "original_sha256", "original_size", "seed_roles", "seeds", "status",
    }
    if not isinstance(payload, dict) or set(payload) != required:
        raise ArchiveError("archive index has an unsupported schema")
    if payload["codec"] != CODEC or payload["status"] != "verified":
        raise ArchiveError("archive index is not a verified zstd archive")
    for field in ("archive_sha256", "original_sha256"):
        if not isinstance(payload[field], str) or not re.fullmatch(r"[0-9a-f]{64}", payload[field]):
            raise ArchiveError(f"invalid {field} in archive index")
    for field in ("archive_size", "original_size"):
        if not isinstance(payload[field], int) or isinstance(payload[field], bool) or not 0 <= payload[field] <= MAX_BYTES:
            raise ArchiveError(f"invalid {field} in archive index")
    relative_original = Path(payload["originalpath"])
    relative_archive = Path(payload["archivepath"])
    if relative_original.is_absolute() or relative_archive.is_absolute() or ".." in relative_original.parts or ".." in relative_archive.parts:
        raise ArchiveError("archive index contains an unsafe path")
    original = _checked_path(root / relative_original, root, must_exist=False)
    archive = _checked_path(root / relative_archive, root, must_exist=True)
    if original.suffix != ".jsonl" or archive != original.with_name(original.name + ".zst"):
        raise ArchiveError("archive index paths do not match the expected JSONL layout")
    if index != original.with_name(original.name + ".archive.json"):
        raise ArchiveError("index filename does not match originalpath")
    return original, archive, payload


def restore_index(index_path: Path, *, root: Path = ROOT, apply: bool = False) -> dict:
    root = root.resolve(strict=True)
    original, archive, payload = _validated_index(index_path, root)
    archive_stat = archive.stat()
    if archive_stat.st_size > MAX_BYTES:
        raise ArchiveError(f"archive exceeds {MAX_BYTES} byte limit")
    if _hash_file(archive) != (payload["archive_size"], payload["archive_sha256"]):
        raise ArchiveError("archive bytes do not match archive index")
    if original.exists():
        if original.is_symlink():
            raise ArchiveError("restore destination is a symlink")
        if _hash_file(original) == (payload["original_size"], payload["original_sha256"]):
            return {**payload, "dry_run": not apply, "restored": False, "already_present": True}
        raise ArchiveError("restore destination exists with different bytes; refusing overwrite")
    if not apply:
        fd, name = tempfile.mkstemp(prefix=f".{original.name}.verify.", dir=original.parent)
        os.close(fd)
        temporary = Path(name)
        try:
            _decompress_verified(archive, temporary, payload["original_size"], payload["original_sha256"])
            _verify_seed_metadata(temporary, payload)
        finally:
            temporary.unlink(missing_ok=True)
        return {**payload, "dry_run": True, "would_restore": True}

    fd, temporary = tempfile.mkstemp(prefix=f".{original.name}.restore.", suffix=".tmp", dir=original.parent)
    temp_raw = Path(temporary)
    try:
        _decompress_verified(archive, temp_raw, payload["original_size"], payload["original_sha256"])
        _verify_seed_metadata(temp_raw, payload)
        # link() is an atomic exclusive publish: it never overwrites a destination.
        try:
            os.link(temp_raw, original)
        except FileExistsError:
            if _hash_file(original) == (payload["original_size"], payload["original_sha256"]):
                return {**payload, "dry_run": False, "restored": False, "already_present": True}
            raise ArchiveError("restore destination appeared with different bytes; refusing overwrite")
        _fsync_dir(original.parent)
        if _hash_file(original) != (payload["original_size"], payload["original_sha256"]):
            original.unlink(missing_ok=True)
            raise ArchiveError("restored file failed post-publish verification")
        return {**payload, "dry_run": False, "restored": True, "already_present": False}
    finally:
        temp_raw.unlink(missing_ok=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    archive_parser = subparsers.add_parser("archive", help="archive one JSONL evidence file")
    archive_parser.add_argument("file", type=Path)
    archive_parser.add_argument("--apply", action="store_true", help="write archive and then remove raw source")
    restore_parser = subparsers.add_parser("restore", help="restore a JSONL file from its archive index")
    restore_parser.add_argument("index", type=Path)
    restore_parser.add_argument("--apply", action="store_true", help="write the original raw file")
    args = parser.parse_args(argv)
    try:
        result = archive_file(args.file, apply=args.apply) if args.command == "archive" else restore_index(args.index, apply=args.apply)
    except (ArchiveError, OSError, subprocess.SubprocessError) as error:
        parser.exit(2, f"evidence_archive: {error}\n")
    print(json.dumps(result, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
