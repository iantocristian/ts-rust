"""Fail-closed, untimed census protocol and independent count reconciliation."""
from collections import Counter
import json

ARENAS = {'core_nodes','core_aux','symbols','tables','declarations','flows','flow_lists'}
FIELDS = {'version','index','bound_in_place','core_shapes','node_runtime_ids_by_shape','symbol_runtime_ids_assigned',
    'identifiers','arenas','aux_variants','source_metadata_variants','text_slice_elements','metadata_text_slice_elements',
    'node_lists','node_backings','declarations','tables','flow_data','binding_maps','lazy_slots_before','lazy_slots_after'}


def natural(value):
    if type(value) is not int or value < 0:
        raise ValueError('expected nonnegative integer')
    return value


def counts(value):
    if type(value) is not dict or any(type(k) is not str for k in value):
        raise ValueError('expected named count map')
    for n in value.values(): natural(n)
    return value


def histogram(value, pair=False):
    rows=[]
    for key, frequency in counts(value).items():
        dimensions = key.split('/')
        if len(dimensions) != (2 if pair else 1) or any(not part.isdecimal() or str(int(part)) != part for part in dimensions):
            raise ValueError('invalid histogram key')
        numbers=tuple(map(int,dimensions))
        if pair and numbers[0] > numbers[1]: raise ValueError('length exceeds capacity')
        if frequency == 0: raise ValueError('zero histogram bin')
        rows.append((*numbers,frequency))
    return rows


def sum_frequency(value, pair=False):
    return sum(row[-1] for row in histogram(value,pair))


def sum_elements(value):
    return sum(length*frequency for length,frequency in histogram(value))


def require(condition, message):
    if not condition: raise ValueError(message)


