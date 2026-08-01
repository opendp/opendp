"""High-level mechanism for applying mbi mechanisms to dataframes with mixed types."""

from math import sqrt
from dataclasses import dataclass, field
from typing import (
    Any,
    Iterator,
    Literal,
    Mapping,
    Optional,
    Sequence,
    TypeAlias,
    Union,
    cast,
    TYPE_CHECKING,
)
from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import (
    make_adaptive_composition,
    make_approximate,
    make_composition,
)
from opendp.extras.mbi._aim import AIM
from opendp.extras.mbi._utilities import (
    prior,
    get_std,
    identity_query_precision,
    row_major_order,
    SelectPrefixQuery,
    Algorithm,
    Marginals,
    ONEWAY_UNKEYED,
)
from opendp.measurements import make_private_lazyframe
from opendp.mod import (
    ApproximateDivergence,
    FrameDistance,
    LazyFrameDomain,
    Measure,
    Measurement,
    OpenDPException,
    Queryable,
    binary_search,
    binary_search_param,
)
from opendp.domains import _lazyframe_from_domain
from opendp.transformations import make_stable_lazyframe
from opendp.typing import RuntimeType


if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.polars import Bound


@dataclass
class ContingencyTable:
    """Approximate a k-way contingency table via low-order marginals."""

    keys: dict[str, Any]  # dict[str, Series]
    """Unique keys for each column"""
    cuts: dict[str, Any]  # dict[str, Series]
    """Cut points (left-open) for each numeric column"""
    marginals: Marginals
    """Measurements used to fit the model, keyed by clique"""
    model: Any  # MarkovRandomField
    """MarkovRandomField spanning the same columns as keys"""
    thresholds: dict[str, int] = field(default_factory=dict)
    """Cut-off point for discovered stable keys. 
    Any category appearing fewer than threshold times is attributed to the null category."""

    def __post_init__(self):
        from mbi import MarkovRandomField, Domain  # type: ignore[import-untyped,import-not-found]
        import polars as pl  # type: ignore[import-not-found]

        if not isinstance(self.model, MarkovRandomField):  # pragma: no cover
            raise ValueError("model must be a MarkovRandomField")

        for col in self.keys:
            self.keys[col] = pl.Series(col, self.keys[col])
        for col in self.cuts:
            self.cuts[col] = pl.Series(col, self.cuts[col])

        key_shape = set((c, len(k)) for c, k in self.keys.items())
        if set(self.model.domain.config.items()) != key_shape:
            raise ValueError(
                "Model domain must match key attrs and sizes.\n"
                f"  Model shape: {self.model.domain}\n"
                f"  Key shape:   {Domain.fromdict(dict(key_shape))}"
            )

        for col, cutset in self.cuts.items():
            if col not in self.keys:
                raise ValueError(f'"{col}" in cuts is not present in keys')
            if len(self.keys[col]) - 1 != len(cutset):
                msg = f'"{col}" keyset length ({len(self.keys[col])}) must be one greater than the number of cuts ({len(cutset)})'
                raise ValueError(msg)

        if isinstance(self.marginals, dict):
            self.marginals = Marginals({
                tuple(clique): [measurement]
                for clique, measurement in self.marginals.items()
            })

    def synthesize(
        self, rows: Optional[int] = None, method: Literal["round", "sample"] = "round"
    ):
        """Generate synthetic data that is consistent with the contingency table.

        :param rows: if not set, approximately matches number of rows in original data
        :param method: "round" rounds to projection, "sample" samples from densities
        """
        import polars as pl  # type: ignore[import-not-found]
        from mbi import MarkovRandomField  # type: ignore[import-untyped,import-not-found]

        model = cast(MarkovRandomField, self.model)

        indices = model.synthetic_data(rows, method).df
        data = {
            c: _deindex(indices[c].to_numpy(), self.keys[c], cuts=self.cuts.get(c))
            for c in self.keys
        }
        return pl.DataFrame(data)

    def project(self, attrs: str | Sequence[str]):
        """Return counts corresponding to each combination of self.keys in `attrs` in an ndarray.

        :param attrs: attributes to preserve. All other attributes are marginalized.
        """
        from mbi import MarkovRandomField  # type: ignore[import-untyped,import-not-found]

        model = cast(MarkovRandomField, self.model)
        return model.project(attrs).values

    def project_melted(self, attrs: str | Sequence[str]):
        """Return counts corresponding to each combination of self.keys in `attrs` in a dataframe.

        :param attrs: attributes to preserve. All other attributes are marginalized.
        """
        import polars as pl
        import numpy as np

        attrs = [attrs] if isinstance(attrs, str) else list(attrs)

        unknown = [attr for attr in attrs if attr not in self.keys]
        if unknown:
            raise ValueError(f"attrs ({unknown}) are not present in keys")

        lengths_arr = np.asarray(self.project(attrs).ravel())
        lengths = pl.Series("len", lengths_arr).to_frame()

        if not attrs:
            # row_major_order has no initializer for an empty iterator of
            # keysets; the zero-way projection is a single-row frame
            return lengths

        return row_major_order(self.keys[attr] for attr in attrs).hstack(lengths)

    def std(self, attrs: Sequence[str]) -> float:
        """Estimate the standard deviation of the count estimate.
        The estimate does not take cuts into account.

        When marginals are estimated with gaussian noise,
        the consistent (variance-weighted) marginals are also gaussian-distributed.
        Therefore the standard deviation matches the noise scale parameter.

        When marginals are estimated with laplace noise,
        the consistent marginals are a sum of laplacians, which is not laplace-distributed.
        Therefore the standard deviation doesn't correspond to the laplace distribution.
        Under the central limit theorem, the distribution will tend towards gaussian.

        If the estimate is gaussian-distributed,
        use :py:func:`~opendp.accuracy.gaussian_scale_to_accuracy`
        to construct a confidence interval.

        Only complete identity measurements over a clique that covers
        ``attrs`` contribute precision. Marginals released via a partial
        (prefix) query don't contribute a per-cell precision, so this may
        raise if ``attrs`` is only covered by such measurements.

        :param attrs: attributes to preserve in uncertainty estimate
        """
        from mbi import MarkovRandomField  # type: ignore[import-untyped,import-not-found]

        model = cast(MarkovRandomField, self.model)

        if isinstance(attrs, str):  # pragma: no cover
            raise ValueError(f"attrs ({attrs}) must be a sequence of strings")

        attrs_clique = set(attrs)
        size = model.domain.size

        inv_var_sum = 0.0
        for measurement in self.marginals.flatten():
            clique = measurement.clique
            # a measurement informs a projection only if it was taken over a
            # clique that covers (is a superset of) the requested attrs --
            # marginalizing down sums independent noisy cells together
            if not attrs_clique.issubset(clique):
                continue
            inv_var_sum += identity_query_precision(measurement) / (
                size(clique) / size(attrs_clique)
            )

        if inv_var_sum == 0:
            raise ValueError(f"attrs ({attrs}) are not covered by the query set")

        return sqrt(1 / inv_var_sum)

    @property
    def schema(self) -> dict[str, Any]:
        """Returns the data schema."""
        return {keyset.name: keyset.dtype for keyset in self.keys.values()}


