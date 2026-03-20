"""AIM mechanism from `MMSM22 <https://arxiv.org/abs/2201.12677>`_."""

from dataclasses import dataclass, replace
from itertools import combinations, chain
from math import pi, sqrt, prod

from opendp._lib import import_optional_dependency
from opendp.combinators import (
    make_adaptive_composition,
    make_fully_adaptive_composition,
    make_privacy_filter,
)
from opendp.domains import atom_domain, vector_domain
from opendp.extras.mbi._utilities import (
    TypedDictDomain,
    get_associated_metric,
    make_noise_marginal,
    make_stable_marginals,
    prior,
    weight_marginals,
    Count,
    Algorithm,
)
from opendp.measurements import then_noisy_max
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.metrics import linf_distance
from opendp.mod import (
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Metric,
    OdometerQueryable,
    Queryable,
    Transformation,
    binary_search_chain,
)
from opendp._internal import _make_transformation, _new_pure_function

from typing import Any, Optional, Sequence, TypeAlias, cast, TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


@dataclass(kw_only=True, frozen=True)
class AIM(Algorithm):
    """AIM mechanism from `MMSM22 <https://arxiv.org/abs/2201.12677>`_.

    Adaptively chooses and estimates the least-well-approximated marginal.
    The stronger the correlation amongst a clique of columns, 
    the more likely AIM is to select the clique.

    The algorithm starts with a small per-step privacy budget, 
    and in each step increases the budget if the last measured marginal doesn't sufficiently improve the model.

    ..
        >>> import pytest  # `pip install opendp[mbi]` is necessary
        >>> _ = pytest.importorskip("mbi")

    .. code:: pycon

        >>> import opendp.prelude as dp
        >>> import polars as pl

        >>> dp.enable_features("contrib")

        >>> context = dp.Context.compositor(
        ...     data=pl.scan_csv(dp.examples.get_france_lfs_path(), ignore_errors=True),
        ...     privacy_unit=dp.unit_of(contributions=36),
        ...     privacy_loss=dp.loss_of(rho=0.1, delta=1e-7),
        ... )

        >>> table_aim = (
        ...     context.query(rho=0.1, delta=1e-7)
        ...     # transformations/truncation may be applied here
        ...     .select("SEX", "AGE", "HWUSUAL", "ILOSTAT")
        ...     .contingency_table(
        ...         keys={"SEX": [1, 2]}, 
        ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
        ...         algorithm=dp.mbi.AIM()
        ...     )
        ...     .release()
        ... )

        >>> table_aim.synthesize() # doctest: +SKIP
        shape: (3_807_732, 4)
        ┌─────┬───────────┬───────────┬─────────┐
        │ SEX ┆ AGE       ┆ HWUSUAL   ┆ ILOSTAT │
        │ --- ┆ ---       ┆ ---       ┆ ---     │
        │ i64 ┆ f64       ┆ f64       ┆ i64     │
        ╞═════╪═══════════╪═══════════╪═════════╡
        │ 1   ┆ 55.446336 ┆ 20.776579 ┆ 1       │
        │ 1   ┆ 28.21838  ┆ 40.53348  ┆ 1       │
        │ 2   ┆ 43.291215 ┆ 34.406155 ┆ 1       │
        │ 1   ┆ 55.106615 ┆ 22.413161 ┆ 1       │
        │ 2   ┆ 42.585227 ┆ 40.11279  ┆ 3       │
        │ …   ┆ …         ┆ …         ┆ …       │
        │ 1   ┆ 58.197292 ┆ 40.139579 ┆ 1       │
        │ 1   ┆ 59.371221 ┆ 19.671153 ┆ 1       │
        │ 2   ┆ 19.862917 ┆ 40.339046 ┆ 9       │
        │ 1   ┆ 19.492355 ┆ 32.233661 ┆ 1       │
        │ 2   ┆ 60.863244 ┆ 40.737908 ┆ 3       │
        └─────┴───────────┴───────────┴─────────┘

    The algorithm is similar to the Multiplicative Weights Exponential Mechanism (MWEM) introduced in `HLM10 <https://arxiv.org/abs/1012.4763>`_,
    in that the exponential mechanism selects a marginal in each step.

    AIM differs from MWEM in that the distribution is represented via a graphical model and fitted via mirror descent,
    instead of joint distribution densities and the multiplicative weights update rule.
    This allows AIM to support higher-dimensional datasets.
    """

    queries: list[Count] | int = 3
    """Explicit workload of interactions, or maximum degree of interactions to consider."""
    measure_split: float = 0.9
    """Remaining proportion of budget to allocate to measuring marginals.
    
    The complement is spent on selecting marginals."""
    max_size: float = 80.0
    """Maximum memory constraint in MB for the marginal selection."""
    rounds: Optional[int] = None
    """Maximum number of rounds to run the algorithm."""

    def __post_init__(self):
        super().__post_init__()

        if isinstance(self.queries, int) and self.queries < 1:
            raise ValueError(f"queries ({self.queries}) must be positive")

        if isinstance(self.queries, list) and not self.queries:
            raise ValueError("queries must not be non-empty")

        if not (0 < self.measure_split <= 1):
            raise ValueError(f"measure_split ({self.measure_split}) must be in (0, 1]")

        if self.max_size <= 0.0:
            raise ValueError(f"max_size ({self.max_size}) must be positive")

    def make_marginals(
        self,
        input_domain: LazyFrameDomain,
        input_metric: FrameDistance,
        output_measure: Measure,
        d_in: list["Bound"],
        d_out: float,
        *,
        marginals: dict[tuple[str, ...], Any],
        model,  # MarkovRandomField
    ) -> Measurement:
        """Implements AIM (Adaptive Iterative Mechanism) for ordinal data.

        :param input_domain: domain of input data
        :param input_metric: how to compute distance between datasets
        :param output_measure: how to measure privacy of release
        :param d_in: distance between adjacent input datasets
        :param d_out: upper bound on the privacy loss
        :param marginals: prior marginal releases
        :param model: warm-start fit of MarkovRandomField
        """
        import_optional_dependency("mbi")
        from mbi import MarkovRandomField  # type: ignore[import-untyped,import-not-found]

        if not isinstance(model, MarkovRandomField):
            raise ValueError("model must be a MarkovRandomField")

        queries = _expand_queries(self.queries, input_domain.columns)
        algorithm = replace(self, queries=queries)

        lp_metric = get_associated_metric(output_measure)
        cliques = [query.by for query in queries]

        t_marginals = make_stable_marginals(
            input_domain, input_metric, lp_metric, cliques
        )
        d_marginals = t_marginals.map(d_in)

        T = algorithm.rounds or (16 * len(input_domain.columns))

        def function(
            qbl: OdometerQueryable,
        ) -> tuple[dict[tuple[str, ...], Any], MarkovRandomField]:
            # mutable state
            current_model = model
            current_marginals = marginals.copy()
            d_step = d_out / T
            d_spent = 0.0

            while d_step:
                m_step = _make_aim_marginal(
                    *t_marginals.output_space,
                    output_measure,
                    t_marginals.map(d_in),
                    d_step,
                    marginals=current_marginals,
                    model=current_model,
                    max_size=algorithm.max_size * (d_spent + d_step) / d_out,
                    algorithm=algorithm,
                )
                if not m_step:
                    break

                R: TypeAlias = tuple[
                    dict[tuple[str, ...], Any], MarkovRandomField, bool
                ]
                current_marginals, current_model, is_significant = cast(R, qbl(m_step))

                if not is_significant:
                    d_step *= 4

                d_spent = qbl.privacy_loss(d_marginals)
                d_step = min(d_step, max(prior(d_out - d_spent), 0))

            return current_marginals, current_model

        return (
            t_marginals
            >> make_privacy_filter(
                make_fully_adaptive_composition(
                    *t_marginals.output_space, output_measure
                ),
                d_in=d_marginals,
                d_out=d_out,
            )
            >> _new_pure_function(function)
        )


