from opendp._lib import get_np_csprng
import pytest
from ..helpers import optional_dependency
try:
    # So randomgen will be in sys.modules, if possible.
    import randomgen # type: ignore[import-not-found,import-untyped] # noqa F401
except ModuleNotFoundError:
    pass

def test_np_rng():
    n_cats = 100
    n_samples = 100_000
    
    with optional_dependency('randomgen'):
        np_csprng = get_np_csprng()
    
    np = pytest.importorskip('numpy')
    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[1]
    with optional_dependency('sklearn'):
        scipy = pytest.importorskip('scipy')
        assert scipy.stats.chisquare(counts).pvalue > .0001