def validate_record(row,index):
    require(type(row) is dict and set(row)==FIELDS,'unknown/missing owner record fields')
    require(type(row['version']) is int and row['version']==1 and type(row['index']) is int and row['index']==index,'record identity/order changed')
    require(type(row['bound_in_place']) is bool,'invalid binding path')
    arenas=row['arenas']; require(set(arenas)==ARENAS,'missing arena')
    for arena in arenas.values():
        require(set(arena)=={'len','capacity','pages','directory_capacity','element_bytes'},'malformed arena')
        for n in arena.values(): natural(n)
        require(arena['len']<=arena['capacity'] and arena['pages']<=arena['directory_capacity'] and arena['element_bytes']>0,'invalid arena capacity')
        require((arena['capacity']==0)==(arena['pages']==0),'invalid empty arena')
    shapes=counts(row['core_shapes']); runtime=counts(row['node_runtime_ids_by_shape'])
    require(sum(shapes.values())==arenas['core_nodes']['len'],'physical node count mismatch')
    require(all(n<=shapes.get(k,0) for k,n in runtime.items()),'runtime IDs exceed physical nodes')
    require(natural(row['symbol_runtime_ids_assigned'])<=arenas['symbols']['len'],'runtime IDs exceed symbols')
    aux=counts(row['aux_variants']); metadata=counts(row['source_metadata_variants'])
    require(set(aux)<={'List','Nodes','Text','File','SourceFiles','SourceMetadata'},'unknown auxiliary variant')
    require(sum(aux.values())==arenas['core_aux']['len'],'auxiliary inventory incomplete')
    require(set(metadata)<={'Nodes','Text','Comments','Pragmas','References','DiagnosticDirectives'},'unknown source metadata variant')
    require(sum(metadata.values())==aux.get('SourceMetadata',0),'source metadata inventory incomplete')
    for k in ('text_slice_elements','metadata_text_slice_elements'): natural(row[k])
    identifiers=row['identifiers']; require(set(identifiers)=={'shapes','classes','selected_bytes','suffix_selected_bytes','fallback_selected_bytes','fallback_unique_values','fallback_unique_selected_bytes','fallback_length_histogram'},'malformed identifier census')
    identifier_shapes=counts(identifiers['shapes']); classes=counts(identifiers['classes'])
    require(identifier_shapes=={k:n for k,n in shapes.items() if k in {'Identifier','PrivateIdentifier'}},'identifier shape inventory differs')
    require(set(classes)<={'source_suffix','length_escape','invalid_range','byte_mismatch'},'unknown identifier class')
    total=sum(identifier_shapes.values()); fallback=total-classes.get('source_suffix',0)
    require(sum(classes.values())==total,'identifier class count mismatch')
    for k in ('selected_bytes','suffix_selected_bytes','fallback_selected_bytes','fallback_unique_values','fallback_unique_selected_bytes'): natural(identifiers[k])
    require(identifiers['selected_bytes']==identifiers['suffix_selected_bytes']+identifiers['fallback_selected_bytes'],'identifier byte split mismatch')
    require(sum_frequency(identifiers['fallback_length_histogram'])==fallback,'fallback histogram count mismatch')
    require(sum_elements(identifiers['fallback_length_histogram'])==identifiers['fallback_selected_bytes'],'fallback histogram byte mismatch')
    require(identifiers['fallback_unique_values']<=fallback and identifiers['fallback_unique_selected_bytes']<=identifiers['fallback_selected_bytes'],'invalid unique fallback count')
    lists=row['node_lists']; require(set(lists)=={'exposed_length_histogram','distinct_descriptors','nil','missing','allocated_empty','nonzero_start'},'malformed node lists')
    n_lists=aux.get('List',0); require(sum_frequency(lists['exposed_length_histogram'])==n_lists,'list header count mismatch')
    for k,v in lists.items():
        if k!='exposed_length_histogram': require(natural(v)<=n_lists,'list subset exceeds header count')
    backings=row['node_backings']; require(set(backings)=={'physical_length_histogram','physical_lengths_in_aux_order','nonnull_elements'},'malformed backing records')
    require(sum_frequency(backings['physical_length_histogram'])==aux.get('Nodes',0),'physical backing count mismatch')
    require(type(backings['physical_lengths_in_aux_order']) is list,'missing backing order')
    order=Counter(str(natural(n)) for n in backings['physical_lengths_in_aux_order'])
    require(dict(order)==backings['physical_length_histogram'],'ordered backing lengths differ from histogram')
    require(natural(backings['nonnull_elements'])<=sum_elements(backings['physical_length_histogram']),'nonnull backing count exceeds storage')
    declarations=row['declarations']; require(set(declarations)=={'physical_length_histogram','nonnull_elements','not_referenced_by_physical_symbols','distinct_symbol_descriptors','symbol_length_capacity_histogram'},'malformed declarations')
    require(sum_frequency(declarations['physical_length_histogram'])==arenas['declarations']['len'],'declaration backing count mismatch')
    require(sum_frequency(declarations['symbol_length_capacity_histogram'],True)==arenas['symbols']['len'],'symbol declaration count mismatch')
    require(natural(declarations['nonnull_elements'])<=sum_elements(declarations['physical_length_histogram']),'declaration backing bounds')
    require(natural(declarations['not_referenced_by_physical_symbols'])<=arenas['declarations']['len'],'obsolete declaration bounds')
    require(natural(declarations['distinct_symbol_descriptors'])<=arenas['symbols']['len'],'symbol descriptor bounds')
    require(set(row['tables'])=={'length_capacity_histogram'} and sum_frequency(row['tables']['length_capacity_histogram'],True)==arenas['tables']['len'],'table inventory mismatch')
    require(set(row['flow_data'])=={'None','Ast','SwitchClause','ReduceLabel'} and sum(counts(row['flow_data']).values())==arenas['flows']['len'],'flow discriminator inventory mismatch')
    maps=row['binding_maps']; require(set(maps)=={'overlay','bindings','flow_slots'},'binding map inventory mismatch')
    for key in ('overlay','bindings'):
        require(set(maps[key])=={'len','capacity'} and natural(maps[key]['len'])<=natural(maps[key]['capacity']),'map bounds')
    slots=maps['flow_slots']; require(type(slots) is list and len(slots)==6,'invalid flow slot report')
    for n in slots: natural(n)
    require(slots[1]<=slots[2]<=slots[3] and slots[4]<=slots[5] and slots[0]<=256*slots[1]+slots[4],'invalid flow slot capacity')
    for key in ('lazy_slots_before','lazy_slots_after'):
        value=row[key]; require(type(value) is list and len(value)==4,'lazy inventory malformed')
        for n in value: natural(n)
        require(value[2]<=value[0] and value[3]<=value[1],'invalid lazy reservations')
    require(row['lazy_slots_before']==row['lazy_slots_after'],'observer changed lazy storage')
    return row


