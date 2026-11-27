"""Standalone tests for the common sklearn privacy-composition primitives."""

import pytest

pytest.importorskip("sklearn")

import opendp.prelude as dp
from opendp._internal import _make_measurement
from opendp.extras.sklearn._composition import (
    allocate_private_stages,
    checked_child_fit,
)

dp.enable_features("contrib")


def _space():
    return (
        dp.sklearn.sklearn_fit_domain(dp.vector_domain(dp.atom_domain(T=int))),
        dp.symmetric_distance(),
    )


def _allocate(names, weights=None):
    domain, metric = _space()
    return allocate_private_stages(
        domain,
        metric,
        dp.max_divergence(),
        1,
        1.0,
        names,
        weights,
    )


def test_private_stage_allocation_and_weights():
    with pytest.raises(ValueError, match="no private fitting"):
        _allocate([])
    with pytest.raises(ValueError, match="name exactly"):
        _allocate(["a", "b"], {"a": 1})
    with pytest.raises(ValueError, match="name exactly"):
        _allocate(["a", "b"], [1, 2])
    for weight in (0, -1, float("nan"), float("inf")):
        with pytest.raises(ValueError, match="finite and positive"):
            _allocate(["a", "b"], {"a": weight, "b": 1})

    compositor, budgets = _allocate(["a", "b"])
    assert compositor is not None
    assert len(budgets) == 2
    assert budgets[0] == pytest.approx(0.5)
    assert budgets[1] == pytest.approx(0.5)

    _, weighted = _allocate(["a", "b"], {"a": 1, "b": 3})
    assert weighted[0] == pytest.approx(0.25)
    assert weighted[1] == pytest.approx(0.75)


def test_checked_child_fit_uses_stability_and_enforces_budget():
    class Constant:
        def __init__(self, multiplier=1):
            self.multiplier = multiplier

        def make_measurement(self, domain, metric, measure, d_in, d_out):
            return _make_measurement(
                domain,
                metric,
                measure,
                lambda _data: 1,
                lambda d: self.multiplier * d_out * d / d_in,
            )

    domain, metric = _space()
    measure = dp.max_divergence()
    for prefix in (None, dp.t.make_identity(domain, metric)):
        measurement = checked_child_fit(
            Constant(),
            "a",
            prefix,
            (domain, metric),
            measure,
            1,
            0.5,
        )
        assert measurement.check(1, 0.5)
        assert measurement.map(1) == pytest.approx(0.5)

    with pytest.raises(ValueError, match="exceeds"):
        checked_child_fit(
            Constant(multiplier=2),
            "over-budget",
            None,
            (domain, metric),
            measure,
            1,
            0.5,
        )