def _make_aim_marginal(
    input_domain: ExtrinsicDomain,  # TypedDictDomain of {clique: d-dimensional counts}
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    marginals: dict[tuple[str, ...], Any],
    model,  # MarkovRandomField
    max_size: float,
    algorithm: AIM,
) -> Optional[Measurement]:
    """Create an interactive measurement that computes one step of the AIM algorithm."""
    import numpy as np  # type: ignore[import-not-found]
    from mbi import MarkovRandomField, LinearMeasurement  # type: ignore[import-not-found]
    from opendp.extras.numpy import NPArrayDDomain

    model = cast(MarkovRandomField, model)

    cardinalities = {
        c: d.cast(NPArrayDDomain).shape
        for c, d in input_domain.cast(TypedDictDomain).items()
    }

    # factors to convert stddev -> scale and scale -> half-distribution expectation
    if output_measure == max_divergence():
        to_scale, to_mu = sqrt(2), 1.0
    elif output_measure == zero_concentrated_divergence():
        to_scale, to_mu = 1.0, sqrt(2 / pi)

    d_measure = d_out * algorithm.measure_split
    d_select = max(prior(prior(d_out - d_measure)), 0)

    m_select = _make_aim_select(  # type: ignore[misc]
        input_domain,
        input_metric,
        output_measure,
        queries=algorithm.queries,  # type: ignore[arg-type]
        model=model,
        d_in=d_in,
        d_out=d_select,
        max_size=max_size,
    )

    if not m_select:
        return None

    def function(
        qbl: Queryable,
    ) -> tuple[dict[tuple[str, ...], Any], MarkovRandomField, bool]:
        # SELECT a query that best reduces the error
        selected_clique = qbl(m_select)

        # MEASURE selected marginal with noise
        m_measure = binary_search_chain(
            lambda s: make_noise_marginal(
                input_domain,
                input_metric,
                output_measure,
                clique=selected_clique,
                scale=s,
            ),
            d_in=d_in,
            d_out=d_measure,
            T=float,
        )

        new_marginal: LinearMeasurement = qbl(m_measure)

        # GENERATE an updated probability distribution
        prev_tab = model.project(selected_clique).values

        all_marginals = weight_marginals(marginals, new_marginal)

        new_model: MarkovRandomField = algorithm.estimator(
            model.domain,
            list(all_marginals.values()),
            potentials=model.potentials.expand(list(all_marginals.keys())),
        )

        next_tab = new_model.project(selected_clique).values
        diff = float(np.linalg.norm((next_tab - prev_tab).flatten(), 1))

        size = prod(cardinalities[selected_clique])
        mean = new_marginal.stddev * to_scale * to_mu * size

        # Update is significant if the L1 norm of the update to the selected marginal
        #    is greater than the expectation of the noise's half-distribution.
        is_significant = diff > mean

        return all_marginals, new_model, is_significant

    return make_adaptive_composition(
        input_domain,
        input_metric,
        output_measure,
        d_in=d_in,
        d_mids=[d_select, d_measure],
    ) >> _new_pure_function(function)


