#!/usr/bin/env python3
"""Small archive recipe runner. Never captures, builds or measures a workload.

Prepared recipes deliberately fail closed until paths, anchor hashes and frozen
replay commands have been supplied. Replay executes only the specified reviewed
Python validators, after verifying every archived regular-file member.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile

from archive_helpers import identity, read, require, safe_name, sha, verify_candidate, write

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def files(root, recipe):
    found = set()
    for entry in recipe['retain']:
        require(entry['kind'] in ('file', 'tree'), 'pending input: ' + entry['role'])
        path = root / safe_name(entry['path'])
        require(path.exists() and not path.is_symlink(), 'missing/link input: ' + str(path))
        if entry['kind'] == 'file':
            require(path.is_file(), 'expected file: ' + str(path))
            found.add(path)
            continue
        require(path.is_dir(), 'expected tree: ' + str(path))
        for parent, dirs, names in os.walk(path, followlinks=False):
            dirs[:] = [name for name in dirs if name not in recipe['excluded_directory_names']
                       and not name.endswith(tuple(recipe['excluded_directory_suffixes']))]
            for name in dirs + names:
                require(not (Path(parent) / name).is_symlink(), 'unaccounted symlink')
            found.update(Path(parent) / name for name in names)
    return sorted(found)


def validate_recipe(root, recipe):
    require(recipe['version'] == 1 and recipe['status'] == 'ready' and not recipe['pending'],
            'recipe remains prepared/pending')
    require(set(row['role'] for row in recipe['replays']) == set(recipe['required_replay_roles']),
            'missing or unexpected replay obligation')
    for row in recipe['anchors']:
        expected = row['sha256']
        require(isinstance(expected, str) and len(expected) == 64, 'unfilled anchor: ' + row['role'])
        require(sha(root / safe_name(row['path'])) == expected, 'anchor changed: ' + row['role'])
    for row in recipe['bundles']:
        verify_candidate(root / safe_name(row['path']), row['manifest_sha256'])
    for row in recipe['replays']:
        require(sha(root / safe_name(row['script'])) == row['script_sha256'], 'replay helper changed')
        require(isinstance(row['args'], list) and all(isinstance(v, str) for v in row['args']), 'invalid replay args')


def pack(root, recipe_path, output):
    recipe = read(recipe_path)
    validate_recipe(root, recipe)
    require(not output.exists(), 'refuse to overwrite an archive')
    selected = files(root, recipe)
    members = {safe_name(p.relative_to(root).as_posix()): identity(p) for p in selected}
    require(all(row['script'] in members for row in recipe['replays']), 'replay helper not retained')
    output.mkdir(parents=True)
    path = output / 'review.tar.xz'
    with tarfile.open(path, 'w:xz', preset=3) as archive:
        for name in members:
            archive.add(root / name, arcname=name, recursive=False)
    require({p.relative_to(root).as_posix(): identity(p) for p in selected} == members, 'input changed during packaging')
    write(output / 'archive.json', {'version': 1, 'scope': recipe['scope'], 'recipe': recipe,
          'recipe_sha256': sha(recipe_path), 'members': members, 'archive': {'path': path.name, **identity(path)},
          'status': 'integrity_packaged_pending_replay', 'omissions': recipe['omissions']})


def replay(manifest_path, output):
    manifest = read(manifest_path)
    require(not output.exists(), 'refuse to overwrite replay output')
    archive_path = manifest_path.parent / safe_name(manifest['archive']['path'])
    require(identity(archive_path) == {k: v for k, v in manifest['archive'].items() if k != 'path'}, 'archive hash changed')
    output.mkdir(parents=True)
    root = output / 'extracted'
    members = manifest['members']; seen = set()
    with tarfile.open(archive_path, 'r:xz') as archive:
        for member in archive:
            name = safe_name(member.name)
            require(member.isfile() and name in members and name not in seen, 'unexpected archive member')
            seen.add(name); target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, target.open('xb') as destination:
                shutil.copyfileobj(source, destination)
            require(identity(target) == members[name], 'extracted member changed')
    require(seen == set(members), 'archive omitted members')
    validate_recipe(root, manifest['recipe'])
    results = []
    for index, row in enumerate(manifest['recipe']['replays']):
        destination = output / ('validator-' + str(index))
        args = [arg.replace('{root}', str(root)).replace('{output}', str(destination)) for arg in row['args']]
        command = [sys.executable, str(root / row['script']), *args]
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, check=False)
        (output / (row['role'] + '.stdout')).write_text(result.stdout)
        (output / (row['role'] + '.stderr')).write_text(result.stderr)
        results.append({'role': row['role'], 'argv': command, 'exit_code': result.returncode})
        write(output / 'replay-results.json', results)
        require(result.returncode == 0, 'retained replay failed: ' + row['role'])
    write(output / 'report.json', {'status': 'integrity_and_declared_replays_verified',
          'members': len(members), 'validators': results,
          'scope': 'No archive-layer acceptance inference; gate decisions remain in the retained measured reports.'})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    package = sub.add_parser('pack'); package.add_argument('--root', type=Path, default=ROOT)
    package.add_argument('--recipe', type=Path, default=HERE / 'recipe.json'); package.add_argument('--output', type=Path, required=True)
    check = sub.add_parser('replay'); check.add_argument('--manifest', type=Path, required=True); check.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'pack': pack(args.root.resolve(), args.recipe.resolve(), args.output.resolve())
    else: replay(args.manifest.resolve(), args.output.resolve())
