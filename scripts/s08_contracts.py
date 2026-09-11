"""Strict P0 supplemental protocol validation, independent of Rust completion."""
from s08_oracle import canonical, digest

MODES = ('identity', 'assignable', 'subtype', 'strict_subtype', 'comparable')
COUNTERS = ('types_created', 'signatures_created', 'instantiations')
FIRST_CALL_CLASSES = ('cold_lazy_resolution', 'cold_cache_evaluation',
                      'uncached_shortcut', 'primitive_identity_shortcut')


def fields(value, names):
    if type(value) is not dict or set(value) != set(names.split()):
        raise ValueError('missing or extra protocol field: ' + names)


def integer(value, minimum=0, maximum=(1 << 63) - 1):
    if type(value) is not int or not minimum <= value <= maximum:
        raise ValueError('invalid protocol integer')


def hex_bytes(value):
    if type(value) is not str or len(value) % 2 or any(c not in '0123456789abcdef' for c in value):
        raise ValueError('noncanonical byte transport')
    return bytes.fromhex(value)


def diagnostics(rows):
    if type(rows) is not list:
        raise ValueError('diagnostic array required')
    for row in rows:
        fields(row, 'file pos end code category key_hex text_hex args chain related')
        if row['file'] is not None and type(row['file']) is not str:
            raise ValueError('invalid diagnostic file')
        integer(row['pos'], -1)
        integer(row['end'], row['pos'])
        integer(row['code'], 1)
        integer(row['category'], 0, 3)
        hex_bytes(row['key_hex'])
        hex_bytes(row['text_hex'])
        if row['args'] is not None and (type(row['args']) is not list or
                                       any(type(v) is not str for v in row['args'])):
            raise ValueError('diagnostic arguments must be strings or null')
        diagnostics(row['chain'])
        diagnostics(row['related'])


def state(row):
    fields(row, 'caches instantiations signatures_created types_created')
    fields(row['caches'], ' '.join(MODES))
    for field in ('instantiations', 'signatures_created', 'types_created'):
        integer(row[field])
    for cache in row['caches'].values():
        fields(cache, 'entries result_flags')
        integer(cache['entries'])
        if type(cache['result_flags']) is not list or len(cache['result_flags']) != cache['entries']:
            raise ValueError('relation cache count mismatch')
        for flag in cache['result_flags']:
            integer(flag, 0, 15)
        if cache['result_flags'] != sorted(cache['result_flags']):
            raise ValueError('cache flags are not canonical')


def relation_start_spec(definition):
    """Validate authored classifications separately from the native action request."""
    fields(definition.get('starting_cache_entries'), ' '.join(MODES))
    fields(definition.get('first_call'), ' '.join(MODES))
    for entries in definition['starting_cache_entries'].values():
        integer(entries)
    populated = {mode for mode, entries in definition['starting_cache_entries'].items() if entries}
    expected_setup = ('type_resolution_populates_assignable_cache' if populated == {'assignable'}
                      else 'type_resolution_with_empty_caches' if not populated else None)
    if expected_setup is None or definition.get('setup') != expected_setup:
        raise ValueError('setup classification contradicts declared cache state')
    classes = definition['first_call'].values()
    if any(value not in FIRST_CALL_CLASSES for value in classes):
        raise ValueError('unknown first relation classification')
    if 'primitive_identity_shortcut' in classes:
        if (definition.get('resolved_primitive') != 'number'
                or set(classes) != {'primitive_identity_shortcut'}):
            raise ValueError('primitive identity fixture must declare number in every mode')
    elif 'resolved_primitive' in definition:
        raise ValueError('unexpected primitive identity expectation')


def relation_start(definition, group, mode):
    """Check setup and actual first-call work without assuming a cache hit or allocation."""
    first = group['actions'][0]
    before, after = first['before'], first['after']
    lookup = group['before_lookup']
    if any(cache['entries'] for cache in lookup['caches'].values()):
        raise ValueError('fresh checker relation cache is already populated')
    if any(before[key] < lookup[key] for key in COUNTERS):
        raise ValueError('native setup counter decreased')
    if {name: cache['entries'] for name, cache in before['caches'].items()} != definition['starting_cache_entries']:
        raise ValueError('relation starting cache state differs from declaration: ' + definition['id'])
    classification = definition['first_call'][mode]
    created = any(after[key] > before[key] for key in COUNTERS)
    cache_changed = before['caches'] != after['caches']
    if classification.startswith('cold_') or classification == 'uncached_shortcut':
        if any(cache['entries'] for cache in before['caches'].values()):
            raise ValueError('declared cold/uncached relation was prewarmed during setup')
    if classification == 'cold_lazy_resolution':
        if not created:
            raise ValueError('declared cold relation omitted lazy creation/resolution work')
    elif classification == 'cold_cache_evaluation':
        if created or not cache_changed or after['caches'][mode]['entries'] <= before['caches'][mode]['entries']:
            raise ValueError('declared cold cache evaluation changed its work class')
    elif classification == 'uncached_shortcut':
        if before != after:
            raise ValueError('declared uncached shortcut changed relation state')
    elif classification == 'primitive_identity_shortcut':
        # Both declarations resolve to the native number intrinsic. The unrelated
        # assignable entry created by ReturnType setup does not make this a hit.
        if (group['display_hex'] != {'A': b'number'.hex(), 'B': b'number'.hex()}
                or group['type_flags'] != {'A': 64, 'B': 64}):
            raise ValueError('primitive identity declarations no longer resolve to number')
        if any(action['before'] != action['after'] or not action['result']
               for action in group['actions']):
            raise ValueError('primitive identity shortcut changed state or outcome')


