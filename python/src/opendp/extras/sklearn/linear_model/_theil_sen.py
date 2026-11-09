"""Sklearn estimator for private matched-pair slope and median-residual regression."""

from __future__ import annotations

from sklearn.base import RegressorMixin
from sklearn.utils.validation import check_is_fitted

from opendp._lib import import_optional_dependency
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    _validate_bounds,
    make_private_theil_sen,
)


class TheilSenRegressor(RegressorMixin, DPEstimator):  # type: ignore
    """Differentially private univariate Theil–Sen regression.

    Estimates a median of sampled pairwise slopes, then a median residual
    conditional on the released slope. Unlike the classical Theil–Sen estimator,
    it does not compute all pairs.

    With unsized domains, ``contributions=1`` protects an aligned row insertion
    or removal; with fixed-size domains, ``contributions=2`` protects replacement.
    Standalone ``fit`` updates and returns this estimator; a
    :class:`~opendp.context.Context` releases a fitted clone instead.
    Both required bounds are finite, strictly increasing public candidate
    intervals, not clipping bounds on private features or targets.

    :param slope_bounds: Public candidate interval ``(lower, upper)`` for the slope.
    :param intercept_bounds: Public candidate interval ``(lower, upper)`` for the intercept.

    Example:

    .. code:: pycon

        >>> import numpy as np
        >>> import opendp.prelude as dp
        >>> dp.enable_features("contrib", "idealized-numerics", "honest-but-curious")
        >>> X = np.array([[-1.], [0.], [1.], [2.]])
        >>> y = np.array([-1., 1., 3., 5.])
        >>> model = dp.sklearn.linear_model.TheilSenRegressor(
        ...     slope_bounds=(-5., 5.), intercept_bounds=(-10., 10.)
        ... )
        >>> _ = model.fit(
        ...     X, y,
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

    def _prepare_X(self, X):
        np = import_optional_dependency("numpy")
        X = np.asarray(X)
        if X.ndim == 1:
            X = X.reshape(-1, 1)
        if X.ndim != 2 or X.shape[1] != 1:
            raise ValueError("X must have shape (n_samples, 1)")
        return X

    def predict(self, X):
        check_is_fitted(self)
        X = self._prepare_X(X)
        return X[:, 0] * self.coef_[0] + self.intercept_
