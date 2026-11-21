"""Stable sklearn input bridges: typed frames, privacy accounting, and fit."""

import numpy as np
import pytest

pytest.importorskip("sklearn")

import opendp.prelude as dp
from sklearn.base import BaseEstimator
from opendp.extras.sklearn._estimator import MeasurementMixin
from opendp.mod import OpenDPException

dp.enable_features("contrib")


class _TestEstimator(MeasurementMixin, BaseEstimator):
    def __init__(self, value=7, fail_ingest=False):
        self.value = value
        self.fail_ingest = fail_ingest

    def make_measurement(self, domain, metric, measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit_numpy(domain, metric)
        desc = numeric.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
        value = self.value + desc.X.descriptor.num_columns + int(desc.y is not None)
        measurement = dp.m.make_user_measurement(
            numeric.output_domain,
            numeric.output_metric,
            measure,
            lambda data: value,
            lambda d: d_out * d / numeric.map(d_in),
            TO=int,
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        if self.fail_ingest:
            raise ValueError("Could not construct fitted estimator")
        self.release_ = release


def _frame_domain(*, nullable=False, nan=False, with_y=True):
    cols = [
        dp.series_domain("a", dp.atom_domain(T=dp.f64, nan=nan)),
        dp.series_domain("b", dp.atom_domain(T=dp.f64, nan=False)),
    ]
    if with_y:
        cols.append(
            dp.series_domain(
                "target",
                dp.atom_domain(T=dp.i32, bounds=(0, 1)),
            )
        )
    if nullable:
        cols[0] = dp.series_domain(
            "a",
            dp.option_domain(dp.atom_domain(T=dp.f64, nan=False)),
        )
    return dp.dataframe_domain(cols)


def _frame():
    pl = pytest.importorskip("polars")
    return pl.DataFrame(
        {
            "a": pl.Series([1.0, 2.0, 3.0], dtype=pl.Float64),
            "b": pl.Series([3.0, 4.0, 5.0], dtype=pl.Float64),
            "target": pl.Series([0, 1, 0], dtype=pl.Int32),
        }
    )


def test_dataframe_to_sklearn_fit():
    pytest.importorskip("polars")
    domain = _frame_domain()
    packed = dp.sklearn.make_sklearn_fit(domain, dp.symmetric_distance(), y="target")
    aligned = packed(_frame())
    assert aligned.X.columns == ["a", "b"]
    assert aligned.y.name == "target"
    assert packed.map(2) == 2

    numeric = dp.sklearn.make_sklearn_fit_numpy(*packed.output_space)
    result = numeric(aligned)
    np.testing.assert_array_equal(result.X, [[1, 3], [2, 4], [3, 5]])
    np.testing.assert_array_equal(result.y, [0, 1, 0])
    assert result.X.dtype == np.float64
    assert result.y.dtype == np.int32
    assert numeric.output_domain.member(result)
    desc = numeric.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert desc.X.descriptor.num_columns == 2
    assert desc.y.element_domain == domain.get_series_domain("target").element_domain


def test_uniform_and_nonnull_preconditions():
    pytest.importorskip("polars")
    domain = _frame_domain(nullable=True)
    packed = dp.sklearn.make_sklearn_fit(domain, dp.symmetric_distance(), y="target")
    with pytest.raises(ValueError, match="nullable"):
        dp.sklearn.make_sklearn_fit_numpy(*packed.output_space)

    nonuniform = dp.dataframe_domain(
        [
            dp.series_domain("a", dp.atom_domain(T=dp.i32)),
            dp.series_domain("b", dp.atom_domain(T=dp.f64, nan=False)),
            dp.series_domain("target", dp.atom_domain(T=dp.i32)),
        ]
    )
    packed = dp.sklearn.make_sklearn_fit(
        nonuniform, dp.symmetric_distance(), y="target"
    )
    with pytest.raises(TypeError, match="uniform"):
        dp.sklearn.make_sklearn_fit_numpy(*packed.output_space)


def test_string_targets_convert_to_python_lists():
    pl = pytest.importorskip("polars")
    domain = dp.dataframe_domain(
        [
            dp.series_domain("x", dp.atom_domain(T=dp.f64)),
            dp.series_domain("group", dp.atom_domain(T=str)),
        ]
    )
    frame = pl.DataFrame({"x": [1.0, 2.0], "group": ["A", "B"]})
    packed = dp.sklearn.make_sklearn_fit(domain, dp.symmetric_distance(), y="group")
    numeric = dp.sklearn.make_sklearn_fit_numpy(*packed.output_space)
    result = numeric(packed(frame))
    assert result.y == ["A", "B"]
    assert numeric.output_domain.member(result)


def test_frame_distance_flattens_global_row_bound():
    pytest.importorskip("polars")
    packed = dp.sklearn.make_sklearn_fit(
        _frame_domain(),
        dp.frame_distance(dp.symmetric_distance()),
        y="target",
    )
    conversion = packed >> dp.sklearn.make_sklearn_fit_numpy(*packed.output_space)
    assert conversion.output_metric == dp.symmetric_distance()
    assert conversion.map([dp.polars.Bound(by=[], per_group=2, num_groups=1)]) == 2


def test_array2_packaging_preserves_feature_domain():
    data = np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
    domain = dp.numpy.array2_domain(
        T=dp.f64,
        num_columns=3,
        size=2,
        nan=False,
    )
    conversion = dp.sklearn.make_sklearn_fit(domain, dp.symmetric_distance())
    actual = conversion(data)
    assert actual.X is data
    assert actual.y is None
    assert conversion.map(2) == 2
    assert conversion.output_domain.member(actual)
    desc = conversion.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert desc.X is domain


def test_fit_query_from_explicit_numpy_targets():
    X = np.array([[1.0, 2.0], [3.0, 4.0]])
    y = np.array([0, 1], dtype=np.int32)
    data = dp.sklearn.SklearnFit(X, y)
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
        dp.vector_domain(dp.atom_domain(T=dp.i32)),
    )
    ctx = dp.Context.compositor(
        data,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    fitted = ctx.query().sklearn(_TestEstimator()).release()
    assert fitted.release_ == 10


def test_fit_query_from_eager_dataframe():
    pytest.importorskip("polars")
    ctx = dp.Context.compositor(
        _frame(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_frame_domain(),
        split_evenly_over=2,
    )
    original = _TestEstimator()
    fitted = ctx.query().sklearn(original, y="target").release()
    assert fitted is not original and fitted.release_ == 10
    assert not hasattr(original, "release_")
    assert ctx.current_privacy_loss() == [0.5]


def test_sklearn_retains_enclosing_release_wrapper():
    from opendp.context import Query

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False)
    )
    query = Query(
        chain=(domain, dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    query = query.new_with(
        chain=query._chain,
        wrap_release=lambda fitted: ("wrapped", fitted),
    ).sklearn(_TestEstimator())
    data = dp.sklearn.SklearnFit(np.ones((2, 2)))
    assert isinstance(query.resolve()(data), _TestEstimator)
    kind, fitted = query.release(data=data)
    assert kind == "wrapped"
    assert isinstance(fitted, _TestEstimator) and fitted.release_ == 9


def test_query_keeps_dataframe_features_until_the_estimator():
    pytest.importorskip("polars")

    class NumericEstimator(_TestEstimator):
        def make_measurement(self, domain, metric, measure, d_in, d_out):
            desc = domain.cast(dp.sklearn.SklearnFitDescriptor)
            assert isinstance(desc.X, dp.DataFrameDomain)
            assert isinstance(desc.y, dp.SeriesDomain)
            return super().make_measurement(domain, metric, measure, d_in, d_out)

    ctx = dp.Context.compositor(
        _frame(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_frame_domain(),
        split_evenly_over=1,
    )
    fitted = ctx.query().sklearn(NumericEstimator(), y="target").release()
    assert fitted.release_ == 10


def test_sklearn_requires_explicit_collect():
    lazy = _frame().lazy()
    domain = dp.lazyframe_domain(
        [_frame_domain().get_series_domain(name) for name in ("a", "b", "target")]
    )
    ctx = dp.Context.compositor(
        lazy,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    with pytest.raises(TypeError, match="collect"):
        ctx.query().sklearn(_TestEstimator(), y="target")
    query = ctx.query().select("a", "target").collect()
    assert isinstance(query._chain.output_domain, dp.DataFrameDomain)
    fitted = query.sklearn(_TestEstimator(), y="target").release()
    assert fitted.release_ == 9
    assert ctx.current_privacy_loss() == [1.0]


def test_ingest_is_after_accounted_release():
    data = dp.sklearn.SklearnFit(
        np.array([[1.0, 2.0], [4.0, 5.0]]),
        np.array([3.0, 6.0]),
    )
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
        dp.vector_domain(dp.atom_domain(T=dp.f64)),
    )
    ctx = dp.Context.compositor(
        data,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    model = _TestEstimator(fail_ingest=True)
    query = ctx.query().sklearn(model)
    assert ctx.current_privacy_loss() == []
    with pytest.raises(OpenDPException, match="construct fitted estimator"):
        query.release()
    assert not hasattr(model, "release_")


def test_dataframe_input_is_preserved_for_heterogeneous_estimators():
    pytest.importorskip("polars")
    domain = dp.dataframe_domain(
        [
            dp.series_domain("age", dp.atom_domain(T=dp.i32)),
            dp.series_domain("region", dp.atom_domain(T=str)),
            dp.series_domain("target", dp.atom_domain(T=dp.i32)),
        ]
    )
    conv = dp.sklearn.make_sklearn_fit(
        domain,
        dp.symmetric_distance(),
        y="target",
    )
    desc = conv.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(desc.X, dp.DataFrameDomain)
    assert desc.X.columns == ["age", "region"]
    pl = __import__("polars")
    frame = pl.DataFrame(
        {
            "age": pl.Series([1, 2], dtype=pl.Int32),
            "region": pl.Series(["East", "West"], dtype=pl.String),
            "target": pl.Series([0, 1], dtype=pl.Int32),
        }
    )
    data = conv(frame)
    assert data.X.columns == ["age", "region"]
    assert data.y.name == "target"
    assert data.y.dtype == pl.Int32
    assert conv.output_domain.cast(dp.sklearn.SklearnFitDescriptor).y == (
        domain.get_series_domain("target")
    )
    assert data.y.to_list() == [0, 1]


def test_standalone_fit_accepts_polars_series_target():
    pl = pytest.importorskip("polars")
    X = np.array([[1.0, 2.0], [3.0, 4.0]])
    x_domain = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    for T, values, dtype in (
        (dp.i32, [0, 1], pl.Int32),
        (str, ["a", "b"], pl.String),
    ):
        model = _TestEstimator()
        fitted = model.fit(
            X,
            pl.Series("label", values, dtype=dtype),
            domain=dp.sklearn.sklearn_fit_domain(
                x_domain,
                dp.series_domain("label", dp.atom_domain(T=T)),
            ),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
        )
        assert fitted is model
        assert fitted.release_ == 10


def test_private_polars_prediction_preserves_schema_and_targets():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model import LogisticRegression

    model = LogisticRegression()
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    frame = _frame()
    ctx = dp.Context.compositor(
        frame,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_frame_domain(),
        split_evenly_over=2,
    )

    for method, columns in (
        ("predict", ["prediction"]),
        ("predict_proba", ["proba_0", "proba_1"]),
    ):
        trans = (
            ctx.query()
            .sklearn(model, y="target", method=method)
            .resolve(allow_transformations=True)
        )
        with pytest.warns(UserWarning, match="X has feature names"):
            result = trans(frame)
        assert isinstance(result.X, pl.DataFrame)
        assert result.X.columns == columns
        assert result.X.height == frame.height
        assert result.y.equals(frame["target"])
        assert isinstance(
            trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor).X,
            dp.DataFrameDomain,
        )
        assert trans.map(1) == 1
        if method == "predict":
            assert result.X["prediction"].to_list() == [0, 0, 1]
        else:
            np.testing.assert_allclose(result.X.to_numpy().sum(axis=1), 1)

    assert ctx.current_privacy_loss() == []


def test_sklearn_input_bridge_rejects_public_schema_errors():
    from opendp.extras.sklearn._make_sklearn_fit import (
        make_sklearn_fit,
        make_sklearn_fit_numpy,
    )

    pl = pytest.importorskip("polars")
    assert isinstance(
        dp.domain_of(pl.Series("column", [1.0]), infer=True), dp.SeriesDomain
    )
    metric = dp.symmetric_distance()
    with pytest.raises(TypeError, match="collect"):
        make_sklearn_fit(
            dp.lazyframe_domain([dp.series_domain("x", dp.atom_domain(T=dp.f64))]),
            metric,
        )
    with pytest.raises(ValueError):
        make_sklearn_fit(dp.atom_domain(T=dp.f64), metric)
    with pytest.raises(ValueError, match="existing DataFrame column"):
        make_sklearn_fit(_frame_domain(), metric, y="absent")
    with pytest.raises(ValueError, match="feature column"):
        make_sklearn_fit(
            dp.dataframe_domain([dp.series_domain("y", dp.atom_domain(T=dp.f64))]),
            metric,
            y="y",
        )
    nullable_y = dp.dataframe_domain(
        [
            dp.series_domain("x", dp.atom_domain(T=dp.f64)),
            dp.series_domain("y", dp.option_domain(dp.atom_domain(T=dp.f64))),
        ]
    )
    packed = make_sklearn_fit(nullable_y, metric, y="y")
    with pytest.raises(ValueError, match="non-nullable"):
        make_sklearn_fit_numpy(*packed.output_space)

    X = _frame_domain(with_y=False)
    for target, match in [
        (
            dp.series_domain("y", dp.option_domain(dp.atom_domain(T=dp.f64))),
            "non-nullable",
        ),
        (dp.series_domain("y", dp.categorical_domain()), "numeric, boolean, or string"),
    ]:
        aligned = dp.sklearn.sklearn_fit_domain(X, target)
        with pytest.raises((ValueError, TypeError), match=match):
            make_sklearn_fit_numpy(aligned, metric)


def test_sklearn_numpy_target_schema_errors():
    from opendp.extras.sklearn._make_sklearn_fit import make_sklearn_fit

    metric = dp.symmetric_distance()
    domain = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    for y in (0, -1, "label"):
        with pytest.raises(TypeError, match="SklearnFit"):
            make_sklearn_fit(domain, metric, y=y)

    with pytest.raises(ValueError):
        make_sklearn_fit(dp.sklearn.sklearn_fit_domain(domain), metric)


def test_sklearn_query_rejects_duplicate_targets_and_lazy_features():
    from opendp.context import Query

    metric = dp.symmetric_distance()
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    )
    query = Query((domain, metric), dp.max_divergence(), d_in=1, d_out=1.0)
    with pytest.raises(ValueError, match="already present in SklearnFit"):
        query.sklearn(_TestEstimator(), y=0)

    lazy = dp.sklearn.sklearn_fit_domain(
        dp.lazyframe_domain([dp.series_domain("x", dp.atom_domain(T=dp.f64))])
    )
    query = Query((lazy, metric), dp.max_divergence(), d_in=1, d_out=1.0)
    with pytest.raises(TypeError, match="collect"):
        query.sklearn(_TestEstimator())
