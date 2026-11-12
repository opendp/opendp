"""Private Theil-Sen regressor."""

from __future__ import annotations

import numpy as np
from sklearn.base import RegressorMixin
from sklearn.utils.validation import check_is_fitted, validate_data

from opendp.extras.sklearn._estimator import DPEstimator
from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)


class TheilSenRegressor(RegressorMixin, DPEstimator):  # type: ignore
    """Private univariate Theil-Sen regression.

    :param slope_bounds: Public interval for slope candidates.
    :param intercept_bounds: Public interval for intercept candidates.
    """

    def __init__(self, *, slope_bounds, intercept_bounds):
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
        slope, intercept = release
        self.coef_ = np.asarray([slope])
        self.intercept_ = float(intercept)
        self.n_features_in_ = 1

    def predict(self, X):
        check_is_fitted(self)
        X = validate_data(self, X, reset=False)
        return X[:, 0] * self.coef_[0] + self.intercept_
