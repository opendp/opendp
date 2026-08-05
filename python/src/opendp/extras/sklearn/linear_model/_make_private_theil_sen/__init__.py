"""Calibrated OpenDP measurement for Theil-Sen regression."""

from __future__ import annotations

from typing import Any

from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.mod import Domain, Measure, Measurement, Metric


def _numpy():
    return import_optional_dependency("numpy")


def pairwise_predict(data, x_cuts):
    """Return randomized pairwise line predictions at ``x_cuts``."""
    np = _numpy()
    data = np.array(data, copy=True)[: len(data) // 2 * 2]
    np.random.shuffle(data)
    p1, p2 = np.array_split(data, 2)
    dx, dy = (p2 - p1).T
    x_bar, y_bar = (p1 + p2).T / 2
    points = dy / dx * (x_cuts[None].T - x_bar) + y_bar
    return points.T[dx != 0]


def make_pairwise_predict(input_domain, input_metric, x_cuts, runs: int = 1, T=float):
    """Make the public pairwise-prediction transformation."""
    import opendp.prelude as dp

    np = _numpy()
    output_domain = dp.numpy.array2_domain(num_columns=2, T=T)
    return dp.t.make_user_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=output_domain,
        output_metric=dp.symmetric_distance(),
        function=lambda x: np.vstack(
            [pairwise_predict(x, x_cuts) for _ in range(runs)]
        ),
        stability_map=lambda d_in: d_in * runs,
    )


def make_select_column(input_domain, input_metric, j, T=float):
    import opendp.prelude as dp

    return dp.t.make_user_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=dp.vector_domain(dp.atom_domain(T=T)),
        output_metric=dp.symmetric_distance(),
        function=lambda x: x[:, j],
        stability_map=lambda d_in: d_in,
    )


def make_private_percentile_medians(
    input_domain,
    input_metric,
    output_measure,
    y_bounds,
    scale,
    candidates_count=100,
):
    import opendp.prelude as dp

    np = _numpy()
    m_median = dp.m.then_private_quantile(
        output_measure=output_measure,
        candidates=np.linspace(*y_bounds, candidates_count),
        alpha=0.5,
        scale=scale,
    )
    return dp.c.make_composition(
        [
            make_select_column(input_domain, input_metric, 0)
            >> dp.t.then_drop_null()
            >> m_median,
            make_select_column(input_domain, input_metric, 1)
            >> dp.t.then_drop_null()
            >> m_median,
        ]
    )


def _make_private_theil_sen_with_scale(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    *,
    x_bounds,
    y_bounds,
    scale: float,
    runs: int = 1,
    candidates_count: int = 100,
    fraction_bounds=(0.25, 0.75),
) -> Measurement:
    """Build the scale-parameterized Theil-Sen measurement."""
    import opendp.prelude as dp

    np = _numpy()
    desc = input_domain.descriptor
    if getattr(desc, "num_columns", None) != 2:
        raise ValueError("TheilSenRegressor requires a two-column input domain")  # pragma: no cover
    if input_metric != dp.symmetric_distance():
        raise ValueError("TheilSenRegressor supports symmetric_distance() only")  # pragma: no cover
    if len(x_bounds) != 1:
        raise ValueError("x_bounds must contain exactly one feature bound")  # pragma: no cover
    if len(fraction_bounds) != 2 or fraction_bounds[0] >= fraction_bounds[1]:
        raise ValueError("fraction_bounds must contain two increasing values")  # pragma: no cover
    if not 0 < fraction_bounds[0] < fraction_bounds[1] < 1:
        raise ValueError("fraction_bounds must lie strictly between zero and one")  # pragma: no cover
    if runs < 1 or candidates_count < 2 or scale <= 0:
        raise ValueError("runs, candidates_count, and scale must be positive")  # pragma: no cover

    x_bounds = x_bounds[0]
    x_cuts = x_bounds[0] + (x_bounds[1] - x_bounds[0]) * np.asarray(
        fraction_bounds
    )
    P_inv = np.linalg.inv(np.vstack([x_cuts, np.ones_like(x_cuts)]).T)

    pairwise = make_pairwise_predict(
        input_domain, input_metric, x_cuts, runs=runs
    )
    pair_domain = pairwise.output_domain
    pair_metric = pairwise.output_metric
    medians = make_private_percentile_medians(
        pair_domain,
        pair_metric,
        output_measure,
        y_bounds,
        scale,
        candidates_count=candidates_count,
    )
    return (
        pairwise
        >> medians
        >> _new_pure_function(lambda ys: tuple(P_inv @ np.asarray(ys)))
    )


def make_private_theil_sen(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    *,
    x_bounds,
    y_bounds,
    runs: int = 1,
    candidates_count: int = 100,
    fraction_bounds=(0.25, 0.75),
) -> Measurement:
    """Construct a calibrated Theil-Sen measurement over paired ``[x, y]`` rows."""
    import opendp.prelude as dp

    if d_in <= 0 or d_out <= 0:
        raise ValueError("d_in and d_out must be positive")  # pragma: no cover

    def make(scale: float) -> Measurement:
        return _make_private_theil_sen_with_scale(
            input_domain,
            input_metric,
            output_measure,
            x_bounds=x_bounds,
            y_bounds=y_bounds,
            scale=scale,
            runs=runs,
            candidates_count=candidates_count,
            fraction_bounds=fraction_bounds,
        )

    return dp.binary_search_chain(
        make,
        d_in=d_in,
        d_out=d_out,
        T=float,
    )