def _make_aim_select(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    queries: list[Count],
    model,  # MarkovRandomField
    max_size: float,
) -> Optional[Measurement]:
    """Make a measurement that selects a set of marginal query that will minimize error."""
    import mbi

    # factor to convert scale -> half-distribution expectation
    if output_measure == max_divergence():
        to_mu = 1.0
    elif output_measure == zero_concentrated_divergence():
        to_mu = sqrt(2 / pi)

    model = cast(mbi.MarkovRandomField, model)

    def is_small(clique: tuple[str, ...]) -> bool:
        model_size = mbi.junction_tree.hypothetical_model_size(
            model.domain, cliques=model.cliques + [clique]
        )
        return model_size <= max_size

    # only keep queries that fit the memory constraint
    candidates = [q for q in queries if is_small(q.by)]

    if not candidates:
        return None

    def make(scale: float) -> Measurement:
        return _make_aim_scores(
            input_domain, input_metric, candidates, scale * to_mu, model
        ) >> then_noisy_max(output_measure=output_measure, scale=scale)

    try:
        return binary_search_chain(
            make, d_in=d_in, d_out=d_out, T=float
        ) >> _new_pure_function(lambda idx: candidates[idx].by)
    except ValueError:  # pragma: no cover
        # can fail in the unlikely scenario where
        #    d_out is so small that no scale is suitable
        return None


def _make_aim_scores(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    queries: list[Count],
    expectation: float,
    model,  # MarkovRandomField
) -> Transformation:
    """Make a transformation that assigns a score representing how poorly each query is estimated."""
    from opendp.extras.numpy import NPArrayDDomain
    import numpy as np
    from mbi import MarkovRandomField  # type: ignore[import-not-found]

    model = cast(MarkovRandomField, model)

    for value_domain in input_domain.cast(TypedDictDomain).values():
        value_domain.cast(NPArrayDDomain)  # pragma: no cover

    def score_query(query: Count, exact: np.ndarray):
        penalty = expectation * prod(exact.shape)
        synth = model.project(query.by).values

        return (np.linalg.norm((exact - synth).flatten(), 1) - penalty) * query.weight

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=vector_domain(atom_domain(T="f64", nan=False)),
        output_metric=linf_distance(T="f64", monotonic=False),
        function=lambda exact_tabs: [
            score_query(query, exact_tabs[query.by]) for query in queries
        ],
        stability_map=lambda d_in: max(d_in[q.by] * q.weight for q in queries),
    )


def _expand_queries(queries: list[Count] | int, columns: list[str]) -> list[Count]:
    """Expand queries to include all subsets.

    :param queries: Explicit queries to shape workload, or maximum degree to preserve.
    :param columns: All columns in the data, for use when queries is a degree.
    """
    if isinstance(queries, int):
        queries = [
            Count(by) for by in combinations(columns, min(queries, len(columns)))
        ]

    groupings = set(chain.from_iterable(_powerset(q.by) for q in queries))

    def compute_weight(new_by: tuple[str, ...]) -> float:
        return sum(q.weight * len(set(new_by) & set(q.by)) for q in queries)

    return [Count(by, compute_weight(by)) for by in groupings if by]


def _powerset(x: Sequence):
    """returns all subsets"""
    return chain.from_iterable(combinations(x, r) for r in range(len(x) + 1))
