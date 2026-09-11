"""Join the typed closure to explicit reviewed homes and immutable S07 obligations."""
from collections import Counter

from s04_common import strict_json_loads
from s08_inventory import sites
from s08_oracle import ROOT, canonical, digest

REVIEW = 'tools/s08/inventory/review.json'


def boundary(site):
    file, signature, value = site['file'], site['signature'], site['value']
    if 'reflect.' in signature or 'reflect.' in value:
        return 'reflection_runtime'
    if (file.startswith('internal/collections/') or 'iter.Seq[' in value
            or value.startswith(('maps.Keys[','maps.Values['))
            or ('MultiMap[' in value and ').Values(' in value)):
        return 'iterator_continuation'
    if file == 'internal/compiler/program.go' and 'CheckerPool' in signature:
        return 'checker_pool_factory'
    if file == 'internal/ast/ast.go' and signature.endswith(' T'):
        return 'encoder_generic_template'
    if file.startswith('internal/sourcemap/'):
        return 'source_map_host'
    if 'ExtendedConfigCacheEntry' in signature:
        return 'extended_config_cache'
    if file == 'internal/diagnostics/diagnostics.go' and 'language.' in signature:
        return 'locale_matcher'
    if file.startswith(('internal/contentmapper/', 'internal/ipc/')) and ('ctx' not in value and 'context.Context' not in value):
        return 'mapper_process_runtime'
    if signature in ('func() error', 'func() <-chan struct{}') and file.startswith(('internal/checker/', 'internal/compiler/', 'internal/execute/', 'internal/ipc/')):
        return 'context_runtime'
    raise ValueError('unreviewed unresolved callback: ' + str(site))


def prepare(closure):
    review = strict_json_loads((ROOT / REVIEW).read_bytes())
    subset = strict_json_loads((ROOT / 'data/s07/subset.json').read_bytes())
    partition = strict_json_loads((ROOT / 'data/s07/e2-acceptance.json').read_bytes())
    original = strict_json_loads((ROOT / 'data/s07/checker-obligations.json').read_bytes())
    tiers = {r['id']: r['tier'] for r in partition['variants']}
    libraries = {}
    for case in subset['cases']:
        for variant in case['variants']:
            if tiers.get(variant['id']) != 'acceptance':
                continue
            for index in variant['dependency_closure']:
                file = subset['file_observations'][index]
                libraries.setdefault((file['Name'], file['SHA256']), variant['id'])
    definitions = {r['id']: r for r in closure['functions']}
    selectors = {r['selector'] for r in closure['functions']}
    obligations = []
    fixture_ids = {r['id'] for r in strict_json_loads((ROOT / 'tools/s08/contracts/relations.json').read_bytes())['cases']}
    for row in original['obligations']:
        family = row['family']
        home, fixtures = review['families'][family]
        if not set(fixtures) <= fixture_ids:
            raise ValueError('unknown supplemental obligation fixture')
        selector = 'internal/checker::Checker.' + row['pinned_source_anchor']['function']
        if selector not in selectors:
            raise ValueError('obligation source anchor absent from typed closure: ' + selector)
        obligations.append(dict(id=row['id'], family=family, native_selector=selector,
                                home='crates/ts_checker: ' + home,
                                original_witness_tier=tiers[row['input_witness']],
                                acceptance_library_witness=libraries.get((row['library'], row['source_sha256'])),
                                supplemental_fixtures=fixtures,
                                status='pending_semantic_port',
                                test_status='Go supplemental contract observed; Rust algorithm pending'))
    source_homes = []
    for file in sorted({r['file'] for r in definitions.values()}):
        if not file:
            home, status = 'enclosing generic/function wrapper', 'synthetic_SSA_body'
        elif file.startswith('internal/'):
            home, status = review['packages'][file.split('/')[1]]
        else:
            raise ValueError('unmapped project source: ' + file)
        source_homes.append(dict(file=file, home=home, status=status))
    unresolved = []
    for site in sites(closure):
        if site['targets']:
            continue
        category = boundary(site)
        unresolved.append(dict(site=site['index'], identity_sha256=digest(canonical(site)),
                               boundary=category, resolution=review['callback_boundaries'][category]))
    interfaces = []
    for interface, members in closure['interfaces'].items():
        for member in members:
            name = member['name']
            if name in review['project_reference_members']:
                status = 'guarded_project_reference_boundary'
                contract = 'Not selected by acceptance; reject unsupported real use explicitly, never supply a false empty answer.'
            elif name == 'BindSourceFiles':
                status = 'bound_input_precondition'
                contract = 'Complete bound roots before checker creation; checker does not mutate binder state.'
            elif interface.startswith('internal/checker::'):
                status = 'trait_declared_compiler_adapter_pending' if name in review['trait_members'] else 'trait_and_adapter_pending'
                contract = 'Port native host semantics over frozen program inputs; no default value stands in for an implementation.'
            elif interface.endswith('SymbolTracker'):
                status = 'trait_declared_implementation_pending'
                contract = 'Implement balanced fallback-node stack, symbol accessibility and diagnostic callbacks alongside the real node builder.'
            else:
                status = 'declaration_diagnostic_path_pending'
                contract = 'Audit actual declaration-transform use at P5. Required callbacks execute; output-only callbacks retain an explicit phase guard. No successful no-op stand-in.'
            implementations = sorted({r['selector'] for r in definitions.values() if r['selector'].endswith('.' + name)})
            interfaces.append(dict(interface=interface, member=name, source=member,
                                   same_named_native_methods=implementations, status=status, contract=contract))
    return dict(version=1, pin=subset['pin'], scope=review['scope'],
                sources={p: digest((ROOT / p).read_bytes()) for p in (REVIEW, 'scripts/s08_audit.py',
                    'data/s07/checker-obligations.json', 'data/s07/e2-acceptance.json',
                    'tools/s08/contracts/relations.json', 'crates/ts_checker/src/host.rs')},
                closure_sha256=digest(canonical(closure) + b'\n'),
                counts=dict(obligations=len(obligations), families=dict(Counter(r['family'] for r in obligations)),
                            library_obligations_loaded_by_acceptance=sum(r['acceptance_library_witness'] is not None for r in obligations),
                            interface_members=len(interfaces), reviewed_static_boundaries=len(unresolved)),
                obligations=obligations, source_homes=source_homes, interfaces=interfaces, static_boundaries=unresolved,
                validation='Exact partition, all 675 immutable obligation IDs, source anchors, every expanded interface member and every unresolved static site. Matching method names are navigation aids, not proofs of dynamic dispatch.')
