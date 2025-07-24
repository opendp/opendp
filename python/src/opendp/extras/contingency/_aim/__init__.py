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
from opendp.extras._utilities import to_then
from opendp.extras.contingency._utilities import (
    Count,
    make_noise_marginal,
    make_stable_marginals,
    mirror_descent,
    prior,
    get_cardinalities,
)
from opendp.extras.contingency.elements import DictDescriptor
from opendp.measurements import (
    then_report_noisy_max,
)
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.metrics import (
    frame_distance,
    linf_distance,
    symmetric_distance,
)
from opendp.mod import (
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Metric,
    Queryable,
    Transformation,
    binary_search_chain,
)
from opendp._internal import (
    _make_transformation,
    _new_pure_function,
)

from typing import Any, TypeAlias, cast


@dataclass(kw_only=True, frozen=True)
class AIM:
    """AIM mechanism from `MMSM22 <https://arxiv.org/abs/2201.12677>`_."""

    queries: list[Count] | int = 3
    """Workload of queries, each query is a list of columns to group by"""
    alpha: float = 0.9
    """Proportion of budget to allocate to the measure step"""
    max_size: int = 80
    """Maximum memory constraint for the marginal selection"""


def make_aim_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in: list[Any],  # list[Bound]
    d_out: float,
    releases: list[Any],  # list[LinearMeasurement]
    config: AIM = AIM(),
    estimator=mirror_descent,
) -> Measurement:
    """Implements AIM (Adaptive Iterative Mechanism) for ordinal data.

    :param input_domain: domain of input data
    :param input_metric: how to compute distance between datasets
    :param output_measure: how to measure privacy of release
    :param d_in: distance between adjacent input datasets
    :param d_out: upper bound on the privacy loss
    :param releases: LinearMeasurement list of prior releases
    :param config: settings for the AIM algorithm
    :param estimator: optimizer to use to fit a MarkovRandomField
    """
    import_optional_dependency("mbi")
    import mbi
    from mbi import MarkovRandomField, LinearMeasurement

    cardinalities = get_cardinalities(input_domain)

    if input_metric != frame_distance(symmetric_distance()):
        raise ValueError(
            f"input_metric ({input_metric}) must be frame_distance(symmetric_distance())"
        )

    if output_measure not in {max_divergence(), zero_concentrated_divergence()}:
        raise ValueError(
            f"output_measure ({output_measure}) must be max_divergence or zero_concentrated_divergence"
        )

    queries = expand_queries(config.queries, input_domain.columns)
    config = replace(config, queries=queries)

    mbi_domain = mbi.Domain(input_domain.columns, shape=cardinalities.values())

    cliques = [query.by for query in config.queries]
    t_marginals = make_stable_marginals(
        input_domain, input_metric, cliques, cardinalities
    )

    T = 16 * len(input_domain.columns)

    def function(qbl: Queryable) -> MarkovRandomField:
        # mutable state
        model = estimator(mbi_domain, releases)
        current_releases = releases.copy()
        d_step = d_out / T

        while d_step:
            m_step = make_aim_marginal(
                *t_marginals.output_space,
                output_measure,
                t_marginals.map(d_in),
                d_step,
                model,
                releases=current_releases,
                config=config,
                estimator=estimator,
            )
            if not m_step:
                break

            R: TypeAlias = tuple[MarkovRandomField, LinearMeasurement, float]
            model, marginal, error = cast(R, qbl(m_step))

            current_releases.append(marginal)

            size = prod(cardinalities[c] for c in marginal.clique)

            if error <= marginal.stddev * sqrt(2 / pi) * size:
                d_step *= 4

            d_step_max = prior(d_out - qbl.privacy_loss(t_marginals.map(d_in)))
            d_step = min(d_step, max(d_step_max, 0))

        return model

    return (
        t_marginals
        >> make_privacy_filter(
            make_fully_adaptive_composition(*t_marginals.output_space, output_measure),
            d_in=t_marginals.map(d_in),
            d_out=d_out,
        )
        >> _new_pure_function(function)
    )


then_aim_marginals = to_then(make_aim_marginals)


