from functools import reduce
from math import sqrt
from typing import Any, Callable, Literal, Optional, Sequence, Type, Union, cast
from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import (
    make_adaptive_composition,
    make_approximate,
    make_composition,
)
from opendp.extras._utilities import to_then
from opendp.extras.contingency._fixed import Fixed, then_fixed_marginals
from opendp.extras.contingency._aim import AIM, then_aim_marginals
from opendp.extras.contingency._mst import MST, then_mst_marginals
from opendp.measurements import make_private_lazyframe
from opendp.measures import max_divergence, zero_concentrated_divergence
from opendp.mod import (
    ApproximateDivergence,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    LazyFrameDomain,
    Queryable,
    Transformation,
    binary_search,
)
from opendp.domains import _lazyframe_from_domain
from opendp.transformations import make_stable_lazyframe
from opendp.typing import RuntimeType

_CONSTRUCTORS: dict[Type, Callable] = {
    AIM: then_aim_marginals,
    MST: then_mst_marginals,
    Fixed: then_fixed_marginals,
}


def make_contingency_table(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in: list[Any],  # list[Bound]
    d_marginals: float,
    d_keys: Optional[tuple[float, float]] = None,
    keys: Optional[dict[str, list[Any]]] = None,
    cuts: Optional[dict[str, list[float]]] = None,
    config: Union[AIM, MST, Fixed] = AIM(),
) -> Measurement:
    """Return a measurement that releases an approximation to a contingency table.

    :param input_domain: domain of input data
    :param input_metric: how to compute distance between datasets
    :param output_measure: how to measure privacy of release
    :param d_in: bounds on distances between adjacent input datasets
    :param d_marginals: upper bound on the privacy loss for estimating marginals
    :param d_keys: upper bound on the privacy loss for estimating stable keys
    :param keys: dictionary of column names and unique categories
    :param cuts: dictionary of column names and bin edges for numerical columns
    :param config: configuration for internal estimation algorithm
    """
    import_optional_dependency("mbi")
    import polars as pl
    import mbi

    plan = _lazyframe_from_domain(input_domain)
    schema = plan.collect_schema()

    then_marginals_algorithm = _CONSTRUCTORS.get(type(config))
    if then_marginals_algorithm is None:
        raise ValueError(f"algorithm ({config.__class__.__name__}) not recognized")

    keys = keys or dict()
    cuts = cuts or dict()

    for col, keyset in keys.items():
        keys[col] = pl.Series(col, keyset, dtype=schema[col])
    for col, cutset in cuts.items():
        cuts[col] = pl.Series(col, cutset, dtype=schema[col])

    if cuts:
        exprs = []
        for col, cutset in cuts.items():
            exprs.append(pl.col(col).cut(cutset))

            series = pl.Series(col, [], dtype=schema[col]).cut(cutset)
            keys[col] = series.cat.get_categories().cast(pl.Categorical)
        plan = plan.with_columns(exprs)

    if unwanted := set(keys) - set(input_domain.columns):  # pragma: no cover
        raise ValueError(f"keys has columns not present the domain: {unwanted}")
    if unwanted := set(cuts) - set(input_domain.columns):  # pragma: no cover
        raise ValueError(f"cuts has columns not present the domain: {unwanted}")

    if len(keys) == len(input_domain.columns):
        if d_keys:
            raise ValueError("d_keys must be None, because `keys` spans all columns")
        d_mids = [d_marginals]
        m_keys = None
    else:
        if not d_keys:
            raise ValueError(
                "d_keys must be set, because `keys` does not span all columns"
            )

        if RuntimeType.infer(d_keys) != output_measure.distance_type:
            raise ValueError(  # pragma: no cover
                f"d_keys type ({RuntimeType.infer(d_keys)}) "
                f"must match distance type ({output_measure.distance_type})"
            )

        d_mids = [d_keys, (d_marginals, 0.0)]
        m_keys, scale_keys = make_keys(
            input_domain, input_metric, output_measure, d_in, d_keys, keys, plan
        )

    m_comp = make_adaptive_composition(
        input_domain, input_metric, output_measure, d_in, d_mids
    )

    if isinstance(output_measure, ApproximateDivergence):
        inner_measure = output_measure
    else:
        inner_measure = output_measure

    if inner_measure == max_divergence():
        std_rescale = sqrt(2)
    elif inner_measure == zero_concentrated_divergence():
        std_rescale = 1
    else:
        raise ValueError(
            "output_measure must be (approximate) max_divergence() or zero_concentrated_divergence()"
        )

    def function(qbl: Queryable) -> ContingencyTable:
        releases = []
        stable_keys = keys.copy()

        if m_keys:
            stable_counts = qbl(m_keys)
            stable_keys |= {k: v[k] for k, v in stable_counts.items()}

            for name, counts in stable_counts.items():
                lm = mbi.LinearMeasurement(
                    noisy_measurement=counts["len"].to_numpy(),
                    clique=[name],
                    stddev=scale_keys * std_rescale,
                )
                releases.append(lm)

        t_index = make_indexed(input_domain, input_metric, stable_keys, plan)

        m_marginals = then_marginals_algorithm(
            output_measure=inner_measure,
            d_in=t_index.map(d_in),
            d_out=d_marginals,
            releases=releases,
            config=config,
        )

        m_index_marginals = t_index >> m_marginals

        if isinstance(output_measure, ApproximateDivergence):
            m_index_marginals = make_approximate(m_index_marginals)

        model = qbl(m_index_marginals)
        column_keys = [stable_keys[c] for c in input_domain.columns]

        return ContingencyTable(model=model, keys=column_keys, cuts=cuts)

    return m_comp >> _new_pure_function(function)


