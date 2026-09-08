import io
from pathlib import Path
import tarfile
import tempfile
import unittest

import replay_lists as replay


class ArchiveTests(unittest.TestCase):
    def test_member_inventory_requires_every_nonbinary_source_and_raw_stream(self):
        build = {"artifacts": {"normal": {"path": "artifacts/normal"}, "allocation": {"path": "artifacts/allocation"}},
            "inventory": {"source/main.rs": replay.identity(b"source"), "artifacts/normal": replay.identity(b"normal"), "artifacts/allocation": replay.identity(b"allocation")}}
        capture = {"inventory": {"input.ndjson": replay.identity(b"shared input"), "raw/000.stdout.gz": replay.identity(b"stdout"), "raw/000.stderr.gz": replay.identity(b"stderr")}}
        members = {"build/manifest.json": b"build", "capture/manifest.json": b"capture", "build/source/main.rs": b"source", "capture/raw/000.stdout.gz": b"stdout", "capture/raw/000.stderr.gz": b"stderr"}
        self.assertEqual(replay.validate_members(members, build, capture), {"artifacts/normal", "artifacts/allocation"})
        for name in ("build/source/main.rs", "capture/raw/000.stdout.gz", "capture/raw/000.stderr.gz"):
            changed = dict(members)
            del changed[name]
            with self.subTest(name=name), self.assertRaises(ValueError):
                replay.validate_members(changed, build, capture)
        with self.assertRaises(ValueError):
            replay.validate_members({**members, "capture/raw/000.stdout.gz": b"changed"}, build, capture)
        with self.assertRaises(ValueError):
            replay.validate_members({**members, "extra": b"unrecorded"}, build, capture)

    def test_archive_parser_rejects_duplicate_links_and_traversal(self):
        for names, symlink in ((["file", "file"], False), (["../escape"], False), (["/absolute"], False), (["link"], True)):
            with tempfile.TemporaryDirectory() as temporary:
                path = Path(temporary) / "artifact.tar.xz"
                with tarfile.open(path, "w:xz") as archive:
                    for name in names:
                        member = tarfile.TarInfo(name)
                        if symlink:
                            member.type = tarfile.SYMTYPE
                            member.linkname = "elsewhere"
                            archive.addfile(member)
                        else:
                            member.size = 1
                            archive.addfile(member, io.BytesIO(b"x"))
                with self.subTest(names=names), self.assertRaises(ValueError):
                    replay.read_members(path)


if __name__ == "__main__":
    unittest.main()
