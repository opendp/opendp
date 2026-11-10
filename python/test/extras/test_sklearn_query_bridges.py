"""Stable sklearn input bridges: typed frames, privacy accounting, and fit."""

import numpy as np
import pytest

import opendp.prelude as dp
from opendp._internal import _make_transformation
from opendp.extras.sklearn._estimator import DPTransformer

pytest.importorskip("sklearn")
dp.enable_features("contrib")


class _TestEstimator(DPTransformer):
    def __init__(self, value=7, fail_ingest=False):
        self.value = value
        self.fail_ingest = fail_ingest

    def make(self, domain, metric, measure, d_in, d_out):
        numeric = dp.sklearn.make_sklearn_fit(domain, metric, as_numpy=True)
        desc = numeric.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
        value = self.value + desc.X.descriptor.num_columns + int(desc.y is not None)
        measurement = dp.m.make_user_measurement(
            *numeric.output_space, measure,
            lambda data: value,
            lambda d: d_out * d / numeric.map(d_in),
            TO=int,
        )
        return numeric >> measurement

    def _ingest_release(self, release):
        if self.fail_ingest:
            raise ValueError("Could not construct fitted estimator")
        self.release_ = release

    def make_transform(self, input_domain, input_metric):
        self.release_
        return _make_transformation(
            input_domain, input_metric, input_domain, input_metric,
            lambda data: data, lambda d: d,
        )


def _frame_domain(*, nullable=False, nan=False, with_y=True):
    cols = [
        dp.series_domain("a", dp.atom_domain(T=dp.f64, nan=nan)),
        dp.series_domain("b", dp.atom_domain(T=dp.f64, nan=False)),
    ]
    if with_y:
        cols.append(dp.series_domain(
            "target", dp.atom_domain(T=dp.i32, bounds=(0, 1)),
        ))
    if nullable:
        cols[0] = dp.series_domain(
            "a", dp.option_domain(dp.atom_domain(T=dp.f64, nan=False)),
        )
    return dp.dataframe_domain(cols)


def _frame():
    pl = pytest.importorskip("polars")
    return pl.DataFrame({
        "a": pl.Series([1., 2., 3.], dtype=pl.Float64),
        "b": pl.Series([3., 4., 5.], dtype=pl.Float64),
        "target": pl.Series([0, 1, 0], dtype=pl.Int32),
    })






def test_dataframe_to_sklearn_fit():
    pytest.importorskip("polars")
    domain = _frame_domain()
    convert = dp.sklearn.make_dataframe_to_sklearn_fit(
        domain, dp.symmetric_distance(), y="target",
    )
    result = convert(_frame())
    np.testing.assert_array_equal(result.X, [[1, 3], [2, 4], [3, 5]])
    np.testing.assert_array_equal(result.y, [0, 1, 0])
    assert result.X.dtype == np.float64
    assert result.y.dtype == np.int32
    assert convert.map(2) == 2
    assert convert.output_domain.member(result)
    descriptor = convert.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert descriptor.X.descriptor.nan is False
    assert descriptor.X.descriptor.num_columns == 2

    # The same input can remain in Polars until a numerical estimator needs it.
    as_polars = dp.sklearn.make_dataframe_to_sklearn_fit(
        domain, dp.symmetric_distance(), y="target", as_numpy=False,
    )
    aligned = as_polars(_frame())
    assert aligned.X.columns == ["a", "b"]
    assert aligned.y.name == "target"
    assert as_polars.output_domain.member(aligned)
    to_numpy = dp.sklearn.make_sklearn_fit(*as_polars.output_space, as_numpy=True)
    numeric = to_numpy(aligned)
    np.testing.assert_array_equal(numeric.X, result.X)
    np.testing.assert_array_equal(numeric.y, result.y)
    assert to_numpy.output_domain.member(numeric)




