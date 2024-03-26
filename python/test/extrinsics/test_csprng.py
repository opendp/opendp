from opendp._lib import get_rng
import pytest

def test_np_rng():
    n_cats = 100
    n_samples = 100_000

    np = pytest.importorskip('numpy')
    counts = np.unique(get_rng().integers(n_cats, size=n_samples), return_counts=True)[1]
    pytest.importorskip('sklearn')
    scipy = pytest.importorskip('scipy')
    assert scipy.stats.chisquare(counts).pvalue > .0001