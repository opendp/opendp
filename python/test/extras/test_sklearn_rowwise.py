"""Rowwise inference for fitted sklearn-compatible estimators."""

import numpy as np
import pytest

pytest.importorskip("sklearn")

import opendp.prelude as dp
from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn._rowwise import _make_rowwise_transform
from sklearn.base import BaseEstimator
from sklearn.preprocessing import StandardScaler
from sklearn.utils.validation import check_is_fitted

dp.enable_features("contrib")


@pytest.fixture(autouse=True)
def allow_test_rowwise_callbacks():
    # Test doubles assert their own rowwise contracts instead of using library code.
    from opendp.mod import GLOBAL_FEATURES

    enabled = "honest-but-curious" in GLOBAL_FEATURES
    dp.enable_features("honest-but-curious")
    yield
    if not enabled:
        dp.disable_features("honest-but-curious")


class IdentityFitted(MeasurementMixin, TransformationMixin, BaseEstimator):
    def make_measurement(self, domain, metric, measure, d_in, d_out):
        return dp.m.make_user_measurement(
            domain,
            metric,
            measure,
            lambda data: 7,
            lambda d: d_out * d / d_in,
            TO=int,
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


def test_custom_rowwise_requires_honest_but_curious():
    from opendp.mod import OpenDPException
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    fitted = IdentityFitted()
    fitted.release_ = 7
    trusted = LogisticRegression()
    trusted._ingest_release(np.array([2.0, -1.0, 0.0]))

    class Override(LogisticRegression):
        def predict(self, X):
            return np.repeat(X[:1, 0], len(X))

    overridden = Override()
    overridden._ingest_release(np.array([2.0, -1.0, 0.0]))
    patched = LogisticRegression()
    patched._ingest_release(np.array([2.0, -1.0, 0.0]))
    patched.predict_proba = lambda X: np.repeat(X[:1, :1], 2, axis=0)

    dp.disable_features("honest-but-curious")
    try:
        assert predict_transform(trusted, domain(), "predict").map(1) == 1
        for model, method in (
            (fitted, "transform"),
            (overridden, "predict"),
            (patched, "predict"),
        ):
            with pytest.raises(OpenDPException, match="honest-but-curious"):
                predict_transform(model, domain(), method)
        dp.enable_features("honest-but-curious")
        assert predict_transform(fitted, domain(), "transform").map(1) == 1
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

    model = LogisticRegression()
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

    model = LogisticRegression()
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


def test_empty_inference_and_sklearn_tie_breaking():
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

    model = LogisticRegression()
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

    model = LogisticRegression()
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

    class UnknownWidth(BadDomain):
        def rowwise_output_domain(self, input_domain, *, method):
            return dp.numpy.array2_domain(T=dp.f64)

    with pytest.raises(ValueError, match="public output width"):
        predict_transform(UnknownWidth(), domain(), "transform")


def test_logistic_regression_rejects_invalid_inference_contracts():
    from opendp.extras.sklearn.linear_model import LogisticRegression

    model = LogisticRegression()
    model._ingest_release(np.zeros(3))
    with pytest.raises(ValueError, match="feature count"):
        model.rowwise_output_domain(
            dp.numpy.array2_domain(T=dp.f64, num_columns=3), method="predict"
        )

    features = dp.numpy.array2_domain(T=dp.f64, num_columns=2)
    model.classes_ = np.array([1, 2])
    with pytest.raises(ValueError, match="0/1 classes"):
        model.rowwise_output_domain(features, method="predict")

    model.classes_ = np.array([0, 1])
    with pytest.raises(ValueError, match="does not support rowwise inference"):
        model.rowwise_output_domain(features, method="transform")


def test_rowwise_inference_requires_aligned_input_domain():
    ctx = dp.Context.compositor(
        np.ones((2, 2)), dp.unit_of(contributions=1), dp.loss_of(epsilon=1)
    )
    with pytest.raises(ValueError):
        ctx.query().sklearn(IdentityFitted(), method="transform")