def test_uniform_and_nonnull_preconditions():
    pytest.importorskip("polars")
    domain = _frame_domain(nullable=True)
    with pytest.raises(ValueError, match="nullable"):
        dp.sklearn.make_dataframe_to_sklearn_fit(
            domain, dp.symmetric_distance(), y="target",
        )

    nonuniform = dp.dataframe_domain([
        dp.series_domain("a", dp.atom_domain(T=dp.i32)),
        dp.series_domain("b", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("target", dp.atom_domain(T=dp.i32)),
    ])
    with pytest.raises(TypeError, match="uniform"):
        dp.sklearn.make_dataframe_to_sklearn_fit(
            nonuniform, dp.symmetric_distance(), y="target",
        )

    nullable_target = dp.dataframe_domain([
        dp.series_domain("a", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("target", dp.option_domain(dp.atom_domain(T=dp.i32))),
    ])
    with pytest.raises(ValueError, match="nullable"):
        dp.sklearn.make_dataframe_to_sklearn_fit(
            nullable_target, dp.symmetric_distance(), y="target",
        )


def test_nan_policy_and_string_targets():
    pl = pytest.importorskip("polars")
    nan_domain = _frame_domain(nan=True)
    desc = dp.sklearn.make_sklearn_fit(
        nan_domain, dp.symmetric_distance(), y="target"
    ).output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert desc.X.descriptor.nan is True

    text_domain = dp.dataframe_domain([
        dp.series_domain("x", dp.atom_domain(T=dp.f32, nan=False)),
        dp.series_domain("group", dp.atom_domain(T=str)),
    ])
    frame = pl.DataFrame({
        "x": pl.Series([1., 2.], dtype=pl.Float32),
        "group": pl.Series(["A", "B"], dtype=pl.String),
    })
    convert = dp.sklearn.make_dataframe_to_sklearn_fit(
        text_domain, dp.symmetric_distance(), y="group",
    )
    out = convert(frame)
    assert out.y == ["A", "B"]
    assert convert.output_domain.member(out)


def test_frame_distance_flattens_global_row_bound():
    pytest.importorskip("polars")
    domain = _frame_domain()
    conversion = dp.sklearn.make_dataframe_to_sklearn_fit(
        domain, dp.frame_distance(dp.symmetric_distance()), y="target",
    )
    bounds = [dp.polars.Bound(by=[], per_group=2, num_groups=1)]
    assert conversion.map(bounds) == 2
    assert conversion.output_metric == dp.symmetric_distance()
    with pytest.raises(ValueError, match="global"):
        conversion.map([dp.polars.Bound(by=["a"], per_group=2)])


def test_array2_target_selection_and_non_nan():
    data = np.array([[1., 2., 3.], [4., 5., 6.]])
    domain = dp.numpy.array2_domain(
        T=float, num_columns=3, size=2, nan=False,
    )
    conversion = dp.sklearn.make_sklearn_fit(
        domain, dp.symmetric_distance(), y=-1,
    )
    actual = conversion(data)
    np.testing.assert_array_equal(actual.X, data[:, :2])
    np.testing.assert_array_equal(actual.y, data[:, -1])
    assert conversion.output_domain.member(actual)
    desc = conversion.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert desc.X.descriptor.nan is False
    assert desc.y.element_domain.nan is False
    with pytest.raises(TypeError, match="integer"):
        dp.sklearn.make_sklearn_fit(domain, dp.symmetric_distance(), y="label")


def test_fit_and_transform_query_from_eager_dataframe():
    pytest.importorskip("polars")
    ctx = dp.Context.compositor(
        _frame(), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.), domain=_frame_domain(),
        split_evenly_over=2,
    )
    original = _TestEstimator()
    fitted = ctx.query().sklearn(original, y="target").release()
    assert fitted is not original and fitted.release_ == 10
    assert not hasattr(original, "release_")
    assert ctx.current_privacy_loss() == [0.5]
    transform_query = ctx.query().sklearn(fitted.as_transformer(), y="target")
    assert transform_query.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == [0.5]


def test_sklearn_resolve_returns_fitted_estimator():
    from opendp.context import Query

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False)
    )
    original = _TestEstimator()
    query = Query(
        chain=(domain, dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    ).sklearn(original)
    measurement = query.resolve()
    assert measurement.check(1, 1.0)
    data = dp.sklearn.SklearnFit(np.ones((2, 2)))
    first = measurement(data)
    second = measurement(data)
    assert isinstance(first, _TestEstimator)
    assert first.release_ == second.release_ == 9
    assert first is not second and first is not original
    assert not hasattr(original, "release_")


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
        def make(self, domain, metric, measure, d_in, d_out):
            desc = domain.cast(dp.sklearn.SklearnFitDescriptor)
            assert isinstance(desc.X, dp.DataFrameDomain)
            assert isinstance(desc.y, dp.SeriesDomain)
            return super().make(domain, metric, measure, d_in, d_out)

    ctx = dp.Context.compositor(
        _frame(), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0), domain=_frame_domain(),
        split_evenly_over=1,
    )
    fitted = ctx.query().sklearn(NumericEstimator(), y="target").release()
    assert fitted.release_ == 10


