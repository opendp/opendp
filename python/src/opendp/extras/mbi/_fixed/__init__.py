"""Releases all queries in a fixed workload."""

from dataclasses import dataclass
from typing import Any, TYPE_CHECKING

from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.extras.mbi._utilities import (
    get_associated_metric,
    get_cardinalities,
    then_noise_marginals,
    make_stable_marginals,
    Algorithm,
    Count,
    Marginals,
    OnewayType,
    ONEWAY_UNKEYED
)
from opendp.mod import (
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    binary_search_chain,
)

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


@dataclass(kw_only=True, frozen=True)
class Fixed(Algorithm):
    """Releases all queries in a fixed workload."""

    queries: list[Count]
    """Workload of queries."""
    oneway: OnewayType = ONEWAY_UNKEYED
    """Only fit one-way marginals for columns missing keys.
    
    The fixed algorithm differs from other algorithms
    in that it only estimates marginals with missing keys, 
    not all unknown first-order marginals.
    """

    @property
    def needs_model(self) -> bool:
        # Fixed workloads are predetermined and do not need a model for
        # workload selection. The fixed release supplies the constraint used
        # to fit the final model.
        return False

    def __post_init__(self):
        super().__post_init__()

        if not self.queries:
            raise ValueError("queries must have at least one element")

        if not all(isinstance(q, Count) for q in self.queries):
            raise ValueError("queries must be of type Count")

    def make_marginals(
        self,
        input_domain: LazyFrameDomain,
        input_metric: FrameDistance,
        output_measure: Measure,
        d_in: list["Bound"],
        d_out: float,
        *,
        marginals: Marginals,
        model: Any,  # MarkovRandomField
    ):
        """Implements a "Fixed" algorithm over ordinal data.

        The algorithm estimates fixed cliques.

        :param input_domain: domain of input data
        :param input_metric: how to compute distance between datasets
        :param output_measure: how to measure privacy of release
        :param d_in: distance between adjacent input datasets
        :param d_out: upper bound on the privacy loss
        :param marginals: prior marginal releases
        :param model: warm-start fit of MarkovRandomField
        """
        import_optional_dependency("mbi")
        from mbi import Domain, MarkovRandomField  # type: ignore[import-untyped,import-not-found]

        if model is not None and not isinstance(model, MarkovRandomField):
            raise ValueError("model must be a MarkovRandomField")

        lp_metric = get_associated_metric(output_measure)
        cliques = [q.by for q in self.queries]
        weights = [q.weight for q in self.queries]

        def make(scale: float) -> Measurement:
            return make_stable_marginals(
                input_domain, input_metric, lp_metric, cliques  # type: ignore[arg-type]
            ) >> then_noise_marginals(
                output_measure, cliques, scale, weights
            )  # type: ignore[return-type]

        m_marginals = binary_search_chain(make, d_in, d_out, T=float)

        def function(
            new_releases: list,
        ) -> tuple[Marginals, MarkovRandomField]:
            all_marginals = marginals.add(*new_releases)

            if model is None:
                model_domain = Domain.fromdict(get_cardinalities(input_domain))
                potentials = None
            else:
                model_domain = model.domain
                potentials = model.potentials

            new_model = self.estimator(
                model_domain,
                all_marginals.flatten(),
                potentials=potentials,
            )
            return all_marginals, new_model

        return m_marginals >> _new_pure_function(function)