def make_contingency_table(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in: list["Bound"],
    d_out: Union[float, tuple[float, float]],
    keys: Optional[Mapping[str, Sequence]] = None,
    cuts: Optional[Mapping[str, Sequence[float]]] = None,
    table: Optional[ContingencyTable] = None,
    algorithm: Algorithm = AIM(),
) -> tuple[Measurement, Optional[float], Optional[int]]:
    """Return a measurement that releases a :class:`.ContingencyTable`,
    as well as the noise scale and optional threshold for one-way marginals.

    :param input_domain: domain of input data
    :param input_metric: how to compute distance between datasets
    :param output_measure: how to measure privacy of release
    :param d_in: bounds on distances between adjacent input datasets
    :param d_out: upper bound on the privacy loss
    :param keys: dictionary of column names and unique categories
    :param cuts: dictionary of column names and left-open bin edges
    :param table: ContingencyTable from prior release
    :param algorithm: configuration for internal estimation algorithm
    """
    import_optional_dependency("mbi")
    import polars as pl
    import mbi

    plan = _lazyframe_from_domain(input_domain)
    schema = plan.collect_schema()

    keys_pl: dict[str, pl.Series] = {
        col: _unique(_with_null(pl.Series(col, keyset, dtype=schema.get(col))), "keys")
        for col, keyset in (keys or dict()).items()
    }

    cuts_pl: dict[str, pl.Series] = {
        col: _unique(
            _increasing(
                _exterior_representable(
                    _finite(
                        _non_null(
                            _nonempty(pl.Series(col, cutset, dtype=schema.get(col)))
                        )
                    )
                )
            ),
            "cuts",
        )
        for col, cutset in (cuts or dict()).items()
    }

    # add cut bin labels to keys
    def get_categories(cutset):
        labels = [f"({lb}, {rb}]" for lb, rb in zip(["-inf", *cutset], [*cutset, "inf"])]
        return pl.Series(cutset.name, labels)

    keys_pl |= {col: get_categories(cutset) for col, cutset in cuts_pl.items()}

    if table:
        keys_pl |= table.keys
        cuts_pl |= table.cuts
        model = cast(mbi.MarkovRandomField, table.model)
        marginals = table.marginals.copy()
        thresholds = table.thresholds.copy()
    else:
        model = None
        marginals = Marginals()
        thresholds = {}

    if cuts_pl:
        plan = plan.with_columns(pl.col(c).cut(cutset, labels=get_categories(cutset)) for c, cutset in cuts_pl.items() if c in schema)  # type: ignore[arg-type]

    if (QO := RuntimeType.infer(d_out)) != output_measure.distance_type:
        raise ValueError(f"d_out type ({QO}) must be {output_measure.distance_type}")

    delta = None
    if isinstance(d_out, tuple):
        d_out, delta = d_out

    all_keys_known = set(input_domain.columns) <= set(keys_pl)
    if delta and all_keys_known:
        message = f"delta ({delta}) must be zero because keys and cuts span all columns"
        raise ValueError(message)
    if not delta and not all_keys_known:
        message = f"delta ({delta}) must be nonzero because keys and cuts don't span all columns"
        raise ValueError(message)

    if algorithm.oneway_split is None:
        oneway_split = 0.5
        if algorithm.oneway == ONEWAY_UNKEYED:
            columns = set(input_domain.columns)
            oneway_split *= len(columns - set(keys_pl)) / len(columns)
    else:
        oneway_split = algorithm.oneway_split

    if oneway_split == 0:
        # the oneway phase is the only place private keys are discovered for
        # columns without a known keyset, so it can't be skipped for those
        if not all_keys_known:
            message = "oneway_split must be positive when discovering private keysets"
            raise ValueError(message)

        d_multiway: Union[float | tuple[float, float]] = d_out
        d_mids = [d_multiway if delta is None else (d_multiway, 0.0)]
        m_oneway = None
        scale, threshold = None, None

    else:
        d_oneway = d_out * oneway_split
        d_multiway = float(prior(prior(d_out - d_oneway)))
        if delta is not None:
            d_oneway = d_oneway, delta  # type: ignore[assignment]

        d_mids = [d_oneway, d_multiway if delta is None else (d_multiway, 0.0)]  # type: ignore[has-type]
        has_prior_total = any(
            identity_query_precision(measurement) > 0
            for measurement in marginals.flatten()
        )
        m_oneway, scale, threshold = _make_oneway_marginals(
            input_domain,
            input_metric,
            output_measure,
            d_in=d_in,
            d_out=d_oneway,
            keys=keys_pl,
            plan=plan,
            unknown_only=algorithm.oneway == ONEWAY_UNKEYED,
            has_prior_total=has_prior_total
        )
        if threshold is not None:
            thresholds |= {
                c: threshold for c in input_domain.columns if c not in keys_pl
            }

    m_comp = make_adaptive_composition(
        input_domain, input_metric, output_measure, d_in, d_mids
    )

    if isinstance(output_measure, ApproximateDivergence):
        inner_measure = output_measure.inner_measure
    else:
        inner_measure = output_measure

    if "len" in input_domain.columns:
        raise ValueError('input_domain must not contain a column named "len"')

    def postprocess(qbl: Queryable) -> ContingencyTable:
        stable_keys = keys_pl.copy()
        current_marginals = marginals.copy()

        if m_oneway:
            new_keys, oneway_releases = qbl(m_oneway)
            stable_keys |= new_keys
            current_marginals = current_marginals.add(*oneway_releases)

        mbi_domain = mbi.Domain.fromdict({c: len(k) for c, k in stable_keys.items()})

        potentials = None
        if isinstance(model, mbi.MarkovRandomField):
            import attr  # type: ignore[import-not-found]

            potentials = attr.evolve(model.potentials, domain=mbi_domain)

        current_model = algorithm.estimator(
            mbi_domain, current_marginals.flatten(), potentials=potentials
        )

        t_index = make_stable_lazyframe(
            input_domain,
            input_metric,
            plan.with_columns(
                pl.col(c)
                .replace_strict(
                    stable_keys[c],
                    pl.Series(range(len(stable_keys[c])), dtype=pl.UInt32),
                    default=_get_null_index(stable_keys[c], len(stable_keys[c]) - 1),
                )
                .clip(0, len(stable_keys[c]) - 1)
                for c in input_domain.columns
            ),
            MO="FrameDistance<SymmetricDistance>"
        )

        m_marginals = algorithm.make_marginals(
            input_domain=t_index.output_domain,
            input_metric=t_index.output_metric,
            output_measure=inner_measure,
            d_in=t_index.map(d_in),
            d_out=d_multiway,  # type: ignore[arg-type]
            marginals=current_marginals,
            model=current_model,
        )

        m_index_marginals = t_index >> m_marginals

        if delta is not None:
            m_index_marginals = make_approximate(m_index_marginals)

        T: TypeAlias = tuple[Marginals, mbi.MarkovRandomField]
        current_marginals, current_model = cast(T, qbl(m_index_marginals))

        ordered_keys = {c: stable_keys[c] for c in input_domain.columns}
        ordered_keys |= stable_keys

        return ContingencyTable(
            keys=ordered_keys,
            cuts=cuts_pl,
            marginals=current_marginals,
            model=current_model,
            thresholds=thresholds,
        )

    return m_comp >> _new_pure_function(postprocess), scale, threshold


