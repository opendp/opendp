"""Rowwise inference for fitted sklearn-compatible estimators."""

import numpy as np
import pytest

pytest.importorskip("sklearn")

import opendp.prelude as dp
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn._rowwise import _make_rowwise_transform
from opendp._internal import _make_measurement
from sklearn.base import BaseEstimator
from sklearn.preprocessing import StandardScaler
from sklearn.utils.validation import check_is_fitted

dp.enable_features("contrib")


class IdentityFitted(MeasurementMixin, TransformationMixin, BaseEstimator):
    def make_measurement(self, domain, metric, measure, d_in, d_out):
        return _make_measurement(
            domain, metric, measure, lambda data: 7, lambda d: d_out * d / d_in
        )

    def _ingest_release(self, release):
        self.release_ = release

    def transform(self, X):
        return X

    def rowwise_output_domain(self, input_domain, *, method):
        if method != "transform":
            raise ValueError(f"Unsupported method {method!r}")
        check_is_fitted(self)
        return input_domain


def domain(width=2, y=False):
    features = dp.numpy.array2_domain(T=dp.f64, num_columns=width, nan=False)
    target = dp.vector_domain(dp.atom_domain(T=dp.i32)) if y else None
    return dp.sklearn.sklearn_fit_domain(features, target)


def predict_transform(model, input_domain, method):
    return _make_rowwise_transform(
        input_domain, dp.symmetric_distance(), model, method=method
    )


def test_fitted_transform_query_needs_no_frozen_wrapper_or_budget():
    X = np.array([[1.0, 2.0], [3.0, 4.0]])
    ctx = dp.Context.compositor(
        dp.sklearn.SklearnFit(X),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain(),
        split_evenly_over=2,
    )
    fitted = ctx.query().sklearn(IdentityFitted()).release()
    assert ctx.current_privacy_loss() == [0.5]
    query = ctx.query().sklearn(fitted, method="transform")
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == [0.5]


def test_custom_rowwise_respects_hbc():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    fitted = IdentityFitted()
    fitted.release_ = 7
    trusted = LogisticRegression(classes=(0, 1))
    trusted._ingest_release(np.array([2.0, -1.0, 0.0]))

    class Override(LogisticRegression):
        def predict(self, X):
            return np.repeat(X[:1, 0], len(X))

    overridden = Override(classes=(0, 1))
    overridden._ingest_release(np.array([2.0, -1.0, 0.0]))
    from opendp.mod import OpenDPException

    cases = (
        (fitted, "transform"),
        (overridden, "predict"),
    )
    assert predict_transform(trusted, domain(), "predict").map(1) == 1
    # HBC is enabled globally for tests; unreviewed rowwise estimators are allowed.
    for model, method in cases:
        assert predict_transform(model, domain(), method).map(1) == 1

    dp.disable_features("honest-but-curious")
    try:
        # Reviewed implementations still work with the flag disabled.
        assert predict_transform(trusted, domain(), "predict").map(1) == 1
        for model, method in cases:
            with pytest.raises(OpenDPException, match="honest-but-curious"):
                predict_transform(model, domain(), method)
    finally:
        dp.enable_features("honest-but-curious")


@pytest.mark.parametrize("route", ["standalone", "context"])
def test_public_logistic_fitting_and_inference_without_hbc(route):
    from opendp.extras.sklearn.linear_model import LogisticRegression

    X = np.array([[1.0, 0.0], [0.0, 2.0]])
    y = np.array([0, 1], dtype=np.int32)
    input_domain = domain(y=True)
    privacy_unit = dp.unit_of(contributions=1)
    privacy_loss = dp.loss_of(rho=1.0)
    dp.disable_features("honest-but-curious")
    try:
        model = LogisticRegression(classes=(0, 1), max_iter=1, reference_size=2)
        if route == "standalone":
            fitted = model.fit(
                X,
                y,
                domain=input_domain,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
            )
        else:
            ctx = dp.Context.compositor(
                dp.sklearn.SklearnFit(X, y),
                domain=input_domain,
                split_evenly_over=1,
                privacy_unit=privacy_unit,
                privacy_loss=privacy_loss,
            )
            fitted = ctx.query().sklearn(model).release()
        trans = predict_transform(fitted, input_domain, "predict")
        assert trans.map(1) == 1
        result = trans(dp.sklearn.SklearnFit(X, y))
        assert trans.output_domain.member(result)
        np.testing.assert_array_equal(result.y, y)
    finally:
        dp.enable_features("honest-but-curious")


