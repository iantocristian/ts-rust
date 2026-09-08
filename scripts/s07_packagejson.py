#!/usr/bin/env python3
"""Observe unchanged packagejson.Parse and field readers at the Go pin."""
import argparse
import hashlib
import json
import shutil
import sys
from s04_common import command, strict_json_loads
from s06_build import ROOT, oracle_export


QUALIFICATION = {
    'id': 'go-json-semantic-error-modal-verb',
    'module': 'github.com/go-json-experiment/json',
    'version': 'v0.0.0-20260623181947-01eb4420fa68',
    'source': 'errors.go:errorModalVerb (lines 322-333)',
    'source_test': 'errors_test.go:TestSemanticError (lines 107-111)',
    'alternate_prefix': 'json: unable to ',
    'canonical_prefix': 'json: cannot ',
    'scope': 'Only the top-level error string of a parseable=false observation; all other bytes remain exact.',
}


def normalize_error_prefixes(raw):
    """Change only source-proven modal-verb prefixes, preserving every other byte.

    Decode member boundaries rather than replacing arbitrary JSON substrings:
    dependency names/values and nested package fields can also contain "error".
    The pinned Go observer emits these ASCII prefixes without escape sequences.
    """
    rows = strict_json_loads(raw)
    if (type(rows) is not list or any(type(row) is not dict
            or type(row.get('error')) is not str or type(row.get('parseable')) is not bool
            or type(row.get('id')) is not str for row in rows)):
        raise ValueError('invalid package JSON observation envelope')
    text = raw.decode('utf-8')
    decoder = json.JSONDecoder()
    pos = 0
    replacements = []
    qualified = []

    def whitespace(index):
        while index < len(text) and text[index] in ' \t\r\n':
            index += 1
        return index

    pos = whitespace(pos) + 1  # opening array; strict decoding checked syntax
    for index, row in enumerate(rows):
        pos = whitespace(pos) + 1  # opening object
        while True:
            pos = whitespace(pos)
            if text[pos] == '}':
                pos += 1
                break
            key, pos = decoder.raw_decode(text, pos)
            pos = whitespace(pos) + 1  # colon
            start = whitespace(pos)
            value, pos = decoder.raw_decode(text, start)
            prefix = QUALIFICATION['alternate_prefix']
            if (key == 'error' and row['parseable'] is False
                    and type(value) is str and value.startswith(prefix)
                    and text.startswith('"' + prefix, start)):
                replacements.append((start + 1, start + 1 + len(prefix)))
                qualified.append({'id': row['id'], 'path': f'/{index}/error',
                                  'raw': value,
                                  'normalized': QUALIFICATION['canonical_prefix'] + value[len(prefix):]})
            pos = whitespace(pos)
            if text[pos] == ',':
                pos += 1
        pos = whitespace(pos)
        if text[pos] == ',':
            pos += 1
    for start, end in reversed(replacements):
        text = text[:start] + QUALIFICATION['canonical_prefix'] + text[end:]
    return text.encode('utf-8'), qualified


def compare_observations(frozen, fresh):
    expected, frozen_qualified = normalize_error_prefixes(frozen)
    actual, fresh_qualified = normalize_error_prefixes(fresh)
    def diagnostics(raw):
        return [{'id': row['id'], 'error': row['error']}
                for row in strict_json_loads(raw) if row['error']]
    return {
        'schema': 1, 'operation': 'packagejson_source_check',
        'qualification': QUALIFICATION,
        'frozen_sha256': hashlib.sha256(frozen).hexdigest(),
        'fresh_sha256': hashlib.sha256(fresh).hexdigest(),
        'normalized_frozen_sha256': hashlib.sha256(expected).hexdigest(),
        'normalized_fresh_sha256': hashlib.sha256(actual).hexdigest(),
        'raw_equal': frozen == fresh, 'observations_equal': expected == actual,
        'normalized_errors': {'frozen': frozen_qualified, 'fresh': fresh_qualified},
        'raw_diagnostics': {'frozen': diagnostics(frozen), 'fresh': diagnostics(fresh)},
    }


