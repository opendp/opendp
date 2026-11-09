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

from sklearn.base import RegressorMixin

from opendp._lib import import_optional_dependency
from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)

# Keep the root logistic estimator available alongside Theil-Sen.
from opendp.extras.sklearn.linear_model._logistic_regression import LogisticRegression

__all__ = ["TheilSenRegressor", "make_private_theil_sen", "LogisticRegression"]


class TheilSenRegressor(RegressorMixin, DPEstimator):  # type: ignore
    """Differentially private Theil-Sen regression.

    Fits a line to one numeric feature column and a numeric target vector.
    With size-free component domains, ``privacy_unit=dp.unit_of(contributions=1)``
    protects one aligned row insertion or removal under symmetric distance
    (unbounded DP). No public sample count is required. An exact-size domain
    instead uses ``contributions=2`` to protect one row replacement.

    Standalone ``fit(X, y, ...)`` updates and returns this estimator. In a
    :class:`~opendp.context.Context`, ``context.query().sklearn(model).release()``
    returns a fitted clone, leaving ``model`` unchanged.

    :param x_bounds: Public feature interval, as a single pair ``((lower, upper),)``,
        used to locate the two reference points.
    :param y_bounds: Public interval ``(lower, upper)`` for candidate predictions
        at the reference points.
    :param runs: Positive number of randomized pairings. Defaults to 1.
    :param candidates_count: Number of equally spaced candidate predictions in
        ``y_bounds``; at least 2. Defaults to 100.
    :param fraction_bounds: Two increasing fractions strictly between 0 and 1
        locating reference points within ``x_bounds``. Defaults to ``(0.25, 0.75)``.

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
        ...     x_bounds=((-2., 3.),), y_bounds=(-5., 10.)
        ... )
        >>> _ = model.fit(
        ...     X, y, domain=domain,
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ... )
        >>> model.predict(np.array([[0.], [1.]])).shape
        (2,)
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
