import opendp.prelude as dp
import numpy as np

dp.enable_features("contrib", "honest-but-curious")


# pairwise-predict
def pairwise_predict(data, x_cuts):
    data = np.array(data, copy=True)[: len(data) // 2 * 2]
    np.random.shuffle(data)
    p1, p2 = np.array_split(data, 2)
    dx, dy = (p2 - p1).T
    x_bar, y_bar = (p1 + p2).T / 2
    points = dy / dx * (x_cuts[None].T - x_bar) + y_bar
    return points.T[dx != 0]


def make_pairwise_predict(x_cuts, runs: int = 1):
    return dp.t.make_user_transformation(
        input_domain=dp.numpy.array2_domain(num_columns=2, T=float),
        input_metric=dp.symmetric_distance(),
        output_domain=dp.numpy.array2_domain(num_columns=2, T=float),
        output_metric=dp.symmetric_distance(),
        function=lambda x: np.vstack([pairwise_predict(x, x_cuts) for _ in range(runs)]),
        stability_map=lambda d_in: d_in * runs,
    )


# /pairwise-predict


# private-medians
def make_select_column(j):
    return dp.t.make_user_transformation(
        input_domain=dp.numpy.array2_domain(num_columns=2, T=float),
        input_metric=dp.symmetric_distance(),
        output_domain=dp.vector_domain(dp.atom_domain(T=float)),
        output_metric=dp.symmetric_distance(),
        function=lambda x: x[:, j],
        stability_map=lambda d_in: d_in,
    )


def make_private_percentile_medians(output_measure, y_bounds, scale):
    m_median = dp.m.then_private_quantile(
        output_measure,
        candidates=np.linspace(*y_bounds, 100),
        alpha=0.5,
        scale=scale,
    )
    return dp.c.make_composition(
        [
            make_select_column(0) >> dp.t.then_drop_null() >> m_median,
            make_select_column(1) >> dp.t.then_drop_null() >> m_median,
        ]
    )


# /private-medians


# mechanism
def make_private_theil_sen(output_measure, x_bounds, y_bounds, scale, runs=1):
    x_cuts = x_bounds[0] + (x_bounds[1] - x_bounds[0]) * np.array([0.25, 0.75])
    p_inv = np.linalg.inv(np.vstack([x_cuts, np.ones_like(x_cuts)]).T)

    return (
        make_pairwise_predict(x_cuts, runs)
        >> make_private_percentile_medians(output_measure, y_bounds, scale)
        >> (lambda ys: p_inv @ ys)
    )


x_bounds = (-3.0, 3.0)
y_bounds = (-10.0, 10.0)
meas = make_private_theil_sen(dp.max_divergence(), x_bounds, y_bounds, scale=1.0)
meas.map(1)
# /mechanism


# release
def f(x):
    return x * 2 + 1


x = np.random.normal(size=100, loc=0, scale=1.0)
y = f(x) + np.random.normal(size=100, loc=0, scale=0.5)
slope, intercept = meas(np.stack([x, y], axis=1))
(slope, intercept)
# /release
