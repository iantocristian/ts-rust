"""Fixed S07 aggregation and uncertainty rules; no selection of fast samples."""
import math
import random
from statistics import median

SEED = 0x5307
RESAMPLES = 10_000


def sample_values(values):
    if not values or any(type(value) not in {int, float} or not 0 < value <= 2**127 or not math.isfinite(value) for value in values):
        raise ValueError("benchmark samples must be nonempty positive finite numbers")
    return values


def relative_mad(values):
    middle = median(sample_values(values))
    return median([abs(value - middle) for value in values]) / middle


def ratio_summary(go, rust, timing=False):
    sample_values(go)
    sample_values(rust)
    if len(go) != len(rust) or len(go) not in {7, 14, 21}:
        raise ValueError("S07 requires equal complete batches of 7, 14 or 21 samples")
    summary = {"samples_per_runtime": len(go), "go_median": median(go), "rust_median": median(rust),
               "ratio": median(rust) / median(go), "go_relative_mad": relative_mad(go), "rust_relative_mad": relative_mad(rust)}
    if timing:
        randomizer = random.Random(SEED)
        ratios = sorted(median(randomizer.choices(rust, k=len(rust))) / median(randomizer.choices(go, k=len(go))) for _ in range(RESAMPLES))
        # Frozen nearest-order-statistic bounds, no interpolation/version drift.
        summary["bootstrap"] = {"algorithm": "independent-median-ratio/order-statistic-v1", "seed": SEED,
                                "resamples": RESAMPLES, "confidence": 0.95,
                                "lower": ratios[249], "upper": ratios[9749]}
        summary["stable"] = summary["go_relative_mad"] <= 0.05 and summary["rust_relative_mad"] <= 0.05 and ratios[9749] <= 1.0
        summary["needs_more"] = len(go) < 21 and (ratios[249] <= 1.0 < ratios[9749] or summary["go_relative_mad"] > 0.05 or summary["rust_relative_mad"] > 0.05)
    return summary
