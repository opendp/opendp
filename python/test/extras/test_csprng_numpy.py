from opendp._lib import get_np_csprng
import pytest
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")

try:
    # So randomgen will be in sys.modules, if possible.
    import randomgen  # type: ignore[import-not-found,import-untyped] # noqa F401
except ModuleNotFoundError:
    pass


def test_np_rng():
    n_cats = 100
    n_samples = 100_000

    np_csprng = get_np_csprng()

    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[
        1
    ]
    scipy = pytest.importorskip("scipy")
    assert scipy.stats.chisquare(counts).pvalue > 0.0001
