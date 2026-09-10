"""Validate an invocation receipt without turning unknown exit status into success."""
import hashlib
import re


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def validate_receipt_status(receipt):
    common = {'version', 'build_manifest_sha256', 'child_returncode', 'raw_sha256',
              'stdout_sha256', 'stderr_sha256', 'capture_tool_inputs'}
    require(type(receipt) is dict, 'receipt must be an object')
    code = receipt.get('child_returncode')
    if type(code) is int and code == 0:
        require(set(receipt) == common | {'command'}, 'normal receipt fields differ')
        command = receipt['command']
        require(type(command) is list and len(command) == 3 and
                all(type(v) is str and v for v in command), 'invalid child command receipt')
    elif code is None:
        require(set(receipt) == common | {'recovery_note'}, 'unknown exit status requires explicit recovery fields')
        require(type(receipt['recovery_note']) is str and receipt['recovery_note'].strip(),
                'unknown exit status requires a nonempty recovery note')
    else:
        raise ValueError('receipt records a failed or invalid child exit status')
    require(type(receipt['version']) is int and receipt['version'] == 1, 'invalid receipt version')
    return 'recorded_success' if code == 0 else 'documented_unknown_exit_recovery'


def validate_receipt(receipt, build_sha, raw, stdout, stderr, tool_inputs=None):
    status = validate_receipt_status(receipt)
    require(receipt['build_manifest_sha256'] == build_sha, 'receipt build identity differs')
    for name, content in [('raw_sha256', raw), ('stdout_sha256', stdout), ('stderr_sha256', stderr)]:
        require(receipt[name] == sha(content), 'receipt raw identity differs: ' + name)
    tools = receipt['capture_tool_inputs']
    require(type(tools) is dict and tools and all(type(name) is str and name and
            type(digest) is str and re.fullmatch('[0-9a-f]{64}', digest)
            for name, digest in tools.items()), 'invalid capture tool receipt')
    if tool_inputs is not None:
        require(tools == tool_inputs, 'receipt capture tool inputs differ')
    return status
