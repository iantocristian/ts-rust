"""P0 measurement domains. These validators do not generate measurement evidence."""
import math
from statistics import median

from s08_contracts import fields, integer


def validate_samples(samples, workload_sha256, executed_queries):
    if type(samples) is not list or len(samples) != 7:
        raise ValueError('exactly seven complete measured samples required')
    for row in samples:
        fields(row, 'state workload_sha256 executed_queries elapsed_seconds requested_bytes retained_bytes')
        if row['state'] != 'completed' or row['workload_sha256'] != workload_sha256:
            raise ValueError('incomplete or different measured workload')
        integer(row['executed_queries'], 1)
        if row['executed_queries'] != executed_queries:
            raise ValueError('timed work omitted queries')
        seconds = row['elapsed_seconds']
        if type(seconds) not in (int, float) or not math.isfinite(seconds) or seconds <= 0:
            raise ValueError('elapsed denominator must be positive and finite')
        integer(row['requested_bytes'])
        # A negative delta is unusable; never clamp it to zero.
        integer(row['retained_bytes'])


def measured_ratios(numerator, denominator, workload_sha256, executed_queries):
    for samples in (numerator, denominator):
        validate_samples(samples, workload_sha256, executed_queries)
    result = {}
    for key in ('elapsed_seconds', 'requested_bytes', 'retained_bytes'):
        a, b = median(r[key] for r in numerator), median(r[key] for r in denominator)
        if b <= 0:
            raise ValueError('unusable measured denominator: ' + key)
        result[key] = a / b
    result['throughput'] = 1 / result['elapsed_seconds']
    return result


def type_mean_ratio(numerator, denominator):
    """Aggregate census totals: preserve bytes and counts beside the result."""
    for row in (numerator, denominator):
        fields(row, 'state charged_type_storage_bytes logical_retained_types unassigned_allocations')
        if row['state'] != 'complete' or row['unassigned_allocations'] != []:
            raise ValueError('incomplete structural census')
        integer(row['charged_type_storage_bytes'], 1)
        integer(row['logical_retained_types'], 1)
    return ((numerator['charged_type_storage_bytes'] / numerator['logical_retained_types']) /
            (denominator['charged_type_storage_bytes'] / denominator['logical_retained_types']))
