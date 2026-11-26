"""Stable sklearn-to-Polars query handoff and privacy accounting."""

import numpy as np
import pytest
import opendp.prelude as dp

from opendp._internal import _make_transformation

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


def _space(n=3):
    x_domain = dp.numpy.array2_domain(
        T=dp.f32,
        num_columns=2,
        nan=False,
    )
    y_domain = dp.vector_domain(dp.atom_domain(T=dp.i32, bounds=(0, 1)))
    domain = dp.sklearn.sklearn_fit_domain(x_domain, y_domain)
    data = dp.sklearn.SklearnFit(
        np.array([[0.2, 1.0], [0.2, 1.0], [0.8, 2.0]][:n], dtype=np.float32),
        np.array([0, 1, 0][:n], dtype=np.int32),
    )
    return domain, data


def _context(domain, data, *, d_in=1):
    return dp.Context.compositor(
        data,
        dp.unit_of(contributions=d_in),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )


def test_conversion_preserves_rows_types_and_targets():
    domain, data = _space()
    transform = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain,
        dp.symmetric_distance(),
        target_name="y",
    )
    assert transform.output_domain.columns == ["x0", "x1", "y"]
    assert transform.map(2) == 2

    result = transform(data)
    assert isinstance(result, pl.LazyFrame)
    assert result.collect_schema() == {
        "x0": pl.Float32,
        "x1": pl.Float32,
        "y": pl.Int32,
    }
    expected = result.collect()
    assert expected.get_column("y").to_list() == [0, 1, 0]
    assert (
        expected.get_column("x0").to_list()[0] == expected.get_column("x0").to_list()[1]
    )


def test_custom_names_and_empty_input():
    domain, _ = _space()
    transform = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain,
        dp.symmetric_distance(),
        feature_names=("first", "second"),
        target_name="label",
    )
    data = dp.sklearn.SklearnFit(
        np.empty((0, 2), dtype=np.float32),
        np.empty((0,), dtype=np.int32),
    )
    out = transform(data).collect()
    assert out.shape == (0, 3)
    assert out.columns == ["first", "second", "label"]
    assert out.schema == {
        "first": pl.Float32,
        "second": pl.Float32,
        "label": pl.Int32,
    }


def test_no_target_and_missing_schema_are_explicit():
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
    )
    conversion = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain, dp.symmetric_distance()
    )
    assert conversion.output_domain.columns == ["x0", "x1"]
    result = conversion(
        dp.sklearn.SklearnFit(np.ones((2, 2), dtype=np.float64))
    ).collect()
    assert result.shape == (2, 2)
    assert result.dtypes == [pl.Float64, pl.Float64]

    unknown = dp.sklearn.sklearn_fit_domain(dp.numpy.array2_domain(T=dp.f64))
    with pytest.raises(ValueError, match="public width"):
        dp.sklearn.make_sklearn_fit_to_lazyframe(unknown, dp.symmetric_distance())


@pytest.mark.parametrize(
    "names, target",
    [
        (["a"], "y"),
        (["a", "a"], "y"),
        (["a", "y"], "y"),
    ],
)
def test_reject_bad_column_names(names, target):
    domain, _ = _space()
    with pytest.raises(ValueError):
        dp.sklearn.make_sklearn_fit_to_lazyframe(
            domain,
            dp.symmetric_distance(),
            feature_names=names,
            target_name=target,
        )


def test_symbolic_handoff_only_consumes_privacy_on_aggregation():
    domain, data = _space()
    ctx = _context(domain, data)
    query = ctx.query().lazy(target_name="y")

    assert ctx.current_privacy_loss() == []
    assert query.collect_schema() == {
        "x0": pl.Float32,
        "x1": pl.Float32,
        "y": pl.Int32,
    }

    collected = query.collect()
    assert isinstance(collected, dp.Query)
    assert isinstance(collected._chain, dp.Transformation)
    assert isinstance(collected._chain.output_domain, dp.DataFrameDomain)
    assert ctx.current_privacy_loss() == []

    # The existing Polars DP API handles the resulting stable LazyFrame.
    result = query.select(dp.len(signed=True)).release().collect()
    assert result.height == 1 and "len" in result.columns
    assert ctx.current_privacy_loss() == [1.0]


