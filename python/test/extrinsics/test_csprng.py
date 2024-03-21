from opendp._lib import np_csprng
import pytest

@pytest.mark.skipif(np_csprng is None, reason='randomgen not installed')
def test_np_rng():
    n_cats = 100
    n_samples = 100_000

    np = pytest.importorskip('numpy')
    assert np_csprng is not None # for mypy
    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[1]
    pytest.importorskip('sklearn')
    scipy = pytest.importorskip('scipy')
    assert scipy.stats.chisquare(counts).pvalue > .0001