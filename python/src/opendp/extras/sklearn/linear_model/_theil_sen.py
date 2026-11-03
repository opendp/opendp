"""Sklearn estimator for private matched-pair slope and median-residual regression."""

from __future__ import annotations

from sklearn.base import RegressorMixin

from opendp._lib import import_optional_dependency
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    _validate_bounds,
    make_private_theil_sen,
)


class TheilSenRegressor(RegressorMixin, DPEstimator):  # type: ignore
    """Differentially private, Theil-Sen-inspired regression.

    Fits one numeric feature and target by privately estimating the median slope
    of one secure random matching, then the median residual conditional on that
    slope. This approximates matched-pair medians, not the classical all-pairs
    Theil-Sen estimator. Each stage receives half the privacy budget.
    With size-free component domains, ``privacy_unit=dp.unit_of(contributions=1)``
    protects one aligned row insertion or removal under symmetric distance
    (unbounded DP). No public sample count is required. An exact-size domain
    instead uses ``contributions=2`` to protect one row replacement.

    Standalone ``fit(X, y, ...)`` updates and returns this estimator. In a
    :class:`~opendp.context.Context`, ``context.query().sklearn(model).release()``
    returns a fitted clone, leaving ``model`` unchanged.

    Both coefficient intervals are required public, finite, strictly increasing
    pairs. They specify candidate intervals, not clipping constraints on private
    features or targets. Nonfinite slopes are discarded; nonfinite residuals are
    replaced per row by finite interval endpoints, without dropping rows.

    :param slope_bounds: Public candidate interval ``(lower, upper)`` for the slope.
    :param intercept_bounds: Public candidate interval ``(lower, upper)`` for the intercept.

    Example:

    .. code:: pycon

        >>> import numpy as np
        >>> import opendp.prelude as dp
        >>> dp.enable_features("contrib", "idealized-numerics", "honest-but-curious")
        >>> X = np.array([[-1.], [0.], [1.], [2.]])
        >>> y = np.array([-1., 1., 3., 5.])
        >>> domain = dp.sklearn.sklearn_fit_domain(
        ...     X=dp.numpy.array2_domain(T=float, num_columns=1),
        ...     y=dp.vector_domain(dp.atom_domain(T=float)),
        ... )
        >>> model = dp.sklearn.linear_model.TheilSenRegressor(
        ...     slope_bounds=(-5., 5.), intercept_bounds=(-10., 10.)
        ... )
        >>> _ = model.fit(
        ...     X, y, domain=domain,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ... )
        >>> model.predict(np.array([[0.], [1.]])).shape
        (2,)
    """

    def __init__(self, *, slope_bounds, intercept_bounds):
        _validate_bounds(slope_bounds, "slope_bounds")
        _validate_bounds(intercept_bounds, "intercept_bounds")
        self.slope_bounds = slope_bounds
        self.intercept_bounds = intercept_bounds

    def make(
        self,
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_out,
    ):
        return make_private_theil_sen(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_out,
            slope_bounds=self.slope_bounds,
            intercept_bounds=self.intercept_bounds,
        )

    def _ingest_release(self, release):
        np = import_optional_dependency("numpy")
        slope, intercept = release
        self.coef_ = np.asarray([slope])
        self.intercept_ = float(intercept)
        self.n_features_in_ = 1

    def _check_is_fitted(self):
        if not hasattr(self, "coef_"):
            raise ValueError("TheilSenRegressor instance is not fitted yet")

    def _prepare_X(self, X):
        np = import_optional_dependency("numpy")
        X = np.asarray(X)
        if X.ndim == 1:
            X = X.reshape(-1, 1)
        if X.ndim != 2 or X.shape[1] != 1:
            raise ValueError("X must have shape (n_samples, 1)")
        return X

    def predict(self, X):
        self._check_is_fitted()
        X = self._prepare_X(X)
        return X[:, 0] * self.coef_[0] + self.intercept_
