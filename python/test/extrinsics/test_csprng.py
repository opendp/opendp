from opendp._lib import get_np_csprng
import pytest
from ..helpers import optional_dependency

def test_np_rng():
    n_cats = 100
    n_samples = 100_000
    
    with optional_dependency('randomgen'):
        np_csprng = get_np_csprng()
    
    np = pytest.importorskip('numpy')
    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[1]
    pytest.importorskip('sklearn') # TODO: implicit dependency should use optional_dependency
    scipy = pytest.importorskip('scipy')
    assert scipy.stats.chisquare(counts).pvalue > .0001