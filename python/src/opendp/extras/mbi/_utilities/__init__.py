from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Callable, Literal, Optional, Any, Iterator, get_args, TYPE_CHECKING
from functools import reduce
from math import sqrt
from opendp._internal import (
    _extrinsic_distance,
    _extrinsic_domain,
    _make_transformation,
    _new_pure_function,
)
from opendp.combinators import make_composition
from opendp.domains import atom_domain, vector_domain
from opendp.extras._utilities import to_then
from opendp.measurements import then_noise
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.metrics import (
    _get_bound,
    frame_distance,
    l1_distance,
    l2_distance,
    symmetric_distance,
)
from opendp.core import as_array
from opendp.mod import (
    ApproximateDivergence,
    AtomDomain,
    Domain,
    ExtrinsicDistance,
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Metric,
    Transformation,
)

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


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
    domain,  # mbi.Domain
    loss_fn,  # marginal_loss.MarginalLossFn | list[LinearMeasurement]
    *,
    potentials=None,  # CliqueVector | None
):  # MarkovRandomField
    """Fit a MarkovRandomField over the domain and loss function using mirror descent.

    If you want to use a custom estimator, consider this a contract/example.
    Your function can then close over configuration for any MBI estimator."""
    from mbi.estimation import mirror_descent as _mirror_descent  # type: ignore

    return _mirror_descent(domain, loss_fn, potentials=potentials)


OnewayType = Literal["all", "unkeyed"]
ONEWAY_ALL, ONEWAY_UNKEYED = get_args(OnewayType)


@dataclass(kw_only=True, frozen=True)
class Algorithm(ABC):
    """Base class for configuration of contingency table algorithms."""

    estimator: Callable = mirror_descent
    """Optimizer to use to fit a MarkovRandomField.

    Defaults to :py:func:`~opendp.extras.mbi.mirror_descent`.
    Any function matching the signature of ``mirror_descent`` 
    can be used to customize how the MarkovRandomField is optimized/estimated.
    See `mbi.estimation <https://private-pgm.readthedocs.io/en/latest/_autosummary_output/mbi.estimation.html>`_ for other optimizers.
    """
    oneway: OnewayType = ONEWAY_ALL
    """Fit one-way marginals for all columns, or only unkeyed columns."""
    oneway_split: Optional[float] = None
    """Proportion of budget to use for oneway release.
    
    When ``oneway_split`` is not set, defaults to half of the budget.

    If oneway is ``unkeyed``, budget is further reduced by the proportion of columns with missing keys or cuts.
    That is, when all columns have keys, then ``oneway_split`` is zero,
    and when no columns have keys, then ``oneway_split`` is one-half.
    """

    @property
    def needs_model(self) -> bool:
        """Whether a fitted model is needed to select the workload."""
        return True

    def __post_init__(self):
        if self.oneway not in get_args(OnewayType):
            raise ValueError(
                f"oneway ({self.oneway}) must be in {get_args(OnewayType)}"
            )

        if self.oneway_split is not None and not (0 <= self.oneway_split < 1):
            raise ValueError(f"oneway_split ({self.oneway_split}) must be in [0, 1)")

    @abstractmethod
    def make_marginals(
        self,
        input_domain: LazyFrameDomain,
        input_metric: FrameDistance,
        output_measure: Measure,
        d_in: list["Bound"],
        d_out: float,
        *,
        marginals: "Marginals",
        model,  # MarkovRandomField
    ) -> Measurement: ...


def typed_dict_domain(domains: dict[Any, Domain]) -> ExtrinsicDomain:
    """Domain containing a fixed-key dictionary of elements, each with its own domain."""

    def _member(x):
        if not isinstance(x, dict):
            raise ValueError("data must be a dict")
        if set(domains.keys()) != set(x.keys()):
            raise ValueError("data must share key-set with domain")
        return all(domains[k].member(x[k]) for k in domains)

    ident = ", ".join(f"{k}: {str(d)}" for k, d in domains.items())
    return _extrinsic_domain(
        identifier=f"TypedDictDomain({ident})",
        member=_member,
        descriptor=TypedDictDomain(domains),
    )


class TypedDictDomain(dict[tuple[str, ...], Domain]):
    pass


def typed_dict_distance(inner_metric: Metric) -> ExtrinsicDistance:
    """Dictionary distance metric.

    The metric forms a valid metric space when paired with a typed dict domain
    where the inner metric can be applied on values.
    The metric computes the distances between values in each key-value pair of the dictionary.
    """
    return _extrinsic_distance(
        identifier=f"TypedDictDistance({inner_metric})",
        descriptor=TypedDictDistance(inner_metric),
    )


