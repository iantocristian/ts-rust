"""Register program acceptance counterexamples in the normal tracker self-tests."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from test_s07_program_compare import ProgramComparisonTests
