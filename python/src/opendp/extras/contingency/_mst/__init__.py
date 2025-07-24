from dataclasses import dataclass
import itertools
from typing import Any

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import then_adaptive_composition
from opendp.domains import atom_domain, vector_domain
from opendp.extras._utilities import to_then
from opendp.extras.contingency._utilities import (
    Count,
    make_noise_marginals,
    mirror_descent,
    prior,
    make_stable_marginals,
    get_cardinalities,
)
from opendp.extras.contingency.elements import DictDescriptor
from opendp.measurements import make_report_noisy_max
from opendp.metrics import linf_distance
from opendp.mod import (
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Metric,
    Queryable,
    binary_search_chain,
)


@dataclass(kw_only=True, frozen=True)
class MST:
    """MST mechanism from `MMS21 <https://arxiv.org/abs/2108.04978>`_."""

    alpha: float = 0.9
    """Proportion of budget to allocate to the measure step"""


def make_mst_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in: list[Any],  # list[Bound]
    d_out: float,
    releases: list[Any],  # list[LinearMeasurement]
    config: MST = MST(),
    estimator=mirror_descent,
):
    """Implements MST (Minimum Spanning Tree) over ordinal data.

    :param input_domain: domain of input data
    :param input_metric: how to compute distance between datasets
    :param output_measure: how to measure privacy of release
    :param d_in: distance between adjacent input datasets
    :param d_out: upper bound on the privacy loss
    :param releases: LinearMeasurement list of prior releases
    :param config: settings for the MST algorithm
    :param estimator: optimizer to use to fit a MarkovRandomField
    """
    import_optional_dependency("mbi")
    import mbi

    d_measure = d_out * config.alpha
    d_select = prior(prior(d_out - d_measure))

    cardinalities = get_cardinalities(input_domain)

    edges = list(itertools.combinations(input_domain.columns, 2))
    t_marginals = make_stable_marginals(
        input_domain, input_metric, cliques=edges, cardinalities=cardinalities
    )
    d_marginals = t_marginals.map(d_in)

    mbi_domain = mbi.Domain(input_domain.columns, shape=cardinalities.values())

    def function(qbl: Queryable) -> mbi.MarkovRandomField:
        model = estimator(mbi_domain, releases)

        # SELECT a set of queries that best reduces the error
        m_select = make_mst_select(
            *t_marginals.output_space,
            output_measure,
            d_in=d_marginals,
            d_out=d_select,
            edges=edges,
            model=model,
        )

        selected_cliques = qbl(m_select)

        # MEASURE selected marginals with noise
        m_measure = binary_search_chain(
            lambda s: make_noise_marginals(
                *t_marginals.output_space,
                output_measure,
                list(map(Count, selected_cliques)),
                scale=s,
            ),
            d_in=d_marginals,
            d_out=d_measure,
            T=float,
        )

        new_releases = qbl(m_measure)

        # GENERATE (fit a MarkovRandomField)
        return estimator(model.domain, releases + new_releases)

    return (
        t_marginals
        >> then_adaptive_composition(
            output_measure=output_measure,
            d_in=t_marginals.map(d_in),
            d_mids=[d_select, d_measure],
        )
        >> _new_pure_function(function)
    )


then_mst_marginals = to_then(make_mst_marginals)


def make_mst_select(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    output_measure: Measure,
    d_in,
    d_out,
    edges: list[tuple[str, str]],
    model,  # MarkovRandomField
) -> Measurement:
    """Make a measurement that selects a set of cliques that will minimize error."""
    import mbi
    from scipy.cluster.hierarchy import DisjointSet

    if not isinstance(model, mbi.MarkovRandomField):
        raise ValueError("model must be a MarkovRandomField")

    num_selections = len(model.domain.attrs) - 1
    d_select = prior(d_out / num_selections)

    t_mst_scores = make_mst_scores(input_domain, input_metric, edges, model)

    def function(qbl: Queryable) -> list[tuple[str, str]]:
        selected_edges = []
        components = DisjointSet(model.domain.attrs)

        for _ in range(num_selections):
            # filter down to only include edges that aren't connected
            unconnected = [
                i for i, e in enumerate(edges) if not components.connected(*e)
            ]

            t_unconnected = _make_transformation(
                *(t_mst_scores.output_space * 2),
                function=lambda scores: [scores[i] for i in unconnected],
                stability_map=lambda d_in: d_in,
            )

            m_rnm = binary_search_chain(
                lambda s: make_report_noisy_max(
                    *t_unconnected.output_space, output_measure=output_measure, scale=s
                ),
                d_in=d_in,
                d_out=d_select,
                T=float,
            )

            selected_edge = edges[unconnected[qbl(t_unconnected >> m_rnm)]]

            selected_edges.append(selected_edge)
            components.merge(*selected_edge)

        return selected_edges

    return (
        t_mst_scores
        >> then_adaptive_composition(
            output_measure,
            d_in=t_mst_scores.map(d_in),
            d_mids=[d_select] * num_selections,
        )
        >> _new_pure_function(function)
    )


def make_mst_scores(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    queries: list[tuple[str, str]],
    model,  # MarkovRandomField
):
    """Make a transformation that assigns a score representing how poorly each query is estimated."""
    from opendp.extras.numpy import NPArrayDDescriptor
    import numpy as np
    import mbi

    if not isinstance(input_domain.descriptor, DictDescriptor):
        raise ValueError("input_domain must be a DictDomain")

    if not all(
        isinstance(d, ExtrinsicDomain) and isinstance(d.descriptor, NPArrayDDescriptor)
        for d in input_domain.descriptor.values()
    ):
        raise ValueError("input_domain values must be dp.numpy.arrayd_domain")

    if not isinstance(model, mbi.MarkovRandomField):
        raise ValueError("model must be a MarkovRandomField")

    def score_query(query: tuple[str, str], exact) -> float:
        synth = model.project(query).values
        return np.linalg.norm(exact - synth, 1)

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=vector_domain(atom_domain(T="f64", nan=False)),
        output_metric=linf_distance(T="f64", monotonic=False),
        function=lambda exact_tabs: [
            score_query(query, exact_tabs[query]) for query in queries
        ],
        stability_map=lambda d_in: d_in,
    )