@dataclass(frozen=True)
class TypedDictDistance:
    inner_metric: Metric


def get_std(measure: Measure, scale: float) -> float:
    if isinstance(measure, ApproximateDivergence):
        measure = measure.inner_measure

    if measure == max_divergence():
        return scale * sqrt(2)
    if measure == zero_concentrated_divergence():
        return scale
    message = f"output_measure ({measure}) must be max_divergence() or zero_concentrated_divergence()"
    raise ValueError(message)


def get_associated_metric(measure: Measure) -> Metric:
    if measure == max_divergence():
        return l1_distance(T="u32")
    if measure == zero_concentrated_divergence():
        return l2_distance(T="u32")
    message = f"output_measure ({measure}) must be max_divergence() or zero_concentrated_divergence()"
    raise ValueError(message)


def prior(x: float) -> float:
    """Returns the next smaller (prior) float"""
    import numpy as np

    return np.nextafter(x, -np.inf)


def get_cardinalities(input_domain: LazyFrameDomain) -> dict[str, int]:
    """Retrieves the cardinalities of each column in a lazyframe domain"""

    if not isinstance(input_domain, LazyFrameDomain):
        raise ValueError("input_domain must be dp.LazyFrameDomain")

    def get_cardinality(col):
        element_domain = input_domain.get_series_domain(col).element_domain
        if not isinstance(element_domain, AtomDomain):
            raise ValueError("input_domain columns must contain atomic data")

        bounds = element_domain.bounds
        if bounds is None:
            raise ValueError("input_domain columns must be bounded")
        lower, upper = bounds
        if lower != 0:
            raise ValueError("input_domain columns must be lower bounded by zero")
        return upper + 1

    return {col: get_cardinality(col) for col in input_domain.columns}


def make_stable_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_inner_metric: Metric,
    cliques: list[tuple[str, ...]],
) -> Transformation:
    """Return a transformation that computes all marginals in a workload."""
    from opendp.extras.numpy import arrayd_domain
    from opendp.extras.polars import Bound
    import polars as pl  # type: ignore[import-not-found]
    import numpy as np  # type: ignore[import-not-found]

    if input_metric != frame_distance(symmetric_distance()):
        message = f"input_metric ({input_metric}) must be frame_distance(symmetric_distance())"
        raise ValueError(message)

    cardinalities = get_cardinalities(input_domain)

    metrics = {l1_distance(T="u32"): 1, l2_distance(T="u32"): 2}
    if output_inner_metric not in metrics:
        message = f"inner_output_metric ({output_inner_metric}) must be in {set(metrics.keys())}"
        raise ValueError(message)
    p = metrics[output_inner_metric]

    def shape(by: tuple[str, ...]) -> tuple[int, ...]:
        return tuple(cardinalities[c] for c in by)

    def pivot(x, clique: tuple[str, ...]):
        y = np.zeros(shape(clique), dtype=np.int32)
        # u32 counts are clipped before narrowing to i32: numpy treats
        # uint32->int32 as a same-kind cast and silently wraps instead of erroring.
        y[tuple(x[clique].to_numpy().T)] = (
            x["len"].clip(upper_bound=np.iinfo(np.int32).max).cast(pl.Int32).to_numpy()
        )
        return y

    def function(data):
        dfs = pl.collect_all([data.group_by(c).agg(pl.len()) for c in cliques])
        return {c: pivot(m, c) for m, c in zip(dfs, cliques)}

    def count_sensitivity(bounds: list[Bound], clique: tuple[str, ...]) -> float:
        l1 = _get_bound(bounds, []).per_group

        bound: Bound = _get_bound(bounds, [pl.col(c) for c in clique])
        l0 = (bound.num_groups or l1) ** (1 / p)
        li = bound.per_group or l1
        return min(l1, l0 * li)

    def stability_map(d_in: list[Bound]) -> dict[tuple[str, ...], float]:
        return {clique: count_sensitivity(d_in, clique) for clique in cliques}

    return _make_transformation(
        input_domain,
        input_metric,
        output_domain=typed_dict_domain(
            {c: arrayd_domain(shape=shape(c), T="i32") for c in cliques}
        ),
        output_metric=typed_dict_distance(output_inner_metric),
        function=function,
        stability_map=stability_map,
    )


def make_noise_marginals(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    output_measure: Measure,
    cliques: list[tuple[str, ...]],
    scale: float,
    weights: Optional[list[float]] = None,
) -> Measurement:
    """Make a measurement that releases multiple DP marginals"""
    measurements = [
        make_noise_marginal(
            input_domain, input_metric, output_measure, clique, scale / weight
        )
        for clique, weight in zip(cliques, weights or [1] * len(cliques))
    ]
    return make_composition(measurements)


