"""Sklearn estimator wrapper for differentially private logistic regression."""

import numpy as np
from scipy.special import expit  # type: ignore[import-untyped]

from opendp._internal import _make_measurement
from opendp.extras.sklearn._domains import SklearnFitDomain
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import (
    make_private_logistic_regression as _make_private_logistic_regression,
)


class LogisticRegression(DPEstimator):
    """
    DP Logistic Regression

    The interface is parallel to that offered by sklearn's
    `LogisticRegression <https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html>`_.
    Fitting is performed with differentially private stochastic gradient descent (DP-SGD):
    each iteration clips per-example gradients to an L2 norm bound, sums them under Gaussian
    noise, and takes a gradient step, with the privacy budget split across iterations under
    zero-concentrated differential privacy. Regularization is applied as post-processing on
    the noised gradient, so it adds no privacy cost. After ``fit``, the model exposes
    ``coef_``, ``intercept_``, and ``classes_``, and supports ``predict`` and ``predict_proba``.

    Data is expected as a single 2-dimensional array whose last column is the binary target
    and whose remaining columns are the features. Pass this array as ``X`` to standalone
    ``fit`` or as ``SklearnFit(X)`` to a Context. A separate ``y`` is not supported.

    An explicit ``sklearn_fit_domain(array2_domain(..., size=n, nan=False))`` is
    required for standalone fitting and Context construction. The row count ``n``
    and the absence of NaNs must be specified publicly, not inferred from private data.

    :param n_iters: Number of DP-SGD iterations. Also controls the budget split: the total privacy budget is divided across this many gradient steps, so more iterations means more noise per step.
    :param learning_rate: Step size for the gradient update applied to the weights each iteration.
    :param clip_norm: Per-example gradient clipping bound. Each example's gradient is scaled to have L2 norm at most this value, bounding one record's influence on the update. Larger values distort gradients less but require more noise.
    :param penalty: Regularization type, following scikit-learn: ``'l2'`` (default), ``'l1'``, ``'elasticnet'``, or ``None``. The intercept is not regularized.
    :param C: Inverse of regularization strength; must be positive. Following scikit-learn, smaller values specify stronger regularization. Defaults to 1.0.
    :param l1_ratio: Elastic-net mixing parameter in [0, 1], used only when ``penalty='elasticnet'``. ``l1_ratio=1`` is equivalent to L1, ``l1_ratio=0`` to L2. Defaults to ``None``.
    """

    def __init__(
        self,
        n_iters=100,
        learning_rate=0.1,
        clip_norm=1.0,
        penalty="l2",
        C=1.0,
        l1_ratio=None,
    ):
        self.n_iters = n_iters
        self.learning_rate = learning_rate
        self.clip_norm = clip_norm
        self.penalty = penalty
        self.C = C
        self.l1_ratio = l1_ratio

    def make(self, input_domain, input_metric, output_measure, d_in, d_out):
        fit_desc = input_domain.cast(SklearnFitDomain)
        if fit_desc.y is not None:
            raise ValueError(
                "LogisticRegression expects the target in the last column of X"
            )
        measurement = _make_private_logistic_regression(
            fit_desc.X,
            input_metric,
            output_measure,
            d_in,
            d_out,
            n_iters=self.n_iters,
            learning_rate=self.learning_rate,
            clip_norm=self.clip_norm,
            penalty=self.penalty,
            C=self.C,
            l1_ratio=self.l1_ratio,
        )
        return _make_measurement(
            input_domain,
            input_metric,
            measurement.output_measure,
            lambda data: measurement(data.X),
            measurement.map,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = theta[:-1]
        self.intercept_ = theta[-1]
        self.classes_ = np.array([0, 1])

    def predict_proba(self, X):
        Xb = np.hstack([np.asarray(X), np.ones((len(X), 1))])
        p = expit(Xb @ np.concatenate([self.coef_, [self.intercept_]]))
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return (self.predict_proba(X)[:, 1] >= 0.5).astype(int)
