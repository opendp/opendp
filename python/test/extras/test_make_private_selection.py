import opendp.prelude as dp

from ..helpers import optional_dependency


def test_private_selection_threshold():
    from opendp.extras._make_private_selection import (
        make_select_private_candidate_above_threshold,
    )

    dp.enable_features("contrib", "floating-point")

    bounds = (0.0, 100.0)
    range_ = bounds[1] - bounds[0]
    epsilon = 1.0
    threshold = 23

    space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()

    # score
    count = space >> dp.t.then_count() >> dp.m.then_laplace(scale=2 / epsilon)

    # output
    sum_ = (
        space
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=2 * range_ / epsilon)
    )

    mech_with_score = dp.c.make_basic_composition([count, sum_])

    with optional_dependency("numpy"), optional_dependency("randomgen"):

        import numpy as np

        meas_pst = make_select_private_candidate_above_threshold(
            mech_with_score, threshold=threshold, stop_probability=0
        )

        data = np.random.default_rng(seed=42).normal(10, 5, 20)

        score, _ = meas_pst(data)

    assert score >= threshold

    assert meas_pst.map(1) == 2 * mech_with_score.map(1)