then_noise_marginals = to_then(make_noise_marginals)


def make_noise_marginal(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    output_measure: Measure,
    clique: tuple[str, ...],
    scale: float,
) -> Measurement:
    """Make a measurement that releases a DP marginal"""
    # use jax arrays because lms will be used in optimization
    import jax.numpy as np  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]
    from opendp.extras.numpy import NPArrayDDomain

    clique_domain = input_domain.cast(TypedDictDomain)[clique]
    inner_metric = input_metric.cast(TypedDictDistance).inner_metric

    clique_domain.cast(NPArrayDDomain)

    associated_metric = get_associated_metric(output_measure)

    if inner_metric != associated_metric:
        message = f"input_metric's inner metric ({inner_metric}) doesn't match the output_measure's associated metric ({associated_metric})"
        raise ValueError(message)

    t_marginal = _make_transformation(
        input_domain,
        input_metric,
        output_domain=vector_domain(atom_domain(T="i32")),
        output_metric=inner_metric,
        function=lambda exact_tabs: exact_tabs[clique].astype(np.int32).flatten(),
        stability_map=lambda d_in: d_in[clique],
    )

    def function(x):
        return LinearMeasurement(x, clique, stddev=get_std(output_measure, scale))

    return (
        t_marginal
        >> then_noise(output_measure, scale)
        >> as_array()
        >> _new_pure_function(function)
    )


def row_major_order(keys: Iterator):
    """Joins a list of keysets that align to row-major ordering."""

    def reducer(a, b):
        return a.join(b, how="cross", maintain_order="left_right")

    return reduce(reducer, (keyset.to_frame() for keyset in keys))


@dataclass(frozen=True)
class SelectPrefixQuery:
    """Select a prefix of a flattened MBI marginal.

    Used when the final cell is a latent synthetic ``other`` bucket rather than
    a directly measured count.
    """

    length: int

    def __call__(self, factor):
        return factor.datavector()[: self.length]


def _queries_equal(left: Callable, right: Callable) -> bool:
    if left is right:
        return True
    try:
        result = left == right
        return result if isinstance(result, bool) else False
    except Exception:  # pragma: no cover - defensive for user-defined queries
        return False


def identity_query_precision(measurement) -> float:
    """Return precision contributed by a full-datavector observation.

    Partial queries do not provide a uniform uncertainty estimate for every
    cell in a marginal, so they contribute zero precision.
    """
    from mbi import Factor  # type: ignore[import-untyped,import-not-found]

    if _queries_equal(measurement.query, Factor.datavector):
        return 1 / measurement.stddev**2
    return 0.0


class Marginals:
    """A multimap from clique to independent :py:class:`mbi.LinearMeasurement`.

    A clique identifies the marginal a query operates on; it does not
    uniquely identify the observation. Several independent measurements can
    legitimately share a clique: an initial partial (prefix) release, a later
    full release, a zero-way total, or any other linear query over the same
    marginal. Distinct query operators are kept as separate observations,
    even when their cliques match.
    """

    def __init__(self, by_clique: Optional[dict[tuple[str, ...], list]] = None):
        self.by_clique: dict[tuple[str, ...], list] = {
            clique: list(group) for clique, group in (by_clique or {}).items()
        }

    def cliques(self) -> list[tuple[str, ...]]:
        """Unique cliques with at least one measurement."""
        return list(self.by_clique)

    def flatten(self) -> list:
        """All measurements, across all cliques."""
        return [
            measurement for group in self.by_clique.values() for measurement in group
        ]

    def copy(self) -> "Marginals":
        return Marginals(self.by_clique)

    def add(self, *new_measurements) -> "Marginals":
        """Return a new collection with `new_measurements` appended.

        Each measurement is kept atomic: independent observations sharing a
        clique (a repeated identity release, a prefix release alongside a
        later full release, a repeated zero-way total, ...) are never
        combined. Inverse-variance coalescing is exact only for the default
        quadratic loss, and `Algorithm.estimator` is a public extension point
        that may use an L1 or other custom loss, so combining measurements
        here would silently change the meaning of the supplied collection
        for such an estimator. MBI already sums losses across atomic
        measurements, so keeping them separate is correct for any loss."""
        from mbi import LinearMeasurement  # type: ignore[import-untyped,import-not-found]

        result = self.copy()

        for new in new_measurements:
            if not isinstance(new, LinearMeasurement):
                raise ValueError("each new marginal must be of type LinearMeasurement")

            result.by_clique.setdefault(new.clique, []).append(new)

        return result
