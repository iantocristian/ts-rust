"""Fresh pinned S05 oracle exports; reusable compiler caches remain external."""

import hashlib
import io
from pathlib import Path
import shutil
import sys
import tarfile
import tempfile

from s04 import go_environment, verified_upstream
from s04_common import command, strict_json_loads

ROOT = Path(__file__).resolve().parents[1]


def build_oracle():
    upstream = verified_upstream()
    pin = strict_json_loads((ROOT / "data/upstream.json").read_bytes())["pin"]
    env = go_environment()
    destination = ROOT / "target/s05-oracle"
    destination.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="export-", dir=destination) as temporary:
        checkout = Path(temporary)
        archive = command(["git", "archive", pin, "tsc/go.mod", "tsc/go.sum", "tsc/internal"], cwd=upstream)
        try:
            with tarfile.open(fileobj=io.BytesIO(archive)) as stream:
                stream.extractall(checkout, filter="data")
        except tarfile.TarError as error:
            raise RuntimeError(f"cannot unpack pinned S05 source: {error}") from error
        for source in (ROOT / "scripts/s05_oracle").glob("*.go"):
            if source.name == "scanner_bridge.go":
                target = checkout / "tsc/internal/scanner/s05_bridge.go"
            elif source.name == "identifier_bridge.go":
                target = checkout / "tsc/internal/stringutil/s05_bridge.go"
            else:
                target = checkout / "tsc/internal/vfs/s05oracle" / source.name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        shutil.copyfile(ROOT / "scripts/s04_oracle/decode_bridge.go", checkout / "tsc/internal/vfs/internal/s04_bridge.go")
        executable = destination / "oracle"
        sys.stderr.buffer.write(command(["go", "test", "-trimpath", "-mod=readonly", "./internal/vfs/s05oracle"], cwd=checkout / "tsc", env=env))
        command(["go", "build", "-trimpath", "-mod=readonly", "-o", str(executable), "./internal/vfs/s05oracle"], cwd=checkout / "tsc", env=env)
    print(f"S05 Go oracle sha256={hashlib.sha256(executable.read_bytes()).hexdigest()}", file=sys.stderr)
    return executable, env