def test_arbitrary_sklearn_estimator_lacks_rowwise_support():
    fitted = StandardScaler().fit(np.ones((2, 2)))
    ctx = dp.Context.compositor(
        np.ones((2, 2)), dp.unit_of(contributions=1), dp.loss_of(epsilon=1)
    )
    with pytest.raises(TypeError, match="TransformationMixin"):
        ctx.query().sklearn(fitted, method="transform")
    with pytest.raises(TypeError, match="TransformationMixin"):
        _make_rowwise_transform(domain(), dp.symmetric_distance(), fitted)


def test_predictions_are_rowwise_with_aligned_targets():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    input_domain = domain(y=True)
    X = np.array([[1.0, 0.0], [0.0, 2.0]], dtype=np.float64)
    y = np.array([0, 1], dtype=np.int32)
    data = dp.sklearn.SklearnFit(X, y)
    for method, expected in (
        ("predict", [[1], [0]]),
        ("predict_proba", None),
    ):
        trans = predict_transform(model, input_domain, method)
        assert trans.map(1) == 1
        result = trans(data)
        if expected is not None:
            np.testing.assert_array_equal(result.X, expected)
        else:
            assert result.X.shape == (2, 2)
            np.testing.assert_allclose(result.X.sum(axis=1), [1, 1])
        np.testing.assert_array_equal(result.y, y)
        assert trans.output_domain.member(result)


def test_query_predicts_without_freezing_or_spending_privacy():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    X = np.array([[1.0, 0.0], [0.0, 2.0]], dtype=np.float64)
    ctx = dp.Context.compositor(
        dp.sklearn.SklearnFit(X),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1),
        domain=domain(),
    )
    privacy_before = ctx.current_privacy_loss()
    trans = (
        ctx.query().sklearn(model, method="predict").resolve(allow_transformations=True)
    )
    assert trans.map(1) == 1
    np.testing.assert_array_equal(trans(dp.sklearn.SklearnFit(X)).X, [[1], [0]])
    assert ctx.current_privacy_loss() == privacy_before
    with pytest.raises(TypeError, match="no sklearn 'transform' method"):
        ctx.query().sklearn(model, method="transform")


def test_rowwise_queries_chain_without_privacy_distances():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    data = dp.sklearn.SklearnFit(np.array([[1.0, 0.0], [0.0, 2.0]]))
    query = dp.Query((domain(), dp.symmetric_distance()), dp.max_divergence())
    first = query.sklearn(model, method="predict_proba")
    result = first.sklearn(model, method="predict").resolve(allow_transformations=True)
    expected = model.predict(model.predict_proba(data.X))
    assert result.map(1) == 1
    np.testing.assert_array_equal(result(data).X[:, 0], expected)


def test_empty_inference_and_sklearn_tie_breaking():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.zeros(3))
    assert model.predict(np.zeros((1, 2))).tolist() == [0]

    for method, width in (("predict", 1), ("predict_proba", 2)):
        empty = np.empty((0, 2))
        assert getattr(model, method)(empty).shape[0] == 0
        trans = predict_transform(model, domain(), method)
        result = trans(dp.sklearn.SklearnFit(empty))
        assert result.X.shape == (0, width)
        assert trans.output_domain.member(result)


def test_rowwise_transform_snapshots_coefficients():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.array([2.0, -1.0, 0.0]))
    trans = predict_transform(model, domain(), "predict")
    data = dp.sklearn.SklearnFit(np.array([[1.0, 0.0]]))
    original = trans(data).X.copy()
    model.coef_[:] = -100.0
    np.testing.assert_array_equal(trans(data).X, original)


def test_rowwise_rejects_unsupported_method_and_metric():
    class Untrusted:
        def rowwise_output_domain(self, features, *, method):
            return features

    with pytest.raises(TypeError, match="TransformationMixin"):
        _make_rowwise_transform(domain(), dp.symmetric_distance(), Untrusted())

    class Rowwise(TransformationMixin, BaseEstimator):
        def transform(self, X):
            return X

        def rowwise_output_domain(self, features, *, method):
            if method != "transform":
                raise ValueError("Unsupported")
            return features

    for metric in (
        dp.insert_delete_distance(),
        dp.change_one_distance(),
        dp.hamming_distance(),
        dp.frame_distance(dp.symmetric_distance()),
    ):
        with pytest.raises(ValueError, match="symmetric_distance"):
            _make_rowwise_transform(domain(), metric, Rowwise())


def test_invalid_rowwise_output_domain_and_missing_public_width():
    class BadDomain(TransformationMixin, BaseEstimator):
        def transform(self, X):
            return X

        def rowwise_output_domain(self, input_domain, *, method):
            return None

    with pytest.raises(TypeError, match="must return a Domain"):
        predict_transform(BadDomain(), domain(), "transform")
    with pytest.raises(TypeError, match="no sklearn 'predict' method"):
        predict_transform(BadDomain(), domain(), "predict")

    class WrongDomain(BadDomain):
        def rowwise_output_domain(self, input_domain, *, method):
            return dp.atom_domain(T=dp.f64)

    with pytest.raises(TypeError, match="Rowwise output"):
        predict_transform(WrongDomain(), domain(), "transform")

    class UnknownWidth(BadDomain):
        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64)

    with pytest.raises(ValueError, match="public output width"):
        predict_transform(UnknownWidth(), domain(), "transform")