def test_fit_from_lazyframe_with_stable_select():
    pytest.importorskip("polars")
    lazy = _frame().lazy()
    domain = dp.lazyframe_domain([
        _frame_domain().get_series_domain(name)
        for name in ["a", "b", "target"]
    ])
    ctx = dp.Context.compositor(
        lazy, dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.), domain=domain,
        split_evenly_over=1,
    )
    fitted = (
        ctx.query().select("a", "target")
        .collect()
        .sklearn(_TestEstimator(), y="target")
        .release()
    )
    assert fitted.release_ == 9
    assert ctx.current_privacy_loss() == [1.0]


def test_ingest_is_after_accounted_release():
    data = np.array([[1., 2., 3.], [4., 5., 6.]])
    ctx = dp.Context.compositor(
        data, dp.unit_of(contributions=1), dp.loss_of(epsilon=1.),
        split_evenly_over=1,
    )
    model = _TestEstimator(fail_ingest=True)
    query = ctx.query().sklearn(model, y=2)
    assert ctx.current_privacy_loss() == []
    with pytest.raises(ValueError, match="construct fitted estimator"):
        query.release()
    assert ctx.current_privacy_loss() == [1.0]
    assert not hasattr(model, "release_")



def test_sklearn_requires_explicit_collect():
    pytest.importorskip("polars")
    domain = dp.lazyframe_domain([
        _frame_domain().get_series_domain(name) for name in ("a", "b", "target")
    ])
    ctx = dp.Context.compositor(
        _frame().lazy(), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.), domain=domain, split_evenly_over=1,
    )
    with pytest.raises(TypeError, match="collect"):
        ctx.query().sklearn(_TestEstimator(), y="target")
    query = ctx.query().select("a", "target").collect()
    assert isinstance(query._chain.output_domain, dp.DataFrameDomain)
    fitted = query.sklearn(_TestEstimator(), y="target").release()
    assert fitted.release_ == 9


def test_dataframe_input_is_preserved_for_heterogeneous_estimators():
    pytest.importorskip("polars")
    domain = dp.dataframe_domain([
        dp.series_domain("age", dp.atom_domain(T=dp.i32)),
        dp.series_domain("region", dp.atom_domain(T=str)),
        dp.series_domain("target", dp.atom_domain(T=dp.i32)),
    ])
    conv = dp.sklearn.make_sklearn_fit(
        domain, dp.symmetric_distance(), y="target", as_numpy=False,
    )
    desc = conv.output_domain.cast(dp.sklearn.SklearnFitDescriptor)
    assert isinstance(desc.X, dp.DataFrameDomain)
    assert desc.X.columns == ["age", "region"]
    pl = __import__("polars")
    frame = pl.DataFrame({
        "age": pl.Series([1, 2], dtype=pl.Int32),
        "region": pl.Series(["East", "West"], dtype=pl.String),
        "target": pl.Series([0, 1], dtype=pl.Int32),
    })
    data = conv(frame)
    assert data.X.columns == ["age", "region"]
    assert data.y.name == "target"
    assert data.y.dtype == pl.Int32
    assert conv.output_domain.member(data)
    assert conv.output_domain.cast(dp.sklearn.SklearnFitDescriptor).y == (
        domain.get_series_domain("target")
    )
    assert data.y.to_list() == [0, 1]


