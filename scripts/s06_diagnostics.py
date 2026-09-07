"""Two approved diagnostic qualifications; this is not diagnostic-byte parity.

S06 implementation plan section 10 records forty independent Go observations
for each request. Only the first parse diagnostic's argument can vary. Rust
retains the S05 lexical candidate order; all other data remains exact.
"""
from s06_protocol import parser_request

_RULES = {}
for name, source, rust, other in (
    ('constructor', b'constructorabcdefghij x;', b'const ructorabcdefghij', b'constructor abcdefghij'),
    ('typeof', b'typeofabcdefghijklmno x;', b'type ofabcdefghijklmno', b'typeof abcdefghijklmno'),
):
    identifier = 'parser/text/split-keyword-' + name
    _RULES[identifier] = {
        'request': parser_request(identifier, None, source, '/s06/fixture.ts', '/s06/fixture.ts'),
        'rust': [rust.hex()],
        'oracle': ([rust.hex()], [other.hex()]),
        'stable': {
            'collection': 'parse', 'index': 0, 'code': 1435, 'category': 1,
            'pos': 0, 'end': len(source.split(b' ')[0]),
            'key_hex': b'Unknown_keyword_or_identifier_Did_you_mean_0_1435'.hex(),
            'text_hex': '',
        },
    }


def compare_diagnostic(request, stage, pair):
    """Return a measured rule result, or None for ordinary exact comparison.

    Even equal arguments fail this rule if Rust chose the other Go variant:
    that would violate the approved deterministic Rust behavior. The complete
    request is bound here, including source bytes, options, path, and operations.
    """
    rule = _RULES.get(request['id'])
    if stage != 'parse' or rule is None or request != rule['request']:
        return None
    if not any(record is not None and record['kind'] == 'diagnostic'
               and record['value']['collection'] == 'parse' and record['value']['index'] == 0
               for record in pair):
        return None
    values = [record['value'] if record is not None and record['kind'] == 'diagnostic'
              else None for record in pair]
    stable = all(value is not None and {key: item for key, item in value.items() if key != 'args_hex'} == rule['stable']
                 for value in values)
    arguments = [None if value is None else value['args_hex'] for value in values]
    passed = stable and arguments[0] in rule['oracle'] and arguments[1] == rule['rust']
    return {'pass': passed, 'rule': request['id'], 'collection': 'parse', 'index': 0,
            'oracle_args_hex': arguments[0], 'rust_args_hex': arguments[1],
            'raw_exact': pair[0] == pair[1]}