@pytest.mark.parametrize(
    "names",
    [{"feature_names": ["renamed"]}, {"target_name": "target"}],
)
def test_eager_dataframe_lazy_rejects_column_names(names):
    frame = pl.DataFrame({"feature": pl.Series([0.2, 0.8], dtype=pl.Float64)})
    domain = dp.dataframe_domain(
        [dp.series_domain("feature", dp.atom_domain(T=dp.f64))]
    )
    ctx = _context(domain, frame)
    assert ctx.query().lazy().collect_schema() == {"feature": pl.Float64}
    with pytest.raises(
        ValueError, match="DataFrame schema already determines column names"
    ):
        ctx.query().lazy(**names)
    assert ctx.current_privacy_loss() == []


def test_previous_stability_map_is_used_for_calibration():
    domain, data = _space()
    ctx = _context(domain, data)
    stable = _make_transformation(
        domain,
        dp.symmetric_distance(),
        domain,
        dp.symmetric_distance(),
        lambda d: d,
        lambda d: d * 2,
    )
    query = ctx.query().new_with(chain=stable).lazy()
    assert query._query._chain.map(1) == 2
    measure = query.select(dp.len(signed=True)).resolve()
    assert measure.input_domain == domain
    assert measure.input_metric == dp.symmetric_distance()
    assert measure.check(1, 1.0)
    assert ctx.current_privacy_loss() == []
    answer = query.select(dp.len(signed=True)).release().collect()
    assert answer.shape == (1, 1)
    assert ctx.current_privacy_loss() == [1.0]


def test_typed_lazyframe_prefix_passes_through_without_reconversion():
    domain, data = _space()
    ctx = _context(domain, data)
    output_domain = dp.lazyframe_domain(
        [
            dp.series_domain("cluster__prediction", dp.atom_domain(T=dp.i32)),
            dp.series_domain("distances__distance", dp.atom_domain(T=dp.f64)),
            dp.series_domain("y", dp.atom_domain(T=dp.i32, bounds=(0, 1))),
        ]
    )
    executed = []

    def stable_inference(value):
        executed.append(True)
        return pl.DataFrame(
            {
                "cluster__prediction": pl.Series(
                    [int(v >= 0.5) for v in value.X[:, 0]], dtype=pl.Int32
                ),
                "distances__distance": pl.Series(
                    np.abs(value.X[:, 0].astype(np.float64) - 0.5),
                    dtype=pl.Float64,
                ),
                "y": pl.Series(value.y, dtype=pl.Int32),
            }
        ).lazy()

    transform = _make_transformation(
        domain,
        dp.symmetric_distance(),
        output_domain,
        dp.symmetric_distance(),
        stable_inference,
        lambda d: d,
    )
    query = ctx.query().new_with(chain=transform).lazy()
    assert executed == []
    assert query.collect_schema() == {
        "cluster__prediction": pl.Int32,
        "distances__distance": pl.Float64,
        "y": pl.Int32,
    }
    with pytest.raises(ValueError, match="already determines column names"):
        ctx.query().new_with(chain=transform).lazy(feature_names=["anything"])

    # Filtering on both typed columns exercises the lazy handoff without
    # requiring an unsupported public max_length bound for private sums.
    dp_query = query.filter(
        (pl.col("cluster__prediction") == 1) & (pl.col("distances__distance") > 0.0)
    ).select(dp.len(signed=True))
    assert executed == []
    answer = dp_query.release().collect()
    assert executed == [True]
    assert answer.shape == (1, 1)