def _unique(series, arg: str):
    """raise error if elements are not unique"""
    if series.n_unique() != series.len():  # pragma: no cover
        # this is actually tested, but coverage doesn't see it
        raise ValueError(f'{arg} must be unique: "{series.name}" has duplicates')
    return series


def _increasing(series):
    """raise error if elements are not increasing"""
    import polars as pl

    diff = series.cast(pl.Float64).diff(null_behavior="drop")
    if diff.le(0).any():  # pragma: no cover
        # this is actually tested, but coverage doesn't see it
        message = f'cuts must be strictly increasing: "{series.name}" is not strictly increasing'
        raise ValueError(message)
    return series


def _nonempty(series):
    """raise error if a cutset is empty"""
    if series.len() == 0:
        raise ValueError(f'cuts must not be empty: "{series.name}" has no cut points')
    return series


def _non_null(series):
    """raise error if a cutset contains a null value

    Downstream checks (`_finite`'s `is_finite`, `_increasing`'s `diff`,
    `_unique`'s `n_unique`) don't reliably reject a null on their own, so
    this must run before them."""
    if series.null_count():
        raise ValueError(f'cuts must not contain null values: "{series.name}"')
    return series


def _finite(series):
    """raise error if a float cutset contains nan, +inf, or -inf"""
    if series.dtype.is_float() and not series.is_finite().all():
        message = f'cuts must be finite: "{series.name}" contains a nan or infinite value'
        raise ValueError(message)
    return series


