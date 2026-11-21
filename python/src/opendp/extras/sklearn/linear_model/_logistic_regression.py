"""Sklearn estimator wrapper for differentially private logistic regression."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]
from sklearn.base import BaseEstimator, ClassifierMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression as _make_private_logistic_regression,
)


class LogisticRegression(
    ClassifierMixin, MeasurementMixin, TransformationMixin, BaseEstimator
):
    """Differentially private binary logistic regression with sklearn's classifier interface.

    Uses full-batch noisy gradient descent with per-record L2 clipping and Gaussian noise,
    with proximal updates for L1 regularization.
    Labels should be encoded as 0 and 1.
    Unsized inputs require a public ``reference_size``.

    :param alpha: finite, nonnegative regularization strength for the summed loss;
        zero disables regularization, and the intercept is not regularized
    :param l1_ratio: finite L1 fraction in [0, 1]; zero selects L2, one selects L1
    :param fit_intercept: whether to fit an unregularized intercept (default True);
        if False, omit it from gradients and noise and set ``intercept_`` to zero
    :param max_iter: positive integer number of full-batch steps; budget is split
        across exactly this many steps, without early stopping
    :param learning_rate: finite positive step size before division by the normalizer
    :param clip_norm: finite positive per-record L2 gradient bound
    :param reference_size: public finite positive integer normalizer, defaulting to
        the known domain size. Required for unsized inputs; need not equal their
        actual size. Changes step size, not the objective or privacy calibration.
    """

    def __init__(
        self,
        *,
        alpha=1.0,
        l1_ratio=0.0,
        fit_intercept=True,
        max_iter=100,
        learning_rate=0.1,
        clip_norm=1.0,
        reference_size: int | None = None,
    ):
        self.alpha = alpha
        self.l1_ratio = l1_ratio
        self.fit_intercept = fit_intercept
        self.max_iter = max_iter
        self.learning_rate = learning_rate
        self.clip_norm = clip_norm
        self.reference_size = reference_size

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        return _make_private_logistic_regression(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            max_iter=self.max_iter,
            fit_intercept=self.fit_intercept,
            learning_rate=self.learning_rate,
            clip_norm=self.clip_norm,
            alpha=self.alpha,
            l1_ratio=self.l1_ratio,
            reference_size=self.reference_size,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = (theta[:-1] if self.fit_intercept else theta).reshape(1, -1)
        self.intercept_ = theta[-1:] if self.fit_intercept else np.zeros(1)
        self.classes_ = np.array([0, 1])
        self.n_features_in_ = self.coef_.shape[1]

    def predict_proba(self, X):
        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        X = validate_data(
            self, X, reset=False, ensure_all_finite=False, ensure_min_samples=0
        )
        X = np.nan_to_num(np.asarray(X), nan=0.0, posinf=0.0, neginf=0.0)
        with np.errstate(all="ignore"):
            z = np.nan_to_num(X @ self.coef_[0] + self.intercept_[0], nan=0.0)
        p = expit(z)
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return self.classes_[(self.predict_proba(X)[:, 1] > 0.5).astype(int)]

    def rowwise_output_domain(self, feature_domain, *, method):
        """Certify independent per-row classification and its output domain."""
        from opendp.extras.numpy import NPArray2Domain, array2_domain

        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        x = feature_domain.cast(NPArray2Domain)
        if x.num_columns != self.n_features_in_:
            raise ValueError("Prediction feature count differs from fitted model")
        if not np.array_equal(self.classes_, [0, 1]):
            raise ValueError("Only 0/1 classes are currently supported")

        if method == "predict":
            num_columns = 1
            dtype = np.int64
        elif method == "predict_proba":
            num_columns = 2
            dtype = np.float64
        else:
            raise ValueError(f"Method {method!r} is not certified rowwise")

        return array2_domain(T=dtype, num_columns=num_columns, size=x.size, nan=False)
