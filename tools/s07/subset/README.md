# S07 source subset classifier

This classifier implements the proposed version 1 feature boundary in
`docs/S07-implementation-plan.md` section 6. It reads no Rust binding, loading,
checking, timing or memory result. Every physical compiler/conformance case and
every effective Go harness variant remains in the disposition manifest.

The syntax table covers the actual pinned `ast.Kind` enum. The option table
covers every JSON field reflected from `core.CompilerOptions`. The directive
table covers the complete physical corpus, and the pragma table is checked
against the literal name guards in the Go AST of `parser.extractPragmas`.
Unclassified, missing, duplicate or reordered inventory is an error. The three
non-syntax tables currently allow or preserve all entries; adding a reject row
fails until its exact value/boundary predicate is implemented.

The observer traverses parsed children and lazily parsed JSDoc through original
Go APIs. Recovered input remains eligible unless its tree contains an excluded
feature. Nonempty type arguments are checked on every supported payload in
`Node.TypeArgumentList`; `TypeParameter` and `JSDocTemplateTag` independently
reject explicit generic parameters. Initial configuration JSON does not become
source type syntax. Library and dependency features are recorded as later
checker obligations rather than source-case exclusions.

Physical loading, virtual unit extraction, option expansion and parser metadata
use the unchanged S06 access-only bridge. Accepted compiler/harness settings call
the pinned `SetOptionsFromTestConfig`. Removed settings retain actual native
CLI diagnostics and an explicit rejected option outcome; their remaining partial
options are never reported as a successful configuration. The seven legacy
`module:none` expansion qualifications remain labeled and retain diagnostic 6046.

The fixture adapter mirrors only the original compiler-runner input assembly:
fixture defaults, path-valued option normalization, roots, symlinks, and optional
`/.lib` population. Those source anchors are stated at the corresponding code.
Actual resolution and ordered library/dependency inclusion run separately through
the original compiler loader using `scripts/s07_program.py`. No substitute lib
dependency algorithm is used. Source option/config diagnostics and loader include
diagnostics remain distinct; this capture does not claim `NewProgram` option
verification or checker diagnostics.

The emitted-output-only rule requires `NoTypesAndSymbols=true`, an exact emitted
baseline, and absence of errors/types/symbols baselines for the same effective
configured name. Each baseline records its pinned path and Git blob identity.
The source extension is removed once, preserving earlier dots in configured names.
Neither diagnostics nor unsupported Rust operations can exclude individual cases.

The commands are:

```sh
python3 scripts/s07.py observe-subset --output target/s07-subset/source-observations.ndjson
python3 scripts/s07.py subset-candidate --observations target/s07-subset/source-observations.ndjson --output target/s07-subset/review
python3 scripts/s07_program.py --requests target/s07-subset/review/loading-requests.candidate.json --output target/s07-subset/review/loading-observations.candidate.json
python3 scripts/s07.py subset-review --observations target/s07-subset/source-observations.ndjson --loader-observations target/s07-subset/review/loading-observations.candidate.json --output target/s07-subset/review
python3 scripts/s07.py freeze-subset --review path/to/independent-review.json --write-manifest
python3 scripts/s07.py freeze-subset
PYTHONPATH=scripts python3 -m unittest scripts/test_s07_subset.py
```

Use the pinned Go toolchain and preserve caller homes/cache settings. The first
command records producer hashes and the observation digest. The loader manifest
binds the exact candidate requests to the original Go source. Classification
records its own implementation/table hashes. Review is for the exact joined
candidate digests in `review-payload.json`; changing a predicate, table, source
observation, dependency closure or obligation invalidates that review.

`freeze-subset` checks by default. Only its explicit `--write-manifest` operation
writes the reviewed artifacts under `data/s07`. The independent review supplies
the pin, candidate digests, reviewer, status `accepted`, a findings list, and
`required_program_operations_reviewed=true`. This review checks the complete
required operation scope; it must not use implementation failures to change
case membership. Pending implementation remains a sprint blocker.

The checker-obligation scope is explicit: syntax found in actual loaded library
declarations, with byte/source anchors and a witness program. Generic
instantiation, conditional types and other advanced features are required when
those declarations are used in S08. The source-only capture does not claim these
checker operations executed. E2/E7/E8 share the variant and dependency identities;
later checker/experiment metrics remain absent. Mandatory ownership and checker
fixture specifications have separate identities and completion states.
