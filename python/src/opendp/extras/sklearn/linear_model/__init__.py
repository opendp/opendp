"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.linear_model``.

Differentially private sklearn-style linear models.
"""

from __future__ import annotations

from opendp._lib import import_optional_dependency
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)

_sklearn_base = import_optional_dependency("sklearn.base", raise_error=False)


class _FallbackRegressorMixin:
    """Minimal fallback used when scikit-learn is not installed."""


_RegressorMixin = (
    _sklearn_base.RegressorMixin
    if _sklearn_base is not None
    else _FallbackRegressorMixin
)

__all__ = ["TheilSenRegressor", "make_private_theil_sen"]


class TheilSenRegressor(_RegressorMixin, DPEstimator):  # type: ignore
    """Differentially private Theil-Sen regression.

    The private training query contains paired rows ``[x, y]``.  Privacy
    allocation and the input domain are supplied by an OpenDP Context.
    """

    def __init__(
        self,
        *,
        x_bounds,
        y_bounds,
        runs=1,
        candidates_count=100,
        fraction_bounds=(0.25, 0.75),
    ):
        self.x_bounds = x_bounds
        self.y_bounds = y_bounds
        self.runs = runs
        self.candidates_count = candidates_count
        self.fraction_bounds = fraction_bounds

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
            x_bounds=self.x_bounds,
            y_bounds=self.y_bounds,
            runs=self.runs,
            candidates_count=self.candidates_count,
            fraction_bounds=self.fraction_bounds,
        )

    def _prepare_fit_query(self, X, y=None, **fit_params):
        if y is not None:
            raise TypeError("y must be included as the second column of the private query")
        self._reject_fit_params(fit_params)
        return X

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

    def score(self, X, y, sample_weight=None):
        if sample_weight is not None:
            raise NotImplementedError("sample_weight is not supported")
        np = import_optional_dependency("numpy")
        y = np.asarray(y)
        predictions = self.predict(X)
        if y.ndim != 1 or y.shape[0] != predictions.shape[0]:
            raise ValueError("y must be a one-dimensional array matching X")
        residual = np.sum((y - predictions) ** 2)
        centered = np.sum((y - np.mean(y)) ** 2)
        return 1.0 - residual / centered if centered else 1.0 if residual == 0 else 0.0
