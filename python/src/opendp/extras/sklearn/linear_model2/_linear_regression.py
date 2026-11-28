"""Differentially private least-squares regression via private sufficient statistics."""

from __future__ import annotations

import numpy as np
from sklearn.base import BaseEstimator, RegressorMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import MeasurementMixin, TransformationMixin
from opendp.extras.sklearn.linear_model2._make_private_linear_regression import (
    make_private_linear_regression,
)


class LinearRegression(
    RegressorMixin, MeasurementMixin, TransformationMixin, BaseEstimator
):
    """Differentially private least-squares regression.

    Clips feature rows and targets to public bounds, then privately releases
    sufficient statistics using Gaussian (zCDP) or Laplace (pure-DP) noise.
    Use ``privacy_unit=dp.unit_of(contributions=2)`` to protect one row
    replacement.

    :param fit_intercept: Whether to fit an intercept (default True).
        The intercept is included in spectral regularization.
    :param clip_norm: Public positive L2 clipping radius for feature rows.
    :param y_bound: Public positive absolute clipping bound for targets.
    :param failure_probability: Tail probability for spectral regularization
        (default 0.05), not the differential privacy delta parameter.

    :ivar coef_: Privately fitted coefficients, shape ``(n_features,)``.
    :ivar intercept_: Fitted intercept, or 0.0 when ``fit_intercept=False``.
    :ivar n_features_in_: Number of fitted feature columns.

    Only single-target dense inputs are supported. Inherited ``score`` is
    ordinary R-squared evaluation, not a private measurement.

    Example:

    .. code:: pycon

        >>> import numpy as np
        >>> import opendp.prelude as dp
        >>> dp.enable_features("contrib", "idealized-numerics")
        >>> X = np.array([[-1.0], [0.0], [1.0]])
        >>> y = np.array([-1.0, 0.0, 1.0])
        >>> model = dp.sklearn.linear_model2.LinearRegression(
        ...     clip_norm=1.0, y_bound=1.0
        ... )
        >>> _ = model.fit(
        ...     X, y,
        ...     privacy_unit=dp.unit_of(contributions=2),
        ...     privacy_loss=dp.loss_of(rho=1.0),
        ... )
        >>> model.predict(np.zeros((2, 1))).shape
        (2,)

    Reference: Aryanfard et al., `On Differentially Private Mechanisms
    for Linear Regression <https://arxiv.org/abs/2611.08737>`_, arXiv, 2026.
    This variant does not separately release a private minimum eigenvalue.
    """

    def __init__(
        self,
        *,
        fit_intercept=True,
        clip_norm=1.0,
        y_bound=1.0,
        failure_probability=0.05,
    ):
        self.fit_intercept = fit_intercept
        self.clip_norm = clip_norm
        self.y_bound = y_bound
        self.failure_probability = failure_probability

    def make_measurement(self, input_domain, input_metric, output_measure, d_in, d_out):
        return make_private_linear_regression(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            fit_intercept=self.fit_intercept,
            clip_norm=self.clip_norm,
            y_bound=self.y_bound,
            failure_probability=self.failure_probability,
        )

    def _ingest_release(self, release):
        theta = np.asarray(release, dtype=float)
        self.coef_ = theta[:-1].copy() if self.fit_intercept else theta.copy()
        self.intercept_ = float(theta[-1]) if self.fit_intercept else 0.0
        self.n_features_in_ = len(self.coef_)

    def predict(self, X):
        """Predict one value per row, using sklearn's standard output shape.

        Nonfinite features are treated as zero for data-independent rowwise
        inference; this is intentionally different from sklearn's default.
        """
        check_is_fitted(self, ["coef_", "intercept_"])
        X = validate_data(
            self, X, reset=False, ensure_all_finite=False, ensure_min_samples=0
        )
        X = np.nan_to_num(np.asarray(X, dtype=float), nan=0.0, posinf=0.0, neginf=0.0)
        with np.errstate(all="ignore"):
            scores = X @ self.coef_ + self.intercept_
        return np.nan_to_num(scores)

    def rowwise_output_domain(self, feature_domain, *, method):
        """Certify one finite prediction per input row for ``predict`` only."""
        from opendp.extras.numpy import NPArray2Domain, array2_domain
        from opendp.typing import _NUMERIC_TYPES

        check_is_fitted(self, ["coef_", "intercept_"])
        if method != "predict":
            raise ValueError(f"Method {method!r} does not support rowwise inference")
        desc = feature_domain.cast(NPArray2Domain)
        if desc.num_columns != self.n_features_in_ or desc.T not in _NUMERIC_TYPES:
            raise ValueError(
                "Prediction requires numeric features with fitted column count"
            )
        return array2_domain(T=float, num_columns=1, size=desc.size, nan=False)