def retain_raw_output(root, fresh):
    folder = root / 'target/s07-packagejson'
    folder.mkdir(parents=True, exist_ok=True)
    raw_path = folder / (hashlib.sha256(fresh).hexdigest() + '.json')
    raw_path.write_bytes(fresh)  # survive oracle_export cleanup, including failures
    return raw_path


def publish_check(root, fresh, manifest, *, write=False):
    raw_path = retain_raw_output(root, fresh)
    folder = raw_path.parent
    observations = root / 'data/s07/packagejson-observations.json'
    manifest_path = root / 'data/s07/packagejson-manifest.json'
    expected_manifest = (json.dumps(manifest, sort_keys=True, separators=(',', ':')) + '\n').encode()
    if write:
        observations.write_bytes(normalize_error_prefixes(fresh)[0])
        manifest_path.write_bytes(expected_manifest)
    frozen = observations.read_bytes()
    report = compare_observations(frozen, fresh)
    report.update(pin=manifest['pin'], requests=manifest['requests'],
                  fresh_artifact=str(raw_path.resolve()),
                  manifest_sha256=hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
                  manifest_current=manifest_path.read_bytes() == expected_manifest)
    (folder / 'comparison.json').write_text(json.dumps(report, sort_keys=True) + '\n')
    print(json.dumps(report, sort_keys=True), flush=True)
    if not report['observations_equal']:
        raise ValueError('pinned package JSON evidence changed beyond qualified prefix: packagejson-observations.json')
    if not report['manifest_current']:
        raise ValueError('pinned package JSON evidence changed: packagejson-manifest.json')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    adapter = ROOT / 'tools/s07/packagejson/export_test.go'
    requests = ROOT / 'tools/s07/packagejson/requests.json'
    rows = strict_json_loads(requests.read_bytes())
    ids = [r['id'] for r in rows]
    if len(ids) != len(set(ids)) or any(not isinstance(v, str) or not v for v in ids):
        raise ValueError('invalid or duplicate request identities')
    for row in rows:
        value = row['hex']
        if not isinstance(value, str) or len(value) % 2 or any(c not in '0123456789abcdef' for c in value):
            raise ValueError('malformed request hex')
    with oracle_export() as (checkout, env, pin):
        package = checkout / 'tsc/internal/packagejson'
        source_hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(package.glob('*.go'))}
        shutil.copyfile(adapter, package / 's07_export_test.go')
        output = checkout / 'packagejson.json'
        env.update(S07_PACKAGEJSON_REQUESTS=str(requests), S07_PACKAGEJSON_OUTPUT=str(output))
        repo_path = f'-gcflags=github.com/microsoft/TypeScript/tsc/internal/repo=-trimpath={checkout}/s07-unmatched-prefix'
        command(['go', 'test', '-trimpath', '-mod=readonly', repo_path, './internal/packagejson', '-count=1'], cwd=checkout / 'tsc', env=env)
        data = output.read_bytes()
        retain_raw_output(ROOT, data)
        if [r['id'] for r in strict_json_loads(data)] != ids:
            raise ValueError('missing, extra, or reordered observation rows')
    normalized = normalize_error_prefixes(data)[0]
    manifest = {'version':2,'pin':pin,'source_sha256':source_hashes,'inputs_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [adapter, requests, ROOT/'scripts/s07_packagejson.py', ROOT/'scripts/s04.py', ROOT/'scripts/s04_common.py', ROOT/'scripts/s06_build.py', ROOT/'data/upstream.json', ROOT/'data/s04/toolchains.toml', ROOT/'upstream/tsc/go.mod', ROOT/'upstream/tsc/go.sum']},'observations_sha256':hashlib.sha256(normalized).hexdigest(),'requests':len(rows),'qualification':QUALIFICATION}
    publish_check(ROOT, data, manifest, write=args.write)
    print(f'{len(rows)} direct Go package JSON observations', file=sys.stderr)


if __name__ == '__main__':
    main()
