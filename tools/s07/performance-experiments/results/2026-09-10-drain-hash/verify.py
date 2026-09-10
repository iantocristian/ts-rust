#!/usr/bin/env python3
"""Reassemble the exact split archive and run its unchanged offline replay."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
PART_LIMIT = 48 * 1024 * 1024
BOOTSTRAP = (
    "target/s07-bis/drain-hash/archive/archive.py",
    "target/s07-bis/drain-hash/archive/archive_helpers.py",
)


def require(value, message):
    if not value:
        raise ValueError(message)


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON key: " + key)
            result[key] = value
        return result
    return json.loads(path.read_bytes(), object_pairs_hook=unique)


def local_file(directory, name):
    require(isinstance(name, str) and name and Path(name).name == name
            and name not in (".", ".."), "invalid distribution filename")
    path = directory / name
    require(path.is_file() and not path.is_symlink(), "missing or linked file: " + name)
    return path


def verify_identity(path, record):
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    require(path.stat().st_size == record["bytes"] and digest == record["sha256"],
            "identity changed: " + path.name)


def verify(output):
    require(not output.exists(), "replay output must be a fresh path")
    distribution = read_json(HERE / "distribution.json")
    require(distribution["version"] == 1 and distribution["part_limit_bytes"] == PART_LIMIT,
            "unsupported distribution")
    manifest_ref = distribution["archive_manifest"]
    manifest_path = local_file(HERE, manifest_ref["path"])
    verify_identity(manifest_path, manifest_ref)
    manifest = read_json(manifest_path)
    archive_ref = distribution["archive"]
    require(archive_ref == manifest["archive"], "archive and distribution disagree")
    require(archive_ref["path"] == "review.tar.xz", "unexpected archive filename")
    parts = distribution["parts"]
    require(isinstance(parts, list) and parts and
            [p["path"] for p in parts] ==
            [f'review.tar.xz.part-{index:03}' for index in range(1, len(parts) + 1)],
            "part order or names changed")
    require(all(type(p["bytes"]) is int and 0 < p["bytes"] <= PART_LIMIT for p in parts)
            and sum(p["bytes"] for p in parts) == archive_ref["bytes"], "invalid part sizes")
    with TemporaryDirectory(prefix="s07-bis-archive-") as temporary:
        staging = Path(temporary)
        assembled = staging / archive_ref["path"]
        total_hash = hashlib.sha256()
        with assembled.open("xb") as destination:
            for part in parts:
                source = local_file(HERE, part["path"])
                part_hash = hashlib.sha256(); size = 0
                with source.open("rb") as stream:
                    while chunk := stream.read(1024 * 1024):
                        destination.write(chunk); total_hash.update(chunk)
                        part_hash.update(chunk); size += len(chunk)
                require(size == part["bytes"] and part_hash.hexdigest() == part["sha256"],
                        "archive part changed: " + part["path"])
        require(assembled.stat().st_size == archive_ref["bytes"]
                and total_hash.hexdigest() == archive_ref["sha256"], "assembled archive changed")
        copied_manifest = staging / "archive.json"
        copied_manifest.write_bytes(manifest_path.read_bytes())
        verify_identity(copied_manifest, manifest_ref)
        bootstrap = staging / "bootstrap"
        bootstrap.mkdir()
        seen = set()
        with tarfile.open(assembled, "r:xz") as archive:
            for member in archive:
                if member.name not in BOOTSTRAP:
                    continue
                require(member.name not in seen and member.isfile(),
                        "duplicate or non-regular replay bootstrap")
                seen.add(member.name)
                expected = manifest["members"][member.name]
                require(member.size == expected["bytes"], "bootstrap size changed")
                with archive.extractfile(member) as source:
                    raw = source.read()
                require(len(raw) == expected["bytes"]
                        and hashlib.sha256(raw).hexdigest() == expected["sha256"],
                        "bootstrap bytes changed")
                (bootstrap / Path(member.name).name).write_bytes(raw)
        require(seen == set(BOOTSTRAP), "replay bootstrap is missing")
        return subprocess.run([sys.executable, str(bootstrap / "archive.py"), "replay",
                               "--manifest", str(copied_manifest), "--output", str(output)],
                              check=False).returncode


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True,
                        help="Fresh directory for the unchanged archive replay and all five validators")
    args = parser.parse_args()
    raise SystemExit(verify(args.output.resolve()))
