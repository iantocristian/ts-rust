# Pinned package JSON observations

`python3 scripts/s07_packagejson.py` checks fresh observations from unchanged Go
`packagejson.Parse`, Expected fields, JSONValue, and ExportsOrImports readers.
`--write` explicitly replaces reviewed observations. The producer exports the
pinned compiler, executes the original package tests, and fingerprints its
inputs, package sources, requests and observations.

The 243 requests contain 79 explicit boundary cases and all 164 additional
unique package bodies from the Go-selected eligible loader requests (SHA-256
`0d6c163243a19a8ff8092f9af936f84fc34b23004ee126bb4d46115f4b0e7792`). Each corpus
body retains its original case/path and content-derived identity. No Rust result
selects the request inventory.

The Rust test compares parseability, all Expected presence/validity/null/type
and partial-value states, content mapper fields, untyped field values, object
key order, object classification, and falsiness. The source resolver discards
Parse's error text, so it is retained for diagnosis but not asserted as a Rust
public error contract. Syntax errors yield zero fields and remain distinguishable
from a missing package via `ParsedPackageJson::parseable` and the resolver cache.

The source check qualifies exactly one demonstrated nondeterministic rendering:
the pinned `go-json-experiment/json` dependency chooses `json: cannot ` or
`json: unable to ` once per process in `errors.go:errorModalVerb` (lines 322–333).
Its own `errors_test.go` normalizes this prefix at lines 107–111. We apply the
same replacement only to the top-level error of a failed parse. The remaining
message body, fields, object order, and all other serialized bytes stay exact.
Changed bodies and unknown prefixes still fail; manifest/source drift also fails.

Every check preserves its original Go output under `target/s07-packagejson/`
using the raw SHA-256 as its filename. `comparison.json` and the program helper
report retain the qualification, both raw diagnostic sets, affected row paths,
raw/normalized hashes and the manifest result. Frozen observations use the
canonical prefix; a default check never rewrites them.