def test_frozen_adapter_can_run_stable_transform_without_budget():
    ctx = dp.Context.compositor(
        _frame(), dp.unit_of(contributions=1), dp.loss_of(epsilon=1.),
        domain=_frame_domain(), split_evenly_over=1,
    )
    fitted = ctx.query().sklearn(_TestEstimator(), y="target").release()
    frozen = fitted.as_transformer("transform")
    assert frozen._has_private_fit() is False
    before = ctx.current_privacy_loss()
    transform = ctx.query().sklearn(frozen, y="target")
    assert transform.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == before



def test_inferred_nullable_frame_requires_explicit_numeric_schema():
    pl = pytest.importorskip("polars")
    frame = pl.DataFrame({"a": [1., 2.]})
    domain = dp.domain_of(frame, infer=True)
    assert domain.get_series_domain("a").nullable

    from opendp.extras.numpy._make_dataframe_to_numpy import make_dataframe_to_numpy
    with pytest.raises(ValueError, match="non-nullable"):
        make_dataframe_to_numpy(domain, dp.symmetric_distance())


def test_logistic_frozen_prediction_and_probabilities():
    pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression()
    model._ingest_release(np.array([2., -1., 0.], dtype=np.float64))
    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False)
    )
    data = dp.sklearn.SklearnFit(np.array([
        [1., 0.], [0., 2.]
    ], dtype=np.float64))
    predictions = model.as_transformer("predict").make_transform(
        domain, dp.symmetric_distance()
    )
    assert predictions.map(1) == 1
    predicted = predictions(data)
    frame = predicted.X
    assert frame.columns == ["prediction"]
    assert frame.get_column("prediction").to_list() == [1, 0]
    assert predicted.y is None
    assert predictions.output_domain.member(predicted)

    probabilities = model.as_transformer("predict_proba").make_transform(
        domain, dp.symmetric_distance()
    )
    estimated = probabilities(data)
    proba = estimated.X
    assert proba.columns == ["proba_0", "proba_1"]
    assert estimated.y is None
    assert probabilities.output_domain.member(estimated)
    np.testing.assert_allclose(
        proba.select(["proba_0", "proba_1"]).to_numpy().sum(axis=1),
        [1., 1.],
    )


def test_frozen_logistic_converts_dataframe_features_itself():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model._logistic_regression import LogisticRegression

    model = LogisticRegression()
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    domain = dp.sklearn.sklearn_fit_domain(dp.dataframe_domain([
        dp.series_domain("a", dp.atom_domain(T=dp.f64, nan=False)),
        dp.series_domain("b", dp.atom_domain(T=dp.f64, nan=False)),
    ]))
    trans = model.as_transformer("predict").make_transform(
        domain, dp.symmetric_distance(),
    )
    output = trans(dp.sklearn.SklearnFit(pl.DataFrame({
        "a": pl.Series([1.0, 0.0], dtype=pl.Float64),
        "b": pl.Series([0.0, 2.0], dtype=pl.Float64),
    })))
    assert output.X.columns == ["prediction"]
    assert output.X["prediction"].to_list() == [1, 0]
    assert trans.output_domain.member(output)