def validate_capture(rows,child,expected):
    from s04_common import strict_json_loads
    require(type(child) is dict and set(child)=={'version','diagnostic_only','runtime','workers','files','loaded_bytes','loaded_input_sha256','nodes','symbols','parse_diagnostics','bind_diagnostics','bound_in_place_files','fallback_files','domain'},'unknown/missing child fields')
    require(type(child['version']) is int and child['version']==1 and child['diagnostic_only'] is True and child['runtime']=='rust' and type(child['workers']) is int and child['workers']==1 and type(child['domain']) is str,'invalid child protocol')
    for key in ('files','loaded_bytes','nodes','symbols','parse_diagnostics','bind_diagnostics'):
        require(natural(child[key])==expected[key],f'work counter differs: {key}')
    require(child['loaded_input_sha256']==expected['loaded_input_sha256'],'loaded input digest mismatch')
    parsed=[]
    for index,line in enumerate(rows.splitlines()): parsed.append(validate_record(strict_json_loads(line),index))
    require(len(parsed)==expected['files'],'missing/extra physical owner records')
    require(sum(r['arenas']['core_nodes']['len'] for r in parsed)==expected['nodes'],'physical/logical nodes differ')
    require(sum(r['arenas']['symbols']['len'] for r in parsed)==expected['symbols'],'physical/logical symbols differ')
    require(sum(r['bound_in_place'] for r in parsed)==natural(child['bound_in_place_files']),'binding paths differ')
    require(child['bound_in_place_files']+natural(child['fallback_files'])==expected['files'],'incomplete binding paths')
    return parsed


def aggregate(rows):
    def merge(values):
        result=Counter()
        for value in values: result.update(value)
        return dict(sorted(result.items()))
    arenas={}
    for name in sorted(ARENAS):
        widths={r['arenas'][name]['element_bytes'] for r in rows}
        require(len(widths)==1,'arena element widths changed across files')
        arenas[name]=merge({k:v for k,v in r['arenas'][name].items() if k!='element_bytes'} for r in rows)
        arenas[name]['element_bytes']=widths.pop()
    return {
        'files':len(rows),
        'arenas':arenas,
        'core_shapes':merge(r['core_shapes'] for r in rows),
        'node_runtime_ids_by_shape':merge(r['node_runtime_ids_by_shape'] for r in rows),
        'symbol_runtime_ids_assigned':sum(r['symbol_runtime_ids_assigned'] for r in rows),
        'identifier_classes':merge(r['identifiers']['classes'] for r in rows),
        'identifier_bytes':{key:sum(r['identifiers'][key] for r in rows) for key in ('selected_bytes','suffix_selected_bytes','fallback_selected_bytes','fallback_unique_values','fallback_unique_selected_bytes')},
        'aux_variants':merge(r['aux_variants'] for r in rows),
        'physical_node_backing_lengths':merge(r['node_backings']['physical_length_histogram'] for r in rows),
        'physical_declaration_backing_lengths':merge(r['declarations']['physical_length_histogram'] for r in rows),
        'table_length_capacity_histogram':merge(r['tables']['length_capacity_histogram'] for r in rows),
        'flow_data':merge(r['flow_data'] for r in rows),
    }
