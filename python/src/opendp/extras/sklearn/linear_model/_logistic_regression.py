"""Sklearn estimator wrapper for differentially private logistic regression."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]
from sklearn.base import ClassifierMixin

from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression as _make_private_logistic_regression,
)


class LogisticRegression(ClassifierMixin, DPEstimator):
    """
    DP Logistic Regression

    The interface is parallel to that offered by sklearn's
    `LogisticRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html>`_.
    Fitting uses full-batch noisy gradient descent. Each iteration computes gradients
    for every record, clips their L2 norms, privately sums them with Gaussian noise,
    and updates the weights with regularization. Privacy loss is composed across
    iterations under zero-concentrated differential privacy. Regularization is
    postprocessing and adds no privacy cost. After ``fit``, the classifier exposes
    ``coef_`` of shape ``(1, n_features)``, ``intercept_`` of shape ``(1,)``,
    ``classes_=[0, 1]``, and ``n_features_in_``.

    Fit separate features and binary targets with
    ``fit(X, y, privacy_unit=..., privacy_loss=..., domain=...)``. For Context
    fitting, supply ``SklearnFit(X, y)`` and release a fitted clone with
    ``context.query().sklearn(model).release()``.

    Specify ``domain=sklearn_fit_domain(X_domain, y_domain)`` explicitly.
    ``X_domain`` must be a numeric ``NPArray2Domain`` with a public feature count,
    a known positive row count, and ``nan=False``. Feature row-norm and cardinality
    constraints are supported and retained on the input domain. ``y_domain`` must
    be a numeric or boolean ``NPArray1Domain`` excluding NaNs and aligned with the
    feature rows. Binary labels are expected to be encoded as 0 and 1; the domain
    need not restrict their values. Dataset size and necessary numerical assumptions
    must be public, not inferred from private data. For example::

        domain = dp.sklearn.sklearn_fit_domain(
            dp.numpy.array2_domain(T=float, size=100, num_columns=3, nan=False),
            dp.numpy.array1_domain(T=float, size=100, nan=False),
        )

    :param n_iters: Number of full-batch gradient steps. The privacy budget is split across these steps; more iterations means more noise per step.
    :param learning_rate: Step size for the gradient update applied to the weights each iteration.
    :param clip_norm: Per-example gradient clipping bound. Each example's gradient is scaled to have L2 norm at most this value, bounding one record's influence on the update. Larger values distort gradients less but require more noise.
    :param alpha: Finite, nonnegative regularization parameter, with strength ``alpha / n``. Zero disables regularization. Defaults to 1.0. The intercept is not regularized.
    :param l1_ratio: Finite elastic-net mixing parameter in [0, 1]. Zero (default) selects L2, one selects L1, and intermediate values mix both.
    """

    def __init__(
        self,
        n_iters=100,
        learning_rate=0.1,
        clip_norm=1.0,
        alpha=1.0,
        l1_ratio=0.0,
    ):
        self.n_iters = n_iters
        self.learning_rate = learning_rate
        self.clip_norm = clip_norm
        self.alpha = alpha
        self.l1_ratio = l1_ratio

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        return _make_private_logistic_regression(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            n_iters=self.n_iters,
            learning_rate=self.learning_rate,
            clip_norm=self.clip_norm,
            alpha=self.alpha,
            l1_ratio=self.l1_ratio,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = theta[:-1].reshape(1, -1)
        self.intercept_ = theta[-1:]
        self.classes_ = np.array([0, 1])
        self.n_features_in_ = self.coef_.shape[1]

    def predict_proba(self, X):
        p = expit(np.asarray(X) @ self.coef_[0] + self.intercept_[0])
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return self.classes_[(self.predict_proba(X)[:, 1] >= 0.5).astype(int)]
