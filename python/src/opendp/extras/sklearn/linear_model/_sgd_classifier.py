"""Sklearn-style interface to full-batch differentially private gradient descent."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]
from sklearn.base import BaseEstimator, ClassifierMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression as _make_private_logistic_regression,
)


class SGDClassifier(ClassifierMixin, MeasurementMixin, TransformationMixin, BaseEstimator):
    """Differentially private linear classification using noisy gradients.

    The constructor largely follows sklearn.linear_model.SGDClassifier,
    with a fixed learning rate rather than exposing a schedule selector.
    Only loss="log_loss" is implemented. The sklearn default loss="hinge"
    intentionally raises until hinge loss is supported. Optimization runs
    for exactly max_iter steps, without convergence-based early stopping.

    For example: SGDClassifier(loss="log_loss").
    Each max_iter step makes a *full-batch* Gaussian-noised
    gradient measurement; this is not yet minibatch SGD. The intercept is
    unregularized and alpha applies to the average loss (sklearn convention).
    Labels should be 0 or 1.

    shuffle, epsilon, power_t, validation_fraction, and n_iter_no_change have
    no effect for the currently supported fixed-step logistic loss. Other
    unimplemented behavior (including warm_start, class_weight, average,
    user-provided random_state, and early_stopping) is rejected.
    sklearn's epsilon is a loss-specific threshold, not a privacy budget.

    :param clip_norm: finite positive per-record L2 gradient clipping bound
    :param reference_size: public positive gradient normalizer; required for
        unsized input domains, otherwise defaults to their fixed row count
    """

    def __init__(
        self,
        loss="hinge",
        *,
        penalty="l2",
        alpha=0.0001,
        l1_ratio=0.15,
        fit_intercept=True,
        max_iter=100,
        shuffle=True,
        verbose=0,
        epsilon=0.1,
        n_jobs=None,
        random_state=None,
        eta0=0.1,
        power_t=0.5,
        early_stopping=False,
        validation_fraction=0.1,
        n_iter_no_change=5,
        class_weight=None,
        warm_start=False,
        average=False,
        clip_norm=1.0,
        reference_size: int | None = None,
    ):
        self.loss = loss
        self.penalty = penalty
        self.alpha = alpha
        self.l1_ratio = l1_ratio
        self.fit_intercept = fit_intercept
        self.max_iter = max_iter
        self.shuffle = shuffle
        self.verbose = verbose
        self.epsilon = epsilon
        self.n_jobs = n_jobs
        self.random_state = random_state
        self.eta0 = eta0
        self.power_t = power_t
        self.early_stopping = early_stopping
        self.validation_fraction = validation_fraction
        self.n_iter_no_change = n_iter_no_change
        self.class_weight = class_weight
        self.warm_start = warm_start
        self.average = average
        self.clip_norm = clip_norm
        self.reference_size = reference_size

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        # Reject unsupported sklearn controls rather than silently interpreting
        # them as implemented, or inspecting private data for convergence.
        if self.loss != "log_loss":
            raise ValueError('only loss="log_loss" is supported')
        if self.verbose != 0:
            raise ValueError("verbose is unsupported")
        if self.n_jobs is not None:
            raise ValueError("n_jobs is unsupported")
        if self.random_state is not None:
            raise ValueError("random_state is unsupported for private randomness")
        if self.early_stopping:
            raise ValueError("early_stopping is unsupported")
        if self.class_weight is not None:
            raise ValueError("class_weight is unsupported")
        if self.warm_start:
            raise ValueError("warm_start is unsupported")
        if self.average:
            raise ValueError("average is unsupported")

        return _make_private_logistic_regression(
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
        """Describe the output domain of independent per-row classification."""
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
            raise ValueError(f"Method {method!r} does not support rowwise inference")

        return array2_domain(T=dtype, num_columns=num_columns, size=x.size, nan=False)