def test_sklearn_groupby_with_public_keys():
    domain, data = _space()
    ctx = _context(domain, data)
    keys = pl.LazyFrame([pl.Series("y", [0, 1], dtype=pl.Int32)])
    grouped = (
        ctx.query()
        .lazy(target_name="y")
        .group_by("y")
        .agg(dp.len(signed=True))
        .with_keys(keys)
    )
    released = grouped.release().collect().sort("y")
    assert released.get_column("y").to_list() == [0, 1]
    assert released.get_column("len").dtype in (pl.Int32, pl.Int64)
    assert ctx.current_privacy_loss() == [1.0]


def test_measured_query_cannot_be_made_lazy():
    domain, data = _space()
    ctx = _context(domain, data)
    query = ctx.query().lazy().select(dp.len(signed=True))
    with pytest.raises((ValueError, AttributeError)):
        query._query.new_with(chain=query.resolve()).lazy()


def test_default_lazy_hides_target_unless_explicitly_included():
    domain, data = _space()
    ctx = _context(domain, data)
    plain = ctx.query().lazy()
    assert plain.collect_schema() == {
        "x0": pl.Float32,
        "x1": pl.Float32,
    }
    explicit = ctx.query().lazy(target_name="outcome")
    assert explicit.collect_schema() == {
        "x0": pl.Float32,
        "x1": pl.Float32,
        "outcome": pl.Int32,
    }
    assert ctx.current_privacy_loss() == []


def test_dataframe_features_and_series_target_can_enter_polars_lazily():
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain(
            [
                dp.series_domain("feature", dp.atom_domain(T=dp.f64, nan=False)),
                dp.series_domain("text", dp.atom_domain(T=str)),
            ]
        ),
        dp.series_domain("label", dp.atom_domain(T=dp.i32)),
    )
    y = pl.Series("label", [0, 1], dtype=pl.Int32)
    data = dp.sklearn.SklearnFit(
        pl.DataFrame(
            {
                "feature": pl.Series([0.25, 0.75], dtype=pl.Float64),
                "text": pl.Series(["a", "b"], dtype=pl.String),
            }
        ),
        y,
    )
    ctx = _context(domain, data)
    plain = ctx.query().lazy()
    assert plain.collect_schema() == {"feature": pl.Float64, "text": pl.String}
    with pytest.raises(ValueError, match="already determined"):
        ctx.query().lazy(feature_names=("other", "text"))
    included = ctx.query().lazy(target_name="outcome")
    assert included.collect_schema() == {
        "feature": pl.Float64,
        "text": pl.String,
        "outcome": pl.Int32,
    }
    assert ctx.current_privacy_loss() == []


def test_sklearn_to_polars_rejects_unsupported_public_configuration():
    domain, _ = _space()
    make = dp.sklearn.make_sklearn_fit_to_lazyframe
    metric = dp.symmetric_distance()

    for unsupported_metric in (
        dp.discrete_distance(),
        dp.insert_delete_distance(),
        dp.hamming_distance(),
    ):
        with pytest.raises(ValueError, match="row-adjacency"):
            make(domain, unsupported_metric)
    with pytest.raises(ValueError, match="nonempty string"):
        make(domain, metric, target_name="")
    without_target = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    )
    with pytest.raises(ValueError, match="No target"):
        make(without_target, metric, target_name="y")
    unsupported = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
        dp.atom_domain(T=dp.i32),
    )
    with pytest.raises(TypeError, match="SeriesDomain or VectorDomain"):
        make(unsupported, metric, target_name="y")


def test_sklearn_to_polars_optional_and_nonatomic_targets():
    from opendp.extras.sklearn._to_polars import _series_dtype

    with pytest.raises(TypeError, match="atomic element"):
        _series_dtype(dp.vector_domain(dp.atom_domain(T=dp.i32)))

    assert _series_dtype(dp.option_domain(dp.atom_domain(T=dp.i32))) == pl.Int32


