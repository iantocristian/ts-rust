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
