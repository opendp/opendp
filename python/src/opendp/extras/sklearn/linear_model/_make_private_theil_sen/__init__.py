from opendp.mod import Measure
import opendp.prelude as dp
from opendp._lib import import_optional_dependency


def pairwise_predict(data, x_cuts):
    """
    The function matches data points together into pairs,
    fits a line that passes through each pair of points,
    and then computes the y values of each line at `x_cuts`.
    That is, it predicts the y values at `x_cuts`, pairwise.
    """
    np = import_optional_dependency("numpy")
    # Get an even number of rows.
    data = np.array(data, copy=True)[: len(data) // 2 * 2]

    # Shuffling the data improves the utility of results.
    np.random.shuffle(data)

    # Split data into pairs, where pair i is (p1[i], p2[i]).
    p1, p2 = np.array_split(data, 2)

    # Compute differences.
    dx, dy = (p2 - p1).T

    # Compute the midpoints of the pairs.
    x_bar, y_bar = (p1 + p2).T / 2

    # Compute y values on the pairwise slopes at x_cuts.
    points = dy / dx * (x_cuts[None].T - x_bar) + y_bar

    # Pairs where the x difference is zero are degenerate.
    return points.T[dx != 0]


def make_pairwise_predict(x_cuts, runs: int = 1, T=float):
    """
    The parameter `runs` controls how many times randomized pairwise predictions are computed.
    The default is 1. Increasing `runs` can improve the robustness and accuracy of the results;
    however, it can also increase computational cost and amount of noise needed later in the algorithm.
    """
    np = import_optional_dependency("numpy")
    return dp.t.make_user_transformation(
        # Outputs are Nx2 float numpy arrays.
        input_domain=dp.numpy.array2_domain(num_columns=2, T=T),
        # Neighboring input datasets differ by addition/removal of rows.
        input_metric=dp.symmetric_distance(),
        # Outputs are Nx2 float numpy arrays, but are half as long.
        output_domain=dp.numpy.array2_domain(num_columns=2, T=T),
        # Neighboring output datasets also differ by additions/removals.
        output_metric=dp.symmetric_distance(),
        # Apply the function `runs` times.
        function=lambda x: np.vstack(
            [pairwise_predict(x, x_cuts) for _ in range(runs)]
        ),
        # Each execution of pairwise predict contributes b_in records.
        stability_map=lambda b_in: b_in * runs,
    )


def make_select_column(j, T=float):
    return dp.t.make_user_transformation(
        input_domain=dp.numpy.array2_domain(num_columns=2, T=T),
        input_metric=dp.symmetric_distance(),
        output_domain=dp.vector_domain(dp.atom_domain(T=T)),
        output_metric=dp.symmetric_distance(),
        function=lambda x: x[:, j],
        stability_map=lambda b_in: b_in,
    )


def make_private_percentile_medians(output_measure, y_bounds, scale, candidates_count=100):
    np = import_optional_dependency("numpy")
    # this median mechanism favors candidates closest to the true median
    m_median = dp.m.then_private_quantile(
        output_measure=output_measure,
        # Evenly spaced points between y_bounds
        candidates=np.linspace(*y_bounds, candidates_count),
        alpha=0.5,
        scale=scale,
    )
    # apply the median mechanism to the 25th and 75th percentile columns
    return dp.c.make_composition(
        [
            make_select_column(0) >> dp.t.then_drop_null() >> m_median,
            make_select_column(1) >> dp.t.then_drop_null() >> m_median,
        ]
    )


def make_private_theil_sen(
    output_measure: Measure,
    x_bounds: tuple[float, float],
    y_bounds: tuple[float, float],
    scale: float,
    runs: int = 1,
    candidates_count: int = 100,
    fraction_bounds: tuple[float, float] = (0.25, 0.75),
) -> dp.Measurement:
    """
    Makes a measurement that takes a numpy array of (x, y) pairs,
    and returns a (slope, intercept) tuple.

    >>> import numpy as np
    >>> meas = make_private_theil_sen(dp.max_divergence(), (0, 100), (0, 100), scale=1.0)
    >>> slope, intercept = meas(np.array([[x, x] for x in range(100)]))
    """
    np = import_optional_dependency("numpy")
    # x_cuts are the fraction_bounds (for example, 25th and 75th percentiles) of x_bounds.
    # We'll predict y's at these x_cuts.
    x_cuts = x_bounds[0] + (x_bounds[1] - x_bounds[0]) * np.array(fraction_bounds)

    # we want coefficients, not y values!
    # Luckily y values are related to coefficients via a linear system
    P_inv = np.linalg.inv(np.vstack([x_cuts, np.ones_like(x_cuts)]).T)

    return (
        # pairwise_predict will return a 2xN array of y values at the x_cuts.
        make_pairwise_predict(x_cuts, runs)
        # privately select median y values for each x_cut
        >> make_private_percentile_medians(
            output_measure, y_bounds, scale, candidates_count=candidates_count
        )
        # transform median y values to coefficients (slope and intercept)
        >> (lambda ys: P_inv @ ys)
    )
