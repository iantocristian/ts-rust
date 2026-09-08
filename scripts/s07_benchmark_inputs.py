"""Bind every native preload to ordered frozen bytes and parse options."""
import hashlib
import struct

DOMAIN = b'S07-loaded-inputs-v1\0'


def loaded_input_digest(recipes):
    digest = hashlib.sha256(DOMAIN)
    digest.update(struct.pack('>Q', len(recipes)))
    for row in recipes:
        for name in ('filename', 'path'):
            value = row[name].encode('utf-8')
            digest.update(struct.pack('>Q', len(value)))
            digest.update(value)
        if type(row['script_kind']) is not int or not -(2**31) <= row['script_kind'] < 2**31:
            raise ValueError('invalid loaded script kind')
        if any(type(row[key]) is not bool for key in ('jsx', 'force')):
            raise ValueError('invalid loaded parse option')
        if type(row['source_bytes']) is not int or not 0 <= row['source_bytes'] < 2**64:
            raise ValueError('invalid loaded source length')
        source_hash = row['source_sha256']
        if type(source_hash) is not str or len(source_hash) != 64 or any(c not in '0123456789abcdef' for c in source_hash):
            raise ValueError('invalid loaded source digest')
        digest.update(struct.pack('>i??Q', row['script_kind'], row['jsx'], row['force'], row['source_bytes']))
        digest.update(bytes.fromhex(source_hash))
    return digest.hexdigest()
