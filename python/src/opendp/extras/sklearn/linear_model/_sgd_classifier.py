"""Sklearn-style interface to full-batch differentially private gradient descent."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]
from sklearn.base import BaseEstimator, ClassifierMixin
from sklearn.utils.metaestimators import available_if
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.linear_model._make_private_sgd_classifier import (
    _normalize_classes,
    make_private_sgd_classifier as _make_private_sgd_classifier,
)


def _supports_predict_proba(estimator):
    return estimator.loss == "log_loss"


class SGDClassifier(
    ClassifierMixin, MeasurementMixin, TransformationMixin, BaseEstimator
):
    """Differentially private binary classifier using noisy full-batch gradients.

    Supports ``loss="hinge"`` (default) and ``loss="log_loss"``.
    Fits for exactly ``max_iter`` iterations.
    Classes are public; records with unknown target labels contribute zero gradients.

    :param classes: Two distinct public string or discrete numeric labels, in any order.
    :param clip_norm: Per-record L2 gradient clipping bound.
    :param reference_size: Public gradient normalizer, required for unsized inputs.

    :example:

    >>> import numpy as np
    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib", "idealized-numerics")
    >>> X = np.array([[0.0], [1.0], [2.0], [3.0]])
    >>> y = np.array([0.0, 0.0, 1.0, 1.0])
    >>> model = dp.sklearn.linear_model.SGDClassifier(
    ...     classes=(0, 1), reference_size=4, max_iter=2
    ... )
    >>> _ = model.fit(
    ...     X, y, privacy_unit=dp.unit_of(contributions=1),
    ...     privacy_loss=dp.loss_of(rho=1.0),
    ... )
    >>> model.predict(X).shape
    (4,)
    """

    def __init__(
        self,
        loss="hinge",
        *,
        classes,
        penalty="l2",
        alpha=0.0001,
        l1_ratio=0.15,
        fit_intercept=True,
        max_iter=100,
        eta0=0.1,
        clip_norm=1.0,
        reference_size: int | None = None,
    ):
        self.loss = loss
        self.classes = classes
        self.penalty = penalty
        self.alpha = alpha
        self.l1_ratio = l1_ratio
        self.fit_intercept = fit_intercept
        self.max_iter = max_iter
        self.eta0 = eta0
        self.clip_norm = clip_norm
        self.reference_size = reference_size

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        return _make_private_sgd_classifier(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            max_iter=self.max_iter,
            fit_intercept=self.fit_intercept,
            loss=self.loss,
            penalty=self.penalty,
            eta0=self.eta0,
            clip_norm=self.clip_norm,
            alpha=self.alpha,
            l1_ratio=self.l1_ratio,
            reference_size=self.reference_size,
            classes=self.classes,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = (theta[:-1] if self.fit_intercept else theta).reshape(1, -1)
        self.intercept_ = theta[-1:] if self.fit_intercept else np.zeros(1)
        self.classes_ = _normalize_classes(self.classes)
        self.n_features_in_ = self.coef_.shape[1]

    def decision_function(self, X):
        """Return signed distances to the decision boundary."""
        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        X = validate_data(
            self, X, reset=False, ensure_all_finite=False, ensure_min_samples=0
        )
        X = np.nan_to_num(np.asarray(X), nan=0.0, posinf=0.0, neginf=0.0)
        with np.errstate(all="ignore"):
            return np.nan_to_num(X @ self.coef_[0] + self.intercept_[0], nan=0.0)

    @available_if(_supports_predict_proba)
    def predict_proba(self, X):
        """Return logistic probabilities (only for loss="log_loss")."""
        p = expit(self.decision_function(X))
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return self.classes_[(self.decision_function(X) > 0).astype(int)]

    def rowwise_output_domain(self, feature_domain, *, method):
        """Describe the output domain of independent per-row classification."""
        from opendp.extras.numpy import NPArray2Domain, array2_domain
        from opendp.typing import _NUMERIC_TYPES

        check_is_fitted(self, ["coef_", "intercept_", "classes_"])
        x = feature_domain.cast(NPArray2Domain)
        if x.num_columns != self.n_features_in_:
            raise ValueError("Prediction feature count differs from fitted model")
        if x.T not in _NUMERIC_TYPES | {"bool"}:
            raise ValueError("Prediction requires numeric or boolean features")
        dtype: type[np.generic]
        if method == "predict":
            num_columns = 1
            dtype = self.classes_.dtype.type
        elif method == "decision_function":
            num_columns = 1
            dtype = np.float64
        elif method == "predict_proba" and self.loss == "log_loss":
            num_columns = 2
            dtype = np.float64
        else:
            raise ValueError(f"Method {method!r} does not support rowwise inference")

        return array2_domain(T=dtype, num_columns=num_columns, size=x.size, nan=False)