def _exterior_representable(series):
    """raise error if a cutset leaves no finite, representable value for the
    final (upper) exterior bin `(cuts[-1], +inf]`

    The first bin `(-inf, cuts[0]]` never needs a value beyond `cuts[0]`
    itself, so the dtype minimum is a valid first cut. But `_deindex` must
    produce a value strictly greater than `cuts[-1]`; if `cuts[-1]` is
    already at (or, for floats, too close to) the dtype's maximum, no such
    value is representable."""
    import numpy as np  # type: ignore[import-not-found]

    cut_values = series.to_numpy()

    if series.dtype.is_integer():
        info = np.iinfo(cut_values.dtype)
        if cut_values[-1] == info.max:
            message = (
                f'"{series.name}" integer cuts: the last cut cannot be the '
                f"dtype maximum ({info.max}), since the final exterior bin "
                "would contain no representable value"
            )
            raise ValueError(message)

    elif series.dtype.is_float():
        with np.errstate(over="ignore"):
            next_value = np.nextafter(cut_values[-1], np.inf)
        if not np.isfinite(next_value):
            message = (
                f'"{series.name}" float cuts must leave a finite representable '
                "value above the final cut"
            )
            raise ValueError(message)

    return series


def _with_null(series):
    """create new series leading with null if null is not present"""
    return (
        series if series.is_null().any() else series.clone().append(series.clear(n=1))
    )


