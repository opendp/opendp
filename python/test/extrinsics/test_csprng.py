from opendp._lib import np_csprng
import pytest
import sys


@pytest.mark.skipif("numpy" not in sys.modules, reason="requires the Numpy library")
def test_np_rng():
    import numpy as np # type: ignore[import-not-found] 

    # 1 out of every 100k tests should fail, on average
    confidence = 1 - 1 / 100_000
    n_cats = 100
    n_samples = 100_000
    samples = np_csprng.integers(0, n_cats, size=n_samples)

    o = np.unique(samples, return_counts=True)[1]
    e = np.full(n_cats, int(n_samples // n_cats))
    chisq_stat = ((o - e) ** 2 / e).sum()

    # from scipy.stats import chi2 # commented to avoid dependency
    # critical_value = chi2.ppf(q=confidence, df=n_cats)
    critical_value = 161.31865695904807  # works out to this constant
    # p_value = 1 - chi2.cdf(x=critical_value, df=100)
    p_value = 9.999999999998899e-05 # works out to this constant
    # from scipy.stats import ncx2
    # power = 1 - ncx2.cdf(critical_value, df=n_cats, nc=chisq_stat)
    power = 0.9863769951346321

    if chisq_stat > critical_value:
        raise Exception(
            f"chisq statistic ({chisq_stat}) exceeded critical value ({critical_value}). "
            f"level of confidence={confidence}, p-value={p_value}"
        )

    print("critical value:", critical_value)
    print("test statistic:", chisq_stat)
    print("P-value:", p_value)
    print("power β:", power)
