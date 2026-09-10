"""Unchanged pure helpers from the retained v2 package; no old selection/acceptance code."""
import hashlib
import json
from pathlib import Path, PurePosixPath

def require(condition, message):
    if not condition:
        raise ValueError(message)

def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()

def read(path):
    return json.loads(Path(path).read_text())

def write(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n")

def identity(path):
    return {"bytes": Path(path).stat().st_size, "sha256": sha(path)}

def safe_name(name):
    path = PurePosixPath(name)
    require(name and not path.is_absolute() and ".." not in path.parts
            and path.as_posix() == name, "unsafe member: " + name)
    return name

def verify_candidate(directory, expected):
    require(sha(directory / "manifest.json") == expected, "candidate manifest changed")
    manifest = read(directory / "manifest.json")
    for name, record in manifest["inventory"].items():
        safe_name(name)
        if not name.startswith("workload/"):
            require(identity(directory / name) == record, "candidate member changed: " + name)
    files = manifest["source_fingerprint"]["files"]
    canonical = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    require(hashlib.sha256(canonical).hexdigest() == manifest["source_fingerprint"]["sha256"],
            "source fingerprint changed")
    for name, digest in files.items():
        require(sha(directory / "source" / name) == digest, "frozen source changed")
    for artifact in manifest["artifacts"].values():
        require(sha(directory / artifact["path"]) == artifact["sha256"], "binary changed")
    return manifest
