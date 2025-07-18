from dataclasses import dataclass
from math import sqrt
from opendp._internal import _make_transformation, _new_pure_function
from opendp.combinators import make_composition
from opendp.domains import atom_domain, vector_domain
from opendp.extras._utilities import to_then
from opendp.extras.contingency.elements import (
    DictDescriptor,
    dict_domain,
    linf_norm,
)
from opendp.measurements import then_noise
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.metrics import l1_distance, l2_distance
from opendp.mod import (
    AtomDomain,
    ExtrinsicDistance,
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Transformation,
)


@dataclass
class Count:
    """Denotes a count query."""

    by: tuple[str, ...]
    """Columns to group by."""
    weight: float = 1.0
    """Importance of this count query. 
    
    - Used by AIM to prioritize cliques.
    - Used by Fixed to distribute privacy budget.
    """

    def __post_init__(self):
        self.by = tuple(self.by)
        if self.weight < 0:
            raise ValueError(f"weight ({self.weight}) must be non-negative")


def mirror_descent(
    domain,
    loss_fn,
    *,
    potentials=None,
):
    """Fit a MarkovRandomField over the domain and loss function using mirror descent.

    Any function following this signature can be passed into `estimator`
    to customize how the MarkovRandomField is fitted.
    See `mbi.estimation <https://private-pgm.readthedocs.io/en/latest/_autosummary_output/mbi.estimation.html>`_ for other optimizers.

    :param domain: mbi.Domain
    :param loss_fn: list[mbi.LinearMeasurement] or mbi.MarginalLossFn
    :param potentials: mbi.CliqueVector for warm-start of model fit
    :return: mbi.MarkovRandomField
    """
    import mbi

    return mbi.estimation.mirror_descent(
        domain=domain, loss_fn=loss_fn, potentials=potentials
    )


def prior(x: float) -> float:
    """Returns the next smaller (prior) float"""
    import numpy as np

    return np.nextafter(x, -np.inf)


def get_cardinalities(input_domain: LazyFrameDomain) -> dict[str, int]:
    """Retrieves the cardinalities of each column in a lazyframe domain"""

    if not isinstance(input_domain, LazyFrameDomain):
        raise ValueError("input_domain must be dp.LazyFrameDomain")

    def get(col):
        element_domain = input_domain.get_series_domain(col).element_domain
        if not isinstance(element_domain, AtomDomain):
            raise ValueError("input_domain columns must contain atomic data")
        lower, upper = element_domain.bounds
        if lower != 0:
            raise ValueError(
                "input_domain columns must be integral bounded between [0, b]"
            )
        return upper + 1

    return {col: get(col) for col in input_domain.columns}


def make_stable_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    cliques: list[tuple[str, ...]],
    cardinalities: dict[str, int],
) -> Transformation:
    """Return a transformation that computes all marginals in a workload."""
    from opendp.extras.numpy import arrayd_domain
    from opendp.extras.polars import Bound
    import polars as pl
    import numpy as np

    def shape(by: tuple[str, ...]) -> tuple[int, ...]:
        return tuple(cardinalities[c] for c in by)

    def pivot(x, clique: tuple[str, ...]):
        y = np.zeros(shape(clique), dtype=np.int32)
        y[*x[clique].to_numpy().T] = x["len"].to_numpy()
        return y

    def function(data):
        dfs = pl.collect_all([data.group_by(c).agg(pl.len()) for c in cliques])
        return {c: pivot(m, c) for m, c in zip(dfs, cliques)}

    def stability_map(d_in: list[Bound]) -> int:
        return next(d for d in d_in if not d.by).per_group

    return _make_transformation(
        input_domain,
        input_metric,
        output_domain=dict_domain(
            {c: arrayd_domain(shape=shape(c), T="i64") for c in cliques}
        ),
        output_metric=linf_norm(l1_distance(T="f64")),
        function=function,
        stability_map=stability_map,
    )


def make_noise_marginals(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    output_measure: Measure,
    queries: list[Count],
    scale: float,
) -> Measurement:
    """Make a measurement that releases multiple DP marginals"""
    measurements = [
        make_noise_marginal(input_domain, input_metric, output_measure, query, scale)
        for query in queries
    ]
    return make_composition(measurements)


then_noise_marginals = to_then(make_noise_marginals)


def make_noise_marginal(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    output_measure: Measure,
    query: Count,
    scale: float,
) -> Measurement:
    """Make a measurement that releases a DP marginal"""
    from opendp.extras.numpy import NPArrayDDescriptor
    import numpy as np
    import mbi

    descriptor = input_domain.descriptor
    if not isinstance(descriptor, DictDescriptor):
        raise ValueError("input_domain must be DictDomain")

    clique_domain = descriptor[query.by]
    if not (
        isinstance(clique_domain, ExtrinsicDomain)
        and isinstance(clique_domain.descriptor, NPArrayDDescriptor)
    ):
        raise ValueError("input_domain descriptor value must be dp.numpy.arrayd_domain")

    if input_metric != linf_norm(l1_distance(T="f64")):
        raise ValueError('input_metric must be linf_norm(l1_distance(T="f64"))')

    if output_measure == max_divergence():
        sensitivity_metric, rescale = l1_distance(T="f64"), sqrt(2)
    elif output_measure == zero_concentrated_divergence():
        sensitivity_metric, rescale = l2_distance(T="f64"), 1
    else:
        raise ValueError(
            "output_measure must be max_divergence() or zero_concentrated_divergence()"
        )

    t_marginal = _make_transformation(
        input_domain,
        input_metric,
        output_domain=vector_domain(atom_domain(T="i32")),
        output_metric=sensitivity_metric,
        function=lambda exact_tabs: exact_tabs[query.by].astype(np.int32).flatten(),
        stability_map=lambda d_in: d_in,
    )

    def postprocess(x):
        return mbi.LinearMeasurement(
            noisy_measurement=x,
            clique=query.by,
            stddev=scale / query.weight * rescale,
        )

    return (
        t_marginal
        >> then_noise(output_measure, scale / query.weight)
        >> _new_pure_function(postprocess)
    )