def _get_null_index(series, default=None) -> Optional[int]:
    """returns the index of null in a polars `series`, else `default`"""
    null_indices = series.is_null().arg_true()
    return null_indices.item() if null_indices.len() else default


def _make_oneway_marginals(
    input_domain: LazyFrameDomain,
    input_metric: FrameDistance,
    output_measure: Measure,
    d_in,
    d_out,
    keys: dict[str, Any],
    plan: Any,
    unknown_only: bool,
    has_prior_total: bool,
) -> tuple[Measurement, float, Optional[int]]:
    """Returns a measurement that releases keys and counts for one-way marginals,
    as well as the discovered scale and threshold."""
    from opendp.extras.polars import dp_len
    import polars as pl  # type: ignore[import-not-found]
    from mbi import LinearMeasurement  # type: ignore[import-not-found]
    import numpy as np  # type: ignore[import-not-found]

    def group_by_agg(plan: pl.LazyFrame, name: str) -> pl.LazyFrame:
        """Aggregate the compute plan"""
        keyset = keys.get(name)
        if keyset is None:
            return plan.group_by(name).agg(dp_len(signed=True))

        # fold all unknown values into null
        if isinstance(keyset.dtype, pl.Categorical):  # pragma: no cover
            keyset = keyset.cast(pl.Enum(keyset.cast(pl.String)))
        replace = pl.col(name).replace_strict(keys[name], keyset, default=None)

        return pl.LazyFrame([keyset]).join(
            plan.select(replace).group_by(name).agg(dp_len(signed=True)),
            on=[name],
            how="left",
            nulls_equal=True,
        )

    def _make(scale: float, threshold: Optional[int] = None) -> Measurement:
        std = get_std(output_measure, scale)

        # all columns that should be estimated
        names = [
            name
            for name in input_domain.columns
            if name not in keys or not unknown_only
        ]

        will_release_full_identity = any(
            name in keys
            for name in names
        )

        # estimating the total from an identity marginal requires a queried
        # column with known keys; when `unknown_only`, keyed columns are
        # excluded from `names`, so no such marginal is ever available.
        should_total = not (has_prior_total or will_release_full_identity)

        one_way_measurements = [
            make_private_lazyframe(
                input_domain,
                input_metric,
                output_measure,
                group_by_agg(plan, name),
                global_scale=scale,
                threshold=threshold,
            )
            >> _new_pure_function(lambda x: x.collect())
            for name in names
        ]
        if should_total:
            m_total = make_private_lazyframe(
                input_domain,
                input_metric,
                output_measure,
                plan.select(dp_len(signed=True)),
                global_scale=scale,
            ) >> _new_pure_function(lambda x: x.collect())
            one_way_measurements.append(m_total)

        def postprocess(counts: list[pl.DataFrame]):
            new_keys = dict()
            lm_counts = []

            if should_total:
                total = counts.pop().item()
                lm_counts.append(
                    LinearMeasurement(
                        np.asarray([total], dtype=float), clique=(), stddev=std
                    )
                )

            for name, count in zip(names, counts):
                count = count.sort(name)
                lookup = dict(zip(count[name], count["len"]))

                if name in keys:
                    values_iter: Iterator[int] = (lookup[k] for k in keys[name])
                    values = np.fromiter(values_iter, dtype=np.int64)
                    lm_counts.append(
                        LinearMeasurement(values, clique=[name], stddev=std)
                    )
                else:
                    dtype = count[name].dtype
                    observed_keys = [key for key in lookup if key is not None]

                    # Null represents both actual nulls and values omitted by
                    # private key discovery. Keep it as a latent final cell.
                    new_keys[name] = pl.Series(
                        name, [*observed_keys, None], dtype=dtype
                    )

                    if observed_keys:
                        values = np.fromiter(
                            (lookup[key] for key in observed_keys), dtype=np.int64
                        )
                        lm_counts.append(
                            LinearMeasurement(
                                values,
                                clique=[name],
                                stddev=std,
                                query=SelectPrefixQuery(len(observed_keys)),
                            )
                        )

            return new_keys, lm_counts

        return make_composition(one_way_measurements) >> _new_pure_function(postprocess)

    if isinstance(output_measure, ApproximateDivergence):
        compare_s = lambda s: _make(s, threshold=2**32 - 1).map(d_in)[0] < d_out[0]  # type: ignore[index]
        scale = binary_search(compare_s, T=float)

        try:
            _make(scale, threshold=None)
            threshold: Optional[int] = None  # pragma: no cover
        except OpenDPException:
            compare_t = lambda t: _make(scale, t).map(d_in)[1] < d_out[1]  # type: ignore[index,arg-type]
            threshold = binary_search(compare_t, T=int)  # type: ignore[assignment]

    else:
        scale = binary_search_param(_make, d_in, d_out, T=float)
        threshold = None

    return _make(scale, threshold), scale, threshold