def make_aim_marginal(
    input_domain: LazyFrameDomain,
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    model,  # MarkovRandomField
    releases: list[Any],  # list[LinearMeasurement]
    config: AIM,
    estimator=mirror_descent,
) -> Measurement:
    """Create an interactive measurement that computes one step of the AIM algorithm."""
    import numpy as np
    from mbi import MarkovRandomField, LinearMeasurement

    if not isinstance(model, MarkovRandomField):
        raise ValueError("model must be a MarkovRandomField")

    d_measure = d_out * config.alpha
    d_select = prior(prior(d_out - d_measure))

    m_select = make_aim_select(  # type: ignore[misc]
        input_domain,
        input_metric,
        output_measure,
        queries=config.queries,
        model=model,
        d_in=d_in,
        d_out=d_select,
        max_size=config.max_size,
    )

    if not m_select:
        return

    def function(qbl: Queryable) -> tuple[MarkovRandomField, LinearMeasurement, float]:
        # SELECT a query that best reduces the error
        selected_clique = qbl(m_select)

        # MEASURE selected marginal with noise
        m_measure = binary_search_chain(
            lambda s: make_noise_marginal(
                input_domain,
                input_metric,
                output_measure,
                query=Count(selected_clique),
                scale=s,
            ),
            d_in=d_in,
            d_out=d_measure,
            T=float,
        )

        marginal = qbl(m_measure)

        # GENERATE an updated probability distribution
        prev_tab = model.project(selected_clique).values

        new_model: MarkovRandomField = estimator(
            model.domain,
            releases + [marginal],
            potentials=model.potentials.expand(model.cliques + [selected_clique]),
        )

        next_tab = new_model.project(selected_clique).values
        error = np.linalg.norm((next_tab - prev_tab).flatten(), 1)

        return new_model, marginal, error

    return make_adaptive_composition(
        input_domain,
        input_metric,
        output_measure,
        d_in=d_in,
        d_mids=[d_select, d_measure],
    ) >> _new_pure_function(function)


def make_aim_select(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    queries: list[Count],
    model,  # MarkovRandomField
    max_size: int,
) -> Measurement:
    """Make a measurement that selects a set of marginal query that will minimize error."""
    import mbi

    if not isinstance(model, mbi.MarkovRandomField):
        raise ValueError("model should be of type MarkovRandomField")

    # space threshold; new query cannot push memory above this value
    maximum_size_mb = d_in / d_out * max_size

    # only keep queries that fit the memory constraint
    candidates = [
        q
        for q in queries
        if q.by not in model.cliques
        and mbi.junction_tree.hypothetical_model_size(
            model.domain, model.cliques + [q.by]
        )
        <= maximum_size_mb
    ]

    if not candidates:
        return None

    return binary_search_chain(
        lambda scale: make_aim_scores(
            input_domain,
            input_metric,
            candidates,
            model,
            scale,
        )
        >> then_report_noisy_max(output_measure=output_measure, scale=scale),
        d_in=d_in,
        d_out=d_out,
        T=float,
    ) >> _new_pure_function(lambda idx: candidates[idx].by)


def make_aim_scores(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    queries: list[Count],
    model,  # MarkovRandomField
    scale: float,
) -> Transformation:
    """Make a transformation that assigns a score representing how poorly each query is estimated."""
    from opendp.extras.numpy import NPArrayDDescriptor
    import numpy as np

    if not isinstance(input_domain.descriptor, DictDescriptor):
        raise ValueError("input_domain must be a DictDomain")

    if not all(
        isinstance(d, ExtrinsicDomain) and isinstance(d.descriptor, NPArrayDDescriptor)
        for d in input_domain.descriptor.values()
    ):
        raise ValueError("input_domain values must be dp.numpy.arrayd_domain")

    def score_query(query: Count, exact):
        d = input_domain.descriptor[query.by]
        penalty = sqrt(2 / pi) * scale * prod(d.descriptor.shape)
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
        stability_map=lambda d_in: d_in * max(q.weight for q in queries),
    )


def expand_queries(queries: list[Count] | int, columns: list[str]) -> list[Count]:
    """Expand queries to include all subsets.

    :param queries: Explicit queries to shape workload, or maximum degree to preserve.
    :param columns: All columns in the data, for use when queries is a degree.
    """
    if isinstance(queries, int):
        queries = [Count(by) for by in combinations(columns, queries)]

    groupings = set(chain.from_iterable(powerset(q.by) for q in queries))

    def compute_weight(new_by: tuple[str, ...]) -> float:
        return sum(q.weight * len(set(new_by) & set(q.by)) for q in queries)

    return [Count(by, compute_weight(by)) for by in groupings if by]


def powerset(x: list):
    """returns all subsets"""
    return chain.from_iterable(combinations(x, r) for r in range(len(x) + 1))
