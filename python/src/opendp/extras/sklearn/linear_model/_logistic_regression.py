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

    NumPy feature and binary target arrays can be fitted without an explicit
    domain. Dtypes and feature count are assumed public; dataset size and
    finiteness are never inferred. Floating-point domains allow NaNs by default::

        model = dp.sklearn.linear_model.LogisticRegression(reference_size=1000)
        model.fit(
            X, y,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(rho=1.0),
        )

    Inferred domains are unsized, so a public ``reference_size`` is required for
    optimization scaling. It need not equal the actual dataset size. Each aligned
    feature/target pair is one record under add/remove adjacency. For Context
    fitting, supply ``SklearnFit(X, y)`` and release a fitted clone with
    ``context.query().sklearn(model).release()``.

    Explicit ``domain=sklearn_fit_domain(X_domain, y_domain)`` remains supported:
    numeric ``NPArray2Domain`` features (public feature count) and numeric or boolean
    ``NPArray1Domain`` targets, with aligned rows. A known domain size is used as
    the default normalizer and enables fixed-size substitution analysis (even
    contributions, typically 2). Feature row-norm and cardinality constraints are
    retained. No private record count is used in optimization or privacy calibration.

    Nonfinite features are zero-imputed in training and prediction. Records with
    nonfinite targets contribute zero gradient rows, including for the intercept.
    Extreme arithmetic is sanitized before gradient clipping. This handling is
    deterministic and independent across rows. Binary labels should be encoded as
    0 and 1; the target domain need not restrict their values.

    :param n_iters: Number of full-batch gradient steps. The privacy budget is split across these steps; more iterations means more noise per step.
    :param learning_rate: Step size for the gradient update applied to the weights each iteration.
    :param clip_norm: Per-example gradient clipping bound. Each example's gradient is scaled to have L2 norm at most this value, bounding one record's influence on the update. Larger values distort gradients less but require more noise.
    :param alpha: Finite, nonnegative regularization parameter for the summed loss objective. Zero disables regularization. Defaults to 1.0. The intercept is not regularized.
    :param l1_ratio: Finite elastic-net mixing parameter in [0, 1]. Zero (default) selects L2, one selects L1, and intermediate values mix both.
    :param reference_size: Public finite positive integer normalizer, required for unsized domains. Defaults to the known domain size when available. Each update multiplies the noisy gradient sum plus regularization gradient by ``learning_rate / normalizer``; this changes only the step size, not the objective or privacy calibration.
    """

    def __init__(
        self,
        n_iters=100,
        learning_rate=0.1,
        clip_norm=1.0,
        alpha=1.0,
        l1_ratio=0.0,
        reference_size: int | None = None,
    ):
        self.n_iters = n_iters
        self.learning_rate = learning_rate
        self.clip_norm = clip_norm
        self.alpha = alpha
        self.l1_ratio = l1_ratio
        self.reference_size = reference_size

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
            reference_size=self.reference_size,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release)
        self.coef_ = theta[:-1].reshape(1, -1)
        self.intercept_ = theta[-1:]
        self.classes_ = np.array([0, 1])
        self.n_features_in_ = self.coef_.shape[1]

    def predict_proba(self, X):
        X = np.nan_to_num(np.asarray(X), nan=0.0, posinf=0.0, neginf=0.0)
        with np.errstate(all="ignore"):
            z = np.nan_to_num(X @ self.coef_[0] + self.intercept_[0], nan=0.0)
        p = expit(z)
        return np.column_stack([1 - p, p])

    def predict(self, X):
        return self.classes_[(self.predict_proba(X)[:, 1] >= 0.5).astype(int)]
