# Experiment evidence JSONL archives

`scripts/evidence_archive.py` stores one selected `data/experiments/**/*.jsonl`
file as a deterministic Zstandard stream beside its original name. The helper
is deliberately per-file; it has no recursive or batch operation. It never
changes a manifest, protocol, frozen source, SQLite file, or run identity.

Run an inspection first:

```sh
python3 scripts/evidence_archive.py archive data/experiments/example/raw.jsonl
```

The default is dry-run. To write and verify the archive, then remove that one
raw JSONL, pass `--apply`:

```sh
python3 scripts/evidence_archive.py archive data/experiments/example/raw.jsonl --apply
```

This creates `raw.jsonl.zst` and `raw.jsonl.archive.json`. The JSON index keeps
the original and compressed sizes and SHA-256 digests, codec, verified status,
and all declared seed values and roles recognized by the established seed
scanner's scalar, array, and CLI patterns. Keeping those seed declarations in
the ordinary JSON index lets the seed scanner continue to see an archived
result while ignoring the compressed stream. Compression uses the fixed zstd
level 3 and one thread: identical input with the same zstd implementation and
settings produces the same stream, without spending high-level compression CPU.
Each operation is limited to 1 GiB per file, a 64 MiB seed-scan line, and a
120-second zstd process.
Seed metadata is capped at 128 roles and 100,000 distinct role/value pairs; an
input above a cap is rejected before the raw file can be removed. The `zstd`
command line tool must be installed.

Before removing a raw source, the helper checks its size and hash, streams a
compressed temporary file, fully decompresses and verifies that file, publishes
the archive and index without replacing different existing files, verifies the
published pair, then checks the source fingerprint and bytes again. A failed
check leaves the raw source in place. Repeating archive on an already archived
path verifies the existing archive and index and is idempotent.

Restore using the index path. This also defaults to dry-run:

```sh
python3 scripts/evidence_archive.py restore data/experiments/example/raw.jsonl.archive.json --apply
```

Restore verifies both stored digests and the decompressed original digest before
an exclusive atomic publish. An existing raw file with the same digest is
accepted as already restored; a different file is never overwritten. Paths
must stay under this checkout's `data/experiments` tree, and symlinked paths
and traversal in an index are rejected. The helper does not remove compressed
archives or indexes during restore.

Frozen analyzers and older runners may expect the original `raw.jsonl` path.
Restore the evidence first, then invoke those tools. The archive index is
storage metadata only: it does not create a run, change protocol identity, or
replace the original bytes. No real experiment data should be used for tests;
the test suite creates a temporary allowlisted root and uses a copied fixture.
