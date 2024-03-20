from opendp._lib import np_csprng
import pytest


def test_np_rng():
    n_cats = 100
    n_samples = 100_000

    np = pytest.importorskip('numpy')
    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[1]
    scipy = pytest.importorskip('scipy')
    assert scipy.stats.chisquare(counts).pvalue > .0001