def test_rowwise_spec_accepts_vector_domain():
    from opendp.extras.sklearn._frozen import FrozenTransformer, _RowwiseSpec

    class VectorPrediction(_TestEstimator):
        def _make_rowwise_method(self, method, input_domain):
            assert method == "predict"
            return _RowwiseSpec(
                lambda X: X[:, 0].tolist(),
                dp.vector_domain(dp.atom_domain(T=dp.f64, nan=True)),
            )

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=True),
    )
    transform = FrozenTransformer(VectorPrediction(), method="predict").make_transform(
        domain, dp.symmetric_distance()
    )
    output = transform(dp.sklearn.SklearnFit(
        np.array([[1.0, 2.0], [3.0, 4.0]], dtype=np.float64),
    ))
    assert output == [1.0, 3.0]
    assert transform.output_domain.member(output)
    assert transform.map(2) == 2


def test_standalone_fit_leaves_representation_to_estimator():
    pl = pytest.importorskip("polars")

    class NumericEstimator(_TestEstimator):
        def make(self, domain, metric, measure, d_in, d_out):
            x = domain.cast(dp.sklearn.SklearnFitDescriptor).X
            assert isinstance(x, dp.DataFrameDomain)
            return super().make(domain, metric, measure, d_in, d_out)

    X = pl.DataFrame({
        "a": pl.Series([1., 2.], dtype=pl.Float64),
        "b": pl.Series([3., 4.], dtype=pl.Float64),
    })
    domain = dp.sklearn.sklearn_fit_domain(
        dp.dataframe_domain([
            dp.series_domain(name, dp.atom_domain(T=dp.f64, nan=False))
            for name in ("a", "b")
        ])
    )
    fitted = NumericEstimator().fit(
        X,
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )
    assert fitted.release_ == 9


def test_standalone_fit_accepts_polars_series_target():
    pl = pytest.importorskip("polars")
    X = np.array([[1., 2.], [3., 4.]])
    x_domain = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    for T, values, dtype in (
        (dp.i32, [0, 1], pl.Int32),
        (str, ["a", "b"], pl.String),
    ):
        model = _TestEstimator()
        fitted = model.fit(
            X, pl.Series("label", values, dtype=dtype),
            domain=dp.sklearn.sklearn_fit_domain(
                x_domain, dp.series_domain("label", dp.atom_domain(T=T)),
            ),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
        )
        assert fitted is model
        assert fitted.release_ == 10


def test_public_prediction_accepts_polars_dataframe():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression()
    model._ingest_release(np.array([2., -1., 0.]))
    X = np.array([[1., 0.], [0., 2.]])
    frame = pl.DataFrame({"a": X[:, 0], "b": X[:, 1]})
    np.testing.assert_array_equal(model.predict(frame), model.predict(X))
    np.testing.assert_allclose(model.predict_proba(frame), model.predict_proba(X))


def test_public_transform_only_exists_when_implemented():
    pl = pytest.importorskip("polars")

    class Scale(_TestEstimator):
        def transform(self, X):
            return np.asarray(X) * 2

    class NoPublicTransform(_TestEstimator):
        pass

    assert not hasattr(NoPublicTransform(), "transform")
    assert not hasattr(NoPublicTransform(), "predict")
    assert not hasattr(NoPublicTransform(), "predict_proba")
    result = Scale().transform(pl.DataFrame({"x": [1., 2.]}))
    np.testing.assert_array_equal(result, [[2.], [4.]])


def test_frozen_rowwise_numpy_output_domain():
    from opendp.extras.sklearn._frozen import FrozenTransformer, _RowwiseSpec

    class ArrayPrediction(_TestEstimator):
        def _make_rowwise_method(self, method, input_domain):
            assert method == "predict"
            return _RowwiseSpec(
                lambda X: X[:, :1].copy(),
                dp.numpy.array2_domain(T=dp.f64, num_columns=1, nan=False),
            )

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False),
    )
    transform = FrozenTransformer(ArrayPrediction(), method="predict").make_transform(
        domain, dp.symmetric_distance()
    )
    data = dp.sklearn.SklearnFit(np.array([[1.0, 2.0], [3.0, 4.0]]))
    output = transform(data)
    assert isinstance(output, dp.sklearn.SklearnFit)
    np.testing.assert_array_equal(output.X, [[1.0], [3.0]])
    assert output.y is None
    assert transform.output_domain.member(output)
    assert transform.map(1) == 1


