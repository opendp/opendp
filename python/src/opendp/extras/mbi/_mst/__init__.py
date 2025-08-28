"""MST mechanism from `MMS21 <https://arxiv.org/abs/2108.04978>`_."""

from dataclasses import dataclass
import itertools
from typing import Any, Optional, cast, TYPE_CHECKING

from opendp._internal import _make_transformation, _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import then_adaptive_composition
from opendp.domains import atom_domain, vector_domain
from opendp.extras.mbi._utilities import (
    TypedDictDistance,
    TypedDictDomain,
    get_associated_metric,
    make_noise_marginals,
    prior,
    make_stable_marginals,
    weight_marginals,
    Algorithm,
)
from opendp.measurements import make_noisy_max
from opendp.metrics import linf_distance
from opendp.mod import (
    ExtrinsicDistance,
    ExtrinsicDomain,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    Queryable,
    binary_search_chain,
)

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


@dataclass(kw_only=True, frozen=True)
class MST(Algorithm):
    """MST mechanism from `MMS21 <https://arxiv.org/abs/2108.04978>`_.
    
    MST greedily chooses pairs of columns that are most poorly represented 
    by the DP contingency table in a way that guarantees all columns become 
    connected by a minimum spanning tree.
    MST then releases all of the selected marginals.

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

        >>> table_mst = (
        ...     context.query(rho=0.1, delta=1e-7)
        ...     # transformations/truncation may be applied here
        ...     .select("SEX", "AGE", "HWUSUAL", "ILOSTAT")
        ...     .contingency_table(
        ...         keys={"SEX": [1, 2]}, 
        ...         cuts={"AGE": [20, 40, 60], "HWUSUAL": [1, 20, 40]},
        ...         algorithm=dp.mbi.MST()
        ...     )
        ...     .release()
        ... )

        >>> table_mst.synthesize() # doctest: +SKIP
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
    """

    measure_split: float = 0.9
    """Remaining proportion of budget to allocate to measuring marginals.
    
    The complement is spent on selecting marginals."""
    num_selections: Optional[int] = None
    """Number of second-order marginals to estimate.
    
    Defaults to one fewer than the number of columns in the data."""

    def __post_init__(self):
        super().__post_init__()

        if self.measure_split is not None and not (0 < self.measure_split <= 1):
            raise ValueError(f"measure_split ({self.measure_split}) must be in (0, 1]")

        if self.num_selections is not None and self.num_selections < 1:
            raise ValueError(f"num_selections ({self.num_selections}) must be positive")

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
    ) -> Measurement:
        """Implements MST (Minimum Spanning Tree) over ordinal data.

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

        d_measure = d_out * self.measure_split
        d_select = prior(prior(d_out - d_measure))

        lp_metric = get_associated_metric(output_measure)
        edges = list(itertools.combinations(input_domain.columns, 2))

        t_marginals = make_stable_marginals(input_domain, input_metric, lp_metric, edges)  # type: ignore[arg-type]
        d_marginals = t_marginals.map(d_in)

        def function(
            qbl: Queryable,
        ) -> tuple[dict[tuple[str, ...], Any], MarkovRandomField]:

            # SELECT a set of queries that best reduces the error
            m_select = _make_mst_select(
                *t_marginals.output_space,
                output_measure,
                d_in=d_marginals,
                d_out=d_select,
                edges=edges,
                model=model,
                num_selections=self.num_selections,
            )

            selected_cliques = qbl(m_select)

            # MEASURE selected marginals with noise
            m_measure = binary_search_chain(
                lambda s: make_noise_marginals(
                    *t_marginals.output_space,
                    output_measure,
                    selected_cliques,
                    scale=s,
                ),
                d_in=d_marginals,
                d_out=d_measure,
                T=float,
            )

            all_marginals = weight_marginals(marginals, *qbl(m_measure))

            # GENERATE (fit a MarkovRandomField)
            new_model = self.estimator(
                model.domain,
                list(all_marginals.values()),
                potentials=model.potentials.expand(model.cliques + selected_cliques),
            )
            return all_marginals, new_model

        return (
            t_marginals
            >> then_adaptive_composition(
                output_measure=output_measure,
                d_in=t_marginals.map(d_in),
                d_mids=[d_select, d_measure],
            )
            >> _new_pure_function(function)
        )


def _make_mst_select(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    output_measure: Measure,
    d_in,
    d_out,
    edges: list[tuple[str, str]],
    model,  # MarkovRandomField
    num_selections: Optional[int] = None,
) -> Measurement:
    """Make a measurement that selects a set of cliques that will minimize error."""
    from mbi import MarkovRandomField  # type: ignore[import-not-found]
    from scipy.cluster.hierarchy import DisjointSet  # type: ignore[import-untyped,import-not-found]

    model = cast(MarkovRandomField, model)

    max_selections = len(model.domain.attrs) - 1
    num_selections = min(max(0, num_selections or max_selections), max_selections)
    d_select = prior(d_out / num_selections)

    t_mst_scores = _make_mst_scores(input_domain, input_metric, edges, model)
    d_mst_scores = t_mst_scores.map(d_in)

    def function(qbl: Queryable) -> list[tuple[str, str]]:
        selected_edges = []
        components = DisjointSet(model.domain.attrs)

        for _ in range(num_selections):
            # filter down to only include edges that aren't connected
            unconnected = [
                i for i, e in enumerate(edges) if not components.connected(*e)
            ]

            t_unconnected = _make_transformation(  # type: ignore[misc]
                *t_mst_scores.output_space,
                *t_mst_scores.output_space,
                function=lambda scores: [scores[i] for i in unconnected],
                stability_map=lambda d_in: d_in,
            )

            m_rnm = binary_search_chain(
                lambda s: make_noisy_max(
                    *t_unconnected.output_space, output_measure=output_measure, scale=s
                ),
                d_in=d_mst_scores,
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
            d_in=d_mst_scores,
            d_mids=[d_select] * num_selections,
        )
        >> _new_pure_function(function)
    )


def _make_mst_scores(
    input_domain: ExtrinsicDomain,
    input_metric: ExtrinsicDistance,
    queries: list[tuple[str, str]],
    model,  # MarkovRandomField
):
    """Make a transformation that assigns a score representing how poorly each query is estimated."""
    from opendp.extras.numpy import NPArrayDDomain
    import numpy as np  # type: ignore[import-not-found]
    from mbi import MarkovRandomField  # type: ignore[import-not-found]

    for value_domain in input_domain.cast(TypedDictDomain).values():
        value_domain.cast(NPArrayDDomain)

    input_metric.cast(TypedDictDistance)

    model = cast(MarkovRandomField, model)

    def score_query(query: tuple[str, str], exact) -> float:
        synth = model.project(query).values
        return float(np.linalg.norm(exact - synth, 1))

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=vector_domain(atom_domain(T="f64", nan=False)),
        output_metric=linf_distance(T="f64", monotonic=False),
        function=lambda exact_tabs: [
            score_query(query, exact_tabs[query]) for query in queries
        ],
        stability_map=lambda d_in: max(d_in.values()),
    )