def _deindex(indices, keys, cuts=None):
    """Translate keyset indices into categorical data.

    `indices` are bin indices into `cuts`: `0` selects the bin
    `(-inf, cuts[0]]`, `i` (for `0 < i < len(cuts)`) selects
    `(cuts[i - 1], cuts[i]]`, and `len(cuts)` selects `(cuts[-1], +inf]`.
    """
    import numpy as np

    if cuts is None:
        return keys[indices]

    cut_values = cuts.to_numpy()
    indices = np.asarray(indices)
    values = np.empty(indices.shape, dtype=cut_values.dtype)

    first = indices == 0
    last = indices == len(cut_values)
    interior = ~(first | last)

    # the first bin's representative is the cut itself: `value <= cuts[0]`
    values[first] = cut_values[0]

    bin_indices = indices[interior]
    lower = cut_values[bin_indices - 1]
    upper = cut_values[bin_indices]

    if cuts.dtype.is_integer():
        # the last cut can't be the dtype maximum (`_exterior_representable`
        # rejects that at construction time), so `+ 1` never overflows
        values[last] = cut_values[-1] + 1
        if lower.size:
            values[interior] = np.random.randint(
                lower + 1, upper + 1, size=lower.shape, dtype=cut_values.dtype
            )

    elif cuts.dtype.is_float():
        # `_exterior_representable` guarantees this is finite
        with np.errstate(over="ignore"):
            values[last] = np.nextafter(cut_values[-1], np.inf)

        if lower.size:
            # sample from (lower, upper], which is left-open; `uniform` is
            # lower-inclusive, so shift the sampled range to the next
            # representable float strictly above `lower`
            lower_open = np.nextafter(lower, upper)
            values[interior] = np.random.uniform(lower_open, upper)

    else:
        raise TypeError(f"cuts must be integer or float, got {cuts.dtype}")

    return values