def test_public_sklearn_to_polars_without_hbc():
    domain, data = _space()
    dp.disable_features("honest-but-curious")
    try:
        conversion = dp.sklearn.make_sklearn_fit_to_lazyframe(
            domain, dp.symmetric_distance(), target_name="target"
        )
        result = conversion(data).collect()
        assert result["target"].to_list() == data.y.tolist()
        assert conversion.map(1) == 1
        ctx = _context(domain, data)
        answer = ctx.query().lazy().select(dp.len(signed=True)).release().collect()
        assert answer.shape == (1, 1)
        assert ctx.current_privacy_loss() == [1.0]
    finally:
        dp.enable_features("honest-but-curious")


@pytest.mark.parametrize("nullable", [False, True])
@pytest.mark.parametrize("bounded", [False, True])
def test_dataframe_conversion_preserves_target_nullability_and_input(nullable, bounded):
    frame = pl.DataFrame({"feature": pl.Series([1, 2], dtype=pl.Int32)})
    target = pl.Series("original", [0, None if nullable else 1], dtype=pl.Int32)
    element = dp.atom_domain(T=dp.i32, bounds=(0, 1))
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain([dp.series_domain("feature", dp.atom_domain(T=dp.i32))]),
        dp.series_domain(
            "original", dp.option_domain(element) if nullable else element
        ),
    )
    metric = dp.change_one_distance() if bounded else dp.symmetric_distance()
    conversion = dp.sklearn.make_sklearn_fit_to_lazyframe(
        domain, metric, target_name="renamed"
    )
    data = dp.sklearn.SklearnFit(frame, target)
    result = conversion(data).collect()
    assert result["renamed"].to_list() == target.to_list()
    assert result["feature"].to_list() == [1, 2]
    assert conversion.output_domain.get_series_domain("renamed").nullable == nullable
    assert (
        conversion.output_domain.get_series_domain("renamed").element_domain == element
    )
    assert conversion.output_metric == metric
    assert conversion.map(1) == 1
    assert frame.columns == ["feature"]
    assert target.name == "original"


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_bounded_numpy_conversion_preserves_aligned_rows_and_metric(route):
    X = np.array([[0.25, 0.5], [0.75, 1.5]], dtype=np.float64)
    y = np.array([0, 1], dtype=np.int32)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False),
        dp.vector_domain(dp.atom_domain(T=dp.i32, bounds=(0, 1))),
    )
    data = dp.sklearn.SklearnFit(X, y)
    assert domain.member(data)
    metric = dp.change_one_distance()
    names = dict(feature_names=("left", "right"), target_name="label")
    if route == "standalone":
        conversion = dp.sklearn.make_sklearn_fit_to_lazyframe(domain, metric, **names)
    else:
        context = dp.Context.compositor(
            data,
            dp.unit_of(changes=1),
            dp.loss_of(epsilon=1.0),
            domain=domain,
            split_evenly_over=1,
        )
        query = context.query().lazy(**names)
        conversion = query._query.resolve(allow_transformations=True)
        assert context.current_privacy_loss() == []

    assert conversion.input_metric == conversion.output_metric == metric
    assert conversion.map(1) == 1
    assert conversion.map(2) == 2
    result = conversion(data).collect()
    assert result.schema == {"left": pl.Float64, "right": pl.Float64, "label": pl.Int32}
    np.testing.assert_array_equal(result.select("left", "right").to_numpy(), X)
    np.testing.assert_array_equal(result["label"].to_numpy(), y)

    # Replacing one aligned input row replaces just that output row.
    neighbor = dp.sklearn.SklearnFit(X.copy(), y.copy())
    neighbor.X[0] = [2.0, 3.0]
    assert neighbor.y is not None
    neighbor.y[0] = 1
    assert domain.member(neighbor)
    changed = conversion(neighbor).collect()
    assert changed.row(0) == (2.0, 3.0, 1)
    assert changed.row(1) == result.row(1)
