from opendp.mod import ApproximateDivergence
import opendp.prelude as dp
import pytest


def test_deprecated_measure_aliases():
    with pytest.deprecated_call():
        assert str(dp.max_divergence()) == str(dp.pure_dp())
    with pytest.deprecated_call():
        assert str(dp.smoothed_max_divergence()) == str(dp.profile_dp())
    with pytest.deprecated_call():
        assert str(dp.fixed_smoothed_max_divergence()) == str(dp.approx_dp())
    with pytest.deprecated_call():
        assert dp.zero_concentrated_divergence() == dp.zcdp()
    with pytest.deprecated_call():
        assert dp.renyi_divergence() == dp.renyi_dp()
    with pytest.deprecated_call():
        assert dp._approximate_divergence_get_inner_measure(dp.approximate(dp.pure_dp())) == dp.pure_dp()


def test_approx_divergence():
    measure = dp.approximate(dp.pure_dp())
    assert isinstance(measure, ApproximateDivergence)
    assert measure.inner_measure == dp.pure_dp()


def test_inequality():
    measure = dp.pure_dp()
    assert measure != str(measure)
