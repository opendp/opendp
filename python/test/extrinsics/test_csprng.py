from opendp._lib import np_csprng
import pytest
import sys


@pytest.mark.skipif("numpy" not in sys.modules, reason="requires the Numpy library")
def test_np_rng():
    import numpy as np # type: ignore[import-not-found] 
    from scipy.stats import chisquare # type: ignore[import]

    n_cats = 100
    n_samples = 100_000
    counts = np.unique(np_csprng.integers(n_cats, size=n_samples), return_counts=True)[1]
    assert chisquare(counts).pvalue > .0001