then_contingency_table = to_then(make_contingency_table)


def make_keys(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in,
    d_out,
    keys: dict[str, Any],
    plan: Any,
) -> Measurement:
    from opendp.extras.polars import dp_len

    def _marginals(scale: float, threshold: int) -> tuple[Measurement, float]:
        names = [name for name in input_domain.columns if name not in keys]
        one_way_measurements = [
            make_private_lazyframe(
                input_domain,
                input_metric,
                output_measure,
                plan.group_by(name).agg(dp_len()),
                global_scale=scale,
                threshold=threshold,
            )
            >> _new_pure_function(lambda x: x.collect())
            for name in names
        ]
        return make_composition(one_way_measurements) >> _new_pure_function(
            lambda r: dict(zip(names, r))
        )

    scale_uni = binary_search(
        lambda s: _marginals(s, threshold=2**32 - 1).map(d_in)[0] < d_out[0], T=float
    )
    threshold_uni = binary_search(
        lambda t: _marginals(scale_uni, threshold=t).map(d_in)[1] < d_out[1], T=int
    )

    return _marginals(scale_uni, threshold_uni), scale_uni


def make_indexed(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    keys: dict[str, Any],
    plan: Any,
) -> Transformation:
    import polars as pl

    replacers = (
        pl.col(name).replace_strict(
            keyset,
            pl.Series(list(range(len(keyset))), dtype=pl.UInt32),
            default=len(keyset),
        )
        for name, keyset in keys.items()
    )
    # discard any "default" category
    filters = (pl.col(col) < len(keyset) for col, keyset in keys.items())
    clips = (pl.col(name).clip(0, len(keyset) - 1) for name, keyset in keys.items())

    return make_stable_lazyframe(
        input_domain,
        input_metric,
        plan.with_columns(replacers)
        .filter(reduce(lambda l, r: l.and_(r), filters))
        .with_columns(clips),
    )


class ContingencyTable:
    """Approximate a contingency table via lower-order marginals."""

    def __init__(
        self,
        model: Optional[Any],  # Optional[MarkovRandomField]
        keys: list[Any],  # list[Series]
        cuts: dict[str, Any],  # dict[str, Series]
    ):
        from mbi import MarkovRandomField
        from polars import Series

        self.model = cast(MarkovRandomField, model)
        self.keys = cast(list[Series], keys)
        self.cuts = cast(dict[str, Series], cuts)

    def synthesize(
        self, rows: Optional[int] = None, method: Literal["round", "sample"] = "round"
    ):
        """Generate synthetic data that is consistent with the contingency table.

        :param rows: if not set, approximately matches number of rows in data
        :param method: "round" rounds to projection, "sample" samples from densities
        """
        import numpy as np
        import polars as pl

        def sample_numeric(indices, keys, cuts=None):
            if cuts is None:
                return keys[indices]

            if cuts.dtype.is_integer():
                edges = np.array([cuts[0] - 1, *cuts, cuts[-1] + 1])
                return np.random.randint(edges[indices], edges[indices + 1])
            elif cuts.dtype.is_float():
                edges = np.array([cuts[0], *cuts, cuts[-1]])
                diff = edges[indices + 1] - edges[indices]
                return edges[indices] + diff * np.random.random_sample(len(indices))

        synthetic_indices = self.model.synthetic_data(rows, method).df
        data = {
            keyset.name: sample_numeric(
                synthetic_indices[keyset.name].to_numpy(),
                keyset,
                cuts=self.cuts.get(keyset.name),
            )
            for keyset in self.keys
        }
        return pl.DataFrame(data)

    def project(self, attrs: str | Sequence[str]):
        """Return counts corresponding to combinations of self.keys."""
        return self.model.project(attrs).values

    @property
    def columns(self) -> list[str]:
        """Returns the columns spanned by the table."""
        return [keyset.name for keyset in self.keys]
