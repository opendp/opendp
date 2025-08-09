from opendp.mod import ApproximateDivergence
import opendp.prelude as dp


def test_approx_divergence():
    measure = dp.approximate(dp.max_divergence())
    assert isinstance(measure, ApproximateDivergence)
    assert measure.inner_measure == dp.max_divergence()


def test_inequality():
    measure = dp.max_divergence()
    assert measure != str(measure)