def test_logistic_regression_rejects_invalid_inference_contracts():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.zeros(3))
    with pytest.raises(ValueError, match="feature count"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=dp.f64, num_columns=3), method="predict"
        )

    with pytest.raises(ValueError, match="numeric or boolean"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=str, num_columns=2), method="predict"
        )

    features = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    with pytest.raises(ValueError, match="does not support rowwise inference"):
        model.rowwise_output_domain(features, method="transform")


def test_rowwise_inference_packages_numpy_input_without_budget():
    fitted = IdentityFitted()
    fitted.release_ = 7
    data = np.ones((2, 2))
    query = dp.Query(
        (dp.numpy.array2_domain(T=dp.f64, num_columns=2), dp.symmetric_distance()),
        dp.max_divergence(),
    )
    trans = query.sklearn(fitted, method="transform").resolve(
        allow_transformations=True
    )
    assert trans.map(1) == 1
    np.testing.assert_array_equal(trans(data).X, data)


def test_rowwise_dataframe_output_reuses_names_and_accepts_dataframe():
    pl = pytest.importorskip("polars")

    class FrameStage(TransformationMixin, BaseEstimator):
        def __init__(self, return_frame=False):
            self.return_frame = return_frame

        def transform(self, X):
            return pl.DataFrame({"value": np.ones(len(X))})

        def predict(self, X):
            values = np.ones(len(X))
            return pl.DataFrame({"value": values}) if self.return_frame else values

        def rowwise_output_domain(self, features, *, method):
            return dp.dataframe_domain(
                [dp.series_domain("value", dp.atom_domain(T=dp.f64))]
            )

    for method, return_frame in (
        ("transform", True),
        ("predict", True),
        ("predict", False),
    ):
        trans = predict_transform(FrameStage(return_frame), domain(), method)
        features = trans.output_domain.cast(dp.sklearn.SklearnFitDescriptor).X
        assert features.columns == ["value"]
        for n in (2, 0):
            result = trans(dp.sklearn.SklearnFit(np.ones((n, 2))))
            assert isinstance(result.X, pl.DataFrame)
            assert result.X.schema == {"value": pl.Float64}
            assert result.X.shape == (n, 1)
            assert result.X["value"].to_list() == [1.0] * n


def test_trusted_rowwise_validates_frozen_and_module_identity(monkeypatch):
    """Only exact reviewed classes are trusted."""
    from sklearn.frozen import FrozenEstimator
    from opendp.extras.sklearn._rowwise import _trusted_rowwise
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )
    import sys

    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.zeros(3))
    assert _trusted_rowwise(model)
    assert _trusted_rowwise(FrozenEstimator(model))

    class UnsafeFrozen(FrozenEstimator):
        pass

    assert not _trusted_rowwise(UnsafeFrozen(model))

    module = sys.modules[LogisticRegression.__module__]
    with monkeypatch.context() as patch:
        patch.setattr(module, "LogisticRegression", object())
        assert not _trusted_rowwise(model)


def test_trusted_rowwise_recursively_checks_composites(monkeypatch):
    """Composite inference is trusted only when all its children are."""
    import sys
    from types import ModuleType
    from opendp.extras.sklearn._rowwise import _trusted_rowwise
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    def composite_class(module_name, name):
        module = ModuleType(module_name)
        cls = type(name, (), {"__module__": module_name})
        setattr(module, name, cls)
        monkeypatch.setitem(sys.modules, module_name, module)
        return cls

    Pipeline = composite_class("opendp.extras.sklearn.pipeline", "Pipeline")
    ColumnTransformer = composite_class(
        "opendp.extras.sklearn.compose", "ColumnTransformer"
    )
    model = LogisticRegression(classes=(0, 1))
    model._ingest_release(np.zeros(3))

    pipeline = Pipeline()
    pipeline.steps = [
        ("unused", None),
        ("passthrough", "passthrough"),
        ("classifier", model),
    ]
    assert _trusted_rowwise(pipeline)
    pipeline.steps = [("unsafe", IdentityFitted()), ("classifier", model)]
    assert not _trusted_rowwise(pipeline)

    columns = ColumnTransformer()
    columns.transformers = [("passthrough", "passthrough", [0])]
    assert _trusted_rowwise(columns)
    columns.transformers = [("unsafe", IdentityFitted(), [0])]
    assert not _trusted_rowwise(columns)
