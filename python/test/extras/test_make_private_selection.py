import opendp.prelude as dp

from ..helpers import optional_dependency


def test_private_selection_threshold():
    from opendp._extrinsics._make_private_selection import make_private_selection_threshold

    dp.enable_features("contrib", "floating-point")

    bounds = (0., 100.)
    range_ = bounds[1] - bounds[0]
    epsilon = 1.0
    threshold = 23

    input_domain = dp.vector_domain(dp.atom_domain(T=float))
    input_metric = dp.symmetric_distance()

    # score
    count = (dp.t.make_count(input_domain, input_metric)
             >> dp.m.then_geometric(scale=2 / epsilon))

    # output
    sum_ = (dp.t.make_clamp(input_domain, input_metric, bounds)
            >> dp.t.then_sum()
            >> dp.m.then_laplace(scale=2 * range_ / epsilon))

    mech_with_score = dp.c.make_basic_composition([count, sum_])

    with optional_dependency("numpy"):

        import numpy as np

        meas_pst = make_private_selection_threshold(mech_with_score,
                                                    threshold=threshold,
                                                    stop_probability=0)

        data = np.random.normal(10, 5, 20)

        score, _ = meas_pst(data)

    assert score >= threshold

    assert meas_pst.map(1) == 2 * mech_with_score.map(1)