def ordering(rows):
    if type(rows) is not list or len(rows) % 12:
        raise ValueError('ordering requires reverse, ten shuffles and pairwise matrix')
    for offset in range(0, len(rows), 12):
        group = rows[offset:offset + 12]
        matrix = group[-1]
        fields(matrix, 'type pairwise')
        n = len(matrix['pairwise'])
        if not n or matrix['type'] not in ('A', 'B'):
            raise ValueError('invalid union ordering type')
        for row in matrix['pairwise']:
            if type(row) is not list or len(row) != n:
                raise ValueError('incomplete comparator matrix')
            for value in row:
                integer(value, -(1 << 63))
        for row in group[:-1]:
            fields(row, 'type input sorted')
            if row['type'] != matrix['type']:
                raise ValueError('ordering changed its type')
            for key in ('input', 'sorted'):
                for value in row[key]:
                    integer(value)
                if sorted(row[key]) != list(range(n)):
                    raise ValueError('missing/extra ordering element')
            if any(matrix['pairwise'][a][b] > 0 for a, b in zip(row['sorted'], row['sorted'][1:])):
                raise ValueError('native sorted list contradicts its comparator')
        if group[0]['input'] != list(reversed(range(n))):
            raise ValueError('missing reverse permutation')


def validate(spec, request, text_spec, residual_spec, observed):
    fields(observed, 'request_sha256 go goos goarch rows supplemental')
    if observed['request_sha256'] != digest(canonical(request) + b'\n'):
        raise ValueError('native fixture request identity mismatch')
    if [r['id'] for r in observed['rows']] != [r['id'] for r in request]:
        raise ValueError('missing, extra or reordered native fixture')
    for definition, case, actual in zip(spec['cases'], request, observed['rows'], strict=True):
        relation_start_spec(definition)
        fields(actual, 'id state groups diagnostics global_diagnostics')
        if actual['state'] != 'executed' or len(actual['groups']) != len(MODES):
            raise ValueError('unexecuted native fixture or missing relation mode')
        diagnostics(actual['diagnostics'])
        diagnostics(actual['global_diagnostics'])
        if sorted(d['code'] for d in actual['diagnostics']) != definition['expected_diagnostic_codes'] or actual['global_diagnostics']:
            raise ValueError('unexpected fixture diagnostics: ' + case['id'])
        for mode_index, group in enumerate(actual['groups']):
            fields(group, 'before_lookup actions display_hex in_alias_display_hex type_flags union_ordering final')
            state(group['before_lookup'])
            state(group['final'])
            fields(group['display_hex'], 'A B')
            fields(group['in_alias_display_hex'], 'A B')
            fields(group['type_flags'], 'A B')
            for name in ('A', 'B'):
                hex_bytes(group['display_hex'][name])
                hex_bytes(group['in_alias_display_hex'][name])
                integer(group['type_flags'][name], 0, (1 << 32) - 1)
            ordering(group['union_ordering'])
            expected_actions = case['actions'][mode_index * 4:(mode_index + 1) * 4]
            if canonical([r['action'] for r in group['actions']]) != canonical(expected_actions):
                raise ValueError('missing/reordered relation actions')
            for index, action in enumerate(group['actions']):
                fields(action, 'action before after result ternary_calls diagnostics')
                state(action['before'])
                state(action['after'])
                diagnostics(action['diagnostics'])
                if type(action['result']) is not bool or not action['ternary_calls']:
                    raise ValueError('unobserved native relation result')
                for ternary in action['ternary_calls']:
                    integer(ternary, -1, 3)
                    if ternary not in (-1, 0, 1, 3):
                        raise ValueError('invalid native ternary')
                if action['result'] != (action['ternary_calls'][-1] != 0):
                    raise ValueError('relation result contradicts native ternary')
                if index and action['before'] != group['actions'][index - 1]['after']:
                    raise ValueError('broken cold/repeat state sequence')
                for counter in COUNTERS:
                    if action['after'][counter] < action['before'][counter]:
                        raise ValueError('native cumulative counter decreased')
            relation_start(definition, group, MODES[mode_index])
    fields(observed['supplemental'], 'residuals text')
    residuals = observed['supplemental']['residuals']
    if set(residuals) != set(residual_spec['cases']):
        raise ValueError('missing residual comparator observation')
    if residuals['foreign-checker'] != {'state': 'panic', 'message': 'Cannot compare types from different checkers'}:
        raise ValueError('foreign-checker comparator contract changed')
    for key, value in residuals.items():
        if key == 'foreign-checker':
            continue
        def ints(xs):
            if type(xs) is not list or not xs:
                raise ValueError('incomplete residual comparison')
            for v in xs:
                if type(v) is list:
                    ints(v)
                else:
                    integer(v, -(1 << 63))
        ints(value)
    text = observed['supplemental']['text']
    if [r['id'] for r in text] != [r['id'] for r in text_spec['cases']]:
        raise ValueError('missing text observation')
    for row in text:
        fields(row, 'id original_hex synthetic_hex type_default_hex type_full_hex type_short_hex truncated parse_diagnostics')
        for key in ('original_hex', 'synthetic_hex', 'type_default_hex', 'type_full_hex', 'type_short_hex'):
            hex_bytes(row[key])
        if type(row['truncated']) is not bool:
            raise ValueError('invalid truncation outcome')
        diagnostics(row['parse_diagnostics'])
        if row['parse_diagnostics']:
            raise ValueError('unexpected text fixture parse error')


def compare(expected, actual):
    """Compare semantic observations exactly; host metadata is provenance only."""
    for key in ('request_sha256', 'rows', 'supplemental'):
        if canonical(expected[key]) != canonical(actual[key]):
            raise ValueError('native supplemental contract drift in ' + key)
