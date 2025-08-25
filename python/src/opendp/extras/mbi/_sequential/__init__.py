"""Release multiple algorithms sequentially."""

from dataclasses import dataclass
from typing import Any, Optional, TYPE_CHECKING

from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import make_adaptive_composition
from opendp.extras.mbi._utilities import (
    Algorithm,
    get_associated_metric,
    get_cardinalities,
)
from opendp.metrics import frame_distance, symmetric_distance
from opendp.mod import (
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Queryable,
    binary_search_param,
)

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


@dataclass(kw_only=True, frozen=True)
class Sequential(Algorithm):
    """Release multiple algorithms sequentially.

    Use this in conjunction with Fixed to ensure specific marginals are estimated.
    """

    algorithms: list[Algorithm]
    """Sequence of algorithms."""
    weights: Optional[list[float]] = None
    """Budget allocation amongst algorithms.
    
    Defaults to equal budget for each algorithm."""

    def __post_init__(self):
        super().__post_init__()

        if not self.algorithms:
            raise ValueError("algorithms must contain at least one element")

        if any(not isinstance(a, Algorithm) for a in self.algorithms):
            raise ValueError(f"algorithms ({self.algorithms}) must be instances of Algorithm")

        if self.weights is not None:
            if len(self.weights) != len(self.algorithms):
                msg = "algorithms and weights must contain the same number of elements"
                raise ValueError(msg)

            if any(w <= 0 for w in self.weights):
                raise ValueError(f"weights ({self.weights}) must be positive")

    def make_marginals(
        self,
        input_domain: LazyFrameDomain,
        input_metric: FrameDistance,
        output_measure: Measure,
        d_in: list["Bound"],
        d_out: float,
        *,
        marginals: dict[tuple[str, ...], Any],
        model: Any,  # MarkovRandomField
    ):
        """Implements a "Sequential" algorithm over ordinal data.

        The algorithm applies each algorithm sequentially.

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
        import numpy as np  # type: ignore[import-not-found]

        get_cardinalities(input_domain)
        if input_metric != frame_distance(symmetric_distance()):
            message = f"input_metric ({input_metric}) must be frame_distance(symmetric_distance())"
            raise ValueError(message)
        get_associated_metric(output_measure)

        if not isinstance(model, MarkovRandomField):
            raise ValueError("model must be a MarkovRandomField")

        weights = np.array(self.weights or [1] * len(self.algorithms))

        scale = binary_search_param(
            lambda scale: make_adaptive_composition(
                input_domain, input_metric, output_measure, d_in, d_mids=weights * scale
            ),
            d_in=d_in,
            d_out=d_out,
            T=float,
        )
        d_mids = weights * scale

        def function(
            qbl: Queryable,
        ) -> tuple[dict[tuple[str, ...], Any], MarkovRandomField]:
            all_marginals = marginals.copy()
            current_model = model
            for algo, d_mid in zip(self.algorithms, d_mids):
                m_algo = algo.make_marginals(
                    input_domain,
                    input_metric,
                    output_measure,
                    d_in,
                    d_mid,
                    marginals=all_marginals,
                    model=current_model,
                )

                all_marginals, current_model = qbl(m_algo)

            return all_marginals, current_model

        return make_adaptive_composition(
            input_domain, input_metric, output_measure, d_in, d_mids=d_mids
        ) >> _new_pure_function(function)
