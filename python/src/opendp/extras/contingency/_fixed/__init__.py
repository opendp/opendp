from dataclasses import dataclass
from typing import Any

from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras._utilities import to_then
from opendp.extras.contingency._utilities import (
    mirror_descent,
    then_noise_marginals,
    Count,
    get_cardinalities
)
from opendp.extras.contingency._aim import make_stable_marginals
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.metrics import frame_distance, symmetric_distance
from opendp.mod import FrameDistance, LazyFrameDomain, Measure, binary_search_chain


@dataclass(kw_only=True, frozen=True)
class Fixed:
    queries: list[Count]
    """Workload of queries, each query is a list of columns to group by."""


def make_fixed_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    releases: list[Any],  # list[LinearMeasurement]
    config: Fixed,
    estimator=mirror_descent,
):
    """Implements a "Fixed" algorithm over ordinal data.

    The algorithm estimates the fixed cliques specified in the config.

    :param input_domain: domain of input data
    :param input_metric: how to compute distance between datasets
    :param output_measure: how to measure privacy of release
    :param d_in: distance between adjacent input datasets
    :param d_out: upper bound on the privacy loss
    :param releases: LinearMeasurement list of prior releases
    :param config: settings for the Fixed algorithm
    :param estimator: optimizer to use to fit a MarkovRandomField
    """
    import_optional_dependency("mbi")
    import mbi

    cards = get_cardinalities(input_domain)

    if input_metric != frame_distance(symmetric_distance()):
        raise ValueError(
            f"input_metric ({input_metric}) must be frame_distance(symmetric_distance())"
        )

    if output_measure not in {max_divergence(), zero_concentrated_divergence()}:
        raise ValueError(
            f"output_measure ({output_measure}) must be max_divergence or zero_concentrated_divergence"
        )

    cliques = [query.by for query in config.queries]
    m_marginals = binary_search_chain(
        lambda s: (
            make_stable_marginals(input_domain, input_metric, cliques, cards)
            >> then_noise_marginals(output_measure, config.queries, s)
        ),
        d_in,
        d_out,
    )

    mbi_domain = mbi.Domain(input_domain.columns, cards.values())

    def function(new_releases: list[Any]) -> mbi.MarkovRandomField:
        return estimator(mbi_domain, [*releases, *new_releases])

    return m_marginals >> _new_pure_function(function)


then_fixed_marginals = to_then(make_fixed_marginals)
