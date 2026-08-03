from opendp.mod import ApproximateDivergence
import opendp.prelude as dp


def test_approx_divergence():
    measure = dp.approximate(dp.pure_dp())
    assert isinstance(measure, ApproximateDivergence)
    assert measure.inner_measure == dp.pure_dp()


def test_inequality():
    measure = dp.pure_dp()
    assert measure != str(measure)