def test_logistic_rowwise_requires_public_feature_width():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression()
    model._ingest_release(np.array([2.0, -1.0, 0.0], dtype=np.float64))
    unsized = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, nan=False),
    )
    with pytest.raises(ValueError, match="feature count"):
        model.as_transformer("predict").make_transform(
            unsized, dp.symmetric_distance(),
        )


def test_arbitrary_rowwise_method_requires_opendp_estimator():
    from opendp.extras.sklearn._frozen import FrozenTransformer, _RowwiseSpec

    class Untrusted:
        def _make_rowwise_method(self, method, input_domain):
            return _RowwiseSpec(
                lambda X: X[:, :1],
                dp.numpy.array2_domain(T=dp.f64, num_columns=1),
            )

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=2),
    )
    with pytest.raises(TypeError, match="OpenDP estimator"):
        FrozenTransformer(Untrusted(), "predict").make_transform(
            domain, dp.symmetric_distance(),
        )


def test_generic_rowwise_does_not_claim_identifier_metric_stability():
    from opendp.extras.sklearn._frozen import FrozenTransformer, _RowwiseSpec

    class Rowwise(_TestEstimator):

        def _make_rowwise_method(self, method, domain):
            return _RowwiseSpec(
                lambda X: X,
                dp.numpy.array2_domain(T=dp.f64, num_columns=1),
            )

    domain = dp.sklearn.sklearn_fit_domain(
        dp.numpy.array2_domain(T=dp.f64, num_columns=1),
    )
    frozen = FrozenTransformer(Rowwise(), "predict")
    with pytest.raises(ValueError, match="row-adjacency"):
        frozen.make_transform(
            domain, dp.frame_distance(dp.symmetric_distance()),
        )


def test_frozen_predictions_do_not_merge_aligned_target_into_features():
    pl = pytest.importorskip("polars")
    from opendp.extras.sklearn.linear_model._logistic_regression import LogisticRegression

    model = LogisticRegression()
    model._ingest_release(np.array([2.0, -1.0, 0.0], dtype=np.float64))
    X = dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False)
    y = dp.vector_domain(dp.atom_domain(T=dp.i32))
    domain = dp.sklearn.sklearn_fit_domain(X, y)
    trans = model.as_transformer("predict").make_transform(
        domain, dp.symmetric_distance(),
    )
    data = dp.sklearn.SklearnFit(
        np.array([[1., 0.], [0., 1.]], dtype=np.float64),
        np.array([0, 1], dtype=np.int32),
    )
    out = trans(data)
    assert isinstance(out, dp.sklearn.SklearnFit)
    assert isinstance(out.X, pl.DataFrame)
    assert out.X.columns == ["prediction"]
    np.testing.assert_array_equal(out.y, data.y)
    assert trans.output_domain.member(out)


def test_standalone_fit_converts_explicit_series_target_independently():
    pl = pytest.importorskip("polars")
    x_domain = dp.numpy.array2_domain(T=dp.f64, num_columns=2, nan=False)
    y_domain = dp.series_domain("actual_label", dp.atom_domain(T=dp.i32))
    fit_domain = dp.sklearn.sklearn_fit_domain(x_domain, y_domain)
    model = _TestEstimator()
    model.fit(
        np.array([[1.0, 2.0], [3.0, 4.0]], dtype=np.float64),
        pl.Series("actual_label", [0, 1], dtype=pl.Int32),
        domain=fit_domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
    )
    assert model.release_ == 10

