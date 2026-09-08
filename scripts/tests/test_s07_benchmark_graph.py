"""Register graph prerequisite failure-path checks in normal test discovery."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from s07_benchmark_graph_tests import PrerequisiteTests

__all__ = ["PrerequisiteTests"]
